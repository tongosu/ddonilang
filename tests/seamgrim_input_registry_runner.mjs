import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/input_registry.js");
  const mod = await import(pathToFileURL(modulePath).href);
  const {
    createInputRegistryState,
    getSelectedInputRegistryItem,
    registerFormulaInput,
    registerLessonInput,
    restoreInputRegistrySession,
    serializeInputRegistrySession,
    upsertInputRegistryItem,
  } = mod;

  let state = createInputRegistryState();
  state = registerFormulaInput(state, {
    id: "formula:main",
    label: "수식 입력",
    formulaText: "y = a*x + b",
    derivedDdn: "채비 { a:수 <- 1. b:수 <- 0. }.",
  });
  state = upsertInputRegistryItem(
    state,
    {
      id: "formula:main",
      type: "formula",
      payload: {
        derived_ddn: "채비 { a:수 <- 2. b:수 <- 1. }.",
      },
    },
    { select: true },
  );
  const selectedFormula = getSelectedInputRegistryItem(state);
  assert(selectedFormula?.type === "formula", "input registry: formula source selected");
  assert(
    String(selectedFormula?.payload?.formula_text ?? "") === "y = a*x + b",
    "input registry: formula source keeps original formula text after apply",
  );
  assert(
    String(selectedFormula?.payload?.derived_ddn ?? "").includes("a:수 <- 2"),
    "input registry: formula source updates derived ddn",
  );

  state = registerLessonInput(state, {
    id: "lesson:physics_pendulum",
    lessonId: "physics_pendulum",
    label: "진자",
    requiredViews: ["2d", "graph", "text"],
    ddnText: "(매마디)마다 { t 보여주기. }.",
  });
  const selectedLesson = getSelectedInputRegistryItem(state);
  assert(selectedLesson?.type === "lesson", "input registry: lesson source selected");
  assert(
    JSON.stringify(selectedLesson?.payload?.required_views ?? []) ===
      JSON.stringify(["space2d", "graph", "text"]),
    "input registry: lesson source provides normalized required_views",
  );

  const sessionPayload = serializeInputRegistrySession(state);
  assert(
    String(sessionPayload?.schema ?? "") === "seamgrim.input_registry.v0",
    "input registry: session schema",
  );
  const restored = restoreInputRegistrySession(sessionPayload);
  const restoredSelected = getSelectedInputRegistryItem(restored);
  assert(restoredSelected?.id === "lesson:physics_pendulum", "input registry: selected id restore");
  assert(
    String(restoredSelected?.payload?.lesson_id ?? "") === "physics_pendulum",
    "input registry: lesson payload restore",
  );
  assert(
    restored.registry.some(
      (entry) =>
        entry.id === "formula:main" &&
        String(entry.payload?.formula_text ?? "") === "y = a*x + b",
    ),
    "input registry: formula entry restore",
  );

  console.log("seamgrim input registry runner ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
