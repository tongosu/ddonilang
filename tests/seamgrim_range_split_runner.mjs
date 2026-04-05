import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function stableJson(value) {
  return JSON.stringify(value);
}

function normalizeRange(raw) {
  const row = raw && typeof raw === "object" ? raw : {};
  return {
    x_min: Number(row.x_min ?? row.xMin),
    x_max: Number(row.x_max ?? row.xMax),
    y_min: Number(row.y_min ?? row.yMin),
    y_max: Number(row.y_max ?? row.yMax),
  };
}

async function main() {
  const root = process.cwd();
  const snapshotSessionModule = await import(
    pathToFileURL(path.resolve(root, "solutions/seamgrim_ui_mvp/ui/snapshot_session_contract.js")).href
  );
  const { buildRuntimeSnapshotBundleV0 } = snapshotSessionModule;

  const runtimeGraph = {
    schema: "seamgrim.graph.v0",
    axis: {
      x_min: 0,
      x_max: 4,
      y_min: 0,
      y_max: 8,
    },
    series: [
      {
        id: "main",
        points: [
          { x: 0, y: 0 },
          { x: 1, y: 2 },
          { x: 2, y: 4 },
          { x: 3, y: 6 },
          { x: 4, y: 8 },
        ],
      },
    ],
    meta: {
      update: "replace",
      result_hash: "result-hash-fixed",
    },
  };

  const sessionA = {
    controls: { a: 2, b: 0 },
    sample: { var: "x", x_min: 0, x_max: 4, step: 1 },
    view: {
      panX: 0,
      panY: 0,
      zoom: 1,
      graph: {
        auto_fit: false,
        axis: { x_min: -1, x_max: 5, y_min: -1, y_max: 9 },
      },
    },
    active_run_id: "run:math_line",
  };

  const sessionB = {
    ...sessionA,
    view: {
      ...sessionA.view,
      graph: {
        auto_fit: false,
        axis: { x_min: -2, x_max: 6, y_min: -2, y_max: 10 },
      },
    },
  };

  const sessionC = {
    ...sessionA,
    sample: { var: "x", x_min: 0, x_max: 8, step: 1 },
  };

  const bundleA = buildRuntimeSnapshotBundleV0({
    timestamp: "2026-03-29T00:00:00Z",
    lessonId: "math_line",
    lessonTitle: "range split",
    ddnText: "#name: range-split",
    requiredViews: ["graph"],
    runtimeGraph,
    runtimeSessionState: sessionA,
    runtimeHash: "result-hash-fixed",
  });

  const bundleB = buildRuntimeSnapshotBundleV0({
    timestamp: "2026-03-29T00:00:00Z",
    lessonId: "math_line",
    lessonTitle: "range split",
    ddnText: "#name: range-split",
    requiredViews: ["graph"],
    runtimeGraph,
    runtimeSessionState: sessionB,
    runtimeHash: "result-hash-fixed",
  });

  const bundleC = buildRuntimeSnapshotBundleV0({
    timestamp: "2026-03-29T00:00:00Z",
    lessonId: "math_line",
    lessonTitle: "range split",
    ddnText: "#name: range-split",
    requiredViews: ["graph"],
    runtimeGraph,
    runtimeSessionState: sessionC,
    runtimeHash: "result-hash-fixed",
  });

  assert(bundleA.snapshot.schema === "seamgrim.snapshot.v0", "range split: snapshot schema");
  assert(bundleA.session.schema === "seamgrim.session.v0", "range split: session schema");
  assert(
    stableJson(bundleA.snapshot.run.graph?.sample) === stableJson(sessionA.sample),
    "range split: snapshot graph.sample should be persisted separately",
  );
  assert(
    stableJson(bundleA.session.sample) === stableJson(sessionA.sample),
    "range split: session sample should be persisted",
  );
  assert(
    stableJson(bundleA.snapshot.run.graph?.series) === stableJson(bundleB.snapshot.run.graph?.series),
    "range split: viewport change must not alter graph series payload",
  );
  assert(
    bundleA.snapshot.run.hash?.result === bundleB.snapshot.run.hash?.result,
    "range split: viewport change must not alter result hash",
  );
  assert(
    bundleA.snapshot.run.hash?.input === bundleB.snapshot.run.hash?.input,
    "range split: viewport change must not alter input hash",
  );
  assert(
    stableJson(bundleA.snapshot.run.graph?.view) !== stableJson(bundleB.snapshot.run.graph?.view),
    "range split: snapshot graph.view should follow viewport state",
  );
  assert(
    stableJson(normalizeRange(bundleA.session.view?.graph?.axis)) === stableJson(normalizeRange(sessionA.view.graph.axis)),
    "range split: session view.graph.axis restore payload",
  );
  assert(
    bundleA.snapshot.run.hash?.input !== bundleC.snapshot.run.hash?.input,
    "range split: sample change should produce different input hash",
  );

  console.log("seamgrim range split runner ok");
}

main().catch((error) => {
  console.error(String(error?.stack ?? error));
  process.exit(1);
});
