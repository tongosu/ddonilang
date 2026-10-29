#!/usr/bin/env node

import { evaluateLiveTraceReactionCommit } from "../solutions/seamgrim_ui_mvp/ui/live_trace_reaction_runtime.js";

const OK = "seamgrim_live_trace_reaction_vectors: ok";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function baseTraceReactions() {
  return {
    schema: "seamgrim.sealed_trace_reactions.v1",
    lesson_id: "vector",
    mode: "sealed_observation_trace",
    sealed: true,
    state_hash_participation: false,
    trace_hash: "candidate:test:rules",
    predicate_contract: {
      source: "sealed_observation_trace_evaluator",
      condition_evaluation_claim: true,
      trace_rules: { epsilon: 0.001, tolerance: 0.005 },
    },
    predicates: [
      { predicate_id: "edge.theta_zero_crossing", kind: "edge", condition_kind: "zero_crossing" },
      { predicate_id: "edge.theta_extremum", kind: "edge", condition_kind: "extremum" },
    ],
    condition_rules: [
      { rule_id: "rule.theta.zero", condition_kind: "zero_crossing", predicate_id: "edge.theta_zero_crossing", target_key: "theta", epsilon: 0.001 },
      { rule_id: "rule.theta.extremum", condition_kind: "extremum", predicate_id: "edge.theta_extremum", target_key: "theta", epsilon: 0.001 },
    ],
    trace_rows: [],
    actions: [
      { action_id: "highlight.theta", kind: "highlight", when: "edge.theta_zero_crossing", priority: 10, targets: { shape_ids: ["theta"] }, style: { stroke: "#f97316" } },
      { action_id: "focus.bob", kind: "camera_focus", when: "edge.theta_extremum", priority: 10, target_anchor: "obs.bob" },
    ],
  };
}

function observation(tick, theta, omega = 0) {
  return {
    values: { tick_id: tick, t: tick * 0.1, theta, omega },
    all_values: { tick_id: tick, t: tick * 0.1, theta, omega },
    tick_id: tick,
  };
}

function apply(session, obs, overrides = {}) {
  return evaluateLiveTraceReactionCommit({
    session,
    traceReactions: baseTraceReactions(),
    observation: obs,
    views: {
      space2d: {
        schema: "seamgrim.space2d.v0",
        shapes: [{ id: "theta", kind: "line", x1: 0, y1: 0, x2: 1, y2: 1 }],
      },
    },
    viewPlanRuntime: {
      views: {
        view_plan: {
          anchors: [{ anchor_id: "obs.bob", value: { x: 0.5, y: -0.5 } }],
        },
      },
    },
    lineageId: overrides.lineageId ?? "lineage:a",
    runId: overrides.runId ?? "run:a",
    rulesetRef: "candidate:test:rules",
    lessonId: "vector",
  });
}

let step = apply(null, observation(0, -1));
assert(step.report.live_online_claim === true, "live claim missing");
assert(step.report.once_per_tick === true, "once-per-tick flag missing");
assert(step.report.lineage_reset === true, "first lineage should reset");
assert(step.report.lineage_reset_count === 1, "first lineage reset count mismatch");
assert(step.report.live_sample_count === 1, "first sample count mismatch");
assert(step.report.state_hash_participation === false, "live reaction must stay outside state_hash");

step = apply(step.session, observation(1, 1));
assert(step.report.resolved_scene_trace.selected_actions.includes("highlight.theta"), `zero-crossing action missing: ${JSON.stringify(step.report)}`);
assert(step.report.repeated_emission === false, "first emission should not be repeated");
assert(step.report.emissions.some((row) => row.tick_id === 1 && row.selected_actions.includes("highlight.theta")), "first emission history missing");
assert(step.report.fired_ticks.includes(1), "first fired tick missing");

const duplicate = apply(step.session, observation(1, 1));
assert(duplicate.report.duplicate_delivery === true, "same payload duplicate not recognized");
assert(duplicate.report.repeated_emission === true, "same payload duplicate should not emit again");
assert(duplicate.report.emissions.length === step.report.emissions.length, "duplicate delivery added an emission");
assert(!duplicate.report.diagnostics.some((row) => row.code === "E_VIEW_LIVE_DUPLICATE_TICK_MISMATCH"), "same payload duplicate diagnosed as mismatch");

const mismatch = apply(duplicate.session, observation(1, 0.5));
assert(mismatch.report.duplicate_mismatch === true, "different payload duplicate not recognized");
assert(mismatch.report.diagnostics.some((row) => row.code === "E_VIEW_LIVE_DUPLICATE_TICK_MISMATCH"), "mismatch diagnostic missing");
assert(mismatch.report.diagnostics.some((row) => row.what && row.where && row.fix), "mismatch diagnostic lacks 3-layer message");

const gap = apply(mismatch.session, observation(4, -1));
assert(gap.report.diagnostics.some((row) => row.code === "W_VIEW_LIVE_TICK_GAP"), "tick gap warning missing");

const outOfOrder = apply(gap.session, observation(3, -0.5));
assert(outOfOrder.report.diagnostics.some((row) => row.code === "E_VIEW_LIVE_OUT_OF_ORDER"), "out-of-order diagnostic missing");

const changed = apply(outOfOrder.session, observation(0, 2), { lineageId: "lineage:b", runId: "run:b" });
assert(changed.report.lineage_reset === true, "lineage change did not reset");
assert(changed.report.live_sample_count === 1, "lineage reset did not clear samples");
assert(changed.report.lineage_id === "lineage:b", "lineage id did not update");
assert(changed.report.lineage_reset_count > step.report.lineage_reset_count, "lineage reset count did not increase");

process.stdout.write(`${OK}\n`);
