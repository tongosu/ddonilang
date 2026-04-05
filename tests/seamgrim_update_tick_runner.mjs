import fs from "fs";
import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function loadJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf-8"));
}

function asStableJson(value) {
  return JSON.stringify(value);
}

async function main() {
  const root = process.cwd();
  const updateTickMod = await import(
    pathToFileURL(path.resolve(root, "solutions/seamgrim_ui_mvp/ui/update_tick_contract.js")).href
  );
  const snapshotSessionMod = await import(
    pathToFileURL(path.resolve(root, "solutions/seamgrim_ui_mvp/ui/snapshot_session_contract.js")).href
  );

  const { resolveGraphUpdateTick } = updateTickMod;
  const { buildSnapshotV0, buildSessionV0 } = snapshotSessionMod;

  const appendGraphPath = path.resolve(
    root,
    "pack/seamgrim_update_tick_basics/c01_append_ticks/expected.graph.v0.json",
  );
  const replaceGraphPath = path.resolve(
    root,
    "pack/seamgrim_update_tick_basics/c02_replace_ticks/expected.graph.v0.json",
  );
  const appendGraph = loadJson(appendGraphPath);
  const replaceGraph = loadJson(replaceGraphPath);

  const appendInfo = resolveGraphUpdateTick(appendGraph);
  assert(appendInfo.update === "append", "update/tick: append case update");
  assert(appendInfo.tick === 2, "update/tick: append case tick");
  assert(
    asStableJson(appendInfo.ticks) === asStableJson([2]),
    "update/tick: append case ticks",
  );

  const replaceInfo = resolveGraphUpdateTick(replaceGraph);
  assert(replaceInfo.update === "replace", "update/tick: replace case update");
  assert(replaceInfo.tick === 5, "update/tick: replace case tick");
  assert(
    asStableJson(replaceInfo.ticks) === asStableJson([5]),
    "update/tick: replace case ticks",
  );

  const snapshot = buildSnapshotV0({
    timestamp: "2026-03-29T00:00:00Z",
    note: "append",
    run: {
      id: "run-append",
      label: "append",
      source: { kind: "ddn", text: "#name: append" },
      inputs: { sample: { var: "x", x_min: 0, x_max: 2, step: 1 } },
      graph: appendGraph,
    },
  });
  assert(snapshot.schema === "seamgrim.snapshot.v0", "snapshot: schema");
  assert(snapshot.run.update === "append", "snapshot: run.update");
  assert(snapshot.run.tick === 2, "snapshot: run.tick");
  assert(
    asStableJson(snapshot.run.ticks) === asStableJson([2]),
    "snapshot: run.ticks",
  );
  assert(
    snapshot.run.graph?.meta?.update === "append",
    "snapshot: graph.meta.update",
  );

  const session = buildSessionV0({
    timestamp: "2026-03-29T00:00:00Z",
    lessonId: "update_tick_pack",
    ddnText: "#name: update_tick_pack",
    requiredViews: ["graph"],
    inputRegistryState: {
      registry: [{ id: "ddn:pack", type: "ddn", label: "pack", payload: { ddn_text: "#name: pack" } }],
      selectedId: "ddn:pack",
    },
    overlayRuns: [
      {
        id: "run-append",
        visible: true,
        layer_index: 1,
        graph: appendGraph,
      },
      {
        id: "run-replace",
        visible: true,
        layer_index: 0,
        graph: replaceGraph,
      },
    ],
    cursor: { id: "run-replace", t: 0, tick: 5 },
  });
  assert(session.schema === "seamgrim.session.v0", "session: schema");
  assert(Array.isArray(session.layers) && session.layers.length === 2, "session: layer count");
  assert(session.layers[0].id === "run-replace", "session: layer order");
  assert(session.layers[0].update === "replace", "session: replace layer update");
  assert(session.layers[0].tick === 5, "session: replace layer tick");
  assert(
    asStableJson(session.layers[0].ticks) === asStableJson([5]),
    "session: replace layer ticks",
  );
  assert(session.layers[1].update === "append", "session: append layer update");
  assert(session.layers[1].tick === 2, "session: append layer tick");

  const noTickGraph = {
    schema: "seamgrim.graph.v0",
    series: [{ id: "main", points: [{ x: 0, y: 0 }, { x: 1, y: 1 }, { x: 2, y: 2 }] }],
    meta: { update: "append" },
  };
  const noTickInfo = resolveGraphUpdateTick(noTickGraph);
  assert(noTickInfo.update === "append", "tick fallback: update");
  assert(noTickInfo.tick === 2, "tick fallback: implicit last tick");
  assert(
    asStableJson(noTickInfo.ticks) === asStableJson([0, 1, 2]),
    "tick fallback: row index ticks",
  );
  assert(noTickInfo.implicitTicks === true, "tick fallback: implicit flag");

  console.log("seamgrim update tick runner ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
