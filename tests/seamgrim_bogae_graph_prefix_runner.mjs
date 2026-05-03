import { buildGraphFromValueResources } from "../solutions/seamgrim_ui_mvp/ui/graph_autorender.js";
import { extractStructuredViewsFromState } from "../solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function pointCount(series, index) {
  const row = Array.isArray(series) ? series[index] : null;
  return Array.isArray(row?.points) ? row.points.length : 0;
}

function main() {
  const prefixState = {
    schema: "seamgrim.state.v0",
    state_hash: "blake3:prefix-state",
    resources: {
      value: {},
      value_json: {
        "그래프_주계열": [
          { x: 0, y: 1 },
          { x: 1, y: 2 },
          { x: 2, y: 4 },
        ],
        "보개_그래프_보조": {
          points: [
            { x: 0, y: 2 },
            { x: 1, y: 3 },
          ],
        },
      },
    },
    channels: [],
    row: [],
    patch: [],
    view_meta: {},
  };
  const graph = buildGraphFromValueResources(prefixState);
  const series = Array.isArray(graph?.series) ? graph.series : [];
  assert(graph?.schema === "seamgrim.graph.v0", "graph prefix: schema");
  assert(graph?.meta?.source === "value-prefix", "graph prefix: source");
  assert(series.length === 2, `graph prefix: series count ${series.length}`);
  assert(pointCount(series, 0) === 3, "graph prefix: primary points");
  assert(pointCount(series, 1) === 2, "graph prefix: secondary points");

  const viewState = {
    schema: "seamgrim.state.v0",
    state_hash: "blake3:view-boundary-state",
    resources: {
      value: {},
      value_json: {},
    },
    channels: [],
    row: [],
    patch: [],
    view_meta: {
      graph: {
        schema: "seamgrim.graph.v0",
        series: [{ id: "view", points: [{ x: 0, y: 0 }, { x: 1, y: 1 }] }],
      },
    },
  };
  const views = extractStructuredViewsFromState(viewState, {
    preferPatch: false,
    allowObservationOutputFallback: false,
  });
  assert(views?.graphSource === "view_meta", "view boundary: graph source");
  assert(viewState.state_hash === "blake3:view-boundary-state", "view boundary: state hash unchanged");

  const report = {
    schema: "ddn.roadmap_v2.bogae_graph_prefix.report.v1",
    ok: true,
    cases: [
      {
        id: "prefix_value_json_to_graph",
        ok: true,
        source: graph.meta.source,
        series_count: series.length,
        primary_points: pointCount(series, 0),
        secondary_points: pointCount(series, 1),
      },
      {
        id: "view_meta_graph_boundary",
        ok: true,
        view_source: views.graphSource,
        state_hash_unchanged: viewState.state_hash === "blake3:view-boundary-state",
      },
    ],
  };
  console.log(JSON.stringify(report, null, 2));
}

try {
  main();
} catch (error) {
  console.error(String(error?.stack ?? error));
  process.exit(1);
}
