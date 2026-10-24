const MAEGIM_PLAN_SCHEMA = "ddn.maegim_control_plan.v1";
const ACTION_MAP_SCHEMA = "ddn.input_action_map.v1";
// Owner D12: Alpha1 host time step is one, not a real-time frame interval.

function asFiniteNumber(value, fallback = null) {
  const numeric = Number(value);
  return Number.isFinite(numeric) ? numeric : fallback;
}

function stableJson(value) {
  if (Array.isArray(value)) return `[${value.map(stableJson).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableJson(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function stableHash(text) {
  let hash = 0x811c9dc5;
  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

function normalizeControl(raw, sourcePath) {
  const name = String(raw?.name ?? "").trim();
  // A Rust plan without a verified literal is not a zero-valued control.
  if (raw?.init_expr_canon == null || String(raw.init_expr_canon).trim() === "") return null;
  const value = asFiniteNumber(raw?.init_expr_canon);
  if (!name || value === null) return null;
  const minRaw = asFiniteNumber(raw?.range?.min_expr_canon);
  const maxRaw = asFiniteNumber(raw?.range?.max_expr_canon);
  const stepRaw = asFiniteNumber(raw?.step_expr_canon);
  const splitCount = asFiniteNumber(raw?.split_count_expr_canon);
  const inferredSpan = Math.max(Math.abs(value), 1);
  const min = minRaw !== null && maxRaw !== null ? Math.min(minRaw, maxRaw) : value - inferredSpan;
  const max = minRaw !== null && maxRaw !== null ? Math.max(minRaw, maxRaw) : value + inferredSpan;
  const derivedStep = splitCount !== null && splitCount > 0 && max > min ? (max - min) / splitCount : null;
  const step = stepRaw !== null && stepRaw > 0
    ? stepRaw
    : derivedStep ?? (Math.abs(value) >= 20 ? 1 : Math.abs(value) >= 5 ? 0.5 : 0.1);
  return {
    control_id: `${sourcePath}:${name}`,
    name,
    label: name,
    type_name: String(raw?.type_name ?? "수").trim() || "수",
    value,
    min,
    max,
    step,
    source_path: sourcePath,
    source: "wasm_canon_maegim_plan",
  };
}

export function normalizeMaegimPlans(planRows = []) {
  const controls = [];
  const warnings = [];
  const seen = new Set();
  for (const row of Array.isArray(planRows) ? planRows : []) {
    const sourcePath = String(row?.source_path ?? "lesson.ddn").trim() || "lesson.ddn";
    const plan = row?.plan && typeof row.plan === "object" ? row.plan : null;
    if (!plan || String(plan.schema ?? "") !== MAEGIM_PLAN_SCHEMA) continue;
    for (const raw of Array.isArray(plan.controls) ? plan.controls : []) {
      const control = normalizeControl(raw, sourcePath);
      if (!control || seen.has(control.name)) continue;
      seen.add(control.name);
      controls.push(control);
    }
    for (const warning of Array.isArray(plan.warnings) ? plan.warnings : []) {
      if (!warning || typeof warning !== "object") continue;
      warnings.push({
        code: String(warning.code ?? "W_MAEGIM_PLAN").trim() || "W_MAEGIM_PLAN",
        message: String(warning.message ?? "").trim(),
        source_path: sourcePath,
      });
    }
  }
  return { controls, warnings };
}

function normalizeAction(raw) {
  const actionId = String(raw?.action_id ?? "").trim();
  const snapshot = raw?.input_snapshot && typeof raw.input_snapshot === "object"
    ? raw.input_snapshot
    : null;
  const intent = String(snapshot?.intent ?? "").trim();
  const rawKeys = snapshot?.keys;
  const keys = typeof rawKeys === "number" && Number.isFinite(rawKeys) ? rawKeys : null;
  const lastKey = String(snapshot?.last_key ?? "").trim();
  // Reject conflicting authored values; never silently change an imported input.
  // Variable real-time rates belong to the Alpha2 input contract.
  const dtText = snapshot?.dt_text ?? null;
  if (dtText !== null && (typeof dtText !== "string" || dtText.trim() !== "1")) return null;
  if (snapshot?.dt != null && snapshot.dt !== 1) return null;
  if (!snapshot || !actionId || !intent || keys === null || !Number.isInteger(keys) || keys < 0) return null;
  return {
    action_id: actionId,
    label: String(raw?.label ?? actionId).trim() || actionId,
    glyph: String(raw?.glyph ?? raw?.label ?? actionId).trim() || actionId,
    input_snapshot: {
      intent,
      keys: Math.max(0, Math.trunc(keys)),
      last_key: lastKey,
      pointer_x: asFiniteNumber(snapshot.pointer_x, 0),
      pointer_y: asFiniteNumber(snapshot.pointer_y, 0),
      // Omitted values are bound from the Rust host policy at execution.
      dt: snapshot.dt ?? null,
      dt_text: dtText?.trim() ?? null,
    },
  };
}

export function normalizeActionMap(payload, projectId = "") {
  if (!payload || typeof payload !== "object" || String(payload.schema ?? "") !== ACTION_MAP_SCHEMA) {
    return null;
  }
  const currentProjectId = String(projectId).trim();
  const declaredProjectId = String(payload.project_id ?? "").trim();
  if (currentProjectId && (!declaredProjectId || declaredProjectId !== currentProjectId)) return null;
  const actions = [];
  const seen = new Set();
  for (const raw of Array.isArray(payload.actions) ? payload.actions : []) {
    const action = normalizeAction(raw);
    if (!action || seen.has(action.action_id)) return null;
    seen.add(action.action_id);
    actions.push(action);
  }
  if (!actions.length) return null;
  const normalized = {
    schema: ACTION_MAP_SCHEMA,
    project_id: declaredProjectId || currentProjectId,
    actions,
  };
  return {
    ...normalized,
    action_map_hash: `candidate:fnv1a32:action_map:${stableHash(stableJson(normalized))}`,
  };
}

export function buildProjectControlModel({ projectId = "", planRows = [], actionMap = null } = {}) {
  const { controls, warnings } = normalizeMaegimPlans(planRows);
  const normalizedActionMap = normalizeActionMap(actionMap, projectId);
  const planIdentity = {
    schema: "seamgrim.workbench.project_control_model.v1",
    project_id: String(projectId),
    controls,
    action_map_hash: normalizedActionMap?.action_map_hash ?? "",
  };
  return {
    ...planIdentity,
    status: "ready",
    source: "rust_wasm_authoring_plan",
    controls,
    control_values: Object.fromEntries(controls.map((control) => [control.name, control.value])),
    warnings,
    action_map: normalizedActionMap,
    actions: normalizedActionMap?.actions ?? [],
    authoring_plan_hash: `candidate:fnv1a32:authoring_plan:${stableHash(stableJson(planIdentity))}`,
  };
}

export function emptyProjectControlModel(projectId = "", status = "idle") {
  return {
    schema: "seamgrim.workbench.project_control_model.v1",
    project_id: String(projectId),
    status,
    source: "rust_wasm_authoring_plan",
    controls: [],
    control_values: {},
    warnings: [],
    action_map: null,
    actions: [],
    authoring_plan_hash: "",
    error: null,
  };
}

export function buildSetupRecipe(model, values = {}) {
  const controls = (Array.isArray(model?.controls) ? model.controls : []).map((control) => {
    const value = asFiniteNumber(values?.[control.name], control.value);
    return { name: control.name, value };
  });
  if (!controls.length) return null;
  return {
    schema: "seamgrim.workbench.run_recipe_candidate.v1",
    project_id: String(model?.project_id ?? ""),
    authoring_plan_hash: String(model?.authoring_plan_hash ?? ""),
    source: "wasm_canon_maegim_plan",
    controls,
    values: Object.fromEntries(controls.map((control) => [control.name, control.value])),
  };
}

export function actionById(model, actionId) {
  return (Array.isArray(model?.actions) ? model.actions : [])
    .find((action) => action.action_id === String(actionId)) ?? null;
}
