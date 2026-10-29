function toFiniteNumber(raw) {
  const num = Number(raw);
  return Number.isFinite(num) ? num : null;
}

function toInteger(raw, fallback = 0) {
  const num = Number(raw);
  if (!Number.isFinite(num)) return fallback;
  return Math.trunc(num);
}

function toText(raw) {
  return String(raw ?? "").trim();
}

export function normalizeUpdateMode(raw) {
  const token = toText(raw).toLowerCase();
  if (token === "append") return "append";
  return "replace";
}

function readPrimarySeries(graph) {
  const row = graph && typeof graph === "object" ? graph : {};
  const series = Array.isArray(row.series) ? row.series : [];
  if (!series.length) return null;
  return (
    series.find((entry) => Array.isArray(entry?.points) && entry.points.length > 0) ??
    series[0] ??
    null
  );
}

function readPointCount(graph) {
  const series = readPrimarySeries(graph);
  const points = Array.isArray(series?.points) ? series.points : [];
  return points.length;
}

function buildImplicitTicksFromPointCount(pointCount) {
  const out = [];
  const count = Math.max(0, Math.trunc(Number(pointCount) || 0));
  for (let i = 0; i < count; i += 1) {
    out.push(i);
  }
  return out;
}

function normalizeTicksArray(rawTicks) {
  const source = Array.isArray(rawTicks) ? rawTicks : [];
  const out = [];
  source.forEach((tick) => {
    const parsed = toFiniteNumber(tick);
    if (parsed === null) return;
    out.push(parsed);
  });
  return out;
}

export function resolveGraphUpdateTick(graph) {
  const row = graph && typeof graph === "object" ? graph : {};
  const meta = row.meta && typeof row.meta === "object" ? row.meta : {};
  const sample = row.sample && typeof row.sample === "object" ? row.sample : {};
  const update = normalizeUpdateMode(row.update ?? meta.update);
  const explicitTicks = normalizeTicksArray(row.ticks ?? sample.ticks);
  const tickFromMeta = toFiniteNumber(row.tick ?? meta.tick ?? sample.tick);
  if (explicitTicks.length > 0) {
    const tick = explicitTicks[explicitTicks.length - 1];
    return { update, tick, ticks: explicitTicks, implicitTicks: false };
  }
  if (tickFromMeta !== null) {
    return { update, tick: tickFromMeta, ticks: [tickFromMeta], implicitTicks: false };
  }
  const fallbackTicks = buildImplicitTicksFromPointCount(readPointCount(row));
  return {
    update,
    tick: fallbackTicks.length ? fallbackTicks[fallbackTicks.length - 1] : null,
    ticks: fallbackTicks,
    implicitTicks: true,
  };
}

export function buildSnapshotRunUpdateTick(runLike) {
  const row = runLike && typeof runLike === "object" ? runLike : {};
  const graph = row.graph && typeof row.graph === "object" ? row.graph : {};
  const graphMeta = graph.meta && typeof graph.meta === "object" ? graph.meta : {};
  const info = resolveGraphUpdateTick(graph);
  return {
    id: toText(row.id),
    label: toText(row.label),
    source: row.source && typeof row.source === "object" ? { ...row.source } : {},
    inputs: row.inputs && typeof row.inputs === "object" ? { ...row.inputs } : {},
    graph: {
      ...graph,
      meta: {
        ...graphMeta,
        update: info.update,
        ...(info.tick === null ? {} : { tick: info.tick }),
      },
      ...(info.implicitTicks ? {} : { ticks: info.ticks }),
    },
    update: info.update,
    ...(info.tick === null ? {} : { tick: info.tick }),
    ticks: info.ticks,
    hash: {
      input: toText(row?.hash?.input ?? graphMeta.source_input_hash),
      result: toText(row?.hash?.result ?? graphMeta.result_hash),
    },
  };
}

export function buildSessionLayerUpdateTick(runLike, index = 0) {
  const row = runLike && typeof runLike === "object" ? runLike : {};
  const info = resolveGraphUpdateTick(row.graph);
  const layer = {
    id: toText(row.id) || `run-${index + 1}`,
    order: toInteger(row.order ?? row.layer_index ?? row.layerIndex, index),
    visible: row.visible !== false,
    update: info.update,
    ticks: info.ticks,
  };
  const groupId = toText(row.group_id ?? row.groupId);
  if (groupId) layer.group_id = groupId;
  if (info.tick !== null) layer.tick = info.tick;
  return layer;
}

export function sortSessionLayers(layers) {
  const rows = Array.isArray(layers) ? layers : [];
  return [...rows].sort((a, b) => {
    const orderA = toInteger(a?.order, 0);
    const orderB = toInteger(b?.order, 0);
    if (orderA !== orderB) return orderA - orderB;
    return toText(a?.id).localeCompare(toText(b?.id), "ko");
  });
}
