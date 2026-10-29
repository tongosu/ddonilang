#!/usr/bin/env node

import {
  buildSealedObservationTraceArtifact,
  evaluateSealedTracePredicates,
} from "../solutions/seamgrim_ui_mvp/ui/sealed_trace_evaluator.js";

const OK = "seamgrim_trace_predicate_evaluator_vectors: ok";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function trace(rows, rules = null) {
  return buildSealedObservationTraceArtifact({
    lessonId: "vector",
    timelineSamples: rows.map((row, index) => ({ index, values: row })),
    rules,
  });
}

function evaluate(rows, conditionRules) {
  return evaluateSealedTracePredicates({
    observationTrace: trace(rows),
    conditionRules,
  });
}

function evaluateRaw(samples, conditionRules) {
  return evaluateSealedTracePredicates({
    observationTrace: {
      schema: "seamgrim.sealed_observation_trace.v1",
      trace_hash_candidate: "candidate:test:raw",
      samples,
    },
    conditionRules,
  });
}

function rule(kind, overrides = {}) {
  const defaults = {
    zero_crossing: { rule_id: "rule.theta.zero", predicate_id: "edge.theta_zero_crossing", target_key: "theta", epsilon: 0.001 },
    sign_change: { rule_id: "rule.omega.sign", predicate_id: "edge.omega_sign_change", target_key: "omega", epsilon: 0.001 },
    extremum: { rule_id: "rule.theta.extremum", predicate_id: "edge.theta_extremum", target_key: "theta", epsilon: 0.001 },
    tolerance_entry: { rule_id: "rule.theta.tolerance", predicate_id: "level.theta_tolerance_entry", target_key: "theta", target: 0, tolerance: 0.05, epsilon: 0.001 },
  }[kind];
  return { ...defaults, condition_kind: kind, ...overrides };
}

function baseRow(tick, theta, omega = 0) {
  return { tick_id: tick, t: tick * 0.1, theta, omega };
}

function eventTicks(result, kind) {
  return result.events
    .filter((row) => row.condition_kind === kind)
    .map((row) => row.tick_id);
}

function requireDiag(result, code) {
  assert(result.diagnostics.some((row) => row.code === code), `diagnostic missing: ${code}`);
}

{
  const result = evaluate([baseRow(0, -1), baseRow(1, 0), baseRow(2, 1)], [rule("zero_crossing")]);
  assert(JSON.stringify(eventTicks(result, "zero_crossing")) === "[1]", `exact zero crossing mismatch: ${JSON.stringify(result.events)}`);
}

{
  const result = evaluate([baseRow(0, -1), baseRow(1, 0.0005), baseRow(2, -0.0004), baseRow(3, 1)], [rule("zero_crossing")]);
  assert(JSON.stringify(eventTicks(result, "zero_crossing")) === "[1]", `zero plateau crossing mismatch: ${JSON.stringify(result.events)}`);
}

{
  const result = evaluate([baseRow(0, 1), baseRow(1, -1)], [rule("zero_crossing")]);
  assert(JSON.stringify(eventTicks(result, "zero_crossing")) === "[1]", `between-sample crossing mismatch: ${JSON.stringify(result.events)}`);
}

{
  const result = evaluate([baseRow(0, 1), baseRow(1, 2), baseRow(2, 3)], [rule("zero_crossing")]);
  assert(result.events.length === 0, `zero-crossing negative vector should not emit: ${JSON.stringify(result.events)}`);
}

{
  const rows = [baseRow(0, 0, -1), baseRow(1, 0, 0), baseRow(2, 0, 0.0002), baseRow(3, 0, 1)];
  const result = evaluate(rows, [rule("sign_change")]);
  assert(JSON.stringify(eventTicks(result, "sign_change")) === "[1]", `omega sign-change plateau mismatch: ${JSON.stringify(result.events)}`);
}

{
  const result = evaluate([baseRow(0, 0), baseRow(1, 1), baseRow(2, 0), baseRow(3, -1), baseRow(4, 0)], [rule("extremum")]);
  assert(JSON.stringify(eventTicks(result, "extremum")) === "[1,3]", `extremum min/max mismatch: ${JSON.stringify(result.events)}`);
  assert(result.events[0].detail.extremum_kind === "max", "first extremum must be max");
  assert(result.events[1].detail.extremum_kind === "min", "second extremum must be min");
}

{
  const result = evaluate([baseRow(0, 0), baseRow(1, 1), baseRow(2, 1.0002), baseRow(3, 0)], [rule("extremum")]);
  assert(JSON.stringify(eventTicks(result, "extremum")) === "[1]", `flat plateau extremum mismatch: ${JSON.stringify(result.events)}`);
  assert(result.events[0].detail.plateau_start_tick === 1 && result.events[0].detail.plateau_end_tick === 2, "plateau tick range missing");
}

{
  const rows = [baseRow(0, 0.2), baseRow(1, 0.04), baseRow(2, 0.03), baseRow(3, 0.2), baseRow(4, 0.01)];
  const result = evaluate(rows, [rule("tolerance_entry")]);
  assert(JSON.stringify(eventTicks(result, "tolerance_entry")) === "[1,4]", `tolerance re-entry mismatch: ${JSON.stringify(result.events)}`);
  assert(result.level_states.filter((row) => row.inside).length === 3, "inside level state count mismatch");
}

{
  const result = evaluate([baseRow(0, 0.01), baseRow(1, 0.02), baseRow(2, 0.2)], [rule("tolerance_entry")]);
  assert(eventTicks(result, "tolerance_entry").length === 0, "initial inside should not emit entry");
}

{
  const duplicate = evaluateRaw([baseRow(0, 0), baseRow(0, 1)], [rule("zero_crossing")]);
  requireDiag(duplicate, "E_TRACE_SAMPLE_DUPLICATE_TICK");
  const reversed = evaluateRaw([baseRow(2, 0), baseRow(1, 1)], [rule("zero_crossing")]);
  requireDiag(reversed, "E_TRACE_SAMPLE_TICK_ORDER");
  const missing = evaluateRaw([{ tick_id: 0, t: 0, theta: 1 }], [rule("zero_crossing")]);
  requireDiag(missing, "E_TRACE_SAMPLE_MISSING_VALUE");
  const nonFinite = evaluateRaw([{ tick_id: 0, t: 0, theta: Number.NaN, omega: 0 }], [rule("zero_crossing")]);
  requireDiag(nonFinite, "E_TRACE_SAMPLE_NONFINITE");
}

{
  const rows = [baseRow(0, 1, -1), baseRow(1, 0, 0), baseRow(2, -1, 1)];
  const rules = [rule("zero_crossing"), rule("sign_change")];
  const left = evaluate(rows, rules);
  const right = evaluate(rows, rules);
  assert(JSON.stringify(left) === JSON.stringify(right), "evaluation must be byte-stable for same trace and rules");
  assert(String(left.evaluation_hash_candidate ?? "").startsWith("candidate:fnv1a32:trace_predicate_events:"), "evaluation hash candidate missing");
}

process.stdout.write(`${OK}\n`);
