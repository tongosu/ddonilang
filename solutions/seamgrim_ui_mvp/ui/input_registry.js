import { normalizeViewFamilyList } from "./view_family_contract.js";

export const INPUT_REGISTRY_SCHEMA = "seamgrim.input_registry.v0";

function normalizeSourceType(raw) {
  const value = String(raw ?? "").trim().toLowerCase();
  if (value === "ddn" || value === "formula" || value === "lesson" || value === "dataset") {
    return value;
  }
  return "ddn";
}

function normalizeId(raw) {
  return String(raw ?? "").trim();
}

function toPlainObject(value) {
  return value && typeof value === "object" ? { ...value } : {};
}

function normalizePayload(type, rawPayload, previousPayload = null) {
  const base = toPlainObject(rawPayload);
  const prev = toPlainObject(previousPayload);
  if (type === "formula") {
    const formulaText =
      String(base.formula_text ?? "").trim() || String(prev.formula_text ?? "").trim();
    const derivedDdn =
      String(base.derived_ddn ?? "").trim() || String(prev.derived_ddn ?? "").trim();
    return {
      ...prev,
      ...base,
      formula_text: formulaText,
      derived_ddn: derivedDdn,
    };
  }
  if (type === "lesson") {
    const requiredViews = normalizeViewFamilyList(
      base.required_views ?? base.requiredViews ?? prev.required_views ?? prev.requiredViews ?? [],
    );
    return {
      ...prev,
      ...base,
      lesson_id:
        String(base.lesson_id ?? "").trim() ||
        String(base.lessonId ?? "").trim() ||
        String(prev.lesson_id ?? "").trim() ||
        String(prev.lessonId ?? "").trim(),
      required_views: requiredViews,
    };
  }
  return {
    ...prev,
    ...base,
  };
}

function normalizeEntry(rawEntry, previousEntry = null) {
  const prev = previousEntry && typeof previousEntry === "object" ? previousEntry : null;
  const id = normalizeId(rawEntry?.id ?? prev?.id ?? "");
  if (!id) return null;
  const type = normalizeSourceType(rawEntry?.type ?? prev?.type ?? "ddn");
  const payload = normalizePayload(type, rawEntry?.payload, prev?.payload);
  const label =
    String(rawEntry?.label ?? "").trim() || String(prev?.label ?? "").trim() || id;
  return {
    id,
    type,
    label,
    payload,
  };
}

export function createInputRegistryState(raw = null) {
  const obj = raw && typeof raw === "object" ? raw : {};
  const registry = Array.isArray(obj.registry) ? obj.registry : [];
  const normalizedRegistry = [];
  registry.forEach((entry) => {
    const normalized = normalizeEntry(entry);
    if (!normalized) return;
    const index = normalizedRegistry.findIndex((item) => item.id === normalized.id);
    if (index >= 0) {
      normalizedRegistry[index] = normalizeEntry(normalized, normalizedRegistry[index]);
      return;
    }
    normalizedRegistry.push(normalized);
  });
  const selectedIdRaw = normalizeId(obj.selectedId ?? obj.selected_id ?? "");
  const selectedId = normalizedRegistry.some((entry) => entry.id === selectedIdRaw)
    ? selectedIdRaw
    : "";
  return {
    registry: normalizedRegistry,
    selectedId,
  };
}

export function upsertInputRegistryItem(state, rawEntry, { select = false } = {}) {
  const base = createInputRegistryState(state);
  const entry = normalizeEntry(rawEntry);
  if (!entry) return base;
  const nextRegistry = [...base.registry];
  const index = nextRegistry.findIndex((item) => item.id === entry.id);
  if (index >= 0) {
    nextRegistry[index] = normalizeEntry(entry, nextRegistry[index]);
  } else {
    nextRegistry.push(entry);
  }
  const selectedId = select
    ? entry.id
    : base.selectedId && nextRegistry.some((item) => item.id === base.selectedId)
      ? base.selectedId
      : "";
  return {
    registry: nextRegistry,
    selectedId,
  };
}

export function selectInputRegistryItem(state, id) {
  const base = createInputRegistryState(state);
  const normalizedId = normalizeId(id);
  if (!normalizedId) {
    return {
      ...base,
      selectedId: "",
    };
  }
  const found = base.registry.some((entry) => entry.id === normalizedId);
  return {
    ...base,
    selectedId: found ? normalizedId : "",
  };
}

export function getSelectedInputRegistryItem(state) {
  const base = createInputRegistryState(state);
  return base.registry.find((entry) => entry.id === base.selectedId) ?? null;
}

export function registerFormulaInput(
  state,
  { id = "formula:default", label = "Formula", formulaText = "", derivedDdn = "" } = {},
) {
  return upsertInputRegistryItem(
    state,
    {
      id,
      type: "formula",
      label,
      payload: {
        formula_text: String(formulaText ?? ""),
        derived_ddn: String(derivedDdn ?? ""),
      },
    },
    { select: true },
  );
}

export function registerLessonInput(
  state,
  {
    id = "",
    lessonId = "",
    label = "",
    requiredViews = [],
    ddnText = "",
  } = {},
) {
  const registryId = normalizeId(id) || (normalizeId(lessonId) ? `lesson:${normalizeId(lessonId)}` : "");
  if (!registryId) return createInputRegistryState(state);
  return upsertInputRegistryItem(
    state,
    {
      id: registryId,
      type: "lesson",
      label: String(label ?? "").trim() || registryId,
      payload: {
        lesson_id: normalizeId(lessonId),
        required_views: normalizeViewFamilyList(requiredViews),
        ddn_text: String(ddnText ?? ""),
      },
    },
    { select: true },
  );
}

export function registerDdnInput(
  state,
  { id = "ddn:default", label = "DDN", ddnText = "", derivedFrom = "" } = {},
) {
  const registryId = normalizeId(id) || "ddn:default";
  return upsertInputRegistryItem(
    state,
    {
      id: registryId,
      type: "ddn",
      label: String(label ?? "").trim() || registryId,
      payload: {
        ddn_text: String(ddnText ?? ""),
        derived_from: String(derivedFrom ?? "").trim(),
      },
    },
    { select: true },
  );
}

export function serializeInputRegistrySession(state) {
  const base = createInputRegistryState(state);
  return {
    schema: INPUT_REGISTRY_SCHEMA,
    inputs: {
      registry: base.registry.map((entry) => ({
        id: entry.id,
        type: entry.type,
        label: entry.label,
        payload: toPlainObject(entry.payload),
      })),
      selected_id: base.selectedId,
    },
  };
}

export function restoreInputRegistrySession(payload) {
  const row = payload && typeof payload === "object" ? payload : {};
  const inputs = row.inputs && typeof row.inputs === "object" ? row.inputs : {};
  return createInputRegistryState({
    registry: Array.isArray(inputs.registry) ? inputs.registry : [],
    selectedId: String(inputs.selected_id ?? ""),
  });
}
