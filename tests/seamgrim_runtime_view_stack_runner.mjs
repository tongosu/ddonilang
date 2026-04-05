import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js");
  const mod = await import(pathToFileURL(modulePath).href);
  const {
    extractStructuredViewsFromState,
    flattenMirrorInputEntries,
    resolveStructuredViewStackFromState,
  } = mod;

  assert(typeof extractStructuredViewsFromState === "function", "runtime view stack: extractStructuredViewsFromState export");
  assert(typeof flattenMirrorInputEntries === "function", "runtime view stack: flattenMirrorInputEntries export");
  assert(typeof resolveStructuredViewStackFromState === "function", "runtime view stack: resolveStructuredViewStackFromState export");

  const inferredPayload = {
    schema: "seamgrim.engine_response.v0",
    tick_id: 0,
    state_hash: "blake3:inferred",
    view_meta: {
      space2d: { schema: "seamgrim.space2d.v0", points: [{ x: 0, y: 0 }] },
    },
    state: {
      channels: [],
      row: [],
      resources: {
        json: {},
        fixed64: {},
        handle: {},
        value: {
          graph_like: JSON.stringify({
            schema: "seamgrim.graph.v0",
            series: [{ id: "y", points: [{ x: 0, y: 1 }] }],
          }),
        },
      },
    },
  };
  const inferredViews = extractStructuredViewsFromState(inferredPayload, { preferPatch: false });
  assert(inferredViews.viewStack?.primary?.family === "space2d", "runtime view stack: inferred primary");
  assert(
    JSON.stringify(inferredViews.families) === JSON.stringify(["space2d", "graph"]),
    "runtime view stack: inferred families",
  );

  const explicitPayload = {
    ...inferredPayload,
    view_meta: {
      ...inferredPayload.view_meta,
      primary: { family: "graph", role: "main" },
      secondary: [{ family: "table", role: "data" }],
      overlays: [{ family: "text", role: "label" }],
    },
    state: {
      channels: [],
      row: [],
      resources: {
        json: {},
        fixed64: {},
        handle: {},
        value: {
          graph_like: JSON.stringify({
            schema: "seamgrim.graph.v0",
            series: [{ id: "y", points: [{ x: 0, y: 1 }] }],
          }),
          table_like: JSON.stringify({
            schema: "seamgrim.table.v0",
            columns: [{ key: "x" }],
            rows: [{ x: 1 }],
          }),
          text_like: JSON.stringify({ markdown: "설명" }),
          structure_like: JSON.stringify({
            schema: "seamgrim.structure.v0",
            nodes: [{ id: "A" }, { id: "B" }],
            edges: [{ from: "A", to: "B" }],
          }),
        },
      },
    },
  };
  const explicitViews = extractStructuredViewsFromState(explicitPayload, { preferPatch: false });
  assert(explicitViews.viewStack?.primary?.family === "graph", "runtime view stack: explicit primary");
  assert(explicitViews.viewStack?.secondary?.[0]?.family === "table", "runtime view stack: explicit secondary");
  assert(explicitViews.viewStack?.overlays?.[0]?.family === "text", "runtime view stack: explicit overlay");
  assert(explicitViews.structure?.nodes?.length === 2, "runtime view stack: structure extracted");

  const resolved = resolveStructuredViewStackFromState(explicitViews);
  assert(resolved.primary?.family === "graph", "runtime view stack: standalone resolve primary");
  assert(
    JSON.stringify(resolved.families) === JSON.stringify(["space2d", "graph", "table", "text", "structure"]),
    "runtime view stack: resolved families order",
  );

  const mirrorInputEntries = flattenMirrorInputEntries({
    dt: 0.1,
    pointer: { px: 12, py: -4 },
    keys: ["ArrowLeft", "Space"],
    empty: {},
  }, { maxEntries: 8 });
  assert(
    JSON.stringify(mirrorInputEntries) === JSON.stringify([
      ["dt", 0.1],
      ["pointer.px", 12],
      ["pointer.py", -4],
      ["keys[0]", "ArrowLeft"],
      ["keys[1]", "Space"],
      ["empty", "{}"],
    ]),
    "runtime view stack: mirror input entries flattened",
  );

  console.log("seamgrim runtime view stack ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
