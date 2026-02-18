function normalizeNumber(raw) {
  const n = Number(raw);
  return Number.isFinite(n) ? n : null;
}

export function parsePointsFromValueString(valueText) {
  if (typeof valueText !== "string") return [];
  const maps = valueText.match(/짝맞춤\{[^}]*\}/g);
  if (!maps) return [];
  const points = [];
  maps.forEach((chunk) => {
    const xMatch = chunk.match(/(?:\"x\"|x)\s*=>\s*([+-]?\d+(?:\.\d+)?)/);
    const yMatch = chunk.match(/(?:\"y\"|y)\s*=>\s*([+-]?\d+(?:\.\d+)?)/);
    if (!xMatch || !yMatch) return;
    const x = normalizeNumber(xMatch[1]);
    const y = normalizeNumber(yMatch[1]);
    if (x === null || y === null) return;
    points.push({ x, y });
  });
  return points;
}

export function buildGraphFromValueResources(state, preferPatch = false) {
  const series = [];
  const seen = new Set();
  const prefixes = ["그래프_", "보개_그래프_"];

  const takeEntry = (tag, value) => {
    if (typeof tag !== "string") return;
    if (!prefixes.some((prefix) => tag.startsWith(prefix))) return;
    if (seen.has(tag)) return;
    const points = parsePointsFromValueString(value);
    if (!points.length) return;
    series.push({ name: tag, points });
    seen.add(tag);
  };

  if (preferPatch && Array.isArray(state?.patch)) {
    state.patch.forEach((op) => {
      if (!op || op.op !== "set_resource_value") return;
      takeEntry(op.tag, op.value);
    });
  }

  if (state?.resources?.value) {
    Object.entries(state.resources.value).forEach(([tag, value]) => takeEntry(tag, value));
  }

  if (!series.length) return null;
  return {
    schema: "seamgrim.graph.v0",
    graph_kind: "timeseries",
    series,
    meta: { source: "value-prefix" },
  };
}
