import { buildSealedObservationTraceArtifact } from "./sealed_trace_evaluator.js";
import { buildSeamgrimTraceReactionRuntime } from "./trace_reaction_runtime.js";

function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function stableCanonicalJson(value) {
  if (Array.isArray(value)) {
    return `[${value.map((item) => stableCanonicalJson(item)).join(",")}]`;
  }
  if (isObject(value)) {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableCanonicalJson(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function fnv1a32(text = "") {
  let hash = 0x811c9dc5;
  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

function hashCandidate(value, label = "live_trace") {
  return `candidate:fnv1a32:${label}:${fnv1a32(stableCanonicalJson(value))}`;
}

function toFiniteNumber(value, fallback = NaN) {
  const number = Number(value);
  return Number.isFinite(number) ? number : fallback;
}

function readObservationValues(observation = {}) {
  return isObject(observation?.all_values)
    ? observation.all_values
    : isObject(observation?.values)
      ? observation.values
      : {};
}

function normalizeLiveSample(observation = {}, fallbackTick = 0) {
  const values = readObservationValues(observation);
  const tick = toFiniteNumber(values.tick_id ?? observation?.tick_id, fallbackTick);
  return {
    tick_id: Math.max(0, Math.trunc(Number.isFinite(tick) ? tick : fallbackTick)),
    t: toFiniteNumber(values.t),
    theta: toFiniteNumber(values.theta),
    omega: toFiniteNumber(values.omega),
  };
}

function samePayload(left, right) {
  if (!isObject(left) || !isObject(right)) return false;
  return ["t", "theta", "omega"].every((key) => {
    const a = Number(left[key]);
    const b = Number(right[key]);
    return Number.isFinite(a) && Number.isFinite(b) && Math.abs(a - b) <= 1e-12;
  });
}

function diag(code, message, extra = {}, severity = "error") {
  return {
    code,
    severity,
    message,
    what: message,
    where: String(extra.where ?? extra.tick_id ?? extra.lineage_id ?? "live_observation_commit"),
    fix: String(extra.fix ?? "같은 계보에서는 tick 순서와 payload를 안정적으로 전달하세요."),
    ...extra,
  };
}

function ensureSession(session = null) {
  if (isObject(session)) return session;
  return {
    schema: "seamgrim.live_trace_reaction_session.v1",
    lineage_id: "",
    run_id: "",
    last_tick: null,
    samples: [],
    sample_by_tick: new Map(),
    emitted_by_tick: new Map(),
    emissions: [],
    duplicate_count: 0,
    reset_count: 0,
  };
}

function resetForLineage(session, lineageId, runId) {
  session.lineage_id = lineageId;
  session.run_id = runId;
  session.last_tick = null;
  session.samples = [];
  session.sample_by_tick = new Map();
  session.emitted_by_tick = new Map();
  session.emissions = [];
  session.duplicate_count = 0;
  session.reset_count = Math.max(0, Number(session.reset_count) || 0) + 1;
  return session;
}

function buildTraceFromSession({ session, traceReactions, lessonId = "" }) {
  return buildSealedObservationTraceArtifact({
    lessonId,
    source: "seamgrim_wasm_committed_observation_live",
    timelineSamples: session.samples.map((sample, index) => ({
      index,
      values: sample,
    })),
    rules: traceReactions?.predicate_contract?.trace_rules ?? null,
  });
}

function buildObservationFromSample(sample, sourceObservation = {}) {
  const sourceValues = readObservationValues(sourceObservation);
  return {
    ...(isObject(sourceObservation) ? sourceObservation : {}),
    tick_id: sample.tick_id,
    values: {
      ...sourceValues,
      tick_id: sample.tick_id,
      t: sample.t,
      theta: sample.theta,
      omega: sample.omega,
    },
    all_values: {
      ...sourceValues,
      tick_id: sample.tick_id,
      t: sample.t,
      theta: sample.theta,
      omega: sample.omega,
    },
  };
}

function findPendingEventTick({ runtime, traceReactions, session, currentTick }) {
  const events = Array.isArray(runtime?.report?.predicate_events?.events)
    ? runtime.report.predicate_events.events
    : [];
  const actionPredicates = new Set((Array.isArray(traceReactions?.actions) ? traceReactions.actions : [])
    .map((action) => String(action?.when ?? "").trim())
    .filter(Boolean));
  const ticks = events
    .filter((event) => actionPredicates.has(String(event?.predicate_id ?? "").trim()))
    .map((event) => Math.max(0, Math.trunc(toFiniteNumber(event?.tick_id, -1))))
    .filter((eventTick) => eventTick >= 0 && eventTick <= currentTick && !session.emitted_by_tick.has(eventTick))
    .sort((left, right) => left - right);
  return ticks.length ? ticks[0] : currentTick;
}

export function createLiveTraceReactionSession(seed = {}) {
  const session = ensureSession(seed);
  if (!(session.sample_by_tick instanceof Map)) session.sample_by_tick = new Map();
  if (!(session.emitted_by_tick instanceof Map)) session.emitted_by_tick = new Map();
  if (!Array.isArray(session.samples)) session.samples = [];
  if (!Array.isArray(session.emissions)) session.emissions = [];
  return session;
}

export function evaluateLiveTraceReactionCommit({
  session = null,
  traceReactions = null,
  observation = null,
  views = null,
  viewPlanRuntime = null,
  lineageId = "",
  runId = "",
  rulesetRef = "",
  lessonId = "",
} = {}) {
  const liveSession = createLiveTraceReactionSession(session);
  const normalizedLineageId = String(lineageId ?? "").trim() || "lineage:default";
  const normalizedRunId = String(runId ?? "").trim() || "run:default";
  const diagnostics = [];
  let lineageReset = false;
  if (liveSession.lineage_id !== normalizedLineageId) {
    resetForLineage(liveSession, normalizedLineageId, normalizedRunId);
    lineageReset = true;
  } else {
    liveSession.run_id = normalizedRunId;
  }

  const sample = normalizeLiveSample(observation, Number(liveSession.last_tick ?? -1) + 1);
  const tick = sample.tick_id;
  const payloadHash = hashCandidate(sample, "live_observation_payload");
  const previousSameTick = liveSession.sample_by_tick.get(tick);
  let duplicateDelivery = false;
  let mismatchDuplicate = false;
  if (previousSameTick) {
    duplicateDelivery = true;
    if (!samePayload(previousSameTick.sample, sample)) {
      mismatchDuplicate = true;
      diagnostics.push(diag("E_VIEW_LIVE_DUPLICATE_TICK_MISMATCH", "같은 계보와 마디에 다른 observation payload가 다시 도착했습니다.", {
        tick_id: tick,
        lineage_id: normalizedLineageId,
        previous_payload_hash: previousSameTick.payload_hash,
        incoming_payload_hash: payloadHash,
        fix: "같은 tick을 다시 전달하려면 동일 payload를 보내거나 새 lineage를 시작하세요.",
      }));
    }
  }
  if (!duplicateDelivery) {
    if (liveSession.last_tick !== null && tick < liveSession.last_tick) {
      diagnostics.push(diag("E_VIEW_LIVE_OUT_OF_ORDER", "이전 마디보다 작은 tick이 live evaluator에 도착했습니다.", {
        tick_id: tick,
        previous_tick_id: liveSession.last_tick,
        lineage_id: normalizedLineageId,
        fix: "WASM commit 순서대로 observation을 전달하세요.",
      }));
    } else if (liveSession.last_tick !== null && tick > liveSession.last_tick + 1) {
      diagnostics.push(diag("W_VIEW_LIVE_TICK_GAP", "live observation tick 사이에 빈 마디가 있습니다.", {
        tick_id: tick,
        previous_tick_id: liveSession.last_tick,
        lineage_id: normalizedLineageId,
        fix: "누락된 마디가 의도인지 확인하고 완전 재현 주장을 보류하세요.",
      }, "warning"));
    }
    liveSession.samples.push(sample);
    liveSession.samples.sort((left, right) => left.tick_id - right.tick_id);
    liveSession.sample_by_tick.set(tick, { sample, payload_hash: payloadHash });
    liveSession.last_tick = tick;
  } else {
    liveSession.duplicate_count += 1;
  }

  const observationTrace = buildTraceFromSession({
    session: liveSession,
    traceReactions,
    lessonId,
  });
  const liveTraceReactions = isObject(traceReactions)
    ? {
      ...traceReactions,
      mode: "live_committed_observation",
      live_online_claim: true,
      predicate_contract: {
        ...(isObject(traceReactions.predicate_contract) ? traceReactions.predicate_contract : {}),
        source: "sealed_observation_trace_evaluator",
        live_adapter: "committed_observation_once_per_tick",
      },
    }
    : null;
  const committedObservation = buildObservationFromSample(sample, observation);
  const runtime = buildSeamgrimTraceReactionRuntime({
    traceReactions: liveTraceReactions,
    observationTrace,
    observation: committedObservation,
    views,
    viewPlanRuntime,
  });
  if (!runtime?.report) {
    return {
      session: liveSession,
      views,
      report: null,
    };
  }
  const emissionTick = findPendingEventTick({
    runtime,
    traceReactions: liveTraceReactions,
    session: liveSession,
    currentTick: tick,
  });
  const emissionSample = liveSession.sample_by_tick.get(emissionTick)?.sample ?? sample;
  const emissionObservation = buildObservationFromSample(emissionSample, observation);
  const emissionRuntime = emissionTick === tick
    ? runtime
    : buildSeamgrimTraceReactionRuntime({
      traceReactions: liveTraceReactions,
      observationTrace,
      observation: emissionObservation,
      views,
      viewPlanRuntime,
    });
  const activeRuntime = emissionRuntime?.report ? emissionRuntime : runtime;
  const existingEmission = liveSession.emitted_by_tick.get(emissionTick);
  const selectedActions = Array.isArray(activeRuntime.report?.resolved_scene_trace?.selected_actions)
    ? activeRuntime.report.resolved_scene_trace.selected_actions.slice()
    : [];
  const eventHash = String(activeRuntime.report?.predicate_event_hash_candidate ?? "");
  const emissionHash = hashCandidate({ tick: emissionTick, selectedActions, eventHash }, "live_reaction_emission");
  const repeatEmission = Boolean(existingEmission && existingEmission === emissionHash);
  if (!existingEmission && selectedActions.length > 0) {
    liveSession.emitted_by_tick.set(emissionTick, emissionHash);
      liveSession.emissions.push({
        tick_id: emissionTick,
        commit_tick_id: tick,
        selected_actions: selectedActions,
        emission_hash_candidate: emissionHash,
      });
  }
  const report = {
    ...activeRuntime.report,
    schema: "seamgrim.live_trace_reaction.runtime.v1",
    mode: "live_committed_observation",
    live_online_claim: true,
    state_hash_participation: false,
    lineage_id: normalizedLineageId,
    run_id: normalizedRunId,
    ruleset_ref: String(rulesetRef ?? "").trim(),
    committed_observation: sample,
    committed_observation_hash_candidate: payloadHash,
    commit_tick_id: tick,
    utterance_tick_id: emissionTick,
    delayed_event_delivery: emissionTick < tick,
    lineage_reset: lineageReset,
    duplicate_delivery: duplicateDelivery,
    duplicate_mismatch: mismatchDuplicate,
    repeated_emission: repeatEmission,
    once_per_tick: true,
    lineage_reset_count: Math.max(0, Number(liveSession.reset_count) || 0),
    live_sample_count: liveSession.samples.length,
    emissions: liveSession.emissions.slice(),
    fired_ticks: liveSession.emissions.map((row) => row.tick_id),
    diagnostics: [...diagnostics, ...(Array.isArray(activeRuntime.report.diagnostics) ? activeRuntime.report.diagnostics : [])],
  };
  return {
    session: liveSession,
    views: activeRuntime.views,
    report,
  };
}

export const __test__ = {
  stableCanonicalJson,
  hashCandidate,
  normalizeLiveSample,
};
