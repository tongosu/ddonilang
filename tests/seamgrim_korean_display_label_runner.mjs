import { formatDisplayLabel } from "../solutions/seamgrim_ui_mvp/ui/display_label_contract.js";
import { buildTablePreviewHtml } from "../solutions/seamgrim_ui_mvp/ui/components/table_preview.js";
import {
  normalizeRuntimeTableView,
  renderRuntimeTable,
} from "../solutions/seamgrim_ui_mvp/ui/screens/run.js";
import { buildObserveSummaryViewModel } from "../solutions/seamgrim_ui_mvp/ui/run_observe_summary_contract.js";
import { formatInspectorReportText } from "../solutions/seamgrim_ui_mvp/ui/inspector_contract.js";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

const table = {
  schema: "seamgrim.table.v0",
  meta: { source: "observation_output" },
  columns: [{ key: "host_exts" }, { key: "title" }, { key: "column" }],
  rows: [{ host_exts: 2, title: "예제", column: "A" }],
};

assert(formatDisplayLabel("host_exts") === "호스트 확장", "host_exts should be shown as Korean label");
assert(formatDisplayLabel("title") === "제목", "title should be shown as Korean label");
assert(formatDisplayLabel("column") === "열", "column should be shown as Korean label");

const previewHtml = buildTablePreviewHtml(table, { maxCols: 3, maxRows: 1 });
assert(previewHtml.includes("<th>호스트 확장</th>"), "preview table should localize host_exts column");
assert(previewHtml.includes("<th>제목</th>"), "preview table should localize title column");
assert(!previewHtml.includes("<th>host_exts</th>"), "preview table should not expose host_exts as visible column name");

const normalized = normalizeRuntimeTableView(table, { maxRows: 1 });
assert(normalized.columns[0].key === "host_exts", "runtime table should preserve raw key for data access");
assert(normalized.columns[0].label === "호스트 확장", "runtime table should localize visible label");
const container = { innerHTML: "" };
renderRuntimeTable(container, table, { maxRows: 1 });
assert(container.innerHTML.includes('data-col-key="host_exts"'), "rendered table should keep raw data key");
assert(container.innerHTML.includes(">호스트 확장<"), "rendered table should show Korean column label");

const observe = buildObserveSummaryViewModel({
  channels: 1,
  displayRows: [{ family: "table", label: "표", available: true, strict: true, source: "observation_output" }],
  availableRows: [{ family: "table" }],
  normalizedOutputRows: [{ key: "host_exts", value: "2", source: "table.row" }],
  outputRowsMetric: "1행",
  outputRowsPreview: "호스트 확장=2",
  observeOutputActionCode: "open_ddn",
  escapeHtml,
  summarizeFamilyMetric: () => "1열 · 1행",
  buildFamilyActionHint: () => "권장: 표를 점검하세요.",
});
assert(observe.cardsHtml.includes("보임표 행"), "observe card should use Korean table.row label");
assert(observe.cardsHtml.includes("출처=관찰 출력"), "observe tooltip should localize source label");
assert(!observe.cardsHtml.includes("source="), "observe tooltip should not expose source= chrome");

const inspector = formatInspectorReportText({
  lesson: { id: "lesson-1", title: "교과", description: "설명", ddn_meta: { required_views: ["table"] } },
  hash: { state: "abc", input: "def", result: "ghi" },
  runtime: { t: 0, tick: 1, playing: false, speed: 1 },
  schema: [{ name: "table", status: "ok", expected: "seamgrim.table.v0", actual: "seamgrim.table.v0" }],
  bridge_check: { ok: true },
  view_contract: {
    schema: "seamgrim.view_contract.v1",
    source: "observation_output",
    families: ["table"],
    strict: true,
    source_map: { table: { source: "observation_output", schema: "seamgrim.table.v0", available: true } },
  },
  runs: [],
  logs: [],
});
assert(inspector.includes("교과 제목:"), "inspector should localize lesson title label");
assert(inspector.includes("보기 계약 출처:"), "inspector should localize view contract labels");
assert(inspector.includes("기대값="), "inspector should localize expected label");
assert(!inspector.includes("lesson.title:"), "inspector should not expose lesson.title label");
assert(!inspector.includes("view_contract.source:"), "inspector should not expose view_contract.source label");
assert(!inspector.includes("expected="), "inspector should not expose expected= label");

console.log("seamgrim korean display label runner ok");
