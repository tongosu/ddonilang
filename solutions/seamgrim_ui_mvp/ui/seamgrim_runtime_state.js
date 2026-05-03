import {
  VIEW_FAMILY_PRIORITY,
  normalizeViewFamily,
  orderViewFamiliesByPriority,
} from "./view_family_contract.js";

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

export function flattenMirrorInputEntries(input, { maxEntries = 20 } = {}) {
  const limit = Math.max(1, Math.trunc(Number(maxEntries) || 20));
  const entries = [];

  function pushEntry(key, value) {
    if (!key || entries.length >= limit) return;
    entries.push([key, value]);
  }

  function walk(value, path) {
    if (entries.length >= limit) return;
    if (value === null || value === undefined) {
      pushEntry(path, value);
      return;
    }
    if (typeof value !== "object") {
      pushEntry(path, value);
      return;
    }
    if (Array.isArray(value)) {
      if (!value.length) {
        pushEntry(path, "[]");
        return;
      }
      value.forEach((item, index) => {
        if (entries.length >= limit) return;
        walk(item, path ? `${path}[${index}]` : `[${index}]`);
      });
      return;
    }
    const objectEntries = Object.entries(value);
    if (!objectEntries.length) {
      pushEntry(path, "{}");
      return;
    }
    objectEntries.forEach(([key, nested]) => {
      if (entries.length >= limit) return;
      walk(nested, path ? `${path}.${key}` : key);
    });
  }

  if (!input || typeof input !== "object") return entries;
  walk(input, "");
  return entries;
}

function readNumber(raw, fallback = 0) {
  const value = Number(raw);
  return Number.isFinite(value) ? value : fallback;
}

function readString(raw, fallback = "") {
  if (typeof raw === "string") return raw;
  if (raw === null || raw === undefined) return fallback;
  return String(raw);
}

export function normalizeWasmStatePayload(payload) {
  const parsed = typeof payload === "string" ? tryParseJson(payload) : payload;
  const obj = normalizeObject(parsed);
  const nestedState = normalizeObject(obj.state);
  const topSchema = readString(obj.schema, "");
  const engineSchema = topSchema === "seamgrim.engine_response.v0"
    ? topSchema
    : readString(obj.engine_schema || nestedState.engine_schema, "");
  const channelsTop = Array.isArray(obj.channels) ? obj.channels : null;
  const channelsNested = Array.isArray(nestedState.channels) ? nestedState.channels : null;
  const channels = channelsTop ?? channelsNested ?? [];
  const rowTop = Array.isArray(obj.row) ? obj.row : null;
  const rowNested = Array.isArray(nestedState.row) ? nestedState.row : null;
  const row = rowTop ?? rowNested ?? [];
  const patch = normalizePatch(nestedState.patch ?? obj.patch);
  const resourcesRaw = normalizeObject(
    (nestedState.resources && typeof nestedState.resources === "object")
      ? nestedState.resources
      : obj.resources,
  );
  const streams = normalizeResourceMap(nestedState.streams ?? obj.streams);
  const viewMeta = normalizeObject(obj.view_meta ?? nestedState.view_meta);
  const observationManifest = normalizeObject(obj.observation_manifest ?? nestedState.observation_manifest);
  const input = normalizeObject(obj.input ?? nestedState.input);
  const tickId = readNumber(obj.tick_id ?? nestedState.tick_id, 0);
  const stateHash = readString(obj.state_hash ?? nestedState.state_hash, "");
  const viewHashRaw = obj.view_hash ?? nestedState.view_hash;
  const viewHash = viewHashRaw === null || viewHashRaw === undefined ? null : readString(viewHashRaw, "");
  const currentlineContext = obj.currentline_context ?? nestedState.currentline_context ?? null;

  return {
    // Keep wrapper/output contract stable for existing wasm packs.
    schema: "seamgrim.state.v0",
    engine_schema: engineSchema,
    tick_id: tickId,
    state_hash: stateHash,
    input,
    channels,
    row,
    patch,
    resources: {
      json: normalizeResourceMap(resourcesRaw.json),
      fixed64: normalizeResourceMap(resourcesRaw.fixed64),
      handle: normalizeResourceMap(resourcesRaw.handle),
      value: normalizeResourceMap(resourcesRaw.value),
      value_json: normalizeResourceMap(resourcesRaw.value_json),
      component: normalizeResourceMap(resourcesRaw.component),
    },
    streams,
    view_meta: viewMeta,
    observation_manifest: observationManifest,
    view_hash: viewHash,
    currentline_context: currentlineContext,
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

function readChannelKey(channel, index) {
  if (typeof channel === "string") {
    return channel.trim();
  }
  if (!channel || typeof channel !== "object") return "";
  const direct = String(channel.key ?? "").trim();
  if (direct) return direct;
  const fallback = String(channel.name ?? channel.id ?? channel.label ?? channel.token ?? "").trim();
  if (fallback) return fallback;
  return "";
}

function normalizeChannels(rawChannels) {
  if (!Array.isArray(rawChannels)) return [];
  return rawChannels
    .map((channel, index) => {
      const key = readChannelKey(channel, index);
      if (!key) return null;
      const dtype =
        channel && typeof channel === "object" ? String(channel.dtype ?? channel.type ?? "unknown") : "unknown";
      const role =
        channel && typeof channel === "object" ? String(channel.role ?? channel.kind ?? "상태") : "상태";
      return { key, dtype, role };
    })
    .filter(Boolean);
}

export function extractObservationChannelsFromState(state) {
  const normalized = normalizeWasmStatePayload(state);
  const baseChannels = normalizeChannels(normalized.channels);
  const baseRow = Array.isArray(normalized.row) ? normalized.row : [];
  const values = {};
  const baseIndexByKey = new Map();

  baseChannels.forEach((channel, index) => {
    const key = String(channel?.key ?? "").trim();
    if (!key) return;
    values[key] = baseRow[index];
    if (!baseIndexByKey.has(key)) {
      baseIndexByKey.set(key, index);
    }
  });

  const manifestChannels = normalizeManifestNodes(normalized.observation_manifest);
  if (!manifestChannels.length) {
    return { channels: baseChannels, row: baseRow, values, all_values: { ...values } };
  }

  const row = manifestChannels.map((channel, index) => {
    if (Object.prototype.hasOwnProperty.call(values, channel.key)) {
      return values[channel.key];
    }
    const hit = baseIndexByKey.get(channel.key);
    if (Number.isInteger(hit)) {
      return baseRow[hit];
    }
    return baseRow[index];
  });
  const manifestValues = {};
  manifestChannels.forEach((channel, index) => {
    manifestValues[channel.key] = row[index];
  });
  const allValues = {
    ...values,
    ...manifestValues,
  };
  return {
    channels: manifestChannels,
    row,
    values: manifestValues,
    all_values: allValues,
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

function normalizeOutputToken(raw) {
  let text = String(raw ?? "").trim();
  if (!text) return "";
  if (
    (text.startsWith('"') && text.endsWith('"')) ||
    (text.startsWith("'") && text.endsWith("'"))
  ) {
    text = text.slice(1, -1).trim();
  }
  return text;
}

function parseLegacyCollectionText(text) {
  const src = String(text ?? "").trim();
  const match = src.match(/^차림\s*\[(.*)\]$/su);
  if (!match) return null;
  const body = String(match[1] ?? "");
  if (!body.trim()) return [];
  const out = [];
  let start = 0;
  let depth = 0;
  let quote = null;
  for (let i = 0; i < body.length; i += 1) {
    const ch = body[i];
    if (quote) {
      if (ch === quote && body[i - 1] !== "\\") {
        quote = null;
      }
      continue;
    }
    if (ch === '"' || ch === "'") {
      quote = ch;
      continue;
    }
    if (ch === "[" || ch === "(" || ch === "{") {
      depth += 1;
      continue;
    }
    if (ch === "]" || ch === ")" || ch === "}") {
      depth = Math.max(0, depth - 1);
      continue;
    }
    if (ch === "," && depth === 0) {
      const token = body.slice(start, i).trim();
      if (token) out.push(token);
      start = i + 1;
    }
  }
  const tail = body.slice(start).trim();
  if (tail) out.push(tail);
  return out;
}

function collectOutputTokens(raw, out, depth = 0) {
  if (depth > 6 || raw === null || raw === undefined) return;
  if (typeof raw === "string") {
    if (raw === "") {
      out.push("");
      return;
    }
    const text = raw.trim();
    if (!text) return;
    const parsed = tryParseJson(text);
    if (parsed !== null && (Array.isArray(parsed) || typeof parsed === "object")) {
      collectOutputTokens(parsed, out, depth + 1);
      return;
    }
    const legacy = parseLegacyCollectionText(text);
    if (legacy !== null) {
      collectOutputTokens(legacy, out, depth + 1);
      return;
    }
    out.push(text);
    return;
  }
  if (typeof raw === "number" || typeof raw === "boolean") {
    out.push(String(raw));
    return;
  }
  if (Array.isArray(raw)) {
    raw.forEach((item) => collectOutputTokens(item, out, depth + 1));
    return;
  }
  if (!raw || typeof raw !== "object") return;
  const preferredFields = ["lines", "items", "values", "list", "buffer", "row", "data", "entries", "content"];
  let hit = false;
  preferredFields.forEach((key) => {
    if (!Object.prototype.hasOwnProperty.call(raw, key)) return;
    hit = true;
    collectOutputTokens(raw[key], out, depth + 1);
  });
  if (hit) return;
  Object.values(raw).forEach((value) => collectOutputTokens(value, out, depth + 1));
}

function parseSpace2dFromOutputLines(rawLines) {
  const lines = Array.isArray(rawLines)
    ? rawLines.map((line) => normalizeOutputToken(line)).filter(Boolean)
    : [];
  if (!lines.length) return null;
  const spaceMarkers = new Set(["space2d", "2d", "공간", "공간2d"]);
  const shapeMarkers = new Set(["space2d.shape", "space2d_shape", "shape2d"]);
  const seriesPrefix = "series:";
  const keySet = new Set([
    "x1",
    "y1",
    "x2",
    "y2",
    "cx",
    "cy",
    "r",
    "x",
    "y",
    "size",
    "stroke",
    "fill",
    "color",
    "width",
    "token",
    "id",
    "name",
    "label",
    "토큰",
    "group_id",
    "group",
    "groupid",
    "그룹",
    "묶음",
  ]);

  function readNumber(text) {
    const num = Number(text);
    if (!Number.isFinite(num)) return null;
    return num;
  }

  function parseShape(startIdx) {
    let idx = startIdx;
    while (idx < lines.length && !lines[idx]) idx += 1;
    if (idx >= lines.length) return { shape: null, next: idx };
    const kindRaw = String(lines[idx] ?? "").trim().toLowerCase();
    idx += 1;
    const data = { kind: kindRaw };
    while (idx < lines.length) {
      const key = String(lines[idx] ?? "").trim();
      if (!key) {
        idx += 1;
        continue;
      }
      const lowerKey = key.toLowerCase();
      if (spaceMarkers.has(lowerKey) || shapeMarkers.has(lowerKey) || lowerKey.startsWith(seriesPrefix)) {
        break;
      }
      if (!keySet.has(lowerKey)) break;
      if (idx + 1 >= lines.length) {
        idx += 1;
        break;
      }
      const valueText = String(lines[idx + 1] ?? "").trim();
      if (
        [
          "stroke",
          "fill",
          "color",
          "token",
          "id",
          "name",
          "label",
          "토큰",
          "group_id",
          "group",
          "groupid",
          "그룹",
          "묶음",
        ].includes(lowerKey)
      ) {
        data[lowerKey] = valueText;
      } else {
        const valueNum = readNumber(valueText);
        if (valueNum !== null) data[lowerKey] = valueNum;
      }
      idx += 2;
    }

    if (["line", "선", "segment"].includes(kindRaw)) {
      if (![data.x1, data.y1, data.x2, data.y2].every((v) => Number.isFinite(v))) {
        return { shape: null, next: idx };
      }
      const groupId =
        data.group_id ??
        data.group ??
        data.groupid ??
        data["그룹"] ??
        data["묶음"];
      return {
        shape: {
          kind: "line",
          x1: Number(data.x1),
          y1: Number(data.y1),
          x2: Number(data.x2),
          y2: Number(data.y2),
          stroke: data.stroke,
          width: data.width,
          token: data.token ?? data["토큰"],
          id: data.id ?? data.name ?? data.label,
          group_id: groupId,
        },
        next: idx,
      };
    }
    if (["circle", "원"].includes(kindRaw)) {
      const cx = Number.isFinite(data.cx) ? data.cx : data.x;
      const cy = Number.isFinite(data.cy) ? data.cy : data.y;
      if (![cx, cy, data.r].every((v) => Number.isFinite(v))) {
        return { shape: null, next: idx };
      }
      const groupId =
        data.group_id ??
        data.group ??
        data.groupid ??
        data["그룹"] ??
        data["묶음"];
      return {
        shape: {
          kind: "circle",
          x: Number(cx),
          y: Number(cy),
          r: Number(data.r),
          stroke: data.stroke,
          fill: data.fill,
          width: data.width,
          token: data.token ?? data["토큰"],
          id: data.id ?? data.name ?? data.label,
          group_id: groupId,
        },
        next: idx,
      };
    }
    if (["point", "점"].includes(kindRaw)) {
      if (![data.x, data.y].every((v) => Number.isFinite(v))) {
        return { shape: null, next: idx };
      }
      const groupId =
        data.group_id ??
        data.group ??
        data.groupid ??
        data["그룹"] ??
        data["묶음"];
      return {
        shape: {
          kind: "point",
          x: Number(data.x),
          y: Number(data.y),
          size: data.size,
          color: data.color,
          stroke: data.stroke,
          token: data.token ?? data["토큰"],
          id: data.id ?? data.name ?? data.label,
          group_id: groupId,
        },
        next: idx,
      };
    }
    return { shape: null, next: idx };
  }

  const points = [];
  const currentShapes = [];
  let latestShapes = [];
  let idx = 0;
  while (idx < lines.length) {
    const token = String(lines[idx] ?? "").trim().toLowerCase();
    if (!token) {
      idx += 1;
      continue;
    }
    if (spaceMarkers.has(token)) {
      if (currentShapes.length) {
        latestShapes = [...currentShapes];
        currentShapes.length = 0;
      }
      const n1 = readNumber(lines[idx + 1]);
      const n2 = readNumber(lines[idx + 2]);
      if (n1 !== null && n2 !== null) {
        points.push({ x: n1, y: n2 });
        idx += 3;
        continue;
      }
      idx += 1;
      continue;
    }
    if (shapeMarkers.has(token)) {
      const parsed = parseShape(idx + 1);
      if (parsed.shape) currentShapes.push(parsed.shape);
      idx = Math.max(parsed.next, idx + 1);
      continue;
    }
    idx += 1;
  }
  if (currentShapes.length) {
    latestShapes = [...currentShapes];
  }
  if (!points.length && !latestShapes.length) return null;
  return {
    schema: "seamgrim.space2d.v0",
    points,
    shapes: latestShapes,
    meta: { source: "observation-output-lines" },
  };
}

function parseTextFromOutputLines(rawLines) {
  const lines = Array.isArray(rawLines)
    ? rawLines.map((line) => normalizeOutputToken(line)).filter(Boolean)
    : [];
  if (!lines.length) return null;

  const markerSet = new Set(["text.overlay", "overlay.text", "subtitle", "자막"]);
  const keySet = new Set(["id", "name", "label", "markdown", "text", "글", "x", "y"]);
  let latest = null;
  let idx = 0;

  while (idx < lines.length) {
    const marker = String(lines[idx] ?? "").trim().toLowerCase();
    if (!markerSet.has(marker)) {
      idx += 1;
      continue;
    }
    idx += 1;
    const parsed = {};
    while (idx < lines.length) {
      const keyRaw = String(lines[idx] ?? "").trim();
      const key = keyRaw.toLowerCase();
      if (!keyRaw) {
        idx += 1;
        continue;
      }
      if (markerSet.has(key)) {
        break;
      }
      if (keySet.has(key) && idx + 1 < lines.length) {
        parsed[key] = String(lines[idx + 1] ?? "").trim();
        idx += 2;
        continue;
      }
      if (!parsed.markdown && !parsed.text && !parsed["글"]) {
        parsed.text = keyRaw;
        idx += 1;
        continue;
      }
      break;
    }

    const markdown = String(parsed.markdown ?? parsed.text ?? parsed["글"] ?? "").trim();
    if (!markdown) continue;
    const x = Number(parsed.x);
    const y = Number(parsed.y);
    latest = {
      markdown,
      text: markdown,
      id: String(parsed.id ?? parsed.name ?? parsed.label ?? "").trim(),
      x: Number.isFinite(x) ? x : undefined,
      y: Number.isFinite(y) ? y : undefined,
      source: "observation-output-lines",
    };
  }
  return latest;
}

function extractOutputLinesFromObservation(normalized) {
  const obs = extractObservationChannelsFromState(normalized);
  const allValues = obs?.all_values && typeof obs.all_values === "object" ? obs.all_values : obs?.values ?? {};
  const resourceValues = normalized?.resources?.value && typeof normalized.resources.value === "object"
    ? normalized.resources.value
    : {};
  const resourceValueJson = normalized?.resources?.value_json && typeof normalized.resources.value_json === "object"
    ? normalized.resources.value_json
    : {};
  const patch = Array.isArray(normalized?.patch) ? normalized.patch : [];

  const rows = [];
  Object.entries(allValues).forEach(([key, raw]) => rows.push({ key, raw, source: "observation" }));
  Object.entries(resourceValues).forEach(([key, raw]) => rows.push({ key, raw, source: "resource" }));
  Object.entries(resourceValueJson).forEach(([key, raw]) => rows.push({ key, raw, source: "resource_value_json" }));
  patch.forEach((op) => {
    if (!op || typeof op !== "object") return;
    if (String(op.op ?? "") !== "set_resource_value") return;
    if (Object.prototype.hasOwnProperty.call(op, "value_json")) {
      rows.push({ key: op.tag, raw: op.value_json, source: "patch_value_json" });
    }
    rows.push({ key: op.tag, raw: op.value, source: "patch" });
  });
  if (!rows.length) return [];

  function isOutputKey(key) {
    const lower = String(key ?? "").trim().toLowerCase();
    if (!lower) return false;
    return (
      lower.includes("보개_출력_줄들") ||
      (lower.includes("output") && lower.includes("line")) ||
      (lower.includes("show") && lower.includes("line"))
    );
  }

  const preferredRows = rows.filter((row) => isOutputKey(row.key));
  const fallbackRows = preferredRows.length ? preferredRows : rows;
  let firstPreferredLines = [];

  for (const row of fallbackRows) {
    const tokens = [];
    collectOutputTokens(row.raw, tokens);
    const lines = tokens.map((token) => normalizeOutputToken(token)).filter(Boolean);
    if (!lines.length) continue;
    if (preferredRows.length > 0 && firstPreferredLines.length === 0) {
      firstPreferredLines = lines;
    }
    const hasKnownHint = lines.some((line) => {
      const lower = line.toLowerCase();
      return (
        lower === "table.row" ||
        lower === "space2d" ||
        lower === "space2d.shape" ||
        lower === "space2d_shape" ||
        lower === "shape2d" ||
        lower === "text.overlay" ||
        lower === "overlay.text" ||
        lower === "subtitle" ||
        lower === "자막"
      );
    });
    if (hasKnownHint) return lines;
  }
  if (firstPreferredLines.length > 0) {
    return firstPreferredLines;
  }
  return [];
}

export function extractObservationOutputLinesFromState(state) {
  const normalized = normalizeWasmStatePayload(state);
  const lines = extractOutputLinesFromObservation(normalized);
  return Array.isArray(lines)
    ? lines.map((line) => normalizeOutputToken(line)).filter(Boolean)
    : [];
}

function readConsoleLogEntries(raw, out, depth = 0) {
  if (depth > 6 || raw === null || raw === undefined) return;
  if (typeof raw === "string") {
    const parsed = tryParseJson(raw);
    if (parsed !== null && (Array.isArray(parsed) || typeof parsed === "object")) {
      readConsoleLogEntries(parsed, out, depth + 1);
    }
    return;
  }
  if (Array.isArray(raw)) {
    raw.forEach((item) => readConsoleLogEntries(item, out, depth + 1));
    return;
  }
  if (!raw || typeof raw !== "object") return;
  const hasText = Object.prototype.hasOwnProperty.call(raw, "text")
    || Object.prototype.hasOwnProperty.call(raw, "value")
    || Object.prototype.hasOwnProperty.call(raw, "message");
  const text = String(raw.text ?? raw.value ?? raw.message ?? "");
  if (!text && !hasText) return;
  const tickValue = Number(raw.tick ?? raw.madi ?? raw.tick_id);
  const lineNoValue = Number(raw.line_no ?? raw.lineNo ?? raw.index ?? raw.order);
  const kindText = String(raw.kind ?? raw.level ?? "output").trim().toLowerCase();
  out.push({
    tick: Number.isFinite(tickValue) ? Math.max(0, Math.trunc(tickValue)) : 0,
    line_no: Number.isFinite(lineNoValue) && lineNoValue > 0 ? Math.trunc(lineNoValue) : null,
    text,
    kind: ["warn", "error", "output"].includes(kindText) ? kindText : "output",
  });
}

function extractDirectConsoleLog(normalized, state) {
  const candidates = [
    state?.console_log,
    state?.output_log,
    state?.runtime?.console_log,
    state?.runtime?.output_log,
    normalized?.resources?.value_json?.console_log,
    normalized?.resources?.value_json?.output_log,
    normalized?.resources?.json?.console_log,
    normalized?.resources?.json?.output_log,
  ];
  const out = [];
  candidates.forEach((candidate) => readConsoleLogEntries(candidate, out));
  if (!out.length) return [];
  return out
    .map((entry, index) => ({
      tick: Number.isFinite(entry.tick) ? entry.tick : normalized.tick_id,
      line_no: Number.isFinite(entry.line_no) && entry.line_no > 0 ? entry.line_no : index + 1,
      text: String(entry.text ?? ""),
      kind: ["warn", "error", "output"].includes(String(entry.kind ?? "").trim().toLowerCase())
        ? String(entry.kind).trim().toLowerCase()
        : "output",
    }));
}

function stripStructuredViewOutputLines(rawLines) {
  const lines = Array.isArray(rawLines)
    ? rawLines.map((line) => normalizeOutputToken(line)).filter(Boolean)
    : [];
  if (!lines.length) return [];

  const spaceMarkers = new Set(["space2d", "2d", "공간", "공간2d"]);
  const shapeMarkers = new Set(["space2d.shape", "space2d_shape", "shape2d"]);
  const textMarkers = new Set(["text.overlay", "overlay.text", "subtitle", "자막"]);
  const shapeKeySet = new Set([
    "x1",
    "y1",
    "x2",
    "y2",
    "cx",
    "cy",
    "r",
    "x",
    "y",
    "size",
    "stroke",
    "fill",
    "color",
    "width",
    "token",
    "id",
    "name",
    "label",
    "토큰",
    "group_id",
    "group",
    "groupid",
    "그룹",
    "묶음",
  ]);
  const textKeySet = new Set(["id", "name", "label", "markdown", "text", "글", "x", "y"]);
  const out = [];
  let idx = 0;
  while (idx < lines.length) {
    const line = String(lines[idx] ?? "").trim();
    const lower = line.toLowerCase();
    if (spaceMarkers.has(lower)) {
      const n1 = Number(lines[idx + 1]);
      const n2 = Number(lines[idx + 2]);
      idx += Number.isFinite(n1) && Number.isFinite(n2) ? 3 : 1;
      continue;
    }
    if (shapeMarkers.has(lower)) {
      idx += 1;
      if (idx < lines.length) idx += 1; // shape kind
      while (idx < lines.length) {
        const key = String(lines[idx] ?? "").trim();
        const keyLower = key.toLowerCase();
        if (!key) {
          idx += 1;
          continue;
        }
        if (spaceMarkers.has(keyLower) || shapeMarkers.has(keyLower) || textMarkers.has(keyLower) || keyLower === "table.row") {
          break;
        }
        if (!shapeKeySet.has(keyLower)) break;
        idx += 2;
      }
      continue;
    }
    if (textMarkers.has(lower)) {
      idx += 1;
      while (idx < lines.length) {
        const key = String(lines[idx] ?? "").trim();
        const keyLower = key.toLowerCase();
        if (!key) {
          idx += 1;
          continue;
        }
        if (spaceMarkers.has(keyLower) || shapeMarkers.has(keyLower) || textMarkers.has(keyLower) || keyLower === "table.row") {
          break;
        }
        if (!textKeySet.has(keyLower)) break;
        idx += 2;
      }
      continue;
    }
    out.push(line);
    idx += 1;
  }
  return out;
}

export function extractObservationOutputLogFromState(state) {
  const normalized = normalizeWasmStatePayload(state);
  const direct = extractDirectConsoleLog(normalized, state && typeof state === "object" ? state : {});
  if (direct.length > 0) {
    return direct;
  }
  const lines = extractOutputLinesFromObservation(normalized);
  if (!Array.isArray(lines) || !lines.length) return [];
  const structuredOnlyMarkers = new Set([
    "table.row",
    "space2d",
    "space2d.shape",
    "space2d_shape",
    "shape2d",
    "text.overlay",
    "overlay.text",
    "subtitle",
    "자막",
  ]);
  if (lines.some((line) => String(line ?? "").trim().toLowerCase() === "table.row")) {
    return [];
  }
  const nonStructuredLines = stripStructuredViewOutputLines(lines);
  if (nonStructuredLines.length === 0) {
    return [];
  }
  const filtered = nonStructuredLines
    .filter((line) => line.length > 0 && !structuredOnlyMarkers.has(line.toLowerCase()));
  return filtered.map((text, index) => ({
    tick: Number.isFinite(normalized.tick_id) ? normalized.tick_id : 0,
    line_no: index + 1,
    text,
    kind: "output",
  }));
}

export function extractObservationOutputRowsFromState(state) {
  const normalized = normalizeWasmStatePayload(state);
  const lines = extractOutputLinesFromObservation(normalized);
  const normalizedLines = Array.isArray(lines) ? lines : [];

  const rows = [];
  let i = 0;
  while (i < normalizedLines.length) {
    const marker = String(normalizedLines[i] ?? "").trim().toLowerCase();
    if (marker !== "table.row") {
      i += 1;
      continue;
    }
    const rowTokens = [];
    let j = i + 1;
    while (j < normalizedLines.length) {
      const token = String(normalizedLines[j] ?? "").trim();
      if (!token) {
        j += 1;
        continue;
      }
      if (token.toLowerCase() === "table.row") {
        break;
      }
      rowTokens.push(token);
      j += 1;
    }
    if (rowTokens.length < 2) {
      i += 1;
      continue;
    }

    let key = "";
    let valueToken = "";
    if (rowTokens.length >= 4) {
      key = String(rowTokens[1] ?? "").trim();
      valueToken = String(rowTokens[2] ?? "").trim();
    } else {
      key = String(rowTokens[0] ?? "").trim();
      valueToken = String(rowTokens[1] ?? "").trim();
    }
    if (!key || !valueToken) {
      i = j;
      continue;
    }
    if (valueToken.toLowerCase() === "table.row" || key.toLowerCase() === "table.row") {
      i = j;
      continue;
    }
    if (!key) {
      i += 1;
      continue;
    }
    rows.push({
      key,
      value: valueToken,
      source: "table.row",
      syntheticKey: false,
    });
    i = j;
  }
  if (rows.length > 0) {
    return rows;
  }

  const structuredOnlyMarkers = new Set([
    "space2d",
    "space2d.shape",
    "space2d_shape",
    "shape2d",
    "text.overlay",
    "overlay.text",
    "subtitle",
    "자막",
  ]);
  const hasStructuredMarkers = normalizedLines.some((token) => structuredOnlyMarkers.has(String(token ?? "").trim().toLowerCase()));

  // Legacy/compat output fallback:
  // `보여주기`가 table.row 마커 없이 값 리스트만 남기는 경우에도
  // 하위 패널과 fallback 보개가 값을 읽을 수 있도록 행으로 승격한다.
  const observation = extractObservationChannelsFromState(normalized);
  const channelKeys = Array.isArray(observation?.channels)
    ? observation.channels
        .map((channel) => readChannelKey(channel))
        .map((key) => String(key ?? "").trim())
        .filter((key) => {
          if (!key || key.startsWith("__")) return false;
          const lower = key.toLowerCase();
          if (
            lower.includes("보개_출력_줄들") ||
            (lower.includes("output") && lower.includes("line")) ||
            (lower.includes("show") && lower.includes("line"))
          ) {
            return false;
          }
          return true;
        })
    : [];
  const preferredOrder = ["t", "y", "theta", "omega", "x", "n", "time", "tick", "값", "value"];
  const rankedKeys = channelKeys
    .map((key, index) => {
      const lower = key.toLowerCase();
      const prefIndex = preferredOrder.findIndex((token) => lower === token || lower.includes(token));
      return {
        key,
        rank: prefIndex >= 0 ? prefIndex : (100 + index),
      };
    })
    .sort((a, b) => a.rank - b.rank)
    .map((row) => row.key);
  const observationValues = observation?.all_values && typeof observation.all_values === "object"
    ? observation.all_values
    : observation?.values && typeof observation.values === "object"
      ? observation.values
      : {};
  const observationRows = Object.entries(observationValues)
    .map(([key, raw]) => {
      const keyText = String(key ?? "").trim();
      if (!keyText || keyText.startsWith("__")) return null;
      const lower = keyText.toLowerCase();
      if (
        lower.includes("보개_출력_줄들") ||
        (lower.includes("output") && lower.includes("line")) ||
        (lower.includes("show") && lower.includes("line"))
      ) {
        return null;
      }
      if (raw === null || raw === undefined) return null;
      if (typeof raw === "object") return null;
      const value = String(raw).trim();
      if (!value) return null;
      const rankBase = rankedKeys.indexOf(keyText);
      const rank = rankBase >= 0 ? rankBase : 200;
      return { key: keyText, value, rank };
    })
    .filter(Boolean)
    .sort((a, b) => a.rank - b.rank)
    .map(({ key, value }) => ({
      key,
      value,
      source: "observation",
      syntheticKey: false,
    }));
  if (hasStructuredMarkers && observationRows.length > 0) {
    return observationRows;
  }

  const markerFilteredLines = normalizedLines.filter((token) => !structuredOnlyMarkers.has(String(token ?? "").trim().toLowerCase()));
  // 보여주기 출력 줄이 전혀 없지만 observation 채널 값이 있으면 그 값을 fallback 행으로 반환.
  // (단순 값 계산 DDN에서 보여주기만 사용했을 때 흰 격자를 방지하고 콘솔 보개를 활성화한다.)
  if (markerFilteredLines.length === 0 && observationRows.length > 0) {
    return observationRows.map((row) => ({ ...row, source: "fallback-line" }));
  }
  const fallbackLines = markerFilteredLines.length > 0 ? markerFilteredLines : normalizedLines;
  return fallbackLines
    .map((token, index) => {
      const key = rankedKeys[index] ?? `출력${index + 1}`;
      return {
        key,
        value: String(token ?? "").trim(),
        source: "fallback-line",
        syntheticKey: !rankedKeys[index],
      };
    })
    .filter((row) => row.value.length > 0);
}

function isTableObject(obj) {
  return Boolean(obj && typeof obj === "object" && Array.isArray(obj.columns) && Array.isArray(obj.rows));
}

function isStructureObject(obj) {
  return Boolean(obj && typeof obj === "object" && Array.isArray(obj.nodes) && Array.isArray(obj.edges));
}

export function summarizeStructureView(value, { sampleLimit = 3 } = {}) {
  if (!isStructureObject(value)) return null;
  const normalizedLimit = Math.max(1, Math.trunc(Number(sampleLimit) || 3));
  const nodes = Array.isArray(value.nodes) ? value.nodes : [];
  const edges = Array.isArray(value.edges) ? value.edges : [];
  const nodeSamples = nodes
    .slice(0, normalizedLimit)
    .map((row) => String(row?.label ?? row?.id ?? "").trim())
    .filter(Boolean);
  const edgeSamples = edges
    .slice(0, normalizedLimit)
    .map((row) => {
      const from = String(row?.from ?? "").trim();
      const to = String(row?.to ?? "").trim();
      if (!from || !to) return "";
      const directed = row?.directed === true ? "->" : "-";
      const label = String(row?.label ?? "").trim();
      return label ? `${from} ${directed} ${to} (${label})` : `${from} ${directed} ${to}`;
    })
    .filter(Boolean);
  const directedCount = edges.filter((row) => row?.directed === true).length;
  return {
    title: String(value?.meta?.title ?? value?.title ?? "").trim(),
    nodeCount: nodes.length,
    edgeCount: edges.length,
    directedCount,
    nodeSamples,
    edgeSamples,
  };
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
  const allValues = obs.all_values && typeof obs.all_values === "object" ? obs.all_values : values;
  const keySet = new Set(channels.map((channel) => String(channel?.key ?? "").trim()).filter(Boolean));
  Object.keys(allValues).forEach((key) => {
    const normalizedKey = String(key ?? "").trim();
    if (!normalizedKey) return;
    keySet.add(normalizedKey);
  });
  const keys = Array.from(keySet);
  if (!keys.length) return null;

  const timeCandidates = ["t", "time", "시간", "tick", "프레임수"];
  const yPriorityCandidates = ["theta", "각도", "angle", "rad", "y", "price", "p", "i", "energy"];
  const normalizeKey = (raw) => String(raw ?? "").trim().toLowerCase();
  const numericKeys = keys.filter((key) => Number.isFinite(Number(allValues[key])));
  const timeKey = numericKeys.find((key) => timeCandidates.includes(normalizeKey(key)));
  const yKeyFromPriority = numericKeys.find((key) =>
    yPriorityCandidates.some((token) => normalizeKey(key).includes(token)),
  );
  const yKey = yKeyFromPriority && yKeyFromPriority !== timeKey
    ? yKeyFromPriority
    : numericKeys.find((key) => key !== timeKey);
  if (!yKey) return null;

  const x = Number(timeKey ? allValues[timeKey] : 0);
  const y = Number(allValues[yKey]);
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

function normalizeViewStackEntry(raw, fallbackRole = "") {
  if (!raw || typeof raw !== "object") return null;
  const family = normalizeViewFamily(raw.family ?? raw.view_family ?? raw.kind);
  if (!family) return null;
  const role = String(raw.role ?? fallbackRole ?? "").trim();
  return {
    family,
    role,
    source: "view_meta",
  };
}

function normalizeViewStackEntries(rawList, fallbackRole = "") {
  const rows = Array.isArray(rawList) ? rawList : [];
  return rows.map((row) => normalizeViewStackEntry(row, fallbackRole)).filter(Boolean);
}

function inferAvailableStructuredFamilies({ graph, space2d, table, text, structure }) {
  const out = [];
  if (space2d) out.push("space2d");
  if (graph) out.push("graph");
  if (table) out.push("table");
  if (text) out.push("text");
  if (structure) out.push("structure");
  return out;
}

export function resolveStructuredViewStackFromState(stateLike) {
  const row = stateLike && typeof stateLike === "object" ? stateLike : {};
  const normalized = row.state && typeof row.state === "object"
    ? row.state
    : normalizeWasmStatePayload(row);
  const primaryMeta = normalizeViewStackEntry(normalized?.view_meta?.primary, "main");
  const secondaryMeta = normalizeViewStackEntries(normalized?.view_meta?.secondary, "secondary");
  const overlayMeta = normalizeViewStackEntries(normalized?.view_meta?.overlays, "overlay");
  const availableFamilies = inferAvailableStructuredFamilies(row);
  const seenFamilies = new Set();
  const primary = primaryMeta ?? (() => {
    const inferred = VIEW_FAMILY_PRIORITY.find((family) => availableFamilies.includes(family));
    if (!inferred) return null;
    seenFamilies.add(inferred);
    return { family: inferred, role: "main", source: "inferred" };
  })();

  const secondary = [];
  const overlays = [];
  const pushUnique = (target, entry) => {
    if (!entry || !entry.family) return;
    const key = `${target === overlays ? "overlay" : "body"}:${entry.family}:${entry.role}`;
    if (seenFamilies.has(key)) return;
    seenFamilies.add(key);
    target.push(entry);
  };

  if (primary?.family) {
    seenFamilies.add(primary.family);
  }
  secondaryMeta.forEach((entry) => pushUnique(secondary, entry));
  overlayMeta.forEach((entry) => pushUnique(overlays, entry));
  availableFamilies.forEach((family) => {
    if (!family || family === primary?.family) return;
    const inSecondary = secondary.some((entry) => entry.family === family);
    const inOverlay = overlays.some((entry) => entry.family === family);
    if (inSecondary || inOverlay) return;
    pushUnique(secondary, { family, role: "secondary", source: "inferred" });
  });

  const familySet = new Set();
  [primary, ...secondary, ...overlays].forEach((entry) => {
    if (!entry?.family) return;
    familySet.add(entry.family);
  });
  availableFamilies.forEach((family) => {
    if (family) familySet.add(family);
  });

  const families = orderViewFamiliesByPriority(Array.from(familySet), VIEW_FAMILY_PRIORITY);
  return {
    primary,
    secondary,
    overlays,
    families,
  };
}

export function extractStructuredViewsFromState(
  state,
  { preferPatch = false, allowObservationOutputFallback = true } = {},
) {
  const normalized = normalizeWasmStatePayload(state);
  const resources = normalized?.resources?.value ?? {};
  const patch = normalizePatch(normalized.patch);
  const observationOutputLines = allowObservationOutputFallback ? extractOutputLinesFromObservation(normalized) : [];
  const outputSpace2d = allowObservationOutputFallback ? parseSpace2dFromOutputLines(observationOutputLines) : null;
  const outputText = allowObservationOutputFallback ? parseTextFromOutputLines(observationOutputLines) : null;

  const preferMetaGraph = isGraphObject(normalized?.view_meta?.graph) ? normalized.view_meta.graph : null;
  const preferMetaSpace2d = isSpace2dObject(normalized?.view_meta?.space2d) ? normalized.view_meta.space2d : null;
  const preferMetaText = normalizeTextObject(normalized?.view_meta?.text);
  const preferMetaStructure = isStructureObject(normalized?.view_meta?.structure) ? normalized.view_meta.structure : null;

  const graphCandidate = preferMetaGraph
    ? { obj: preferMetaGraph, raw: JSON.stringify(preferMetaGraph), source: "view_meta" }
    : preferPatch
      ? pickStructuredFromPatch(patch, isGraphObject) ?? pickStructuredFromResources(resources, isGraphObject, ["graph"])
      : pickStructuredFromResources(resources, isGraphObject, ["graph"]) ?? pickStructuredFromPatch(patch, isGraphObject);

  const space2dCandidate = preferMetaSpace2d
    ? { obj: preferMetaSpace2d, raw: JSON.stringify(preferMetaSpace2d), source: "view_meta" }
    : outputSpace2d
      ? { obj: outputSpace2d, raw: JSON.stringify(outputSpace2d), source: "observation_output" }
    : preferPatch
      ? pickStructuredFromPatch(patch, isSpace2dObject) ?? pickStructuredFromResources(resources, isSpace2dObject, ["space2d", "2d"])
      : pickStructuredFromResources(resources, isSpace2dObject, ["space2d", "2d"]) ?? pickStructuredFromPatch(patch, isSpace2dObject);

  const tableCandidate = preferPatch
    ? pickStructuredFromPatch(patch, isTableObject) ?? pickStructuredFromResources(resources, isTableObject, ["table"])
    : pickStructuredFromResources(resources, isTableObject, ["table"]) ?? pickStructuredFromPatch(patch, isTableObject);

  const textCandidate = (() => {
    if (preferMetaText) {
      return {
        obj: preferMetaText,
        raw: JSON.stringify(preferMetaText),
      };
    }
    if (outputText) {
      return {
        obj: normalizeTextObject(outputText),
        raw: JSON.stringify(outputText),
      };
    }
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

  const structureCandidate = preferMetaStructure
    ? {
      obj: preferMetaStructure,
      raw: JSON.stringify(preferMetaStructure),
      source: "view_meta",
    }
    : preferPatch
      ? pickStructuredFromPatch(patch, isStructureObject) ?? pickStructuredFromResources(resources, isStructureObject, ["structure", "구조"])
      : pickStructuredFromResources(resources, isStructureObject, ["structure", "구조"]) ?? pickStructuredFromPatch(patch, isStructureObject);

  const fallbackGraph = graphCandidate?.obj ? null : buildGraphFromObservation(normalized);

  const structured = {
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
    structure: structureCandidate?.obj ?? null,
    structureRaw: structureCandidate?.raw ?? null,
  };
  structured.viewStack = resolveStructuredViewStackFromState(structured);
  structured.families = Array.isArray(structured.viewStack?.families) ? structured.viewStack.families : [];
  return structured;
}
