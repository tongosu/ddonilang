import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, "..");

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

const runtimeStatePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js");
const runtimeStateUrl = pathToFileURL(runtimeStatePath).href;
const observeSummaryContractPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/run_observe_summary_contract.js");
const observeSummaryContractUrl = pathToFileURL(observeSummaryContractPath).href;
const observeFamilyContractPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/run_observe_family_contract.js");
const observeFamilyContractUrl = pathToFileURL(observeFamilyContractPath).href;
const {
  extractStructuredViewsFromState,
  extractObservationOutputLogFromState,
  extractObservationOutputRowsFromState,
} = await import(runtimeStateUrl);
const { buildObserveSummaryViewModel } = await import(observeSummaryContractUrl);
const {
  formatObserveFamilyName,
  buildObserveFamilyActionHint,
  summarizeObserveFamilyMetric,
} = await import(observeFamilyContractUrl);

assert(typeof extractStructuredViewsFromState === "function", "runtime_state export: extractStructuredViewsFromState");
assert(typeof extractObservationOutputLogFromState === "function", "runtime_state export: extractObservationOutputLogFromState");
assert(typeof extractObservationOutputRowsFromState === "function", "runtime_state export: extractObservationOutputRowsFromState");
assert(typeof buildObserveSummaryViewModel === "function", "observe_summary_contract export: buildObserveSummaryViewModel");
assert(typeof formatObserveFamilyName === "function", "observe_family_contract export: formatObserveFamilyName");
assert(typeof buildObserveFamilyActionHint === "function", "observe_family_contract export: buildObserveFamilyActionHint");
assert(typeof summarizeObserveFamilyMetric === "function", "observe_family_contract export: summarizeObserveFamilyMetric");

const outputLinesOnlyPayload = {
  schema: "seamgrim.state.v0",
  channels: [{ key: "보개_출력_줄들", dtype: "text", role: "state" }],
  row: [
    JSON.stringify([
      "space2d",
      "space2d.shape",
      "point",
      "x",
      "0.2",
      "y",
      "0.4",
      "table.row",
      "속도",
      "1.2",
      "table.row",
      "높이",
      "3.4",
    ]),
  ],
  resources: { json: {}, fixed64: {}, handle: {}, value: {} },
};

const strictViews = extractStructuredViewsFromState(outputLinesOnlyPayload, {
  preferPatch: false,
  allowObservationOutputFallback: false,
});
assert(strictViews && typeof strictViews === "object", "strict views: object");
assert(strictViews.space2d === null, "strict views: output-line space2d must be blocked");
assert(strictViews.text === null, "strict views: output-line text must be blocked");
assert(Array.isArray(strictViews.families), "strict views: families array");
assert(strictViews.families.length === 0, "strict views: no structured families from output lines");

const compatViews = extractStructuredViewsFromState(outputLinesOnlyPayload, {
  preferPatch: false,
  allowObservationOutputFallback: true,
});
assert(compatViews && typeof compatViews === "object", "compat views: object");
assert(compatViews.space2d && typeof compatViews.space2d === "object", "compat views: space2d parsed");
assert(Array.isArray(compatViews.space2d.shapes), "compat views: space2d shapes");
assert(compatViews.space2d.shapes.length > 0, "compat views: space2d shape count");

const outputRows = extractObservationOutputRowsFromState(outputLinesOnlyPayload);
assert(Array.isArray(outputRows), "output rows: array");
assert(outputRows.length === 2, "output rows: row count");
assert(outputRows[0].key === "속도" && outputRows[0].value === "1.2", "output rows: first row");
assert(outputRows[1].key === "높이" && outputRows[1].value === "3.4", "output rows: second row");
const outputLog = extractObservationOutputLogFromState({
  ...outputLinesOnlyPayload,
  tick_id: 0,
  row: [JSON.stringify(["0", "1"])],
});
assert(outputLog.length === 2, "output log: count");
assert(outputLog[0].tick === 0 && outputLog[0].line_no === 1, "output log: tick + line");
assert(outputLog[1].text === "1", "output log: text");

const observeSummary = buildObserveSummaryViewModel({
  channels: 1,
  displayRows: [
    { family: "space2d", label: "보개", available: true, source: "view_meta", strict: true },
    { family: "graph", label: "그래프", available: false, source: "off", strict: true },
  ],
  availableRows: [{ family: "space2d", label: "보개", available: true, source: "view_meta", strict: true }],
  nonStrictRows: [],
  normalizedOutputRows: outputRows,
  outputRowsMetric: "2행 · 최근 높이=3.4",
  outputRowsPreview: "속도=1.2 · 높이=3.4",
  views: {
    space2d: { shapes: [{ kind: "circle" }] },
  },
  observeOutputActionCode: "open-ddn-observation-output",
  summarizeFamilyMetric: (family) => (family === "space2d" ? "1개 요소" : "출력 없음"),
  buildFamilyActionHint: ({ family }) => (family === "space2d" ? "권장: 보개를 확인하세요." : "권장: 출력 추가"),
});
assert(observeSummary.level === "ok", "observe summary: level");
assert(observeSummary.summaryText.includes("관찰채널 1개"), "observe summary: summary text");
assert(observeSummary.chipsHtml.includes("보임표:"), "observe summary: chips html");
assert(observeSummary.cardsHtml.includes("data-observe-action=\"open-ddn-observation-output\""), "observe summary: action card");
assert(observeSummary.cardsHtml.includes("data-observe-family=\"space2d\""), "observe summary: family card");
assert(observeSummary.cardsHtml.includes("data-observe-family=\"graph\""), "observe summary: disabled family card");
assert(observeSummary.cardsHtml.includes("aria-label=\""), "observe summary: compact cards keep aria labels");
assert(observeSummary.cardsHtml.includes("title=\""), "observe summary: compact cards keep tooltips");
assert(!observeSummary.cardsHtml.includes("run-observe-channel-guide"), "observe summary: guide text not visible");
assert(!observeSummary.cardsHtml.includes("run-observe-channel-source"), "observe summary: source text not visible");
assert(!observeSummary.cardsHtml.includes("run-observe-channel-rows"), "observe summary: row preview not visible");
assert(!observeSummary.cardsHtml.includes(">권장:"), "observe summary: recommendation text not visible");
assert(observeSummary.cardsHtml.includes("disabled"), "observe summary: unavailable mode remains disabled");

assert(formatObserveFamilyName("space2d") === "보개", "observe family: name mapping");
assert(
  buildObserveFamilyActionHint({ family: "graph", available: true, strict: false }).includes("strict"),
  "observe family: non-strict hint",
);
assert(
  summarizeObserveFamilyMetric("graph", { series: [{ points: [1, 2] }] }, { graph: () => "1개 계열" }) === "1개 계열",
  "observe family: metric reader routing",
);

const explicitStructuredPayload = {
  schema: "seamgrim.state.v0",
  channels: [{ key: "보개_출력_줄들", dtype: "text", role: "state" }],
  row: [JSON.stringify(["space2d", "space2d.shape", "point", "x", "1", "y", "2"])],
  resources: {
    json: {},
    fixed64: {},
    handle: {},
    value: {
      "space2d_strict": JSON.stringify({
        schema: "seamgrim.space2d.v0",
        meta: { source: "view_meta" },
        points: [{ x: 0, y: 0 }],
        shapes: [{ kind: "circle", x: 0, y: 0, r: 0.2 }],
      }),
    },
  },
};

const explicitStrictViews = extractStructuredViewsFromState(explicitStructuredPayload, {
  preferPatch: false,
  allowObservationOutputFallback: false,
});
assert(explicitStrictViews.space2d && typeof explicitStrictViews.space2d === "object", "strict views: explicit space2d allowed");
assert(Array.isArray(explicitStrictViews.space2d.shapes), "strict views: explicit shapes array");
assert(explicitStrictViews.space2d.shapes.length === 1, "strict views: explicit shape count");

console.log("seamgrim observe output contract runner ok");
