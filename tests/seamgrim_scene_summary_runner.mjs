import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function asStableJson(value) {
  return JSON.stringify(value);
}

async function main() {
  const root = process.cwd();
  const sceneModule = await import(
    pathToFileURL(path.resolve(root, "solutions/seamgrim_ui_mvp/ui/scene_summary_contract.js")).href
  );
  const inputRegistryModule = await import(
    pathToFileURL(path.resolve(root, "solutions/seamgrim_ui_mvp/ui/input_registry.js")).href
  );
  const overlayModule = await import(
    pathToFileURL(path.resolve(root, "solutions/seamgrim_ui_mvp/ui/overlay_session_contract.js")).href
  );

  const { buildSceneSummarySnapshot, restoreSceneSummarySession, serializeSceneSummarySession } = sceneModule;
  const { createInputRegistryState, registerLessonInput, restoreInputRegistrySession, serializeInputRegistrySession } = inputRegistryModule;
  const { buildOverlaySessionRunsPayload } = overlayModule;

  let inputRegistry = createInputRegistryState();
  inputRegistry = registerLessonInput(inputRegistry, {
    id: "lesson:pendulum",
    lessonId: "pendulum",
    label: "진자",
    requiredViews: ["2d", "graph", "text"],
    ddnText: "(매마디)마다 { 보임 { t: t. }. }.",
  });

  const overlayRuns = [
    {
      id: "run-b",
      label: "overlay",
      visible: true,
      layer_index: 1,
      opacity: 1,
      graph: {
        series: [{ id: "overlay", points: [{ x: 0, y: 1 }, { x: 1, y: 2 }] }],
        meta: { update: "append", tick: 1 },
      },
    },
    {
      id: "run-a",
      label: "base",
      visible: true,
      layer_index: 0,
      graph: {
        series: [{ id: "main", points: [{ x: 0, y: 0 }, { x: 1, y: 1 }, { x: 2, y: 4 }] }],
        meta: { update: "replace", tick: null },
      },
    },
  ];
  const runtimeDerived = {
    observation: {
      all_values: {
        t: 1,
      },
    },
    views: {
      graph: {
        axis: { x_min: 0, x_max: 2, y_min: 0, y_max: 4 },
        series: [{ id: "main", points: [{ x: 0, y: 0 }, { x: 1, y: 1 }, { x: 2, y: 4 }] }],
        meta: {
          source_input_hash: "input-hash-1",
          result_hash: "result-hash-1",
          update: "append",
          tick: 2,
        },
      },
    },
  };
  const runtimeBefore = asStableJson(runtimeDerived);

  const snapshotA = buildSceneSummarySnapshot({
    timestamp: "2026-03-29T00:00:00Z",
    lessonId: "pendulum",
    lessonTitle: "진자",
    requiredViews: ["2d", "graph", "text"],
    inputRegistryState: inputRegistry,
    overlayRuns,
    runtimeDerived,
    runtimeHash: "result-hash-fallback",
  });
  const snapshotB = buildSceneSummarySnapshot({
    timestamp: "2026-03-29T00:00:00Z",
    lessonId: "pendulum",
    lessonTitle: "진자",
    requiredViews: ["2d", "graph", "text"],
    inputRegistryState: inputRegistry,
    overlayRuns,
    runtimeDerived,
    runtimeHash: "result-hash-fallback",
  });
  assert(
    asStableJson(snapshotA) === asStableJson(snapshotB),
    "scene summary: identical state should produce identical scene snapshot",
  );

  assert(snapshotA.schema === "seamgrim.scene.v0", "scene summary: schema");
  assert(
    asStableJson(snapshotA.required_views) === asStableJson(["space2d", "graph", "text"]),
    "scene summary: required_views normalized",
  );
  assert(
    snapshotA.inputs?.selected_id === "lesson:pendulum",
    "scene summary: selected input id",
  );
  assert(snapshotA.layers?.[0]?.id === "run-a", "scene summary: layer order by index");
  assert(snapshotA.layers?.[1]?.update === "append", "scene summary: layer update preserved");
  assert(snapshotA.hashes?.input_hash === "input-hash-1", "scene summary: input hash");
  assert(snapshotA.hashes?.result_hash === "result-hash-1", "scene summary: result hash");
  assert(snapshotA.time?.frame_count === 3, "scene summary: frame count from graph points");
  assert(
    asStableJson(runtimeDerived) === runtimeBefore,
    "scene summary: preview builder must not mutate runtime state",
  );

  const restoredInputRegistry = restoreInputRegistrySession(serializeInputRegistrySession(inputRegistry));
  const restoredOverlayRuns = buildOverlaySessionRunsPayload(overlayRuns);
  const restoredSnapshot = buildSceneSummarySnapshot({
    timestamp: "2026-03-29T00:00:00Z",
    lessonId: "pendulum",
    lessonTitle: "진자",
    requiredViews: ["2d", "graph", "text"],
    inputRegistryState: restoredInputRegistry,
    overlayRuns: restoredOverlayRuns,
    runtimeDerived,
    runtimeHash: "result-hash-fallback",
  });
  assert(
    asStableJson(restoredSnapshot) === asStableJson(snapshotA),
    "scene summary: snapshot must remain stable after session restore",
  );

  const serialized = serializeSceneSummarySession(snapshotA);
  const restoredSessionScene = restoreSceneSummarySession(serialized);
  assert(
    asStableJson(restoredSessionScene) === asStableJson(snapshotA),
    "scene summary: serialize/restore round-trip",
  );

  console.log("seamgrim scene summary runner ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
