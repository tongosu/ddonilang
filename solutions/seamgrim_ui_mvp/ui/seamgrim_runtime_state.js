import { buildGraphFromValueResources, parsePointsFromValueString } from "./graph_autorender.js";

const STATE_SCHEMA = "seamgrim.state.v0";
const ENGINE_RESPONSE_SCHEMA = "seamgrim.engine_response.v0";
const GRAPH_SCHEMA = "seamgrim.graph.v0";
const SPACE2D_SCHEMA = "seamgrim.space2d.v0";
const TABLE_SCHEMA = "seamgrim.table.v0";
const TEXT_SCHEMA = "seamgrim.text.v0";
const OBSERVATION_MANIFEST_SCHEMA = "ddn.observation_manifest.v0";

function asObject(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) return {};
  return value;
}

function parsePayload(payload) {
  if (typeof payload === "string") {
    return JSON.parse(payload);
  }
  return payload;
}

function normalizeResources(resources) {
  const source = asObject(resources);
  return {
    json: asObject(source.json),
    fixed64: asObject(source.fixed64),
    handle: asObject(source.handle),
    value: asObject(source.value),
  };
}

function normalizeChannels(channels, columnsFallback) {
  const source = Array.isArray(channels)
    ? channels
    : Array.isArray(columnsFallback)
      ? columnsFallback
      : [];
  return source
    .map((entry) => {
      const obj = asObject(entry);
      const key = String(obj.key ?? obj.name ?? "").trim();
      if (!key) return null;
      return {
        key,
        dtype: String(obj.dtype ?? "unknown"),
        role: String(obj.role ?? "state"),
        unit: typeof obj.unit === "string" ? obj.unit : undefined,
      };
    })
    .filter((entry) => entry !== null);
}

function normalizeObservationManifest(manifest) {
  const source = asObject(manifest);
  if (source.schema !== OBSERVATION_MANIFEST_SCHEMA) return null;
  const nodes = Array.isArray(source.nodes)
    ? source.nodes
        .map((entry) => {
          const obj = asObject(entry);
          const name = String(obj.name ?? obj.key ?? "").trim();
          if (!name) return null;
          return {
            name,
            dtype: String(obj.dtype ?? "unknown"),
            role: String(obj.role ?? "상태"),
            unit: typeof obj.unit === "string" ? obj.unit : undefined,
          };
        })
        .filter((entry) => entry !== null)
    : [];
  return {
    schema: OBSERVATION_MANIFEST_SCHEMA,
    version: String(source.version ?? ""),
    nodes,
  };
}

function normalizeRow(row, channelCount) {
  if (!Array.isArray(row)) return [];
  if (!Number.isFinite(channelCount) || channelCount < 0) return row.slice();
  return row.slice(0, channelCount);
}

function parseJsonRaw(raw) {
  if (typeof raw !== "string") return null;
  try {
    return { raw, obj: JSON.parse(raw) };
  } catch (_) {
    return null;
  }
}

function canonicalRaw(value) {
  try {
    return JSON.stringify(value);
  } catch (_) {
    return null;
  }
}

function normalizeStreamOrder(stream) {
  const data = asObject(stream);
  const buffer = Array.isArray(data.buffer) ? data.buffer : [];
  if (!buffer.length) return [];
  const maxLen = buffer.length;
  const lenRaw = Number(data.len);
  const len = Number.isFinite(lenRaw) ? Math.max(0, Math.min(maxLen, Math.floor(lenRaw))) : maxLen;
  if (len <= 0) return [];
  const headRaw = Number(data.head);
  const defaultHead = len - 1;
  const head = Number.isFinite(headRaw)
    ? Math.max(0, Math.min(maxLen - 1, Math.floor(headRaw)))
    : defaultHead;
  const out = [];
  for (let i = 0; i < len; i += 1) {
    const idx = (head - len + 1 + i + maxLen) % maxLen;
    out.push(buffer[idx]);
  }
  return out;
}

function normalizeStreamPoint(value, index) {
  if (typeof value === "number" && Number.isFinite(value)) {
    return { x: index, y: value };
  }
  if (Array.isArray(value) && value.length >= 2) {
    const x = Number(value[0]);
    const y = Number(value[1]);
    if (Number.isFinite(x) && Number.isFinite(y)) return { x, y };
    return null;
  }
  if (value && typeof value === "object") {
    const x = Number(value.x);
    const y = Number(value.y);
    if (Number.isFinite(x) && Number.isFinite(y)) return { x, y };
    const yy = Number(value.value);
    if (Number.isFinite(yy)) return { x: index, y: yy };
    return null;
  }
  if (typeof value === "string") {
    const n = Number(value);
    if (Number.isFinite(n)) return { x: index, y: n };
  }
  return null;
}

function pointsFromStream(stream) {
  const ordered = normalizeStreamOrder(stream);
  const points = [];
  ordered.forEach((item, idx) => {
    const point = normalizeStreamPoint(item, idx);
    if (point) points.push(point);
  });
  return points;
}

function pointsFromResourceTag(state, sourceTag) {
  const valueRaw = state?.resources?.value?.[sourceTag];
  const valuePoints = parsePointsFromValueString(valueRaw);
  if (valuePoints.length) return valuePoints;
  const jsonRaw = state?.resources?.json?.[sourceTag];
  const parsed = parseJsonRaw(jsonRaw);
  if (!parsed) return [];
  const points = Array.isArray(parsed.obj?.points) ? parsed.obj.points : [];
  return points
    .map((point) => normalizeStreamPoint(point, 0))
    .filter((point) => point !== null);
}

function buildGraphFromViewMeta(state) {
  const hints = state?.view_meta?.graph_hints;
  if (!Array.isArray(hints) || hints.length === 0) return null;
  const series = [];
  hints.forEach((hint, index) => {
    const item = asObject(hint);
    const sourceTag = String(item.source ?? item.series_id ?? "").trim();
    if (!sourceTag) return;
    const streamPoints = pointsFromStream(state?.streams?.[sourceTag]);
    const points = streamPoints.length ? streamPoints : pointsFromResourceTag(state, sourceTag);
    if (!points.length) return;
    const id = String(item.series_id ?? sourceTag ?? `series_${index + 1}`);
    const label = String(item.y_label ?? item.label ?? id);
    series.push({ id, label, points });
  });
  if (!series.length) return null;
  return {
    schema: GRAPH_SCHEMA,
    graph_kind: "timeseries",
    series,
    meta: { source: "view-meta" },
  };
}

function buildGraphFromStreams(state) {
  const streams = asObject(state?.streams);
  const keys = Object.keys(streams).sort((a, b) => a.localeCompare(b));
  if (!keys.length) return null;
  const series = [];
  keys.forEach((name) => {
    const points = pointsFromStream(streams[name]);
    if (!points.length) return;
    series.push({ id: name, label: name, points });
  });
  if (!series.length) return null;
  return {
    schema: GRAPH_SCHEMA,
    graph_kind: "timeseries",
    series,
    meta: { source: "streams" },
  };
}

function collectJsonEntries(state, preferPatch = false) {
  const entries = [];
  const pushEntry = (tag, raw, source) => {
    const parsed = parseJsonRaw(raw);
    if (!parsed) return;
    entries.push({ tag, source, raw: parsed.raw, obj: parsed.obj });
  };

  const pushPatchEntries = () => {
    const patch = Array.isArray(state?.patch) ? state.patch : [];
    patch.forEach((op, idx) => {
      if (!op || (op.op !== "set_resource_json" && op.op !== "set_component_json")) return;
      const tag =
        op.op === "set_resource_json"
          ? String(op.tag ?? "")
          : `component:${String(op.entity ?? "?")}:${String(op.tag ?? "?")}`;
      pushEntry(tag, op.value, `patch:${idx}`);
    });
  };

  const pushResourceEntries = () => {
    const jsonResources = asObject(state?.resources?.json);
    Object.entries(jsonResources).forEach(([tag, raw]) => pushEntry(tag, raw, "resources"));
  };

  if (preferPatch) {
    pushPatchEntries();
    pushResourceEntries();
  } else {
    pushResourceEntries();
    pushPatchEntries();
  }
  return entries;
}

function isGraphEmpty(graph) {
  if (!graph || !Array.isArray(graph.series) || !graph.series.length) return true;
  return !graph.series.some((series) => Array.isArray(series?.points) && series.points.length);
}

function pickGraphFromEntries(entries) {
  return entries.find((entry) => entry.obj?.schema === GRAPH_SCHEMA) ?? null;
}

function pickSpace2dFromEntries(entries) {
  return entries.find((entry) => entry.obj?.schema === SPACE2D_SCHEMA) ?? null;
}

function pickTableFromEntries(entries) {
  return (
    entries.find((entry) => entry.obj?.schema === TABLE_SCHEMA) ??
    entries.find((entry) => entry.obj?.matrix) ??
    entries.find((entry) => Array.isArray(entry.obj?.columns) && Array.isArray(entry.obj?.rows)) ??
    null
  );
}

function pickTextFromEntries(entries) {
  return (
    entries.find((entry) => entry.obj?.schema === TEXT_SCHEMA) ??
    entries.find((entry) => typeof entry.obj?.content === "string") ??
    null
  );
}

function pickStructureFromEntries(entries) {
  return (
    entries.find(
      (entry) => Array.isArray(entry.obj?.nodes) && Array.isArray(entry.obj?.edges),
    ) ?? null
  );
}

export function normalizeWasmStatePayload(payload) {
  const parsed = parsePayload(payload);
  if (!parsed || typeof parsed !== "object") {
    throw new Error("Unexpected wasm state payload: object가 아닙니다.");
  }

  if (parsed.schema === STATE_SCHEMA) {
    const channels = normalizeChannels(parsed.channels, parsed.columns);
    const row = normalizeRow(parsed.row, channels.length);
    return {
      ...parsed,
      schema: STATE_SCHEMA,
      resources: normalizeResources(parsed.resources),
      patch: Array.isArray(parsed.patch) ? parsed.patch : [],
      view_meta: asObject(parsed.view_meta),
      streams: asObject(parsed.streams),
      channels,
      row,
      observation_manifest: normalizeObservationManifest(parsed.observation_manifest),
    };
  }

  if (parsed.schema === ENGINE_RESPONSE_SCHEMA) {
    const state = asObject(parsed.state);
    const viewMetaRaw = asObject(parsed.view_meta);
    const viewMeta = { ...viewMetaRaw };
    if (
      viewMeta.space2d &&
      typeof viewMeta.space2d === "object" &&
      Array.isArray(viewMeta.draw_list)
    ) {
      // Stage-A migration: draw_list is kept for compatibility but ignored when space2d exists.
      viewMeta.draw_list_ignored = true;
    }
    const channels = normalizeChannels(
      parsed.channels ?? state.channels,
      parsed.columns ?? state.columns,
    );
    const row = normalizeRow(parsed.row ?? state.row, channels.length);
    const normalized = {
      schema: STATE_SCHEMA,
      tick_id:
        Number.isFinite(parsed.tick_id) ? Number(parsed.tick_id) : Number(state.tick_id ?? 0),
      state_hash: String(parsed.state_hash ?? state.state_hash ?? ""),
      input: asObject(parsed.input ?? state.input),
      resources: normalizeResources(parsed.resources ?? state.resources),
      patch: Array.isArray(parsed.patch)
        ? parsed.patch
        : Array.isArray(state.patch)
          ? state.patch
          : [],
      channels,
      row,
      view_meta: viewMeta,
      streams: asObject(state.streams ?? parsed.streams),
      view_hash: parsed.view_hash ?? null,
      engine_schema: ENGINE_RESPONSE_SCHEMA,
      observation_manifest: normalizeObservationManifest(
        parsed.observation_manifest ?? state.observation_manifest,
      ),
    };
    return normalized;
  }

  throw new Error(`Unexpected schema: ${String(parsed.schema)}`);
}

export function extractObservationChannelsFromState(state) {
  const normalized = normalizeWasmStatePayload(state);
  const baseChannels = Array.isArray(normalized.channels) ? normalized.channels : [];
  const baseRow = Array.isArray(normalized.row) ? normalized.row : [];
  const baseValues = {};
  baseChannels.forEach((channel, index) => {
    const key = String(channel?.key ?? "").trim();
    if (!key) return;
    baseValues[key] = baseRow[index];
  });

  const manifestNodes = Array.isArray(normalized?.observation_manifest?.nodes)
    ? normalized.observation_manifest.nodes
    : [];
  if (!manifestNodes.length) {
    return { channels: baseChannels, row: baseRow, values: baseValues };
  }

  const manifestChannels = manifestNodes.map((node) => ({
    key: String(node.name),
    dtype: String(node.dtype ?? "unknown"),
    role: String(node.role ?? "상태"),
    unit: typeof node.unit === "string" ? node.unit : undefined,
  }));
  const row = manifestChannels.map((channel) => baseValues[channel.key]);
  const values = {};
  manifestChannels.forEach((channel, index) => {
    values[channel.key] = row[index];
  });
  return { channels: manifestChannels, row, values };
}

export function extractStructuredViewsFromState(state, { preferPatch = false } = {}) {
  const normalized = normalizeWasmStatePayload(state);
  const entries = collectJsonEntries(normalized, preferPatch);

  const metaGraph = buildGraphFromViewMeta(normalized);
  const streamGraph = metaGraph ? null : buildGraphFromStreams(normalized);
  const prefixGraph = metaGraph || streamGraph ? null : buildGraphFromValueResources(normalized, preferPatch);
  const entryGraph = metaGraph || streamGraph || prefixGraph ? null : pickGraphFromEntries(entries);

  const metaSpace2d = normalized?.view_meta?.space2d && typeof normalized.view_meta.space2d === "object"
    ? { obj: normalized.view_meta.space2d, raw: canonicalRaw(normalized.view_meta.space2d), source: "view_meta" }
    : null;
  const entrySpace2d = metaSpace2d ? null : pickSpace2dFromEntries(entries);

  const graphEntry = entryGraph;
  let graph = null;
  let graphRaw = null;
  let graphSource = null;
  if (metaGraph) {
    graph = metaGraph;
    graphRaw = canonicalRaw(metaGraph);
    graphSource = "P1:view_meta.graph_hints";
  } else if (streamGraph) {
    graph = streamGraph;
    graphRaw = canonicalRaw(streamGraph);
    graphSource = "P2:state.streams";
  } else if (prefixGraph) {
    graph = prefixGraph;
    graphRaw = canonicalRaw(prefixGraph);
    graphSource = "P3:resources.value.prefix";
  } else if (graphEntry) {
    graph = graphEntry.obj;
    graphRaw = graphEntry.raw;
    graphSource = "P4:json.schema";
  }

  const space2dEntry = metaSpace2d ?? entrySpace2d;
  const space2d = space2dEntry?.obj ?? null;
  const space2dRaw = space2dEntry?.raw ?? null;

  if (graph && space2d && isGraphEmpty(graph)) {
    graph = null;
    graphRaw = null;
    graphSource = null;
  }

  const tableEntry = pickTableFromEntries(entries);
  const textEntry = pickTextFromEntries(entries);
  const structureEntry = pickStructureFromEntries(entries);

  return {
    state: normalized,
    graph,
    graphRaw,
    graphSource,
    space2d,
    space2dRaw,
    table: tableEntry?.obj ?? null,
    tableRaw: tableEntry?.raw ?? null,
    text: textEntry?.obj ?? null,
    textRaw: textEntry?.raw ?? null,
    structure: structureEntry?.obj ?? null,
    structureRaw: structureEntry?.raw ?? null,
  };
}
