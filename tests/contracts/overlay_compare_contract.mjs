function normalizeToken(value) {
  return String(value ?? "").trim().toLowerCase();
}

function pickMetaToken(meta, ...keys) {
  if (!meta || typeof meta !== "object") return "";
  for (const key of keys) {
    const token = normalizeToken(meta[key]);
    if (token) return token;
  }
  return "";
}

function readPrimarySeriesId(graph) {
  const series = Array.isArray(graph?.series) ? graph.series : [];
  for (const row of series) {
    const id = normalizeToken(row?.id);
    if (id) return id;
  }
  return "";
}

function toComparableGraph(raw) {
  const graph = raw && typeof raw === "object" ? raw.graph : null;
  if (!graph || typeof graph !== "object") return null;
  const meta = graph.meta && typeof graph.meta === "object" ? graph.meta : {};
  return {
    graphKind: pickMetaToken(meta, "graph_kind", "kind", "schema"),
    xKind: pickMetaToken(meta, "axis_x_kind", "axis_kind"),
    xUnit: pickMetaToken(meta, "axis_x_unit", "axis_unit"),
    yKind: pickMetaToken(meta, "axis_y_kind", "y_kind"),
    yUnit: pickMetaToken(meta, "axis_y_unit", "y_unit"),
    seriesId: readPrimarySeriesId(graph),
    schemaToken: normalizeToken(graph.schema),
  };
}

function fail(code, reason) {
  return {
    ok: false,
    code,
    reason,
  };
}

export function canOverlayCompareRuns(baseline, variant) {
  const left = toComparableGraph(baseline);
  const right = toComparableGraph(variant);
  if (!left || !right) {
    return fail("graph_missing", "그래프 데이터가 없어 비교할 수 없습니다.");
  }

  const baselineGraphKind = left.graphKind || left.schemaToken;
  const variantGraphKind = right.graphKind || right.schemaToken;
  if (baselineGraphKind !== variantGraphKind) {
    return fail("mismatch_graphKind", "graph_kind 가 서로 다릅니다.");
  }
  if (left.xKind !== right.xKind) {
    return fail("mismatch_xKind", "x축 kind 가 서로 다릅니다.");
  }
  if (left.xUnit !== right.xUnit) {
    return fail("mismatch_xUnit", "x축 unit 이 서로 다릅니다.");
  }
  if (left.yKind !== right.yKind) {
    return fail("mismatch_yKind", "y축 kind 가 서로 다릅니다.");
  }
  if (left.yUnit !== right.yUnit) {
    return fail("mismatch_yUnit", "y축 unit 이 서로 다릅니다.");
  }

  if (!left.seriesId || !right.seriesId) {
    return fail("series_missing", "series_id 가 비어 있어 비교할 수 없습니다.");
  }
  if (left.seriesId !== right.seriesId) {
    return fail("series_mismatch", "series_id 가 서로 다릅니다.");
  }
  return {
    ok: true,
    code: "ok",
    reason: "",
  };
}
