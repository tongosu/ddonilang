function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function toFiniteNumber(value, fallback = NaN) {
  const number = Number(value);
  return Number.isFinite(number) ? number : fallback;
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

function buildHashCandidate(value, label = "sealed_observation_trace") {
  return `candidate:fnv1a32:${label}:${fnv1a32(stableCanonicalJson(value))}`;
}

function stableSortEvents(left, right) {
  const tickDelta = Number(left.tick_id) - Number(right.tick_id);
  if (tickDelta !== 0) return tickDelta;
  const ruleDelta = Number(left.rule_order) - Number(right.rule_order);
  if (ruleDelta !== 0) return ruleDelta;
  return String(left.event_id ?? "").localeCompare(String(right.event_id ?? ""));
}

function traceDiag(code, message, extra = {}) {
  return {
    code,
    severity: "error",
    message,
    ...extra,
  };
}

function readValue(values, key) {
  if (!isObject(values)) return NaN;
  return toFiniteNumber(values[key]);
}

function normalizeTraceSample(raw, index = 0) {
  const row = isObject(raw) ? raw : {};
  const values = isObject(row.values) ? row.values : row;
  const tick = toFiniteNumber(values.tick_id ?? row.tick_id ?? row.index ?? index);
  const sample = {
    tick_id: Math.max(0, Math.trunc(Number.isFinite(tick) ? tick : index)),
    t: readValue(values, "t"),
    theta: readValue(values, "theta"),
    omega: readValue(values, "omega"),
  };
  return sample;
}

function normalizeCurrentObservationSample(observation, fallbackTick = 0) {
  const values = isObject(observation?.all_values)
    ? observation.all_values
    : isObject(observation?.values)
      ? observation.values
      : {};
  return normalizeTraceSample({
    index: fallbackTick,
    tick_id: fallbackTick,
    values,
  }, fallbackTick);
}

function isSameTraceSample(left, right) {
  if (!isObject(left) || !isObject(right)) return false;
  for (const key of ["t", "theta", "omega"]) {
    if (!Number.isFinite(Number(left[key])) || !Number.isFinite(Number(right[key]))) return false;
    if (Math.abs(Number(left[key]) - Number(right[key])) > 1e-12) return false;
  }
  return true;
}

export function buildSealedObservationTraceArtifact({
  lessonId = "",
  timelineSamples = [],
  currentObservation = null,
  source = "seamgrim_wasm_dotbogi_timeline",
  requiredValues = ["tick_id", "t", "theta", "omega"],
  rules = null,
} = {}) {
  const rows = [];
  const sourceRows = Array.isArray(timelineSamples) ? timelineSamples : [];
  sourceRows.forEach((sample, index) => {
    rows.push(normalizeTraceSample(sample, index));
  });
  if (currentObservation) {
    const current = normalizeCurrentObservationSample(currentObservation, rows.length);
    const sameSampleIndex = rows.findIndex((row) => isSameTraceSample(row, current));
    const existingIndex = sameSampleIndex >= 0
      ? sameSampleIndex
      : rows.findIndex((row) => row.tick_id === current.tick_id);
    if (existingIndex >= 0) {
      rows[existingIndex] = { ...current, tick_id: rows[existingIndex].tick_id };
    } else {
      rows.push(current);
    }
  }
  const samples = rows
    .filter((row) => (
      Number.isFinite(row.tick_id)
      && Number.isFinite(row.t)
      && Number.isFinite(row.theta)
      && Number.isFinite(row.omega)
    ))
    .sort((left, right) => left.tick_id - right.tick_id)
    .map((row, index) => ({
      sample_index: index,
      tick_id: row.tick_id,
      t: row.t,
      theta: row.theta,
      omega: row.omega,
    }));
  const traceCore = {
    lesson_id: String(lessonId ?? "").trim(),
    source: String(source ?? "").trim() || "seamgrim_wasm_dotbogi_timeline",
    required_values: requiredValues,
    sample_count: samples.length,
    samples,
    rules: isObject(rules) ? rules : null,
  };
  return {
    schema: "seamgrim.sealed_observation_trace.v1",
    sealed: true,
    state_hash_participation: false,
    runtime_semantics_claim: false,
    provenance: {
      source: traceCore.source,
      lesson_id: traceCore.lesson_id,
      frontdoor: "seamgrim_product_wasm",
      timeline_owner: "dotbogi",
      console_string_parse: false,
    },
    ...traceCore,
    trace_hash_candidate: buildHashCandidate(traceCore, "sealed_observation_trace"),
  };
}

function normalizeTraceSamplesForEvaluation(observationTrace) {
  const samples = Array.isArray(observationTrace?.samples) ? observationTrace.samples : [];
  const diagnostics = [];
  const out = [];
  let previousTick = null;
  samples.forEach((sample, index) => {
    const row = isObject(sample) ? sample : {};
    const tick = toFiniteNumber(row.tick_id);
    if (!Number.isFinite(tick)) {
      diagnostics.push(traceDiag("E_TRACE_SAMPLE_MISSING_VALUE", "trace sample tick_id is missing or non-finite", { sample_index: index, key: "tick_id" }));
      return;
    }
    const tickId = Math.trunc(tick);
    if (previousTick !== null) {
      if (tickId === previousTick) {
        diagnostics.push(traceDiag("E_TRACE_SAMPLE_DUPLICATE_TICK", "trace sample tick_id is duplicated", { sample_index: index, tick_id: tickId }));
      } else if (tickId < previousTick) {
        diagnostics.push(traceDiag("E_TRACE_SAMPLE_TICK_ORDER", "trace samples must be sorted by increasing tick_id", { sample_index: index, tick_id: tickId, previous_tick_id: previousTick }));
      }
    }
    previousTick = tickId;
    const normalized = {
      sample_index: Number.isFinite(Number(row.sample_index)) ? Math.trunc(Number(row.sample_index)) : index,
      tick_id: tickId,
      t: toFiniteNumber(row.t),
      theta: toFiniteNumber(row.theta),
      omega: toFiniteNumber(row.omega),
    };
    for (const key of ["t", "theta", "omega"]) {
      if (!(key in row)) {
        diagnostics.push(traceDiag("E_TRACE_SAMPLE_MISSING_VALUE", "trace sample required value is missing", { sample_index: index, tick_id: tickId, key }));
      } else if (!Number.isFinite(normalized[key])) {
        diagnostics.push(traceDiag("E_TRACE_SAMPLE_NONFINITE", "trace sample required value is non-finite", { sample_index: index, tick_id: tickId, key }));
      }
    }
    out.push(normalized);
  });
  return { samples: out, diagnostics };
}

function normalizeConditionRules(conditionRules = []) {
  const source = Array.isArray(conditionRules) ? conditionRules : [];
  return source
    .map((rule, index) => {
      const conditionKind = String(rule?.condition_kind ?? "").trim();
      const targetDefault = conditionKind === "sign_change" ? "omega" : "theta";
      const epsilon = toFiniteNumber(rule?.epsilon, 0.001);
      const tolerance = toFiniteNumber(rule?.tolerance, 0.005);
      const target = toFiniteNumber(rule?.target, 0);
      return {
        rule_order: index,
        rule_id: String(rule?.rule_id ?? `rule.${conditionKind || "unknown"}.${index + 1}`).trim(),
        condition_kind: conditionKind,
        predicate_id: String(rule?.predicate_id ?? "").trim(),
        target_key: String(rule?.target_key ?? targetDefault).trim() || targetDefault,
        epsilon: Number.isFinite(epsilon) && epsilon >= 0 ? epsilon : 0.001,
        tolerance: Number.isFinite(tolerance) && tolerance >= 0 ? tolerance : 0.005,
        target: Number.isFinite(target) ? target : 0,
      };
    })
    .filter((rule) => rule.condition_kind && rule.predicate_id);
}

function classifySign(value, epsilon) {
  const number = Number(value);
  if (!Number.isFinite(number)) return 0;
  if (Math.abs(number) <= epsilon) return 0;
  return number < 0 ? -1 : 1;
}

function makeEvent(rule, sample, detail = {}) {
  const conditionKind = String(rule.condition_kind ?? "").trim();
  const tickId = Math.max(0, Math.trunc(Number(sample.tick_id) || 0));
  return {
    event_id: `event.${String(rule.rule_id ?? conditionKind).replace(/[^a-zA-Z0-9._-]+/g, "_")}.${tickId}`,
    rule_id: rule.rule_id,
    rule_order: rule.rule_order,
    predicate_id: rule.predicate_id,
    condition_kind: conditionKind,
    tick_id: tickId,
    t: sample.t,
    sample_index: sample.sample_index,
    target_key: rule.target_key,
    value: sample[rule.target_key],
    epsilon: rule.epsilon,
    ...(conditionKind === "tolerance_entry" ? { target: rule.target, tolerance: rule.tolerance } : {}),
    detail,
  };
}

function evaluateZeroLikeRule(samples, rule) {
  const events = [];
  let previousNonZeroSign = null;
  let pendingZeroSample = null;
  for (const sample of samples) {
    const value = sample[rule.target_key];
    const sign = classifySign(value, rule.epsilon);
    if (sign === 0) {
      if (!pendingZeroSample) pendingZeroSample = sample;
      continue;
    }
    if (pendingZeroSample) {
      if (previousNonZeroSign !== null && previousNonZeroSign !== sign) {
        events.push(makeEvent(rule, pendingZeroSample, {
          zero_plateau: true,
          previous_sign: previousNonZeroSign,
          next_sign: sign,
        }));
      }
      pendingZeroSample = null;
    } else if (previousNonZeroSign !== null && previousNonZeroSign !== sign) {
      events.push(makeEvent(rule, sample, {
        crossed_between_samples: true,
        previous_sign: previousNonZeroSign,
        next_sign: sign,
      }));
    }
    previousNonZeroSign = sign;
  }
  return events;
}

function buildPlateauGroups(samples, rule) {
  const groups = [];
  for (const sample of samples) {
    const value = sample[rule.target_key];
    const last = groups[groups.length - 1] ?? null;
    if (!last || Math.abs(Number(value) - Number(last.value)) > rule.epsilon) {
      groups.push({
        start: sample,
        end: sample,
        value,
      });
    } else {
      last.end = sample;
    }
  }
  return groups;
}

function evaluateExtremumRule(samples, rule) {
  const events = [];
  const groups = buildPlateauGroups(samples, rule);
  for (let index = 1; index < groups.length - 1; index += 1) {
    const previous = groups[index - 1];
    const current = groups[index];
    const next = groups[index + 1];
    const prevDirection = classifySign(Number(current.value) - Number(previous.value), rule.epsilon);
    const nextDirection = classifySign(Number(next.value) - Number(current.value), rule.epsilon);
    if (prevDirection > 0 && nextDirection < 0) {
      events.push(makeEvent(rule, current.start, {
        extremum_kind: "max",
        plateau_start_tick: current.start.tick_id,
        plateau_end_tick: current.end.tick_id,
      }));
    } else if (prevDirection < 0 && nextDirection > 0) {
      events.push(makeEvent(rule, current.start, {
        extremum_kind: "min",
        plateau_start_tick: current.start.tick_id,
        plateau_end_tick: current.end.tick_id,
      }));
    }
  }
  return events;
}

function evaluateToleranceEntryRule(samples, rule) {
  const events = [];
  const levelStates = [];
  let previousInside = null;
  for (const sample of samples) {
    const value = Number(sample[rule.target_key]);
    const inside = Math.abs(value - rule.target) <= rule.tolerance;
    levelStates.push({
      rule_id: rule.rule_id,
      predicate_id: rule.predicate_id,
      tick_id: sample.tick_id,
      sample_index: sample.sample_index,
      inside,
    });
    if (inside && previousInside === false) {
      events.push(makeEvent(rule, sample, {
        level_state: "entry",
        inside,
      }));
    }
    previousInside = inside;
  }
  return { events, levelStates };
}

function evaluateRule(samples, rule) {
  if (rule.condition_kind === "zero_crossing" || rule.condition_kind === "sign_change") {
    return { events: evaluateZeroLikeRule(samples, rule), levelStates: [] };
  }
  if (rule.condition_kind === "extremum") {
    return { events: evaluateExtremumRule(samples, rule), levelStates: [] };
  }
  if (rule.condition_kind === "tolerance_entry") {
    return evaluateToleranceEntryRule(samples, rule);
  }
  return {
    events: [],
    levelStates: [],
    diagnostics: [traceDiag("E_TRACE_RULE_UNSUPPORTED", "unsupported sealed trace condition rule", { rule_id: rule.rule_id, condition_kind: rule.condition_kind })],
  };
}

export function evaluateSealedTracePredicates({ observationTrace = null, conditionRules = [] } = {}) {
  const { samples, diagnostics } = normalizeTraceSamplesForEvaluation(observationTrace);
  const rules = normalizeConditionRules(conditionRules);
  const ruleDiagnostics = [];
  if (!rules.length) {
    ruleDiagnostics.push(traceDiag("E_TRACE_RULE_MISSING", "sealed trace condition rules are missing"));
  }
  const events = [];
  const levelStates = [];
  if (diagnostics.length === 0) {
    for (const rule of rules) {
      const result = evaluateRule(samples, rule);
      events.push(...(Array.isArray(result.events) ? result.events : []));
      levelStates.push(...(Array.isArray(result.levelStates) ? result.levelStates : []));
      ruleDiagnostics.push(...(Array.isArray(result.diagnostics) ? result.diagnostics : []));
    }
  }
  const allDiagnostics = [...diagnostics, ...ruleDiagnostics];
  const sortedEvents = events.sort(stableSortEvents).map((event) => {
    const { rule_order: _ruleOrder, ...publicEvent } = event;
    return publicEvent;
  });
  const core = {
    trace_hash_candidate: String(observationTrace?.trace_hash_candidate ?? ""),
    rules: rules.map((rule) => ({
      rule_id: rule.rule_id,
      condition_kind: rule.condition_kind,
      predicate_id: rule.predicate_id,
      target_key: rule.target_key,
      epsilon: rule.epsilon,
      ...(rule.condition_kind === "tolerance_entry" ? { target: rule.target, tolerance: rule.tolerance } : {}),
    })),
    events: sortedEvents,
    level_states: levelStates,
    diagnostics: allDiagnostics,
  };
  return {
    schema: "seamgrim.trace_predicate_events.v1",
    byte_stable: true,
    ...core,
    evaluation_hash_candidate: buildHashCandidate(core, "trace_predicate_events"),
  };
}

export const __test__ = {
  stableCanonicalJson,
  buildHashCandidate,
  normalizeTraceSamplesForEvaluation,
};
