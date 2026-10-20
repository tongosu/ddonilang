const REQUIRED_PENDULUM_ANCHORS = Object.freeze([
  "obs.theta",
  "obs.angular_velocity",
  "obs.bob",
  "obs.speed",
  "obs.potential_energy",
  "obs.kinetic_energy",
]);

const ANCHOR_PRESENTATION = Object.freeze({
  "obs.theta": {
    role: "angle",
    graph_key: "theta",
    unit: "rad",
    space_targets: { group_ids: ["measure.angle.theta"] },
    expression_label: "각도재기",
  },
  "obs.angular_velocity": {
    role: "angular_velocity",
    graph_key: "omega",
    unit: "rad/s",
    space_targets: { roles: ["pendulum_bob"] },
    expression_label: "각속도",
  },
  "obs.bob": {
    role: "bob_position",
    graph_key: "",
    unit: "m",
    space_targets: { shape_ids: ["pendulum.bob"], roles: ["pendulum_bob"] },
    expression_label: "추 위치",
  },
  "obs.speed": {
    role: "speed",
    graph_key: "speed",
    unit: "m/s",
    space_targets: { roles: ["pendulum_bob"] },
    expression_label: "속도 크기",
  },
  "obs.potential_energy": {
    role: "potential_energy",
    graph_key: "potential_energy",
    unit: "J/kg",
    space_targets: { shape_ids: ["pendulum.rod"], roles: ["pendulum_bob"] },
    expression_label: "위치에너지",
  },
  "obs.kinetic_energy": {
    role: "kinetic_energy",
    graph_key: "kinetic_energy",
    unit: "J/kg",
    space_targets: { roles: ["pendulum_bob"] },
    expression_label: "운동에너지",
  },
});

function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function toFiniteNumber(value, fallback = NaN) {
  const number = Number(value);
  return Number.isFinite(number) ? number : fallback;
}

function readObservationValue(observation, key) {
  const normalized = String(key ?? "").trim();
  if (!normalized) return NaN;
  const candidates = [
    observation?.all_values?.[normalized],
    observation?.values?.[normalized],
    observation?.row?.[normalized],
    observation?.channels?.[normalized]?.value,
  ];
  for (const candidate of candidates) {
    const number = toFiniteNumber(candidate);
    if (Number.isFinite(number)) return number;
  }
  return NaN;
}

function readPointFromObservation(observation, sourceRef) {
  const x = readObservationValue(observation, sourceRef?.x_key);
  const y = readObservationValue(observation, sourceRef?.y_key);
  return Number.isFinite(x) && Number.isFinite(y) ? { x, y } : null;
}

function makeSemanticDiagnostic(code, message, extra = {}, severity = "warning") {
  return {
    code,
    severity,
    message,
    what: String(extra.what ?? message),
    where: String(extra.where ?? extra.anchor_id ?? "semantic_cursor"),
    fix: String(extra.fix ?? "view_plan anchor에 명시적인 source_ref와 provenance를 추가하세요."),
    ...extra,
  };
}

function readAnchorValue(anchor, observation) {
  const sourceRef = isObject(anchor?.source_ref) ? anchor.source_ref : null;
  if (!sourceRef) return { ok: false, value: null };
  const kind = String(sourceRef.kind ?? "").trim();
  if (kind === "observation") {
    const value = readObservationValue(observation, sourceRef.key);
    return { ok: Number.isFinite(value), value };
  }
  if (kind === "point2") {
    const value = readPointFromObservation(observation, sourceRef);
    return { ok: Boolean(value), value };
  }
  if (kind === "constant") {
    return { ok: true, value: sourceRef.value ?? null };
  }
  return { ok: false, value: null };
}

function formatScalar(value, unit = "") {
  const number = toFiniteNumber(value);
  if (!Number.isFinite(number)) return "-";
  const abs = Math.abs(number);
  const text = abs >= 100 ? number.toFixed(1) : abs >= 10 ? number.toFixed(2) : number.toFixed(3);
  const normalized = text.replace(/\.?0+$/u, "");
  const suffix = String(unit ?? "").trim();
  return suffix ? `${normalized} ${suffix}` : normalized;
}

function formatAnchorValue(value, unit = "") {
  if (isObject(value) && Number.isFinite(Number(value.x)) && Number.isFinite(Number(value.y))) {
    return `(${formatScalar(value.x)}, ${formatScalar(value.y)}) ${String(unit ?? "").trim()}`.trim();
  }
  return formatScalar(value, unit);
}

function normalizeSourceRef(anchor) {
  const sourceRef = isObject(anchor?.source_ref) ? anchor.source_ref : null;
  const provenance = isObject(anchor?.provenance) ? anchor.provenance : null;
  const path = String(provenance?.path ?? "").trim();
  const sourceSpan = String(provenance?.source_span ?? "").trim();
  return {
    source_ref: sourceRef,
    provenance,
    source_available: Boolean(sourceRef && path && sourceSpan),
    source_path: path,
    source_span: sourceSpan,
  };
}

function normalizeTargets(targets = {}) {
  const read = (key) => (Array.isArray(targets?.[key])
    ? targets[key].map((item) => String(item ?? "").trim()).filter(Boolean)
    : []);
  return {
    shape_ids: read("shape_ids"),
    group_ids: read("group_ids"),
    roles: read("roles"),
  };
}

function targetMatches(shape, targets) {
  if (!isObject(shape) || !isObject(targets)) return false;
  const shapeId = String(shape.id ?? "").trim();
  const groupId = String(shape.group_id ?? "").trim();
  const role = String(shape.role ?? "").trim();
  const normalized = normalizeTargets(targets);
  return (shapeId && normalized.shape_ids.includes(shapeId))
    || (groupId && normalized.group_ids.includes(groupId))
    || (role && normalized.roles.includes(role));
}

function buildAnchorRow(anchor, observation) {
  const anchorId = String(anchor?.anchor_id ?? "").trim();
  const declaredPresentation = isObject(anchor?.presentation) ? anchor.presentation : {};
  const presentation = {
    ...(ANCHOR_PRESENTATION[anchorId] ?? {}),
    ...declaredPresentation,
  };
  const resolved = readAnchorValue(anchor, observation);
  const source = normalizeSourceRef(anchor);
  const unit = String(anchor?.unit ?? "").trim() || String(presentation.unit ?? "").trim();
  return {
    anchor_id: anchorId,
    role: String(presentation.role ?? anchor?.value_type ?? "").trim(),
    label: String(anchor?.label ?? anchorId).trim() || anchorId,
    unit,
    value_type: String(anchor?.value_type ?? "").trim(),
    value: resolved.value,
    value_text: resolved.ok ? formatAnchorValue(resolved.value, unit) : "-",
    ok: Boolean(resolved.ok),
    graph_key: String(anchor?.graph_key ?? presentation.graph_key ?? "").trim(),
    space_targets: normalizeTargets(anchor?.space_targets ?? presentation.space_targets ?? {}),
    expression_label: String(presentation.expression_label ?? anchor?.label ?? anchorId).trim(),
    ...source,
  };
}

export function buildSeamgrimSemanticCursorState({
  viewPlanRuntime = null,
  observation = null,
  selectedAnchorId = "",
  selectedOrigin = "",
} = {}) {
  const runtimeViewPlan = viewPlanRuntime?.views?.view_plan ?? viewPlanRuntime?.view_plan ?? null;
  const anchors = Array.isArray(runtimeViewPlan?.anchors) ? runtimeViewPlan.anchors : [];
  const lessonId = String(runtimeViewPlan?.lesson_id ?? "").trim();
  const declaredRequired = Array.isArray(runtimeViewPlan?.required_anchor_ids)
    ? runtimeViewPlan.required_anchor_ids.map((item) => String(item ?? "").trim()).filter(Boolean)
    : [];
  const requiredAnchorIds = declaredRequired.length
    ? declaredRequired
    : lessonId === "rep_phys_pendulum_wave1_v1"
      ? [...REQUIRED_PENDULUM_ANCHORS]
      : anchors.map((anchor) => String(anchor?.anchor_id ?? "").trim()).filter(Boolean);
  const byId = new Map();
  const diagnostics = [];
  for (const anchor of anchors) {
    const anchorId = String(anchor?.anchor_id ?? "").trim();
    if (!anchorId) continue;
    byId.set(anchorId, buildAnchorRow(anchor, observation ?? {}));
  }
  for (const anchorId of requiredAnchorIds) {
    if (!byId.has(anchorId)) {
      diagnostics.push(makeSemanticDiagnostic(
        "E_SEMANTIC_ANCHOR_MISSING",
        `${anchorId} 뜻 anchor가 view_plan에 없습니다.`,
        { anchor_id: anchorId, severity: "error" },
        "error",
      ));
    }
  }
  const rows = anchors
    .map((anchor) => byId.get(String(anchor?.anchor_id ?? "").trim()))
    .filter(Boolean);
  for (const row of rows) {
    if (!row.source_available) {
      diagnostics.push(makeSemanticDiagnostic(
        "W_SEMANTIC_SOURCE_REF_MISSING",
        `${row.anchor_id} anchor는 DDN 원문 위치로 이동할 수 없습니다.`,
        {
          anchor_id: row.anchor_id,
          where: row.anchor_id,
          fix: "변수명 추측 대신 view_plan provenance.path/source_span을 명시하세요.",
        },
      ));
    }
  }
  const requested = String(selectedAnchorId ?? "").trim();
  const selected = byId.get(requested) ?? rows[0] ?? null;
  return {
    schema: "seamgrim.semantic_cursor.v1",
    state_hash_participation: false,
    selected_anchor_id: String(selected?.anchor_id ?? ""),
    selected_origin: String(selectedOrigin ?? "").trim(),
    selected_anchor: selected,
    anchors: rows,
    diagnostics,
  };
}

export function applySemanticCursorToViews({ views = null, cursorState = null } = {}) {
  if (!isObject(views) || !isObject(cursorState?.selected_anchor)) return views;
  const selected = cursorState.selected_anchor;
  const targets = selected.space_targets;
  const space2d = isObject(views.space2d) ? views.space2d : null;
  let nextSpace2d = space2d;
  if (space2d && Array.isArray(space2d.shapes)) {
    const shapes = space2d.shapes.map((shape) => {
      if (!targetMatches(shape, targets)) return shape;
      const width = toFiniteNumber(shape.width, 1.4);
      return {
        ...shape,
        stroke: "#f59e0b",
        color: String(shape.color ?? shape.fill ?? "#f59e0b"),
        width: Math.max(2.4, width + 1.6),
        opacity: 1,
        semantic_cursor_selected: true,
        semantic_anchor_id: selected.anchor_id,
      };
    });
    nextSpace2d = {
      ...space2d,
      shapes,
      meta: {
        ...(isObject(space2d.meta) ? space2d.meta : {}),
        semantic_cursor: {
          anchor_id: selected.anchor_id,
          label: selected.label,
          graph_key: selected.graph_key,
          source_available: selected.source_available,
        },
        state_hash_participation: false,
      },
    };
  }
  const graph = isObject(views.graph) ? {
    ...views.graph,
    meta: {
      ...(isObject(views.graph.meta) ? views.graph.meta : {}),
      semantic_cursor: {
        anchor_id: selected.anchor_id,
        graph_key: selected.graph_key,
        current_point: views.graph?.meta?.current_point ?? null,
      },
    },
  } : views.graph;
  return {
    ...views,
    space2d: nextSpace2d,
    graph,
    semantic_cursor: cursorState,
  };
}

export const __test__ = {
  REQUIRED_PENDULUM_ANCHORS,
  ANCHOR_PRESENTATION,
  formatAnchorValue,
  normalizeTargets,
};
