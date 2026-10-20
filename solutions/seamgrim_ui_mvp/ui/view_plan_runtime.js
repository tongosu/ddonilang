const EPSILON = 1e-9;

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

export function buildViewHashCandidate(value, label = "view") {
  return `candidate:fnv1a32:${label}:${fnv1a32(stableCanonicalJson(value))}`;
}

function readObservationValue(observation, key) {
  const normalizedKey = String(key ?? "").trim();
  if (!normalizedKey) return NaN;
  const directCandidates = [
    observation?.all_values?.[normalizedKey],
    observation?.values?.[normalizedKey],
    observation?.row?.[normalizedKey],
    observation?.channels?.[normalizedKey]?.value,
  ];
  for (const candidate of directCandidates) {
    const number = toFiniteNumber(candidate);
    if (Number.isFinite(number)) return number;
  }
  return NaN;
}

function resolveAnchorValue(anchor, observation) {
  const sourceRef = anchor?.source_ref;
  if (!isObject(sourceRef)) return { ok: false, value: null };
  const kind = String(sourceRef.kind ?? "").trim();
  if (kind === "observation") {
    const value = readObservationValue(observation, sourceRef.key);
    return { ok: Number.isFinite(value), value };
  }
  if (kind === "constant") {
    return { ok: true, value: sourceRef.value ?? null };
  }
  if (kind === "point2") {
    const x = readObservationValue(observation, sourceRef.x_key);
    const y = readObservationValue(observation, sourceRef.y_key);
    return { ok: Number.isFinite(x) && Number.isFinite(y), value: { x, y } };
  }
  return { ok: false, value: null };
}

function resolveSemanticBindings(viewPlan, observation) {
  const anchors = Array.isArray(viewPlan?.anchors) ? viewPlan.anchors : [];
  const byId = new Map();
  const diagnostics = [];
  for (const anchor of anchors) {
    const anchorId = String(anchor?.anchor_id ?? "").trim();
    if (!anchorId) continue;
    const resolved = resolveAnchorValue(anchor, observation);
    const row = {
      anchor_id: anchorId,
      label: String(anchor?.label ?? anchorId).trim() || anchorId,
      unit: String(anchor?.unit ?? "").trim(),
      value_type: String(anchor?.value_type ?? "").trim(),
      source_ref: anchor?.source_ref ?? null,
      provenance: anchor?.provenance ?? null,
      ok: Boolean(resolved.ok),
      value: resolved.value,
    };
    byId.set(anchorId, row);
    if (!resolved.ok) {
      diagnostics.push({
        code: "W_VIEW_PLAN_ANCHOR_UNRESOLVED",
        severity: "warning",
        message: `${anchorId} 관찰값을 아직 찾지 못했습니다.`,
        anchor_id: anchorId,
      });
    }
  }
  return { anchors: byId, diagnostics };
}

function readPoint(binding) {
  const value = binding?.value;
  if (!isObject(value)) return null;
  const x = toFiniteNumber(value.x);
  const y = toFiniteNumber(value.y);
  return Number.isFinite(x) && Number.isFinite(y) ? { x, y } : null;
}

function vectorLength(x, y) {
  return Math.hypot(Number(x) || 0, Number(y) || 0);
}

function unitVectorFromPoints(from, to, fallback = { x: 0, y: -1 }) {
  const dx = (to?.x ?? 0) - (from?.x ?? 0);
  const dy = (to?.y ?? 0) - (from?.y ?? 0);
  const length = vectorLength(dx, dy);
  if (length <= EPSILON) return fallback;
  return { x: dx / length, y: dy / length };
}

function normalizeAngleSweep(theta) {
  const value = toFiniteNumber(theta, 0);
  return Math.max(-Math.PI * 1.5, Math.min(Math.PI * 1.5, value));
}

function buildArcPolyline({ pivot, radius, theta, id, stroke }) {
  const sweep = normalizeAngleSweep(theta);
  const steps = Math.max(6, Math.min(48, Math.ceil(Math.abs(sweep) / (Math.PI / 32))));
  const points = [];
  const baseAngle = -Math.PI / 2;
  for (let index = 0; index <= steps; index += 1) {
    const ratio = steps === 0 ? 0 : index / steps;
    const angle = baseAngle + sweep * ratio;
    points.push({
      x: pivot.x + Math.cos(angle) * radius,
      y: pivot.y + Math.sin(angle) * radius,
    });
  }
  return {
    id,
    kind: "polyline",
    role: "angle_measure_arc",
    group_id: "measure.angle.theta",
    points,
    stroke,
    width: 2,
  };
}

function formatAngleLabel(theta) {
  const value = toFiniteNumber(theta, 0);
  return `θ ${value.toFixed(3)} rad`;
}

function formatMeasureNumber(value, digits = 3) {
  const number = toFiniteNumber(value, 0);
  const fixed = number.toFixed(digits);
  return fixed.replace(/\.?0+$/u, "");
}

function normalizeMeasureVisibility({ angleMeasureVisible = true, measureVisibility = null } = {}) {
  const source = isObject(measureVisibility) ? measureVisibility : {};
  return {
    angle: angleMeasureVisible !== false && source.angle !== false,
    length: source.length !== false,
    vector: source.vector !== false,
    axes: source.axes !== false,
    trail: source.trail !== false,
  };
}

function findMeasure(viewPlan, kind) {
  const measures = Array.isArray(viewPlan?.measures) ? viewPlan.measures : [];
  return measures.find((item) => String(item?.kind ?? "").trim() === kind) ?? null;
}

function makeMeasureMeta({ measure, kind, label, unit, value = null, objectIds, bounds, accessibilityName }) {
  return {
    measure_id: String(measure?.measure_id ?? `measure.${kind}`),
    kind,
    label: String(label ?? "").trim(),
    unit: String(unit ?? "").trim(),
    value,
    object_ids: Array.isArray(objectIds) ? objectIds : [],
    bounds,
    accessibility_name: String(accessibilityName ?? label ?? kind).trim(),
    state_hash_participation: false,
  };
}

function buildAngleMeasureShapes({ viewPlan, bindings, diagnostics }) {
  const measure = findMeasure(viewPlan, "angle");
  if (!measure) return { shapes: [], meta: null };
  const pivot = readPoint(bindings.get(String(measure.pivot_anchor ?? "").trim()));
  const bob = readPoint(bindings.get(String(measure.end_anchor ?? "").trim()));
  const thetaBinding = bindings.get(String(measure.value_anchor ?? "").trim());
  const theta = toFiniteNumber(thetaBinding?.value, NaN);
  if (!pivot || !bob || !Number.isFinite(theta)) return { shapes: [], meta: null };

  const expectedUnit = String(measure.unit ?? "").trim();
  const actualUnit = String(thetaBinding?.unit ?? "").trim();
  if (expectedUnit && actualUnit && expectedUnit !== actualUnit) {
    diagnostics.push({
      code: String(measure?.diagnostics?.unit_mismatch ?? "E_VIEW_PLAN_UNIT_MISMATCH"),
      severity: "error",
      message: `각도재기 단위가 맞지 않습니다: ${actualUnit} 값을 ${expectedUnit} 눈금에 연결했습니다.`,
      measure_id: String(measure.measure_id ?? ""),
    });
  }

  const style = isObject(measure.style) ? measure.style : {};
  const ref = measure.reference_direction ?? { x: 0, y: -1 };
  const refUnit = unitVectorFromPoints({ x: 0, y: 0 }, { x: ref.x, y: ref.y }, { x: 0, y: -1 });
  const bobUnit = unitVectorFromPoints(pivot, bob, refUnit);
  const length = Math.max(0.25, vectorLength(bob.x - pivot.x, bob.y - pivot.y));
  const baselineEnd = {
    x: pivot.x + refUnit.x * length,
    y: pivot.y + refUnit.y * length,
  };
  const currentEnd = {
    x: pivot.x + bobUnit.x * length,
    y: pivot.y + bobUnit.y * length,
  };
  const arcRadius = Math.max(0.18, Math.min(0.46, length * 0.32));
  const midAngle = -Math.PI / 2 + normalizeAngleSweep(theta) / 2;
  const labelRadius = arcRadius + 0.12;
  const labelPoint = {
    x: pivot.x + Math.cos(midAngle) * labelRadius,
    y: pivot.y + Math.sin(midAngle) * labelRadius,
  };

  const shapes = [
    {
      id: "measure.angle.theta.baseline",
      kind: "line",
      role: "angle_measure_baseline",
      group_id: "measure.angle.theta",
      x1: pivot.x,
      y1: pivot.y,
      x2: baselineEnd.x,
      y2: baselineEnd.y,
      stroke: String(style.baseline_color ?? "#94a3b8"),
      width: 1.4,
    },
    {
      id: "measure.angle.theta.current",
      kind: "line",
      role: "angle_measure_current_line",
      group_id: "measure.angle.theta",
      x1: pivot.x,
      y1: pivot.y,
      x2: currentEnd.x,
      y2: currentEnd.y,
      stroke: String(style.current_color ?? "#0ea5e9"),
      width: 1.6,
    },
    buildArcPolyline({
      pivot,
      radius: arcRadius,
      theta,
      id: "measure.angle.theta.arc",
      stroke: String(style.arc_color ?? "#ef4444"),
    }),
    {
      id: "measure.angle.theta.label",
      kind: "text",
      role: "angle_measure_label",
      group_id: "measure.angle.theta",
      x: labelPoint.x,
      y: labelPoint.y,
      text: formatAngleLabel(theta),
      color: String(style.label_color ?? "#991b1b"),
      size: 12,
    },
  ];
  return {
    shapes,
    meta: {
      measure_id: String(measure.measure_id ?? "measure.angle.theta"),
      kind: "angle",
      pivot,
      bob,
      value: theta,
      unit: expectedUnit || actualUnit,
      label: formatAngleLabel(theta),
      object_ids: shapes.map((shape) => shape.id),
      accessibility_name: `각도재기 ${formatAngleLabel(theta)}`,
      state_hash_participation: false,
      bounds: computeBounds([...shapes, { kind: "point", x: pivot.x, y: pivot.y }, { kind: "point", x: bob.x, y: bob.y }]),
    },
  };
}

function buildLengthMeasureShapes({ viewPlan, bindings }) {
  const measure = findMeasure(viewPlan, "length");
  if (!measure) return { shapes: [], meta: null };
  const start = readPoint(bindings.get(String(measure.start_anchor ?? "").trim()));
  const end = readPoint(bindings.get(String(measure.end_anchor ?? "").trim()));
  if (!start || !end) return { shapes: [], meta: null };
  const style = isObject(measure.style) ? measure.style : {};
  const dx = end.x - start.x;
  const dy = end.y - start.y;
  const length = vectorLength(dx, dy);
  if (length <= EPSILON) return { shapes: [], meta: null };
  const unit = String(measure.unit ?? bindings.get(String(measure.end_anchor ?? "").trim())?.unit ?? "m").trim() || "m";
  const labelText = `${String(measure.label ?? "길이").trim() || "길이"} ${formatMeasureNumber(length)} ${unit}`.trim();
  const dir = { x: dx / length, y: dy / length };
  const normal = { x: -dir.y, y: dir.x };
  const offset = toFiniteNumber(measure.offset, 0.11);
  const tick = toFiniteNumber(measure.tick_size, 0.055);
  const s = { x: start.x + normal.x * offset, y: start.y + normal.y * offset };
  const e = { x: end.x + normal.x * offset, y: end.y + normal.y * offset };
  const mid = { x: (s.x + e.x) / 2 + normal.x * 0.055, y: (s.y + e.y) / 2 + normal.y * 0.055 };
  const shapes = [
    {
      id: "measure.length.rod.line",
      kind: "line",
      role: "length_measure_line",
      group_id: "measure.length.rod",
      x1: s.x,
      y1: s.y,
      x2: e.x,
      y2: e.y,
      stroke: String(style.line_color ?? "#16a34a"),
      width: 1.8,
    },
    {
      id: "measure.length.rod.start_tick",
      kind: "line",
      role: "length_measure_tick",
      group_id: "measure.length.rod",
      x1: s.x - normal.x * tick,
      y1: s.y - normal.y * tick,
      x2: s.x + normal.x * tick,
      y2: s.y + normal.y * tick,
      stroke: String(style.line_color ?? "#16a34a"),
      width: 1.4,
    },
    {
      id: "measure.length.rod.end_tick",
      kind: "line",
      role: "length_measure_tick",
      group_id: "measure.length.rod",
      x1: e.x - normal.x * tick,
      y1: e.y - normal.y * tick,
      x2: e.x + normal.x * tick,
      y2: e.y + normal.y * tick,
      stroke: String(style.line_color ?? "#16a34a"),
      width: 1.4,
    },
    {
      id: "measure.length.rod.label",
      kind: "text",
      role: "length_measure_label",
      group_id: "measure.length.rod",
      x: mid.x,
      y: mid.y,
      text: labelText,
      color: String(style.label_color ?? "#166534"),
      size: 12,
    },
  ];
  return {
    shapes,
    meta: makeMeasureMeta({
      measure,
      kind: "length",
      label: labelText,
      unit,
      value: length,
      objectIds: shapes.map((shape) => shape.id),
      accessibilityName: `길이재기 ${labelText}`,
      bounds: computeBounds([...shapes, { kind: "point", x: start.x, y: start.y }, { kind: "point", x: end.x, y: end.y }]),
    }),
  };
}

function buildVectorMeasureShapes({ viewPlan, bindings }) {
  const measure = findMeasure(viewPlan, "vector");
  if (!measure) return { shapes: [], meta: null };
  const origin = readPoint(bindings.get(String(measure.origin_anchor ?? "").trim()));
  const pivot = readPoint(bindings.get(String(measure.reference_anchor ?? "").trim()));
  const magnitudeBinding = bindings.get(String(measure.magnitude_anchor ?? "").trim());
  const angularVelocityBinding = bindings.get(String(measure.sign_anchor ?? "").trim());
  const magnitude = toFiniteNumber(magnitudeBinding?.value, NaN);
  if (!origin || !pivot || !Number.isFinite(magnitude)) return { shapes: [], meta: null };
  const style = isObject(measure.style) ? measure.style : {};
  const rod = unitVectorFromPoints(pivot, origin, { x: 0, y: -1 });
  const sign = toFiniteNumber(angularVelocityBinding?.value, 1) < 0 ? -1 : 1;
  const tangent = { x: -rod.y * sign, y: rod.x * sign };
  const scale = Math.max(0.16, Math.min(0.62, toFiniteNumber(measure.scale, 0.24) * Math.max(0.8, Math.abs(magnitude))));
  const end = { x: origin.x + tangent.x * scale, y: origin.y + tangent.y * scale };
  const unit = String(measure.unit ?? magnitudeBinding?.unit ?? "").trim();
  const labelText = `${String(measure.label ?? "벡터").trim() || "벡터"} ${formatMeasureNumber(magnitude)} ${unit}`.trim();
  const shapes = [
    {
      id: "measure.vector.velocity.arrow",
      kind: "arrow",
      role: "vector_arrow",
      group_id: "measure.vector.velocity",
      x1: origin.x,
      y1: origin.y,
      x2: end.x,
      y2: end.y,
      stroke: String(style.arrow_color ?? "#7c3aed"),
      width: 2.2,
      label: "",
    },
    {
      id: "measure.vector.velocity.label",
      kind: "text",
      role: "vector_arrow_label",
      group_id: "measure.vector.velocity",
      x: (origin.x + end.x) / 2,
      y: (origin.y + end.y) / 2 + 0.12,
      text: labelText,
      color: String(style.label_color ?? "#5b21b6"),
      size: 12,
    },
  ];
  return {
    shapes,
    meta: makeMeasureMeta({
      measure,
      kind: "vector",
      label: labelText,
      unit,
      value: magnitude,
      objectIds: shapes.map((shape) => shape.id),
      accessibilityName: `벡터 화살표 ${labelText}`,
      bounds: computeBounds(shapes),
    }),
  };
}

function buildAxesTickShapes({ viewPlan, bindings }) {
  const measure = findMeasure(viewPlan, "axes");
  if (!measure) return { shapes: [], meta: null };
  const origin = readPoint(bindings.get(String(measure.origin_anchor ?? "").trim())) ?? { x: 0, y: 0 };
  const range = isObject(measure.range) ? measure.range : {};
  const tickStep = Math.max(0.1, toFiniteNumber(measure.tick_step, 0.5));
  const xMin = toFiniteNumber(range.x_min, -1.2);
  const xMax = toFiniteNumber(range.x_max, 1.2);
  const yMin = toFiniteNumber(range.y_min, -1.2);
  const yMax = toFiniteNumber(range.y_max, 0.3);
  const style = isObject(measure.style) ? measure.style : {};
  const stroke = String(style.axis_color ?? "#334155");
  const tickColor = String(style.tick_color ?? "#64748b");
  const shapes = [
    {
      id: "measure.axes.x_axis",
      kind: "line",
      role: "axis_ticks",
      group_id: "measure.axes",
      x1: xMin,
      y1: origin.y,
      x2: xMax,
      y2: origin.y,
      stroke,
      width: 1.2,
    },
    {
      id: "measure.axes.y_axis",
      kind: "line",
      role: "axis_ticks",
      group_id: "measure.axes",
      x1: origin.x,
      y1: yMin,
      x2: origin.x,
      y2: yMax,
      stroke,
      width: 1.2,
    },
  ];
  const addTick = (axis, value) => {
    const rounded = Number(value.toFixed(6));
    if (axis === "x") {
      shapes.push({
        id: `measure.axes.x_tick.${rounded}`,
        kind: "line",
        role: "axis_tick",
        group_id: "measure.axes",
        x1: rounded,
        y1: origin.y - 0.035,
        x2: rounded,
        y2: origin.y + 0.035,
        stroke: tickColor,
        width: 1,
      });
    } else {
      shapes.push({
        id: `measure.axes.y_tick.${rounded}`,
        kind: "line",
        role: "axis_tick",
        group_id: "measure.axes",
        x1: origin.x - 0.035,
        y1: rounded,
        x2: origin.x + 0.035,
        y2: rounded,
        stroke: tickColor,
        width: 1,
      });
    }
  };
  for (let x = Math.ceil(xMin / tickStep) * tickStep; x <= xMax + EPSILON; x += tickStep) addTick("x", x);
  for (let y = Math.ceil(yMin / tickStep) * tickStep; y <= yMax + EPSILON; y += tickStep) addTick("y", y);
  return {
    shapes,
    meta: makeMeasureMeta({
      measure,
      kind: "axes",
      label: String(measure.label ?? "좌표축·눈금"),
      unit: String(measure.unit ?? "m"),
      value: { x_min: xMin, x_max: xMax, y_min: yMin, y_max: yMax, tick_step: tickStep },
      objectIds: shapes.map((shape) => shape.id),
      accessibilityName: `좌표축 눈금 ${formatMeasureNumber(tickStep)} ${String(measure.unit ?? "m")}`,
      bounds: computeBounds(shapes),
    }),
  };
}

function buildTrajectoryTrailShapes({ viewPlan, bindings }) {
  const measure = findMeasure(viewPlan, "trajectory");
  if (!measure) return { shapes: [], meta: null };
  const pivot = readPoint(bindings.get(String(measure.pivot_anchor ?? "").trim()));
  const bob = readPoint(bindings.get(String(measure.end_anchor ?? "").trim()));
  const theta = toFiniteNumber(bindings.get(String(measure.value_anchor ?? "").trim())?.value, NaN);
  if (!pivot || !bob || !Number.isFinite(theta)) return { shapes: [], meta: null };
  const radius = Math.max(0.1, vectorLength(bob.x - pivot.x, bob.y - pivot.y));
  const sweep = normalizeAngleSweep(theta);
  const minSweep = toFiniteNumber(measure.min_visible_sweep, 0.22);
  const start = Math.min(0, sweep) - (Math.abs(sweep) < minSweep ? minSweep / 2 : 0);
  const end = Math.max(0, sweep) + (Math.abs(sweep) < minSweep ? minSweep / 2 : 0);
  const samples = Math.max(8, Math.min(48, Math.floor(toFiniteNumber(measure.samples, 24))));
  const points = [];
  const baseAngle = -Math.PI / 2;
  for (let index = 0; index <= samples; index += 1) {
    const ratio = samples === 0 ? 0 : index / samples;
    const angle = baseAngle + start + (end - start) * ratio;
    points.push({
      x: pivot.x + Math.cos(angle) * radius,
      y: pivot.y + Math.sin(angle) * radius,
    });
  }
  const style = isObject(measure.style) ? measure.style : {};
  const shapes = [
    {
      id: "measure.trajectory.bob.trail",
      kind: "polyline",
      role: "trajectory_trail",
      group_id: "measure.trajectory.bob",
      points,
      stroke: String(style.trail_color ?? "#f97316"),
      width: 2,
    },
  ];
  return {
    shapes,
    meta: makeMeasureMeta({
      measure,
      kind: "trajectory",
      label: String(measure.label ?? "궤적선"),
      unit: String(measure.unit ?? "m"),
      value: { radius, theta, sample_count: points.length },
      objectIds: shapes.map((shape) => shape.id),
      accessibilityName: `궤적선 ${points.length}점`,
      bounds: computeBounds(shapes),
    }),
  };
}

function computeBounds(shapes) {
  const xs = [];
  const ys = [];
  const add = (x, y) => {
    const nx = toFiniteNumber(x);
    const ny = toFiniteNumber(y);
    if (Number.isFinite(nx) && Number.isFinite(ny)) {
      xs.push(nx);
      ys.push(ny);
    }
  };
  for (const shape of Array.isArray(shapes) ? shapes : []) {
    if (Array.isArray(shape?.points)) {
      shape.points.forEach((point) => add(point?.x, point?.y));
      continue;
    }
    add(shape?.x, shape?.y);
    add(shape?.cx, shape?.cy);
    add(shape?.x1, shape?.y1);
    add(shape?.x2, shape?.y2);
  }
  if (!xs.length || !ys.length) {
    return { x_min: -1, x_max: 1, y_min: -1, y_max: 1 };
  }
  return {
    x_min: Math.min(...xs),
    x_max: Math.max(...xs),
    y_min: Math.min(...ys),
    y_max: Math.max(...ys),
  };
}

function expandBounds(bounds, padding = 0.2, minSpan = 1.0) {
  const xMin = toFiniteNumber(bounds?.x_min, -1);
  const xMax = toFiniteNumber(bounds?.x_max, 1);
  const yMin = toFiniteNumber(bounds?.y_min, -1);
  const yMax = toFiniteNumber(bounds?.y_max, 1);
  const xMid = (xMin + xMax) / 2;
  const yMid = (yMin + yMax) / 2;
  const xSpan = Math.max(minSpan, Math.abs(xMax - xMin) + padding * 2);
  const ySpan = Math.max(minSpan, Math.abs(yMax - yMin) + padding * 2);
  return {
    x_min: xMid - xSpan / 2,
    x_max: xMid + xSpan / 2,
    y_min: yMid - ySpan / 2,
    y_max: yMid + ySpan / 2,
  };
}

function resolveCameraPolicy(viewPlan, policyId, bindings, bounds, diagnostics) {
  const policies = Array.isArray(viewPlan?.camera?.policies) ? viewPlan.camera.policies : [];
  if (policies.length === 0) {
    return {
      policy_id: "",
      mode: "none",
      range: expandBounds(bounds, 0.24, 1.2),
    };
  }
  const defaultPolicy = String(viewPlan?.camera?.default_policy ?? "").trim();
  const requested = String(policyId ?? "").trim() || defaultPolicy || "fit";
  let policy = policies.find((item) => String(item?.policy_id ?? "").trim() === requested) ?? null;
  if (!policy) {
    policy = policies.find((item) => String(item?.policy_id ?? "").trim() === defaultPolicy) ?? policies[0] ?? null;
    diagnostics.push({
      code: String(viewPlan?.camera?.conflict_diagnostic ?? "W_VIEW_CAMERA_POLICY_CONFLICT"),
      severity: "warning",
      message: `카메라 정책 ${requested}을 찾지 못해 기본 정책을 사용했습니다.`,
      requested_policy: requested,
    });
  }
  const mode = String(policy?.mode ?? policy?.policy_id ?? "fit").trim() || "fit";
  if (mode === "fixed" && isObject(policy?.range)) {
    return { policy_id: String(policy.policy_id ?? "fixed"), mode, range: policy.range };
  }
  if (mode === "follow") {
    const target = readPoint(bindings.get(String(policy?.target_anchor ?? "").trim())) ?? { x: 0, y: 0 };
    const span = policy?.span ?? {};
    const xSpan = Math.max(0.5, toFiniteNumber(span.x, 1.8));
    const ySpan = Math.max(0.5, toFiniteNumber(span.y, 1.8));
    return {
      policy_id: String(policy?.policy_id ?? "follow"),
      mode,
      range: {
        x_min: target.x - xSpan / 2,
        x_max: target.x + xSpan / 2,
        y_min: target.y - ySpan / 2,
        y_max: target.y + ySpan / 2,
      },
    };
  }
  return {
    policy_id: String(policy?.policy_id ?? "fit"),
    mode: "fit",
    range: expandBounds(bounds, toFiniteNumber(policy?.padding, 0.24), toFiniteNumber(policy?.min_span, 1.2)),
  };
}

function enrichGraphWithCursor(graph, viewPlan, bindings, observation) {
  const cursor = viewPlan?.graph_cursor;
  if (!isObject(cursor)) return graph;
  const xBinding = bindings.get(String(cursor.x_anchor ?? "").trim());
  const yBinding = bindings.get(String(cursor.y_anchor ?? "").trim());
  const x = toFiniteNumber(xBinding?.value);
  const y = toFiniteNumber(yBinding?.value);
  if (!Number.isFinite(x) || !Number.isFinite(y)) return graph;
  const next = isObject(graph) ? { ...graph } : {};
  next.meta = {
    ...(isObject(next.meta) ? next.meta : {}),
    current_point: {
      cursor_id: String(cursor.cursor_id ?? "graph.current"),
      series_id: String(cursor.series_id ?? ""),
      x,
      y,
      x_unit: String(xBinding?.unit ?? ""),
      y_unit: String(yBinding?.unit ?? ""),
      label: String(cursor.label ?? "현재점"),
      tick_id: toFiniteNumber(observation?.values?.tick_id ?? observation?.tick_id, 0),
    },
  };
  return next;
}

export function buildSeamgrimViewPlanRuntime({
  viewPlan,
  observation,
  views,
  cameraPolicyId,
  angleMeasureVisible = true,
  measureVisibility = null,
} = {}) {
  if (!isObject(viewPlan) || String(viewPlan.schema ?? "") !== "seamgrim.view_plan.v1") {
    return null;
  }
  const visibility = normalizeMeasureVisibility({ angleMeasureVisible, measureVisibility });
  const diagnostics = [];
  const resolved = resolveSemanticBindings(viewPlan, observation ?? {});
  diagnostics.push(...resolved.diagnostics);
  const bindings = resolved.anchors;
  const incomingSpace2d = isObject(views?.space2d) && String(views.space2d.schema ?? "") === "seamgrim.space2d.v0"
    ? views.space2d
    : null;
  const hasAuthoritativeBase = Boolean(
    incomingSpace2d
    && [incomingSpace2d.points, incomingSpace2d.shapes, incomingSpace2d.drawlist].some(Array.isArray),
  );
  const graph = enrichGraphWithCursor(views?.graph ?? null, viewPlan, bindings, observation ?? {});
  if (!hasAuthoritativeBase) {
    const primaryViewTarget = String(viewPlan?.primary_view_target ?? "").trim().toLowerCase();
    const requiresSpace2dBase = primaryViewTarget === "space2d"
      || (primaryViewTarget !== "graph" && (
        (Array.isArray(viewPlan?.measures) && viewPlan.measures.length > 0)
        || isObject(viewPlan?.camera)
      ));
    const viewPlanHash = buildViewHashCandidate({
      viewPlan,
      cameraPolicyId: String(cameraPolicyId ?? ""),
      measureVisibility: visibility,
    }, "view_plan");
    const tickId = Number.isFinite(Number(observation?.values?.tick_id ?? observation?.tick_id))
      ? Number(observation?.values?.tick_id ?? observation?.tick_id)
      : null;
    const presentationMeasureRows = [
      { key: "angle", runtime: buildAngleMeasureShapes({ viewPlan, bindings, diagnostics }) },
      { key: "length", runtime: buildLengthMeasureShapes({ viewPlan, bindings, diagnostics }) },
      { key: "vector", runtime: buildVectorMeasureShapes({ viewPlan, bindings, diagnostics }) },
      { key: "axes", runtime: buildAxesTickShapes({ viewPlan, bindings, diagnostics }) },
      { key: "trail", runtime: buildTrajectoryTrailShapes({ viewPlan, bindings, diagnostics }) },
    ];
    const presentationShapes = presentationMeasureRows
      .filter((row) => visibility[row.key] !== false && row.runtime?.shapes?.length)
      .flatMap((row) => row.runtime.shapes);
    if (requiresSpace2dBase && !presentationShapes.length) {
      diagnostics.push({
        code: "E_VIEW_PLAN_SPACE2D_BASE_REQUIRED_CANDIDATE",
        severity: "warning",
        message: "권위 있는 seamgrim.space2d.v0 base가 없어 view-plan 표시 도형을 만들지 않았습니다.",
      });
    }
    const presentationCamera = presentationShapes.length
      ? resolveCameraPolicy(viewPlan, cameraPolicyId, bindings, computeBounds(presentationShapes), diagnostics)
      : null;
    const presentationSpace2d = presentationShapes.length ? {
      schema: "seamgrim.space2d.v0",
      source: "view_plan_presentation_fallback",
      camera: presentationCamera ? {
        x_min: presentationCamera.range.x_min,
        x_max: presentationCamera.range.x_max,
        y_min: presentationCamera.range.y_min,
        y_max: presentationCamera.range.y_max,
        policy_id: presentationCamera.policy_id,
        mode: presentationCamera.mode,
      } : undefined,
      points: [],
      shapes: presentationShapes,
      meta: {
        authority_class: "PRESENTATION_ONLY",
        base_space2d_status: "no_view",
        canonical_space2d_synthesis: false,
        state_hash_participation: false,
        view_plan_hash: viewPlanHash,
      },
    } : null;
    const viewPlanRuntime = {
      schema: "seamgrim.view_plan.runtime.v1",
      lesson_id: String(viewPlan.lesson_id ?? ""),
      required_anchor_ids: Array.isArray(viewPlan.required_anchor_ids)
        ? viewPlan.required_anchor_ids.map((item) => String(item ?? "").trim()).filter(Boolean)
        : [],
      binding_mode: "explicit",
      state_hash_participation: false,
      presentation_only: true,
      base_space2d_status: requiresSpace2dBase ? "no_view" : "not_required",
      camera_policy: presentationCamera?.policy_id ?? "",
      tick_id: tickId,
      diagnostics,
      anchors: Array.from(bindings.values()),
      graph_current_point: graph?.meta?.current_point ?? null,
      view_plan_hash: viewPlanHash,
      world_draw_payload_hash_candidate: presentationSpace2d
        ? buildViewHashCandidate(presentationShapes, "world_draw_payload")
        : "",
    };
    return {
      views: {
        ...(isObject(views) ? views : {}),
        space2d: presentationSpace2d,
        space2dAuthorityClass: presentationSpace2d ? "PRESENTATION_ONLY" : null,
        space2dPresentationBaseHash: "",
        graph,
        view_plan: viewPlanRuntime,
      },
      report: viewPlanRuntime,
    };
  }
  const angle = buildAngleMeasureShapes({ viewPlan, bindings, diagnostics });
  const length = buildLengthMeasureShapes({ viewPlan, bindings, diagnostics });
  const vector = buildVectorMeasureShapes({ viewPlan, bindings, diagnostics });
  const axes = buildAxesTickShapes({ viewPlan, bindings, diagnostics });
  const trail = buildTrajectoryTrailShapes({ viewPlan, bindings, diagnostics });
  const measureRows = [
    { key: "angle", role: "angle_measure", runtime: angle },
    { key: "length", role: "length_measure", runtime: length },
    { key: "vector", role: "vector_arrow", runtime: vector },
    { key: "axes", role: "axis_ticks", runtime: axes },
    { key: "trail", role: "trajectory_trail", runtime: trail },
  ];
  const visibleMeasureRows = measureRows.filter((row) => visibility[row.key] !== false && row.runtime?.shapes?.length);
  const managedMeasureRows = measureRows.filter((row) => row.runtime?.meta);
  const measureComponents = measureRows
    .map((row) => row.runtime?.meta ? {
      ...row.runtime.meta,
      visible: visibility[row.key] !== false,
      role: row.role,
    } : null)
    .filter(Boolean);
  const incomingShapes = Array.isArray(incomingSpace2d?.shapes) ? incomingSpace2d.shapes : [];
  const preservedWorldShapes = incomingShapes;
  const preservedWorldRoles = (Array.isArray(incomingSpace2d?.meta?.view_roles) ? incomingSpace2d.meta.view_roles : [])
    .map((role) => String(role ?? "").trim())
    .filter(Boolean);
  const shapes = [
    ...preservedWorldShapes,
    ...visibleMeasureRows.flatMap((row) => row.runtime.shapes),
  ];
  const shapeBounds = computeBounds(shapes);
  const camera = resolveCameraPolicy(viewPlan, cameraPolicyId, bindings, shapeBounds, diagnostics);
  const optionalFinite = (value) => Number.isFinite(Number(value)) ? Number(value) : null;
  const tickId = optionalFinite(observation?.values?.tick_id ?? observation?.tick_id);
  const theta = optionalFinite(bindings.get("obs.theta")?.value);
  const omega = optionalFinite(bindings.get("obs.angular_velocity")?.value);
  const time = optionalFinite(bindings.get("obs.time")?.value);
  const shapePayloadForHash = shapes.map((shape) => ({
    ...shape,
    points: Array.isArray(shape.points) ? shape.points.map((point) => ({ x: point.x, y: point.y })) : undefined,
  }));
  const space2d = {
    ...incomingSpace2d,
    schema: "seamgrim.space2d.v0",
    source: "authoritative_structured+view_plan_presentation",
    camera: {
      x_min: camera.range.x_min,
      x_max: camera.range.x_max,
      y_min: camera.range.y_min,
      y_max: camera.range.y_max,
      policy_id: camera.policy_id,
      mode: camera.mode,
    },
    points: Array.isArray(incomingSpace2d?.points) ? incomingSpace2d.points : [],
    shapes,
    drawlist: Array.isArray(incomingSpace2d?.drawlist) ? incomingSpace2d.drawlist : undefined,
    meta: {
      ...(isObject(incomingSpace2d?.meta) ? incomingSpace2d.meta : {}),
      source: "view_plan",
      authority_class: "PRESENTATION_ONLY",
      base_authority_class: "AUTHORITATIVE_STRUCTURED",
      binding_mode: "explicit",
      view_roles: Array.from(new Set([
        ...preservedWorldRoles,
        "semantic_binding",
        ...visibleMeasureRows.map((row) => row.role),
        "same_tick_graph_cursor",
        "camera_policy",
      ].map((role) => String(role ?? "").trim()).filter(Boolean))),
      axes: ["x", "y"],
      camera_policy: camera.policy_id,
      camera_mode: camera.mode,
      state_hash_participation: false,
      tick_id: tickId,
      angle_measure: angle.meta,
      angle_measure_visible: visibility.angle,
      measure_components: measureComponents,
      measure_visibility: visibility,
      graph_current_point: graph?.meta?.current_point ?? null,
      view_plan_hash: buildViewHashCandidate({ viewPlan, cameraPolicyId: camera.policy_id, measureVisibility: visibility }, "view_plan"),
      world_draw_payload_hash_candidate: buildViewHashCandidate(shapePayloadForHash, "world_draw_payload"),
      composed_with_runtime_space2d: true,
      preserved_world_shape_count: preservedWorldShapes.length,
      observation_snapshot: {
        theta,
        angular_velocity: omega,
        time,
      },
    },
  };
  return {
    views: {
      ...(isObject(views) ? views : {}),
      space2d,
      space2dAuthorityClass: "PRESENTATION_ONLY",
      space2dPresentationBaseHash: String(views?.space2dCanonicalPayloadHash ?? ""),
      graph,
      view_plan: {
        schema: "seamgrim.view_plan.runtime.v1",
        lesson_id: String(viewPlan.lesson_id ?? ""),
        required_anchor_ids: Array.isArray(viewPlan.required_anchor_ids)
          ? viewPlan.required_anchor_ids.map((item) => String(item ?? "").trim()).filter(Boolean)
          : [],
        binding_mode: "explicit",
        presentation_only: true,
        base_space2d_status: "available",
        state_hash_participation: false,
        camera_policy: camera.policy_id,
        angle_measure_visible: visibility.angle,
        measure_visibility: visibility,
        diagnostics,
        anchors: Array.from(bindings.values()),
        angle_measure: angle.meta,
        measure_components: measureComponents,
        graph_current_point: graph?.meta?.current_point ?? null,
        view_plan_hash: space2d.meta.view_plan_hash,
        world_draw_payload_hash_candidate: space2d.meta.world_draw_payload_hash_candidate,
        same_tick: {
          tick_id: tickId,
          time,
          theta,
          angular_velocity: omega,
        },
      },
    },
    report: {
      schema: "seamgrim.view_plan.runtime.v1",
      lesson_id: String(viewPlan.lesson_id ?? ""),
      binding_mode: "explicit",
      presentation_only: true,
      base_space2d_status: "available",
      camera_policy: camera.policy_id,
      diagnostics,
      tick_id: tickId,
      theta,
      angular_velocity: omega,
      time,
      angle_label: angle.meta?.label ?? "",
      angle_measure_visible: visibility.angle,
      length_label: length.meta?.label ?? "",
      vector_label: vector.meta?.label ?? "",
      axes_label: axes.meta?.label ?? "",
      trajectory_label: trail.meta?.label ?? "",
      measure_visibility: visibility,
      measure_components: measureComponents,
      measure_component_kinds: measureComponents.map((item) => item.kind),
      graph_current_point: graph?.meta?.current_point ?? null,
      anchors: Array.from(bindings.values()),
      view_plan_hash: space2d.meta.view_plan_hash,
      world_draw_payload_hash_candidate: space2d.meta.world_draw_payload_hash_candidate,
      state_hash_participation: false,
      composed_with_runtime_space2d: true,
      preserved_world_shape_count: preservedWorldShapes.length,
      composed_shape_count: shapes.length,
    },
  };
}
