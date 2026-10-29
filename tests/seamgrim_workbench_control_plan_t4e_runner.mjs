import fs from "fs";
import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const SOURCE = `채비 {
  g:수 = (9.8) 매김 {
    범위: 1..20.
    간격: 0.1.
  }.
  theta0:수 <- (0.5) 매김 {
    범위: -1.2..1.2.
    분할수: 24.
  }.
}.`;

function authoritativePlan(overrides = {}) {
  return {
    schema: "ddn.maegim_control_plan.v1",
    controls: [
      {
        name: "g",
        decl_kind: "butbak",
        type_name: "수",
        init_expr_canon: "9.8",
        range: { min_expr_canon: "1", max_expr_canon: "20", inclusive_end: false },
        step_expr_canon: "0.1",
        split_count_expr_canon: null,
      },
      {
        name: "theta0",
        decl_kind: "gureut",
        type_name: "수",
        init_expr_canon: "0.5",
        range: { min_expr_canon: "-1.2", max_expr_canon: "1.2", inclusive_end: false },
        step_expr_canon: null,
        split_count_expr_canon: "24",
      },
    ],
    warnings: [],
    ...overrides,
  };
}

async function main() {
  const root = process.cwd();
  const parserPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/components/control_parser.js");
  const sliderPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/components/slider_panel.js");
  const runPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js");
  const appPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/app.js");
  const parserSource = fs.readFileSync(parserPath, "utf8");
  const sliderSource = fs.readFileSync(sliderPath, "utf8");
  const runSource = fs.readFileSync(runPath, "utf8");
  const appSource = fs.readFileSync(appPath, "utf8");

  const parser = await import(`${pathToFileURL(parserPath).href}?t4e=${Date.now()}`);
  const sliderModule = await import(`${pathToFileURL(sliderPath).href}?t4e=${Date.now()}`);
  const { buildControlSpecsFromDdn, applyControlValuesToDdnText } = parser;
  const { SliderPanel } = sliderModule;

  const planJson = JSON.stringify(authoritativePlan());
  const present = buildControlSpecsFromDdn(SOURCE, { maegimControlJson: planJson });
  assert(present.controlPlanStatus === "ready", "A: authoritative plan ready");
  assert(present.source === "maegim_control_json", "A: authoritative source identity");
  assert(present.specs.map((spec) => spec.id).join(",") === "g,theta0", "A/G: identity and ordering");
  assert(present.specs[0].type === "수", "A: exact type");
  assert(present.specs[0].value === 9.8, "A: exact default");
  assert(present.specs[0].min === 1 && present.specs[0].max === 20 && present.specs[0].step === 0.1, "A: exact range/step");
  assert(Math.abs(present.specs[1].step - 0.1) < 1e-12, "A: split count projection");

  const statusEl = { textContent: "", dataset: {} };
  const slider = new SliderPanel({ container: null, statusEl });
  const projected = slider.parseFromDdn(SOURCE, {
    maegimControlJson: planJson,
    controlLabels: { g: "중력" },
    controlBounds: { g: { min: -999, max: 999, step: 77 } },
  });
  assert(projected.specs[0].displayLabel === "중력", "A: presentation label preserved");
  assert(projected.specs[0].min === 1 && projected.specs[0].max === 20 && projected.specs[0].step === 0.1, "A: product metadata cannot override semantic bounds");
  slider.setValues({ g: 12 });
  const applied = applyControlValuesToDdnText(SOURCE, slider.getValues());
  assert(applied.includes("g:수 = (12) 매김 {"), "A: authoritative control identity reaches execution source");

  const empty = buildControlSpecsFromDdn("1 보여주기.", {
    maegimControlJson: JSON.stringify(authoritativePlan({ controls: [] })),
  });
  assert(empty.controlPlanStatus === "empty", "B: legitimate empty plan status");
  assert(empty.specs.length === 0, "B: legitimate empty plan has zero sliders");
  assert(empty.controlPlanDiagnostic === null, "B: legitimate empty plan is not an error");

  const missing = buildControlSpecsFromDdn(SOURCE, { maegimControlJson: "" });
  assert(missing.controlPlanStatus === "missing", "C: missing plan status");
  assert(missing.specs.length === 0, "C: missing plan cannot synthesize sliders");
  assert(missing.controlPlanDiagnostic?.code === "E_WORKBENCH_CONTROL_PLAN_MISSING", "C: missing plan diagnostic");

  const unsupportedType = authoritativePlan();
  unsupportedType.controls[0].type_name = "글";
  const unsupported = buildControlSpecsFromDdn(SOURCE, { maegimControlJson: JSON.stringify(unsupportedType) });
  assert(unsupported.controlPlanStatus === "unsupported", "D: unsupported type status");
  assert(unsupported.specs.length === 0, "D: unsupported type cannot downgrade");
  assert(unsupported.controlPlanDiagnostic?.code === "E_WORKBENCH_CONTROL_PLAN_UNSUPPORTED", "D: unsupported diagnostic");

  const incomplete = authoritativePlan();
  incomplete.controls[0].range = null;
  const incompleteResult = buildControlSpecsFromDdn(SOURCE, { maegimControlJson: JSON.stringify(incomplete) });
  assert(incompleteResult.controlPlanStatus === "unsupported", "D: incomplete slider capability status");
  assert(incompleteResult.specs.length === 0, "D: incomplete plan has no inferred range");

  const malformed = buildControlSpecsFromDdn(SOURCE, { maegimControlJson: "{broken" });
  assert(malformed.controlPlanStatus === "invalid", "E: malformed plan status");
  assert(malformed.specs.length === 0, "E: malformed plan cannot create defaults");
  assert(malformed.controlPlanDiagnostic?.code === "E_WORKBENCH_CONTROL_PLAN_INVALID", "E: malformed diagnostic");

  const repeated = buildControlSpecsFromDdn(SOURCE, { maegimControlJson: planJson });
  assert(JSON.stringify(repeated) === JSON.stringify(present), "G: deterministic identity/order/value contract");

  assert(!parserSource.includes('source: "maegim_inline"'), "F: inline slider source removed");
  assert(!parserSource.includes("inferBounds("), "F: JS bound inference removed");
  assert(!parserSource.includes("inferStepFromValue("), "F: JS step inference removed");
  assert(!sliderSource.includes("controlBounds"), "F: presentation cannot override semantic bounds");
  assert(runSource.includes("refreshAuthoritativeControlPlan("), "F: Run refreshes native control plan");
  assert(runSource.includes("invalidateControlPlanForSource("), "F: stale source invalidates plan");
  const hydrateIndex = appSource.indexOf("await lessonCanonHydrator.hydrateLessonCanon(baseLesson)");
  const openIndex = appSource.indexOf("openRunWithLessonWithSource(hydratedLesson", hydrateIndex);
  assert(hydrateIndex >= 0 && openIndex > hydrateIndex, "F: editor hydrates authoritative plan before Run projection");

  console.log("seamgrim workbench control plan t4e ok (ready=1 empty=1 missing=1 unsupported=2 invalid=1 synthesis=0)");
}

main().catch((error) => {
  console.error(error?.stack ?? String(error));
  process.exit(1);
});
