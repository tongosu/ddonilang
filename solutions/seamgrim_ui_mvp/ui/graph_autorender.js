import { extractStructuredViewsFromState } from "./seamgrim_runtime_state.js";

function readRawState(input) {
  if (input && typeof input === "object" && input.state && typeof input.state === "object") {
    return input.state;
  }
  return input && typeof input === "object" ? input : {};
}

function normalizeResourceMaps(rawState) {
  const resources = rawState?.resources && typeof rawState.resources === "object" ? rawState.resources : {};
  const value = resources?.value && typeof resources.value === "object" ? resources.value : {};
  const valueJson = resources?.value_json && typeof resources.value_json === "object" ? resources.value_json : {};
  return { value, valueJson };
}

function normalizePatchEntries(rawState) {
  if (!Array.isArray(rawState?.patch)) return [];
  return rawState.patch.filter((op) => op && typeof op === "object" && String(op.op ?? "") === "set_resource_value");
}

function looksGraphPrefix(key) {
  const text = String(key ?? "").trim();
  return text.startsWith("그래프_") || text.startsWith("보개_그래프_");
}

function toFiniteNumber(value) {
  const n = Number(value);
  return Number.isFinite(n) ? n : null;
}

function parsePointObject(raw) {
  if (!raw || typeof raw !== "object") return null;
  const x = toFiniteNumber(raw.x);
  const y = toFiniteNumber(raw.y);
  if (x === null || y === null) return null;
  return { x, y };
}

function parsePointArray(raw) {
  if (!Array.isArray(raw)) return [];
  return raw.map((row) => parsePointObject(row)).filter(Boolean);
}

function parseLegacyPointList(text) {
  const src = String(text ?? "").trim();
  const match = src.match(/^차림\s*\[(.*)\]\s*$/su);
  if (!match) return [];
  const body = String(match[1] ?? "");
  const out = [];
  const rowRegex = /짝맞춤\s*\{([^{}]*)\}/gmu;
  let rowMatch = rowRegex.exec(body);
  while (rowMatch) {
    const row = String(rowMatch[1] ?? "");
    const xMatch = row.match(/"x"\s*=>\s*([^,}]+)/u);
    const yMatch = row.match(/"y"\s*=>\s*([^,}]+)/u);
    const x = toFiniteNumber(xMatch?.[1] ?? "");
    const y = toFiniteNumber(yMatch?.[1] ?? "");
    if (x !== null && y !== null) {
      out.push({ x, y });
    }
    rowMatch = rowRegex.exec(body);
  }
  return out;
}

function parseGraphSeriesPoints(raw) {
  if (Array.isArray(raw)) {
    return parsePointArray(raw);
  }
  if (raw && typeof raw === "object") {
    if (Array.isArray(raw.points)) {
      return parsePointArray(raw.points);
    }
    return parsePointArray(raw.series);
  }
  if (typeof raw !== "string") return [];
  const text = raw.trim();
  if (!text) return [];
  if (text.startsWith("{") || text.startsWith("[")) {
    try {
      return parseGraphSeriesPoints(JSON.parse(text));
    } catch (_) {
      // fall through to legacy parser
    }
  }
  return parseLegacyPointList(text);
}

function parseGraphSeriesFromResources(rawState, preferPatch = false) {
  const { value, valueJson } = normalizeResourceMaps(rawState);
  const patchEntries = normalizePatchEntries(rawState);

  const baseRows = [];
  Object.entries(value).forEach(([key, raw]) => {
    baseRows.push({ key, raw });
  });
  Object.entries(valueJson).forEach(([key, raw]) => {
    baseRows.push({ key, raw });
  });

  const patchRows = [];
  patchEntries.forEach((op) => {
    const tag = String(op.tag ?? "").trim();
    if (!tag) return;
    if (Object.prototype.hasOwnProperty.call(op, "value_json")) {
      patchRows.push({ key: tag, raw: op.value_json });
    }
    patchRows.push({ key: tag, raw: op.value });
  });

  const orderedRows = preferPatch ? [...patchRows, ...baseRows] : [...baseRows, ...patchRows];
  const buckets = new Map();
  orderedRows.forEach((row) => {
    const key = String(row?.key ?? "").trim();
    if (!looksGraphPrefix(key)) return;
    const points = parseGraphSeriesPoints(row?.raw);
    if (!points.length) return;
    if (!buckets.has(key)) {
      buckets.set(key, points);
    }
  });

  return Array.from(buckets.entries()).map(([name, points]) => ({ name, points }));
}

export function buildGraphFromValueResources(state, preferPatch = false) {
  const structured = extractStructuredViewsFromState(state, {
    preferPatch: Boolean(preferPatch),
    allowObservationOutputFallback: false,
  });
  if (structured?.graph && typeof structured.graph === "object") {
    return structured.graph;
  }

  const rawState = readRawState(state);
  const series = parseGraphSeriesFromResources(rawState, Boolean(preferPatch));
  if (!series.length) {
    return null;
  }
  return {
    schema: "seamgrim.graph.v0",
    graph_kind: "timeseries",
    series,
    meta: {
      source: "value-prefix",
    },
  };
}
