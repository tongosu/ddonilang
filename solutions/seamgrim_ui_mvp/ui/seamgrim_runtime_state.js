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
  ]);

  function decodeLegacyFixed64Negative(num) {
    if (!Number.isFinite(num) || num >= 0) return num;
    const abs = Math.abs(num);
    const bucket = Math.floor(abs) - 1;
    if (!Number.isFinite(bucket) || bucket < 0) return num;
    const decoded = abs - (2 * bucket) - 2;
    if (!Number.isFinite(decoded) || decoded > 0) return num;
    return decoded;
  }

  function readNumber(text) {
    const num = Number(text);
    if (!Number.isFinite(num)) return null;
    return decodeLegacyFixed64Negative(num);
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
      if (["stroke", "fill", "color", "token", "id", "name", "label", "토큰"].includes(lowerKey)) {
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
        },
        next: idx,
      };
    }
    if (["point", "점"].includes(kindRaw)) {
      if (![data.x, data.y].every((v) => Number.isFinite(v))) {
        return { shape: null, next: idx };
      }
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
  const patch = Array.isArray(normalized?.patch) ? normalized.patch : [];

  const rows = [];
  Object.entries(allValues).forEach(([key, raw]) => rows.push({ key, raw, source: "observation" }));
  Object.entries(resourceValues).forEach(([key, raw]) => rows.push({ key, raw, source: "resource" }));
  patch.forEach((op) => {
    if (!op || typeof op !== "object") return;
    if (String(op.op ?? "") !== "set_resource_value") return;
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

  for (const row of fallbackRows) {
    const tokens = [];
    collectOutputTokens(row.raw, tokens);
    const lines = tokens.map((token) => normalizeOutputToken(token)).filter(Boolean);
    if (!lines.length) continue;
    const hasKnownHint = lines.some((line) => {
      const lower = line.toLowerCase();
      return (
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
  return [];
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

export function extractStructuredViewsFromState(state, { preferPatch = false } = {}) {
  const normalized = normalizeWasmStatePayload(state);
  const resources = normalized?.resources?.value ?? {};
  const patch = normalizePatch(normalized.patch);
  const observationOutputLines = extractOutputLinesFromObservation(normalized);
  const outputSpace2d = parseSpace2dFromOutputLines(observationOutputLines);
  const outputText = parseTextFromOutputLines(observationOutputLines);

  const preferMetaGraph = isGraphObject(normalized?.view_meta?.graph) ? normalized.view_meta.graph : null;
  const preferMetaSpace2d = isSpace2dObject(normalized?.view_meta?.space2d) ? normalized.view_meta.space2d : null;
  const preferMetaText = normalizeTextObject(normalized?.view_meta?.text);

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
