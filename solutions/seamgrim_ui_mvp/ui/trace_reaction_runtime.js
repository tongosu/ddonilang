import { evaluateSealedTracePredicates } from "./sealed_trace_evaluator.js";

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

function buildReactionHashCandidate(value, label = "trace_reaction_frame") {
  return `candidate:fnv1a32:${label}:${fnv1a32(stableCanonicalJson(value))}`;
}

function readObservationValue(observation, key) {
  const normalized = String(key ?? "").trim();
  if (!normalized) return NaN;
  const candidates = [
    observation?.values?.[normalized],
    observation?.all_values?.[normalized],
    observation?.row?.[normalized],
    observation?.channels?.[normalized]?.value,
  ];
  for (const candidate of candidates) {
    const number = toFiniteNumber(candidate);
    if (Number.isFinite(number)) return number;
  }
  return NaN;
}

function readTickAndTime(observation) {
  const tick = toFiniteNumber(
    observation?.values?.tick_id ?? observation?.all_values?.tick_id ?? observation?.tick_id ?? observation?.row?.tick_id,
    0,
  );
  const time = readObservationValue(observation, "t");
  return {
    tick_id: Math.max(0, Math.trunc(Number.isFinite(tick) ? tick : 0)),
    time: Number.isFinite(time) ? time : toFiniteNumber(observation?.time, 0),
  };
}

function rowMatches(row, tickId, time) {
  if (!isObject(row)) return false;
  const tickMin = toFiniteNumber(row.tick_min, -Infinity);
  const tickMax = toFiniteNumber(row.tick_max, Infinity);
  const timeMin = toFiniteNumber(row.time_min, -Infinity);
  const timeMax = toFiniteNumber(row.time_max, Infinity);
  return tickId >= tickMin && tickId <= tickMax && time >= timeMin && time <= timeMax;
}

function normalizePredicates(traceReactions) {
  const predicates = Array.isArray(traceReactions?.predicates) ? traceReactions.predicates : [];
  const byId = new Map();
  predicates.forEach((predicate) => {
    const id = String(predicate?.predicate_id ?? "").trim();
    if (!id) return;
    byId.set(id, {
      predicate_id: id,
      kind: String(predicate?.kind ?? "").trim() || "level",
      label: String(predicate?.label ?? id).trim() || id,
      source: String(predicate?.source ?? "sealed_trace_row").trim() || "sealed_trace_row",
      condition_kind: String(predicate?.condition_kind ?? "").trim(),
    });
  });
  return byId;
}

function resolveActivePredicatesFromRows(traceReactions, tickId, time) {
  const rows = Array.isArray(traceReactions?.trace_rows) ? traceReactions.trace_rows : [];
  const row = rows.find((candidate) => rowMatches(candidate, tickId, time)) ?? null;
  const predicateDefs = normalizePredicates(traceReactions);
  const states = isObject(row?.predicate_states) ? row.predicate_states : {};
  const activeById = new Map();
  Object.keys(states).sort().forEach((id) => {
    if (states[id] !== true) return;
    const def = predicateDefs.get(id) ?? { predicate_id: id, kind: "level", label: id, source: "sealed_trace_row" };
    activeById.set(id, { ...def, active: true });
  });
  const schedule = Array.isArray(traceReactions?.reaction_schedule) ? traceReactions.reaction_schedule : [];
  const activeSchedule = schedule
    .filter((scheduleRow) => rowMatches(scheduleRow, tickId, time))
    .map((scheduleRow) => {
      const predicateId = String(scheduleRow?.predicate_id ?? "").trim();
      const def = predicateDefs.get(predicateId) ?? {
        predicate_id: predicateId,
        kind: "edge",
        label: predicateId,
        source: "sealed_reaction_schedule",
        condition_kind: String(scheduleRow?.condition_kind ?? "").trim(),
      };
      if (predicateId) {
        activeById.set(predicateId, {
          ...def,
          condition_kind: String(scheduleRow?.condition_kind ?? def.condition_kind ?? "").trim(),
          active: true,
          schedule_id: String(scheduleRow?.schedule_id ?? "").trim(),
          utterance_tick: Math.max(0, Math.trunc(toFiniteNumber(scheduleRow?.utterance_tick, tickId))),
        });
      }
      return {
        schedule_id: String(scheduleRow?.schedule_id ?? "").trim(),
        predicate_id: predicateId,
        condition_kind: String(scheduleRow?.condition_kind ?? "").trim(),
        utterance_tick: Math.max(0, Math.trunc(toFiniteNumber(scheduleRow?.utterance_tick, tickId))),
        source_row: String(scheduleRow?.source_row ?? "").trim(),
      };
    });
  return { row, active: Array.from(activeById.values()), activeSchedule };
}

function resolveActivePredicates(traceReactions, tickId, time, observationTrace = null) {
  const rowResult = resolveActivePredicatesFromRows(traceReactions, tickId, time);
  const scheduleKind = String(traceReactions?.predicate_contract?.source ?? "").trim();
  if (scheduleKind !== "sealed_observation_trace_evaluator" || !isObject(observationTrace)) {
    return { ...rowResult, evaluation: null };
  }
  const predicateDefs = normalizePredicates(traceReactions);
  const activeById = new Map(rowResult.active.map((predicate) => [predicate.predicate_id, predicate]));
  const evaluation = evaluateSealedTracePredicates({
    observationTrace,
    conditionRules: traceReactions.condition_rules ?? [],
  });
  const events = Array.isArray(evaluation?.events) ? evaluation.events : [];
  const activeEvents = events.filter((event) => Math.trunc(toFiniteNumber(event?.tick_id, -1)) === tickId);
  const activeSchedule = activeEvents.map((event) => {
    const predicateId = String(event?.predicate_id ?? "").trim();
    const def = predicateDefs.get(predicateId) ?? {
      predicate_id: predicateId,
      kind: String(predicateId).startsWith("level.") ? "level" : "edge",
      label: predicateId,
      source: "sealed_observation_trace_evaluator",
      condition_kind: String(event?.condition_kind ?? "").trim(),
    };
    if (predicateId) {
      activeById.set(predicateId, {
        ...def,
        source: "sealed_observation_trace_evaluator",
        condition_kind: String(event?.condition_kind ?? def.condition_kind ?? "").trim(),
        active: true,
        event_id: String(event?.event_id ?? "").trim(),
        rule_id: String(event?.rule_id ?? "").trim(),
        utterance_tick: tickId,
      });
    }
    return {
      schedule_id: String(event?.event_id ?? "").trim(),
      event_id: String(event?.event_id ?? "").trim(),
      rule_id: String(event?.rule_id ?? "").trim(),
      predicate_id: predicateId,
      condition_kind: String(event?.condition_kind ?? "").trim(),
      utterance_tick: tickId,
      source_row: "sealed_observation_trace",
      source: "sealed_observation_trace_evaluator",
      detail: event?.detail ?? null,
    };
  });
  return {
    row: rowResult.row,
    active: Array.from(activeById.values()),
    activeSchedule,
    evaluation,
  };
}

function cloneShape(shape) {
  if (!isObject(shape)) return shape;
  const next = { ...shape };
  if (Array.isArray(shape.points)) {
    next.points = shape.points.map((point) => (isObject(point) ? { ...point } : point));
  }
  return next;
}

function shapeMatchesTarget(shape, targets) {
  if (!isObject(shape) || !isObject(targets)) return false;
  const id = String(shape.id ?? "").trim();
  const groupId = String(shape.group_id ?? "").trim();
  const role = String(shape.role ?? "").trim();
  const shapeIds = Array.isArray(targets.shape_ids) ? targets.shape_ids.map((item) => String(item ?? "").trim()) : [];
  const groupIds = Array.isArray(targets.group_ids) ? targets.group_ids.map((item) => String(item ?? "").trim()) : [];
  const roles = Array.isArray(targets.roles) ? targets.roles.map((item) => String(item ?? "").trim()) : [];
  return (id && shapeIds.includes(id)) || (groupId && groupIds.includes(groupId)) || (role && roles.includes(role));
}

function applyHighlight(shape, action) {
  const style = isObject(action?.style) ? action.style : {};
  const width = toFiniteNumber(shape.width, 1.5);
  const widthAdd = toFiniteNumber(style.width_add, 0);
  return {
    ...shape,
    stroke: String(style.stroke ?? shape.stroke ?? shape.color ?? "#f97316"),
    color: String(style.color ?? shape.color ?? style.stroke ?? "#f97316"),
    width: Math.max(1, width + (Number.isFinite(widthAdd) ? widthAdd : 0)),
    opacity: toFiniteNumber(style.opacity, toFiniteNumber(shape.opacity, 1)),
    reaction_highlight: true,
  };
}

function applyVisibility(shape, action) {
  const opacity = toFiniteNumber(action?.opacity, action?.visible === false ? 0 : toFiniteNumber(shape.opacity, 1));
  return {
    ...shape,
    opacity: Math.max(0, Math.min(1, Number.isFinite(opacity) ? opacity : 1)),
    reaction_visible: action?.visible === false ? false : true,
  };
}

function readAnchorPoint(viewPlanRuntime, anchorId) {
  const anchors = Array.isArray(viewPlanRuntime?.views?.view_plan?.anchors)
    ? viewPlanRuntime.views.view_plan.anchors
    : [];
  const row = anchors.find((candidate) => String(candidate?.anchor_id ?? "").trim() === String(anchorId ?? "").trim());
  const value = row?.value;
  const x = toFiniteNumber(value?.x);
  const y = toFiniteNumber(value?.y);
  if (Number.isFinite(x) && Number.isFinite(y)) return { x, y };
  const shapes = Array.isArray(viewPlanRuntime?.views?.space2d?.shapes)
    ? viewPlanRuntime.views.space2d.shapes
    : [];
  const shape = shapes.find((candidate) => {
    const id = String(candidate?.id ?? "").trim();
    const role = String(candidate?.role ?? "").trim();
    return (anchorId === "obs.bob" && (id === "pendulum.bob" || role === "pendulum_bob"))
      || id === anchorId
      || role === anchorId;
  });
  const shapeX = toFiniteNumber(shape?.x ?? shape?.cx);
  const shapeY = toFiniteNumber(shape?.y ?? shape?.cy);
  return Number.isFinite(shapeX) && Number.isFinite(shapeY) ? { x: shapeX, y: shapeY } : null;
}

function buildCameraFocusShape(action, viewPlanRuntime) {
  const anchorId = String(action?.target_anchor ?? "").trim();
  const point = readAnchorPoint(viewPlanRuntime, anchorId);
  if (!point) return null;
  const style = isObject(action?.style) ? action.style : {};
  return {
    id: `trace.reaction.${String(action?.action_id ?? "camera_focus").replace(/[^a-zA-Z0-9._-]+/g, "_")}`,
    kind: "circle",
    role: "trace_camera_focus",
    group_id: "sealed_trace_reaction",
    x: point.x,
    y: point.y,
    r: 0.13,
    stroke: String(style.stroke ?? style.color ?? "#22c55e"),
    color: String(style.color ?? style.stroke ?? "#22c55e"),
    opacity: toFiniteNumber(style.opacity, 0.8),
    style: "dashed",
    reaction_camera_focus: true,
    target_anchor: anchorId,
  };
}

function normalizeActionTargets(action) {
  const targets = isObject(action?.targets) ? action.targets : {};
  const parts = [];
  const pushList = (label, list) => {
    if (!Array.isArray(list)) return;
    const values = list.map((item) => String(item ?? "").trim()).filter(Boolean).sort();
    if (values.length) parts.push(`${label}:${values.join("|")}`);
  };
  pushList("shape", targets.shape_ids);
  pushList("group", targets.group_ids);
  pushList("role", targets.roles);
  const anchor = String(action?.target_anchor ?? "").trim();
  if (anchor) parts.push(`anchor:${anchor}`);
  return parts.length ? parts.join(";") : "target:*";
}

function normalizeActionProperty(action) {
  const kind = String(action?.kind ?? "").trim();
  if (kind === "highlight") return "style";
  if (kind === "visibility") return "visibility";
  if (kind === "camera_focus") return "camera_focus";
  return kind || "unknown";
}

function makeReactionDiagnostic(code, message, extra = {}, severity = "error") {
  return {
    code,
    severity,
    message,
    what: String(extra.what ?? message),
    where: String(extra.where ?? extra.write_key ?? extra.tick_id ?? "trace_reaction_action_resolution"),
    fix: String(extra.fix ?? "같은 대상과 속성을 쓰는 action에는 서로 다른 priority를 명시하거나 하나만 남기세요."),
    ...extra,
  };
}

function normalizeActionWriteKey(action) {
  return `${normalizeActionTargets(action)}::${normalizeActionProperty(action)}`;
}

function resolveActionWrites(actions) {
  const activeActions = Array.isArray(actions) ? actions : [];
  const byWrite = new Map();
  for (const action of activeActions) {
    const actionId = String(action?.action_id ?? "").trim();
    if (!actionId) continue;
    const priority = Math.trunc(toFiniteNumber(action?.priority, 0));
    const writeKey = normalizeActionWriteKey(action);
    const rows = byWrite.get(writeKey) ?? [];
    rows.push({ action, action_id: actionId, priority, write_key: writeKey });
    byWrite.set(writeKey, rows);
  }

  const selected = [];
  const suppressed = [];
  const diagnostics = [];
  for (const [writeKey, rows] of byWrite.entries()) {
    rows.sort((left, right) => {
      const priorityDelta = right.priority - left.priority;
      if (priorityDelta !== 0) return priorityDelta;
      return left.action_id.localeCompare(right.action_id);
    });
    const topPriority = rows[0]?.priority ?? 0;
    const topRows = rows.filter((row) => row.priority === topPriority);
    if (topRows.length > 1) {
      const actionIds = topRows.map((row) => row.action_id).sort();
      diagnostics.push(makeReactionDiagnostic(
        "E_TRACE_REACTION_CONFLICT",
        "같은 대상/속성/우선순위 trace reaction 충돌입니다. 암묵 last-write-wins를 적용하지 않습니다.",
        {
          view_code: "E_VIEW_REACTION_CONFLICT",
          write_key: `${writeKey}::${topPriority}`,
          action_ids: actionIds,
          where: `${writeKey} priority=${topPriority}`,
          fix: "같은 대상과 속성을 쓰는 action 중 하나의 priority를 높이거나 충돌 action을 분리하세요.",
        },
      ));
      for (const row of rows) {
        suppressed.push({
          action_id: row.action_id,
          reason: row.priority === topPriority ? "same_priority_conflict" : "blocked_by_conflict",
          write_key: writeKey,
          priority: row.priority,
          selected_priority: topPriority,
        });
      }
      continue;
    }
    const winner = topRows[0];
    selected.push(winner.action);
    for (const row of rows) {
      if (row.action_id === winner.action_id) continue;
      suppressed.push({
        action_id: row.action_id,
        reason: "lower_priority",
        write_key: writeKey,
        priority: row.priority,
        selected_action_id: winner.action_id,
        selected_priority: winner.priority,
      });
    }
  }
  return {
    selected,
    suppressed: suppressed.sort((left, right) => String(left.action_id).localeCompare(String(right.action_id))),
    diagnostics,
  };
}

function findActionConflicts(actions) {
  const byWrite = new Map();
  for (const action of Array.isArray(actions) ? actions : []) {
    const actionId = String(action?.action_id ?? "").trim();
    if (!actionId) continue;
    const priority = Math.trunc(toFiniteNumber(action?.priority, 0));
    const key = `${normalizeActionWriteKey(action)}::${priority}`;
    const rows = byWrite.get(key) ?? [];
    rows.push(actionId);
    byWrite.set(key, rows);
  }
  const conflicts = [];
  for (const [key, actionIds] of byWrite.entries()) {
    if (actionIds.length <= 1) continue;
    conflicts.push(makeReactionDiagnostic(
      "E_TRACE_REACTION_CONFLICT",
      "같은 대상/속성/우선순위 trace reaction 충돌입니다. 암묵 last-write-wins를 적용하지 않습니다.",
      {
        view_code: "E_VIEW_REACTION_CONFLICT",
        write_key: key,
        action_ids: actionIds,
        where: key,
        fix: "같은 대상과 속성을 쓰는 action 중 하나의 priority를 높이거나 충돌 action을 분리하세요.",
      },
    ));
  }
  return conflicts;
}

function applyActionsToSpace2d(space2d, actions, activePredicateIds, viewPlanRuntime) {
  const applied = [];
  const highlightedShapeIds = new Set();
  const hiddenShapeIds = new Set();
  const cameraFocuses = [];
  let shapes = (Array.isArray(space2d?.shapes) ? space2d.shapes : []).map(cloneShape);
  const activeActions = [];
  for (const action of Array.isArray(actions) ? actions : []) {
    const actionId = String(action?.action_id ?? "").trim();
    const when = String(action?.when ?? "").trim();
    const kind = String(action?.kind ?? "").trim();
    if (!actionId || !kind || (when && !activePredicateIds.has(when))) continue;
    activeActions.push(action);
  }
  const resolvedActions = resolveActionWrites(activeActions);
  for (const action of resolvedActions.selected) {
    const actionId = String(action?.action_id ?? "").trim();
    const when = String(action?.when ?? "").trim();
    const kind = String(action?.kind ?? "").trim();
    if (kind === "highlight") {
      shapes = shapes.map((shape) => {
        if (!shapeMatchesTarget(shape, action.targets)) return shape;
        const next = applyHighlight(shape, action);
        highlightedShapeIds.add(String(next.id ?? next.group_id ?? next.role ?? actionId));
        return next;
      });
      applied.push({ action_id: actionId, kind, priority: Math.trunc(toFiniteNumber(action.priority, 0)), when });
    } else if (kind === "visibility") {
      shapes = shapes.map((shape) => {
        if (!shapeMatchesTarget(shape, action.targets)) return shape;
        const next = applyVisibility(shape, action);
        if (next.reaction_visible === false || Number(next.opacity) <= 0.2) {
          hiddenShapeIds.add(String(next.id ?? next.group_id ?? next.role ?? actionId));
        }
        return next;
      });
      applied.push({ action_id: actionId, kind, priority: Math.trunc(toFiniteNumber(action.priority, 0)), when });
    } else if (kind === "camera_focus") {
      const focusShape = buildCameraFocusShape(action, viewPlanRuntime);
      if (focusShape) {
        shapes = [...shapes, focusShape];
        cameraFocuses.push({
          action_id: actionId,
          target_anchor: String(action.target_anchor ?? "").trim(),
          shape_id: focusShape.id,
          span: action.span ?? null,
        });
        applied.push({ action_id: actionId, kind, priority: Math.trunc(toFiniteNumber(action.priority, 0)), when });
      }
    }
  }
  const diagnostics = resolvedActions.diagnostics;
  return {
    space2d: {
      ...space2d,
      shapes,
      meta: {
        ...(isObject(space2d?.meta) ? space2d.meta : {}),
        view_roles: Array.from(new Set([
          ...((Array.isArray(space2d?.meta?.view_roles) ? space2d.meta.view_roles : []).map((item) => String(item ?? "").trim()).filter(Boolean)),
          "sealed_trace_reaction",
        ])),
        sealed_trace_reaction: true,
      },
    },
    applied,
    highlighted_shape_ids: Array.from(highlightedShapeIds).sort(),
    hidden_shape_ids: Array.from(hiddenShapeIds).sort(),
    camera_focuses: cameraFocuses,
    suppressed_actions: resolvedActions.suppressed,
    diagnostics,
  };
}

function buildResolvedSceneTrace({ traceReactions, activeSchedule, applied, diagnostics, tickId, time }) {
  const selectedActionIds = applied.applied.map((action) => action.action_id).filter(Boolean).sort();
  const selectedActionSet = new Set(selectedActionIds);
  const explicitSuppressedRows = Array.isArray(applied.suppressed_actions) ? applied.suppressed_actions : [];
  const explicitSuppressedSet = new Set(explicitSuppressedRows.map((row) => String(row?.action_id ?? "").trim()).filter(Boolean));
  const rows = (Array.isArray(activeSchedule) ? activeSchedule : []).map((scheduleRow) => ({
    schedule_id: String(scheduleRow.schedule_id ?? "").trim(),
    predicate_id: String(scheduleRow.predicate_id ?? "").trim(),
    condition_kind: String(scheduleRow.condition_kind ?? "").trim(),
    utterance_tick: Math.max(0, Math.trunc(toFiniteNumber(scheduleRow.utterance_tick, tickId))),
    selected_actions: applied.applied
      .filter((action) => String(action.when ?? "").trim() === String(scheduleRow.predicate_id ?? "").trim())
      .map((action) => action.action_id)
      .filter(Boolean)
      .sort(),
    suppressed_actions: explicitSuppressedRows
      .filter((row) => {
        const action = (Array.isArray(traceReactions?.actions) ? traceReactions.actions : [])
          .find((candidate) => String(candidate?.action_id ?? "").trim() === String(row?.action_id ?? "").trim());
        return String(action?.when ?? "").trim() === String(scheduleRow.predicate_id ?? "").trim();
      })
      .map((row) => String(row.action_id ?? "").trim())
      .filter(Boolean)
      .sort(),
  }));
  const inactiveActions = (Array.isArray(traceReactions?.actions) ? traceReactions.actions : [])
    .map((action) => String(action?.action_id ?? "").trim())
    .filter((actionId) => actionId && !selectedActionSet.has(actionId))
    .sort();
  const inactiveRows = (Array.isArray(traceReactions?.actions) ? traceReactions.actions : [])
    .map((action) => String(action?.action_id ?? "").trim())
    .filter((actionId) => actionId && !selectedActionSet.has(actionId) && !explicitSuppressedSet.has(actionId))
    .sort()
    .map((actionId) => ({
      action_id: actionId,
      reason: "inactive_predicate",
    }));
  return {
    schema: "seamgrim.resolved_scene_trace.v1",
    stage_plan_created: false,
    implicit_lww_allowed: false,
    tick_id: tickId,
    time,
    utterance_ticks: Array.from(new Set(rows.map((row) => row.utterance_tick))).sort((a, b) => a - b),
    selected_actions: selectedActionIds,
    suppressed_actions: inactiveActions,
    suppressed_action_rows: [...explicitSuppressedRows, ...inactiveRows],
    rows,
    diagnostics,
  };
}

export function buildSeamgrimTraceReactionRuntime({ traceReactions, observationTrace = null, observation, views, viewPlanRuntime } = {}) {
  if (!isObject(traceReactions) || String(traceReactions.schema ?? "") !== "seamgrim.sealed_trace_reactions.v1") {
    return null;
  }
  const sourceSpace2d = views?.space2d;
  if (!isObject(sourceSpace2d)) return null;
  const { tick_id: tickId, time } = readTickAndTime(observation ?? {});
  const { row, active, activeSchedule, evaluation } = resolveActivePredicates(traceReactions, tickId, time, observationTrace);
  const activePredicateIds = new Set(active.map((predicate) => predicate.predicate_id));
  const applied = applyActionsToSpace2d(sourceSpace2d, traceReactions.actions, activePredicateIds, viewPlanRuntime ?? {});
  const actionKinds = Array.from(new Set(applied.applied.map((action) => action.kind))).sort();
  const predicateKinds = Array.from(new Set(active.map((predicate) => predicate.kind))).sort();
  const conditionKinds = Array.from(new Set([
    ...active.map((predicate) => String(predicate.condition_kind ?? "").trim()).filter(Boolean),
    ...activeSchedule.map((scheduleRow) => String(scheduleRow.condition_kind ?? "").trim()).filter(Boolean),
  ])).sort();
  const diagnostics = [
    ...(Array.isArray(evaluation?.diagnostics) ? evaluation.diagnostics : []),
    ...(Array.isArray(applied.diagnostics) ? applied.diagnostics : []),
  ];
  const resolvedSceneTrace = buildResolvedSceneTrace({
    traceReactions,
    activeSchedule,
    applied,
    diagnostics,
    tickId,
    time,
  });
  const frameHash = buildReactionHashCandidate({
    lesson_id: traceReactions.lesson_id,
    trace_hash: traceReactions.trace_hash,
    observation_trace_hash_candidate: String(observationTrace?.trace_hash_candidate ?? ""),
    predicate_event_hash_candidate: String(evaluation?.evaluation_hash_candidate ?? ""),
    row_id: row?.row_id ?? "",
    tick_id: tickId,
    active_predicates: active.map((predicate) => predicate.predicate_id),
    applied_actions: applied.applied.map((action) => action.action_id),
    suppressed_actions: applied.suppressed_actions,
    condition_kinds: conditionKinds,
    highlighted_shape_ids: applied.highlighted_shape_ids,
    hidden_shape_ids: applied.hidden_shape_ids,
    camera_focuses: applied.camera_focuses.map((focus) => ({ action_id: focus.action_id, target_anchor: focus.target_anchor })),
    resolved_scene_trace: resolvedSceneTrace.rows,
  });
  const report = {
    schema: "seamgrim.trace_reaction.runtime.v1",
    trace_schema: String(traceReactions.schema ?? ""),
    lesson_id: String(traceReactions.lesson_id ?? ""),
    mode: String(traceReactions.mode ?? "sealed_observation_trace"),
    sealed: traceReactions.sealed === true,
    live_online_claim: traceReactions.live_online_claim === true,
    runtime_semantics_claim: traceReactions.runtime_semantics_claim === true,
    condition_evaluation_claim: traceReactions?.predicate_contract?.condition_evaluation_claim === true,
    schedule_kind: String(traceReactions?.predicate_contract?.source ?? "").trim(),
    state_hash_participation: traceReactions.state_hash_participation === true,
    view_plan_hash_participation: traceReactions.view_plan_hash_participation === true,
    trace_hash: String(traceReactions.trace_hash ?? ""),
    observation_trace: isObject(observationTrace) ? observationTrace : null,
    observation_trace_hash_candidate: String(observationTrace?.trace_hash_candidate ?? ""),
    trace_sample_count: Math.max(0, Number(observationTrace?.sample_count ?? 0) || 0),
    trace_provenance: isObject(observationTrace?.provenance) ? observationTrace.provenance : null,
    predicate_events: isObject(evaluation) ? evaluation : null,
    predicate_event_hash_candidate: String(evaluation?.evaluation_hash_candidate ?? ""),
    row_id: String(row?.row_id ?? ""),
    tick_id: tickId,
    time,
    reaction_schedule_rows: activeSchedule,
    active_predicates: active,
    predicate_kinds: predicateKinds,
    condition_kinds: conditionKinds,
    applied_actions: applied.applied,
    action_kinds: actionKinds,
    highlighted_shape_ids: applied.highlighted_shape_ids,
    hidden_shape_ids: applied.hidden_shape_ids,
    hidden_count: applied.hidden_shape_ids.length,
    camera_focuses: applied.camera_focuses,
    diagnostics,
    resolved_scene_trace: resolvedSceneTrace,
    reaction_frame_hash: frameHash,
  };
  return {
    views: {
      ...(isObject(views) ? views : {}),
      space2d: applied.space2d,
      trace_reaction: report,
    },
    report,
  };
}
