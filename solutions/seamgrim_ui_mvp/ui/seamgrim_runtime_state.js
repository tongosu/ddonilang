function tryParseJson(raw) {
  if (typeof raw !== "string") return null;
  const text = raw.trim();
  if (!text) return null;
  if (!(text.startsWith("{") || text.startsWith("["))) return null;
  try {
    return JSON.parse(text);
  } catch (_) {
    return null;
  }
}

function normalizeObject(raw) {
  if (!raw || typeof raw !== "object") return {};
  return raw;
}

function normalizePatch(rawPatch) {
  if (!Array.isArray(rawPatch)) return [];
  return rawPatch.filter((op) => op && typeof op === "object");
}

function normalizeResourceMap(rawMap) {
  if (!rawMap || typeof rawMap !== "object") return {};
  return rawMap;
}

export function normalizeWasmStatePayload(payload) {
  const parsed = typeof payload === "string" ? tryParseJson(payload) : payload;
  const obj = normalizeObject(parsed);

  if (obj?.schema === "seamgrim.wasm_state.v2" && obj?.state && typeof obj.state === "object") {
    const state = obj.state;
    return {
      schema: "seamgrim.wasm_state.normalized.v1",
      channels: Array.isArray(state.channels) ? state.channels : [],
      row: Array.isArray(state.row) ? state.row : [],
      patch: normalizePatch(state.patch),
      resources: {
        value: normalizeResourceMap(state?.resources?.value),
        component: normalizeResourceMap(state?.resources?.component),
      },
      streams: normalizeResourceMap(state.streams),
      view_meta: normalizeObject(state.view_meta),
      observation_manifest: normalizeObject(state.observation_manifest),
    };
  }

  return {
    schema: "seamgrim.wasm_state.normalized.v1",
    channels: Array.isArray(obj.channels) ? obj.channels : [],
    row: Array.isArray(obj.row) ? obj.row : [],
    patch: normalizePatch(obj.patch),
    resources: {
      value: normalizeResourceMap(obj?.resources?.value),
      component: normalizeResourceMap(obj?.resources?.component),
    },
    streams: normalizeResourceMap(obj.streams),
    view_meta: normalizeObject(obj.view_meta),
    observation_manifest: normalizeObject(obj.observation_manifest),
  };
}

function normalizeManifestNodes(observationManifest) {
  const nodes = Array.isArray(observationManifest?.nodes) ? observationManifest.nodes : [];
  return nodes
    .map((node) => {
      const name = String(node?.name ?? "").trim();
      if (!name) return null;
      return {
        key: name,
        dtype: String(node?.dtype ?? "unknown"),
        role: String(node?.role ?? "상태"),
        unit: typeof node?.unit === "string" ? node.unit : undefined,
      };
    })
    .filter(Boolean);
}

export function extractObservationChannelsFromState(state) {
  const normalized = normalizeWasmStatePayload(state);
  const baseChannels = Array.isArray(normalized.channels) ? normalized.channels : [];
  const baseRow = Array.isArray(normalized.row) ? normalized.row : [];
  const values = {};

  baseChannels.forEach((channel, index) => {
    const key = String(channel?.key ?? "").trim();
    if (!key) return;
    values[key] = baseRow[index];
  });

  const manifestChannels = normalizeManifestNodes(normalized.observation_manifest);
  if (!manifestChannels.length) {
    return { channels: baseChannels, row: baseRow, values };
  }

  const row = manifestChannels.map((channel) => values[channel.key]);
  const manifestValues = {};
  manifestChannels.forEach((channel, index) => {
    manifestValues[channel.key] = row[index];
  });
  return {
    channels: manifestChannels,
    row,
    values: manifestValues,
  };
}

function parseResourceValue(rawValue) {
  if (rawValue && typeof rawValue === "object") return rawValue;
  return tryParseJson(rawValue);
}

function isGraphObject(obj) {
  return Boolean(obj && typeof obj === "object" && Array.isArray(obj.series));
}

function isSpace2dObject(obj) {
  if (!obj || typeof obj !== "object") return false;
  return Array.isArray(obj.points) || Array.isArray(obj.shapes) || Array.isArray(obj.drawlist);
}

function isTableObject(obj) {
  return Boolean(obj && typeof obj === "object" && Array.isArray(obj.columns) && Array.isArray(obj.rows));
}

function normalizeTextObject(value) {
  if (typeof value === "string") {
    return { markdown: value, text: value };
  }
  if (!value || typeof value !== "object") return null;
  if (typeof value.markdown === "string" || typeof value.text === "string") return value;
  if (Array.isArray(value.lines)) {
    return { ...value, markdown: value.lines.join("\n") };
  }
  return null;
}

function buildGraphFromObservation(normalized) {
  const obs = extractObservationChannelsFromState(normalized);
  const channels = Array.isArray(obs.channels) ? obs.channels : [];
  const values = obs.values ?? {};
  const keys = channels.map((channel) => String(channel?.key ?? "").trim()).filter(Boolean);
  if (!keys.length) return null;

  const timeCandidates = ["t", "time", "시간", "tick", "프레임수"];
  const normalizeKey = (raw) => String(raw ?? "").trim().toLowerCase();
  const timeKey = keys.find((key) => timeCandidates.includes(normalizeKey(key)));
  const yKey = keys.find((key) => key !== timeKey);
  if (!yKey) return null;

  const x = Number(timeKey ? values[timeKey] : 0);
  const y = Number(values[yKey]);
  if (!Number.isFinite(y)) return null;
  return {
    axis: {
      x_min: Number.isFinite(x) ? x - 1 : -1,
      x_max: Number.isFinite(x) ? x + 1 : 1,
      y_min: y - 1,
      y_max: y + 1,
    },
    series: [{ id: yKey, points: [{ x: Number.isFinite(x) ? x : 0, y }] }],
  };
}

function pickStructuredFromPatch(patch, checker) {
  for (let i = patch.length - 1; i >= 0; i -= 1) {
    const op = patch[i];
    if (String(op?.op ?? "") !== "set_resource_value") continue;
    const parsed = parseResourceValue(op?.value);
    if (checker(parsed)) {
      return { obj: parsed, raw: typeof op.value === "string" ? op.value : JSON.stringify(parsed) };
    }
  }
  return null;
}

function pickStructuredFromResources(resources, checker, keyHints = []) {
  const entries = Object.entries(resources ?? {});
  for (const hint of keyHints) {
    const hit = entries.find(([key]) => String(key).toLowerCase().includes(String(hint).toLowerCase()));
    if (!hit) continue;
    const parsed = parseResourceValue(hit[1]);
    if (checker(parsed)) {
      return {
        obj: parsed,
        raw: typeof hit[1] === "string" ? hit[1] : JSON.stringify(parsed),
      };
    }
  }
  for (const [, raw] of entries) {
    const parsed = parseResourceValue(raw);
    if (!checker(parsed)) continue;
    return {
      obj: parsed,
      raw: typeof raw === "string" ? raw : JSON.stringify(parsed),
    };
  }
  return null;
}

export function extractStructuredViewsFromState(state, { preferPatch = false } = {}) {
  const normalized = normalizeWasmStatePayload(state);
  const resources = normalized?.resources?.value ?? {};
  const patch = normalizePatch(normalized.patch);

  const preferMetaGraph = isGraphObject(normalized?.view_meta?.graph) ? normalized.view_meta.graph : null;
  const preferMetaSpace2d = isSpace2dObject(normalized?.view_meta?.space2d) ? normalized.view_meta.space2d : null;

  const graphCandidate = preferMetaGraph
    ? { obj: preferMetaGraph, raw: JSON.stringify(preferMetaGraph), source: "view_meta" }
    : preferPatch
      ? pickStructuredFromPatch(patch, isGraphObject) ?? pickStructuredFromResources(resources, isGraphObject, ["graph"])
      : pickStructuredFromResources(resources, isGraphObject, ["graph"]) ?? pickStructuredFromPatch(patch, isGraphObject);

  const space2dCandidate = preferMetaSpace2d
    ? { obj: preferMetaSpace2d, raw: JSON.stringify(preferMetaSpace2d), source: "view_meta" }
    : preferPatch
      ? pickStructuredFromPatch(patch, isSpace2dObject) ?? pickStructuredFromResources(resources, isSpace2dObject, ["space2d", "2d"])
      : pickStructuredFromResources(resources, isSpace2dObject, ["space2d", "2d"]) ?? pickStructuredFromPatch(patch, isSpace2dObject);

  const tableCandidate = preferPatch
    ? pickStructuredFromPatch(patch, isTableObject) ?? pickStructuredFromResources(resources, isTableObject, ["table"])
    : pickStructuredFromResources(resources, isTableObject, ["table"]) ?? pickStructuredFromPatch(patch, isTableObject);

  const textCandidate = (() => {
    const fromResource = pickStructuredFromResources(resources, (value) => Boolean(normalizeTextObject(value)), ["text", "설명"]);
    if (fromResource) {
      return {
        obj: normalizeTextObject(fromResource.obj),
        raw: fromResource.raw,
      };
    }
    const fromPatch = pickStructuredFromPatch(patch, (value) => Boolean(normalizeTextObject(value)));
    if (fromPatch) {
      return {
        obj: normalizeTextObject(fromPatch.obj),
        raw: fromPatch.raw,
      };
    }
    return null;
  })();

  const fallbackGraph = graphCandidate?.obj ? null : buildGraphFromObservation(normalized);

  return {
    state: normalized,
    graph: graphCandidate?.obj ?? fallbackGraph ?? null,
    graphRaw: graphCandidate?.raw ?? (fallbackGraph ? JSON.stringify(fallbackGraph) : null),
    graphSource: graphCandidate?.source ?? (fallbackGraph ? "observation-fallback" : null),
    space2d: space2dCandidate?.obj ?? null,
    space2dRaw: space2dCandidate?.raw ?? null,
    table: tableCandidate?.obj ?? null,
    tableRaw: tableCandidate?.raw ?? null,
    text: textCandidate?.obj ?? null,
    textRaw: textCandidate?.raw ?? null,
    structure: null,
    structureRaw: null,
  };
}
