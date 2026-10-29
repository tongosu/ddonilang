#!/usr/bin/env node

import { buildSealedObservationTraceArtifact } from "../solutions/seamgrim_ui_mvp/ui/sealed_trace_evaluator.js";
import { buildSeamgrimTraceReactionRuntime } from "../solutions/seamgrim_ui_mvp/ui/trace_reaction_runtime.js";
import { evaluateLiveTraceReactionCommit } from "../solutions/seamgrim_ui_mvp/ui/live_trace_reaction_runtime.js";

const OK = "seamgrim_offline_live_reaction_parity: ok";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function stableCanonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map((item) => stableCanonicalJson(item)).join(",")}]`;
  if (isObject(value)) {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableCanonicalJson(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function baseTraceReactions(actionRows = null) {
  const actions = actionRows ?? [
    {
      action_id: "highlight.theta",
      kind: "highlight",
      when: "edge.theta_zero_crossing",
      priority: 10,
      targets: { group_ids: ["measure.angle.theta"] },
      style: { stroke: "#f97316", color: "#f97316", width_add: 1.2, opacity: 1 },
    },
    {
      action_id: "focus.bob",
      kind: "camera_focus",
      when: "edge.theta_extremum",
      priority: 10,
      target_anchor: "obs.bob",
      span: { x: 1.2, y: 1.2 },
      style: { stroke: "#22c55e", color: "#22c55e", opacity: 0.8 },
    },
    {
      action_id: "highlight.omega",
      kind: "highlight",
      when: "edge.omega_sign_change",
      priority: 10,
      targets: { roles: ["pendulum_bob"] },
      style: { stroke: "#14b8a6", color: "#14b8a6", width_add: 1, opacity: 1 },
    },
    {
      action_id: "dim.baseline",
      kind: "visibility",
      when: "level.theta_tolerance_entry",
      priority: 10,
      targets: { shape_ids: ["measure.angle.theta.baseline"] },
      visible: false,
      opacity: 0.18,
    },
  ];
  return {
    schema: "seamgrim.sealed_trace_reactions.v1",
    lesson_id: "offline_live_parity",
    mode: "sealed_observation_trace",
    sealed: true,
    live_online_claim: false,
    runtime_semantics_claim: false,
    state_hash_participation: false,
    trace_hash: "candidate:test:offline-live-parity",
    predicate_contract: {
      source: "sealed_observation_trace_evaluator",
      condition_evaluation_claim: true,
      trace_rules: { epsilon: 0.001, tolerance_target: 0, tolerance: 0.005 },
      conflict_diagnostic: "E_TRACE_REACTION_CONFLICT",
      view_conflict_diagnostic: "E_VIEW_REACTION_CONFLICT",
    },
    predicates: [
      { predicate_id: "edge.theta_zero_crossing", kind: "edge", condition_kind: "zero_crossing" },
      { predicate_id: "edge.theta_extremum", kind: "edge", condition_kind: "extremum" },
      { predicate_id: "edge.omega_sign_change", kind: "edge", condition_kind: "sign_change" },
      { predicate_id: "level.theta_tolerance_entry", kind: "level", condition_kind: "tolerance_entry" },
    ],
    condition_rules: [
      { rule_id: "rule.theta.zero", condition_kind: "zero_crossing", predicate_id: "edge.theta_zero_crossing", target_key: "theta", epsilon: 0.001 },
      { rule_id: "rule.theta.extremum", condition_kind: "extremum", predicate_id: "edge.theta_extremum", target_key: "theta", epsilon: 0.001 },
      { rule_id: "rule.omega.sign", condition_kind: "sign_change", predicate_id: "edge.omega_sign_change", target_key: "omega", epsilon: 0.001 },
      { rule_id: "rule.theta.tolerance", condition_kind: "tolerance_entry", predicate_id: "level.theta_tolerance_entry", target_key: "theta", target: 0, tolerance: 0.005, epsilon: 0.001 },
    ],
    trace_rows: [],
    actions,
    resolved_scene_trace_policy: {
      schema: "seamgrim.resolved_scene_trace.v1",
      stage_plan_created: false,
      implicit_lww_allowed: false,
      records_selected_and_suppressed_actions: true,
    },
  };
}

function baseViews() {
  return {
    space2d: {
      schema: "seamgrim.space2d.v0",
      shapes: [
        { id: "measure.angle.theta.arc", group_id: "measure.angle.theta", kind: "line", role: "angle_measure", x1: 0, y1: 0, x2: 0.4, y2: -0.7, width: 1.4 },
        { id: "measure.angle.theta.baseline", group_id: "measure.angle.theta", kind: "line", role: "angle_baseline", x1: 0, y1: 0, x2: 0, y2: -1, width: 1.1 },
        { id: "pendulum.bob", kind: "circle", role: "pendulum_bob", x: 0.4, y: -0.9, r: 0.08, width: 1.2 },
      ],
      meta: { view_roles: ["space2d"] },
    },
  };
}

function viewPlanRuntime() {
  return {
    views: {
      view_plan: {
        camera: { default_policy: "fit" },
        anchors: [
          { anchor_id: "obs.bob", value: { x: 0.4, y: -0.9 } },
          { anchor_id: "obs.theta", value: 0 },
        ],
      },
      space2d: baseViews().space2d,
    },
  };
}

function sample(tick, theta, omega) {
  return { tick_id: tick, t: tick * 0.1, theta, omega };
}

function observationFromSample(row) {
  return {
    tick_id: row.tick_id,
    values: { tick_id: row.tick_id, t: row.t, theta: row.theta, omega: row.omega },
    all_values: { tick_id: row.tick_id, t: row.t, theta: row.theta, omega: row.omega },
  };
}

const normalSamples = [
  sample(0, 0.8, -0.5),
  sample(1, 0.35, -0.4),
  sample(2, -0.1, -0.3),
  sample(3, -0.45, -0.1),
  sample(4, -0.6, -0.02),
  sample(5, -0.4, 0.2),
  sample(6, -0.002, 0.4),
  sample(7, 0.3, 0.5),
  sample(8, 0.55, 0.3),
];

function buildObservationTrace(traceReactions, samples, source = "parity_fixture") {
  return buildSealedObservationTraceArtifact({
    lessonId: traceReactions.lesson_id,
    source,
    timelineSamples: samples.map((row, index) => ({ index, values: row })),
    rules: traceReactions.predicate_contract.trace_rules,
  });
}

function normalizeDiagnostics(diagnostics) {
  return (Array.isArray(diagnostics) ? diagnostics : [])
    .map((row) => ({
      code: String(row?.code ?? ""),
      view_code: String(row?.view_code ?? ""),
      severity: String(row?.severity ?? ""),
      what: String(row?.what ?? ""),
      where: String(row?.where ?? ""),
      fix: String(row?.fix ?? ""),
    }))
    .filter((row) => row.code)
    .sort((left, right) => `${left.code}:${left.where}`.localeCompare(`${right.code}:${right.where}`));
}

function assertThreeLayerDiagnostics(report, label) {
  const diagnostics = normalizeDiagnostics(report?.diagnostics);
  assert(diagnostics.length > 0, `${label}: diagnostic missing`);
  for (const row of diagnostics) {
    assert(row.what && row.where && row.fix, `${label}: diagnostic lacks what/where/fix ${JSON.stringify(row)}`);
  }
  return diagnostics;
}

function normalizeReport(report, cameraPolicy = "fit") {
  const resolved = report?.resolved_scene_trace ?? {};
  return {
    tick_id: Number(report?.tick_id ?? 0),
    rule_ids: (Array.isArray(report?.reaction_schedule_rows) ? report.reaction_schedule_rows : [])
      .map((row) => String(row?.rule_id ?? ""))
      .filter(Boolean)
      .sort(),
    predicate_result: (Array.isArray(report?.active_predicates) ? report.active_predicates : [])
      .map((row) => ({
        predicate_id: String(row?.predicate_id ?? ""),
        condition_kind: String(row?.condition_kind ?? ""),
        active: row?.active === true,
      }))
      .filter((row) => row.predicate_id)
      .sort((left, right) => left.predicate_id.localeCompare(right.predicate_id)),
    selected_actions: (Array.isArray(resolved.selected_actions) ? resolved.selected_actions : []).slice().sort(),
    suppressed_actions: (Array.isArray(resolved.suppressed_actions) ? resolved.suppressed_actions : []).slice().sort(),
    suppressed_action_rows: (Array.isArray(resolved.suppressed_action_rows) ? resolved.suppressed_action_rows : [])
      .map((row) => ({
        action_id: String(row?.action_id ?? ""),
        reason: String(row?.reason ?? ""),
        priority: row?.priority ?? null,
        selected_action_id: String(row?.selected_action_id ?? ""),
        selected_priority: row?.selected_priority ?? null,
      }))
      .sort((left, right) => `${left.action_id}:${left.reason}`.localeCompare(`${right.action_id}:${right.reason}`)),
    diagnostics: normalizeDiagnostics(report?.diagnostics),
    camera_policy: cameraPolicy,
    resolved_scene_trace_schema: String(resolved.schema ?? ""),
    state_hash_participation: report?.state_hash_participation === true,
  };
}

function offlineCanonical(traceReactions, samples, { includeDiagnostics = false } = {}) {
  const observationTrace = buildObservationTrace(traceReactions, samples, "offline_parity_fixture");
  const out = [];
  for (const row of samples) {
    const result = buildSeamgrimTraceReactionRuntime({
      traceReactions,
      observationTrace,
      observation: observationFromSample(row),
      views: clone(baseViews()),
      viewPlanRuntime: viewPlanRuntime(),
    });
    const report = result?.report;
    const selected = report?.resolved_scene_trace?.selected_actions ?? [];
    const diagnostics = report?.diagnostics ?? [];
    if (selected.length > 0 || (includeDiagnostics && diagnostics.length > 0)) {
      out.push(normalizeReport(report));
    }
  }
  return out;
}

function liveCanonical(traceReactions, samples, { includeDiagnostics = false, lineageId = "lineage:normal" } = {}) {
  let session = null;
  const out = [];
  for (const row of samples) {
    const step = evaluateLiveTraceReactionCommit({
      session,
      traceReactions,
      observation: observationFromSample(row),
      views: clone(baseViews()),
      viewPlanRuntime: viewPlanRuntime(),
      lineageId,
      runId: "run:normal",
      rulesetRef: traceReactions.trace_hash,
      lessonId: traceReactions.lesson_id,
    });
    session = step.session;
    const report = step.report;
    const selected = report?.resolved_scene_trace?.selected_actions ?? [];
    const diagnostics = report?.diagnostics ?? [];
    if ((selected.length > 0 && !report?.repeated_emission) || (includeDiagnostics && diagnostics.length > 0)) {
      out.push(normalizeReport(report));
    }
  }
  return out;
}

function runNormalParity() {
  const trace = baseTraceReactions();
  const offline = offlineCanonical(trace, normalSamples);
  const live = liveCanonical(trace, normalSamples);
  assert(offline.length >= 4, `normal offline event count too small: ${JSON.stringify(offline)}`);
  assert(stableCanonicalJson(offline) === stableCanonicalJson(live), `offline/live mismatch\n${stableCanonicalJson(offline)}\n${stableCanonicalJson(live)}`);
  assert(offline.every((row) => row.state_hash_participation === false), "resolved trace participated in state_hash");
  assert(offline.some((row) => row.selected_actions.includes("focus.bob")), "camera focus action missing from parity trace");

  const replayA = stableCanonicalJson({ offline, live });
  const replayB = stableCanonicalJson({
    offline: offlineCanonical(trace, normalSamples),
    live: liveCanonical(trace, normalSamples),
  });
  assert(replayA === replayB, "replay/scrub canonical result changed between repeated runs");
  return { offline, live, canonical_bytes: replayA.length };
}

function runDuplicateFixtures() {
  const trace = baseTraceReactions();
  let session = null;
  let step = evaluateLiveTraceReactionCommit({
    session,
    traceReactions: trace,
    observation: observationFromSample(sample(0, 0.5, -0.2)),
    views: clone(baseViews()),
    viewPlanRuntime: viewPlanRuntime(),
    lineageId: "lineage:dup",
    runId: "run:dup",
    rulesetRef: trace.trace_hash,
    lessonId: trace.lesson_id,
  });
  session = step.session;
  step = evaluateLiveTraceReactionCommit({
    session,
    traceReactions: trace,
    observation: observationFromSample(sample(1, -0.3, -0.2)),
    views: clone(baseViews()),
    viewPlanRuntime: viewPlanRuntime(),
    lineageId: "lineage:dup",
    runId: "run:dup",
    rulesetRef: trace.trace_hash,
    lessonId: trace.lesson_id,
  });
  const emissionCount = step.report.emissions.length;
  const same = evaluateLiveTraceReactionCommit({
    session: step.session,
    traceReactions: trace,
    observation: observationFromSample(sample(1, -0.3, -0.2)),
    views: clone(baseViews()),
    viewPlanRuntime: viewPlanRuntime(),
    lineageId: "lineage:dup",
    runId: "run:dup",
    rulesetRef: trace.trace_hash,
    lessonId: trace.lesson_id,
  });
  assert(same.report.duplicate_delivery === true, "same payload duplicate not marked");
  assert(same.report.repeated_emission === true, "same payload duplicate emitted again");
  assert(same.report.emissions.length === emissionCount, "same payload duplicate changed emission history");

  const mismatch = evaluateLiveTraceReactionCommit({
    session: same.session,
    traceReactions: trace,
    observation: observationFromSample(sample(1, -0.2, -0.2)),
    views: clone(baseViews()),
    viewPlanRuntime: viewPlanRuntime(),
    lineageId: "lineage:dup",
    runId: "run:dup",
    rulesetRef: trace.trace_hash,
    lessonId: trace.lesson_id,
  });
  const diagnostics = assertThreeLayerDiagnostics(mismatch.report, "duplicate mismatch");
  assert(diagnostics.some((row) => row.code === "E_VIEW_LIVE_DUPLICATE_TICK_MISMATCH"), "duplicate mismatch code missing");
}

function runOrderGapLineageFixtures() {
  const trace = baseTraceReactions();
  let step = evaluateLiveTraceReactionCommit({
    session: null,
    traceReactions: trace,
    observation: observationFromSample(sample(0, 0.5, -0.2)),
    views: clone(baseViews()),
    viewPlanRuntime: viewPlanRuntime(),
    lineageId: "lineage:a",
    runId: "run:a",
    rulesetRef: trace.trace_hash,
    lessonId: trace.lesson_id,
  });
  step = evaluateLiveTraceReactionCommit({
    session: step.session,
    traceReactions: trace,
    observation: observationFromSample(sample(3, -0.5, -0.1)),
    views: clone(baseViews()),
    viewPlanRuntime: viewPlanRuntime(),
    lineageId: "lineage:a",
    runId: "run:a",
    rulesetRef: trace.trace_hash,
    lessonId: trace.lesson_id,
  });
  let diagnostics = assertThreeLayerDiagnostics(step.report, "tick gap");
  assert(diagnostics.some((row) => row.code === "W_VIEW_LIVE_TICK_GAP"), "tick gap warning missing");

  step = evaluateLiveTraceReactionCommit({
    session: step.session,
    traceReactions: trace,
    observation: observationFromSample(sample(2, -0.4, -0.1)),
    views: clone(baseViews()),
    viewPlanRuntime: viewPlanRuntime(),
    lineageId: "lineage:a",
    runId: "run:a",
    rulesetRef: trace.trace_hash,
    lessonId: trace.lesson_id,
  });
  diagnostics = assertThreeLayerDiagnostics(step.report, "reverse tick");
  assert(diagnostics.some((row) => row.code === "E_VIEW_LIVE_OUT_OF_ORDER"), "reverse tick diagnostic missing");

  const changed = evaluateLiveTraceReactionCommit({
    session: step.session,
    traceReactions: trace,
    observation: observationFromSample(sample(0, 0.2, 0.1)),
    views: clone(baseViews()),
    viewPlanRuntime: viewPlanRuntime(),
    lineageId: "lineage:b",
    runId: "run:b",
    rulesetRef: trace.trace_hash,
    lessonId: trace.lesson_id,
  });
  assert(changed.report.lineage_reset === true, "lineage transition did not reset");
  assert(changed.report.live_sample_count === 1, "lineage transition did not clear live samples");
  assert(changed.report.lineage_id === "lineage:b", "lineage id not updated");
}

function runConflictFixture() {
  const trace = baseTraceReactions([
    {
      action_id: "conflict.orange",
      kind: "highlight",
      when: "edge.theta_zero_crossing",
      priority: 10,
      targets: { group_ids: ["measure.angle.theta"] },
      style: { stroke: "#f97316" },
    },
    {
      action_id: "conflict.blue",
      kind: "highlight",
      when: "edge.theta_zero_crossing",
      priority: 10,
      targets: { group_ids: ["measure.angle.theta"] },
      style: { stroke: "#2563eb" },
    },
  ]);
  const samples = [sample(0, 0.4, -0.2), sample(1, -0.2, -0.2), sample(2, -0.4, -0.1)];
  const offline = offlineCanonical(trace, samples, { includeDiagnostics: true }).find((row) => row.tick_id === 1);
  const live = liveCanonical(trace, samples, { includeDiagnostics: true, lineageId: "lineage:conflict" }).find((row) => row.tick_id === 1);
  assert(offline && live, "conflict fixture did not produce offline/live reports");
  assert(stableCanonicalJson(offline) === stableCanonicalJson(live), "conflict offline/live canonical mismatch");
  for (const row of [offline, live]) {
    assert(row.selected_actions.length === 0, `conflict selected actions unexpectedly applied: ${JSON.stringify(row.selected_actions)}`);
    assert(row.suppressed_action_rows.some((item) => item.reason === "same_priority_conflict"), "conflict suppression reason missing");
    assert(row.diagnostics.some((diag) => diag.code === "E_TRACE_REACTION_CONFLICT" && diag.view_code === "E_VIEW_REACTION_CONFLICT"), "conflict diagnostic alias missing");
    assert(row.diagnostics.every((diag) => diag.what && diag.where && diag.fix), "conflict diagnostic lacks 3-layer fields");
  }
}

function runPriorityFixture() {
  const trace = baseTraceReactions([
    {
      action_id: "priority.low",
      kind: "highlight",
      when: "edge.theta_zero_crossing",
      priority: 5,
      targets: { group_ids: ["measure.angle.theta"] },
      style: { stroke: "#94a3b8" },
    },
    {
      action_id: "priority.high",
      kind: "highlight",
      when: "edge.theta_zero_crossing",
      priority: 20,
      targets: { group_ids: ["measure.angle.theta"] },
      style: { stroke: "#f97316" },
    },
  ]);
  const samples = [sample(0, 0.4, -0.2), sample(1, -0.2, -0.2), sample(2, -0.4, -0.1)];
  const offline = offlineCanonical(trace, samples).find((row) => row.tick_id === 1);
  const live = liveCanonical(trace, samples, { lineageId: "lineage:priority" }).find((row) => row.tick_id === 1);
  assert(offline && live, "priority fixture did not produce offline/live reports");
  assert(stableCanonicalJson(offline) === stableCanonicalJson(live), "priority offline/live canonical mismatch");
  assert(JSON.stringify(offline.selected_actions) === JSON.stringify(["priority.high"]), `priority winner mismatch: ${JSON.stringify(offline)}`);
  assert(offline.suppressed_action_rows.some((row) => row.action_id === "priority.low" && row.reason === "lower_priority"), "lower priority suppression missing");
  assert(offline.diagnostics.length === 0, `priority resolution should not diagnose: ${JSON.stringify(offline.diagnostics)}`);
}

const normal = runNormalParity();
runDuplicateFixtures();
runOrderGapLineageFixtures();
runConflictFixture();
runPriorityFixture();

assert(normal.canonical_bytes > 1000, "normal canonical parity payload unexpectedly small");
process.stdout.write(`${OK}\n`);
