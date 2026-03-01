import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const root = process.cwd();
  const runPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js");
  const runMod = await import(pathToFileURL(runPath).href);
  const {
    RunScreen,
    synthesizePendulumSpace2dFromObservation,
    synthesizePointSpace2dFromObservation,
    synthesizeSpace2dFromGraph,
    synthesizeSpace2dFromObservation,
  } = runMod;

  assert(typeof synthesizeSpace2dFromObservation === "function", "run export: synthesizeSpace2dFromObservation");
  assert(typeof synthesizePendulumSpace2dFromObservation === "function", "run export: synthesizePendulumSpace2dFromObservation");
  assert(typeof synthesizePointSpace2dFromObservation === "function", "run export: synthesizePointSpace2dFromObservation");
  assert(typeof synthesizeSpace2dFromGraph === "function", "run export: synthesizeSpace2dFromGraph");
  assert(typeof RunScreen === "function", "run export: RunScreen");

  const pendulumFromValues = synthesizeSpace2dFromObservation({
    channels: [{ key: "theta" }, { key: "L" }],
    row: [0.5, 1.0],
    values: { theta: 0.5, L: 1.0 },
  });
  assert(pendulumFromValues && Array.isArray(pendulumFromValues.shapes), "pendulum from values: shapes");
  assert(pendulumFromValues.meta?.title === "pendulum-observation-fallback", "pendulum from values: title");

  const pendulumFromRow = synthesizeSpace2dFromObservation({
    channels: ["t", "theta"],
    row: [0.02, 0.42],
  });
  assert(pendulumFromRow && Array.isArray(pendulumFromRow.shapes), "pendulum from row: shapes");
  assert(pendulumFromRow.meta?.title === "pendulum-observation-fallback", "pendulum from row: title");

  const pointFromRow = synthesizeSpace2dFromObservation({
    channels: ["x", "y"],
    row: [1.2, -0.4],
  });
  assert(pointFromRow && Array.isArray(pointFromRow.shapes), "point fallback from row: shapes");
  assert(pointFromRow.meta?.title === "xy-observation-fallback", "point fallback from row: title");

  const none = synthesizeSpace2dFromObservation({ channels: ["t"], row: [0.1] });
  assert(none === null, "fallback none: null result");

  const pendulumFromGraph = synthesizeSpace2dFromGraph({
    series: [
      {
        id: "theta",
        points: [
          { x: 0, y: 0.5 },
          { x: 0.1, y: 0.3 },
        ],
      },
    ],
  });
  assert(pendulumFromGraph && Array.isArray(pendulumFromGraph.shapes), "pendulum from graph: shapes");
  assert(pendulumFromGraph.meta?.title === "pendulum-graph-fallback", "pendulum from graph: title");

  const pointFromGraph = synthesizeSpace2dFromGraph({
    axis: { x_min: -2, x_max: 2, y_min: -1, y_max: 1 },
    series: [{ id: "y", points: [{ x: 1.25, y: -0.5 }] }],
  });
  assert(pointFromGraph && Array.isArray(pointFromGraph.shapes), "point from graph: shapes");
  assert(pointFromGraph.meta?.title === "graph-point-fallback", "point from graph: title");

  const runScreenDtGuard = new RunScreen({
    root: null,
    wasmState: { fpsLimit: 30, dtMax: 0 },
  });
  const guardedInput = runScreenDtGuard.getStepInput();
  assert(Number.isFinite(guardedInput.dt) && guardedInput.dt > 0, "run step input: dt guard for zero dtMax");

  console.log("seamgrim pendulum bogae fallback runner ok");
}

main().catch((err) => {
  console.error(String(err?.stack || err));
  process.exit(1);
});
