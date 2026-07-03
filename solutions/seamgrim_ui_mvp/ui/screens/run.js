import {
  applyWasmLogicAndDispatchState,
  createManagedRafStepLoop,
  formatObservationCellValue,
  readWasmClientParseWarnings,
  stepWasmClientParsed,
} from "../wasm_page_common.js";
import {
  extractObservationChannelsFromState,
  extractObservationOutputLogFromState,
  extractObservationOutputLinesFromState,
  extractObservationOutputRowsFromState,
  extractStructuredViewsFromState,
  resolveStructuredViewStackFromState,
} from "../seamgrim_runtime_state.js";
import { applyControlValuesToDdnText } from "../components/control_parser.js";
import { Bogae } from "../components/bogae.js";
import { DotbogiPanel } from "../components/dotbogi.js";
import {
  buildGraphPreviewHtml,
  buildGraphSummaryMarkdown,
} from "../components/graph_preview.js";
import {
  buildStructurePreviewHtml,
  buildStructureSummaryMarkdown,
} from "../components/structure_preview.js";
import {
  hasSpatialViewFamily,
  normalizeViewFamilyList,
  resolveRunDockPanelOrderFromFamilies,
} from "../view_family_contract.js";
import {
  buildFamilyPreviewResult,
  buildPreviewResultCollection,
} from "../preview_result_contract.js";
import {
  applyPreviewViewModelMetadata,
  buildPreviewViewModel,
} from "../preview_view_model.js";
import { SliderPanel } from "../components/slider_panel.js";
import { OverlayDescription } from "../components/overlay.js";
import { markdownToHtml } from "../components/markdown.js";
import { showGlobalToast } from "../components/toast.js";
import { preprocessDdnText } from "../runtime/ddn_preprocess.js";
import { pickDdnFileFromLocal, readTextFromLocalFile, saveDdnToFile } from "./editor.js";
import { buildInspectorReport, formatInspectorReportText } from "../inspector_contract.js";
import {
  buildWarningShortcutHint,
  classifyToUserCategory,
  mapParseWarningToUserMessage,
  resolveUserWarningCause,
  resolveRuntimeGuideText,
} from "../run_warning_contract.js";
import { buildRuntimeHintViewModel } from "../run_runtime_hint_contract.js";
import { buildRunExecStatusViewModel } from "../run_exec_status_contract.js";
import { buildRunActionRailViewModel } from "../run_action_rail_contract.js";
import { buildWarningPanelViewModel } from "../run_warning_panel_contract.js";
import {
  buildFirstRunHintText,
  resolveFirstRunStepByTarget,
  SEAMGRIM_FIRST_RUN_PATH_TEXT,
} from "../first_run_catalog.js";
import {
  buildObserveActionPlan,
  OBSERVE_ACTION_OPEN_DDN_OBSERVE_OUTPUT,
  normalizeObserveAction,
} from "../run_observe_action_contract.js";
import {
  buildObserveFamilyActionHint,
  formatObserveFamilyName,
  summarizeObserveFamilyMetric as summarizeObserveFamilyMetricContract,
} from "../run_observe_family_contract.js";
import {
  buildClassroomAssignmentList,
  buildClassroomExportReport,
  buildClassroomRunResultSummary,
  formatClassroomExportReportText,
} from "../studio_classroom_mode.js";
import {
  buildStudioLocalPackageManifest,
  buildStudioLocalPackagePayload,
  formatStudioLocalPackageIndexText,
} from "../studio_local_share_package.js";
import {
  buildLessonPublicationReviewSurface,
  DEFAULT_LESSON_PUBLICATION_CANDIDATE_IDS,
  formatLessonPublicationReviewSurfaceText,
} from "../studio_lesson_publication_review_surface.js";
import { buildObserveSummaryViewModel } from "../run_observe_summary_contract.js";
import {
  resolveGraphTabMode,
  resolveSubpanelTabs,
  SUBPANEL_TAB,
  SUBPANEL_TAB_LABEL,
} from "../subpanel_tab_policy.js";
import { formatDisplayLabel, formatSourceLabel } from "../display_label_contract.js";
import {
  buildNumericTrackRunPreset,
  buildNumericTrackRunResultLink,
  formatNumericTrackRunResultLinkText,
  formatNumericTrackRunPresetText,
} from "../numeric_curriculum_track.js";

export { buildWarningShortcutHint, mapParseWarningToUserMessage };

const RUN_UI_PREFS_STORAGE_KEY = "seamgrim.ui.run_prefs.v1";
const DEFAULT_ONBOARDING_STATUS_TEXT = `온보딩: ${SEAMGRIM_FIRST_RUN_PATH_TEXT} 순서로 보세요.`;
const RUNTIME_TABLE_CELL_MAX_CHARS = 24;
const RUNTIME_TABLE_CELL_MIN_CHARS = 12;
const RUNTIME_TABLE_CELL_MAX_CHARS_LIMIT = 48;
const RUNTIME_TABLE_CELL_ESTIMATED_PX_PER_CHAR = 7;
const RUNTIME_TABLE_CELL_WIDTH_PADDING_PX = 80;
const RUNTIME_INPUT_MASK_LIMIT = (1 << 9) - 1;
const RUNTIME_INPUT_BITS = Object.freeze({
  ArrowUp: 1 << 0,
  ArrowLeft: 1 << 1,
  ArrowDown: 1 << 2,
  ArrowRight: 1 << 3,
  Space: 1 << 4,
  Enter: 1 << 5,
  Escape: 1 << 6,
  KeyZ: 1 << 7,
  KeyX: 1 << 8,
});
const RUN_TAB_IDS = Object.freeze(["graph", "console", "output", "mirror", "overlay"]);
const RUN_PRIMARY_VIEW_IDS = Object.freeze(["sim", "graph", "table"]);
const RUN_DOCK_TARGETS = Object.freeze(["space2d", "graph"]);
const RUN_DOCK_SPEEDS = Object.freeze([0.5, 1, 1.5, 2]);
const RUN_MANAGER_MAX_RUNS = 12;
const RUN_ENGINE_MODE_ONESHOT = "oneshot";
const RUN_ENGINE_MODE_LIVE = "live";
const RUN_ENGINE_MODE_IDS = Object.freeze([RUN_ENGINE_MODE_ONESHOT, RUN_ENGINE_MODE_LIVE]);
const RUN_ENGINE_STATUS_IDS = Object.freeze(["idle", "running", "paused", "done", "blocked", "fatal"]);
const OBSERVE_FAMILY_ORDER = Object.freeze(["space2d", "graph", "table", "text", "structure"]);
const OBSERVE_GUIDE_STATUS_TTL_MS = 3200;
const RUN_HINT_VIEW_SOURCE_WARN_LABEL = "보기소스경고";
const PLATFORM_SERVER_EXCHANGE_EVENT = "seamgrim:platform-server-adapter-exchange";
const PLATFORM_UI_ACTION_EVENT = "seamgrim:platform-ui-action";
const PLATFORM_UI_ACTION_LOGIN = "login";
const ADVANCED_EXPORTS_QUERY_KEY = "advancedExports";

function isLocalDevHost() {
  const host = String(globalThis?.location?.hostname ?? "").trim().toLowerCase();
  return !host || host === "localhost" || host === "127.0.0.1" || host === "::1";
}

function shouldEnableAdvancedExports() {
  try {
    if (!isLocalDevHost()) return false;
    if (globalThis?.SEAMGRIM_ADVANCED_EXPORTS === true) return true;
    const params = new URLSearchParams(globalThis?.location?.search ?? "");
    const raw = String(params.get(ADVANCED_EXPORTS_QUERY_KEY) ?? "").trim().toLowerCase();
    return raw === "1" || raw === "true" || raw === "yes";
  } catch (_) {
    return false;
  }
}

const INITIAL_ADVANCED_EXPORTS_ENABLED = shouldEnableAdvancedExports();

const ADVANCED_EXPORT_PANEL_HTML = `
  <div class="run-publication-prep-export hidden" data-run-publication-prep-export aria-label="공개 준비안 내보내기" aria-hidden="true">
    <div class="run-publication-prep-head">
      <span class="run-publication-prep-title">공개 준비안</span>
      <span class="run-publication-prep-meta" data-run-publication-prep-meta>기본 시작 · 후보 0개</span>
      <button id="btn-run-publication-prep-copy" type="button">준비안 복사</button>
    </div>
    <pre class="run-publication-prep-text" data-run-publication-prep-text>항목	값</pre>
  </div>
  <div class="run-registry-seed-export hidden" data-run-registry-seed-export aria-label="Registry seed 내보내기" aria-hidden="true">
    <div class="run-registry-seed-head">
      <span class="run-registry-seed-title">Registry seed</span>
      <span class="run-registry-seed-meta" data-run-registry-seed-meta>기본 시작 · seed 0개</span>
      <button id="btn-run-registry-seed-copy" type="button">Seed 복사</button>
    </div>
    <pre class="run-registry-seed-text" data-run-registry-seed-text>lesson_id	registry_id	draft_only	publish_claim</pre>
  </div>
  <div class="run-approval-continuity-export hidden" data-run-approval-continuity-export aria-label="승인 연속성 패킷 내보내기" aria-hidden="true">
    <div class="run-approval-continuity-head">
      <span class="run-approval-continuity-title">승인 연속성</span>
      <span class="run-approval-continuity-meta" data-run-approval-continuity-meta>기본 시작 · 승인 대기</span>
      <button id="btn-run-approval-continuity-copy" type="button">연속성 복사</button>
    </div>
    <pre class="run-approval-continuity-text" data-run-approval-continuity-text>항목	값</pre>
  </div>
  <div class="run-benchmark-lts-export hidden" data-run-benchmark-lts-export aria-label="Benchmark LTS matrix 내보내기" aria-hidden="true">
    <div class="run-benchmark-lts-head">
      <span class="run-benchmark-lts-title">Benchmark/LTS</span>
      <span class="run-benchmark-lts-meta" data-run-benchmark-lts-meta>기본 시작 · ready 0/0</span>
      <button id="btn-run-benchmark-lts-copy" type="button">Matrix 복사</button>
    </div>
    <pre class="run-benchmark-lts-text" data-run-benchmark-lts-text>matrix_id	kind	required	ready</pre>
  </div>
  <div class="run-education-operations-lts-export hidden" data-run-education-operations-lts-export aria-label="Education operations LTS 내보내기" aria-hidden="true">
    <div class="run-education-operations-lts-head">
      <span class="run-education-operations-lts-title">운영 LTS</span>
      <span class="run-education-operations-lts-meta" data-run-education-operations-lts-meta>기본 시작 · ready 0/0</span>
      <button id="btn-run-education-operations-lts-copy" type="button">운영 LTS 복사</button>
    </div>
    <pre class="run-education-operations-lts-text" data-run-education-operations-lts-text>operation_id	kind	required	ready</pre>
  </div>
`;

function buildSafeDownloadName(value, fallback = "seamgrim-file", extension = "json") {
  const base = String(value ?? "")
    .trim()
    .replace(/[^a-zA-Z0-9._-]+/g, "_")
    .replace(/^_+|_+$/g, "");
  const stem = base || fallback;
  const ext = String(extension ?? "").trim().replace(/^\.+/u, "") || "txt";
  return stem.toLowerCase().endsWith(`.${ext.toLowerCase()}`) ? stem : `${stem}.${ext}`;
}

function buildSafeJsonDownloadName(value, fallback = "seamgrim-teacher-package") {
  return buildSafeDownloadName(value, fallback, "json");
}

function saveJsonTextToFile(text, filename = "seamgrim-teacher-package.json") {
  const blob = new Blob([String(text ?? "")], { type: "application/json;charset=utf-8" });
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = buildSafeJsonDownloadName(filename);
  document.body?.appendChild?.(link);
  link.click();
  link.remove?.();
  setTimeout(() => URL.revokeObjectURL(link.href), 800);
}

function saveTsvTextToFile(text, filename = "seamgrim-student-results.tsv") {
  const blob = new Blob([String(text ?? "")], { type: "text/tab-separated-values;charset=utf-8" });
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = buildSafeDownloadName(filename, "seamgrim-student-results", "tsv");
  document.body?.appendChild?.(link);
  link.click();
  link.remove?.();
  setTimeout(() => URL.revokeObjectURL(link.href), 800);
}

function savePlainTextToFile(text, filename = "seamgrim-student-result.txt") {
  const blob = new Blob([String(text ?? "")], { type: "text/plain;charset=utf-8" });
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = buildSafeDownloadName(filename, "seamgrim-student-result", "txt");
  document.body?.appendChild?.(link);
  link.click();
  link.remove?.();
  setTimeout(() => URL.revokeObjectURL(link.href), 800);
}

function buildLocalPackageStudentGuideText(model = {}) {
  const guideReport = Array.isArray(model?.payload?.reports)
    ? model.payload.reports.find((report) => String(report?.title ?? report?.report_id ?? "").includes("학생 배포 안내문"))
    : null;
  const guideText = String(guideReport?.text ?? "").trim();
  if (guideText) return guideText;
  const title = String(model?.lesson_title ?? model?.manifest?.title ?? "셈그림 수업").trim() || "셈그림 수업";
  const packageId = String(model?.package_id ?? model?.manifest?.package_id ?? "").trim();
  const sessionLabel = String(model?.session_label ?? model?.manifest?.session_label ?? "").trim();
  const instructionRows = Array.isArray(model?.payload?.student_instructions)
    ? model.payload.student_instructions.map((item) => String(item ?? "").trim()).filter(Boolean)
    : [];
  const rows = [
    `${title} 배포 안내`,
    ...(sessionLabel ? [`차시: ${sessionLabel}`] : []),
    ...(instructionRows.length
      ? instructionRows.map((item, index) => `${index + 1}. ${item}`)
      : [
          "1. 셈그림 Studio에서 배포 열기 또는 붙여넣기 열기를 누릅니다.",
          "2. 선생님이 보낸 JSON 배포를 선택하거나 붙여넣습니다.",
          "3. 받은 수업 실행을 눌러 결과를 확인합니다.",
          "4. 이름을 입력하고 결과 복사 또는 결과 저장으로 교사에게 제출합니다.",
        ]),
  ];
  if (packageId && !instructionRows.some((item) => item.includes("배포 코드"))) {
    rows.push(`배포 코드: ${packageId}`);
  }
  if (!instructionRows.some((item) => item.includes("학생 이름") && item.includes("수업 코드") && item.includes("배포 코드"))) {
    rows.push("제출 결과에는 학생 이름, 차시, 수업 코드, 배포 코드, 상태 기록이 함께 들어갑니다.");
  }
  return rows.join("\n");
}

function buildLocalPackageStudentGuideFileName({ packageId = "seamgrim", sessionLabel = "" } = {}) {
  const packageText = String(packageId ?? "").trim() || "seamgrim";
  const session = String(sessionLabel ?? "").trim();
  const sessionSuffix = session ? `_${session}` : "";
  return buildSafeDownloadName(`${packageText}${sessionSuffix}_student_guide.txt`, "seamgrim-student-guide", "txt");
}

function buildLocalPackageGuideSummaryText(model = {}) {
  const instructionRows = Array.isArray(model?.payload?.student_instructions)
    ? model.payload.student_instructions.map((item) => String(item ?? "").trim()).filter(Boolean)
    : [];
  const contextRows = instructionRows
    .filter((item) => /^(차시|교과 수|첫 수업|첫 수업 코드|수업|수업 코드|배포 코드|목표|오늘 활동)\s*:/u.test(item))
    .slice(0, 6);
  const resultRow = instructionRows.find((item) => item.includes("결과 확인"));
  const submitRow = instructionRows.find((item) => item.includes("결과 복사") || item.includes("교사에게 제출"));
  const flowRow = resultRow
    ? `배포 열기 또는 붙여넣기 열기 → 받은 수업 실행 → ${resultRow.replace(/[.。]\s*$/, "")}`
    : "배포 열기 또는 붙여넣기 열기 → 받은 수업 실행 → 결과 확인";
  return [
    "학생 배포 안내문",
    ...contextRows,
    flowRow,
    ...(submitRow ? [submitRow.replace(/[.。]\s*$/, "")] : []),
  ].join("\n");
}

function buildLocalPackageMaterialsSummaryText(model = {}) {
  const payload = model?.payload ?? {};
  const lessons = Array.isArray(payload.lessons) ? payload.lessons : [];
  const reports = Array.isArray(payload.reports) ? payload.reports : [];
  const lessonLabel = lessons.length > 0 ? `교과 ${lessons.length}개` : "교과 없음";
  const reportLabels = reports
    .map((item) => String(item?.title ?? item?.report_id ?? "").trim())
    .filter(Boolean);
  const reportText = reportLabels.length ? reportLabels.join(", ") : "교사용 자료 없음";
  return `포함 자료: ${lessonLabel} · ${reportText}`;
}

function getLocalPackageReportsFromLesson(lesson = {}) {
  const directReports = Array.isArray(lesson?.localPackageReports) ? lesson.localPackageReports : [];
  if (directReports.length > 0) return directReports;
  const payloadReports = Array.isArray(lesson?.localPackagePayload?.reports) ? lesson.localPackagePayload.reports : [];
  return payloadReports;
}

function findLocalPackageReportFromLesson(lesson = {}, matcher = () => false) {
  return getLocalPackageReportsFromLesson(lesson).find((report) => matcher({
    ...report,
    title: String(report?.title ?? "").trim(),
    report_id: String(report?.report_id ?? "").trim(),
    path: String(report?.path ?? "").trim(),
    text: String(report?.text ?? "").trim(),
  })) ?? null;
}

function localPackageReportFileName(report = {}, fallback = "seamgrim-material.txt") {
  const path = String(report?.path ?? "").replace(/\\/g, "/").trim();
  const name = path.split("/").filter(Boolean).pop();
  return buildSafeDownloadName(name || fallback, fallback.replace(/\.[^.]+$/u, "") || "seamgrim-material", name?.split(".").pop() || "txt");
}

function resolveLocalPackageResultInstruction(lesson = {}) {
  const rows = Array.isArray(lesson?.localPackageStudentInstructions)
    ? lesson.localPackageStudentInstructions.map((item) => String(item ?? "").trim()).filter(Boolean)
    : [];
  return rows.find((item) => item.includes("결과 확인")) || "";
}

function renderLocalPackageStudentInstructionHtml(lesson = {}) {
  const rows = Array.isArray(lesson?.localPackageStudentInstructions)
    ? lesson.localPackageStudentInstructions.map((item) => String(item ?? "").trim()).filter(Boolean)
    : [];
  const sourceMaterials = Array.isArray(lesson?.localPackageStudentMaterialsSummary)
    ? lesson.localPackageStudentMaterialsSummary
    : lesson?.localPackageMaterialsSummary;
  const materials = Array.isArray(sourceMaterials)
    ? sourceMaterials.map((item) => String(item ?? "").trim()).filter(Boolean)
    : [];
  if (rows.length === 0 && materials.length === 0) return "";
  const hasMultipleLessons = rows.some((row) => /^교과 수\s*:/u.test(row));
  const selectedLessonTitle = String(lesson?.title ?? "").trim();
  const selectedLessonId = String(lesson?.localPackageLessonId ?? lesson?.id ?? "").trim();
  const selectedRows = hasMultipleLessons
    ? [
        ...(selectedLessonTitle && !rows.some((row) => row === `오늘 수업: ${selectedLessonTitle}`) ? [`오늘 수업: ${selectedLessonTitle}`] : []),
        ...(selectedLessonId && !rows.some((row) => row === `오늘 수업 코드: ${selectedLessonId}`) ? [`오늘 수업 코드: ${selectedLessonId}`] : []),
      ]
    : [];
  const selectedInsertIndex = selectedRows.length
    ? Math.max(
        rows.findIndex((row) => /^첫 수업 코드\s*:/u.test(row)),
        rows.findIndex((row) => /^첫 수업\s*:/u.test(row)),
        rows.findIndex((row) => /^교과 수\s*:/u.test(row)),
      ) + 1
    : 0;
  const instructionRows = selectedRows.length
    ? [
        ...rows.slice(0, Math.max(0, selectedInsertIndex)),
        ...selectedRows,
        ...rows.slice(Math.max(0, selectedInsertIndex)),
      ]
    : rows;
  return `
    <div class="run-delivery-instructions-title">받은 수업 안내</div>
    ${materials.length ? `
      <div class="run-delivery-instructions-materials">받은 자료: ${escapeHtml(materials.join(" · "))}</div>
    ` : ""}
    ${instructionRows.length ? `
      <ol class="run-delivery-instructions-list">
        ${instructionRows.map((row) => `<li>${escapeHtml(row)}</li>`).join("")}
      </ol>
    ` : ""}
  `;
}

function buildLessonOnboardingStatusText(lesson, fallback = DEFAULT_ONBOARDING_STATUS_TEXT) {
  const step = resolveFirstRunStepByTarget({
    id: lesson?.id,
    source: lesson?.source,
    firstRunPath: lesson?.firstRunPath,
  });
  return buildFirstRunHintText(step) || fallback;
}
const PLATFORM_UI_ACTION_REQUEST_ACCESS = "request_access";
const PLATFORM_UI_ACTION_OPEN_LOCAL_SAVE = "open_local_save";
const STUDIO_EDITOR_RATIO_DEFAULT = 2 / 3;
const STUDIO_EDITOR_RATIO_MIN = 0.15;
const STUDIO_EDITOR_RATIO_MAX = 0.85;
const STUDIO_EDITOR_RATIO_STORAGE_KEY_V1 = "seamgrim.ui.studio_editor_ratio.v1";
const STUDIO_EDITOR_RATIO_STORAGE_KEY = "seamgrim.ui.studio_editor_ratio.v3";
const STUDIO_LAYOUT_SPLITTER_WIDTH = 6;
const STUDIO_LAYOUT_MIN_VISUAL_WIDTH_PX = 420;
const STUDIO_LAYOUT_SUBPANEL_MIN_HEIGHT_PX = 300;
const STUDIO_LAYOUT_BOGAE_ASPECT_RATIO = 16 / 9;
const STUDIO_VIEW_MODE_BASIC = "basic";
const STUDIO_VIEW_MODE_ANALYZE = "analyze";
const STUDIO_VIEW_MODE_FULL = "full";
const STUDIO_VIEW_MODE_IDS = Object.freeze([
  STUDIO_VIEW_MODE_BASIC,
  STUDIO_VIEW_MODE_ANALYZE,
  STUDIO_VIEW_MODE_FULL,
]);
const STUDIO_VIEW_MODE_STORAGE_KEY = "seamgrim.ui.run_view_mode.v1";
const BOGAE_TOOLBAR_COMPACT_THRESHOLD_PX = 860;
const RUN_PRIMARY_VIEW_STORAGE_KEY = "seamgrim.ui.run_primary_view.v1";
const RUN_MINIMAL_UI_STORAGE_KEY = "seamgrim.ui.run_minimal_ui.v1";
const LEGACY_RANGE_COMMENT_RE = /\/\/\s*범위\s*\(/u;
const LEGACY_RANGE_HASH_RE = /#\s*범위\s*(?:\(|:)/u;
const LEGACY_SETUP_COLON_RE = /^\s*채비\s*:\s*\{/mu;
const LEGACY_HOOK_COLON_RE = /^\s*\(\s*(?:시작|처음|매마디|매틱|[1-9][0-9]*\s*마디)\s*\)\s*(?:할때|마다)\s*:\s*\{/mu;
const LEGACY_HOOK_ALIAS_RE = /\(\s*(?:처음|매틱)\s*\)\s*(?:할때|마다)/u;
const LEGACY_SETTING_ALIAS_RE = /^\s*(?:title|desc)\s*:/imu;
const LEGACY_MAX_MADI_RE = /최대마디/u;
const RUN_MAIN_EXECUTE_LABEL_DEFAULT = "▶ 수업 실행";
const RUN_MAIN_EXECUTE_LABEL_COMPACT = "▶ 실행";
const RUN_MAIN_EXECUTE_LABEL_LOCAL_PACKAGE = "▶ 받은 수업 실행";
const RUN_MAIN_EXECUTE_LABEL_LOCAL_PACKAGE_COMPACT = "▶ 받은 수업";
const RUN_MAIN_EXECUTE_LABEL_RESUME = "▶ 재개";
const RUN_MAIN_PAUSE_LABEL_DEFAULT = "⏸ 일시정지";
const RUN_MAIN_PAUSE_LABEL_COMPACT = "⏸ 일시정지";
const RUN_MAIN_RESET_LABEL = "↺ 초기화";
const RUN_MAIN_STEP_LABEL = "▷ 한 단계씩";
const RUN_FINITE_LIVE_FPS = 6;
const EMPTY_INITIAL_WASM_STATE_HASH = "blake3:352bd266dae53c6e6a29244011cfa029813d0ab8434b2a2b830a487d882832ba";

export function resolveRunEngineModeFromDdnText(text = "") {
  const source = String(text ?? "");
  const tickHookRe = /^\s*\(\s*(?:매마디|매틱|[1-9][0-9]*\s*마디)\s*\)\s*마다\s*\{/mu;
  return tickHookRe.test(source) ? RUN_ENGINE_MODE_LIVE : RUN_ENGINE_MODE_ONESHOT;
}

function readConfiguredMadiFromClient(client) {
  if (!client || typeof client.configuredMadi !== "function") return 0;
  const value = Number(client.configuredMadi());
  return Number.isFinite(value) && value > 0 ? Math.floor(value) : 0;
}

export function readConfiguredMadiFromDdnText(text = "") {
  const source = String(text ?? "");
  const blocks = source.matchAll(/설정\s*\{([\s\S]*?)\}\s*\.?/gu);
  for (const match of blocks) {
    const body = String(match?.[1] ?? "");
    const valueMatch = body.match(/(?:마디수|max_madi|maxMadi|ticks?)\s*:\s*([1-9][0-9]*)\s*\.?/u);
    if (!valueMatch) continue;
    const value = Number(valueMatch[1]);
    if (Number.isFinite(value) && value > 0) return Math.floor(value);
  }
  const resourceMatch = source.match(
    /^\s*마디수\s*(?::\s*[^<=>\r\n]+?)?\s*(?:<-|=)\s*\(?\s*([1-9][0-9]*)(?:\.0+)?\s*\)?(?:\s+매김(?=\s|\{|\.|$)|\s*\.|\s*$)/mu,
  );
  if (resourceMatch) {
    const value = Number(resourceMatch[1]);
    if (Number.isFinite(value) && value > 0) return Math.floor(value);
  }
  return 0;
}

function normalizeRunEngineMode(mode, fallback = RUN_ENGINE_MODE_ONESHOT) {
  const normalized = String(mode ?? "").trim().toLowerCase();
  if (RUN_ENGINE_MODE_IDS.includes(normalized)) return normalized;
  return RUN_ENGINE_MODE_IDS.includes(fallback) ? fallback : RUN_ENGINE_MODE_ONESHOT;
}

export function resolveRunLoopFps({
  fpsLimit = 30,
  playbackSpeed = 1,
  engineMode = RUN_ENGINE_MODE_ONESHOT,
  runtimeMaxMadi = 0,
} = {}) {
  const rawBase = Math.max(1, Number(fpsLimit ?? 30) || 30);
  const finiteLive =
    normalizeRunEngineMode(engineMode, RUN_ENGINE_MODE_ONESHOT) === RUN_ENGINE_MODE_LIVE
    && Math.max(0, Math.trunc(Number(runtimeMaxMadi) || 0)) > 0;
  const base = finiteLive ? Math.min(rawBase, RUN_FINITE_LIVE_FPS) : rawBase;
  const speed = normalizeDockSpeed(playbackSpeed);
  return Math.max(1, Math.round(base * speed));
}

export function hasReachedRuntimeMaxMadi(runtimeTickCounter = 0, runtimeMaxMadi = 0) {
  const current = Math.max(0, Math.trunc(Number(runtimeTickCounter) || 0));
  const max = Math.max(0, Math.trunc(Number(runtimeMaxMadi) || 0));
  return max > 0 && current >= max;
}

function normalizeLegacyInitExpr(raw) {
  const token = String(raw ?? "").trim();
  if (!token) return "(0)";
  if (token.startsWith("(") && token.endsWith(")")) return token;
  return `(${token})`;
}

export function resolveBogaeToolbarCompact({
  toolbarWidth = 0,
  visualColumnWidth = 0,
  thresholdPx = BOGAE_TOOLBAR_COMPACT_THRESHOLD_PX,
} = {}) {
  const threshold = Math.max(1, Number.isFinite(Number(thresholdPx)) ? Number(thresholdPx) : BOGAE_TOOLBAR_COMPACT_THRESHOLD_PX);
  const toolbar = Number(toolbarWidth);
  const visual = Number(visualColumnWidth);
  const width = Number.isFinite(toolbar) && toolbar > 0
    ? toolbar
    : (Number.isFinite(visual) && visual > 0 ? visual : 0);
  if (!(width > 0)) return false;
  return width < threshold;
}

function normalizeStudioViewMode(raw, fallback = STUDIO_VIEW_MODE_BASIC) {
  const mode = String(raw ?? "").trim().toLowerCase();
  if (STUDIO_VIEW_MODE_IDS.includes(mode)) return mode;
  return STUDIO_VIEW_MODE_IDS.includes(fallback) ? fallback : STUDIO_VIEW_MODE_BASIC;
}

export function resolveRunMainControlLabels({ isPaused = false, compact = false, launchKind = "" } = {}) {
  const compactMode = Boolean(compact);
  const isLocalPackageImport = normalizeRunLaunchKind(launchKind) === "local_package_import";
  return {
    execute: isPaused
      ? RUN_MAIN_EXECUTE_LABEL_RESUME
      : isLocalPackageImport
        ? (compactMode ? RUN_MAIN_EXECUTE_LABEL_LOCAL_PACKAGE_COMPACT : RUN_MAIN_EXECUTE_LABEL_LOCAL_PACKAGE)
        : (compactMode ? RUN_MAIN_EXECUTE_LABEL_COMPACT : RUN_MAIN_EXECUTE_LABEL_DEFAULT),
    pause: compactMode ? RUN_MAIN_PAUSE_LABEL_COMPACT : RUN_MAIN_PAUSE_LABEL_DEFAULT,
    reset: RUN_MAIN_RESET_LABEL,
    step: RUN_MAIN_STEP_LABEL,
  };
}

function formatCompactStateHash(hashText) {
  const value = String(hashText ?? "").trim();
  if (!value || value === "-") return "상태 기록: -";
  if (value.length <= 28) return `상태 기록: ${value}`;
  return `상태 기록: ${value.slice(0, 12)}...${value.slice(-8)}`;
}

function buildStudentResultReturnText({ lesson = null, hashText = "", resultInstruction = "", studentName = "" } = {}) {
  const title = String(lesson?.title ?? lesson?.id ?? "받은 수업").trim() || "받은 수업";
  const lessonId = String(lesson?.localPackageLessonId ?? lesson?.id ?? "").trim();
  const student = String(studentName ?? "").trim();
  const sessionLabel = String(lesson?.localPackageSessionLabel ?? "").trim();
  const packageId = String(lesson?.localPackageId ?? "").trim();
  const packageTitle = String(lesson?.localPackageTitle ?? "").trim();
  const result = String(resultInstruction ?? "").trim() || resolveLocalPackageResultInstruction(lesson);
  const hash = String(hashText ?? "").trim();
  return [
    ...(student ? [`학생: ${student}`] : []),
    ...(sessionLabel ? [`차시: ${sessionLabel}`] : []),
    `수업: ${title}`,
    ...(lessonId ? [`수업 코드: ${lessonId}`] : []),
    ...(packageTitle ? [`배포 묶음: ${packageTitle}`] : []),
    ...(packageId ? [`배포 코드: ${packageId}`] : []),
    ...(result ? [result.replace(/[.。]\s*$/, "")] : []),
    `상태 기록: ${hash}`,
  ].join("\n");
}

function getStudentResultAllowedPackageLessons(lesson = null) {
  const rows = Array.isArray(lesson?.localPackagePayload?.lessons)
    ? lesson.localPackagePayload.lessons
    : [];
  if (!String(lesson?.localPackageId ?? "").trim() || rows.length <= 1) return [];
  return rows
    .map((row) => ({
      lesson_id: String(row?.lesson_id ?? "").trim(),
      title: String(row?.title ?? row?.lesson_id ?? "").trim(),
    }))
    .filter((row) => row.lesson_id || row.title);
}

function findStudentResultAllowedPackageLesson({ lesson = null, lessonId = "", title = "" } = {}) {
  const rows = getStudentResultAllowedPackageLessons(lesson);
  if (!rows.length) return null;
  const id = String(lessonId ?? "").trim();
  const name = String(title ?? "").trim();
  return rows.find((row) => (
    (id && row.lesson_id === id)
      || (name && row.title === name)
  )) ?? null;
}

function parseStudentResultReturnText(text, { lesson = null } = {}) {
  const source = String(text ?? "");
  const rows = source
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  const fields = new Map();
  rows.forEach((line) => {
    const index = line.indexOf(":");
    if (index <= 0) return;
    const key = line.slice(0, index).trim();
    const value = line.slice(index + 1).trim();
    if (key) fields.set(key, value);
  });
  const studentName = fields.get("학생") || fields.get("이름") || fields.get("student") || "";
  const sessionLabel = fields.get("차시") || fields.get("세션") || fields.get("session") || "";
  const title = fields.get("수업") || "";
  const lessonId = fields.get("수업 코드") || fields.get("lesson_id") || fields.get("lesson id") || "";
  const packageTitle = fields.get("배포 묶음") || "";
  const packageId = fields.get("배포 코드") || fields.get("package_id") || fields.get("package id") || "";
  const stateHash = fields.get("상태 기록") || "";
  const reportStatus = fields.get("확인 상태") || "";
  const reportNotes = fields.get("비고") || "";
  const hasStudentName = Boolean(String(studentName ?? "").trim());
  const expectedTitle = String(lesson?.title ?? lesson?.id ?? "").trim();
  const expectedSessionLabel = String(lesson?.localPackageSessionLabel ?? "").trim();
  const expectedLessonId = String(lesson?.localPackageLessonId ?? lesson?.id ?? "").trim();
  const expectedPackageTitle = String(lesson?.localPackageTitle ?? "").trim();
  const expectedPackageId = String(lesson?.localPackageId ?? "").trim();
  const sessionMatch = Boolean(!expectedSessionLabel || !sessionLabel || sessionLabel === expectedSessionLabel);
  const packageMatch = Boolean(!expectedPackageTitle || !packageTitle || packageTitle === expectedPackageTitle);
  const packageIdMatch = Boolean(!expectedPackageId || !packageId || packageId === expectedPackageId);
  const allowedPackageLesson = packageIdMatch
    ? findStudentResultAllowedPackageLesson({ lesson, lessonId, title })
    : null;
  const hasAllowedPackageLessons = getStudentResultAllowedPackageLessons(lesson).length > 0;
  const lessonMatch = Boolean(
    !expectedTitle
      || !title
      || title === expectedTitle
      || (hasAllowedPackageLessons && allowedPackageLesson?.title === title),
  );
  const lessonIdMatch = Boolean(
    !expectedLessonId
      || !lessonId
      || lessonId === expectedLessonId
      || (hasAllowedPackageLessons && allowedPackageLesson?.lesson_id === lessonId),
  );
  const reportStatusAccepted = !reportStatus || reportStatus === "확인됨";
  const reportNotesClean = String(reportNotes ?? "").trim();
  const reportNotesAccepted = !reportNotesClean || reportNotesClean === "-";
  const accepted = Boolean(hasStudentName && stateHash && stateHash !== "-" && sessionMatch && lessonMatch && lessonIdMatch && packageMatch && packageIdMatch && reportStatusAccepted && reportNotesAccepted);
  return {
    schema: "seamgrim.student_result_return_review.v1",
    accepted,
    student_name: studentName,
    student_name_missing: !hasStudentName,
    session_label: sessionLabel,
    lesson_title: title,
    lesson_id: lessonId,
    package_title: packageTitle,
    package_id: packageId,
    state_hash: stateHash,
    report_status: reportStatus,
    report_notes: reportNotes,
    session_match: sessionMatch,
    lesson_match: lessonMatch,
    lesson_id_match: lessonIdMatch,
    package_match: packageMatch,
    package_id_match: packageIdMatch,
    account_required: false,
    cloud_sync: false,
    permission_system: false,
  };
}

function isStudentResultReturnReviewCoreAccepted(review = {}) {
  return Boolean(
    String(review?.student_name ?? "").trim()
      && String(review?.state_hash ?? "").trim()
      && String(review?.state_hash ?? "").trim() !== "-"
      && review?.session_match !== false
      && review?.lesson_match !== false
      && review?.lesson_id_match !== false
      && review?.package_match !== false
      && review?.package_id_match !== false,
  );
}

function splitStudentResultReturnBlocks(text) {
  const source = String(text ?? "").trim();
  if (!source) return [];
  const lines = source.split(/\r?\n/u).map((line) => line.trim());
  const tableHeaderIndex = lines.findIndex((line) => line === "학생\t차시\t수업\t수업 코드\t배포 묶음\t배포 코드\t확인 상태\t상태 기록\t비고");
  if (tableHeaderIndex >= 0) {
    const prefixBlocks = lines.slice(0, tableHeaderIndex)
      .join("\n")
      .split(/\r?\n\s*\r?\n/u)
      .map((block) => block.trim())
      .filter((block) => /^\s*(?:학생|student)\s*:/imu.test(block) && /상태\s*기록\s*:/u.test(block));
    const tableBlocks = lines.slice(tableHeaderIndex + 1)
      .filter((line) => line && !line.startsWith("#"))
      .map((line) => line.split("\t"))
      .filter((cells) => cells.length >= 8)
      .map((cells) => [
        ...(String(cells[0] ?? "").trim() ? [`학생: ${String(cells[0] ?? "").trim()}`] : []),
        ...(String(cells[1] ?? "").trim() ? [`차시: ${String(cells[1] ?? "").trim()}`] : []),
        ...(String(cells[2] ?? "").trim() ? [`수업: ${String(cells[2] ?? "").trim()}`] : []),
        ...(String(cells[3] ?? "").trim() ? [`수업 코드: ${String(cells[3] ?? "").trim()}`] : []),
        ...(String(cells[4] ?? "").trim() ? [`배포 묶음: ${String(cells[4] ?? "").trim()}`] : []),
        ...(String(cells[5] ?? "").trim() ? [`배포 코드: ${String(cells[5] ?? "").trim()}`] : []),
        ...(String(cells[6] ?? "").trim() ? [`확인 상태: ${String(cells[6] ?? "").trim()}`] : []),
        `상태 기록: ${String(cells[7] ?? "").trim()}`,
        ...(String(cells[8] ?? "").trim() ? [`비고: ${String(cells[8] ?? "").trim()}`] : []),
      ].join("\n"))
      .filter(Boolean);
    if (tableBlocks.length > 0) return [...prefixBlocks, ...tableBlocks];
  }
  const paragraphs = source.split(/\r?\n\s*\r?\n/u).map((block) => block.trim()).filter(Boolean);
  if (paragraphs.length > 1) return paragraphs;
  const blocks = [];
  let current = [];
  source.split(/\r?\n/u).forEach((line) => {
    if (/^\s*(?:학생|student)\s*:/iu.test(line) && current.length > 0) {
      blocks.push(current.join("\n").trim());
      current = [];
    }
    current.push(line);
  });
  const tail = current.join("\n").trim();
  if (tail) blocks.push(tail);
  return blocks.length ? blocks : [source];
}

function normalizeStudentResultReturnBlockKey(block) {
  return String(block ?? "")
    .replace(/\r\n?/gu, "\n")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .join("\n")
    .trim();
}

function collectStudentResultReturnBlockKeys(text) {
  return new Set(
    splitStudentResultReturnBlocks(text)
      .map((block) => normalizeStudentResultReturnBlockKey(block))
      .filter(Boolean),
  );
}

function normalizeStudentRosterName(name) {
  return String(name ?? "").trim().replace(/\s+/gu, " ").toLowerCase();
}

function isStudentRosterIndexCell(cell) {
  return /^(?:번호|순번|no\.?|#|\d+|[A-Z]\d{0,3})$/iu.test(String(cell ?? "").trim());
}

function isStudentRosterNameHeaderCell(cell) {
  return /^(?:학생\s*)?(?:이름|성명|name)\s*[:：]?$/iu.test(String(cell ?? "").trim());
}

function isStudentRosterStudentHeaderCell(cell) {
  const text = String(cell ?? "").trim();
  return text === "학생" || isStudentRosterNameHeaderCell(text);
}

function isStudentRosterTableHeaderCells(cells = []) {
  const first = String(cells[0] ?? "").trim();
  const second = String(cells[1] ?? "").trim();
  if (isStudentRosterStudentHeaderCell(first)) return true;
  return isStudentRosterIndexCell(first) && isStudentRosterNameHeaderCell(second);
}

function extractStudentRosterNameFromCells(cells = []) {
  if (!Array.isArray(cells) || cells.length === 0) return "";
  if (isStudentRosterTableHeaderCells(cells)) return "";
  const first = String(cells[0] ?? "").trim();
  const second = String(cells[1] ?? "").trim();
  if (!first) return "";
  if (isStudentRosterIndexCell(first)) return second;
  return first;
}

function extractStudentRosterRecordFromCells(cells = []) {
  if (!Array.isArray(cells) || cells.length === 0) return null;
  const name = normalizeStudentRosterEntry(extractStudentRosterNameFromCells(cells));
  if (!name) return null;
  const first = String(cells[0] ?? "").trim();
  const hasCourseColumns = cells.length >= 6 && !isStudentRosterIndexCell(first);
  return {
    name,
    session_label: hasCourseColumns ? String(cells[1] ?? "").trim() : "",
    lesson_title: hasCourseColumns ? String(cells[2] ?? "").trim() : "",
    lesson_id: hasCourseColumns ? String(cells[3] ?? "").trim() : "",
    package_title: hasCourseColumns ? String(cells[4] ?? "").trim() : "",
    package_id: hasCourseColumns ? String(cells[5] ?? "").trim() : "",
  };
}

function isStudentRosterMetadataLine(line) {
  const raw = String(line ?? "").trim();
  if (!raw) return true;
  if (raw.startsWith("#")) return true;
  const firstCell = raw.split(/[\t,;]+/u)[0]?.trim() || "";
  return /^(?:수업(?:\s*코드)?|lesson(?:[_\s-]?id)?|차시|세션|session(?:[_\s-]?label)?|배포(?:\s*(?:묶음|코드))?|package(?:[_\s-]?id)?)$/iu.test(firstCell);
}

function normalizeStudentRosterEntry(entry) {
  const source = String(entry ?? "");
  const raw = source.trim();
  if (!raw) return "";
  if (isStudentRosterMetadataLine(source)) return "";
  if (source.includes("\t")) {
    const tabCells = source
      .split(/\t/u)
      .map((cell) => cell.trim());
    const tableName = extractStudentRosterNameFromCells(tabCells);
    if (!tableName) return "";
    return normalizeStudentRosterEntry(tableName);
  }
  const cells = raw
    .split(/\t/u)
    .map((cell) => cell.trim())
    .filter(Boolean);
  const candidate = cells.length >= 2
    ? cells.find((cell) => !isStudentRosterIndexCell(cell) && !isStudentRosterNameHeaderCell(cell)) || cells[cells.length - 1]
    : raw;
  const withoutHeader = candidate.replace(/^(?:학생\s*)?(?:이름|성명|name)\s*[:：]?\s*$/iu, "").trim();
  if (!withoutHeader) return "";
  if (isStudentRosterIndexCell(withoutHeader)) return "";
  return withoutHeader
    .replace(/^\s*(?:\d{1,4}|[A-Z]\d{0,3})\s*[.)\]-]?\s+/u, "")
    .replace(/\s+/gu, " ")
    .trim();
}

function collectStudentRosterEntries(text) {
  const source = String(text ?? "").trim();
  if (!source) return [];
  const entries = [];
  source
    .split(/\r?\n/u)
    .forEach((line) => {
      if (!String(line ?? "").trim()) return;
      if (isStudentRosterMetadataLine(line)) return;
      const tabCells = line.split(/\t/u).map((cell) => cell.trim());
      if (tabCells.length >= 2 && line.includes("\t")) {
        const tableName = extractStudentRosterNameFromCells(tabCells);
        if (tableName) entries.push(tableName);
        return;
      }
      const trimmedLine = String(line ?? "").trim();
      const separatedCells = trimmedLine.split(/[;,]+/u).map((cell) => cell.trim()).filter(Boolean);
      if (separatedCells.length >= 2) {
        const hasColumnSignal = separatedCells.some((cell) => isStudentRosterIndexCell(cell) || isStudentRosterStudentHeaderCell(cell));
        if (hasColumnSignal) {
          const tableName = extractStudentRosterNameFromCells(separatedCells);
          if (tableName) entries.push(tableName);
        } else {
          entries.push(...separatedCells);
        }
        return;
      }
      entries.push(trimmedLine);
    });
  return entries
    .map((name) => normalizeStudentRosterEntry(name))
    .filter(Boolean);
}

function collectStudentRosterRecords(text) {
  const source = String(text ?? "").trim();
  if (!source) return [];
  const records = [];
  source
    .split(/\r?\n/u)
    .forEach((line) => {
      if (!String(line ?? "").trim()) return;
      if (isStudentRosterMetadataLine(line)) return;
      if (line.includes("\t")) {
        const tabCells = line.split(/\t/u).map((cell) => cell.trim());
        const record = extractStudentRosterRecordFromCells(tabCells);
        if (record) records.push(record);
        return;
      }
      const trimmedLine = String(line ?? "").trim();
      const separatedCells = trimmedLine.split(/[;,]+/u).map((cell) => cell.trim()).filter(Boolean);
      if (separatedCells.length >= 2) {
        const hasColumnSignal = separatedCells.some((cell) => isStudentRosterIndexCell(cell) || isStudentRosterStudentHeaderCell(cell));
        if (hasColumnSignal) {
          const record = extractStudentRosterRecordFromCells(separatedCells);
          if (record) records.push(record);
        } else {
          separatedCells
            .map((name) => normalizeStudentRosterEntry(name))
            .filter(Boolean)
            .forEach((name) => records.push({
              name,
              session_label: "",
              lesson_title: "",
              lesson_id: "",
              package_title: "",
              package_id: "",
            }));
        }
        return;
      }
      const name = normalizeStudentRosterEntry(trimmedLine);
      if (name) {
        records.push({
          name,
          session_label: "",
          lesson_title: "",
          lesson_id: "",
          package_title: "",
          package_id: "",
        });
      }
    });
  return records;
}

function buildStudentRosterDuplicateKey(record = {}) {
  const name = normalizeStudentRosterName(record?.name);
  if (!name) return "";
  const lessonKey = [
    String(record?.package_id ?? "").trim().toLowerCase(),
    String(record?.session_label ?? "").trim().toLowerCase(),
    String(record?.lesson_id ?? record?.lesson_title ?? "").trim().toLowerCase(),
  ].filter(Boolean).join("|");
  return lessonKey ? `${name}|${lessonKey}` : name;
}

function studentRosterRecordHasCourseContext(record = {}) {
  return Boolean(String(
    record?.lesson_id
      ?? record?.lesson_title
      ?? record?.package_id
      ?? record?.session_label
      ?? "",
  ).trim());
}

function buildStudentRosterExpectationKeyFromParts({
  name = "",
  packageId = "",
  sessionLabel = "",
  lessonId = "",
  lessonTitle = "",
} = {}) {
  const student = normalizeStudentRosterName(name);
  if (!student) return "";
  const lessonKey = [
    String(packageId ?? "").trim().toLowerCase(),
    String(sessionLabel ?? "").trim().toLowerCase(),
    String(lessonId || lessonTitle || "").trim().toLowerCase(),
  ].filter(Boolean).join("|");
  return lessonKey ? `${student}|${lessonKey}` : student;
}

function buildStudentRosterExpectationKeyFromRecord(record = {}) {
  return buildStudentRosterExpectationKeyFromParts({
    name: record.name,
    packageId: record.package_id,
    sessionLabel: record.session_label,
    lessonId: record.lesson_id,
    lessonTitle: record.lesson_title,
  });
}

function buildStudentRosterExpectationKeyFromResultRow(row = {}) {
  return buildStudentRosterExpectationKeyFromParts({
    name: row.student_name,
    packageId: row.package_id,
    sessionLabel: row.session_label,
    lessonId: row.lesson_id,
    lessonTitle: row.lesson_title,
  });
}

function analyzeStudentRosterText(text) {
  const records = collectStudentRosterRecords(text);
  const counts = new Map();
  const displayByKey = new Map();
  records.forEach((record) => {
    const key = buildStudentRosterDuplicateKey(record);
    if (!key) return;
    counts.set(key, Number(counts.get(key) ?? 0) + 1);
    if (!displayByKey.has(key)) displayByKey.set(key, record.name);
  });
  const duplicateNames = Array.from(counts.entries())
    .filter(([, count]) => count > 1)
    .map(([key]) => displayByKey.get(key))
    .filter(Boolean);
  return {
    entry_count: records.length,
    duplicate_count: duplicateNames.length,
    duplicate_names: duplicateNames,
  };
}

function parseStudentRosterText(text) {
  const seen = new Set();
  return collectStudentRosterEntries(text)
    .filter((name) => {
      const key = normalizeStudentRosterName(name);
      if (!key || seen.has(key)) return false;
      seen.add(key);
      return true;
    });
}

function countStudentRosterExpectations(text) {
  const records = collectStudentRosterRecords(text);
  const courseRecords = records.filter((record) => studentRosterRecordHasCourseContext(record));
  if (courseRecords.length > 0) {
    const keys = new Set(courseRecords
      .map((record) => buildStudentRosterExpectationKeyFromRecord(record))
      .filter(Boolean));
    return keys.size;
  }
  return parseStudentRosterText(text).length;
}

function hasStudentRosterCourseExpectations(text) {
  return collectStudentRosterRecords(text).some((record) => studentRosterRecordHasCourseContext(record));
}

function shouldUseStudentRosterTargetCountLabelFromRecords(records = []) {
  const courseRecords = (Array.isArray(records) ? records : [])
    .filter((record) => studentRosterRecordHasCourseContext(record));
  if (!courseRecords.length) return false;
  const targetKeys = new Set(courseRecords
    .map((record) => buildStudentRosterExpectationKeyFromRecord(record))
    .filter(Boolean));
  const studentKeys = new Set(courseRecords
    .map((record) => normalizeStudentRosterName(record?.name))
    .filter(Boolean));
  return targetKeys.size > studentKeys.size;
}

function shouldUseStudentRosterTargetCountLabel(text) {
  return shouldUseStudentRosterTargetCountLabelFromRecords(collectStudentRosterRecords(text));
}

function formatStudentRosterExpectationLabel(count, { courseScoped = false } = {}) {
  const safeCount = Math.max(0, Number(count) || 0);
  return courseScoped ? `제출 대상 ${safeCount}건` : `명단 ${safeCount}명`;
}

function parseStudentRosterMetadataText(text) {
  const metadata = {
    lesson_id: "",
    session_label: "",
    package_id: "",
  };
  String(text ?? "")
    .split(/\r?\n/u)
    .map((line) => line.trim())
    .filter(Boolean)
    .forEach((line) => {
      if (line.startsWith("#")) return;
      const cells = line.split(/[\t,;]+/u).map((cell) => cell.trim());
      const key = String(cells[0] ?? "").trim();
      const value = cells.slice(1).join(" ").trim();
      if (!key || !value) return;
      if (/^(?:수업\s*코드|lesson(?:[_\s-]?id)?)$/iu.test(key)) {
        metadata.lesson_id = metadata.lesson_id || value;
      } else if (/^(?:차시|세션|session(?:[_\s-]?label)?)$/iu.test(key)) {
        metadata.session_label = metadata.session_label || value;
      } else if (/^(?:배포\s*코드|package(?:[_\s-]?id)?)$/iu.test(key)) {
        metadata.package_id = metadata.package_id || value;
      }
    });
  return metadata;
}

function buildStudentRosterMetadataReview(rosterText, lesson = null) {
  const metadata = parseStudentRosterMetadataText(rosterText);
  const expectedLessonId = String(lesson?.localPackageLessonId ?? lesson?.id ?? "").trim();
  const expectedSessionLabel = String(lesson?.localPackageSessionLabel ?? "").trim();
  const expectedPackageId = String(lesson?.localPackageId ?? "").trim();
  const lessonId = String(metadata.lesson_id ?? "").trim();
  const sessionLabel = String(metadata.session_label ?? "").trim();
  const packageId = String(metadata.package_id ?? "").trim();
  const lessonIdMatch = Boolean(!expectedLessonId || !lessonId || lessonId === expectedLessonId);
  const sessionMatch = Boolean(!expectedSessionLabel || !sessionLabel || sessionLabel === expectedSessionLabel);
  const packageIdMatch = Boolean(!expectedPackageId || !packageId || packageId === expectedPackageId);
  return {
    metadata,
    lesson_id_match: lessonIdMatch,
    session_match: sessionMatch,
    package_id_match: packageIdMatch,
    context_match: lessonIdMatch && sessionMatch && packageIdMatch,
  };
}

function findStudentResultReportStatus(text, key) {
  const target = String(key ?? "").trim();
  if (!target) return "";
  const line = String(text ?? "")
    .split(/\r?\n/u)
    .map((row) => row.trim())
    .find((row) => row.startsWith(`${target}\t`));
  return String(line?.split("\t")?.[1] ?? "").trim();
}

function buildStudentResultReportMetadataReview(text, lesson = null) {
  const source = String(text ?? "");
  const hasReportTable = source.includes("학생\t차시\t수업\t수업 코드\t배포 묶음\t배포 코드\t확인 상태\t상태 기록\t비고");
  const metadata = hasReportTable ? parseStudentRosterMetadataText(source) : { lesson_id: "", session_label: "", package_id: "" };
  const base = buildStudentRosterMetadataReview(
    [
      ...(metadata.lesson_id ? [`수업 코드\t${metadata.lesson_id}`] : []),
      ...(metadata.session_label ? [`차시\t${metadata.session_label}`] : []),
      ...(metadata.package_id ? [`배포 코드\t${metadata.package_id}`] : []),
    ].join("\n"),
    lesson,
  );
  if (!hasReportTable) return base;
  const lessonIdStatus = findStudentResultReportStatus(source, "명단 수업 코드 확인");
  const sessionStatus = findStudentResultReportStatus(source, "명단 차시 확인");
  const packageIdStatus = findStudentResultReportStatus(source, "명단 배포 코드 확인");
  const contextStatus = findStudentResultReportStatus(source, "명단 맥락 확인");
  const duplicateRosterStatus = findStudentResultReportStatus(source, "명단 중복 확인");
  const duplicateRosterNames = findStudentResultReportStatus(source, "명단 중복 이름")
    .split(/[|,]+/u)
    .map((name) => name.trim())
    .filter(Boolean);
  const hasSpecificNeed = lessonIdStatus === "필요" || sessionStatus === "필요" || packageIdStatus === "필요";
  const fallbackContextNeed = contextStatus === "필요" && !hasSpecificNeed;
  const lessonIdMatch = base.lesson_id_match && lessonIdStatus !== "필요";
  const sessionMatch = base.session_match && sessionStatus !== "필요";
  const packageIdMatch = base.package_id_match && packageIdStatus !== "필요" && !fallbackContextNeed;
  return {
    metadata: base.metadata,
    lesson_id_match: lessonIdMatch,
    session_match: sessionMatch,
    package_id_match: packageIdMatch,
    context_match: lessonIdMatch && sessionMatch && packageIdMatch,
    duplicate_roster_count: duplicateRosterStatus === "필요" ? Math.max(1, duplicateRosterNames.length) : 0,
    duplicate_roster_names: duplicateRosterNames,
  };
}

function mergeStudentRosterMetadataReviews(primary = null, secondary = null) {
  const primaryMetadata = toPlainObject(primary?.metadata, {});
  const secondaryMetadata = toPlainObject(secondary?.metadata, {});
  const metadata = {
    lesson_id: String(primaryMetadata.lesson_id || secondaryMetadata.lesson_id || "").trim(),
    session_label: String(primaryMetadata.session_label || secondaryMetadata.session_label || "").trim(),
    package_id: String(primaryMetadata.package_id || secondaryMetadata.package_id || "").trim(),
  };
  const lessonIdMatch = primary?.lesson_id_match !== false && secondary?.lesson_id_match !== false;
  const sessionMatch = primary?.session_match !== false && secondary?.session_match !== false;
  const packageIdMatch = primary?.package_id_match !== false && secondary?.package_id_match !== false;
  const primaryDuplicateNames = Array.isArray(primary?.duplicate_roster_names) ? primary.duplicate_roster_names : [];
  const secondaryDuplicateNames = Array.isArray(secondary?.duplicate_roster_names) ? secondary.duplicate_roster_names : [];
  const duplicateRosterNames = Array.from(new Set([...primaryDuplicateNames, ...secondaryDuplicateNames].map((name) => String(name ?? "").trim()).filter(Boolean)));
  const duplicateRosterCount = Math.max(
    Number(primary?.duplicate_roster_count) || 0,
    Number(secondary?.duplicate_roster_count) || 0,
    duplicateRosterNames.length,
  );
  return {
    metadata,
    lesson_id_match: lessonIdMatch,
    session_match: sessionMatch,
    package_id_match: packageIdMatch,
    context_match: lessonIdMatch && sessionMatch && packageIdMatch,
    duplicate_roster_count: duplicateRosterCount,
    duplicate_roster_names: duplicateRosterNames,
  };
}

function buildStudentRosterMetadataReviewNotes(review = null) {
  const notes = [];
  if (review?.lesson_id_match === false) notes.push("명단 수업 코드 확인");
  if (review?.session_match === false) notes.push("명단 차시 확인");
  if (review?.package_id_match === false) notes.push("명단 배포 코드 확인");
  return notes;
}

function buildStudentResultDuplicateNameKey(row = {}) {
  const student = normalizeStudentRosterName(row?.student_name);
  if (!student) return "";
  const lessonKey = [
    String(row?.package_id ?? "").trim().toLowerCase(),
    String(row?.session_label ?? "").trim().toLowerCase(),
    String(row?.lesson_id ?? row?.lesson_title ?? "").trim().toLowerCase(),
  ].filter(Boolean).join("|");
  return lessonKey ? `${student}|${lessonKey}` : student;
}

function buildStudentResultDuplicateStateHashKey(row = {}) {
  const hash = String(row?.state_hash ?? "").trim();
  if (!hash || hash === "-") return "";
  const lessonKey = [
    String(row?.package_id ?? "").trim().toLowerCase(),
    String(row?.session_label ?? "").trim().toLowerCase(),
    String(row?.lesson_id ?? row?.lesson_title ?? "").trim().toLowerCase(),
  ].filter(Boolean).join("|");
  return lessonKey ? `${hash}|${lessonKey}` : hash;
}

function parseStudentResultReturnBatchText(text, { lesson = null, rosterText = "" } = {}) {
  const blocks = splitStudentResultReturnBlocks(text);
  const rosterNames = parseStudentRosterText(rosterText);
  const rosterRecords = collectStudentRosterRecords(rosterText);
  const courseRosterRecords = rosterRecords.filter((record) => studentRosterRecordHasCourseContext(record));
  const useCourseRosterRecords = courseRosterRecords.length > 0;
  const rosterExpectationRows = useCourseRosterRecords
    ? courseRosterRecords
    : rosterNames.map((name) => ({
      name,
      session_label: "",
      lesson_title: "",
      lesson_id: "",
      package_title: "",
      package_id: "",
    }));
  const rosterExpectationByKey = new Map();
  rosterExpectationRows.forEach((record) => {
    const key = useCourseRosterRecords
      ? buildStudentRosterExpectationKeyFromRecord(record)
      : normalizeStudentRosterName(record.name);
    if (key && !rosterExpectationByKey.has(key)) rosterExpectationByKey.set(key, record);
  });
  const rosterExpectationKeys = new Set(rosterExpectationByKey.keys());
  const useRosterTargetCountLabel = useCourseRosterRecords
    && shouldUseStudentRosterTargetCountLabelFromRecords(rosterExpectationRows);
  const rosterAnalysis = analyzeStudentRosterText(rosterText);
  const currentRosterPresent = String(rosterText ?? "").trim().length > 0;
  const rosterMetadataReview = currentRosterPresent
    ? buildStudentRosterMetadataReview(rosterText, lesson)
    : mergeStudentRosterMetadataReviews(
      buildStudentRosterMetadataReview(rosterText, lesson),
      buildStudentResultReportMetadataReview(text, lesson),
    );
  const duplicateRosterNames = Array.from(new Set([
    ...(currentRosterPresent ? rosterAnalysis.duplicate_names : []),
    ...(Array.isArray(rosterMetadataReview.duplicate_roster_names) ? rosterMetadataReview.duplicate_roster_names : []),
  ].map((name) => String(name ?? "").trim()).filter(Boolean)));
  const duplicateRosterCount = Math.max(
    Number(rosterAnalysis.duplicate_count) || 0,
    Number(rosterMetadataReview.duplicate_roster_count) || 0,
    duplicateRosterNames.length,
  );
  const rows = blocks.map((block, index) => {
    const review = parseStudentResultReturnText(block, { lesson });
    if (currentRosterPresent) {
      review.report_status = "";
      review.report_notes = sanitizeStudentResultReportNotesForCurrentRoster(review.report_notes, {
        stateHash: review.state_hash,
      });
      review.accepted = isStudentResultReturnReviewCoreAccepted(review);
    }
    const studentName = String(review.student_name || "").trim() || `학생 ${index + 1}`;
    return {
      ...review,
      row_index: index + 1,
      student_name: studentName,
      raw_text: block,
    };
  });
  const nameCounts = new Map();
  const hashCounts = new Map();
  rows.forEach((row) => {
    const studentKey = buildStudentResultDuplicateNameKey(row);
    const hashKey = buildStudentResultDuplicateStateHashKey(row);
    if (studentKey) nameCounts.set(studentKey, Number(nameCounts.get(studentKey) ?? 0) + 1);
    if (hashKey) hashCounts.set(hashKey, Number(hashCounts.get(hashKey) ?? 0) + 1);
  });
  rows.forEach((row) => {
    const duplicateNameKey = buildStudentResultDuplicateNameKey(row);
    const rosterResultKey = useCourseRosterRecords
      ? buildStudentRosterExpectationKeyFromResultRow(row)
      : normalizeStudentRosterName(row.student_name);
    const hashKey = buildStudentResultDuplicateStateHashKey(row);
    row.duplicate_student_name = Boolean(duplicateNameKey && Number(nameCounts.get(duplicateNameKey) ?? 0) > 1);
    row.duplicate_state_hash = Boolean(hashKey && Number(hashCounts.get(hashKey) ?? 0) > 1);
    row.roster_expected = rosterExpectationKeys.size > 0;
    row.roster_match = Boolean(!rosterExpectationKeys.size || rosterExpectationKeys.has(rosterResultKey));
    row.roster_extra = Boolean(rosterExpectationKeys.size && rosterResultKey && !rosterExpectationKeys.has(rosterResultKey));
    row.roster_lesson_id_match = rosterMetadataReview.lesson_id_match;
    row.roster_session_match = rosterMetadataReview.session_match;
    row.roster_package_id_match = rosterMetadataReview.package_id_match;
    row.roster_missing = Boolean(
      row.roster_expected
        && row.roster_match
        && !String(row.state_hash ?? "").trim()
        && String(row.report_notes ?? "")
          .split("|")
          .map((note) => note.trim())
          .includes("미제출"),
    );
    row.accepted = Boolean(row.accepted && !row.duplicate_student_name && !row.duplicate_state_hash && !row.roster_extra && !row.roster_missing && rosterMetadataReview.context_match);
  });
  if (rosterExpectationKeys.size > 0) {
    const submittedKeys = new Set(rows.map((row) => (
      useCourseRosterRecords
        ? buildStudentRosterExpectationKeyFromResultRow(row)
        : normalizeStudentRosterName(row.student_name)
    )).filter(Boolean));
    rosterExpectationByKey.forEach((record, key) => {
      if (!key || submittedKeys.has(key)) return;
      const studentName = String(record?.name ?? "").trim();
      rows.push({
        schema: "seamgrim.student_result_return_review.v1",
        accepted: false,
        student_name: studentName,
        session_label: String(record?.session_label || lesson?.localPackageSessionLabel || "").trim(),
        lesson_title: String(record?.lesson_title || lesson?.title || lesson?.id || "").trim(),
        lesson_id: String(record?.lesson_id || lesson?.localPackageLessonId || lesson?.id || "").trim(),
        package_title: String(record?.package_title || lesson?.localPackageTitle || "").trim(),
        package_id: String(record?.package_id || lesson?.localPackageId || "").trim(),
        state_hash: "",
        session_match: true,
        lesson_match: true,
        lesson_id_match: true,
        package_match: true,
        package_id_match: true,
        account_required: false,
        cloud_sync: false,
        permission_system: false,
        row_index: rows.length + 1,
        raw_text: "",
        duplicate_student_name: false,
        student_name_missing: false,
        duplicate_state_hash: false,
        roster_expected: true,
        roster_match: true,
        roster_extra: false,
        roster_lesson_id_match: rosterMetadataReview.lesson_id_match,
        roster_session_match: rosterMetadataReview.session_match,
        roster_package_id_match: rosterMetadataReview.package_id_match,
        roster_missing: true,
      });
      submittedKeys.add(key);
    });
    rows.forEach((row, index) => {
      row.row_index = index + 1;
    });
  }
  const acceptedCount = rows.filter((row) => row.accepted === true).length;
  const rejectedCount = rows.length - acceptedCount;
  const missingCount = rows.filter((row) => row.roster_missing === true).length;
  const extraCount = rows.filter((row) => row.roster_extra === true).length;
  const batchAccepted = rows.length > 0 && rejectedCount === 0 && duplicateRosterCount <= 0;
  const nextActionText = buildStudentResultNextActionText({
    row_count: rows.length,
    missing_count: missingCount,
    duplicate_roster_count: duplicateRosterCount,
    duplicate_roster_names: duplicateRosterNames,
    rows,
  });
  const reportText = formatStudentResultReturnBatchReportText(rows, {
    roster_metadata: rosterMetadataReview.metadata,
    roster_lesson_id_match: rosterMetadataReview.lesson_id_match,
    roster_session_match: rosterMetadataReview.session_match,
    roster_package_id_match: rosterMetadataReview.package_id_match,
    roster_context_match: rosterMetadataReview.context_match,
    roster_entry_count: rosterAnalysis.entry_count,
    duplicate_roster_count: duplicateRosterCount,
    duplicate_roster_names: duplicateRosterNames,
    next_action: nextActionText,
  });
  return {
    schema: "seamgrim.student_result_return_batch_review.v1",
    accepted: batchAccepted,
    row_count: rows.length,
    accepted_count: acceptedCount,
    rejected_count: rejectedCount,
    roster_count: rosterExpectationByKey.size,
    roster_count_kind: useRosterTargetCountLabel ? "lesson_targets" : "students",
    missing_count: missingCount,
    extra_count: extraCount,
    roster_entry_count: rosterAnalysis.entry_count,
    duplicate_roster_count: duplicateRosterCount,
    duplicate_roster_names: duplicateRosterNames,
    roster_metadata: rosterMetadataReview.metadata,
    roster_lesson_id_match: rosterMetadataReview.lesson_id_match,
    roster_session_match: rosterMetadataReview.session_match,
    roster_package_id_match: rosterMetadataReview.package_id_match,
    roster_context_match: rosterMetadataReview.context_match,
    account_required: false,
    cloud_sync: false,
    permission_system: false,
    rows,
    report_text: reportText,
    next_action: nextActionText,
  };
}

function buildStudentResultReviewNotes(row = {}) {
  const notes = [];
  if (row.roster_missing === true) notes.push("미제출");
  if (row.roster_extra === true) notes.push("명단에 없음");
  if (row.roster_lesson_id_match === false) notes.push("명단 수업 코드 확인");
  if (row.roster_session_match === false) notes.push("명단 차시 확인");
  if (row.roster_package_id_match === false) notes.push("명단 배포 코드 확인");
  if (row.student_name_missing === true) notes.push("학생 이름 확인");
  if (row.session_match === false) notes.push("차시 확인");
  if (row.lesson_match === false) notes.push("수업 이름 확인");
  if (row.lesson_id_match === false) notes.push("수업 코드 확인");
  if (row.package_match === false) notes.push("배포 묶음 확인");
  if (row.package_id_match === false) notes.push("배포 코드 확인");
  if (!row.state_hash || row.state_hash === "-") {
    notes.push(row.roster_missing === true ? "상태 기록 대기" : "상태 기록 없음");
  }
  if (row.duplicate_student_name === true) notes.push("학생 이름 중복");
  if (row.duplicate_state_hash === true) notes.push("상태 기록 중복");
  String(row.report_notes ?? "")
    .split("|")
    .map((note) => note.trim())
    .filter((note) => note && note !== "-")
    .forEach((note) => {
      if (!notes.includes(note)) notes.push(note);
    });
  return notes;
}

function sanitizeStudentResultReportNotesForCurrentRoster(notes = "", { stateHash = "" } = {}) {
  const staleRosterNotes = new Set([
    "명단 수업 코드 확인",
    "명단 차시 확인",
    "명단 배포 코드 확인",
    "명단에 없음",
  ]);
  if (String(stateHash ?? "").trim()) {
    staleRosterNotes.add("미제출");
    staleRosterNotes.add("상태 기록 대기");
    staleRosterNotes.add("상태 기록 없음");
  }
  return String(notes ?? "")
    .split("|")
    .map((note) => note.trim())
    .filter((note) => note && !staleRosterNotes.has(note))
    .join("|");
}

function collectStudentResultReportDistinctValues(rows = [], field = "") {
  const seen = new Set();
  const values = [];
  (Array.isArray(rows) ? rows : []).forEach((row) => {
    const value = String(row?.[field] ?? "").trim();
    if (!value || seen.has(value)) return;
    seen.add(value);
    values.push(value);
  });
  return values;
}

function buildStudentResultReportScope(rows = [], { lesson = null } = {}) {
  const safeRows = Array.isArray(rows) ? rows : [];
  const lessonIds = collectStudentResultReportDistinctValues(safeRows, "lesson_id");
  const packageIds = collectStudentResultReportDistinctValues(safeRows, "package_id");
  const sessionLabels = collectStudentResultReportDistinctValues(safeRows, "session_label");
  const firstRow = safeRows.find((row) => row && typeof row === "object") || {};
  const multiLesson = lessonIds.length > 1;
  return {
    multi_lesson: multiLesson,
    lesson_ids: lessonIds,
    package_ids: packageIds,
    session_labels: sessionLabels,
    lesson_id: multiLesson
      ? ""
      : String(lesson?.localPackageLessonId ?? lesson?.id ?? lessonIds[0] ?? firstRow.lesson_id ?? "").trim(),
    package_id: String(lesson?.localPackageId ?? packageIds[0] ?? firstRow.package_id ?? "").trim(),
    session_label: String(lesson?.localPackageSessionLabel ?? sessionLabels[0] ?? firstRow.session_label ?? "").trim(),
  };
}

function buildStudentReminderScope(rows = [], { lesson = null, fallbackLessonId = "lesson" } = {}) {
  const safeRows = Array.isArray(rows) ? rows : [];
  const lessonIds = collectStudentResultReportDistinctValues(safeRows, "lesson_id");
  const packageIds = collectStudentResultReportDistinctValues(safeRows, "package_id");
  const sessionLabels = collectStudentResultReportDistinctValues(safeRows, "session_label");
  const firstRow = safeRows.find((row) => row && typeof row === "object") || {};
  const multiLesson = lessonIds.length > 1;
  return {
    multi_lesson: multiLesson,
    lesson_ids: lessonIds,
    package_ids: packageIds,
    session_labels: sessionLabels,
    lesson_id: multiLesson
      ? ""
      : String(firstRow.lesson_id ?? lessonIds[0] ?? lesson?.localPackageLessonId ?? lesson?.id ?? fallbackLessonId ?? "lesson").trim(),
    package_id: String(firstRow.package_id ?? packageIds[0] ?? lesson?.localPackageId ?? "").trim(),
    session_label: String(firstRow.session_label ?? sessionLabels[0] ?? lesson?.localPackageSessionLabel ?? "").trim(),
  };
}

function buildStudentReminderDownloadFileName(scope = {}, suffix = "reminder", fallbackBase = "course") {
  const sessionLabel = String(scope?.session_label ?? "").trim();
  const packageId = String(scope?.package_id ?? "").trim();
  const lessonId = String(scope?.lesson_id ?? "").trim();
  const sessionSuffix = sessionLabel ? `_${sessionLabel}` : "";
  const packageSuffix = packageId ? `_${packageId}` : "";
  const fileStem = scope?.multi_lesson === true
    ? `${packageId || fallbackBase || "course"}${sessionSuffix}_${suffix}`
    : `${lessonId || fallbackBase || "lesson"}${sessionSuffix}${packageSuffix}_${suffix}`;
  return buildSafeDownloadName(`${fileStem}.txt`, `seamgrim-${suffix}`, "txt");
}

function formatStudentResultReturnBatchReportText(rows = [], batchMeta = {}) {
  const metadata = toPlainObject(batchMeta?.roster_metadata, {});
  const safeRows = Array.isArray(rows) ? rows : [];
  const firstRow = safeRows.find((row) => row && typeof row === "object") || {};
  const lessonIds = collectStudentResultReportDistinctValues(safeRows, "lesson_id");
  const lessonTitles = collectStudentResultReportDistinctValues(safeRows, "lesson_title");
  const sessionLabels = collectStudentResultReportDistinctValues(safeRows, "session_label");
  const packageIds = collectStudentResultReportDistinctValues(safeRows, "package_id");
  const lessonId = lessonIds.length <= 1
    ? String(lessonIds[0] ?? metadata.lesson_id ?? firstRow.lesson_id ?? "").trim()
    : "";
  const sessionLabel = sessionLabels.length <= 1
    ? String(sessionLabels[0] ?? metadata.session_label ?? firstRow.session_label ?? "").trim()
    : "";
  const packageId = packageIds.length <= 1
    ? String(packageIds[0] ?? metadata.package_id ?? firstRow.package_id ?? "").trim()
    : "";
  const nextActionText = String(batchMeta?.next_action ?? "").trim();
  const nextActionValue = nextActionText.replace(/^다음 행동:\s*/u, "").trim();
  const lines = [
    "# 셈그림 학생 결과표",
    ...(lessonId ? [`수업 코드\t${lessonId}`] : []),
    ...(lessonIds.length > 1 ? [`수업 범위\t여러 수업 ${lessonIds.length}개`, `수업 코드 목록\t${lessonIds.join("|")}`] : []),
    ...(lessonTitles.length > 1 ? [`수업 제목 목록\t${lessonTitles.join("|")}`] : []),
    ...(sessionLabel ? [`차시\t${sessionLabel}`] : []),
    ...(sessionLabels.length > 1 ? [`차시 목록\t${sessionLabels.join("|")}`] : []),
    ...(packageId ? [`배포 코드\t${packageId}`] : []),
    ...(packageIds.length > 1 ? [`배포 코드 목록\t${packageIds.join("|")}`] : []),
    `명단 수업 코드 확인\t${batchMeta?.roster_lesson_id_match === false ? "필요" : "통과"}`,
    `명단 차시 확인\t${batchMeta?.roster_session_match === false ? "필요" : "통과"}`,
    `명단 배포 코드 확인\t${batchMeta?.roster_package_id_match === false ? "필요" : "통과"}`,
    `명단 맥락 확인\t${batchMeta?.roster_context_match === false ? "필요" : "통과"}`,
    `명단 중복 확인\t${Math.max(0, Number(batchMeta?.duplicate_roster_count) || 0) > 0 ? "필요" : "통과"}`,
    ...(Array.isArray(batchMeta?.duplicate_roster_names) && batchMeta.duplicate_roster_names.length
      ? [`명단 중복 이름\t${batchMeta.duplicate_roster_names.join("|")}`]
      : []),
    ...(nextActionValue ? [`다음 행동\t${nextActionValue}`] : []),
    "",
    "학생\t차시\t수업\t수업 코드\t배포 묶음\t배포 코드\t확인 상태\t상태 기록\t비고",
  ];
  (Array.isArray(rows) ? rows : []).forEach((row) => {
    const notes = buildStudentResultReviewNotes(row);
    lines.push([
      String(row.student_name ?? ""),
      String(row.session_label ?? ""),
      String(row.lesson_title ?? ""),
      String(row.lesson_id ?? ""),
      String(row.package_title ?? ""),
      String(row.package_id ?? ""),
      row.accepted === true ? "확인됨" : "확인 필요",
      String(row.state_hash ?? ""),
      notes.join("|"),
    ].join("\t"));
  });
  return lines.join("\n");
}

function formatDuplicateRosterNamesText(batch = null, { maxNames = 3 } = {}) {
  const names = Array.isArray(batch?.duplicate_roster_names)
    ? batch.duplicate_roster_names.map((name) => String(name ?? "").trim()).filter(Boolean)
    : [];
  if (!names.length) return "";
  const limit = Math.max(1, Number(maxNames) || 3);
  const visible = names.slice(0, limit);
  const remaining = names.length - visible.length;
  return remaining > 0 ? `${visible.join(", ")} 외 ${remaining}명` : visible.join(", ");
}

function buildStudentResultNextActionText(batch = null) {
  const rowCount = Math.max(0, Number(batch?.row_count) || 0);
  if (rowCount <= 0) return "다음 행동: 학생 결과를 붙여넣거나 결과 파일을 여세요.";
  const missingCount = Math.max(0, Number(batch?.missing_count) || 0);
  const duplicateRosterCount = Math.max(0, Number(batch?.duplicate_roster_count) || 0);
  const duplicateRosterNamesText = formatDuplicateRosterNamesText(batch);
  const duplicateRosterTargetText = duplicateRosterNamesText ? ` (${duplicateRosterNamesText})` : "";
  const reviewCount = getStudentResultReviewReminderRows(batch).length;
  if (duplicateRosterCount > 0 && (missingCount > 0 || reviewCount > 0)) {
    return `다음 행동: 명단 중복${duplicateRosterTargetText}을 먼저 정리한 뒤 미제출/확인 필요 안내를 보내세요.`;
  }
  if (duplicateRosterCount > 0) {
    return `다음 행동: 명단 중복${duplicateRosterTargetText}을 정리한 뒤 결과표를 다시 확인하세요.`;
  }
  if (missingCount > 0 && reviewCount > 0) {
    return "다음 행동: 미제출 안내와 확인 필요 안내를 각각 복사하세요.";
  }
  if (missingCount > 0) {
    return "다음 행동: 미제출 안내 복사 후 제출을 다시 요청하세요.";
  }
  if (reviewCount > 0) {
    return "다음 행동: 확인 필요 안내 복사 후 결과를 다시 받으세요.";
  }
  return "다음 행동: 결과표 저장 후 입력 비우기로 다음 반을 준비하세요.";
}

function getStudentResultMissingReminderTargets(batch = null) {
  const rows = Array.isArray(batch?.rows) ? batch.rows : [];
  return rows
    .filter((row) => row?.roster_missing === true)
    .map((row) => ({
      student_name: String(row?.student_name ?? "").trim(),
      lesson_title: String(row?.lesson_title ?? "").trim(),
      lesson_id: String(row?.lesson_id ?? "").trim(),
      package_title: String(row?.package_title ?? "").trim(),
      package_id: String(row?.package_id ?? "").trim(),
      session_label: String(row?.session_label ?? "").trim(),
    }))
    .filter((target) => target.student_name);
}

function formatStudentResultMissingTargetLine(target = {}) {
  const studentName = String(target?.student_name ?? "").trim();
  const lessonTitle = String(target?.lesson_title ?? "").trim();
  const lessonId = String(target?.lesson_id ?? "").trim();
  const sessionLabel = String(target?.session_label ?? "").trim();
  const details = [
    lessonTitle || lessonId,
    lessonTitle && lessonId ? lessonId : "",
    sessionLabel,
  ].filter(Boolean);
  return details.length ? `- ${studentName}: ${details.join(" · ")}` : `- ${studentName}`;
}

function buildStudentResultMissingReminderText(batch = null, { lesson = null } = {}) {
  const rows = Array.isArray(batch?.rows) ? batch.rows : [];
  const missingTargets = getStudentResultMissingReminderTargets(batch);
  const missingNames = Array.from(new Set(missingTargets.map((target) => target.student_name).filter(Boolean)));
  if (!missingNames.length) return "";
  const lessonTitle = String(lesson?.title ?? rows.find((row) => row?.lesson_title)?.lesson_title ?? "받은 수업").trim() || "받은 수업";
  const lessonId = String(lesson?.localPackageLessonId ?? lesson?.id ?? rows.find((row) => row?.lesson_id)?.lesson_id ?? "").trim();
  const packageTitle = String(lesson?.localPackageTitle ?? rows.find((row) => row?.package_title)?.package_title ?? "").trim();
  const packageId = String(lesson?.localPackageId ?? rows.find((row) => row?.package_id)?.package_id ?? "").trim();
  const sessionLabel = String(lesson?.localPackageSessionLabel ?? rows.find((row) => row?.session_label)?.session_label ?? "").trim();
  const targetLines = missingTargets.map((target) => formatStudentResultMissingTargetLine(target));
  return [
    "미제출 안내",
    ...(sessionLabel ? [`차시: ${sessionLabel}`] : []),
    `수업: ${lessonTitle}`,
    ...(lessonId ? [`수업 코드: ${lessonId}`] : []),
    ...(packageTitle ? [`배포 묶음: ${packageTitle}`] : []),
    ...(packageId ? [`배포 코드: ${packageId}`] : []),
    `대상: ${missingNames.join(", ")}`,
    ...(targetLines.length > 1 || targetLines.some((line) => !line.endsWith(` ${missingNames[0]}`))
      ? ["대상 수업:", ...targetLines]
      : []),
    "배포 열기 -> 받은 수업 실행 -> 결과 확인 -> 결과 복사 순서로 제출해 주세요.",
  ].join("\n");
}

function getStudentResultReviewReminderRows(batch = null) {
  const rows = Array.isArray(batch?.rows) ? batch.rows : [];
  return rows.filter((row) => row?.accepted !== true && row?.roster_missing !== true);
}

function getStudentResultReviewReminderTargets(batch = null) {
  return getStudentResultReviewReminderRows(batch)
    .map((row) => {
      const studentName = String(row?.student_name ?? "").trim() || `학생 ${Number(row?.row_index) || ""}`.trim();
      const notes = buildStudentResultReviewNotes(row)
        .filter((note) => note !== "미제출")
        .join(", ") || "확인 필요";
      return {
        student_name: studentName,
        lesson_title: String(row?.lesson_title ?? "").trim(),
        lesson_id: String(row?.lesson_id ?? "").trim(),
        package_title: String(row?.package_title ?? "").trim(),
        package_id: String(row?.package_id ?? "").trim(),
        session_label: String(row?.session_label ?? "").trim(),
        notes,
      };
    })
    .filter((target) => target.student_name);
}

function formatStudentResultReviewTargetLine(target = {}) {
  const studentName = String(target?.student_name ?? "").trim();
  const lessonTitle = String(target?.lesson_title ?? "").trim();
  const lessonId = String(target?.lesson_id ?? "").trim();
  const sessionLabel = String(target?.session_label ?? "").trim();
  const notes = String(target?.notes ?? "확인 필요").trim() || "확인 필요";
  const details = [
    lessonTitle || lessonId,
    lessonTitle && lessonId ? lessonId : "",
    sessionLabel,
  ].filter(Boolean);
  return details.length ? `- ${studentName}: ${details.join(" · ")} — ${notes}` : `- ${studentName}: ${notes}`;
}

function buildStudentResultReviewReminderText(batch = null, { lesson = null } = {}) {
  const rows = getStudentResultReviewReminderRows(batch);
  if (!rows.length) return "";
  const allRows = Array.isArray(batch?.rows) ? batch.rows : rows;
  const lessonTitle = String(lesson?.title ?? allRows.find((row) => row?.lesson_title)?.lesson_title ?? "받은 수업").trim() || "받은 수업";
  const lessonId = String(lesson?.localPackageLessonId ?? lesson?.id ?? allRows.find((row) => row?.lesson_id)?.lesson_id ?? "").trim();
  const packageTitle = String(lesson?.localPackageTitle ?? allRows.find((row) => row?.package_title)?.package_title ?? "").trim();
  const packageId = String(lesson?.localPackageId ?? allRows.find((row) => row?.package_id)?.package_id ?? "").trim();
  const sessionLabel = String(lesson?.localPackageSessionLabel ?? allRows.find((row) => row?.session_label)?.session_label ?? "").trim();
  const targets = getStudentResultReviewReminderTargets(batch).map((target) => formatStudentResultReviewTargetLine(target));
  return [
    "확인 필요 안내",
    ...(sessionLabel ? [`차시: ${sessionLabel}`] : []),
    `수업: ${lessonTitle}`,
    ...(lessonId ? [`수업 코드: ${lessonId}`] : []),
    ...(packageTitle ? [`배포 묶음: ${packageTitle}`] : []),
    ...(packageId ? [`배포 코드: ${packageId}`] : []),
    "대상:",
    ...targets,
    "배포 열기 -> 받은 수업 실행 -> 결과 확인 -> 결과 복사 순서로 다시 제출해 주세요.",
  ].join("\n");
}

function buildStudentRosterTemplateText({ lessonId = "", sessionLabel = "", packageId = "" } = {}) {
  const lesson = String(lessonId ?? "").trim();
  const session = String(sessionLabel ?? "").trim();
  const packageText = String(packageId ?? "").trim();
  return [
    "# 셈그림 학생 명단 양식",
    ...(lesson ? [`수업 코드\t${lesson}`] : []),
    ...(session ? [`차시\t${session}`] : []),
    ...(packageText ? [`배포 코드\t${packageText}`] : []),
    "번호\t이름",
    "1\t",
    "2\t",
    "3\t",
  ].join("\n");
}

function buildStudentRosterTemplateFileName({ lessonId = "lesson", sessionLabel = "", packageId = "" } = {}) {
  const lesson = String(lessonId ?? "").trim() || "lesson";
  const session = String(sessionLabel ?? "").trim();
  const packageText = String(packageId ?? "").trim();
  const sessionSuffix = session ? `_${session}` : "";
  const packageSuffix = packageText ? `_${packageText}` : "";
  return buildSafeDownloadName(`${lesson}${sessionSuffix}${packageSuffix}_student_roster_template.tsv`, "seamgrim-student-roster-template", "tsv");
}

function buildLocalPackageLessonScope(payload = null, { fallbackLessonId = "" } = {}) {
  const lessons = Array.isArray(payload?.lessons) ? payload.lessons : [];
  const lessonIds = collectStudentResultReportDistinctValues(lessons, "lesson_id");
  const fallback = String(fallbackLessonId ?? "").trim();
  if (!lessonIds.length && fallback) lessonIds.push(fallback);
  return {
    multi_lesson: lessonIds.length > 1,
    lesson_ids: lessonIds,
  };
}

function buildTeacherPreparationChecklistText({
  lessonTitle = "",
  lessonId = "",
  sessionLabel = "",
  packageId = "",
  requiredViews = [],
  goals = [],
  missions = [],
} = {}) {
  const title = String(lessonTitle ?? "").trim() || "셈그림 수업";
  const lesson = String(lessonId ?? "").trim();
  const session = String(sessionLabel ?? "").trim();
  const packageText = String(packageId ?? "").trim();
  const viewText = Array.isArray(requiredViews) && requiredViews.length
    ? requiredViews.map((item) => String(item ?? "").trim()).filter(Boolean).join(", ")
    : "결과 화면";
  const firstGoal = Array.isArray(goals) ? String(goals[0] ?? "").trim() : "";
  const firstMission = Array.isArray(missions) ? String(missions[0] ?? "").trim() : "";
  return [
    "셈그림 교사용 배포 준비 체크리스트",
    ...(session ? [`차시: ${session}`] : []),
    `수업: ${title}`,
    ...(lesson ? [`수업 코드: ${lesson}`] : []),
    ...(packageText ? [`배포 코드: ${packageText}`] : []),
    ...(firstGoal ? [`목표: ${firstGoal}`] : []),
    ...(firstMission ? [`오늘 활동: ${firstMission}`] : []),
    "",
    "배포 전",
    "- 배포 JSON 파일을 학생에게 보낼 위치에 준비합니다.",
    "- 학생 안내문을 함께 보냅니다.",
    "- 학생 명단 양식에 번호와 이름을 채워 둡니다.",
    "",
    "수업 중",
    "- 학생은 배포 열기 또는 붙여넣기 열기로 받은 수업을 엽니다.",
    "- 받은 수업 실행 뒤 결과 확인 화면을 봅니다.",
    `- 확인할 결과: ${viewText}`,
    "",
    "수업 후",
    "- 학생은 결과 복사 또는 결과 저장으로 제출합니다.",
    "- 교사는 학생 결과와 명단을 붙여넣고 결과 확인을 누릅니다.",
    "- 확인 필요 학생은 안내문 복사로 다시 제출 요청을 보냅니다.",
  ].join("\n");
}

function buildTeacherPreparationChecklistFileName({ lessonId = "lesson", sessionLabel = "", packageId = "" } = {}) {
  const lesson = String(lessonId ?? "").trim() || "lesson";
  const session = String(sessionLabel ?? "").trim();
  const packageText = String(packageId ?? "").trim();
  const sessionSuffix = session ? `_${session}` : "";
  const packageSuffix = packageText ? `_${packageText}` : "";
  return buildSafeDownloadName(`${lesson}${sessionSuffix}${packageSuffix}_teacher_preparation_checklist.txt`, "seamgrim-teacher-preparation-checklist", "txt");
}

function hashRunSourceText(text) {
  let hash = 2166136261;
  const source = String(text ?? "");
  for (let i = 0; i < source.length; i += 1) {
    hash ^= source.charCodeAt(i);
    hash = Math.imul(hash, 16777619) >>> 0;
  }
  return `fnv1a:${hash.toString(16).padStart(8, "0")}`;
}

export function hasLegacyAutofixCandidate(text) {
  const source = String(text ?? "");
  if (!source.trim()) return false;
  return (
    LEGACY_RANGE_COMMENT_RE.test(source) ||
    LEGACY_RANGE_HASH_RE.test(source) ||
    LEGACY_SETUP_COLON_RE.test(source) ||
    LEGACY_HOOK_COLON_RE.test(source) ||
    LEGACY_HOOK_ALIAS_RE.test(source) ||
    LEGACY_SETTING_ALIAS_RE.test(source) ||
    LEGACY_MAX_MADI_RE.test(source)
  );
}

function ensureSettingsMadiFromResource(sourceText = "") {
  const source = String(sourceText ?? "");
  const resourceMatch = source.match(
    /^\s*마디수\s*(?::\s*[^<=>\r\n]+?)?\s*(?:<-|=)\s*\(?\s*([1-9][0-9]*)(?:\.0+)?\s*\)?(?:\s+매김(?=\s|\{|\.|$)|\s*\.|\s*$)/mu,
  );
  if (!resourceMatch) return { text: source, inserted: false };
  const value = Number(resourceMatch[1]);
  if (!Number.isFinite(value) || value <= 0) return { text: source, inserted: false };
  const settingMatch = source.match(/설정\s*\{([\s\S]*?)\}\s*\.?/u);
  if (!settingMatch) return { text: source, inserted: false };
  const settingBody = String(settingMatch[1] ?? "");
  if (/^\s*마디수\s*:/mu.test(settingBody)) return { text: source, inserted: false };
  const lineInserted = source.replace(
    /^(\s*설명\s*:\s*.*\.\s*)$/mu,
    `$1\n  마디수: ${Math.floor(value)}.`,
  );
  if (lineInserted !== source) return { text: lineInserted, inserted: true };
  return {
    text: source.replace(/(설정\s*\{)/u, `$1\n  마디수: ${Math.floor(value)}.`),
    inserted: true,
  };
}

export function applyLegacyAutofixToDdn(text) {
  const originalText = String(text ?? "");
  let sourceText = originalText.replace(/^(\s*)title\s*:/gimu, "$1제목:");
  sourceText = sourceText.replace(/^(\s*)desc\s*:/gimu, "$1설명:");
  const maxMadiRewrites = (sourceText.match(/최대마디/gu) ?? []).length;
  sourceText = sourceText.replace(/최대마디/gu, "마디수");
  const settingsMadi = ensureSettingsMadiFromResource(sourceText);
  sourceText = settingsMadi.text;
  const lines = sourceText.split(/\r?\n/);
  const out = [];
  const stats = {
    range_rewrites: 0,
    range_hash_rewrites: 0,
    range_skipped: 0,
    range_hash_skipped: 0,
    setup_colon_rewrites: 0,
    hook_colon_rewrites: 0,
    hook_alias_rewrites: 0,
    setting_alias_rewrites: LEGACY_SETTING_ALIAS_RE.test(originalText) ? 1 : 0,
    max_madi_rewrites: maxMadiRewrites,
    settings_madi_insertions: settingsMadi.inserted ? 1 : 0,
  };
  const rangeDeclRe = /^(\s*[A-Za-z_가-힣][A-Za-z0-9_가-힣.]*\s*(?::\s*[A-Za-z_가-힣][A-Za-z0-9_가-힣.]*)?\s*<-\s*)(.+?)\.\s*\/\/\s*범위\s*\(\s*([^,\n)]+)\s*,\s*([^,\n)]+)\s*,\s*([^\)\n]+)\)\s*(\/\/.*)?$/u;
  const rangeHashDeclRe = /^(\s*[A-Za-z_가-힣][A-Za-z0-9_가-힣.]*\s*(?::\s*[A-Za-z_가-힣][A-Za-z0-9_가-힣.]*)?\s*<-\s*)(.+?)\.\s*#\s*범위\s*\(\s*([^,\n)]+)\s*,\s*([^,\n)]+)\s*,\s*([^\)\n]+)\)\s*(\/\/.*)?$/u;
  const setupColonRe = /^(\s*)채비\s*:\s*\{\s*(\/\/.*)?$/u;
  const hookStartColonRe = /^(\s*)\(\s*(시작|처음)\s*\)\s*할때\s*:\s*\{\s*(\/\/.*)?$/u;
  const hookTickColonRe = /^(\s*)\(\s*((?:매마디|매틱)|(?:[1-9][0-9]*\s*마디))\s*\)\s*마다\s*:\s*\{\s*(\/\/.*)?$/u;

  lines.forEach((line) => {
    const rangeMatch = line.match(rangeDeclRe);
    if (rangeMatch) {
      const prefix = String(rangeMatch[1] ?? "");
      const initExpr = normalizeLegacyInitExpr(rangeMatch[2]);
      const minExpr = String(rangeMatch[3] ?? "").trim();
      const maxExpr = String(rangeMatch[4] ?? "").trim();
      const stepExpr = String(rangeMatch[5] ?? "").trim();
      const tail = String(rangeMatch[6] ?? "").trim();
      let rewritten = `${prefix}${initExpr} 매김 { 범위: ${minExpr}..${maxExpr}. 간격: ${stepExpr}. }.`;
      if (tail) rewritten = `${rewritten} ${tail}`;
      out.push(rewritten);
      stats.range_rewrites += 1;
      return;
    }

    const rangeHashMatch = line.match(rangeHashDeclRe);
    if (rangeHashMatch) {
      const prefix = String(rangeHashMatch[1] ?? "");
      const initExpr = normalizeLegacyInitExpr(rangeHashMatch[2]);
      const minExpr = String(rangeHashMatch[3] ?? "").trim();
      const maxExpr = String(rangeHashMatch[4] ?? "").trim();
      const stepExpr = String(rangeHashMatch[5] ?? "").trim();
      const tail = String(rangeHashMatch[6] ?? "").trim();
      let rewritten = `${prefix}${initExpr} 매김 { 범위: ${minExpr}..${maxExpr}. 간격: ${stepExpr}. }.`;
      if (tail) rewritten = `${rewritten} ${tail}`;
      out.push(rewritten);
      stats.range_hash_rewrites += 1;
      return;
    }

    if (LEGACY_RANGE_COMMENT_RE.test(line)) {
      stats.range_skipped += 1;
    }
    if (LEGACY_RANGE_HASH_RE.test(line)) {
      stats.range_hash_skipped += 1;
    }

    const setupMatch = line.match(setupColonRe);
    if (setupMatch) {
      const indent = String(setupMatch[1] ?? "");
      const tail = String(setupMatch[2] ?? "").trim();
      let rewritten = `${indent}채비 {`;
      if (tail) rewritten = `${rewritten} ${tail}`;
      out.push(rewritten);
      stats.setup_colon_rewrites += 1;
      return;
    }

    const hookStartMatch = line.match(hookStartColonRe);
    if (hookStartMatch) {
      const indent = String(hookStartMatch[1] ?? "");
      const tail = String(hookStartMatch[3] ?? "").trim();
      let rewritten = `${indent}(시작)할때 {`;
      if (tail) rewritten = `${rewritten} ${tail}`;
      out.push(rewritten);
      stats.hook_colon_rewrites += 1;
      if (String(hookStartMatch[2] ?? "").trim() !== "시작") {
        stats.hook_alias_rewrites += 1;
      }
      return;
    }

    const hookTickMatch = line.match(hookTickColonRe);
    if (hookTickMatch) {
      const indent = String(hookTickMatch[1] ?? "");
      const name = String(hookTickMatch[2] ?? "").trim();
      const canonical = name === "매마디" || name === "매틱" ? "매마디" : name.replace(/\s+/g, "");
      const tail = String(hookTickMatch[3] ?? "").trim();
      let rewritten = `${indent}(${canonical})마다 {`;
      if (tail) rewritten = `${rewritten} ${tail}`;
      out.push(rewritten);
      stats.hook_colon_rewrites += 1;
      if (name !== canonical) {
        stats.hook_alias_rewrites += 1;
      }
      return;
    }

    let rewritten = line;
    const startAlias = /\(\s*처음\s*\)\s*할때/gu;
    const tickAlias = /\(\s*매틱\s*\)\s*마다/gu;
    const startCount = (rewritten.match(startAlias) || []).length;
    rewritten = rewritten.replace(startAlias, "(시작)할때");
    const tickCount = (rewritten.match(tickAlias) || []).length;
    rewritten = rewritten.replace(tickAlias, "(매마디)마다");
    stats.hook_alias_rewrites += startCount + tickCount;
    out.push(rewritten);
  });

  const nextText = out.join("\n");
  const totalChanged = Object.values(stats).reduce((acc, val) => acc + Number(val || 0), 0);
  return {
    text: nextText,
    changed: nextText !== String(text ?? ""),
    stats,
    total_changes: totalChanged,
  };
}

function normalizeRuntimeInputToken(raw = "") {
  const text = String(raw ?? "").trim();
  if (!text) return "";
  return text;
}

function runtimeInputBitFromToken(token) {
  const normalized = normalizeRuntimeInputToken(token);
  if (!normalized) return 0;
  return Number(RUNTIME_INPUT_BITS[normalized] ?? 0);
}

function runtimeInputTokenFromKeyboardEvent(event) {
  const code = normalizeRuntimeInputToken(event?.code ?? "");
  if (code && Object.prototype.hasOwnProperty.call(RUNTIME_INPUT_BITS, code)) {
    return code;
  }
  const key = String(event?.key ?? "").trim();
  if (!key) return "";
  const lower = key.toLowerCase();
  if (lower === "arrowup" || lower === "up") return "ArrowUp";
  if (lower === "arrowleft" || lower === "left") return "ArrowLeft";
  if (lower === "arrowdown" || lower === "down") return "ArrowDown";
  if (lower === "arrowright" || lower === "right") return "ArrowRight";
  if (lower === " " || lower === "space" || lower === "spacebar") return "Space";
  if (lower === "enter") return "Enter";
  if (lower === "escape" || lower === "esc") return "Escape";
  if (lower === "z") return "KeyZ";
  if (lower === "x") return "KeyX";
  return "";
}

function readStorageJson(key, fallback) {
  try {
    if (typeof window === "undefined" || !window.localStorage) return fallback;
    const raw = window.localStorage.getItem(key);
    if (!raw) return fallback;
    const parsed = JSON.parse(raw);
    return parsed && typeof parsed === "object" ? parsed : fallback;
  } catch (_) {
    return fallback;
  }
}

function writeStorageJson(key, value) {
  try {
    if (typeof window === "undefined" || !window.localStorage) return;
    window.localStorage.setItem(key, JSON.stringify(value));
  } catch (_) {
    // ignore storage write errors
  }
}

function normalizeAxisToken(raw) {
  return String(raw ?? "").trim();
}

function readSelectOptionValues(selectEl) {
  const options = Array.isArray(selectEl?.options)
    ? selectEl.options
    : Array.from(selectEl?.options ?? []);
  return options
    .map((option) => normalizeAxisToken(option?.value))
    .filter(Boolean);
}

function pickPreferredGraphXAxis(candidates = [], yKey = "") {
  const keys = Array.isArray(candidates) ? candidates : [];
  const normalizedY = normalizeAxisToken(yKey).toLowerCase();
  const preferredTime = keys.find((key) => {
    const normalized = String(key).trim().toLowerCase();
    return ["t", "time", "시간", "tick", "프레임수"].includes(normalized);
  });
  const indexKey = keys.find((key) => normalizeAxisToken(key) === "__index__");
  if (preferredTime && preferredTime.toLowerCase() !== normalizedY) return preferredTime;
  if (indexKey && indexKey.toLowerCase() !== normalizedY) return indexKey;
  return keys.find((key) => String(key).trim().toLowerCase() !== normalizedY) ?? "";
}

function normalizeRunTab(raw) {
  const tab = String(raw ?? "").trim().toLowerCase();
  if (tab === "lesson" || tab === "ddn" || tab === "observe") return "console";
  if (tab === "result" || tab === "output" || tab === "table") return "output";
  if (tab === "inspector") return "mirror";
  if (RUN_TAB_IDS.includes(tab)) return tab;
  return RUN_TAB_IDS[0];
}

function normalizeRunPrimaryView(raw, fallback = "sim") {
  const view = String(raw ?? "").trim().toLowerCase();
  if (RUN_PRIMARY_VIEW_IDS.includes(view)) return view;
  if (RUN_PRIMARY_VIEW_IDS.includes(fallback)) return fallback;
  return "sim";
}

function normalizeEngineStatus(raw, fallback = "idle") {
  const status = String(raw ?? "").trim().toLowerCase();
  if (RUN_ENGINE_STATUS_IDS.includes(status)) return status;
  if (RUN_ENGINE_STATUS_IDS.includes(fallback)) return fallback;
  return "idle";
}

function classifyWarningAxis(code) {
  const token = String(code ?? "").trim().toUpperCase();
  if (!token) return "runtime";
  if (token.startsWith("E_PARSE")) return "parse";
  if (token.startsWith("W_PARSE")) return "parse";
  if (token.startsWith("W_BLOCK_HEADER")) return "parse";
  if (token.startsWith("E_RUNTIME") || token.startsWith("E_WASM")) return "runtime";
  return "contract";
}

function buildConsoleWarningSummary(warnings = []) {
  const rows = Array.isArray(warnings) ? warnings : [];
  if (!rows.length) {
    return {
      level: "ok",
      text: "경고 없음",
    };
  }
  let parse = 0;
  let runtime = 0;
  let contract = 0;
  rows.forEach((warning) => {
    const axis = classifyWarningAxis(warning?.code ?? warning?.technical_code ?? "");
    if (axis === "parse") parse += 1;
    else if (axis === "runtime") runtime += 1;
    else contract += 1;
  });
  const parts = [];
  if (parse > 0) parts.push(`문법 ${parse}`);
  if (runtime > 0) parts.push(`입력 ${runtime}`);
  if (contract > 0) parts.push(`논리 ${contract}`);
  const firstMessage = resolveUserWarningCause(rows[0]);
  const suffix = firstMessage ? ` · ${firstMessage}` : "";
  return {
    level: "warn",
    text: `경고 ${rows.length}건 (${parts.join(" / ")})${suffix}`,
  };
}

function normalizePlatformActionRail(raw) {
  if (!Array.isArray(raw)) return [];
  const out = [];
  const seen = new Set();
  raw.forEach((item) => {
    const token = String(item ?? "").trim();
    if (!token || seen.has(token)) return;
    seen.add(token);
    out.push(token);
  });
  return out;
}

function normalizeDockTarget(raw) {
  const target = String(raw ?? "").trim().toLowerCase();
  if (RUN_DOCK_TARGETS.includes(target)) return target;
  return "space2d";
}

function normalizeDockSpeed(raw) {
  const value = Number(raw);
  if (Number.isFinite(value) && RUN_DOCK_SPEEDS.includes(value)) return value;
  return 1;
}

export function normalizeRunMainVisualMode(raw, fallback = "none") {
  const mode = String(raw ?? "").trim().toLowerCase();
  if (["space2d", "graph", "console", "console-grid", "debug-fallback", "none"].includes(mode)) {
    return mode;
  }
  return ["space2d", "graph", "console", "console-grid", "debug-fallback", "none"].includes(String(fallback ?? "").trim().toLowerCase())
    ? String(fallback).trim().toLowerCase()
    : "none";
}

function toPlainObject(raw, fallback = {}) {
  return raw && typeof raw === "object" ? raw : fallback;
}

function escapeHtml(raw) {
  return String(raw ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function shortHash(raw, max = 8) {
  const text = String(raw ?? "").trim();
  if (!text) return "-";
  return text.length > max ? text.slice(0, max) : text;
}

function stableHashHex(raw) {
  const text = String(raw ?? "");
  let hash = 0x811c9dc5;
  for (let i = 0; i < text.length; i += 1) {
    hash ^= text.charCodeAt(i);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

function stableColorHue(label, hash) {
  const seed = `${String(label ?? "").trim()}|${String(hash ?? "").trim()}`;
  const hashed = Number.parseInt(stableHashHex(seed), 16);
  if (!Number.isFinite(hashed)) return 200;
  return hashed % 360;
}

function buildRunColor(hue, alpha = 1) {
  const h = Math.max(0, Math.min(359, Number(hue) || 0));
  const a = Number(alpha);
  if (!Number.isFinite(a) || a >= 1) {
    return `hsl(${h} 76% 56%)`;
  }
  const clamped = Math.max(0.06, Math.min(1, a));
  return `hsl(${h} 76% 56% / ${clamped})`;
}

const RUN_GRAPH_PRIMARY_COLOR = "#22d3ee";
const RUN_GRAPH_RANGE_RECENT_500 = "500";
const RUN_GRAPH_RANGE_RECENT_2000 = "2000";
const RUN_GRAPH_RANGE_ALL = "all";
const RUN_GRAPH_RANGE_OPTIONS = Object.freeze([
  RUN_GRAPH_RANGE_RECENT_500,
  RUN_GRAPH_RANGE_RECENT_2000,
  RUN_GRAPH_RANGE_ALL,
]);

function normalizeGraphRangeSelection(rawRange, fallback = RUN_GRAPH_RANGE_RECENT_500) {
  const range = String(rawRange ?? "").trim().toLowerCase();
  return RUN_GRAPH_RANGE_OPTIONS.includes(range) ? range : fallback;
}

function resolveRunManagerRowColor(run, { isActive = false, alpha = 1 } = {}) {
  if (isActive) {
    const numeric = Number(alpha);
    const clamped = Number.isFinite(numeric) ? Math.max(0.06, Math.min(1, numeric)) : 1;
    if (clamped >= 0.999) return RUN_GRAPH_PRIMARY_COLOR;
    return `rgba(34, 211, 238, ${clamped})`;
  }
  return buildRunColor(run?.hue, alpha);
}

function countGraphSeriesPoints(graphLike) {
  const graph = toPlainObject(graphLike, {});
  const series = Array.isArray(graph.series) ? graph.series : [];
  return series.reduce((acc, item) => {
    const points = Array.isArray(item?.points) ? item.points.length : 0;
    return acc + Math.max(0, points);
  }, 0);
}

function countRunGraphPoints(run) {
  const row = toPlainObject(run, {});
  const graph = toPlainObject(row.graph ?? row.result, {});
  const seriesPoints = countGraphSeriesPoints(graph);
  const directPoints = Math.max(0, Number(row.points) || 0);
  return Math.max(seriesPoints, directPoints);
}

function isRunManagerGraphRunMeaningful(run) {
  return countRunGraphPoints(run) > 0;
}

function isEmptyInitialWasmStateHash(raw) {
  return String(raw ?? "").trim() === EMPTY_INITIAL_WASM_STATE_HASH;
}

function hasMeaningfulSessionRuns(sessionLike = {}) {
  const row = toPlainObject(sessionLike, {});
  const runs = Array.isArray(row.runs) ? row.runs : [];
  const layers = Array.isArray(row.layers) ? row.layers : [];
  return runs.some(isRunManagerGraphRunMeaningful) || layers.some(isRunManagerGraphRunMeaningful);
}

export function buildRunManagerDisplayState({
  overlayRuns = [],
  activeOverlayRunId = "",
  hoverOverlayRunId = "",
  soloOverlayRunId = "",
} = {}) {
  const runs = [...(Array.isArray(overlayRuns) ? overlayRuns : [])]
    .sort((a, b) => normalizeRunManagerLayer(a?.layerIndex, 0) - normalizeRunManagerLayer(b?.layerIndex, 0));
  const activeId = String(activeOverlayRunId ?? "").trim();
  const hoverId = String(hoverOverlayRunId ?? "").trim();
  const soloId = String(soloOverlayRunId ?? "").trim();
  const activeRun = runs.find((row) => String(row?.id ?? "").trim() === activeId) ?? null;
  const activeVisible = activeRun ? activeRun.visible !== false : false;
  const baseVisible = Boolean(activeRun) && activeVisible && (!soloId || soloId === activeId);
  const baseAlpha = 1;
  const rows = runs.map((row) => {
    const id = String(row?.id ?? "").trim();
    const isActive = Boolean(id) && id === activeId;
    const isHovered = Boolean(id) && id === hoverId;
    const isSolo = Boolean(id) && id === soloId;
    const alpha = 1;
    return {
      run: row,
      id,
      label: String(row?.label ?? id).trim() || id,
      visible: row?.visible !== false,
      isActive,
      isHovered,
      isSolo,
      rowColor: resolveRunManagerRowColor(row, { isActive, alpha }),
      hashText: shortHash(row?.hash?.result || row?.hash?.input || ""),
    };
  });
  const overlaySeries = runs.flatMap((run) => {
    const runId = String(run?.id ?? "").trim();
    if (!runId) return [];
    if (runId === activeId) return [];
    if (run.visible === false) return [];
    if (soloId && runId !== soloId) return [];
    const primary = pickPrimarySeriesFromGraph(run?.graph);
    const points = normalizeGraphSeriesPoints(primary?.points);
    if (!points.length) return [];
    const alpha = 1;
    return [{
      id: `overlay:${runId}`,
      color: resolveRunManagerRowColor(run, { isActive: false, alpha }),
      points,
    }];
  });
  return {
    rows,
    overlaySeries,
    activeRun,
    baseVisible,
    baseAlpha,
    baseColor: resolveRunManagerRowColor(activeRun, { isActive: true, alpha: baseAlpha }),
  };
}

function buildRunHistoryExportSummaryRows(runs = []) {
  return (Array.isArray(runs) ? runs : [])
    .map((row, index) => {
      const id = String(row?.id ?? `run-${index + 1}`).trim() || `run-${index + 1}`;
      const label = String(row?.label ?? id).trim() || id;
      const inputHash = String(row?.hash?.input ?? "").trim();
      const resultHash = String(row?.hash?.result ?? "").trim();
      return {
        id,
        label,
        visible: row?.visible !== false,
        layer_index: normalizeRunManagerLayer(row?.layerIndex ?? row?.layer_index, index),
        input_hash: inputHash,
        result_hash: resultHash,
        hash_label: shortHash(resultHash || inputHash || ""),
        point_count: countRunGraphPoints(row),
      };
    })
    .sort((a, b) => a.layer_index - b.layer_index);
}

function formatRunHistoryExportSummaryText(model = {}) {
  const payload = model && typeof model === "object" ? model : {};
  const lines = [
    "Seamgrim run history export summary",
    `schema: ${String(payload.schema ?? "seamgrim.run_history_export_summary.v1")}`,
    `runs: ${Math.max(0, Number(payload.run_count) || 0)}`,
    `visible: ${Math.max(0, Number(payload.visible_count) || 0)}`,
    `hidden: ${Math.max(0, Number(payload.hidden_count) || 0)}`,
    `active: ${String(payload.active_label ?? "").trim() || "-"}`,
    `solo: ${String(payload.solo_label ?? "").trim() || "-"}`,
  ];
  const rows = Array.isArray(payload.runs) ? payload.runs : [];
  if (!rows.length) {
    lines.push("items: -");
    return lines.join("\n");
  }
  lines.push("items:");
  rows.forEach((row, index) => {
    const label = String(row?.label ?? row?.id ?? `run-${index + 1}`).trim() || `run-${index + 1}`;
    const visible = row?.visible === false ? "hidden" : "visible";
    const hash = String(row?.hash_label ?? row?.result_hash ?? row?.input_hash ?? "").trim() || "-";
    const pointCount = Math.max(0, Number(row?.point_count) || 0);
    lines.push(`${index + 1}. ${label} [${visible}] hash:${hash} points:${pointCount}`);
  });
  return lines.join("\n");
}

function normalizeGraphSeriesPoints(points) {
  const source = Array.isArray(points) ? points : [];
  return source
    .map((row) => {
      const x = finiteNumber(row?.x);
      const y = finiteNumber(row?.y);
      if (x === null || y === null) return null;
      return { x, y };
    })
    .filter(Boolean);
}

function normalizeGraphSample(raw) {
  const row = raw && typeof raw === "object" ? raw : {};
  const varName = String(row.var ?? row.variable ?? "").trim();
  const xMin = finiteNumber(row.x_min ?? row.xMin);
  const xMax = finiteNumber(row.x_max ?? row.xMax);
  const step = finiteNumber(row.step);
  if (!varName || xMin === null || xMax === null || step === null || xMax < xMin || step <= 0) {
    return null;
  }
  const tick = finiteNumber(row.tick);
  return {
    var: varName,
    x_min: xMin,
    x_max: xMax,
    step,
    ...(tick === null ? {} : { tick }),
  };
}

function normalizeGraphView(raw) {
  const row = raw && typeof raw === "object" ? raw : {};
  const xMin = finiteNumber(row.x_min ?? row.xMin);
  const xMax = finiteNumber(row.x_max ?? row.xMax);
  const yMin = finiteNumber(row.y_min ?? row.yMin);
  const yMax = finiteNumber(row.y_max ?? row.yMax);
  if ([xMin, xMax, yMin, yMax].some((value) => value === null) || xMax <= xMin || yMax <= yMin) {
    return null;
  }
  return {
    auto: Boolean(row.auto),
    x_min: xMin,
    x_max: xMax,
    y_min: yMin,
    y_max: yMax,
    pan_x: finiteNumber(row.pan_x ?? row.panX) ?? 0,
    pan_y: finiteNumber(row.pan_y ?? row.panY) ?? 0,
    zoom: finiteNumber(row.zoom) ?? 1,
  };
}

function stableStringify(value) {
  if (value === null || value === undefined) return "";
  if (typeof value === "number" || typeof value === "boolean") return JSON.stringify(value);
  if (typeof value === "string") return JSON.stringify(value);
  if (Array.isArray(value)) {
    return `[${value.map((item) => stableStringify(item)).join(",")}]`;
  }
  if (typeof value === "object") {
    const keys = Object.keys(value).sort((a, b) => a.localeCompare(b, "ko"));
    const pairs = keys.map((key) => `${JSON.stringify(key)}:${stableStringify(value[key])}`);
    return `{${pairs.join(",")}}`;
  }
  return JSON.stringify(String(value));
}

function buildRunInputHash({ ddnText = "", controls = {}, sample = null } = {}) {
  const seed = [
    String(ddnText ?? "").trim(),
    stableStringify(controls ?? {}),
    stableStringify(sample ?? {}),
  ].join("||");
  return stableHashHex(seed);
}

function cloneGraphForRunManager(rawGraph) {
  const graph = rawGraph && typeof rawGraph === "object" ? rawGraph : null;
  if (!graph) return null;
  const axisRaw = graph.axis && typeof graph.axis === "object" ? graph.axis : null;
  const axis = axisRaw
    ? {
      x_min: finiteNumber(axisRaw.x_min),
      x_max: finiteNumber(axisRaw.x_max),
      y_min: finiteNumber(axisRaw.y_min),
      y_max: finiteNumber(axisRaw.y_max),
    }
    : null;
  const seriesRaw = Array.isArray(graph.series) ? graph.series : [];
  const series = seriesRaw
    .map((row, index) => {
      const points = normalizeGraphSeriesPoints(row?.points);
      if (!points.length) return null;
      return {
        id: String(row?.id ?? row?.label ?? `series_${index + 1}`).trim() || `series_${index + 1}`,
        points,
      };
    })
    .filter(Boolean);
  if (!series.length) return null;
  const meta = toPlainObject(graph.meta, {});
  const schema = String(graph.schema ?? "").trim();
  const sample = normalizeGraphSample(graph.sample ?? null);
  const view = normalizeGraphView(graph.view ?? null);
  return {
    ...(schema ? { schema } : {}),
    axis: axis && Object.values(axis).every((value) => Number.isFinite(value)) ? axis : null,
    ...(sample ? { sample } : {}),
    ...(view ? { view } : {}),
    series,
    meta: { ...meta },
  };
}

export function resolveLiveRunCaptureGraph({
  runtimeGraph = null,
  runtimeGraphSource = "",
  fallbackGraph = null,
} = {}) {
  const runtime = cloneGraphForRunManager(runtimeGraph);
  const fallback = cloneGraphForRunManager(fallbackGraph);
  if (!runtime) return fallback;
  if (!fallback) return runtime;

  const source = String(runtimeGraphSource ?? runtime?.meta?.source ?? "").trim();
  const runtimePoints = countGraphSeriesPoints(runtime);
  const fallbackPoints = countGraphSeriesPoints(fallback);
  if (source === "observation-fallback") {
    return fallback;
  }
  if (runtimePoints <= 1 && fallbackPoints > runtimePoints) {
    return fallback;
  }
  return runtime;
}

function pickPrimarySeriesFromGraph(graph) {
  const series = Array.isArray(graph?.series) ? graph.series : [];
  if (!series.length) return null;
  return series.find((row) => Array.isArray(row?.points) && row.points.length > 0) ?? null;
}

function normalizeRunManagerLayer(raw, fallback = 0) {
  const num = Number(raw);
  if (!Number.isFinite(num)) return fallback;
  return Math.max(0, Math.trunc(num));
}

function formatAxisRange(range) {
  if (!range || typeof range !== "object") return "-";
  const xMin = Number(range.x_min ?? range.xMin);
  const xMax = Number(range.x_max ?? range.xMax);
  const yMin = Number(range.y_min ?? range.yMin);
  const yMax = Number(range.y_max ?? range.yMax);
  if (![xMin, xMax, yMin, yMax].every(Number.isFinite) || xMax <= xMin || yMax <= yMin) {
    return "-";
  }
  return `x[${formatStatusNumber(xMin, 2)}, ${formatStatusNumber(xMax, 2)}] y[${formatStatusNumber(yMin, 2)}, ${formatStatusNumber(yMax, 2)}]`;
}

function normalizeRunRequiredViews(requiredViews) {
  return normalizeViewFamilyList(requiredViews);
}

function resolveLessonRequiredViewsForRun(lesson) {
  const declared = normalizeRunRequiredViews(lesson?.requiredViews ?? lesson?.required_views ?? []);
  if (declared.length > 0) return declared;
  const inferred = [];
  if (Array.isArray(lesson?.graphCandidates) && lesson.graphCandidates.length > 0) inferred.push("graph");
  if (Array.isArray(lesson?.tableCandidates) && lesson.tableCandidates.length > 0) inferred.push("table");
  if (String(lesson?.textMd ?? "").trim()) inferred.push("text");
  return normalizeRunRequiredViews(inferred);
}

export function resolveRunLayoutProfile(requiredViews) {
  const families = normalizeRunRequiredViews(requiredViews);
  const hasSpatial = hasSpatialViewFamily(families);
  const hasGraph = families.includes("graph");
  const hasTable = families.includes("table");
  const hasTextual = families.includes("text") || families.includes("structure");
  const hasDockPanels = hasGraph || hasTable || hasTextual;
  const mode = hasSpatial && hasDockPanels
    ? "split"
    : hasSpatial
      ? "space_primary"
      : hasDockPanels
        ? "split"
        : "split";

  return {
    mode,
    families,
    hasSpatial,
    hasGraph,
    hasTable,
    hasTextual,
    hasDockPanels,
  };
}

export function resolveRunDockPanelOrder(requiredViews) {
  return resolveRunDockPanelOrderFromFamilies(requiredViews);
}

function hasRuntimeGraphContent(runtimeDerived) {
  const observation = runtimeDerived?.observation ?? null;
  const channels = Array.isArray(observation?.channels) ? observation.channels.length : 0;
  if (channels > 0) return true;
  const graph = runtimeDerived?.views?.graph;
  const series = Array.isArray(graph?.series) ? graph.series : [];
  return series.some((row) => Array.isArray(row?.points) && row.points.length > 0);
}

function hasRuntimeTableContent(runtimeDerived) {
  return Boolean(normalizeRuntimeTableView(runtimeDerived?.views?.table ?? null));
}

function hasRuntimeTextContent(markdown) {
  return Boolean(String(markdown ?? "").trim());
}

export function resolveRunDockPanelVisibility(requiredViews, { runtimeDerived = null, textMarkdown = "" } = {}) {
  const families = new Set(normalizeRunRequiredViews(requiredViews));
  return {
    graph: families.has("graph") || hasRuntimeGraphContent(runtimeDerived),
    table: families.has("table") || hasRuntimeTableContent(runtimeDerived),
    text: families.has("text") || families.has("structure") || hasRuntimeTextContent(textMarkdown),
  };
}

function deriveRunKindAndChannels({ observation = null, hasSpace2d = false } = {}) {
  const channels = Array.isArray(observation?.channels) ? observation.channels.length : 0;
  if (!hasSpace2d && channels <= 0) {
    return { kind: "empty", channels };
  }
  if (!hasSpace2d) {
    return { kind: "obs_only", channels };
  }
  return { kind: "space2d", channels };
}

function hasSpace2dDrawable(space2d) {
  if (!space2d || typeof space2d !== "object") return false;
  const shapes = Array.isArray(space2d.shapes) ? space2d.shapes.length : 0;
  const points = Array.isArray(space2d.points) ? space2d.points.length : 0;
  const drawlist = Array.isArray(space2d.drawlist) ? space2d.drawlist.length : 0;
  return shapes > 0 || points > 0 || drawlist > 0;
}

function parseFiniteNumericValue(raw, depth = 0) {
  if (depth > 2) return null;
  if (raw === null || raw === undefined) return null;
  if (typeof raw === "number") {
    return Number.isFinite(raw) ? raw : null;
  }
  if (typeof raw === "string") {
    const text = raw.trim();
    if (!text) return null;
    const num = Number(text);
    return Number.isFinite(num) ? num : null;
  }
  if (Array.isArray(raw)) {
    for (const item of raw) {
      const hit = parseFiniteNumericValue(item, depth + 1);
      if (hit !== null) return hit;
    }
    return null;
  }
  if (typeof raw === "object") {
    const fields = ["value", "num", "number", "raw", "f64", "i64", "fixed64", "scalar"];
    for (const field of fields) {
      if (!Object.prototype.hasOwnProperty.call(raw, field)) continue;
      const hit = parseFiniteNumericValue(raw[field], depth + 1);
      if (hit !== null) return hit;
    }
  }
  return null;
}

function readObservationChannelKey(channel) {
  if (typeof channel === "string") return channel.trim();
  if (!channel || typeof channel !== "object") return "";
  const direct = String(channel.key ?? "").trim();
  if (direct) return direct;
  return String(channel.name ?? channel.id ?? channel.label ?? channel.token ?? "").trim();
}

function readObservationValueEntries(observation) {
  const valuesSource = observation && typeof observation.all_values === "object"
    ? observation.all_values
    : observation && typeof observation.values === "object"
      ? observation.values
      : {};
  const values = valuesSource && typeof valuesSource === "object" ? valuesSource : {};
  const directEntries = Object.entries(values);
  if (directEntries.length > 0) return directEntries;

  const channels = Array.isArray(observation?.channels) ? observation.channels : [];
  const row = Array.isArray(observation?.row) ? observation.row : [];
  if (!channels.length || !row.length) return [];

  const out = [];
  channels.forEach((channel, index) => {
    const key = readObservationChannelKey(channel);
    if (!key) return;
    out.push([key, row[index]]);
  });
  return out;
}

function isTimeLikeObservationKey(rawKey) {
  const key = String(rawKey ?? "").trim().toLowerCase();
  if (!key) return false;
  return key === "t" || key === "time" || key === "tick" || key === "frame" || key === "프레임수" || key === "시간";
}

function readFallbackAngleFromObservation(observation) {
  const entries = readObservationValueEntries(observation);
  if (!entries.length) return null;

  const aliases = ["theta", "각도", "angle", "rad"];
  for (const [key, raw] of entries) {
    const normalized = String(key ?? "").trim().toLowerCase();
    if (!normalized) continue;
    if (!aliases.some((alias) => normalized.includes(alias))) continue;
    const num = parseFiniteNumericValue(raw);
    if (num !== null) return num;
  }
  return null;
}

function readNumericObservationValue(observation, keys = []) {
  const entries = readObservationValueEntries(observation);
  for (const key of keys) {
    const target = String(key ?? "").trim();
    if (!target) continue;
    const direct = entries.find(([entryKey]) => String(entryKey ?? "").trim() === target);
    const lower = direct
      ? null
      : entries.find(([entryKey]) => String(entryKey ?? "").trim().toLowerCase() === target.toLowerCase());
    const hit = direct ?? lower ?? null;
    if (!hit) continue;
    const num = parseFiniteNumericValue(hit[1]);
    if (num !== null) return num;
  }
  return null;
}

function finiteNumber(raw) {
  const n = Number(raw);
  return Number.isFinite(n) ? n : null;
}

function normalizeEditorRatio(
  raw,
  {
    min = STUDIO_EDITOR_RATIO_MIN,
    max = STUDIO_EDITOR_RATIO_MAX,
    fallback = STUDIO_EDITOR_RATIO_DEFAULT,
  } = {},
) {
  const minRatio = Number.isFinite(Number(min)) ? Number(min) : STUDIO_EDITOR_RATIO_MIN;
  const maxRatio = Number.isFinite(Number(max)) ? Number(max) : STUDIO_EDITOR_RATIO_MAX;
  const lo = Math.max(0, Math.min(1, Math.min(minRatio, maxRatio)));
  const hi = Math.max(lo, Math.min(1, Math.max(minRatio, maxRatio)));
  const value = Number(raw);
  if (!Number.isFinite(value)) {
    return Math.min(hi, Math.max(lo, Number.isFinite(Number(fallback)) ? Number(fallback) : STUDIO_EDITOR_RATIO_DEFAULT));
  }
  return Math.min(hi, Math.max(lo, value));
}

export function resolveStudioLayoutBounds({
  layoutWidth = 0,
  layoutHeight = 0,
  splitterWidth = STUDIO_LAYOUT_SPLITTER_WIDTH,
  toolbarHeight = 0,
  errorBannerHeight = 0,
  minVisualWidth = STUDIO_LAYOUT_MIN_VISUAL_WIDTH_PX,
  subpanelMinHeight = STUDIO_LAYOUT_SUBPANEL_MIN_HEIGHT_PX,
  bogaeAspectRatio = STUDIO_LAYOUT_BOGAE_ASPECT_RATIO,
  baseMinEditorRatio = STUDIO_EDITOR_RATIO_MIN,
  baseMaxEditorRatio = STUDIO_EDITOR_RATIO_MAX,
} = {}) {
  const width = Math.max(0, Number.isFinite(Number(layoutWidth)) ? Number(layoutWidth) : 0);
  const height = Math.max(0, Number.isFinite(Number(layoutHeight)) ? Number(layoutHeight) : 0);
  const splitter = Math.max(0, Number.isFinite(Number(splitterWidth)) ? Number(splitterWidth) : STUDIO_LAYOUT_SPLITTER_WIDTH);
  const toolbar = Math.max(0, Number.isFinite(Number(toolbarHeight)) ? Number(toolbarHeight) : 0);
  const banner = Math.max(0, Number.isFinite(Number(errorBannerHeight)) ? Number(errorBannerHeight) : 0);
  const visualMinWidth = Math.max(0, Number.isFinite(Number(minVisualWidth)) ? Number(minVisualWidth) : STUDIO_LAYOUT_MIN_VISUAL_WIDTH_PX);
  const subpanelMin = Math.max(0, Number.isFinite(Number(subpanelMinHeight)) ? Number(subpanelMinHeight) : STUDIO_LAYOUT_SUBPANEL_MIN_HEIGHT_PX);
  const aspect = Math.max(0.01, Number.isFinite(Number(bogaeAspectRatio)) ? Number(bogaeAspectRatio) : STUDIO_LAYOUT_BOGAE_ASPECT_RATIO);

  const baseMin = Math.max(0, Math.min(1, Number.isFinite(Number(baseMinEditorRatio)) ? Number(baseMinEditorRatio) : STUDIO_EDITOR_RATIO_MIN));
  const baseMax = Math.max(baseMin, Math.min(1, Number.isFinite(Number(baseMaxEditorRatio)) ? Number(baseMaxEditorRatio) : STUDIO_EDITOR_RATIO_MAX));

  if (width <= 0 || height <= 0) {
    return {
      editorRatioMin: baseMin,
      editorRatioMax: baseMax,
      visualMinWidth,
      subpanelMinHeight: subpanelMin,
      availableVisualHeight: 0,
      bogaeFrameMaxWidthPx: 0,
      bogaeFrameMaxHeightPx: 0,
      hasWidthOverflow: false,
      hasHeightOverflow: false,
      hasConstraintOverflow: false,
    };
  }

  const visualBudgetWidth = Math.max(0, width - splitter);
  const maxEditorWidthByVisual = Math.max(0, width - splitter - visualMinWidth);
  const maxEditorRatioByVisual = width > 0 ? maxEditorWidthByVisual / width : 0;
  const editorRatioMax = Math.max(0, Math.min(baseMax, maxEditorRatioByVisual));
  const editorRatioMin = Math.max(0, Math.min(baseMin, editorRatioMax));

  const availableVisualHeight = Math.max(0, height - toolbar - banner);
  const bogaeFrameMaxHeightPx = Math.max(0, availableVisualHeight - subpanelMin);
  const bogaeFrameMaxWidthPx = bogaeFrameMaxHeightPx > 0 ? bogaeFrameMaxHeightPx * aspect : 0;

  const hasWidthOverflow = visualBudgetWidth < visualMinWidth;
  const hasHeightOverflow = availableVisualHeight < subpanelMin;
  const hasConstraintOverflow = hasWidthOverflow || hasHeightOverflow;

  return {
    editorRatioMin,
    editorRatioMax,
    visualMinWidth,
    subpanelMinHeight: subpanelMin,
    availableVisualHeight,
    bogaeFrameMaxWidthPx,
    bogaeFrameMaxHeightPx,
    hasWidthOverflow,
    hasHeightOverflow,
    hasConstraintOverflow,
  };
}

function formatStatusNumber(raw, digits = 3) {
  const n = Number(raw);
  if (!Number.isFinite(n)) return "";
  return Number(n.toFixed(digits)).toString();
}

function normalizeParseWarnings(rawWarnings) {
  if (!Array.isArray(rawWarnings)) return [];
  return rawWarnings
    .filter((warning) => warning && typeof warning === "object")
    .map((warning) => {
      const code = String(warning.code ?? warning.technical_code ?? "").trim();
      const technicalCode = String(warning.technical_code ?? code).trim() || code;
      const technicalMessage = String(warning.technical_message ?? warning.message ?? warning.error ?? "").trim();
      const userMessageRaw = String(warning.user_message ?? warning.userMessage ?? "").trim();
      const userMessage = userMessageRaw || mapParseWarningToUserMessage(code, technicalMessage);
      return {
        code,
        technical_code: technicalCode,
        message: userMessage,
        technical_message: technicalMessage,
        span: warning.span ?? null,
      };
    });
}

function filterInternalLegacyHeaderWarnings(rawWarnings, sourceText = "") {
  const list = Array.isArray(rawWarnings) ? rawWarnings : [];
  if (hasLegacyAutofixCandidate(sourceText)) {
    return list;
  }
  return list.filter((warning) => {
    const code = String(warning?.code ?? warning?.technical_code ?? "").trim();
    return code !== "W_BLOCK_HEADER_COLON_DEPRECATED" && code !== "E_BLOCK_HEADER_COLON_FORBIDDEN";
  });
}

function buildServerRunDiagnosticWarning(payload) {
  if (!payload || typeof payload !== "object") return null;
  const code = String(payload.code ?? payload.technical_code ?? "").trim() || "E_RUNTIME_EXEC_FAILED";
  const technicalMessage = String(payload.technical_message ?? payload.error ?? "").trim();
  const userMessage = String(payload.user_message ?? "").trim()
    || mapParseWarningToUserMessage(code, technicalMessage);
  return {
    code,
    technical_code: code,
    message: technicalMessage || "실행 요청 처리에 실패했습니다.",
    technical_message: technicalMessage || "실행 요청 처리에 실패했습니다.",
    user_message: userMessage,
    span: null,
  };
}

function formatParseWarningSummary(warnings) {
  const normalized = normalizeParseWarnings(warnings);
  if (!normalized.length) return "";
  if (normalized.length === 1) {
    const axis = classifyWarningAxis(normalized[0]?.code ?? normalized[0]?.technical_code ?? "");
    if (axis === "runtime") return "점검 필요: 입력 경고 1건";
    if (axis === "contract") return "점검 필요: 논리 경고 1건";
    return `점검 필요: ${normalized[0].message}`;
  }
  return `점검 필요: ${normalized.length}건`;
}

function translateRunErrorBannerMessage(warning) {
  const code = String(warning?.code ?? warning?.technical_code ?? "").trim();
  const userMessage = resolveUserWarningCause(warning);
  const technicalMessage = String(warning?.technical_message ?? "").trim();
  const message = userMessage || technicalMessage;
  const parseSignal =
    code.startsWith("E_PARSE")
    || message.includes("파싱")
    || technicalMessage.includes("파싱")
    || /\bparse(?:\s+error|\s+failed|\s+failure)?\b/i.test(technicalMessage);
  if (parseSignal) {
    return `문법 문제: ${userMessage || technicalMessage || code || "파싱 오류"}`;
  }
  if (code === "E_WASM_DIRECT_ONLY_FALLBACK_BLOCKED") {
    const protocol = String(globalThis?.location?.protocol ?? "").trim().toLowerCase();
    if (protocol === "file:") {
      return "WASM 로딩 실패: 웹 서버를 통해 실행해 주세요 (file:// 직접 열기 불가)";
    }
    const detail = technicalMessage.toLowerCase();
    if (
      detail.includes("failed to fetch")
      || detail.includes("wasm module")
      || detail.includes("dynamic import")
      || detail.includes("cannot find module")
      || detail.includes("wasm module 로드에 실패")
      || detail.includes("wasm module 초기화에 실패")
      || detail.includes("wasm wrapper 로드에 실패")
    ) {
      return "WASM 로딩 실패: wasm 경로/정적 서버 설정을 확인해 주세요.";
    }
    if (technicalMessage) {
      const singleLine = technicalMessage.replace(/\s+/g, " ").trim();
      const clipped = singleLine.length > 140 ? `${singleLine.slice(0, 140)}...` : singleLine;
      return `WASM 실행 실패: ${clipped}`;
    }
    return "WASM 실행 실패: 브라우저 콘솔과 입력을 점검해 주세요.";
  }
  const category = classifyToUserCategory(code);
  const label = category === "문법" ? "문법" : (category === "논리" ? "논리" : "입력");
  return `${label} 문제: ${message || code || "실행 실패"}`;
}

function readGraphSeriesId(series) {
  if (!series || typeof series !== "object") return "";
  return String(series.id ?? series.name ?? series.label ?? "").trim();
}

function readLatestPointFromSeries(series) {
  if (!series || typeof series !== "object") return null;
  const points = Array.isArray(series.points) ? series.points : [];
  for (let i = points.length - 1; i >= 0; i -= 1) {
    const row = points[i];
    const x = finiteNumber(row?.x);
    const y = finiteNumber(row?.y);
    if (x === null || y === null) continue;
    return { x, y };
  }
  return null;
}

function isPendulumSeriesId(seriesId) {
  const normalized = String(seriesId ?? "").trim().toLowerCase();
  if (!normalized) return false;
  return ["theta", "각도", "angle", "rad"].some((token) => normalized.includes(token));
}

function readPreferredGraphSeries(graph) {
  const seriesList = Array.isArray(graph?.series) ? graph.series : [];
  if (!seriesList.length) return null;

  const pendulumHit = seriesList.find((series) => isPendulumSeriesId(readGraphSeriesId(series)));
  if (pendulumHit) return pendulumHit;

  return seriesList.find((series) => readLatestPointFromSeries(series)) ?? null;
}

export function synthesizePendulumSpace2dFromObservation(observation) {
  const thetaRaw = readNumericObservationValue(observation, ["theta", "각도", "theta_rad"]);
  const theta = thetaRaw === null ? readFallbackAngleFromObservation(observation) : thetaRaw;
  if (theta === null) return null;

  const lengthRaw = readNumericObservationValue(observation, ["L", "length", "len", "길이"]);
  const length = Math.max(0.2, Math.min(5, Number.isFinite(lengthRaw) ? lengthRaw : 1));
  const bx = length * Math.sin(theta);
  const by = -length * Math.cos(theta);
  const span = length + 0.35;

  return {
    schema: "seamgrim.space2d.v0",
    meta: {
      title: "pendulum-observation-fallback",
      source: "observation",
    },
    camera: {
      x_min: -span,
      x_max: span,
      y_min: -(length + 0.6),
      y_max: 0.6,
    },
    points: [{ x: bx, y: by }],
    shapes: [
      { kind: "line", x1: 0, y1: 0, x2: bx, y2: by, stroke: "#9ca3af", width: 0.02, group_id: "pendulum.rod" },
      { kind: "circle", x: bx, y: by, r: 0.08, fill: "#38bdf8", stroke: "#0ea5e9", width: 0.02, group_id: "pendulum.bob" },
      { kind: "point", x: 0, y: 0, size: 0.045, color: "#f59e0b", group_id: "pendulum.pivot" },
    ],
  };
}

export function synthesizePointSpace2dFromObservation(observation) {
  const x = readNumericObservationValue(observation, ["x", "x_pos", "pos_x", "px", "위치x"]);
  const y = readNumericObservationValue(observation, ["y", "y_pos", "pos_y", "py", "위치y"]);
  if (x === null && y === null) return null;

  const px = x === null ? 0 : x;
  const py = y === null ? 0 : y;
  const xMin = Math.min(0, px) - 1;
  const xMax = Math.max(0, px) + 1;
  const yMin = Math.min(0, py) - 1;
  const yMax = Math.max(0, py) + 1;

  return {
    schema: "seamgrim.space2d.v0",
    meta: {
      title: y === null ? "x-observation-fallback" : "xy-observation-fallback",
      source: "observation",
    },
    camera: {
      x_min: xMin,
      x_max: xMax,
      y_min: yMin,
      y_max: yMax,
    },
    points: [{ x: px, y: py }],
    shapes: [
      { kind: "line", x1: xMin, y1: 0, x2: xMax, y2: 0, stroke: "#4b5563", width: 0.01, group_id: "graph.axis.x" },
      { kind: "line", x1: 0, y1: yMin, x2: 0, y2: yMax, stroke: "#374151", width: 0.01, group_id: "graph.axis.y" },
      { kind: "circle", x: px, y: py, r: 0.07, fill: "#22c55e", stroke: "#16a34a", width: 0.02, group_id: "graph.point.focus" },
    ],
  };
}

export function synthesizeSpace2dFromGraph(graph, observation = null) {
  const hitSeries = readPreferredGraphSeries(graph);
  if (!hitSeries) return null;
  const point = readLatestPointFromSeries(hitSeries);
  if (!point) return null;

  const seriesId = readGraphSeriesId(hitSeries);
  if (isPendulumSeriesId(seriesId)) {
    const theta = point.y;
    const lengthRaw = readNumericObservationValue(observation, ["L", "length", "len", "길이"]);
    const length = Math.max(0.2, Math.min(5, Number.isFinite(lengthRaw) ? lengthRaw : 1));
    const bx = length * Math.sin(theta);
    const by = -length * Math.cos(theta);
    const span = length + 0.35;
    return {
      schema: "seamgrim.space2d.v0",
      meta: {
        title: "pendulum-graph-fallback",
        source: "graph",
      },
      camera: {
        x_min: -span,
        x_max: span,
        y_min: -(length + 0.6),
        y_max: 0.6,
      },
      points: [{ x: bx, y: by }],
      shapes: [
        { kind: "line", x1: 0, y1: 0, x2: bx, y2: by, stroke: "#9ca3af", width: 0.02, group_id: "pendulum.rod" },
        { kind: "circle", x: bx, y: by, r: 0.08, fill: "#38bdf8", stroke: "#0ea5e9", width: 0.02, group_id: "pendulum.bob" },
        { kind: "point", x: 0, y: 0, size: 0.045, color: "#f59e0b", group_id: "pendulum.pivot" },
      ],
    };
  }

  const axis = graph && typeof graph === "object" ? graph.axis : null;
  const rawPoints = Array.isArray(hitSeries?.points) ? hitSeries.points : [];
  const trailPoints = rawPoints
    .map((row) => {
      const x = finiteNumber(row?.x);
      const y = finiteNumber(row?.y);
      if (x === null || y === null) return null;
      return { x, y };
    })
    .filter(Boolean)
    .slice(-32);
  if (!trailPoints.length) {
    trailPoints.push({ x: point.x, y: point.y });
  }
  const xs = trailPoints.map((row) => row.x);
  const ys = trailPoints.map((row) => row.y);
  const xMin = finiteNumber(axis?.x_min) ?? (Math.min(0, ...xs) - 1);
  const xMax = finiteNumber(axis?.x_max) ?? (Math.max(0, ...xs) + 1);
  const yMin = finiteNumber(axis?.y_min) ?? (Math.min(0, ...ys) - 1);
  const yMax = finiteNumber(axis?.y_max) ?? (Math.max(0, ...ys) + 1);
  const trailShapes = [];
  for (let i = 1; i < trailPoints.length; i += 1) {
    const from = trailPoints[i - 1];
    const to = trailPoints[i];
    trailShapes.push({
      kind: "line",
      x1: from.x,
      y1: from.y,
      x2: to.x,
      y2: to.y,
      stroke: "rgba(59, 130, 246, 0.52)",
      width: 0.01,
      group_id: "graph.series.trail",
    });
  }
  return {
    schema: "seamgrim.space2d.v0",
    meta: {
      title: "graph-point-fallback",
      source: "graph",
    },
    camera: {
      x_min: xMin,
      x_max: xMax,
      y_min: yMin,
      y_max: yMax,
    },
    points: trailPoints,
    shapes: [
      { kind: "line", x1: xMin, y1: 0, x2: xMax, y2: 0, stroke: "#4b5563", width: 0.01, group_id: "graph.axis.x" },
      { kind: "line", x1: 0, y1: yMin, x2: 0, y2: yMax, stroke: "#374151", width: 0.01, group_id: "graph.axis.y" },
      ...trailShapes,
      { kind: "circle", x: point.x, y: point.y, r: 0.07, fill: "#22c55e", stroke: "#16a34a", width: 0.02, group_id: "graph.point.focus" },
    ],
  };
}

function synthesizePendulumSpace2dFromGraph(graph, observation = null) {
  const candidate = synthesizeSpace2dFromGraph(graph, observation);
  const title = String(candidate?.meta?.title ?? "").trim();
  if (!candidate || !title.startsWith("pendulum-")) return null;
  return candidate;
}

export function synthesizeSpace2dFromObservation(observation) {
  return (
    synthesizePendulumSpace2dFromObservation(observation) ??
    synthesizePointSpace2dFromObservation(observation)
  );
}

function readNumericOutputRowsValue(outputRows = [], keys = []) {
  const rows = normalizeObserveOutputRows(outputRows);
  if (!rows.length) return null;
  const targets = Array.isArray(keys) ? keys.map((key) => String(key ?? "").trim()).filter(Boolean) : [];
  if (!targets.length) return null;
  for (let i = rows.length - 1; i >= 0; i -= 1) {
    const row = rows[i];
    const rowKey = String(row?.key ?? "").trim();
    if (!rowKey) continue;
    const hit = targets.some((target) => rowKey === target || rowKey.toLowerCase() === target.toLowerCase());
    if (!hit) continue;
    const num = parseFiniteNumericValue(row?.value);
    if (num !== null) return num;
  }
  return null;
}

export function synthesizeSpace2dFromOutputRows(outputRows = [], { prevSpace2d = null } = {}) {
  const rows = normalizeObserveOutputRows(outputRows);
  if (!rows.length) return null;
  let x = readNumericOutputRowsValue(rows, ["x", "t", "time", "tick", "프레임수", "시간"]);
  let y = readNumericOutputRowsValue(rows, ["y", "theta", "각도", "value", "값"]);
  if (x === null && y === null) {
    const numericValues = rows
      .map((row) => parseFiniteNumericValue(row?.value))
      .filter((value) => value !== null);
    if (numericValues.length >= 2) {
      x = numericValues[numericValues.length - 2];
      y = numericValues[numericValues.length - 1];
    } else if (numericValues.length === 1) {
      x = 0;
      y = numericValues[0];
    }
  }
  if (x === null && y === null) return null;

  const px = x === null ? 0 : x;
  const py = y === null ? 0 : y;
  const prevPoints = Array.isArray(prevSpace2d?.points)
    ? prevSpace2d.points
        .filter((point) => Number.isFinite(Number(point?.x)) && Number.isFinite(Number(point?.y)))
        .slice(-24)
        .map((point) => ({ x: Number(point.x), y: Number(point.y) }))
    : [];
  const nextPoints = [...prevPoints];
  const lastPoint = nextPoints[nextPoints.length - 1] ?? null;
  if (!lastPoint || lastPoint.x !== px || lastPoint.y !== py) {
    nextPoints.push({ x: px, y: py });
  }
  const points = nextPoints.slice(-25);
  const xs = points.map((point) => point.x);
  const ys = points.map((point) => point.y);
  const xMin = Math.min(-1, ...xs) - 1;
  const xMax = Math.max(1, ...xs) + 1;
  const yMin = Math.min(-1, ...ys) - 1;
  const yMax = Math.max(1, ...ys) + 1;
  const trailShapes = [];
  for (let i = 1; i < points.length; i += 1) {
    const from = points[i - 1];
    const to = points[i];
    trailShapes.push({
      kind: "line",
      x1: from.x,
      y1: from.y,
      x2: to.x,
      y2: to.y,
      stroke: "rgba(59, 130, 246, 0.45)",
      width: 0.01,
      group_id: "output.point.trail",
    });
  }
  for (let i = 0; i < Math.max(0, points.length - 1); i += 1) {
    const trailPoint = points[i];
    trailShapes.push({
      kind: "circle",
      x: trailPoint.x,
      y: trailPoint.y,
      r: 0.03,
      fill: "rgba(96, 165, 250, 0.28)",
      stroke: "rgba(59, 130, 246, 0.45)",
      width: 0.01,
      group_id: "output.point.history",
    });
  }
  return {
    schema: "seamgrim.space2d.v0",
    meta: {
      title: "output-rows-fallback",
      source: "observation_output_rows",
    },
    camera: {
      x_min: xMin,
      x_max: xMax,
      y_min: yMin,
      y_max: yMax,
    },
    points,
    shapes: [
      { kind: "line", x1: xMin, y1: 0, x2: xMax, y2: 0, stroke: "#4b5563", width: 0.01, group_id: "output.axis.x" },
      { kind: "line", x1: 0, y1: yMin, x2: 0, y2: yMax, stroke: "#374151", width: 0.01, group_id: "output.axis.y" },
      ...trailShapes,
      { kind: "circle", x: px, y: py, r: 0.07, fill: "#22c55e", stroke: "#16a34a", width: 0.02, group_id: "output.point.focus" },
    ],
  };
}

export function synthesizeDefaultGridSpace2d() {
  return {
    schema: "seamgrim.space2d.v0",
    meta: {
      title: "default-grid-fallback",
      source: "fallback",
    },
    camera: {
      x_min: -1,
      x_max: 1,
      y_min: -1,
      y_max: 1,
    },
    points: [{ x: 0, y: 0 }],
    shapes: [
      { kind: "line", x1: -1, y1: 0, x2: 1, y2: 0, stroke: "#4b5563", width: 0.01, group_id: "fallback.axis.x" },
      { kind: "line", x1: 0, y1: -1, x2: 0, y2: 1, stroke: "#374151", width: 0.01, group_id: "fallback.axis.y" },
    ],
  };
}

function formatRecentTimeLabel(isoText) {
  const ms = Date.parse(String(isoText ?? ""));
  if (!Number.isFinite(ms)) return "";
  const d = new Date(ms);
  const month = String(d.getMonth() + 1);
  const day = String(d.getDate());
  const hour = String(d.getHours()).padStart(2, "0");
  const minute = String(d.getMinutes()).padStart(2, "0");
  return `${month}/${day} ${hour}:${minute}`;
}

function normalizeRunLaunchKind(raw) {
  const kind = String(raw ?? "").trim().toLowerCase();
  if (!kind) return "manual";
  if (kind === "manual") return "manual";
  if (kind === "browse_select") return "browse_select";
  if (kind === "browse_select_student") return "browse_select_student";
  if (kind === "browse_select_teacher") return "browse_select_teacher";
  if (kind === "editor_run") return "editor_run";
  if (kind === "featured_seed_quick") return "featured_seed_quick";
  if (kind === "local_package_import") return "local_package_import";
  return "manual";
}

function normalizeRunOnboardingProfile(raw) {
  const value = String(raw ?? "").trim().toLowerCase();
  if (value === "student") return "student";
  if (value === "teacher") return "teacher";
  return "";
}

function formatRunLaunchKindLabel(raw) {
  const kind = normalizeRunLaunchKind(raw);
  if (kind === "browse_select_student" || kind === "browse_select_teacher" || kind === "browse_select") return "탐색 선택";
  if (kind === "editor_run") return "편집 실행";
  if (kind === "featured_seed_quick") return "Alt+6";
  if (kind === "local_package_import") return "배포 열기";
  return "수동";
}

function formatRunOnboardingProfileLabel(raw) {
  const profile = normalizeRunOnboardingProfile(raw);
  if (profile === "student") return "학생 시작";
  if (profile === "teacher") return "교사 시작";
  return "기본 시작";
}

function formatRunLayoutModeLabel(raw) {
  const mode = String(raw ?? "").trim() || "split";
  if (mode === "split" || mode === "horizontal") return "화면: 나눠 보기";
  if (mode === "vertical") return "화면: 위아래 보기";
  if (mode === "full") return "화면: 크게 보기";
  return `화면: ${mode}`;
}

function formatRunViewFamilyLabel(row) {
  if (row === "graph") return "그래프";
  if (row === "table") return "표";
  if (row === "space2d") return "그림";
  if (row === "text") return "설명";
  return row;
}

function formatRunRequiredViewsLabel(values = []) {
  const rows = normalizeRunRequiredViews(values);
  const labels = rows.map(formatRunViewFamilyLabel);
  return labels.length ? `결과 확인: ${labels.join(", ")}` : "결과 확인: 실행 화면";
}

function formatRunResultCheckLabel(values = []) {
  const rows = normalizeRunRequiredViews(values);
  const labels = rows.map(formatRunViewFamilyLabel);
  return labels.length ? `결과 확인: ${labels.join(", ")}` : "결과 확인: 실행 화면";
}

function buildRunSummaryText(pref) {
  if (!pref || typeof pref !== "object") return "최근 실행 기록 없음";
  const kind = String(pref.lastRunKind ?? "").trim();
  const channels = Math.max(0, Number.isFinite(Number(pref.lastRunChannels)) ? Math.trunc(Number(pref.lastRunChannels)) : 0);
  const timeLabel = formatRecentTimeLabel(pref.lastRunAt);
  const hash = String(pref.lastRunHash ?? "").trim();
  const shortHash = hash && hash !== "-" ? hash.slice(0, 12) : "";
  let label = "";
  if (kind === "space2d") {
    label = `최근 실행 · 그림 출력 ${channels}개`;
  } else if (kind === "obs_only") {
    label = `최근 실행 · 기록 ${channels}개`;
  } else if (kind === "empty") {
    label = "최근 실행 · 출력 없음";
  } else if (kind === "error") {
    label = "최근 실행 · 실패";
  } else {
    label = "최근 실행 기록 없음";
  }
  if (shortHash) {
    label = `${label} · 기록ID:${shortHash}`;
  }
  return timeLabel ? `${label} · ${timeLabel}` : label;
}

function extractRuntimeDerived(stateJson) {
  if (!stateJson) return null;
  const outputLog = extractObservationOutputLogFromState(stateJson);
  const outputLines = extractObservationOutputLinesFromState(stateJson);
  const outputRows = normalizeObserveOutputRows(extractObservationOutputRowsFromState(stateJson));
  const strictViews = extractStructuredViewsFromState(stateJson, {
    preferPatch: false,
    allowObservationOutputFallback: false,
  });
  const views = mergeRuntimeViewsWithObservationOutputFallback(stateJson, strictViews);
  return {
    observation: extractObservationChannelsFromState(stateJson),
    outputLog,
    outputLines,
    outputRows,
    views,
    graphSource: String(views?.graphSource ?? "").trim() || null,
  };
}

export function mergeRuntimeViewsWithObservationOutputFallback(stateJson, views = null) {
  const baseViews = views && typeof views === "object" ? { ...views } : {};
  const hasBaseSpace2d = hasSpace2dDrawable(baseViews?.space2d);
  const hasBaseText = Boolean(baseViews?.text && typeof baseViews.text === "object");
  if (hasBaseSpace2d && hasBaseText) {
    return baseViews;
  }

  const fallbackViews = extractStructuredViewsFromState(stateJson, {
    preferPatch: false,
    allowObservationOutputFallback: true,
  });
  let changed = false;

  if (!hasBaseSpace2d && hasSpace2dDrawable(fallbackViews?.space2d)) {
    baseViews.space2d = fallbackViews.space2d;
    baseViews.space2dRaw = fallbackViews.space2dRaw ?? null;
    changed = true;
  }
  if (!hasBaseText && fallbackViews?.text && typeof fallbackViews.text === "object") {
    baseViews.text = fallbackViews.text;
    baseViews.textRaw = fallbackViews.textRaw ?? null;
    changed = true;
  }

  if (!changed) {
    return baseViews;
  }

  baseViews.viewStack = resolveStructuredViewStackFromState(baseViews);
  baseViews.families = Array.isArray(baseViews.viewStack?.families) ? baseViews.viewStack.families : [];
  return baseViews;
}

function normalizeServerViewObject(rawView, { source = "api_run" } = {}) {
  if (!rawView || typeof rawView !== "object") return null;
  const view = { ...rawView };
  const meta = toPlainObject(view.meta, {});
  if (!String(meta.source ?? "").trim()) {
    meta.source = source;
  }
  view.meta = meta;
  return view;
}

function inferServerViewFamilies(runtimeViews, hintedFamilies = []) {
  const hinted = normalizeViewFamilyList(hintedFamilies);
  if (hinted.length) return hinted;
  const out = [];
  if (runtimeViews?.graph && typeof runtimeViews.graph === "object") out.push("graph");
  if (runtimeViews?.space2d && typeof runtimeViews.space2d === "object") out.push("space2d");
  if (runtimeViews?.table && typeof runtimeViews.table === "object") out.push("table");
  if (runtimeViews?.text && typeof runtimeViews.text === "object") out.push("text");
  if (runtimeViews?.structure && typeof runtimeViews.structure === "object") out.push("structure");
  return normalizeViewFamilyList(out);
}

function normalizeServerViewContract(rawContract, runtimeViews) {
  const contract = rawContract && typeof rawContract === "object" ? rawContract : {};
  const families = inferServerViewFamilies(runtimeViews, contract.families ?? []);
  const source = String(contract.source ?? "").trim() || "api_run";
  const schema = String(contract.schema ?? "").trim() || "seamgrim.view_contract.v1";
  const byFamilyRaw = contract.by_family && typeof contract.by_family === "object" ? contract.by_family : {};
  const byFamily = {};
  families.forEach((family) => {
    const row = byFamilyRaw[family] && typeof byFamilyRaw[family] === "object" ? byFamilyRaw[family] : {};
    const runtimeView = runtimeViews?.[family] && typeof runtimeViews[family] === "object" ? runtimeViews[family] : null;
    byFamily[family] = {
      schema: String(row.schema ?? runtimeView?.schema ?? "").trim(),
      source: String(row.source ?? runtimeView?.meta?.source ?? source).trim() || source,
      available: runtimeView ? true : Boolean(row.available),
    };
  });
  return {
    schema,
    source,
    families,
    by_family: byFamily,
  };
}

function isNonStrictViewSource(rawSource) {
  const source = String(rawSource ?? "").trim().toLowerCase();
  if (!source) return false;
  return source === "observation_output" || source === "observation-output-lines" || source === "observation-fallback";
}

function resolveRuntimeViewSourceStrictness(runtimeDerived = null) {
  const views = runtimeDerived?.views && typeof runtimeDerived.views === "object" ? runtimeDerived.views : {};
  const contract = views.contract && typeof views.contract === "object" ? views.contract : {};
  const byFamily = contract.by_family && typeof contract.by_family === "object" ? contract.by_family : {};
  const nonStrict = [];
  Object.entries(byFamily).forEach(([family, row]) => {
    const data = row && typeof row === "object" ? row : {};
    const available = Boolean(data.available);
    if (!available) return;
    if (isNonStrictViewSource(data.source)) {
      nonStrict.push(String(family).trim());
    }
  });
  const graphSource = String(runtimeDerived?.graphSource ?? "").trim();
  if (graphSource && isNonStrictViewSource(graphSource) && !nonStrict.includes("graph")) {
    nonStrict.push("graph");
  }
  const nonStrictFamilies = [...new Set(nonStrict.filter(Boolean))];
  return {
    strict: nonStrictFamilies.length === 0,
    nonStrictFamilies,
  };
}

function inferRuntimeFamilyList(views = {}) {
  const hinted = normalizeViewFamilyList(views?.families ?? []);
  if (hinted.length) return hinted;
  const out = [];
  if (views?.graph && typeof views.graph === "object") out.push("graph");
  if (views?.space2d && typeof views.space2d === "object") out.push("space2d");
  if (views?.table && typeof views.table === "object") out.push("table");
  if (views?.text && typeof views.text === "object") out.push("text");
  if (views?.structure && typeof views.structure === "object") out.push("structure");
  return normalizeViewFamilyList(out);
}

function readObserveFamilyRows(runtimeDerived = null) {
  const views = runtimeDerived?.views && typeof runtimeDerived.views === "object" ? runtimeDerived.views : {};
  const contract = views.contract && typeof views.contract === "object" ? views.contract : {};
  const byFamily = contract.by_family && typeof contract.by_family === "object" ? contract.by_family : {};
  const families = inferRuntimeFamilyList(views);
  return families.map((family) => {
    const row = byFamily[family] && typeof byFamily[family] === "object" ? byFamily[family] : {};
    const runtimeView = views[family] && typeof views[family] === "object" ? views[family] : null;
    const available = runtimeView ? true : Boolean(row.available);
    const source = String(row.source ?? runtimeView?.meta?.source ?? runtimeDerived?.graphSource ?? "").trim();
    const strict = available ? !isNonStrictViewSource(source) : true;
    return {
      family,
      label: formatObserveFamilyName(family),
      available,
      source: source || "unknown",
      strict,
    };
  });
}

function summarizeObserveGraphMetric(runtimeView) {
  const series = Array.isArray(runtimeView?.series) ? runtimeView.series : [];
  const pointCount = series.reduce((acc, row) => {
    const points = Array.isArray(row?.points) ? row.points.length : 0;
    return acc + points;
  }, 0);
  return `${series.length}개 계열 · ${pointCount}개 점`;
}

function summarizeObserveTableMetric(runtimeView) {
  const normalized = normalizeRuntimeTableView(runtimeView, { maxRows: 24 });
  if (!normalized) return "표 출력 없음";
  return `${normalized.columns.length}열 · ${normalized.rowCount}행`;
}

function summarizeObserveTextMetric(runtimeView) {
  const markdown = readOverlayMarkdownFromViewText(runtimeView);
  if (!markdown) return "설명 출력 없음";
  const lines = markdown
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  const chars = markdown.replace(/\s+/g, "").length;
  return `${lines.length || 1}줄 · ${chars}자`;
}

function summarizeObserveStructureMetric(runtimeView) {
  const nodes = Array.isArray(runtimeView?.nodes) ? runtimeView.nodes.length : 0;
  const links = Array.isArray(runtimeView?.links)
    ? runtimeView.links.length
    : Array.isArray(runtimeView?.edges)
      ? runtimeView.edges.length
      : 0;
  if (!nodes && !links) return "구조 출력 없음";
  return `${nodes}개 노드 · ${links}개 링크`;
}

function summarizeObserveSpace2dMetric(runtimeView) {
  const points = Array.isArray(runtimeView?.points) ? runtimeView.points.length : 0;
  const shapes = Array.isArray(runtimeView?.shapes) ? runtimeView.shapes.length : 0;
  const drawlist = Array.isArray(runtimeView?.drawlist) ? runtimeView.drawlist.length : 0;
  const total = points + shapes + drawlist;
  if (!total) return "그림 출력 없음";
  return `${total}개 요소 (점 ${points} · 도형 ${shapes} · 그림목록 ${drawlist})`;
}

export function normalizeObserveOutputRows(rawRows = []) {
  if (!Array.isArray(rawRows)) return [];
  return rawRows
    .filter((row) => row && typeof row === "object")
    .map((row) => ({
      key: String(row.key ?? "").trim(),
      value: String(row.value ?? "").trim(),
      source: String(row.source ?? "").trim().toLowerCase(),
      syntheticKey: row.syntheticKey === true,
    }))
    .filter((row) => row.key.length > 0);
}

function classifyObserveOutputRows(outputRows = []) {
  const rows = normalizeObserveOutputRows(outputRows);
  if (!rows.length) {
    return {
      rows,
      hasRows: false,
      hasTableRows: false,
      hasObservationRows: false,
      hasFallbackRows: false,
      syntheticFallbackOnly: false,
      consolePreferred: false,
    };
  }
  const hasTableRows = rows.some((row) => row.source === "table.row");
  const hasObservationRows = rows.some((row) => row.source === "observation");
  const hasFallbackRows = rows.some((row) => row.source === "fallback-line");
  const syntheticFallbackOnly = hasFallbackRows && rows.every((row) => row.source === "fallback-line" && row.syntheticKey);
  return {
    rows,
    hasRows: true,
    hasTableRows,
    hasObservationRows,
    hasFallbackRows,
    syntheticFallbackOnly,
    consolePreferred: !hasTableRows,
  };
}

function normalizeConsoleOutputLines(rawLines = []) {
  if (!Array.isArray(rawLines)) return [];
  const structuredOnlyMarkers = new Set([
    "table.row",
    "space2d",
    "space2d.shape",
    "space2d_shape",
    "shape2d",
    "text.overlay",
    "overlay.text",
    "subtitle",
    "자막",
  ]);
  return rawLines
    .map((line) => String(line ?? "").trim())
    .filter((line) => line.length > 0 && !structuredOnlyMarkers.has(line.toLowerCase()));
}

function normalizeConsoleOutputLog(rawLog = [], { fallbackTick = 0 } = {}) {
  if (!Array.isArray(rawLog)) return [];
  const tickFallback = Number.isFinite(Number(fallbackTick)) ? Math.max(0, Math.trunc(Number(fallbackTick))) : 0;
  return rawLog
    .filter((entry) => entry && typeof entry === "object")
    .map((entry, index) => {
      const tick = Number(entry.tick ?? entry.madi ?? entry.tick_id);
      const lineNo = Number(entry.line_no ?? entry.lineNo ?? entry.index ?? entry.order);
      const text = String(entry.text ?? entry.value ?? entry.message ?? "").trim();
      const kind = String(entry.kind ?? entry.level ?? "output").trim().toLowerCase();
      return {
        tick: Number.isFinite(tick) ? Math.max(0, Math.trunc(tick)) : tickFallback,
        line_no: Number.isFinite(lineNo) && lineNo > 0 ? Math.trunc(lineNo) : index + 1,
        text,
        kind: ["warn", "error", "output"].includes(kind) ? kind : "output",
      };
    })
    .filter((entry) => entry.text.length > 0);
}

function isConsoleLogNumericText(text) {
  const token = String(text ?? "").trim();
  if (!token) return false;
  return Number.isFinite(Number(token));
}

function groupConsoleOutputLogEntries(outputLog = []) {
  const rows = normalizeConsoleOutputLog(outputLog);
  if (!rows.length) return [];
  const groups = [];
  let current = null;
  rows.forEach((row) => {
    const tick = Number.isFinite(Number(row.tick)) ? Math.max(0, Math.trunc(Number(row.tick))) : 0;
    const kind = String(row.kind ?? "output").trim().toLowerCase() || "output";
    if (!current || current.tick !== tick || current.kind !== kind) {
      current = { tick, kind, entries: [] };
      groups.push(current);
    }
    const lineNo = current.entries.length + 1;
    current.entries.push({
      tick,
      line_no: lineNo,
      text: String(row.text ?? "").trim(),
      kind,
      valueType: isConsoleLogNumericText(row.text) ? "number" : "string",
    });
  });
  return groups.filter((group) => Array.isArray(group.entries) && group.entries.length > 0);
}

function buildConsoleLogGroupTitle(group) {
  const tick = Number.isFinite(Number(group?.tick)) ? Math.max(0, Math.trunc(Number(group.tick))) : 0;
  const kind = String(group?.kind ?? "output").trim().toLowerCase();
  if (kind === "warn") return `[${tick}단계] 경고`;
  if (kind === "error") return `[${tick}단계] 오류`;
  return `[${tick}단계] 출력`;
}

const CONSOLE_RICH_COLOR_MAP = Object.freeze({
  빨강: "#dc2626",
  파랑: "#2563eb",
  초록: "#16a34a",
  노랑: "#ca8a04",
  검정: "#111827",
  흰색: "#ffffff",
});

function readConsoleRichBraced(text, marker) {
  const source = String(text ?? "");
  const prefix = `\\${marker}{`;
  if (!source.startsWith(prefix)) return null;
  const closeIdx = source.indexOf("}", prefix.length);
  if (closeIdx < 0) return null;
  return {
    raw: source.slice(0, closeIdx + 1),
    value: source.slice(prefix.length, closeIdx).trim(),
    length: closeIdx + 1,
  };
}

function consoleRichStyleText(style = {}) {
  const parts = [];
  if (style.reverse) {
    parts.push("color:var(--surface,#ffffff)");
    parts.push("background:var(--text,#111827)");
  } else {
    if (style.color) parts.push(`color:${style.color}`);
    if (style.background) parts.push(`background:${style.background}`);
  }
  if (style.bold) parts.push("font-weight:700");
  return parts.join(";");
}

function renderConsoleRichSegment(text, style = {}) {
  if (!text) return "";
  const styleText = consoleRichStyleText(style);
  if (!styleText) return escapeHtml(text);
  return `<span class="run-console-rich-span" style="${escapeHtml(styleText)}">${escapeHtml(text)}</span>`;
}

export function parseConsoleRichMarkup(text) {
  const raw = String(text ?? "");
  let idx = 0;
  let buf = "";
  let plainText = "";
  let html = "";
  let rich = false;
  const warnings = [];
  let style = { color: "", background: "", reverse: false, bold: false };
  const flush = () => {
    if (!buf) return;
    plainText += buf;
    html += renderConsoleRichSegment(buf, style);
    buf = "";
  };
  while (idx < raw.length) {
    const rest = raw.slice(idx);
    const color = readConsoleRichBraced(rest, "색");
    if (color) {
      const mapped = CONSOLE_RICH_COLOR_MAP[color.value];
      if (!mapped) {
        warnings.push(`unknown_color:${color.value}`);
        buf += color.raw;
      } else {
        flush();
        style = { ...style, color: mapped };
        rich = true;
      }
      idx += color.length;
      continue;
    }
    const background = readConsoleRichBraced(rest, "배경");
    if (background) {
      const mapped = CONSOLE_RICH_COLOR_MAP[background.value];
      if (!mapped) {
        warnings.push(`unknown_background:${background.value}`);
        buf += background.raw;
      } else {
        flush();
        style = { ...style, background: mapped };
        rich = true;
      }
      idx += background.length;
      continue;
    }
    if (rest.startsWith("\\반전끝")) {
      flush();
      style = { ...style, reverse: false };
      rich = true;
      idx += "\\반전끝".length;
      continue;
    }
    if (rest.startsWith("\\반전")) {
      flush();
      style = { ...style, reverse: true };
      rich = true;
      idx += "\\반전".length;
      continue;
    }
    if (rest.startsWith("\\굵게끝")) {
      flush();
      style = { ...style, bold: false };
      rich = true;
      idx += "\\굵게끝".length;
      continue;
    }
    if (rest.startsWith("\\굵게")) {
      flush();
      style = { ...style, bold: true };
      rich = true;
      idx += "\\굵게".length;
      continue;
    }
    if (rest.startsWith("\\되돌림")) {
      flush();
      style = { color: "", background: "", reverse: false, bold: false };
      rich = true;
      idx += "\\되돌림".length;
      continue;
    }
    buf += raw[idx];
    idx += 1;
  }
  flush();
  return { plainText, html, rich, warnings };
}

function stripConsoleRichMarkup(text) {
  return parseConsoleRichMarkup(text).plainText;
}

function buildConsoleLogHtml(outputLog = [], { emptyText = "", noteText = "", showGroupTitles = true, showLineNumbers = true } = {}) {
  const groups = groupConsoleOutputLogEntries(outputLog);
  const noteHtml = String(noteText ?? "").trim()
    ? `<div class="run-observe-console-note">${escapeRuntimeTableHtml(noteText)}</div>`
    : "";
  if (!groups.length) {
    return {
      html: emptyText
        ? `${noteHtml}<div class="runtime-table-empty">${escapeRuntimeTableHtml(emptyText)}</div>`
        : noteHtml,
      groupCount: 0,
      lineCount: 0,
    };
  }
  const groupsHtml = groups
    .map((group) => {
      const kind = String(group.kind ?? "output").trim().toLowerCase();
      const rowsHtml = group.entries
        .map((entry) => {
          const rich = parseConsoleRichMarkup(entry.text);
          const warningCount = Array.isArray(rich.warnings) ? rich.warnings.length : 0;
          const warningTitle = warningCount > 0 ? ` title="${escapeHtml(rich.warnings.join(", "))}"` : "";
          return `
            <div class="run-main-console-line" data-kind="${escapeHtml(kind)}">
              ${showLineNumbers ? `<span class="run-main-console-lineno">${entry.line_no}</span>` : ""}
              <code class="run-main-console-code" data-value-type="${escapeHtml(String(entry.valueType ?? "string"))}" data-rich="${rich.rich ? "1" : "0"}" data-rich-warning-count="${warningCount}"${warningTitle}>${rich.html}</code>
            </div>
          `;
        })
        .join("");
      return `
        <section class="run-main-console-group" data-kind="${escapeHtml(kind)}">
          ${showGroupTitles ? `<h4 class="run-main-console-group-title">${escapeHtml(buildConsoleLogGroupTitle(group))}</h4>` : ""}
          <div class="run-main-console-lines">${rowsHtml}</div>
        </section>
      `;
    })
    .join("");
  return {
    html: `${noteHtml}<div class="run-main-console-log">${groupsHtml}</div>`,
    groupCount: groups.length,
    lineCount: groups.reduce((acc, group) => acc + group.entries.length, 0),
  };
}

function formatObserveOutputRowForConsole(row, { valueOnly = false } = {}) {
  const value = String(row?.value ?? "").trim();
  if (valueOnly && value) return value;
  const key = String(row?.key ?? "").trim();
  if (value && key && row?.syntheticKey !== true) return `${key}=${value}`;
  if (value) return value;
  return String(row?.key ?? "").trim();
}

function buildConsoleLogFromOutputRows(outputRows = [], { valueOnly = false } = {}) {
  const rows = normalizeObserveOutputRows(outputRows);
  return rows
    .filter((row) => row.value.length > 0)
    .map((row, index) => ({
      tick: 0,
      line_no: index + 1,
      text: formatObserveOutputRowForConsole(row, { valueOnly }),
      kind: "output",
    }));
}

function clipObserveOutputValue(text, maxLen = 18) {
  const raw = String(text ?? "");
  if (raw.length <= maxLen) return raw;
  return `${raw.slice(0, Math.max(1, maxLen - 1))}…`;
}

export function summarizeObserveOutputRows(outputRows = []) {
  const rows = normalizeObserveOutputRows(outputRows);
  if (!rows.length) return "";
  const last = rows[rows.length - 1];
  const key = clipObserveOutputValue(formatDisplayLabel(last.key), 12);
  const value = clipObserveOutputValue(last.value, 18);
  return `${rows.length}행 · 최근 ${key}=${value}`;
}

export function buildObserveOutputRowsPreview(outputRows = [], { maxRows = 3 } = {}) {
  const rows = normalizeObserveOutputRows(outputRows);
  if (!rows.length) return "";
  const limit = Math.max(1, Math.trunc(Number(maxRows) || 3));
  return rows
    .slice(-limit)
    .map((row) => `${clipObserveOutputValue(formatDisplayLabel(row.key), 10)}=${clipObserveOutputValue(row.value, 12)}`)
    .join(" · ");
}

function buildObserveOutputRowsTableHtml(outputRows = [], { maxRows = 24, outputLog = [] } = {}) {
  const outputMeta = classifyObserveOutputRows(outputRows);
  const rows = outputMeta.rows;
  if (!rows.length) {
    return {
      html: '<div class="runtime-table-empty">관찰 출력행 없음</div>',
      rowCount: 0,
      shownRowCount: 0,
      truncated: false,
      mode: "empty",
    };
  }

  const limit = Math.max(1, Math.trunc(Number(maxRows) || 24));
  const shownRows = rows.slice(-limit);
  if (outputMeta.consolePreferred) {
    const noteText = outputMeta.syntheticFallbackOnly
      ? "결과표 출력 없음, 기본 출력으로 표시합니다."
      : "보임표 행 출력 대신 현재 관찰값을 기본 출력으로 보여줍니다.";
    const normalizedOutputLog = normalizeConsoleOutputLog(outputLog);
    const fallbackConsoleLog = outputMeta.syntheticFallbackOnly && normalizedOutputLog.length > 0
      ? normalizedOutputLog
      : buildConsoleLogFromOutputRows(shownRows);
    const log = buildConsoleLogHtml(fallbackConsoleLog, {
      noteText,
      emptyText: "출력 없음",
    });
    return {
      html: `<div class="run-observe-console-fallback">${log.html}</div>`,
      rowCount: rows.length,
      shownRowCount: rows.length,
      truncated: false,
      mode: "console",
    };
  }
  const bodyHtml = shownRows
    .map(
      (row) => {
        const label = formatDisplayLabel(row.key);
        const title = label !== row.key ? ` title="${escapeRuntimeTableHtml(row.key)}"` : "";
        return `<tr><td${title}>${buildRuntimeTableCellHtml(label)}</td><td>${buildRuntimeTableCellHtml(row.value)}</td></tr>`;
      },
    )
    .join("");
  const moreText =
    rows.length > shownRows.length
      ? `<div class="runtime-table-empty">+${escapeRuntimeTableHtml(rows.length - shownRows.length)}행 더 있음</div>`
      : "";
  return {
    html: `<table class="runtime-table run-observe-output-table"><thead><tr><th>키</th><th>값</th></tr></thead><tbody>${bodyHtml}</tbody></table>${moreText}`,
    rowCount: rows.length,
    shownRowCount: shownRows.length,
    truncated: rows.length > shownRows.length,
    mode: "table",
  };
}

function renderObserveOutputRowsTable(container, outputRows = [], { maxRows = 24, outputLog = [] } = {}) {
  const rows = normalizeObserveOutputRows(outputRows);
  if (!container || typeof container !== "object") {
    return {
      rowCount: rows.length,
      shownRowCount: 0,
      truncated: false,
    };
  }
  const table = buildObserveOutputRowsTableHtml(rows, { maxRows, outputLog });
  if ("innerHTML" in container) {
    container.innerHTML = table.html;
  } else if ("textContent" in container) {
    const shownRows = rows.slice(-Math.max(1, Math.trunc(Number(maxRows) || 24)));
    container.textContent = shownRows.map((row) => `${formatDisplayLabel(row.key)}=${row.value}`).join("\n");
  }
  return table;
}

function summarizeObserveFamilyMetric(family, runtimeView) {
  return summarizeObserveFamilyMetricContract(family, runtimeView, {
    graph: summarizeObserveGraphMetric,
    table: summarizeObserveTableMetric,
    text: summarizeObserveTextMetric,
    structure: summarizeObserveStructureMetric,
    space2d: summarizeObserveSpace2dMetric,
  });
}

function buildObservationFromGraph(graph) {
  const seriesList = Array.isArray(graph?.series) ? graph.series : [];
  if (!seriesList.length) return null;
  const first = seriesList.find((series) => Array.isArray(series?.points) && series.points.length > 0) ?? null;
  if (!first) return null;
  const points = Array.isArray(first.points) ? first.points : [];
  const last = points[points.length - 1];
  const x = finiteNumber(last?.x);
  const y = finiteNumber(last?.y);
  if (x === null || y === null) return null;
  const yKey = String(first?.id ?? first?.label ?? "y").trim() || "y";
  return {
    channels: [
      { key: "x", dtype: "number", role: "state" },
      { key: yKey, dtype: "number", role: "state" },
    ],
    row: [x, y],
    values: {
      x,
      [yKey]: y,
    },
    all_values: {
      x,
      [yKey]: y,
    },
  };
}

function clampRuntimeTableCellChars(value) {
  const numeric = Math.max(1, Math.trunc(Number(value) || 0));
  return Math.max(
    RUNTIME_TABLE_CELL_MIN_CHARS,
    Math.min(RUNTIME_TABLE_CELL_MAX_CHARS_LIMIT, numeric),
  );
}

function escapeRuntimeTableHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function truncateRuntimeTableCellText(value, { maxChars = 24 } = {}) {
  const text = String(value ?? "");
  const limit = clampRuntimeTableCellChars(Number(maxChars) || RUNTIME_TABLE_CELL_MAX_CHARS);
  if (text.length <= limit) {
    return { text, truncated: false };
  }
  return {
    text: `${text.slice(0, Math.max(1, limit - 1))}…`,
    truncated: true,
  };
}

export function resolveRuntimeTableCellMaxChars(container, { maxChars = null } = {}) {
  const direct = Number(maxChars);
  if (Number.isFinite(direct) && direct > 0) {
    return clampRuntimeTableCellChars(direct);
  }

  const datasetValue = Number(container?.dataset?.cellMaxChars ?? "");
  if (Number.isFinite(datasetValue) && datasetValue > 0) {
    return clampRuntimeTableCellChars(datasetValue);
  }

  const width = Number(container?.clientWidth ?? container?.offsetWidth ?? 0);
  if (Number.isFinite(width) && width > 0) {
    const estimated = Math.floor(
      Math.max(0, width - RUNTIME_TABLE_CELL_WIDTH_PADDING_PX) / RUNTIME_TABLE_CELL_ESTIMATED_PX_PER_CHAR,
    );
    if (estimated > 0) {
      return clampRuntimeTableCellChars(estimated);
    }
  }

  return RUNTIME_TABLE_CELL_MAX_CHARS;
}

function buildRuntimeTableCellHtml(value, options = {}) {
  const fullText = String(value ?? "");
  const truncated = truncateRuntimeTableCellText(fullText, {
    maxChars: options?.maxChars ?? RUNTIME_TABLE_CELL_MAX_CHARS,
  });
  const escapedText = escapeRuntimeTableHtml(truncated.text);
  const titleAttr = truncated.truncated ? ` title="${escapeRuntimeTableHtml(fullText)}"` : "";
  return `<span class="runtime-table-celltext"${titleAttr}>${escapedText}</span>`;
}

function readRuntimeTableColumnKey(column, index) {
  const direct = String(column?.key ?? "").trim();
  if (direct) return direct;
  return `col_${index}`;
}

export function normalizeRuntimeTableView(table, { maxRows = 24 } = {}) {
  if (!table || typeof table !== "object") return null;
  const columnsRaw = Array.isArray(table.columns) ? table.columns : [];
  const rowsRaw = Array.isArray(table.rows) ? table.rows : [];
  if (!columnsRaw.length) return null;
  const columns = columnsRaw
    .map((column, index) => {
      const key = readRuntimeTableColumnKey(column, index);
      return {
        key,
        label: formatDisplayLabel(String(column?.label ?? key).trim() || key),
        type: String(column?.type ?? "").trim(),
      };
    })
    .filter((column) => Boolean(column.key));
  if (!columns.length) return null;

  const limitedRows = rowsRaw.slice(0, Math.max(0, Number(maxRows) || 0));
  const rows = limitedRows.map((row, rowIndex) => {
    const rowObj = row && typeof row === "object" ? row : {};
    const cells = columns.map((column, colIndex) => {
      const direct = Object.prototype.hasOwnProperty.call(rowObj, column.key)
        ? rowObj[column.key]
        : Array.isArray(rowObj)
          ? rowObj[colIndex]
          : undefined;
      return formatObservationCellValue(direct, { numericMode: "compact" });
    });
    return {
      key: `row_${rowIndex}`,
      cells,
    };
  });

  return {
    columns,
    rows,
    rowCount: rowsRaw.length,
    shownRowCount: rows.length,
    truncated: rowsRaw.length > rows.length,
    schema: String(table.schema ?? "").trim(),
    source: String(table?.meta?.source ?? "").trim(),
  };
}

function formatRuntimeTableSchemaBadge(schema) {
  const text = String(schema ?? "").trim();
  if (!text) return "";
  if (text.startsWith("seamgrim.")) {
    return text.slice("seamgrim.".length);
  }
  return text;
}

export function summarizeRuntimeTableView(normalized, { emptyText = "" } = {}) {
  if (!normalized || typeof normalized !== "object" || !Array.isArray(normalized.columns) || !normalized.columns.length) {
    return String(emptyText ?? "");
  }
  const columnCount = Math.max(0, normalized.columns.length);
  const rowCount = Math.max(0, Number(normalized.rowCount) || 0);
  const shownRowCount = Math.max(0, Number(normalized.shownRowCount) || 0);
  const schemaBadge = formatRuntimeTableSchemaBadge(normalized.schema);
  const sourceText = String(normalized.source ?? "").trim();
  const sourceBadge = sourceText ? formatSourceLabel(sourceText) : "";
  const suffix = [schemaBadge, sourceBadge].filter(Boolean).join(" · ");
  if (normalized.truncated && rowCount > shownRowCount) {
    return suffix
      ? `${columnCount}열 · ${rowCount}행 중 ${shownRowCount}행 표시 · ${suffix}`
      : `${columnCount}열 · ${rowCount}행 중 ${shownRowCount}행 표시`;
  }
  return suffix ? `${columnCount}열 · ${rowCount}행 · ${suffix}` : `${columnCount}열 · ${rowCount}행`;
}

export function renderRuntimeTable(container, table, options = {}) {
  const {
    maxRows = 24,
    emptyText = "표 출력 없음",
    maxChars = null,
  } = options && typeof options === "object" ? options : {};
  if (!container || typeof container !== "object") return false;
  const normalized = normalizeRuntimeTableView(table, { maxRows });
  const cellMaxChars = resolveRuntimeTableCellMaxChars(container, { maxChars });
  if (!normalized || !normalized.columns.length) {
    if ("innerHTML" in container) {
      container.innerHTML = `<div class="runtime-table-empty">${escapeRuntimeTableHtml(emptyText)}</div>`;
    } else if ("textContent" in container) {
      container.textContent = emptyText;
    }
    return false;
  }

  const head = normalized.columns
    .map((column) => `<th data-col-key="${escapeRuntimeTableHtml(column.key)}">${escapeRuntimeTableHtml(column.label)}</th>`)
    .join("");
  const body = normalized.rows
    .map(
      (row) =>
        `<tr>${row.cells
          .map((cell, index) => `<td data-col-key="${escapeRuntimeTableHtml(normalized.columns[index]?.key ?? "")}">${buildRuntimeTableCellHtml(cell, { maxChars: cellMaxChars })}</td>`)
          .join("")}</tr>`,
    )
    .join("");
  const moreText = normalized.truncated
    ? `<div class="runtime-table-empty">+${escapeRuntimeTableHtml(normalized.rowCount - normalized.rows.length)}행 더 있음</div>`
    : "";
  const html = `<table class="runtime-table"><thead><tr>${head}</tr></thead><tbody>${body}</tbody></table>${moreText}`;
  if ("innerHTML" in container) {
    container.innerHTML = html;
  } else if ("textContent" in container) {
    container.textContent = normalized.rows
      .map((row) => row.cells.join(" | "))
      .join("\n");
  }
  return true;
}

function buildServerPlaybackPlan(graph) {
  const series = Array.isArray(graph?.series) ? graph.series : [];
  const first = series.find((row) => Array.isArray(row?.points) && row.points.length > 1) ?? null;
  if (!first) return null;
  const seriesId = String(first?.id ?? first?.label ?? "y").trim() || "y";
  const frames = [];
  const points = Array.isArray(first.points) ? first.points : [];
  points.forEach((row) => {
    const x = finiteNumber(row?.x);
    const y = finiteNumber(row?.y);
    if (x === null || y === null) return;
    frames.push({ x, y });
  });
  if (frames.length < 2) return null;
  return {
    seriesId,
    frames,
    axis: graph?.axis ?? null,
  };
}

function readOverlayMarkdownFromViewText(textView) {
  if (!textView || typeof textView !== "object") return "";
  const markdown = String(textView.markdown ?? textView.text ?? "").trim();
  return markdown;
}

export function summarizeRuntimeStructureMarkdown(structureView, { sampleLimit = 3 } = {}) {
  return buildStructureSummaryMarkdown(structureView, { sampleLimit });
}

export function summarizeRuntimeGraphMarkdown(graphView, { seriesLimit = 3 } = {}) {
  return buildGraphSummaryMarkdown(graphView, { seriesLimit });
}

function readRuntimeTextMarkdownFromViews(views) {
  const directText = readOverlayMarkdownFromViewText(views?.text);
  if (directText) return directText;
  return summarizeRuntimeStructureMarkdown(views?.structure);
}

export function buildRuntimeStructurePreviewHtml(structureView, { width = 280, height = 164, maxNodes = 8 } = {}) {
  return buildStructurePreviewHtml(structureView, { width, height, maxNodes });
}

export function buildRuntimeGraphPreviewHtml(graphView, { width = 280, height = 164, maxSeries = 3 } = {}) {
  return buildGraphPreviewHtml(graphView, { width, height, maxSeries });
}

function buildMainGraphVisualHtml(graphView) {
  const html = buildRuntimeGraphPreviewHtml(graphView, { width: 960, height: 540, maxSeries: 4 });
  if (String(html ?? "").trim()) return html;
  return "";
}

function buildMainConsoleVisualHtml({ outputRows = [], outputLines = [], outputLog = [], warnings = [], markdown = "" } = {}) {
  const outputMeta = classifyObserveOutputRows(outputRows);
  const normalizedRows = outputMeta.rows;
  const consoleLines = normalizeConsoleOutputLines(outputLines);
  const consoleLog = normalizeConsoleOutputLog(outputLog);
  const consoleLogEntries = !normalizedRows.length
    ? (consoleLog.length > 0
      ? consoleLog
      : consoleLines.map((line, index) => ({ tick: 0, line_no: index + 1, text: line, kind: "output" })))
    : outputMeta.syntheticFallbackOnly
    ? (consoleLog.length > 0
      ? consoleLog
      : consoleLines.length > 0
        ? consoleLines.map((line, index) => ({ tick: 0, line_no: index + 1, text: line, kind: "output" }))
        : buildConsoleLogFromOutputRows(normalizedRows, { valueOnly: true }))
    : buildConsoleLogFromOutputRows(normalizedRows, { valueOnly: true });
  const warningModel = buildConsoleWarningSummary(warnings);
  const table = buildObserveOutputRowsTableHtml(normalizedRows, { maxRows: 12, outputLog: consoleLog });
  const markdownText = String(markdown ?? "").trim();
  const markdownHtml = markdownText ? markdownToHtml(markdownText) : "";
  const hasWarning = Array.isArray(warnings) && warnings.length > 0;
  const hasConsoleLines = consoleLogEntries.length > 0;
  const hasTable = table.rowCount > 0;
  const hasMarkdown = Boolean(markdownHtml);
  if (!hasWarning && !hasTable && !hasMarkdown && !hasConsoleLines) return "";
  const warningHtml = hasWarning
    ? `<div class="run-main-console-banner" data-level="${escapeHtml(String(warningModel.level ?? "warn"))}">${escapeHtml(String(warningModel.text ?? ""))}</div>`
    : "";
  const consoleLinesHtml = !outputMeta.hasTableRows && hasConsoleLines
    ? `
      <section class="run-main-console-card">
        <h3 class="run-main-console-title">기본 출력</h3>
        <div class="run-main-console-body">${buildConsoleLogHtml(consoleLogEntries, { emptyText: "출력 없음", showGroupTitles: false, showLineNumbers: false }).html}</div>
      </section>
    `
    : "";
  const outputHtml = hasTable && outputMeta.hasTableRows
    ? `
      <section class="run-main-console-card">
        <h3 class="run-main-console-title">결과표</h3>
        <div class="run-main-console-body">${table.html}</div>
      </section>
    `
    : "";
  const textHtml = hasMarkdown
    ? `
      <section class="run-main-console-card">
        <h3 class="run-main-console-title">설명</h3>
        <div class="run-main-console-body run-main-console-markdown">${markdownHtml}</div>
      </section>
    `
    : "";
  return `
    <div class="run-main-console-shell">
      ${warningHtml}
      ${consoleLinesHtml}
      ${outputHtml}
      ${textHtml}
    </div>
  `;
}

function readGraphViewSource(views = null) {
  const candidates = [
    views?.graphSource,
    views?.contract?.by_family?.graph?.source,
    views?.graph?.meta?.source,
  ];
  for (const raw of candidates) {
    const source = String(raw ?? "").trim().toLowerCase();
    if (source) return source;
  }
  return "";
}

function lessonRequiresGraphView(requiredViews = []) {
  return normalizeViewFamilyList(requiredViews).includes("graph");
}

function hasConsoleVisualPayload({ outputRows = [], outputLines = [], outputLog = [], warnings = [], markdown = "" } = {}) {
  const outputMeta = classifyObserveOutputRows(outputRows);
  const consoleLines = normalizeConsoleOutputLines(outputLines);
  const consoleLog = normalizeConsoleOutputLog(outputLog);
  const markdownText = String(markdown ?? "").trim();
  return (
    outputMeta.rows.length > 0 ||
    consoleLines.length > 0 ||
    consoleLog.length > 0 ||
    (Array.isArray(warnings) && warnings.length > 0) ||
    Boolean(markdownText)
  );
}

function hasResultPanelPayload({ outputRows = [], outputLines = [], outputLog = [], warnings = [], views = null } = {}) {
  if (hasConsoleVisualPayload({ outputRows, outputLines, outputLog, warnings })) {
    return true;
  }
  return Boolean(views?.table || views?.text || views?.structure);
}

function shouldUseConsoleGridMainVisual({
  observation = null,
  outputRows = [],
  outputLines = [],
  outputLog = [],
  warnings = [],
  views = null,
} = {}) {
  if (readDeclaredBogaeKind(observation) === "console-grid") {
    return true;
  }
  return hasConsoleVisualPayload({
    outputRows,
    outputLines,
    outputLog,
    warnings,
    markdown: readRuntimeTextMarkdownFromViews(views),
  });
}

function hasGraphTabPayload(views = null) {
  return Boolean(views?.graph || views?.table || views?.text || views?.structure);
}

function resolveRunPreferredTab({
  views = null,
  outputRows = [],
  outputLines = [],
  outputLog = [],
  warnings = [],
} = {}) {
  if (hasGraphTabPayload(views)) {
    return SUBPANEL_TAB.GRAPH;
  }
  if (hasResultPanelPayload({ outputRows, outputLines, outputLog, warnings, views })) {
    return SUBPANEL_TAB.OUTPUT;
  }
  return SUBPANEL_TAB.MAEGIM;
}

// observation의 보개종류 채널로 보개 모드를 명시 선언할 수 있다.
// 예: `보개종류 <- "콘솔격자".` 또는 `보개종류 <- "없음".`
function readDeclaredBogaeKind(observation) {
  const allValues = observation?.all_values ?? observation?.values ?? {};
  if (!allValues || typeof allValues !== "object") return null;
  const candidates = ["보개종류", "bogae_kind", "bogaeKind", "view_kind", "viewKind"];
  for (const key of candidates) {
    const val = allValues[key];
    if (val == null) continue;
    const s = String(val).trim().toLowerCase();
    if (["콘솔격자", "console-grid", "consolegrid", "console_grid", "콘솔", "console"].includes(s))
      return "console-grid";
    if (["없음", "none"].includes(s)) return "none";
    if (["공간", "space2d", "space"].includes(s)) return "space2d";
  }
  return null;
}

// 콘솔 격자 캔버스에 넘길 텍스트 줄을 추출한다.
function buildConsoleLinesForGrid({ outputRows = [], outputLog = [], outputLines = [] } = {}) {
  const log = normalizeConsoleOutputLog(outputLog);
  if (log.length > 0) {
    // outputLog에 항목이 있으면 → tick 그룹별 제목 + 값 출력
    const groups = groupConsoleOutputLogEntries(log);
    const lines = [];
    for (const group of groups) {
      lines.push(buildConsoleLogGroupTitle(group));
      for (const entry of group.entries) {
        lines.push(stripConsoleRichMarkup(entry.text));
      }
    }
    return lines;
  }
  const rows = normalizeObserveOutputRows(outputRows);
  if (rows.length > 0) {
    return rows
      .filter((row) => row.value.length > 0)
      .map((row) => stripConsoleRichMarkup(formatObserveOutputRowForConsole(row, { valueOnly: true })));
  }
  const rawLines = normalizeConsoleOutputLines(outputLines);
  return rawLines.map(stripConsoleRichMarkup);
}

export function resolveRunMainVisualMode({
  views = null,
  observation = null,
  outputLog = [],
  outputLines = [],
  outputRows = [],
  warnings = [],
  allowShapeFallback = false,
  prevSpace2d = null,
  lessonRequiredViews = [],
} = {}) {
  // ── 보개종류 명시 선언 ────────────────────────────────────────────────────
  const declaredKind = readDeclaredBogaeKind(observation);
  if (declaredKind === "console-grid") {
    const consoleHtml = buildMainConsoleVisualHtml({ outputRows, outputLines, outputLog, warnings });
    return {
      mode: "console-grid",
      space2d: null,
      graphHtml: "",
      consoleHtml,
      consoleLinesForGrid: buildConsoleLinesForGrid({ outputRows, outputLog, outputLines }),
    };
  }
  if (declaredKind === "none") {
    return { mode: "none", space2d: null, graphHtml: "", consoleHtml: "", consoleLinesForGrid: [] };
  }

  // ── space2d (네이티브) ───────────────────────────────────────────────────
  const nativeSpace2d = views?.space2d ?? null;
  if (hasSpace2dDrawable(nativeSpace2d)) {
    return {
      mode: "space2d",
      space2d: nativeSpace2d,
      graphHtml: "",
      consoleHtml: "",
      consoleLinesForGrid: [],
    };
  }

  // ── terminal/grid2d형 기본 보개 ────────────────────────────────────────
  if (shouldUseConsoleGridMainVisual({
    observation,
    outputRows,
    outputLines,
    outputLog,
    warnings,
    views,
  })) {
    const consoleHtml = buildMainConsoleVisualHtml({
      outputRows,
      outputLines,
      outputLog,
      warnings,
      markdown: readRuntimeTextMarkdownFromViews(views),
    });
    return {
      mode: "console-grid",
      space2d: null,
      graphHtml: "",
      consoleHtml,
      consoleLinesForGrid: buildConsoleLinesForGrid({ outputRows, outputLog, outputLines }),
    };
  }

  // ── allowShapeFallback: 의미있는 space2d 합성 시도 ────────────────────────
  // synthesizeSpace2dFromOutputRows는 제거: x,y 이름의 일반 변수를 2D 좌표로
  // 오인해 단순 계산 DDN에도 debug-fallback이 발화되는 오탐(false-positive) 원인.
  // graph/observation 기반 합성만 허용.
  const syntheticSpace2d = allowShapeFallback
    ? (
      synthesizeSpace2dFromGraph(views?.graph, observation) ??
      synthesizeSpace2dFromObservation(observation)
    )
    : null;
  if (syntheticSpace2d) {
    return {
      mode: "debug-fallback",
      space2d: syntheticSpace2d,
      graphHtml: "",
      consoleHtml: "",
      consoleLinesForGrid: [],
    };
  }

  // ── 기본값: 콘솔 격자 보개 ────────────────────────────────────────────────
  // allowShapeFallback 여부와 무관하게 아무것도 없으면 콘솔 격자를 표시.
  // 이전의 synthesizeDefaultGridSpace2d() (흰 격자) 대신 터미널 격자를 사용한다.
  return {
    mode: "console-grid",
    space2d: null,
    graphHtml: "",
    consoleHtml: buildMainConsoleVisualHtml({ outputRows, outputLines, outputLog, warnings }),
    consoleLinesForGrid: buildConsoleLinesForGrid({ outputRows, outputLog, outputLines }),
  };
}

export function resolveRunPostExecuteTab({
  views = null,
  outputRows = [],
  outputLines = [],
  outputLog = [],
  warnings = [],
} = {}) {
  return resolveRunPreferredTab({
    views,
    outputRows,
    outputLines,
    outputLog,
    warnings,
  });
}

export class RunScreen {
  constructor({
    root,
    wasmState,
    onBack,
    onEditDdn,
    onOpenAdvanced,
    onSelectLesson,
    onGetInspectorContext,
    getOverlaySession,
    getRuntimeSessionV0,
    onOverlaySessionChange,
    onSourceChange,
    onStudioStateChange,
    onSaveDdn,
    onSaveSnapshot,
    onSaveSession,
    allowShapeFallback = false,
    allowServerFallback = false,
  } = {}) {
    this.root = root;
    this.wasmState = wasmState;
    this.onBack = typeof onBack === "function" ? onBack : () => {};
    this.onEditDdn = typeof onEditDdn === "function" ? onEditDdn : () => {};
    this.onOpenAdvanced = typeof onOpenAdvanced === "function" ? onOpenAdvanced : () => {};
    this.onSelectLesson = typeof onSelectLesson === "function" ? onSelectLesson : null;
    this.onGetInspectorContext = typeof onGetInspectorContext === "function" ? onGetInspectorContext : null;
    this.getOverlaySession = typeof getOverlaySession === "function" ? getOverlaySession : null;
    this.getRuntimeSessionV0 = typeof getRuntimeSessionV0 === "function" ? getRuntimeSessionV0 : null;
    this.onOverlaySessionChange = typeof onOverlaySessionChange === "function" ? onOverlaySessionChange : null;
    this.onSourceChange = typeof onSourceChange === "function" ? onSourceChange : null;
    this.onStudioStateChange = typeof onStudioStateChange === "function" ? onStudioStateChange : null;
    this.onSaveDdn = typeof onSaveDdn === "function" ? onSaveDdn : null;
    this.onSaveSnapshot = typeof onSaveSnapshot === "function" ? onSaveSnapshot : null;
    this.onSaveSession = typeof onSaveSession === "function" ? onSaveSession : null;
    this.allowShapeFallback = Boolean(allowShapeFallback);
    this.allowServerFallback = Boolean(allowServerFallback);

    this.lesson = null;
    this.baseDdn = "";
    this.currentDdn = "";
    this.sourceKind = "scratch";
    this.sourceLabel = "새 작업";
    this.engineStatus = "idle";
    this.graphTabMode = "graph";
    this.lastState = null;
    this.lastRuntimeDerived = null;
    this.lastGraphSnapshot = null;
    this.lessonLayoutProfile = resolveRunLayoutProfile([]);

    this.loopActive = false;
    this.screenVisible = false;
    this.loop = null;
    this.viewPanStep = 0.08;
    this.lastRuntimeHash = "-";
    this.lastRuntimeSnapshotKey = "";
    this.lastExecPathHint = "";
    this.lastSpace2dMode = "none";
    this.lastMainVisualMode = "none";
    this.lastRuntimeHintText = "";
    this.observeGuideStatusText = "";
    this.observeGuideStatusExpireAt = 0;
    this.observeGuideStatusTimer = null;
    this.lastPreviewViewModel = null;
    this.lastOverlayMarkdown = "";
    this.lastRuntimeTextMarkdown = "";
    this.lastParseWarnings = [];
    this.renderConsoleWarningSummary();
    this.lastServerFallbackWarning = null;
    this.lastPlatformServerErrorCode = "";
    this.lastPlatformServerActionRail = [];
    this.runtimeTickCounter = 0;
    this.runtimeMaxMadi = 0;
    this.runtimeTimeValue = null;
    this.serverPlayback = null;
    this.activeOverlayRunId = "";
    this.hoverOverlayRunId = "";
    this.soloOverlayRunId = "";
    this.lastRuntimeTableCellMaxChars = 0;
    this.runtimeTableResizeObserver = null;
    this.runtimeTableResizeFallbackInstalled = false;
    this.studioLayoutResizeObserver = null;
    this.studioLayoutResizeFallbackInstalled = false;
    this.activeRunTab = "graph";
    this.primaryView = "sim";
    this.minimalUi = true;
    this.studioViewMode = STUDIO_VIEW_MODE_BASIC;
    this.fullModePanelOpen = false;
    this.dockTarget = "space2d";
    this.playbackPaused = false;
    this.executionPaused = false;
    this.engineMode = RUN_ENGINE_MODE_ONESHOT;
    this.playbackSpeed = 1;
    this.playbackLoop = true;
    this.dockCursorTick = 0;
    this.dockCursorFollowLive = true;
    this.viewPlaybackTimer = null;
    this.lessonOptions = [];
    this.lastInspectorReport = null;
    this.lastInspectorStatusText = "";
    this.heldInputMask = 0;
    this.pulsePressedMask = 0;
    this.lastInputToken = "";
    this.lastLaunchKind = "manual";
    this.lastOnboardingProfile = "";
    this.classroomModeAccess = "default";
    this.overlayRuns = [];
    this.activeOverlayRunId = "";
    this.hoverOverlayRunId = "";
    this.soloOverlayRunId = "";
    this.runManagerSequence = 0;
    this.lastWarningSignature = "";
    this.lastWarningPrimaryActionKind = "";
    this.runAttemptCount = 0;
    this.inlineWarnTimer = null;
    this.inlineWarningModel = null;
    this.runErrorMessage = "";
    this.runErrorDismissed = false;
    this.firstRunSuccessEmitted = false;
    this.pendingRecoveryFromFailure = false;
    this.lastFailureCode = "";
    this.pendingRunRequest = null;
    this.activeRunRequestId = "";
    this.completedRunRequestId = "";
    this.runRequestSequence = 0;
    this.liveReplDebounceMs = 400;
    this.liveReplTimer = null;
    this.liveReplSequence = 0;
    this.boundKeyDownHandler = (event) => {
      this.handleViewHotkeys(event);
      this.handleRuntimeInputKeyDown(event);
    };
    this.boundKeyUpHandler = (event) => {
      this.handleRuntimeInputKeyUp(event);
    };
    this.boundWindowBlurHandler = () => {
      this.clearRuntimeInputState();
    };
    this.boundPlatformServerExchangeHandler = (event) => {
      this.applyPlatformServerAdapterExchange(event?.detail ?? null);
    };
    this.boundRuntimeTableResizeHandler = () => {
      this.refreshRuntimeTableForCurrentWidth();
    };
    this.boundStudioLayoutResizeHandler = () => {
      this.refreshStudioLayoutBounds({ persist: false });
    };
    this.lastRunErrorBannerVisible = false;
    this.lastStudioLayoutBounds = null;
    this.lastBogaeToolbarCompact = false;
    this.uiPrefs = {
      lessons: {},
    };
    this.classroomModeBtns = [];
    this.advancedExportsEnabled = false;
  }

  isSimCorePolicyEnabled() {
    try {
      return Boolean(document?.body?.classList?.contains("policy-sim-core"));
    } catch (_) {
      return false;
    }
  }

  mountAdvancedExportPanelsIfEnabled() {
    if (!shouldEnableAdvancedExports()) return;
    const tools = this.root?.querySelector?.("#run-inspector-tools");
    if (!tools || tools.querySelector("[data-run-publication-prep-export]")) return;
    tools.insertAdjacentHTML("beforeend", ADVANCED_EXPORT_PANEL_HTML);
  }

  init() {
    this.titleEl = this.root.querySelector("#run-lesson-title");
    this.lastSummaryEl = this.root.querySelector("#run-last-summary");
    this.layoutEl = this.root.querySelector(".run-layout");
    this.runLayoutSplitterEl = this.root.querySelector("#run-layout-splitter");
    this.studioSourceLabelEl = this.root.querySelector("#studio-source-label");
    this.runPresetRailEl = this.root.querySelector("[data-run-preset-rail]");
    this.runPresetLaunchKindEl = this.root.querySelector("[data-run-preset-launch-kind]");
    this.runPresetOnboardingEl = this.root.querySelector("[data-run-preset-onboarding]");
    this.runPresetLayoutEl = this.root.querySelector("[data-run-preset-layout]");
    this.runPresetViewsEl = this.root.querySelector("[data-run-preset-views]");
    this.runPresetNumericTrackEl = this.root.querySelector("[data-run-preset-numeric-track]");
    this.runResultNumericLinkEl = this.root.querySelector("[data-run-result-numeric-link]");
    this.runPackageLessonSwitchEl = this.root.querySelector("[data-run-package-lesson-switch]");
    this.runPackageLessonSelectEl = this.root.querySelector("#run-package-lesson-select");
    this.runDeliveryStatusEl = this.root.querySelector("[data-run-delivery-status]");
    this.runDeliveryResultEl = this.root.querySelector("[data-run-delivery-result]");
    this.runDeliveryStudentNameEl = this.root.querySelector("#run-delivery-student-name");
    this.runDeliveryResultCopyBtn = this.root.querySelector("#btn-run-delivery-result-copy");
    this.runDeliveryResultDownloadBtn = this.root.querySelector("#btn-run-delivery-result-download");
    this.runDeliveryInstructionsEl = this.root.querySelector("[data-run-delivery-instructions]");
    this.runLessonBriefEl = this.root.querySelector("[data-run-lesson-brief]");
    this.classroomModeSwitchEl = this.root.querySelector("[data-classroom-mode-switch]");
    this.classroomModeBtns = Array.from(this.root.querySelectorAll("[data-classroom-mode]"));
    this.btnStudioNewEl = this.root.querySelector("#btn-studio-new");
    this.studioInlineWarnEl = this.root.querySelector("#studio-inline-warn");
    this.studioInlineWarnTextEl = this.root.querySelector("#studio-inline-warn-text");
    this.inlineAutofixBtn = this.root.querySelector("#btn-inline-autofix");
    this.runLessonSummaryEl = this.root.querySelector("#run-lesson-summary");
    this.runOnboardingStatusEl = this.root.querySelector("#run-onboarding-status");
    this.runLoadLocalBtn = this.root.querySelector("#btn-ddn-open");
    this.runLoadFileInputEl = this.root.querySelector("#input-ddn-file");
    this.runSaveLocalBtn = this.root.querySelector("#btn-ddn-save");
    this.runLocalSaveStatusEl = this.root.querySelector("#run-local-save-status");
    this.runDdnPreviewEl = this.root.querySelector("#run-ddn-preview");
    this.runLegacyAutofixBtn = this.root.querySelector("#btn-run-legacy-autofix");
    this.runLegacyAutofixStatusEl = this.root.querySelector("#run-legacy-autofix-status");
    this.runInspectorMetaEl = this.root.querySelector("#run-inspector-meta");
    this.runInspectorMetaSummaryEl = this.root.querySelector("#run-inspector-meta-summary");
    this.runInspectorMetaChipsEl = this.root.querySelector("#run-inspector-meta-chips");
    this.runInspectorMetaBodyEl = this.root.querySelector("#run-inspector-meta-body");
    this.runInspectorStatusEl = this.root.querySelector("#run-inspector-status");
    this.mountAdvancedExportPanelsIfEnabled();
    this.runClassroomReportEl = this.root.querySelector("[data-run-classroom-report-export]");
    this.runClassroomReportMetaEl = this.root.querySelector("[data-run-classroom-report-meta]");
    this.runClassroomReportTextEl = this.root.querySelector("[data-run-classroom-report-text]");
    this.runClassroomReportCopyBtn = this.root.querySelector("#btn-run-classroom-report-copy");
    this.runTeacherReportCopyBtn = this.root.querySelector("#btn-run-teacher-report-copy");
    this.runLocalPackageEl = this.root.querySelector("[data-run-local-package-export]");
    this.runLocalPackageMetaEl = this.root.querySelector("[data-run-local-package-meta]");
    this.runLocalPackageCodeEl = this.root.querySelector("[data-run-local-package-code]");
    this.runLocalPackageGuideEl = this.root.querySelector("[data-run-local-package-guide]");
    this.runLocalPackageChecklistEl = this.root.querySelector("[data-run-local-package-checklist]");
    this.runLocalPackageMaterialsEl = this.root.querySelector("[data-run-local-package-materials]");
    this.runLocalPackageTextEl = this.root.querySelector("[data-run-local-package-text]");
    this.runLocalPackageSessionInputEl = this.root.querySelector("#run-local-package-session-input");
    this.runLocalPackageCodeCopyBtn = this.root.querySelector("#btn-run-local-package-code-copy");
    this.runLocalPackageGuideCopyBtn = this.root.querySelector("#btn-run-local-package-guide-copy");
    this.runLocalPackageGuideDownloadBtn = this.root.querySelector("#btn-run-local-package-guide-download");
    this.runLocalPackageChecklistCopyBtn = this.root.querySelector("#btn-run-local-package-checklist-copy");
    this.runLocalPackageChecklistDownloadBtn = this.root.querySelector("#btn-run-local-package-checklist-download");
    this.runLocalPackageCopyBtn = this.root.querySelector("#btn-run-local-package-copy");
    this.runLocalPackageDownloadBtn = this.root.querySelector("#btn-run-local-package-download");
    this.runTeacherPackageCopyBtn = this.root.querySelector("#btn-run-teacher-package-copy");
    this.runTeacherPackageDownloadBtn = this.root.querySelector("#btn-run-teacher-package-download");
    this.runStudentRosterInputEl = this.root.querySelector("#run-student-roster-input");
    this.runStudentRosterFileInputEl = this.root.querySelector("#input-run-student-roster-file");
    this.runStudentRosterFileOpenBtn = this.root.querySelector("#btn-run-student-roster-file-open");
    this.runStudentRosterTemplateCopyBtn = this.root.querySelector("#btn-run-student-roster-template-copy");
    this.runStudentRosterTemplateDownloadBtn = this.root.querySelector("#btn-run-student-roster-template-download");
    this.runStudentResultInputEl = this.root.querySelector("#run-student-result-input");
    this.runStudentResultFileInputEl = this.root.querySelector("#input-run-student-result-file");
    this.runStudentResultFileOpenBtn = this.root.querySelector("#btn-run-student-result-file-open");
    this.runStudentResultReviewBtn = this.root.querySelector("#btn-run-student-result-review");
    this.runStudentRosterOnlyClearBtn = this.root.querySelector("#btn-run-student-roster-only-clear");
    this.runStudentResultOnlyClearBtn = this.root.querySelector("#btn-run-student-result-only-clear");
    this.runStudentResultClearBtn = this.root.querySelector("#btn-run-student-result-clear");
    this.runStudentResultStatusEl = this.root.querySelector("[data-run-student-result-status]");
    this.runStudentResultSummaryEl = this.root.querySelector("[data-run-student-result-summary]");
    this.runStudentResultTableEl = this.root.querySelector("[data-run-student-result-table]");
    this.runStudentResultReportCopyBtn = this.root.querySelector("#btn-run-student-result-report-copy");
    this.runStudentResultReportDownloadBtn = this.root.querySelector("#btn-run-student-result-report-download");
    this.runStudentMissingReminderCopyBtn = this.root.querySelector("#btn-run-student-missing-reminder-copy");
    this.runStudentMissingReminderDownloadBtn = this.root.querySelector("#btn-run-student-missing-reminder-download");
    this.runStudentReviewReminderCopyBtn = this.root.querySelector("#btn-run-student-review-reminder-copy");
    this.runStudentReviewReminderDownloadBtn = this.root.querySelector("#btn-run-student-review-reminder-download");
    this.runPublicationPrepEl = this.root.querySelector("[data-run-publication-prep-export]");
    this.runPublicationPrepMetaEl = this.root.querySelector("[data-run-publication-prep-meta]");
    this.runPublicationPrepTextEl = this.root.querySelector("[data-run-publication-prep-text]");
    this.runPublicationPrepCopyBtn = this.root.querySelector("#btn-run-publication-prep-copy");
    this.runRegistrySeedEl = this.root.querySelector("[data-run-registry-seed-export]");
    this.runRegistrySeedMetaEl = this.root.querySelector("[data-run-registry-seed-meta]");
    this.runRegistrySeedTextEl = this.root.querySelector("[data-run-registry-seed-text]");
    this.runRegistrySeedCopyBtn = this.root.querySelector("#btn-run-registry-seed-copy");
    this.runApprovalContinuityEl = this.root.querySelector("[data-run-approval-continuity-export]");
    this.runApprovalContinuityMetaEl = this.root.querySelector("[data-run-approval-continuity-meta]");
    this.runApprovalContinuityTextEl = this.root.querySelector("[data-run-approval-continuity-text]");
    this.runApprovalContinuityCopyBtn = this.root.querySelector("#btn-run-approval-continuity-copy");
    this.runBenchmarkLtsEl = this.root.querySelector("[data-run-benchmark-lts-export]");
    this.runBenchmarkLtsMetaEl = this.root.querySelector("[data-run-benchmark-lts-meta]");
    this.runBenchmarkLtsTextEl = this.root.querySelector("[data-run-benchmark-lts-text]");
    this.runBenchmarkLtsCopyBtn = this.root.querySelector("#btn-run-benchmark-lts-copy");
    this.runEducationOperationsLtsEl = this.root.querySelector("[data-run-education-operations-lts-export]");
    this.runEducationOperationsLtsMetaEl = this.root.querySelector("[data-run-education-operations-lts-meta]");
    this.runEducationOperationsLtsTextEl = this.root.querySelector("[data-run-education-operations-lts-text]");
    this.runEducationOperationsLtsCopyBtn = this.root.querySelector("#btn-run-education-operations-lts-copy");
    this.syncAdvancedExportVisibility();
    this.runExecUserStatusEl = this.root.querySelector("#run-exec-user-status");
    this.runExecStatusEl = this.root.querySelector("#bogae-status-text");
    this.runClassroomReportCopyBtn?.addEventListener("click", () => {
      void this.handleCopyClassroomReportExport();
    });
    this.runTeacherReportCopyBtn?.addEventListener("click", () => {
      void this.handleCopyClassroomReportExport();
    });
    this.runLocalPackageCopyBtn?.addEventListener("click", () => {
      void this.handleCopyLocalPackageExport();
    });
    this.runLocalPackageDownloadBtn?.addEventListener("click", () => {
      this.handleDownloadLocalPackageExport();
    });
    this.runLocalPackageGuideCopyBtn?.addEventListener("click", () => {
      void this.handleCopyLocalPackageStudentGuide();
    });
    this.runLocalPackageGuideDownloadBtn?.addEventListener("click", () => {
      this.handleDownloadLocalPackageStudentGuide();
    });
    this.runLocalPackageCodeCopyBtn?.addEventListener("click", () => {
      void this.handleCopyLocalPackageCode();
    });
    this.runLocalPackageChecklistCopyBtn?.addEventListener("click", () => {
      void this.handleCopyTeacherPreparationChecklist();
    });
    this.runLocalPackageChecklistDownloadBtn?.addEventListener("click", () => {
      this.handleDownloadTeacherPreparationChecklist();
    });
    this.runLocalPackageSessionInputEl?.addEventListener("input", () => {
      this.syncLocalPackageExport();
    });
    this.runTeacherPackageCopyBtn?.addEventListener("click", () => {
      void this.handleCopyLocalPackageExport();
    });
    this.runTeacherPackageDownloadBtn?.addEventListener("click", () => {
      this.handleDownloadLocalPackageExport();
    });
    this.runStudentRosterFileOpenBtn?.addEventListener("click", () => {
      if (this.runStudentRosterFileInputEl) {
        this.runStudentRosterFileInputEl.value = "";
        this.runStudentRosterFileInputEl.click();
      }
    });
    this.runStudentRosterFileInputEl?.addEventListener("change", (event) => {
      void this.handleLoadStudentRosterFiles(event);
    });
    this.runStudentRosterTemplateCopyBtn?.addEventListener("click", () => {
      void this.handleCopyStudentRosterTemplate();
    });
    this.runStudentRosterTemplateDownloadBtn?.addEventListener("click", () => {
      this.handleDownloadStudentRosterTemplate();
    });
    this.runStudentResultFileOpenBtn?.addEventListener("click", () => {
      if (this.runStudentResultFileInputEl) {
        this.runStudentResultFileInputEl.value = "";
        this.runStudentResultFileInputEl.click();
      }
    });
    this.runStudentResultFileInputEl?.addEventListener("change", (event) => {
      void this.handleLoadStudentResultFiles(event);
    });
    this.runStudentRosterInputEl?.addEventListener("input", () => {
      this.handleStudentResultReviewInputChanged();
    });
    this.runStudentResultInputEl?.addEventListener("input", () => {
      this.handleStudentResultReviewInputChanged();
    });
    this.runStudentResultReviewBtn?.addEventListener("click", () => {
      this.handleReviewStudentResultReturn();
    });
    this.runStudentRosterOnlyClearBtn?.addEventListener("click", () => {
      this.handleClearStudentRosterOnly();
    });
    this.runStudentResultOnlyClearBtn?.addEventListener("click", () => {
      this.handleClearStudentResultOnly();
    });
    this.runStudentResultClearBtn?.addEventListener("click", () => {
      this.handleClearStudentResultReview();
    });
    this.runStudentResultReportCopyBtn?.addEventListener("click", () => {
      void this.handleCopyStudentResultReturnReport();
    });
    this.runStudentResultReportDownloadBtn?.addEventListener("click", () => {
      this.handleDownloadStudentResultReturnReport();
    });
    this.runStudentMissingReminderCopyBtn?.addEventListener("click", () => {
      void this.handleCopyStudentMissingReminder();
    });
    this.runStudentMissingReminderDownloadBtn?.addEventListener("click", () => {
      this.handleDownloadStudentMissingReminder();
    });
    this.runStudentReviewReminderCopyBtn?.addEventListener("click", () => {
      void this.handleCopyStudentReviewReminder();
    });
    this.runStudentReviewReminderDownloadBtn?.addEventListener("click", () => {
      this.handleDownloadStudentReviewReminder();
    });
    this.runPublicationPrepCopyBtn?.addEventListener("click", () => {
      void this.handleCopyPublicationPrepExport();
    });
    this.runRegistrySeedCopyBtn?.addEventListener("click", () => {
      void this.handleCopyRegistrySeedExport();
    });
    this.runApprovalContinuityCopyBtn?.addEventListener("click", () => {
      void this.handleCopyApprovalContinuityExport();
    });
    this.runBenchmarkLtsCopyBtn?.addEventListener("click", () => {
      void this.handleCopyBenchmarkLtsExport();
    });
    this.runEducationOperationsLtsCopyBtn?.addEventListener("click", () => {
      void this.handleCopyEducationOperationsLtsExport();
    });
    (Array.isArray(this.classroomModeBtns) ? this.classroomModeBtns : []).forEach((button) => {
      button.addEventListener("click", () => {
        this.applyClassroomModeSwitch(button.dataset.classroomMode);
      });
    });
    this.runMadiStatusEl = this.root.querySelector("#run-madi-status");
    this.runExecTechEl = this.root.querySelector("#run-exec-tech");
    this.runExecTechSummaryEl = this.root.querySelector("#run-exec-tech-summary");
    this.runExecTechBodyEl = this.root.querySelector("#run-exec-tech-body");
    this.runViewSourceBadgeEl = this.root.querySelector("#run-view-source-badge");
    this.runErrorBannerEl = this.root.querySelector("#run-error-banner");
    this.runErrorTextEl = this.root.querySelector("#run-error-text");
    this.runErrorActionBtn = this.root.querySelector("#btn-run-error-action");
    this.runWarningPlatformLoginBtn = this.root.querySelector("#btn-run-warning-platform-login");
    this.runWarningPlatformRequestAccessBtn = this.root.querySelector("#btn-run-warning-platform-request-access");
    this.runWarningPlatformOpenLocalSaveBtn = this.root.querySelector("#btn-run-warning-platform-open-local-save");
    this.runErrorDismissBtn = this.root.querySelector("#btn-run-error-dismiss");
    this.bogaeWarnBadgeEl = this.root.querySelector("#bogae-warn-badge");
    this.runMirrorDiagnosticsEl = this.root.querySelector("#run-mirror-diagnostics");
    this.runMirrorDiagnosticsSummaryEl = this.root.querySelector("#run-mirror-diagnostics-summary");
    this.runMirrorDiagnosticsChipsEl = this.root.querySelector("#run-mirror-diagnostics-chips");
    this.runMirrorDiagnosticsBodyEl = this.root.querySelector("#run-mirror-diagnostics-body");
    this.runMirrorHashEl = this.root.querySelector("#run-mirror-hash");
    this.runCopyHashBtn = this.root.querySelector("#btn-run-copy-hash");
    this.runMirrorWorldEl = this.root.querySelector("#run-mirror-world");
    this.runMirrorWorldSummaryEl = this.root.querySelector("#run-mirror-world-summary");
    this.runMirrorKvEl = this.root.querySelector("#run-mirror-kv");
    this.runSliderAreaEl = this.root.querySelector("#run-slider-area");
    this.saveSnapshotBtn = this.root.querySelector("#btn-save-snapshot-v0");
    this.saveSessionBtn = this.root.querySelector("#btn-save-session-v0");
    this.runTabButtons = RUN_TAB_IDS.map((tab) => this.root.querySelector(`#run-tab-btn-${tab}`)).filter(Boolean);
    this.runTabPanels = new Map(
      RUN_TAB_IDS.map((tab) => [tab, this.root.querySelector(`#run-tab-panel-${tab}`)]),
    );
    this.runVisualColumnEl = this.root.querySelector(".run-visual-column");
    this.runControlBarEl = this.root.querySelector(".run-control-bar");
    this.bogaeToolbarEl = this.runControlBarEl;
    this.bogaeAreaEl = this.root.querySelector(".bogae-area");
    this.runMainGraphHostEl = this.root.querySelector("#run-main-graph-host");
    this.runMainConsoleHostEl = this.root.querySelector("#run-main-console-host");
    this.dotbogiPanelEl = this.root.querySelector(".dotbogi-panel");
    this.graphPanelEl = this.root.querySelector("#dotbogi-graph");
    this.ensureRunManagerUi();
    this.runManagerPanelEl = this.root.querySelector("#run-manager-panel");
    this.runManagerListEl = this.root.querySelector("#run-manager-list");
    this.runManagerClearBtn = this.root.querySelector("#btn-run-manager-clear");
    this.runManagerPruneBtn = this.root.querySelector("#btn-run-manager-prune");
    this.runHistoryComparisonRailEl = this.root.querySelector("[data-run-history-comparison-rail]");
    this.runHistoryCountEl = this.root.querySelector("[data-run-history-count]");
    this.runHistoryVisibleEl = this.root.querySelector("[data-run-history-visible]");
    this.runHistoryActiveEl = this.root.querySelector("[data-run-history-active]");
    this.runHistorySoloEl = this.root.querySelector("[data-run-history-solo]");
    this.runHistoryExportSummaryEl = this.root.querySelector("[data-run-history-export-summary]");
    this.runHistoryExportSummaryMetaEl = this.root.querySelector("[data-run-history-export-summary-meta]");
    this.runHistoryExportSummaryTextEl = this.root.querySelector("[data-run-history-export-summary-text]");
    this.runHistoryExportSummaryCopyBtn = this.root.querySelector("#btn-run-history-export-summary-copy");
    this.suspendLiveRunCapture = false;
    this.runtimeStatusEl = this.runExecStatusEl;
    this.runtimeTablePanelEl = this.root.querySelector("#runtime-table-panel");
    this.runtimeTableMetaEl = this.root.querySelector("#runtime-table-meta");
    this.runtimeTableEl = this.root.querySelector("#runtime-table");
    this.runtimeTextPanelEl = this.root.querySelector("#runtime-text-panel");
    this.runtimeTextBodyEl = this.root.querySelector("#runtime-text-body");
    this.runOverlayMetaEl = this.root.querySelector("#run-overlay-meta");
    this.runOverlayBodyEl = this.root.querySelector("#run-overlay-body");
    this.runObserveSummaryEl = this.root.querySelector("#run-observe-summary");
    this.runObserveSummaryTextEl = this.root.querySelector("#run-observe-summary-text");
    this.runObserveOutputPanelEl = this.root.querySelector("#run-observe-output-panel");
    this.runObserveOutputMetaEl = this.root.querySelector("#run-observe-output-meta");
    this.runObserveOutputBodyEl = this.root.querySelector("#run-observe-output-body");
    this.runNumericKernelPanelEl = this.root.querySelector("#run-numeric-kernel-panel");
    this.runNumericKernelStatusEl = this.root.querySelector("#run-numeric-kernel-status");
    this.runNumericKernelBodyEl = this.root.querySelector("#run-numeric-kernel-body");
    this.relocateObserveSummaryToOutputPanel();
    this.runConsoleWarningSummaryEl = this.root.querySelector("#run-console-warning-summary");
    this.overlayToggleBtn = this.root.querySelector("#btn-overlay-toggle");
    this.runMainExecuteBtn = this.root.querySelector("#btn-run");
    this.runMainPauseBtn = this.root.querySelector("#btn-pause");
    this.runResetBtn = this.root.querySelector("#btn-reset");
    this.runStepBtn = this.root.querySelector("#btn-step");
    this.runViewModeButtons = STUDIO_VIEW_MODE_IDS
      .map((mode) => this.root.querySelector(`#btn-run-view-${mode}`))
      .filter(Boolean);
    this.runViewModeBasicBtn = this.root.querySelector("#btn-run-view-basic");
    this.runViewModeAnalyzeBtn = this.root.querySelector("#btn-run-view-analyze");
    this.runViewModeFullBtn = this.root.querySelector("#btn-run-view-full");
    this.subpanelEl = this.root.querySelector(".dotbogi-panel.subpanel");
    this.subpanelPanelEl = this.root.querySelector("#subpanel-tab-panel");
    this.runResetBtn?.setAttribute("aria-label", RUN_MAIN_RESET_LABEL);
    this.runResetBtn?.setAttribute("title", "초기화");
    this.runStepBtn?.setAttribute("aria-label", RUN_MAIN_STEP_LABEL);
    this.runStepBtn?.setAttribute("title", "한 단계씩");
    this.dockSpaceRangeEl = this.root.querySelector("#dock-space-range");
    this.dockGraphRangeEl = this.root.querySelector("#dock-graph-range");
    this.dockTargetSelectEl = this.root.querySelector("#select-dock-target");
    this.dockGridCheckEl = this.root.querySelector("#chk-dock-grid");
    this.dockAxisCheckEl = this.root.querySelector("#chk-dock-axis");
    this.dockXTicksCheckEl = this.root.querySelector("#chk-dock-x-ticks");
    this.graphKindSelectEl = this.root.querySelector("#select-graph-kind");
    this.graphRangeSelectEl = this.root.querySelector("#select-graph-range");
    this.dockHighlightCheckEl = this.root.querySelector("#chk-dock-highlight");
    this.dockLoopCheckEl = this.root.querySelector("#chk-dock-loop");
    this.dockSpeedSelectEl = this.root.querySelector("#select-dock-speed");
    this.dockTimeCursorEl = this.root.querySelector("#range-dock-time-cursor");
    this.dockTimeTextEl = this.root.querySelector("#text-dock-time");
    this.installRuntimeTableResizeObserver();
    this.setMainVisualMode("none");

    this.uiPrefs = {
      lessons: {},
      ...readStorageJson(RUN_UI_PREFS_STORAGE_KEY, {}),
    };
    if (!this.uiPrefs.lessons || typeof this.uiPrefs.lessons !== "object") {
      this.uiPrefs.lessons = {};
    }

    this.bogae = new Bogae({
      canvas: this.root.querySelector("#canvas-bogae"),
      onRangeChange: (range) => {
        this.syncDockRangeLabels({ spaceRange: range });
      },
    });
    this.dotbogi = new DotbogiPanel({
      graphCanvas: this.root.querySelector("#canvas-graph"),
      xAxisSelect: this.root.querySelector("#select-x-axis"),
      yAxisSelect: this.root.querySelector("#select-y-axis"),
      onAxisChange: (axis) => {
        this.syncDockRangeLabels({ graphAxis: axis });
      },
    });
    this.overlay = new OverlayDescription(this.root.querySelector("#overlay-description"));

    this.sliderPanel = new SliderPanel({
      container: this.root.querySelector("#slider-list"),
      statusEl: this.root.querySelector("#slider-status"),
      onCommit: () => {
        void this.restart();
      },
    });

    this.root.querySelector("#btn-overlay-toggle")?.addEventListener("click", () => {
      this.switchRunTab(SUBPANEL_TAB.OVERLAY);
      this.runOverlayBodyEl?.scrollIntoView?.({ block: "nearest", inline: "nearest", behavior: "smooth" });
    });

    this.root.querySelector("#btn-advanced-run")?.addEventListener("click", () => {
      this.onOpenAdvanced();
    });

    this.btnStudioNewEl?.addEventListener("click", () => {
      this.handleStudioNewDraft();
    });
    this.runLoadLocalBtn?.addEventListener("click", () => {
      void this.handleRunLocalLoad();
    });
    this.runSaveLocalBtn?.addEventListener("click", () => {
      void this.handleRunLocalSave();
    });
    this.runDdnPreviewEl?.addEventListener("input", () => {
      const next = String(this.runDdnPreviewEl?.value ?? "");
      this.baseDdn = next;
      this.currentDdn = next;
      if (this.lesson && typeof this.lesson === "object") {
        this.lesson.ddnText = next;
      }
      const parsed = this.sliderPanel.parseFromDdn(this.baseDdn, {
        preserveValues: true,
        maegimControlJson: this.lesson?.maegimControlJson ?? "",
      });
      this.syncRunSliderAreaVisibility();
      this.syncGraphDraftFromParsed(parsed);
      this.setStudioShellState({ sourceLabel: this.sourceLabel || "새 작업" });
      this.setRunLocalSaveStatus("저장 필요", { status: "warn" });
      this.syncLegacyAutofixAvailability();
      this.markRuntimeDirtyForSourceEdit();
      this.scheduleInlineWarningRefresh(next);
      this.scheduleLiveReplRestart(next);
      this.onSourceChange?.(next);
    });
    this.runDdnPreviewEl?.addEventListener("keydown", (event) => {
      const withModifier = event.ctrlKey || event.metaKey;
      if (!withModifier) return;
      if (String(event.key ?? "").toLowerCase() !== "enter") return;
      event.preventDefault();
      void this.restart();
    });
    this.runErrorDismissBtn?.addEventListener("click", () => {
      this.clearRunErrorBanner();
    });
    this.runErrorActionBtn?.addEventListener("click", () => {
      void this.handleRunErrorPrimaryAction();
    });
    this.runMirrorDiagnosticsChipsEl?.addEventListener("click", (event) => {
      this.handleMirrorDiagnosticsChipClick(event);
    });
    this.runCopyHashBtn?.addEventListener("click", () => {
      void this.handleCopyRunStateHash();
    });
    this.runDeliveryStudentNameEl?.addEventListener("input", () => {
      this.updateMirrorTab(this.lastState);
      this.syncLocalPackageDeliveryStatus();
    });
    this.runDeliveryResultCopyBtn?.addEventListener("click", () => {
      void this.handleCopyRunStateHash();
    });
    this.runDeliveryResultDownloadBtn?.addEventListener("click", () => {
      this.handleDownloadStudentDeliveryResult();
    });
    this.inlineAutofixBtn?.addEventListener("click", () => {
      void this.handleInlineWarningAction();
    });
    this.bogaeWarnBadgeEl?.addEventListener("click", () => {
      this.switchRunTab(SUBPANEL_TAB.MIRROR);
    });
    this.saveSnapshotBtn?.addEventListener("click", () => {
      this.handleSaveSnapshot();
    });
    this.saveSessionBtn?.addEventListener("click", () => {
      this.handleSaveSession();
    });
    this.runManagerClearBtn?.addEventListener("click", () => {
      this.clearRunManagerRuns();
    });
    this.runManagerPruneBtn?.addEventListener("click", () => {
      this.pruneHiddenRunManagerRuns();
    });
    this.runHistoryExportSummaryCopyBtn?.addEventListener("click", () => {
      void this.handleCopyRunHistoryExportSummary();
    });
    this.root.querySelector("#select-x-axis")?.addEventListener("change", () => {
      this.saveCurrentLessonUiPrefs();
    });
    this.root.querySelector("#select-y-axis")?.addEventListener("change", () => {
      this.saveCurrentLessonUiPrefs();
    });
    this.graphKindSelectEl?.addEventListener("change", () => {
      this.dotbogi?.setGraphKind?.(this.graphKindSelectEl?.value ?? "line");
      this.saveCurrentLessonUiPrefs();
    });
    this.graphRangeSelectEl?.addEventListener("change", () => {
      const nextRange = normalizeGraphRangeSelection(this.graphRangeSelectEl?.value);
      this.dotbogi?.setMaxPointsMode?.(nextRange);
      if (this.graphRangeSelectEl) {
        this.graphRangeSelectEl.value = String(this.dotbogi?.getMaxPointsMode?.() ?? nextRange);
      }
      this.saveCurrentLessonUiPrefs();
    });
    this.runLegacyAutofixBtn?.addEventListener("click", () => {
      void this.applyLegacyAutofix();
    });
    this.runPackageLessonSelectEl?.addEventListener("change", () => {
      void this.handlePackageLessonSelect();
    });
    this.runMainExecuteBtn?.addEventListener("click", () => {
      void this.handleMainExecutionControl("run");
    });
    this.runMainPauseBtn?.addEventListener("click", () => {
      void this.handleMainExecutionControl("pause");
    });
    this.runResetBtn?.addEventListener("click", () => {
      void this.handleResetExecutionControl();
    });
    this.runStepBtn?.addEventListener("click", () => {
      void this.handleStepExecutionControl();
    });
    this.runViewModeBasicBtn?.addEventListener("click", () => {
      this.setStudioViewMode(STUDIO_VIEW_MODE_BASIC);
    });
    this.runViewModeAnalyzeBtn?.addEventListener("click", () => {
      this.setStudioViewMode(STUDIO_VIEW_MODE_ANALYZE);
    });
    this.runViewModeFullBtn?.addEventListener("click", () => {
      this.setStudioViewMode(STUDIO_VIEW_MODE_FULL);
    });
    this.bindRunTabUi();
    this.bindViewDockUi();
    this.switchRunTab("graph");
    this.hydrateRunViewPrefs();
    this.hydrateStudioViewMode();
    this.setPrimaryView(this.primaryView, { persist: false, forceTab: true });
    this.syncDockGuideToggles();
    this.applyDockGuideToggles();
    this.syncDockRangeLabels();
    this.syncDockTimeUi();
    this.hydrateRunManagerFromSession({ publish: false });
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
    this.setInspectorStatus(this.lastInspectorStatusText || "저장 작업 대기");
    this.renderParseWarningPanel();
    this.renderConsoleWarningSummary();
    this.updateObserveSummary({ observation: null, views: null, outputRows: [] });
    this.syncLegacyAutofixAvailability();
    this.setRunOnboardingStatus(DEFAULT_ONBOARDING_STATUS_TEXT, { status: "idle" });
    this.syncRunPresetRail();
    this.syncRunActionRail();
    this.hydrateRunLayoutRatio();
    this.bindRunLayoutSplitter();
    this.installStudioLayoutResizeObserver();
    this.syncSubpanelTabLabels();
    this.syncRunControlState();

    this.loop = createManagedRafStepLoop({
      getFps: () => {
        return resolveRunLoopFps({
          fpsLimit: this.wasmState?.fpsLimit ?? 30,
          playbackSpeed: this.playbackSpeed,
          engineMode: this.engineMode,
          runtimeMaxMadi: this.runtimeMaxMadi,
        });
      },
      isActive: () => this.loopActive,
      setActive: (active) => {
        this.loopActive = Boolean(active);
      },
      onStep: () => {
        this.stepFrame();
      },
      onError: (err) => {
        // runtime loop errors are surfaced through diagnostics/gates.
      },
    });
    if (typeof window !== "undefined" && window?.addEventListener) {
      window.addEventListener("keydown", this.boundKeyDownHandler);
      window.addEventListener("keyup", this.boundKeyUpHandler);
      window.addEventListener("blur", this.boundWindowBlurHandler);
      window.addEventListener(PLATFORM_SERVER_EXCHANGE_EVENT, this.boundPlatformServerExchangeHandler);
    }
    this.applyPlatformServerAdapterExchange(null);
  }

  relocateObserveSummaryToOutputPanel() {
    const outputPanelRoot = this.runTabPanels?.get?.("output");
    if (!outputPanelRoot) return;
    const outputPanel = this.runObserveOutputPanelEl ?? outputPanelRoot.querySelector?.("#run-observe-output-panel");
    if (this.runObserveSummaryEl && this.runObserveSummaryEl.parentElement !== outputPanelRoot) {
      outputPanelRoot.insertBefore(this.runObserveSummaryEl, outputPanel ?? outputPanelRoot.firstChild);
    }
  }

  setMainVisualMode(mode = "none") {
    const next = normalizeRunMainVisualMode(mode, "none");
    this.lastMainVisualMode = next;
    if (this.root?.dataset) {
      this.root.dataset.mainVisualMode = next;
    }
  }

  renderMainVisual({ mode = "none", space2d = null, graphHtml = "", consoleHtml = "", consoleLinesForGrid = [] } = {}) {
    const nextMode = normalizeRunMainVisualMode(mode, "none");
    this.setMainVisualMode(nextMode);
    this.syncBogaeShellForMainVisual(nextMode);
    if (this.runMainGraphHostEl) {
      const showGraph = nextMode === "graph";
      this.runMainGraphHostEl.classList.toggle("hidden", !showGraph);
      this.runMainGraphHostEl.innerHTML = showGraph
        ? (String(graphHtml ?? "").trim() || '<div class="run-main-visual-empty">그래프 출력 없음</div>')
        : "";
    }
    if (this.runMainConsoleHostEl) {
      const showConsole = nextMode === "console" || nextMode === "console-grid" || nextMode === "none";
      this.runMainConsoleHostEl.classList.toggle("hidden", !showConsole);
      this.runMainConsoleHostEl.innerHTML = showConsole
        ? (String(consoleHtml ?? "").trim() || '<div class="run-main-visual-empty">실행 대기</div>')
        : "";
    }
    if (nextMode === "space2d" || nextMode === "debug-fallback") {
      this.bogae.render(space2d ?? null);
      return;
    }
    if (nextMode === "console-grid") {
      // 콘솔 격자 보개: 같은 캔버스에 터미널 스타일 문자 격자를 렌더한다.
      this.bogae.renderConsoleGrid(Array.isArray(consoleLinesForGrid) ? consoleLinesForGrid : []);
      return;
    }
    this.bogae.render(null);
  }

  setStudioShellState(patch = {}) {
    const next = patch && typeof patch === "object" ? patch : {};
    if (Object.prototype.hasOwnProperty.call(next, "sourceKind")) {
      this.sourceKind = String(next.sourceKind ?? "").trim() || this.sourceKind || "scratch";
    }
    if (Object.prototype.hasOwnProperty.call(next, "sourceLabel")) {
      this.sourceLabel = String(next.sourceLabel ?? "").trim() || "새 작업";
      if (this.studioSourceLabelEl) {
        this.studioSourceLabelEl.textContent = this.sourceLabel;
      }
    }
    if (Object.prototype.hasOwnProperty.call(next, "primaryViewFamily")) {
      this.primaryView = normalizeRunPrimaryView(next.primaryViewFamily, this.primaryView);
      if (this.root?.dataset) {
        this.root.dataset.primaryView = this.primaryView;
      }
      this.graphTabMode = resolveGraphTabMode(this.primaryView);
      if (this.root?.dataset) {
        this.root.dataset.graphTabMode = this.graphTabMode;
      }
    }
    if (Object.prototype.hasOwnProperty.call(next, "activeSubpanelTab")) {
      this.activeRunTab = normalizeRunTab(next.activeSubpanelTab);
    }
    if (Object.prototype.hasOwnProperty.call(next, "engineStatus")) {
      this.setEngineStatus(next.engineStatus);
      return;
    }
    this.onStudioStateChange?.({
      sourceKind: this.sourceKind,
      sourceLabel: this.sourceLabel,
      engineStatus: this.engineStatus,
      primaryViewFamily: this.primaryView,
      activeSubpanelTab: this.activeRunTab,
    });
  }

  setEngineStatus(status, { errorMessage = "" } = {}) {
    this.engineStatus = normalizeEngineStatus(status, this.engineStatus);
    if (typeof errorMessage === "string") {
      this.runErrorMessage = String(errorMessage).trim();
      this.runErrorDismissed = false;
    }
    this.syncRunControlState();
    this.syncLocalPackageDeliveryStatus();
    this.onStudioStateChange?.({
      sourceKind: this.sourceKind,
      sourceLabel: this.sourceLabel,
      engineStatus: this.engineStatus,
      primaryViewFamily: this.primaryView,
      activeSubpanelTab: this.activeRunTab,
    });
  }

  setEngineMode(mode = RUN_ENGINE_MODE_ONESHOT) {
    this.engineMode = normalizeRunEngineMode(mode, this.engineMode);
    if (this.root?.dataset) {
      this.root.dataset.engineMode = this.engineMode;
    }
    this.syncRunControlState();
    return this.engineMode;
  }

  resolveRuntimeMaxMadiLimit() {
    const candidates = [
      Number(this.runtimeMaxMadi),
      readConfiguredMadiFromClient(this.wasmState?.client),
      readConfiguredMadiFromDdnText(this.currentDdn),
      readConfiguredMadiFromDdnText(this.baseDdn),
      readConfiguredMadiFromDdnText(this.getEffectiveWasmSource(this.currentDdn || this.baseDdn)),
    ];
    const max = candidates
      .map((value) => Math.max(0, Math.trunc(Number(value) || 0)))
      .filter((value) => value > 0)
      .reduce((acc, value) => Math.max(acc, value), 0);
    this.runtimeMaxMadi = max;
    return max;
  }

  syncSubpanelTabLabels() {
    const tabs = resolveSubpanelTabs(this.primaryView);
    tabs.forEach((tabId) => {
      const button = this.root?.querySelector?.(`#run-tab-btn-${tabId}`);
      if (!button) return;
      button.textContent = SUBPANEL_TAB_LABEL[tabId] ?? button.textContent;
      button.classList.remove("hidden");
    });
    RUN_TAB_IDS.forEach((tabId) => {
      const button = this.root?.querySelector?.(`#run-tab-btn-${tabId}`);
      if (!button) return;
      button.classList.toggle("hidden", !tabs.includes(tabId));
    });
  }

  syncRunControlState() {
    const status = normalizeEngineStatus(this.engineStatus, "idle");
    const engineMode = normalizeRunEngineMode(this.engineMode, RUN_ENGINE_MODE_ONESHOT);
    const compact = this.syncBogaeToolbarCompactState({ refreshControls: false });
    const runtimeMaxMadi = this.resolveRuntimeMaxMadiLimit();
    const isRunning = status === "running";
    const isPaused = status === "paused";
    const isIdle = status === "idle" || status === "done";
    const isBlocked = status === "blocked";
    const isFatal = status === "fatal";
    const reachedMaxMadi = hasReachedRuntimeMaxMadi(this.runtimeTickCounter, runtimeMaxMadi);
    const labels = resolveRunMainControlLabels({
      isPaused,
      compact,
      launchKind: this.lastLaunchKind,
    });
    if (this.runMainExecuteBtn) {
      this.runMainExecuteBtn.disabled = isRunning || isBlocked || isFatal;
      this.runMainExecuteBtn.textContent = labels.execute;
      this.runMainExecuteBtn.setAttribute("aria-label", labels.execute);
    }
    if (this.runMainPauseBtn) {
      this.runMainPauseBtn.disabled = !isRunning;
      this.runMainPauseBtn.textContent = labels.pause;
      this.runMainPauseBtn.setAttribute("aria-label", labels.pause);
    }
    if (this.runResetBtn) {
      this.runResetBtn.disabled = isBlocked || isFatal;
      this.runResetBtn.textContent = labels.reset;
      this.runResetBtn.setAttribute("aria-label", labels.reset);
      this.runResetBtn.setAttribute("title", "초기화");
    }
    if (this.runStepBtn) {
      this.runStepBtn.disabled = isRunning || engineMode === RUN_ENGINE_MODE_ONESHOT || reachedMaxMadi;
      this.runStepBtn.textContent = labels.step;
      this.runStepBtn.setAttribute("aria-label", labels.step);
      this.runStepBtn.setAttribute(
        "title",
        engineMode === RUN_ENGINE_MODE_ONESHOT
          ? "정적 실행에는 다음 단계가 없습니다."
          : (reachedMaxMadi ? "마지막 단계에 도달했습니다." : "한 단계씩"),
      );
    }
    if (this.runExecUserStatusEl) {
      const labelMap = {
        idle: "실행 대기",
        running: "실행 중",
        paused: "일시정지",
        done: "실행 완료",
        blocked: "실행 차단",
        fatal: "실행 실패",
      };
      this.runExecUserStatusEl.textContent = labelMap[status] ?? "실행 대기";
      this.runExecUserStatusEl.dataset.status = isBlocked || isFatal ? "error" : (isPaused ? "warn" : (isRunning ? "ok" : "idle"));
    }
    if (this.runtimeStatusEl?.dataset) {
      this.runtimeStatusEl.dataset.status = isBlocked || isFatal ? "error" : (isPaused ? "warn" : (isRunning ? "ok" : "idle"));
    }
    const showError = (isBlocked || isFatal) && Boolean(String(this.runErrorMessage ?? "").trim()) && !this.runErrorDismissed;
    this.runErrorBannerEl?.classList?.toggle("hidden", !showError);
    if (this.lastRunErrorBannerVisible !== showError) {
      this.lastRunErrorBannerVisible = showError;
      this.refreshStudioLayoutBounds({ persist: false });
    }
    if (showError && this.runErrorTextEl) {
      this.runErrorTextEl.textContent = String(this.runErrorMessage ?? "").trim();
    }
    this.renderWarningBadge();
    this.syncRunMadiStatus();
  }

  syncRunMadiStatus() {
    if (!this.runMadiStatusEl) return;
    const current = Math.max(0, Math.trunc(Number(this.runtimeTickCounter) || 0));
    const max = this.resolveRuntimeMaxMadiLimit();
    const maxText = max > 0 ? String(max) : "-";
    const text = `${current}/${maxText}단계`;
    this.runMadiStatusEl.textContent = text;
    this.runMadiStatusEl.title = max > 0
      ? `현재 단계 ${current} / 전체 단계 ${max}`
      : `현재 단계 ${current} / 전체 단계 없음`;
    this.runMadiStatusEl.setAttribute?.("aria-label", this.runMadiStatusEl.title);
  }

  scheduleInlineWarningRefresh(sourceText = "") {
    if (this.inlineWarnTimer) {
      clearTimeout(this.inlineWarnTimer);
    }
    this.inlineWarnTimer = setTimeout(() => {
      this.renderInlineWarning();
    }, 500);
    this.inlineWarningModel = this.inlineWarningModel && typeof this.inlineWarningModel === "object"
      ? { ...this.inlineWarningModel, sourceText: String(sourceText ?? "") }
      : null;
  }

  setStudioReadinessModel(model = null) {
    this.inlineWarningModel = model && typeof model === "object" ? { ...model } : null;
    this.renderInlineWarning();
  }

  renderInlineWarning() {
    if (!this.studioInlineWarnEl || !this.studioInlineWarnTextEl) return;
    const model = this.inlineWarningModel && typeof this.inlineWarningModel === "object" ? this.inlineWarningModel : null;
    const stage = String(model?.stage ?? "").trim().toLowerCase();
    if (!model || stage === "ready") {
      if (this.inlineAutofixBtn) {
        this.inlineAutofixBtn.dataset.actionKind = "default";
      }
      this.studioInlineWarnEl.classList.add("hidden");
      return;
    }
    this.studioInlineWarnTextEl.textContent = String(model?.user_cause ?? "입력을 점검해 주세요.").trim() || "입력을 점검해 주세요.";
    if (this.inlineAutofixBtn) {
      const action = model?.primary_action && typeof model.primary_action === "object" ? model.primary_action : null;
      const actionKind = String(action?.kind ?? "").trim().toLowerCase();
      const visible = Boolean(actionKind);
      this.inlineAutofixBtn.classList.toggle("hidden", !visible);
      this.inlineAutofixBtn.textContent = String(action?.label ?? "수정 후 다시 실행").trim() || "수정 후 다시 실행";
      this.inlineAutofixBtn.title = String(model?.manual_example ?? action?.detail ?? "").trim();
      this.inlineAutofixBtn.dataset.actionKind = actionKind || "default";
    }
    this.studioInlineWarnEl.classList.remove("hidden");
  }

  async handleInlineWarningAction() {
    const model = this.inlineWarningModel && typeof this.inlineWarningModel === "object" ? this.inlineWarningModel : null;
    const action = model?.primary_action && typeof model.primary_action === "object" ? model.primary_action : null;
    const actionKind = String(action?.kind ?? "").trim().toLowerCase();
    if (!actionKind) return false;
    if (actionKind === "autofix") {
      return this.applyLegacyAutofix();
    }
    if (actionKind === "open_inspector") {
      this.switchRunTab(SUBPANEL_TAB.MIRROR);
      if (this.runMirrorDiagnosticsEl) {
        this.runMirrorDiagnosticsEl.open = true;
      }
      this.runMirrorDiagnosticsEl?.scrollIntoView?.({ block: "nearest", inline: "nearest", behavior: "smooth" });
      return true;
    }
    if (actionKind === "open_ddn" || actionKind === "manual_fix_example") {
      this.focusRunDdnEditor();
      const detail = String(model?.manual_example ?? action?.detail ?? "").trim();
      if (detail) {
        this.setObserveGuideStatus(detail, { ttlMs: 4200 });
      }
      return true;
    }
    if (actionKind === "retry" || actionKind === "run") {
      return this.restart();
    }
    return false;
  }

  async handleCopyRunStateHash() {
    const value = String(this.lastRuntimeHash ?? "-").trim();
    if (!value || value === "-") {
      showGlobalToast("복사할 상태 기록이 없습니다.", { kind: "error" });
      return false;
    }
    const studentDelivery = this.lastLaunchKind === "local_package_import";
    const studentName = studentDelivery ? String(this.runDeliveryStudentNameEl?.value ?? "").trim() : "";
    if (studentDelivery && !studentName) {
      showGlobalToast("학생 이름을 입력한 뒤 결과를 복사하세요.", { kind: "error" });
      this.runDeliveryStudentNameEl?.focus?.();
      return false;
    }
    const copyText = studentDelivery
      ? buildStudentResultReturnText({ lesson: this.lesson, hashText: value, studentName })
      : value;
    const lessonId = String(studentDelivery ? this.lesson?.localPackageLessonId ?? this.lesson?.id : this.lesson?.id ?? "").trim();
    const packageId = studentDelivery ? String(this.lesson?.localPackageId ?? "").trim() : "";
    const sessionLabel = studentDelivery ? String(this.lesson?.localPackageSessionLabel ?? "").trim() : "";
    let ok = false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(copyText);
        ok = true;
      }
    } catch (_) {
      ok = false;
    }
    try {
      window.__SEAMGRIM_RUN_STATE_HASH_COPY_ACTION__ = {
        schema: "seamgrim.run_state_hash_copy_action.v1",
        launch_kind: String(this.lastLaunchKind ?? "manual"),
        student_delivery: studentDelivery,
        submission_method: studentDelivery ? "clipboard" : "state_hash_clipboard",
        student_name: studentName,
        lesson_id: lessonId,
        package_id: packageId,
        session_label: sessionLabel,
        state_hash: value,
        text: copyText,
        copied: ok,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok
      ? studentDelivery ? "제출할 결과를 복사했습니다." : "상태 기록을 복사했습니다."
      : studentDelivery ? "결과 복사에 실패했습니다." : "상태 기록 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  handleDownloadStudentDeliveryResult() {
    const value = String(this.lastRuntimeHash ?? "-").trim();
    if (!value || value === "-") {
      showGlobalToast("저장할 수업 결과가 없습니다.", { kind: "error" });
      return false;
    }
    const studentDelivery = this.lastLaunchKind === "local_package_import";
    const studentName = studentDelivery ? String(this.runDeliveryStudentNameEl?.value ?? "").trim() : "";
    if (studentDelivery && !studentName) {
      showGlobalToast("학생 이름을 입력한 뒤 결과를 저장하세요.", { kind: "error" });
      this.runDeliveryStudentNameEl?.focus?.();
      return false;
    }
    const resultText = studentDelivery
      ? buildStudentResultReturnText({ lesson: this.lesson, hashText: value, studentName })
      : value;
    const lessonId = String(studentDelivery ? this.lesson?.localPackageLessonId ?? this.lesson?.id : this.lesson?.id ?? "lesson").trim() || "lesson";
    const sessionLabel = studentDelivery ? String(this.lesson?.localPackageSessionLabel ?? "").trim() : "";
    const packageId = studentDelivery ? String(this.lesson?.localPackageId ?? "").trim() : "";
    const sessionSuffix = sessionLabel ? `_${sessionLabel}` : "";
    const packageSuffix = packageId ? `_${packageId}` : "";
    const studentSuffix = studentName ? `_${studentName}` : "";
    const fileName = buildSafeDownloadName(`${lessonId}${sessionSuffix}${packageSuffix}${studentSuffix}_student_result.txt`, "seamgrim-student-result", "txt");
    let ok = false;
    try {
      savePlainTextToFile(resultText, fileName);
      ok = true;
    } catch (_) {
      ok = false;
    }
    try {
      window.__SEAMGRIM_RUN_STATE_HASH_DOWNLOAD_ACTION__ = {
        schema: "seamgrim.run_state_hash_download_action.v1",
        downloaded: ok,
        launch_kind: String(this.lastLaunchKind ?? "manual"),
        student_delivery: studentDelivery,
        submission_method: studentDelivery ? "file" : "state_hash_file",
        student_name: studentName,
        lesson_id: lessonId,
        package_id: packageId,
        session_label: sessionLabel,
        state_hash: value,
        file_name: fileName,
        text: resultText,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "제출할 결과를 저장했습니다." : "결과 저장에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  hasStudentResultReviewInput() {
    const resultText = String(this.runStudentResultInputEl?.value ?? "").trim();
    const rosterText = String(this.runStudentRosterInputEl?.value ?? "").trim();
    return Boolean(resultText || rosterText);
  }

  syncStudentResultReviewControls() {
    const enabled = this.hasStudentResultReviewInput();
    if (this.runStudentResultReviewBtn) {
      this.runStudentResultReviewBtn.disabled = !enabled;
    }
    return enabled;
  }

  resolveStudentResultExpectedLesson() {
    const lesson = this.lesson ?? {};
    const existingPackageId = String(lesson?.localPackageId ?? "").trim();
    if (existingPackageId) return lesson;
    try {
      const packageModel = this.buildLocalPackageExportModel();
      const packageTitle = String(packageModel?.manifest?.title ?? "").trim();
      const studentInstructions = Array.isArray(packageModel?.payload?.student_instructions)
        ? packageModel.payload.student_instructions
        : [];
      return {
        ...lesson,
        localPackageLessonId: String(packageModel?.lesson_id ?? lesson?.id ?? "").trim(),
        localPackageSessionLabel: String(packageModel?.session_label ?? "").trim(),
        localPackageTitle: packageTitle,
        localPackageId: String(packageModel?.package_id ?? "").trim(),
        localPackageStudentInstructions: studentInstructions,
        localPackageMaterialsSummary: Array.isArray(packageModel?.manifest?.materials_summary)
          ? packageModel.manifest.materials_summary
          : [],
        localPackageStudentMaterialsSummary: Array.isArray(packageModel?.manifest?.student_materials_summary)
          ? packageModel.manifest.student_materials_summary
          : [],
      };
    } catch (_) {
      return lesson;
    }
  }

  buildStudentResultReviewInputStatusText({ dirty = false, prefix = "" } = {}) {
    const resultText = String(this.runStudentResultInputEl?.value ?? "").trim();
    const rosterText = String(this.runStudentRosterInputEl?.value ?? "").trim();
    const resultCount = resultText ? splitStudentResultReturnBlocks(resultText).length : 0;
    const rosterCount = rosterText ? countStudentRosterExpectations(rosterText) : 0;
    const rosterCourseScoped = rosterText ? shouldUseStudentRosterTargetCountLabel(rosterText) : false;
    if (resultCount <= 0 && rosterCount <= 0) return "결과 대기";
    const rosterAnalysis = analyzeStudentRosterText(rosterText);
    const expectedLesson = this.resolveStudentResultExpectedLesson();
    const rosterMetadataReview = buildStudentRosterMetadataReview(rosterText, expectedLesson);
    const rosterMetadataNotes = buildStudentRosterMetadataReviewNotes(rosterMetadataReview);
    const parts = [
      String(prefix ?? "").trim() || (dirty ? "입력 수정됨" : "입력 준비됨"),
      resultCount > 0 ? `학생 결과 ${resultCount}건` : "학생 결과 대기",
    ];
    if (rosterCount > 0) {
      parts.push(formatStudentRosterExpectationLabel(rosterCount, { courseScoped: rosterCourseScoped }));
    }
    if (rosterAnalysis.duplicate_count > 0) {
      parts.push(`명단 중복 ${rosterAnalysis.duplicate_count}명`);
    }
    if (rosterCount > 0 && resultCount <= 0 && rosterMetadataNotes.length) {
      parts.push(`확인 항목: ${rosterMetadataNotes.slice(0, 2).join(", ")}`);
    }
    if (resultCount > 0) {
      const previewBatch = parseStudentResultReturnBatchText(resultText, { lesson: expectedLesson, rosterText });
      parts.push(`사전 확인 ${previewBatch.accepted_count}/${previewBatch.row_count}`);
      if (previewBatch.rejected_count > 0) {
        const reviewNotes = getStudentResultReviewReminderRows(previewBatch)
          .flatMap((row) => buildStudentResultReviewNotes(row))
          .filter(Boolean);
        const uniqueNotes = Array.from(new Set(reviewNotes)).slice(0, 2);
        parts.push(`확인 필요 ${previewBatch.rejected_count}건`);
        if (uniqueNotes.length) parts.push(`확인 항목: ${uniqueNotes.join(", ")}`);
      }
    }
    parts.push(resultCount > 0 ? "다음 행동: 결과 확인" : "다음 행동: 학생 결과 붙여넣기");
    return parts.join(" · ");
  }

  resetStudentResultReviewArtifacts() {
    if (this.runStudentResultSummaryEl) {
      this.runStudentResultSummaryEl.classList.add("hidden");
      this.runStudentResultSummaryEl.textContent = "";
    }
    if (this.runStudentResultTableEl) {
      this.runStudentResultTableEl.classList.add("hidden");
      this.runStudentResultTableEl.innerHTML = "";
    }
    if (this.runStudentResultReportCopyBtn) {
      this.runStudentResultReportCopyBtn.disabled = true;
    }
    if (this.runStudentResultReportDownloadBtn) {
      this.runStudentResultReportDownloadBtn.disabled = true;
    }
    if (this.runStudentMissingReminderCopyBtn) {
      this.runStudentMissingReminderCopyBtn.disabled = true;
    }
    if (this.runStudentMissingReminderDownloadBtn) {
      this.runStudentMissingReminderDownloadBtn.disabled = true;
    }
    if (this.runStudentReviewReminderCopyBtn) {
      this.runStudentReviewReminderCopyBtn.disabled = true;
    }
    if (this.runStudentReviewReminderDownloadBtn) {
      this.runStudentReviewReminderDownloadBtn.disabled = true;
    }
    try {
      window.__SEAMGRIM_STUDENT_RESULT_RETURN_REVIEW__ = null;
      window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ = null;
    } catch (_) {
      // ignore browser instrumentation errors
    }
  }

  handleStudentResultReviewInputChanged() {
    const enabled = this.syncStudentResultReviewControls();
    const hadReview = Boolean(
      globalThis?.__SEAMGRIM_STUDENT_RESULT_RETURN_REVIEW__
        || globalThis?.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__
    );
    if (hadReview) {
      this.studentResultReviewInputDirtyAfterReview = true;
    }
    this.resetStudentResultReviewArtifacts();
    if (this.runStudentResultStatusEl) {
      const currentState = String(this.runStudentResultStatusEl.dataset?.state ?? "").trim();
      const currentText = String(this.runStudentResultStatusEl.textContent ?? "").trim();
      if (enabled && (this.studentResultReviewInputDirtyAfterReview === true || currentState === "ok" || currentState === "error")) {
        this.runStudentResultStatusEl.dataset.state = "idle";
        this.runStudentResultStatusEl.textContent = this.buildStudentResultReviewInputStatusText({ dirty: true });
      } else if (enabled && (!currentText || currentText === "결과 대기")) {
        this.runStudentResultStatusEl.dataset.state = "idle";
        this.runStudentResultStatusEl.textContent = this.buildStudentResultReviewInputStatusText();
      } else if (!enabled) {
        this.runStudentResultStatusEl.dataset.state = "idle";
        this.runStudentResultStatusEl.textContent = "결과 대기";
      }
    }
    return enabled;
  }

  async handleLoadStudentResultFiles(event = null) {
    const input = event?.currentTarget ?? this.runStudentResultFileInputEl;
    const files = Array.from(input?.files ?? []);
    if (!files.length) return false;
    let ok = false;
    let loadedText = "";
    let resultCount = 0;
    let acceptedCount = 0;
    let rejectedCount = 0;
    let loadedFileCount = 0;
    let skippedDuplicateCount = 0;
    let nextActionText = "";
    let duplicateFileNames = [];
    let resultReportScope = {
      multi_lesson: false,
      lesson_ids: [],
      package_ids: [],
      session_labels: [],
      lesson_id: "",
      package_id: "",
      session_label: "",
    };
    const fileNames = files.map((file) => String(file?.name ?? "result.txt").trim() || "result.txt");
    try {
      const rawChunks = await Promise.all(files.map((file) => readTextFromLocalFile(file)));
      const chunks = rawChunks
        .map((text, index) => ({
          file_name: fileNames[index] || `result-${index + 1}.txt`,
          text: String(text ?? "").trim(),
        }))
        .filter((entry) => entry.text);
      if (!chunks.length) throw new Error("학생 결과 파일이 비어 있습니다.");
      const current = String(this.runStudentResultInputEl?.value ?? "").trim();
      const existingKeys = collectStudentResultReturnBlockKeys(current);
      const acceptedChunks = [];
      duplicateFileNames = [];
      chunks.forEach((entry) => {
        const blockKeys = splitStudentResultReturnBlocks(entry.text)
          .map((block) => normalizeStudentResultReturnBlockKey(block))
          .filter(Boolean);
        const duplicate = blockKeys.length > 0 && blockKeys.every((key) => existingKeys.has(key));
        if (duplicate) {
          skippedDuplicateCount += blockKeys.length;
          duplicateFileNames.push(entry.file_name);
          return;
        }
        acceptedChunks.push(entry.text);
        blockKeys.forEach((key) => existingKeys.add(key));
      });
      loadedFileCount = acceptedChunks.length;
      loadedText = acceptedChunks.join("\n\n");
      if (this.runStudentResultInputEl) {
        this.runStudentResultInputEl.value = [current, loadedText].filter(Boolean).join("\n\n");
      }
      const reviewText = String(this.runStudentResultInputEl?.value ?? "").trim();
      const rosterText = String(this.runStudentRosterInputEl?.value ?? "").trim();
      const expectedLesson = this.resolveStudentResultExpectedLesson();
      const previewBatch = parseStudentResultReturnBatchText(reviewText, {
        lesson: expectedLesson,
        rosterText,
      });
      resultReportScope = buildStudentResultReportScope(previewBatch.rows, { lesson: expectedLesson });
      resultCount = Math.max(0, Number(previewBatch.row_count) || 0);
      acceptedCount = Math.max(0, Number(previewBatch.accepted_count) || 0);
      rejectedCount = Math.max(0, Number(previewBatch.rejected_count) || 0);
      nextActionText = String(previewBatch.next_action ?? buildStudentResultNextActionText(previewBatch)).trim();
      ok = true;
      this.resetStudentResultReviewArtifacts();
      this.studentResultReviewInputDirtyAfterReview = false;
      if (this.runStudentResultStatusEl) {
        this.runStudentResultStatusEl.dataset.state = "idle";
        const skippedText = skippedDuplicateCount > 0 ? ` · 중복 ${skippedDuplicateCount}건 건너뜀` : "";
        this.runStudentResultStatusEl.textContent = this.buildStudentResultReviewInputStatusText({
          prefix: `결과 파일 ${loadedFileCount}개 불러옴${skippedText}`,
        });
      }
      this.syncStudentResultReviewControls();
      this.runStudentResultReviewBtn?.focus?.();
      const skippedToastText = skippedDuplicateCount > 0 ? ` 중복 ${skippedDuplicateCount}건은 건너뛰었습니다.` : "";
      showGlobalToast(`학생 결과 파일 ${loadedFileCount}개를 불러왔습니다.${skippedToastText}`, { kind: "success" });
    } catch (error) {
      if (this.runStudentResultStatusEl) {
        this.runStudentResultStatusEl.dataset.state = "error";
        this.runStudentResultStatusEl.textContent = `결과 파일 불러오기 실패: ${String(error?.message ?? error)}`;
      }
      showGlobalToast("학생 결과 파일 불러오기에 실패했습니다.", { kind: "error" });
    } finally {
      if (input) input.value = "";
    }
    try {
      window.__SEAMGRIM_STUDENT_RESULT_FILE_LOAD_ACTION__ = {
        schema: "seamgrim.student_result_file_load_action.v1",
        loaded: ok,
        file_count: files.length,
        loaded_file_count: loadedFileCount,
        file_names: fileNames,
        duplicate_file_names: duplicateFileNames,
        skipped_duplicate_count: skippedDuplicateCount,
        result_count: resultCount,
        accepted_count: acceptedCount,
        rejected_count: rejectedCount,
        multi_lesson: resultReportScope.multi_lesson,
        lesson_ids: resultReportScope.lesson_ids,
        package_ids: resultReportScope.package_ids,
        session_labels: resultReportScope.session_labels,
        lesson_id: resultReportScope.lesson_id,
        package_id: resultReportScope.package_id,
        session_label: resultReportScope.session_label,
        next_action: nextActionText,
        text: loadedText,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    return ok;
  }

  async handleLoadStudentRosterFiles(event = null) {
    const input = event?.currentTarget ?? this.runStudentRosterFileInputEl;
    const files = Array.from(input?.files ?? []);
    if (!files.length) return false;
    let ok = false;
    let loadedText = "";
    const fileNames = files.map((file) => String(file?.name ?? "roster.txt").trim() || "roster.txt");
    try {
      const chunks = (await Promise.all(files.map((file) => readTextFromLocalFile(file))))
        .map((text) => String(text ?? "").trim())
        .filter(Boolean);
      if (!chunks.length) throw new Error("학생 명단 파일이 비어 있습니다.");
      const current = String(this.runStudentRosterInputEl?.value ?? "").trim();
      loadedText = chunks.join("\n");
      if (this.runStudentRosterInputEl) {
        this.runStudentRosterInputEl.value = [current, loadedText].filter(Boolean).join("\n");
      }
      ok = true;
      this.resetStudentResultReviewArtifacts();
      this.studentResultReviewInputDirtyAfterReview = false;
      if (this.runStudentResultStatusEl) {
        this.runStudentResultStatusEl.dataset.state = "idle";
        this.runStudentResultStatusEl.textContent = this.buildStudentResultReviewInputStatusText({
          prefix: `명단 파일 ${chunks.length}개 불러옴`,
        });
      }
      this.syncStudentResultReviewControls();
      this.runStudentResultReviewBtn?.focus?.();
      showGlobalToast(`학생 명단 파일 ${chunks.length}개를 불러왔습니다.`, { kind: "success" });
    } catch (error) {
      if (this.runStudentResultStatusEl) {
        this.runStudentResultStatusEl.dataset.state = "error";
        this.runStudentResultStatusEl.textContent = `명단 파일 불러오기 실패: ${String(error?.message ?? error)}`;
      }
      showGlobalToast("학생 명단 파일 불러오기에 실패했습니다.", { kind: "error" });
    } finally {
      if (input) input.value = "";
    }
    const rosterTextForAction = String(this.runStudentRosterInputEl?.value ?? loadedText ?? "").trim();
    const resultTextForAction = String(this.runStudentResultInputEl?.value ?? "").trim();
    const nextActionText = resultTextForAction ? "다음 행동: 결과 확인" : "다음 행동: 학생 결과 붙여넣기";
    const expectedLesson = this.resolveStudentResultExpectedLesson();
    const rosterMetadataReview = buildStudentRosterMetadataReview(rosterTextForAction, expectedLesson);
    const rosterAnalysis = analyzeStudentRosterText(rosterTextForAction);
    try {
      window.__SEAMGRIM_STUDENT_ROSTER_FILE_LOAD_ACTION__ = {
        schema: "seamgrim.student_roster_file_load_action.v1",
        loaded: ok,
        file_count: files.length,
        file_names: fileNames,
        roster_count: countStudentRosterExpectations(rosterTextForAction),
        roster_count_kind: shouldUseStudentRosterTargetCountLabel(rosterTextForAction) ? "lesson_targets" : "students",
        roster_entry_count: rosterAnalysis.entry_count,
        duplicate_roster_count: rosterAnalysis.duplicate_count,
        duplicate_roster_names: rosterAnalysis.duplicate_names,
        roster_metadata: rosterMetadataReview.metadata,
        roster_lesson_id_match: rosterMetadataReview.lesson_id_match,
        roster_session_match: rosterMetadataReview.session_match,
        roster_package_id_match: rosterMetadataReview.package_id_match,
        roster_context_match: rosterMetadataReview.context_match,
        next_action: nextActionText,
        text: loadedText,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    return ok;
  }

  buildStudentRosterTemplateModel() {
    const packageRosterReport = findLocalPackageReportFromLesson(this.lesson, (report) => (
      report.title.includes("학생 명단 양식")
        || report.report_id.includes("student_roster")
        || report.path.endsWith(".tsv")
    ));
    const packageModel = typeof this.buildLocalPackageExportModel === "function" ? this.buildLocalPackageExportModel() : null;
    const lessonId = String(this.lesson?.localPackageLessonId ?? packageModel?.lesson_id ?? this.lesson?.id ?? "lesson").trim() || "lesson";
    const sessionLabel = String(this.lesson?.localPackageSessionLabel ?? packageModel?.session_label ?? "").trim();
    const packageId = String(this.lesson?.localPackageId ?? packageModel?.package_id ?? "").trim();
    const packageLessons = Array.isArray(this.lesson?.localPackagePayload?.lessons)
      ? this.lesson.localPackagePayload.lessons
      : [];
    const lessonScope = buildLocalPackageLessonScope({ lessons: packageLessons }, { fallbackLessonId: lessonId });
    const value = String(packageRosterReport?.text ?? "").trim()
      || buildStudentRosterTemplateText({ lessonId, sessionLabel, packageId });
    const fileName = packageRosterReport
      ? localPackageReportFileName(packageRosterReport, buildStudentRosterTemplateFileName({ lessonId, sessionLabel, packageId }))
      : buildStudentRosterTemplateFileName({ lessonId, sessionLabel, packageId });
    return {
      file_name: fileName,
      lesson_id: lessonId,
      package_id: packageId,
      session_label: sessionLabel,
      multi_lesson: lessonScope.multi_lesson,
      lesson_ids: lessonScope.lesson_ids,
      text: value,
    };
  }

  async handleCopyStudentRosterTemplate() {
    const model = this.buildStudentRosterTemplateModel();
    let ok = false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(model.text);
        ok = true;
      }
    } catch (_) {
      ok = false;
    }
    try {
      window.__SEAMGRIM_STUDENT_ROSTER_TEMPLATE_COPY_ACTION__ = {
        schema: "seamgrim.student_roster_template_copy_action.v1",
        copied: ok,
        file_name: model.file_name,
        lesson_id: model.lesson_id,
        package_id: model.package_id,
        session_label: model.session_label,
        multi_lesson: model.multi_lesson,
        lesson_ids: model.lesson_ids,
        text: model.text,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "학생 명단 양식을 복사했습니다." : "학생 명단 양식 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  handleDownloadStudentRosterTemplate() {
    const model = this.buildStudentRosterTemplateModel();
    let ok = false;
    try {
      saveTsvTextToFile(model.text, model.file_name);
      ok = true;
    } catch (_) {
      ok = false;
    }
    try {
      window.__SEAMGRIM_STUDENT_ROSTER_TEMPLATE_DOWNLOAD_ACTION__ = {
        schema: "seamgrim.student_roster_template_download_action.v1",
        downloaded: ok,
        file_name: model.file_name,
        lesson_id: model.lesson_id,
        package_id: model.package_id,
        session_label: model.session_label,
        multi_lesson: model.multi_lesson,
        lesson_ids: model.lesson_ids,
        text: model.text,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "학생 명단 양식을 저장했습니다." : "학생 명단 양식 저장에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  buildTeacherPreparationChecklistModel() {
    const packageChecklistReport = findLocalPackageReportFromLesson(this.lesson, (report) => (
      report.title.includes("교사용 준비 체크리스트")
        || report.report_id.includes("teacher_preparation_checklist")
    ));
    const packageModel = typeof this.buildLocalPackageExportModel === "function" ? this.buildLocalPackageExportModel() : null;
    const lessonId = String(this.lesson?.localPackageLessonId ?? packageModel?.lesson_id ?? this.lesson?.id ?? "lesson").trim() || "lesson";
    const lessonTitle = String(this.lesson?.title ?? packageModel?.lesson_title ?? lessonId).trim() || lessonId;
    const sessionLabel = String(this.lesson?.localPackageSessionLabel ?? packageModel?.session_label ?? "").trim();
    const packageId = String(this.lesson?.localPackageId ?? packageModel?.package_id ?? "").trim();
    const lessonScope = buildLocalPackageLessonScope(this.lesson?.localPackagePayload, { fallbackLessonId: lessonId });
    const requiredViews = resolveLessonRequiredViewsForRun(this.lesson);
    const goals = Array.isArray(this.lesson?.goals)
      ? this.lesson.goals.map((item) => String(item ?? "").trim()).filter(Boolean)
      : [];
    const missions = Array.isArray(this.lesson?.missions)
      ? this.lesson.missions.map((item) => String(item ?? "").trim()).filter(Boolean)
      : [];
    const value = String(packageChecklistReport?.text ?? "").trim()
      || buildTeacherPreparationChecklistText({
        lessonTitle,
        lessonId,
        sessionLabel,
        packageId,
        requiredViews,
        goals,
        missions,
      });
    const fileName = packageChecklistReport
      ? localPackageReportFileName(packageChecklistReport, buildTeacherPreparationChecklistFileName({ lessonId, sessionLabel, packageId }))
      : buildTeacherPreparationChecklistFileName({ lessonId, sessionLabel, packageId });
    return {
      file_name: fileName,
      lesson_id: lessonId,
      lesson_title: lessonTitle,
      package_id: packageId,
      session_label: sessionLabel,
      multi_lesson: lessonScope.multi_lesson,
      lesson_ids: lessonScope.lesson_ids,
      text: value,
    };
  }

  async handleCopyTeacherPreparationChecklist() {
    const model = this.buildTeacherPreparationChecklistModel();
    let ok = false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(model.text);
        ok = true;
      }
    } catch (_) {
      ok = false;
    }
    try {
      window.__SEAMGRIM_TEACHER_PREPARATION_CHECKLIST_COPY_ACTION__ = {
        schema: "seamgrim.teacher_preparation_checklist_copy_action.v1",
        copied: ok,
        file_name: model.file_name,
        lesson_id: model.lesson_id,
        lesson_title: model.lesson_title,
        package_id: model.package_id,
        session_label: model.session_label,
        multi_lesson: model.multi_lesson,
        lesson_ids: model.lesson_ids,
        text: model.text,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "교사용 준비 체크리스트를 복사했습니다." : "교사용 준비 체크리스트 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  handleDownloadTeacherPreparationChecklist() {
    const model = this.buildTeacherPreparationChecklistModel();
    let ok = false;
    try {
      savePlainTextToFile(model.text, model.file_name);
      ok = true;
    } catch (_) {
      ok = false;
    }
    try {
      window.__SEAMGRIM_TEACHER_PREPARATION_CHECKLIST_DOWNLOAD_ACTION__ = {
        schema: "seamgrim.teacher_preparation_checklist_download_action.v1",
        downloaded: ok,
        file_name: model.file_name,
        lesson_id: model.lesson_id,
        lesson_title: model.lesson_title,
        package_id: model.package_id,
        session_label: model.session_label,
        multi_lesson: model.multi_lesson,
        lesson_ids: model.lesson_ids,
        text: model.text,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "교사용 준비 체크리스트를 저장했습니다." : "교사용 준비 체크리스트 저장에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  handleClearStudentResultReview() {
    if (this.runStudentRosterInputEl) {
      this.runStudentRosterInputEl.value = "";
    }
    if (this.runStudentResultInputEl) {
      this.runStudentResultInputEl.value = "";
    }
    if (this.runStudentRosterFileInputEl) {
      this.runStudentRosterFileInputEl.value = "";
    }
    if (this.runStudentResultFileInputEl) {
      this.runStudentResultFileInputEl.value = "";
    }
    this.syncStudentResultReviewControls();
    this.resetStudentResultReviewArtifacts();
    this.studentResultReviewInputDirtyAfterReview = false;
    if (this.runStudentResultStatusEl) {
      this.runStudentResultStatusEl.dataset.state = "idle";
      this.runStudentResultStatusEl.textContent = "결과 대기";
    }
    try {
      window.__SEAMGRIM_STUDENT_RESULT_CLEAR_ACTION__ = {
        schema: "seamgrim.student_result_clear_action.v1",
        cleared: true,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast("학생 결과 입력을 비웠습니다.", { kind: "success" });
    return true;
  }

  handleClearStudentResultOnly() {
    if (this.runStudentResultInputEl) {
      this.runStudentResultInputEl.value = "";
    }
    if (this.runStudentResultFileInputEl) {
      this.runStudentResultFileInputEl.value = "";
    }
    const rosterText = String(this.runStudentRosterInputEl?.value ?? "").trim();
    const rosterCount = rosterText ? countStudentRosterExpectations(rosterText) : 0;
    const rosterCourseScoped = rosterText ? shouldUseStudentRosterTargetCountLabel(rosterText) : false;
    this.syncStudentResultReviewControls();
    this.resetStudentResultReviewArtifacts();
    this.studentResultReviewInputDirtyAfterReview = false;
    if (this.runStudentResultStatusEl) {
      this.runStudentResultStatusEl.dataset.state = "idle";
      this.runStudentResultStatusEl.textContent = this.buildStudentResultReviewInputStatusText({
        prefix: rosterCount > 0 ? "결과만 비움" : "결과 대기",
      });
    }
    try {
      window.__SEAMGRIM_STUDENT_RESULT_ONLY_CLEAR_ACTION__ = {
        schema: "seamgrim.student_result_only_clear_action.v1",
        cleared: true,
        roster_preserved: rosterCount > 0,
        roster_count: rosterCount,
        roster_count_kind: rosterCourseScoped ? "lesson_targets" : "students",
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    if (rosterCount > 0) {
      this.runStudentResultInputEl?.focus?.();
    }
    showGlobalToast(rosterCount > 0 ? "학생 명단은 두고 결과만 비웠습니다." : "학생 결과 입력을 비웠습니다.", { kind: "success" });
    return true;
  }

  handleClearStudentRosterOnly() {
    const resultText = String(this.runStudentResultInputEl?.value ?? "").trim();
    const resultCount = resultText ? splitStudentResultReturnBlocks(resultText).length : 0;
    if (this.runStudentRosterInputEl) {
      this.runStudentRosterInputEl.value = "";
    }
    if (this.runStudentRosterFileInputEl) {
      this.runStudentRosterFileInputEl.value = "";
    }
    this.syncStudentResultReviewControls();
    this.resetStudentResultReviewArtifacts();
    this.studentResultReviewInputDirtyAfterReview = false;
    if (this.runStudentResultStatusEl) {
      this.runStudentResultStatusEl.dataset.state = "idle";
      this.runStudentResultStatusEl.textContent = this.buildStudentResultReviewInputStatusText({
        prefix: resultCount > 0 ? "명단만 비움" : "결과 대기",
      });
    }
    try {
      window.__SEAMGRIM_STUDENT_ROSTER_ONLY_CLEAR_ACTION__ = {
        schema: "seamgrim.student_roster_only_clear_action.v1",
        cleared: true,
        result_preserved: resultCount > 0,
        result_count: resultCount,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    if (resultCount > 0) {
      this.runStudentRosterInputEl?.focus?.();
    }
    showGlobalToast(resultCount > 0 ? "학생 결과는 두고 명단만 비웠습니다." : "학생 명단 입력을 비웠습니다.", { kind: "success" });
    return true;
  }

  handleReviewStudentResultReturn() {
    if (!this.hasStudentResultReviewInput()) {
      this.syncStudentResultReviewControls();
      if (this.runStudentResultStatusEl) {
        this.runStudentResultStatusEl.dataset.state = "idle";
        this.runStudentResultStatusEl.textContent = "결과 대기 · 학생 결과나 명단을 먼저 입력하세요.";
      }
      showGlobalToast("학생 결과나 명단을 먼저 입력하세요.", { kind: "error" });
      return false;
    }
    const text = String(this.runStudentResultInputEl?.value ?? "").trim();
    const rosterText = String(this.runStudentRosterInputEl?.value ?? "").trim();
    const expectedLesson = this.resolveStudentResultExpectedLesson();
    const batch = parseStudentResultReturnBatchText(text, { lesson: expectedLesson, rosterText });
    const model = batch.rows[0] || parseStudentResultReturnText(text, { lesson: expectedLesson });
    const accepted = batch.accepted === true;
    const nextActionText = buildStudentResultNextActionText(batch);
    this.studentResultReviewInputDirtyAfterReview = false;
    if (this.runStudentResultStatusEl) {
      const duplicateRosterNamesText = formatDuplicateRosterNamesText(batch);
      const rosterCountLabel = formatStudentRosterExpectationLabel(batch.roster_count, {
        courseScoped: batch.roster_count_kind === "lesson_targets",
      });
      const duplicateRosterSuffix = batch.duplicate_roster_count > 0
        ? ` · 명단 중복 ${batch.duplicate_roster_count}명${duplicateRosterNamesText ? ` (${duplicateRosterNamesText})` : ""}`
        : "";
      const duplicateRosterOnlyReview = accepted !== true
        && batch.duplicate_roster_count > 0
        && batch.rejected_count <= 0
        && batch.accepted_count === batch.row_count;
      this.runStudentResultStatusEl.dataset.state = accepted ? "ok" : "error";
      this.runStudentResultStatusEl.textContent = batch.roster_count > 0
        ? duplicateRosterOnlyReview
          ? `명단 확인 필요 ${batch.accepted_count}/${batch.row_count} · ${rosterCountLabel}${duplicateRosterSuffix}`
          : accepted
          ? `확인됨 ${batch.accepted_count}/${batch.row_count} · ${rosterCountLabel}${duplicateRosterSuffix}`
          : `확인 필요 ${batch.accepted_count}/${batch.row_count} · 미제출 ${batch.missing_count}명${duplicateRosterSuffix}`
        : batch.row_count > 1
        ? duplicateRosterOnlyReview
          ? `명단 확인 필요 ${batch.accepted_count}/${batch.row_count}${duplicateRosterSuffix}`
          : accepted
          ? `확인됨 ${batch.accepted_count}/${batch.row_count}${duplicateRosterSuffix}`
          : `확인 필요 ${batch.accepted_count}/${batch.row_count}${duplicateRosterSuffix}`
        : model.accepted
          ? `확인됨 · ${model.state_hash.slice(0, 18)}`
          : `확인 필요 · ${buildStudentResultReviewNotes(model).join(", ") || "수업 이름 또는 상태 기록을 확인하세요."}`;
    }
    if (this.runStudentResultSummaryEl) {
      this.runStudentResultSummaryEl.classList.toggle("hidden", batch.row_count <= 0);
      this.runStudentResultSummaryEl.textContent = batch.row_count > 0
        ? [
          `학생 결과 ${batch.row_count}건`,
          `확인 ${batch.accepted_count}건`,
          `확인 필요 ${batch.rejected_count}건`,
          ...(batch.roster_count > 0 ? [formatStudentRosterExpectationLabel(batch.roster_count, {
            courseScoped: batch.roster_count_kind === "lesson_targets",
          }), `미제출 ${batch.missing_count}명`] : []),
          ...(batch.duplicate_roster_count > 0
            ? [`명단 중복 ${batch.duplicate_roster_count}명${formatDuplicateRosterNamesText(batch) ? ` (${formatDuplicateRosterNamesText(batch)})` : ""}`]
            : []),
          nextActionText,
        ].join(" · ")
        : "";
    }
    if (this.runStudentResultTableEl) {
      this.runStudentResultTableEl.classList.toggle("hidden", batch.row_count <= 0);
      this.runStudentResultTableEl.innerHTML = batch.row_count > 0
        ? `
          <table>
            <thead>
              <tr><th>학생</th><th>차시</th><th>수업</th><th>수업 코드</th><th>배포 코드</th><th>상태</th><th>상태 기록</th><th>비고</th></tr>
            </thead>
            <tbody>
              ${batch.rows.map((row) => `
                <tr data-student-result-row="${row.accepted === true ? "ok" : "review"}">
                  <td>${escapeHtml(row.student_name)}</td>
                  <td>${escapeHtml(row.session_label || "-")}</td>
                  <td>${escapeHtml(row.lesson_title || "-")}</td>
                  <td>${escapeHtml(row.lesson_id || "-")}</td>
                  <td>${escapeHtml(row.package_id || "-")}</td>
                  <td>${row.accepted === true ? "확인됨" : "확인 필요"}</td>
                  <td>${escapeHtml(formatCompactStateHash(row.state_hash || "-").replace(/^상태 기록:\s*/, ""))}</td>
                  <td>${escapeHtml(buildStudentResultReviewNotes(row).join(", ") || "-")}</td>
                </tr>
              `).join("")}
            </tbody>
          </table>
        `
        : "";
    }
    if (this.runStudentResultReportCopyBtn) {
      this.runStudentResultReportCopyBtn.disabled = batch.row_count <= 0;
    }
    if (this.runStudentResultReportDownloadBtn) {
      this.runStudentResultReportDownloadBtn.disabled = batch.row_count <= 0;
    }
    if (this.runStudentMissingReminderCopyBtn) {
      this.runStudentMissingReminderCopyBtn.disabled = batch.missing_count <= 0;
    }
    if (this.runStudentMissingReminderDownloadBtn) {
      this.runStudentMissingReminderDownloadBtn.disabled = batch.missing_count <= 0;
    }
    const reviewReminderRows = getStudentResultReviewReminderRows(batch);
    if (this.runStudentReviewReminderCopyBtn) {
      this.runStudentReviewReminderCopyBtn.disabled = reviewReminderRows.length <= 0;
    }
    if (this.runStudentReviewReminderDownloadBtn) {
      this.runStudentReviewReminderDownloadBtn.disabled = reviewReminderRows.length <= 0;
    }
    try {
      batch.next_action = nextActionText;
      window.__SEAMGRIM_STUDENT_RESULT_RETURN_REVIEW__ = model;
      window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ = batch;
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(accepted ? "학생 결과를 확인했습니다." : "학생 결과 확인이 필요합니다.", {
      kind: accepted ? "success" : "error",
    });
    return accepted;
  }

  async handleCopyStudentResultReturnReport() {
    const batch = globalThis?.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__;
    const value = String(batch?.report_text ?? "").trim();
    if (!value) {
      showGlobalToast("복사할 학생 결과 보고가 없습니다.", { kind: "error" });
      return false;
    }
    let ok = false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(value);
        ok = true;
      }
    } catch (_) {
      ok = false;
    }
    const rows = Array.isArray(batch?.rows) ? batch.rows : [];
    const reportScope = buildStudentResultReportScope(rows, { lesson: this.lesson });
    const lessonId = reportScope.lesson_id;
    const packageId = reportScope.package_id;
    const sessionLabel = reportScope.session_label;
    try {
      window.__SEAMGRIM_STUDENT_RESULT_RETURN_REPORT_COPY_ACTION__ = {
        schema: "seamgrim.student_result_return_report_copy_action.v1",
        copied: ok,
        row_count: Math.max(0, Number(batch?.row_count) || 0),
        accepted_count: Math.max(0, Number(batch?.accepted_count) || 0),
        rejected_count: Math.max(0, Number(batch?.rejected_count) || 0),
        roster_count: Math.max(0, Number(batch?.roster_count) || 0),
        missing_count: Math.max(0, Number(batch?.missing_count) || 0),
        roster_entry_count: Math.max(0, Number(batch?.roster_entry_count) || 0),
        duplicate_roster_count: Math.max(0, Number(batch?.duplicate_roster_count) || 0),
        duplicate_roster_names: Array.isArray(batch?.duplicate_roster_names) ? batch.duplicate_roster_names : [],
        roster_metadata: toPlainObject(batch?.roster_metadata, {}),
        roster_lesson_id_match: batch?.roster_lesson_id_match !== false,
        roster_session_match: batch?.roster_session_match !== false,
        roster_package_id_match: batch?.roster_package_id_match !== false,
        roster_context_match: batch?.roster_context_match !== false,
        multi_lesson: reportScope.multi_lesson,
        lesson_ids: reportScope.lesson_ids,
        package_ids: reportScope.package_ids,
        session_labels: reportScope.session_labels,
        lesson_id: lessonId,
        package_id: packageId,
        session_label: sessionLabel,
        next_action: String(batch?.next_action ?? buildStudentResultNextActionText(batch)).trim(),
        text: value,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "학생 결과 보고를 복사했습니다." : "학생 결과 보고 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  handleDownloadStudentResultReturnReport() {
    const batch = globalThis?.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__;
    const value = String(batch?.report_text ?? "").trim();
    if (!value) {
      showGlobalToast("저장할 학생 결과 보고가 없습니다.", { kind: "error" });
      return false;
    }
    const rows = Array.isArray(batch?.rows) ? batch.rows : [];
    const reportScope = buildStudentResultReportScope(rows, { lesson: this.lesson });
    const lessonId = String(reportScope.lesson_id || "lesson").trim() || "lesson";
    const sessionLabel = String(reportScope.session_label ?? "").trim();
    const sessionSuffix = sessionLabel ? `_${sessionLabel}` : "";
    const packageId = String(reportScope.package_id ?? "").trim();
    const packageSuffix = packageId ? `_${packageId}` : "";
    const fileStem = reportScope.multi_lesson
      ? `${packageId || "course"}${sessionSuffix}_student_results`
      : `${lessonId}${sessionSuffix}${packageSuffix}_student_results`;
    const fileName = buildSafeDownloadName(`${fileStem}.tsv`, "seamgrim-student-results", "tsv");
    let ok = false;
    try {
      saveTsvTextToFile(value, fileName);
      ok = true;
    } catch (_) {
      ok = false;
    }
    try {
      window.__SEAMGRIM_STUDENT_RESULT_RETURN_REPORT_DOWNLOAD_ACTION__ = {
        schema: "seamgrim.student_result_return_report_download_action.v1",
        downloaded: ok,
        file_name: fileName,
        row_count: Math.max(0, Number(batch?.row_count) || 0),
        accepted_count: Math.max(0, Number(batch?.accepted_count) || 0),
        rejected_count: Math.max(0, Number(batch?.rejected_count) || 0),
        roster_count: Math.max(0, Number(batch?.roster_count) || 0),
        missing_count: Math.max(0, Number(batch?.missing_count) || 0),
        roster_entry_count: Math.max(0, Number(batch?.roster_entry_count) || 0),
        duplicate_roster_count: Math.max(0, Number(batch?.duplicate_roster_count) || 0),
        duplicate_roster_names: Array.isArray(batch?.duplicate_roster_names) ? batch.duplicate_roster_names : [],
        roster_metadata: toPlainObject(batch?.roster_metadata, {}),
        roster_lesson_id_match: batch?.roster_lesson_id_match !== false,
        roster_session_match: batch?.roster_session_match !== false,
        roster_package_id_match: batch?.roster_package_id_match !== false,
        roster_context_match: batch?.roster_context_match !== false,
        multi_lesson: reportScope.multi_lesson,
        lesson_ids: reportScope.lesson_ids,
        package_ids: reportScope.package_ids,
        session_labels: reportScope.session_labels,
        lesson_id: reportScope.lesson_id,
        package_id: packageId,
        session_label: sessionLabel,
        next_action: String(batch?.next_action ?? buildStudentResultNextActionText(batch)).trim(),
        text: value,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "학생 결과표를 저장했습니다." : "학생 결과표 저장에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  async handleCopyStudentMissingReminder() {
    const batch = globalThis?.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__;
    const value = buildStudentResultMissingReminderText(batch, { lesson: this.lesson });
    if (!value) {
      showGlobalToast("복사할 미제출 안내가 없습니다.", { kind: "error" });
      return false;
    }
    let ok = false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(value);
        ok = true;
      }
    } catch (_) {
      ok = false;
    }
    const missingTargets = getStudentResultMissingReminderTargets(batch);
    const missingNames = Array.from(new Set(missingTargets.map((target) => target.student_name).filter(Boolean)));
    const reminderScope = buildStudentReminderScope(missingTargets, { lesson: this.lesson, fallbackLessonId: "lesson" });
    try {
      window.__SEAMGRIM_STUDENT_MISSING_REMINDER_COPY_ACTION__ = {
        schema: "seamgrim.student_missing_reminder_copy_action.v1",
        copied: ok,
        missing_count: missingNames.length,
        missing_names: missingNames,
        missing_targets: missingTargets,
        multi_lesson: reminderScope.multi_lesson,
        lesson_ids: reminderScope.lesson_ids,
        package_ids: reminderScope.package_ids,
        session_labels: reminderScope.session_labels,
        lesson_id: reminderScope.lesson_id,
        package_id: reminderScope.package_id,
        session_label: reminderScope.session_label,
        next_action: String(batch?.next_action ?? buildStudentResultNextActionText(batch)).trim(),
        duplicate_roster_count: Math.max(0, Number(batch?.duplicate_roster_count) || 0),
        duplicate_roster_names: Array.isArray(batch?.duplicate_roster_names) ? batch.duplicate_roster_names : [],
        roster_metadata: toPlainObject(batch?.roster_metadata, {}),
        roster_lesson_id_match: batch?.roster_lesson_id_match !== false,
        roster_session_match: batch?.roster_session_match !== false,
        roster_package_id_match: batch?.roster_package_id_match !== false,
        roster_context_match: batch?.roster_context_match !== false,
        text: value,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "미제출 안내를 복사했습니다." : "미제출 안내 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  handleDownloadStudentMissingReminder() {
    const batch = globalThis?.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__;
    const value = buildStudentResultMissingReminderText(batch, { lesson: this.lesson });
    if (!value) {
      showGlobalToast("저장할 미제출 안내가 없습니다.", { kind: "error" });
      return false;
    }
    const missingTargets = getStudentResultMissingReminderTargets(batch);
    const reminderScope = buildStudentReminderScope(missingTargets, { lesson: this.lesson, fallbackLessonId: "lesson" });
    const fileName = buildStudentReminderDownloadFileName(reminderScope, "missing_reminder", "course");
    let ok = false;
    try {
      savePlainTextToFile(value, fileName);
      ok = true;
    } catch (_) {
      ok = false;
    }
    const missingNames = Array.from(new Set(missingTargets.map((target) => target.student_name).filter(Boolean)));
    try {
      window.__SEAMGRIM_STUDENT_MISSING_REMINDER_DOWNLOAD_ACTION__ = {
        schema: "seamgrim.student_missing_reminder_download_action.v1",
        downloaded: ok,
        file_name: fileName,
        missing_count: missingNames.length,
        missing_names: missingNames,
        missing_targets: missingTargets,
        multi_lesson: reminderScope.multi_lesson,
        lesson_ids: reminderScope.lesson_ids,
        package_ids: reminderScope.package_ids,
        session_labels: reminderScope.session_labels,
        lesson_id: reminderScope.lesson_id,
        package_id: reminderScope.package_id,
        session_label: reminderScope.session_label,
        next_action: String(batch?.next_action ?? buildStudentResultNextActionText(batch)).trim(),
        duplicate_roster_count: Math.max(0, Number(batch?.duplicate_roster_count) || 0),
        duplicate_roster_names: Array.isArray(batch?.duplicate_roster_names) ? batch.duplicate_roster_names : [],
        roster_metadata: toPlainObject(batch?.roster_metadata, {}),
        roster_lesson_id_match: batch?.roster_lesson_id_match !== false,
        roster_session_match: batch?.roster_session_match !== false,
        roster_package_id_match: batch?.roster_package_id_match !== false,
        roster_context_match: batch?.roster_context_match !== false,
        text: value,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "미제출 안내를 저장했습니다." : "미제출 안내 저장에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  async handleCopyStudentReviewReminder() {
    const batch = globalThis?.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__;
    const value = buildStudentResultReviewReminderText(batch, { lesson: this.lesson });
    if (!value) {
      showGlobalToast("복사할 확인 필요 안내가 없습니다.", { kind: "error" });
      return false;
    }
    let ok = false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(value);
        ok = true;
      }
    } catch (_) {
      ok = false;
    }
    const reviewRows = getStudentResultReviewReminderRows(batch);
    const studentNames = reviewRows
      .map((row) => String(row?.student_name ?? "").trim())
      .filter(Boolean);
    const reviewTargets = getStudentResultReviewReminderTargets(batch);
    const reminderScope = buildStudentReminderScope(reviewTargets, { lesson: this.lesson, fallbackLessonId: "lesson" });
    try {
      window.__SEAMGRIM_STUDENT_REVIEW_REMINDER_COPY_ACTION__ = {
        schema: "seamgrim.student_review_reminder_copy_action.v1",
        copied: ok,
        review_count: reviewRows.length,
        student_names: studentNames,
        review_targets: reviewTargets,
        multi_lesson: reminderScope.multi_lesson,
        lesson_ids: reminderScope.lesson_ids,
        package_ids: reminderScope.package_ids,
        session_labels: reminderScope.session_labels,
        lesson_id: reminderScope.lesson_id,
        package_id: reminderScope.package_id,
        session_label: reminderScope.session_label,
        next_action: String(batch?.next_action ?? buildStudentResultNextActionText(batch)).trim(),
        duplicate_roster_count: Math.max(0, Number(batch?.duplicate_roster_count) || 0),
        duplicate_roster_names: Array.isArray(batch?.duplicate_roster_names) ? batch.duplicate_roster_names : [],
        roster_metadata: toPlainObject(batch?.roster_metadata, {}),
        roster_lesson_id_match: batch?.roster_lesson_id_match !== false,
        roster_session_match: batch?.roster_session_match !== false,
        roster_package_id_match: batch?.roster_package_id_match !== false,
        roster_context_match: batch?.roster_context_match !== false,
        text: value,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "확인 필요 안내를 복사했습니다." : "확인 필요 안내 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  handleDownloadStudentReviewReminder() {
    const batch = globalThis?.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__;
    const value = buildStudentResultReviewReminderText(batch, { lesson: this.lesson });
    if (!value) {
      showGlobalToast("저장할 확인 필요 안내가 없습니다.", { kind: "error" });
      return false;
    }
    const reviewRows = getStudentResultReviewReminderRows(batch);
    const studentNames = reviewRows
      .map((row) => String(row?.student_name ?? "").trim())
      .filter(Boolean);
    const reviewTargets = getStudentResultReviewReminderTargets(batch);
    const reminderScope = buildStudentReminderScope(reviewTargets, { lesson: this.lesson, fallbackLessonId: "lesson" });
    const fileName = buildStudentReminderDownloadFileName(reminderScope, "review_reminder", "course");
    let ok = false;
    try {
      savePlainTextToFile(value, fileName);
      ok = true;
    } catch (_) {
      ok = false;
    }
    try {
      window.__SEAMGRIM_STUDENT_REVIEW_REMINDER_DOWNLOAD_ACTION__ = {
        schema: "seamgrim.student_review_reminder_download_action.v1",
        downloaded: ok,
        file_name: fileName,
        review_count: reviewRows.length,
        student_names: studentNames,
        review_targets: reviewTargets,
        multi_lesson: reminderScope.multi_lesson,
        lesson_ids: reminderScope.lesson_ids,
        package_ids: reminderScope.package_ids,
        session_labels: reminderScope.session_labels,
        lesson_id: reminderScope.lesson_id,
        package_id: reminderScope.package_id,
        session_label: reminderScope.session_label,
        next_action: String(batch?.next_action ?? buildStudentResultNextActionText(batch)).trim(),
        duplicate_roster_count: Math.max(0, Number(batch?.duplicate_roster_count) || 0),
        duplicate_roster_names: Array.isArray(batch?.duplicate_roster_names) ? batch.duplicate_roster_names : [],
        roster_metadata: toPlainObject(batch?.roster_metadata, {}),
        roster_lesson_id_match: batch?.roster_lesson_id_match !== false,
        roster_session_match: batch?.roster_session_match !== false,
        roster_package_id_match: batch?.roster_package_id_match !== false,
        roster_context_match: batch?.roster_context_match !== false,
        text: value,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "확인 필요 안내를 저장했습니다." : "확인 필요 안내 저장에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  renderWarningBadge() {
    if (!this.bogaeWarnBadgeEl) return;
    const count = Array.isArray(this.lastParseWarnings) ? this.lastParseWarnings.length : 0;
    this.bogaeWarnBadgeEl.classList.toggle("hidden", count <= 0);
    this.bogaeWarnBadgeEl.title = count > 0 ? `경고 ${count}개 - 검증 탭에서 상세 확인` : "";
  }

  handleMirrorDiagnosticsChipClick(event) {
    const button = event?.target?.closest?.(".run-mirror-summary-chip");
    if (!button || !this.runMirrorDiagnosticsChipsEl?.contains?.(button)) return false;
    event?.preventDefault?.();
    event?.stopPropagation?.();
    const category = String(button?.dataset?.category ?? "").trim();
    this.activeMirrorDiagnosticCategory = this.activeMirrorDiagnosticCategory === category ? "" : category;
    if (this.runMirrorDiagnosticsEl) {
      this.runMirrorDiagnosticsEl.open = true;
    }
    this.renderMirrorDiagnostics();
    return true;
  }

  renderMirrorDiagnostics() {
    if (!this.runMirrorDiagnosticsBodyEl || !this.runMirrorDiagnosticsSummaryEl) return;
    const warnings = Array.isArray(this.lastParseWarnings) ? this.lastParseWarnings : [];
    if (!warnings.length) {
      this.runMirrorDiagnosticsSummaryEl.title = "진단 상세";
      if (this.runMirrorDiagnosticsChipsEl) {
        this.runMirrorDiagnosticsChipsEl.innerHTML = "";
      }
      this.activeMirrorDiagnosticCategory = "";
      this.runMirrorDiagnosticsBodyEl.textContent = "경고 없음";
      this.runMirrorDiagnosticsEl?.classList?.add?.("hidden");
      this.collapseMirrorDetails();
      return;
    }
    const categoryCounts = new Map();
    const normalizedWarnings = warnings.map((warning) => {
      const code = String(warning?.code ?? warning?.technical_code ?? "").trim();
      const category = classifyToUserCategory(code);
      categoryCounts.set(category, Number(categoryCounts.get(category) ?? 0) + 1);
      return { warning, code, category };
    });
    const activeCategory = String(this.activeMirrorDiagnosticCategory ?? "").trim();
    if (activeCategory && !categoryCounts.has(activeCategory)) {
      this.activeMirrorDiagnosticCategory = "";
    }
    const filteredWarnings = this.activeMirrorDiagnosticCategory
      ? normalizedWarnings.filter((row) => row.category === this.activeMirrorDiagnosticCategory)
      : normalizedWarnings;
    this.runMirrorDiagnosticsEl?.classList?.remove?.("hidden");
    this.runMirrorDiagnosticsSummaryEl.title = this.activeMirrorDiagnosticCategory
      ? `진단 상세 · ${this.activeMirrorDiagnosticCategory} ${filteredWarnings.length}건`
      : `진단 상세 · ${warnings.length}건`;
    if (this.runMirrorDiagnosticsChipsEl) {
      this.runMirrorDiagnosticsChipsEl.innerHTML = Array.from(categoryCounts.entries())
        .map(([category, count]) => (
          `<button type="button" class="run-mirror-summary-chip" data-kind="${escapeHtml(String(category))}" data-category="${escapeHtml(String(category))}" data-active="${this.activeMirrorDiagnosticCategory === category ? "true" : "false"}">${escapeHtml(String(category))} ${escapeHtml(String(count))}</button>`
        ))
        .join("");
    }
    this.runMirrorDiagnosticsBodyEl.innerHTML = filteredWarnings
      .map(({ warning, code, category }, index) => {
        const technical = String(warning?.technical_message ?? "").trim();
        const user = resolveUserWarningCause(warning);
        const entryText = [
          `${index + 1}. [${category}] ${code || "-"}`,
          user ? `- 사용자: ${user}` : "",
          technical ? `- 기술: ${technical}` : "",
        ].filter(Boolean).join("\n");
        return `<div class="run-mirror-diagnostics-entry" data-active="${this.activeMirrorDiagnosticCategory === category ? "true" : "false"}">${escapeHtml(entryText)}</div>`;
      })
      .join("");
  }

  collapseMirrorDetails() {
    if (this.runExecTechEl) {
      this.runExecTechEl.open = false;
    }
    if (this.runMirrorDiagnosticsEl) {
      this.runMirrorDiagnosticsEl.open = false;
    }
  }

  renderGraphTabMode() {
    this.graphTabMode = resolveGraphTabMode(this.primaryView);
    if (this.root?.dataset) {
      this.root.dataset.graphTabMode = this.graphTabMode;
    }
    this.syncDockPanelVisibility();
  }

  handleStudioNewDraft() {
    this.applyBaseDdnText("", { preserveControlValues: false, restart: false });
    this.lastGraphSnapshot = null;
    this.dotbogi?.setPersistedGraph?.(null, { render: false });
    this.clearRunManagerRuns();
    this.lesson = {
      ...(this.lesson && typeof this.lesson === "object" ? this.lesson : {}),
      ddnText: "",
      title: "새 작업",
      id: "",
    };
    this.setStudioShellState({
      sourceKind: "scratch",
      sourceLabel: "새 작업",
      engineStatus: "idle",
      primaryViewFamily: "sim",
      activeSubpanelTab: SUBPANEL_TAB.MAEGIM,
    });
    this.focusRunDdnEditor();
    this.onSourceChange?.("");
  }

  async handleResetExecutionControl() {
    this.clearRunErrorBanner();
    this.executionPaused = true;
    this.haltLoop();
    this.runtimeTickCounter = 0;
    this.runtimeMaxMadi = Math.max(
      readConfiguredMadiFromClient(this.wasmState?.client),
      readConfiguredMadiFromDdnText(this.currentDdn),
      readConfiguredMadiFromDdnText(this.baseDdn),
    );
    this.runtimeTimeValue = null;
    this.serverPlayback = null;
    const preservedGraphSnapshot = cloneGraphForRunManager(this.lastGraphSnapshot ?? null);
    const activeRunId = String(this.activeOverlayRunId ?? "").trim();
    const activeRunIndex = activeRunId ? this.findRunManagerIndexById(activeRunId) : -1;
    if (this.wasmState?.client && typeof this.wasmState.client.resetParsed === "function") {
      try {
        this.suspendLiveRunCapture = true;
        this.wasmState.client.resetParsed(true);
        if (typeof this.wasmState.client.getStateParsed === "function") {
          const resetState = this.wasmState.client.getStateParsed();
          this.lastState = resetState;
          this.applyRuntimeState(resetState, { forceView: true });
          this.setHash(typeof this.wasmState.client.getStateHash === "function" ? this.wasmState.client.getStateHash() : "-");
        }
      } catch (_) {
        this.lastState = null;
      } finally {
        this.suspendLiveRunCapture = false;
      }
    } else {
      this.lastState = null;
      this.lastRuntimeDerived = null;
      this.setHash("-");
    }
    if (preservedGraphSnapshot) {
      this.lastGraphSnapshot = preservedGraphSnapshot;
      if (activeRunIndex >= 0 && this.overlayRuns[activeRunIndex]) {
        this.overlayRuns[activeRunIndex] = this.normalizeRunManagerRun(
          {
            ...this.overlayRuns[activeRunIndex],
            graph: preservedGraphSnapshot,
            result: preservedGraphSnapshot,
          },
          activeRunIndex,
        );
      }
      this.dotbogi?.clearTimeline?.({ preserveAxes: true, preserveView: false });
      this.dotbogi?.setPersistedGraph?.(preservedGraphSnapshot, { render: false });
      this.syncRunManagerOverlaySeries();
    }
    this.lastExecPathHint = "처음 상태로 초기화했습니다. 실행 버튼으로 다시 시작할 수 있습니다.";
    this.syncRunMadiStatus();
    this.setEngineStatus("idle");
    this.updateRuntimeHint();
    return true;
  }

  async handleStepExecutionControl() {
    if (this.engineStatus === "running") return false;
    if (hasReachedRuntimeMaxMadi(this.runtimeTickCounter, this.resolveRuntimeMaxMadiLimit())) {
      this.executionPaused = true;
      this.haltLoop();
      this.lastExecPathHint = "실행 완료";
      this.setEngineStatus("done");
      this.updateRuntimeHint();
      return false;
    }
    if (!this.wasmState?.client || !this.lastState) {
      const restarted = await this.restart({ autoStartLive: false });
      if (!restarted) return false;
      this.executionPaused = true;
      this.haltLoop();
      if (hasReachedRuntimeMaxMadi(this.runtimeTickCounter, this.resolveRuntimeMaxMadiLimit())) {
        this.lastExecPathHint = "실행 완료";
        this.setEngineStatus("done");
        this.updateRuntimeHint();
        return false;
      }
    }
    this.executionPaused = false;
    this.stepFrame();
    this.executionPaused = true;
    this.haltLoop();
    if (this.engineStatus === "done" || hasReachedRuntimeMaxMadi(this.runtimeTickCounter, this.resolveRuntimeMaxMadiLimit())) {
      this.lastExecPathHint = "실행 완료";
      this.setEngineStatus("done");
      this.updateRuntimeHint();
      return true;
    }
    this.lastExecPathHint = "1틱 진행 후 일시정지했습니다.";
    this.setEngineStatus("paused");
    this.updateRuntimeHint();
    return true;
  }

  syncRunSliderAreaVisibility() {
    if (!this.runSliderAreaEl) return;
    const count = Array.isArray(this.sliderPanel?.specs) ? this.sliderPanel.specs.length : 0;
    this.runSliderAreaEl.classList.toggle("hidden", count <= 0);
  }

  syncGraphDraftFromParsed(parsed = {}) {
    const row = parsed && typeof parsed === "object" ? parsed : {};
    this.dotbogi?.setSeedKeys?.(Array.isArray(row.axisKeys) ? row.axisKeys : []);
    this.dotbogi?.setPreferredXKey?.(String(row.defaultXAxisKey ?? ""));
    this.dotbogi?.setPreferredYKey?.(String(row.defaultAxisKey ?? ""));
    if (this.graphRangeSelectEl) {
      const nextRange = String(this.dotbogi?.getMaxPointsMode?.() ?? this.graphRangeSelectEl.value ?? RUN_GRAPH_RANGE_RECENT_500).trim();
      this.graphRangeSelectEl.value = normalizeGraphRangeSelection(nextRange);
    }
    if (this.graphKindSelectEl) {
      const nextKind = String(this.dotbogi?.getGraphKind?.() ?? this.graphKindSelectEl.value ?? "line").trim() || "line";
      this.graphKindSelectEl.value = nextKind;
    }
  }

  renderOverlayTabContent(markdown = "", { sourceLabel = "" } = {}) {
    const text = String(markdown ?? "").trim();
    if (this.runOverlayMetaEl) {
      this.runOverlayMetaEl.textContent = text
        ? (String(sourceLabel ?? "").trim() || "설명 준비됨")
        : "설명 대기";
    }
    if (!this.runOverlayBodyEl) return;
    if (!text) {
      this.runOverlayBodyEl.innerHTML = '<div class="runtime-text-empty">설명 없음</div>';
      return;
    }
    this.runOverlayBodyEl.innerHTML = markdownToHtml(text);
  }

  hydrateRunViewPrefs() {
    try {
      const savedPrimary = String(window?.localStorage?.getItem(RUN_PRIMARY_VIEW_STORAGE_KEY) ?? "").trim();
      this.primaryView = normalizeRunPrimaryView(savedPrimary, "sim");
    } catch (_) {
      this.primaryView = "sim";
    }
    this.graphTabMode = resolveGraphTabMode(this.primaryView);
  }

  setPrimaryView(nextView = "sim", { persist = true, forceTab = false } = {}) {
    const resolved = normalizeRunPrimaryView(nextView, this.primaryView);
    this.primaryView = resolved;
    if (this.root?.dataset) {
      this.root.dataset.primaryView = resolved;
    }
    if (forceTab) {
      this.switchRunTab(this.activeRunTab || SUBPANEL_TAB.MAEGIM);
    }
    this.renderGraphTabMode();
    this.syncSubpanelTabLabels();
    this.syncRunSliderAreaVisibility();
    if (persist) {
      try {
        window?.localStorage?.setItem(RUN_PRIMARY_VIEW_STORAGE_KEY, resolved);
      } catch (_) {
        // ignore storage write failures
      }
    }
    return resolved;
  }

  hydrateStudioViewMode() {
    let saved = "";
    try {
      saved = String(window?.localStorage?.getItem(STUDIO_VIEW_MODE_STORAGE_KEY) ?? "").trim();
    } catch (_) {
      saved = "";
    }
    this.setStudioViewMode(normalizeStudioViewMode(saved, STUDIO_VIEW_MODE_BASIC), {
      persist: false,
      promoteDefaultTab: false,
    });
  }

  syncStudioViewModeUi() {
    const mode = normalizeStudioViewMode(this.studioViewMode, STUDIO_VIEW_MODE_BASIC);
    const layoutClassList = this.layoutEl?.classList;
    layoutClassList?.toggle?.("run-layout--studio-basic", mode === STUDIO_VIEW_MODE_BASIC);
    layoutClassList?.toggle?.("run-layout--studio-analyze", mode === STUDIO_VIEW_MODE_ANALYZE);
    layoutClassList?.toggle?.("run-layout--studio-full", mode === STUDIO_VIEW_MODE_FULL);
    if (this.root?.dataset) {
      this.root.dataset.studioViewMode = mode;
    }
    this.runViewModeButtons.forEach((button) => {
      const targetMode = normalizeStudioViewMode(button?.dataset?.runViewMode ?? "", STUDIO_VIEW_MODE_BASIC);
      button?.classList?.toggle?.("active", targetMode === mode);
      button?.setAttribute?.("aria-pressed", targetMode === mode ? "true" : "false");
    });
    const panelOpen = mode === STUDIO_VIEW_MODE_FULL ? this.fullModePanelOpen : true;
    if (this.subpanelEl?.dataset) {
      this.subpanelEl.dataset.panelOpen = panelOpen ? "1" : "0";
    }
    this.refreshStudioLayoutBounds({ persist: false });
    requestAnimationFrame(() => {
      this.dotbogi?.renderGraph?.();
      if (this.lastMainVisualMode === "console-grid") {
        const lines = Array.isArray(this.lastRuntimeDerived?.outputLines) ? this.lastRuntimeDerived.outputLines : [];
        this.bogae?.renderConsoleGrid?.(lines);
      } else {
        this.bogae?.render?.(this.lastRuntimeDerived?.mainVisualSpace2d ?? null);
      }
    });
  }

  setStudioViewMode(nextMode = STUDIO_VIEW_MODE_BASIC, { persist = true, promoteDefaultTab = true } = {}) {
    const mode = normalizeStudioViewMode(nextMode, this.studioViewMode);
    this.studioViewMode = mode;
    if (mode === STUDIO_VIEW_MODE_FULL) {
      this.fullModePanelOpen = false;
    } else {
      this.fullModePanelOpen = true;
    }
    if (mode === STUDIO_VIEW_MODE_ANALYZE && promoteDefaultTab) {
      this.switchRunTab(SUBPANEL_TAB.MAEGIM);
    }
    this.syncStudioViewModeUi();
    if (persist) {
      try {
        window?.localStorage?.setItem(STUDIO_VIEW_MODE_STORAGE_KEY, mode);
      } catch (_) {
        // ignore storage write failures
      }
    }
    return mode;
  }

  setMinimalUi(enabled = true, { persist = true } = {}) {
    this.minimalUi = true;
    return true;
  }

  readCurrentRunLayoutEditorRatio() {
    if (!this.layoutEl) return STUDIO_EDITOR_RATIO_DEFAULT;
    const inlineRaw = String(this.layoutEl.style?.getPropertyValue?.("--editor-ratio") ?? "").trim();
    const inlineNum = Number(inlineRaw);
    if (Number.isFinite(inlineNum)) return inlineNum;
    try {
      const computedRaw = String(window?.getComputedStyle?.(this.layoutEl)?.getPropertyValue?.("--editor-ratio") ?? "").trim();
      const computedNum = Number(computedRaw);
      if (Number.isFinite(computedNum)) return computedNum;
    } catch (_) {
      // ignore computed style read failures
    }
    return STUDIO_EDITOR_RATIO_DEFAULT;
  }

  resolveStudioLayoutBoundsForCurrentViewport() {
    const layoutRect = this.layoutEl?.getBoundingClientRect?.() ?? null;
    const splitterRect = this.runLayoutSplitterEl?.getBoundingClientRect?.() ?? null;
    const bannerVisible = !this.runErrorBannerEl?.classList?.contains?.("hidden");
    const bannerRect = bannerVisible ? this.runErrorBannerEl?.getBoundingClientRect?.() ?? null : null;
    return resolveStudioLayoutBounds({
      layoutWidth: Number(layoutRect?.width ?? 0),
      layoutHeight: Number(layoutRect?.height ?? 0),
      splitterWidth: Number(splitterRect?.width ?? STUDIO_LAYOUT_SPLITTER_WIDTH),
      toolbarHeight: 0,
      errorBannerHeight: Number(bannerRect?.height ?? 0),
      minVisualWidth: STUDIO_LAYOUT_MIN_VISUAL_WIDTH_PX,
      subpanelMinHeight: STUDIO_LAYOUT_SUBPANEL_MIN_HEIGHT_PX,
      bogaeAspectRatio: STUDIO_LAYOUT_BOGAE_ASPECT_RATIO,
      baseMinEditorRatio: STUDIO_EDITOR_RATIO_MIN,
      baseMaxEditorRatio: STUDIO_EDITOR_RATIO_MAX,
    });
  }

  applyStudioLayoutBounds(bounds = null) {
    if (!this.layoutEl) return;
    if (this.studioViewMode !== STUDIO_VIEW_MODE_BASIC) {
      if (this.layoutEl.style && typeof this.layoutEl.style.setProperty === "function") {
        this.layoutEl.style.setProperty("--bogae-frame-max-width", "100%");
        this.layoutEl.style.setProperty("--bogae-frame-max-height", "none");
      }
      this.runVisualColumnEl?.classList?.remove?.("run-visual-column--scroll-fallback");
      this.syncBogaeToolbarCompactState({ refreshControls: true });
      return;
    }
    const next = bounds && typeof bounds === "object" ? bounds : this.resolveStudioLayoutBoundsForCurrentViewport();
    this.lastStudioLayoutBounds = next;
    const maxWidth = Number(next?.bogaeFrameMaxWidthPx ?? 0);
    const maxHeight = Number(next?.bogaeFrameMaxHeightPx ?? 0);
    if (this.layoutEl.style && typeof this.layoutEl.style.setProperty === "function") {
      if (Number.isFinite(maxWidth) && maxWidth > 0) {
        this.layoutEl.style.setProperty("--bogae-frame-max-width", `${maxWidth}px`);
      } else {
        this.layoutEl.style.setProperty("--bogae-frame-max-width", "100%");
      }
      if (Number.isFinite(maxHeight) && maxHeight > 0) {
        this.layoutEl.style.setProperty("--bogae-frame-max-height", `${maxHeight}px`);
      } else {
        this.layoutEl.style.setProperty("--bogae-frame-max-height", "none");
      }
    }
    this.runVisualColumnEl?.classList?.toggle?.(
      "run-visual-column--scroll-fallback",
      Boolean(next?.hasConstraintOverflow),
    );
    if (this.root?.dataset) {
      this.root.dataset.layoutConstraintOverflow = next?.hasConstraintOverflow ? "1" : "0";
    }
    this.syncBogaeToolbarCompactState({ refreshControls: true });
  }

  refreshStudioLayoutBounds({ persist = false } = {}) {
    const ratio = this.readCurrentRunLayoutEditorRatio();
    return this.setRunLayoutEditorRatio(ratio, { persist });
  }

  setRunLayoutEditorRatio(ratio, { persist = true } = {}) {
    if (!this.layoutEl) return STUDIO_EDITOR_RATIO_DEFAULT;
    const bounds = this.resolveStudioLayoutBoundsForCurrentViewport();
    const normalized = normalizeEditorRatio(ratio, {
      min: bounds.editorRatioMin,
      max: bounds.editorRatioMax,
      fallback: STUDIO_EDITOR_RATIO_DEFAULT,
    });
    if (this.layoutEl.style && typeof this.layoutEl.style.setProperty === "function") {
      this.layoutEl.style.setProperty("--editor-ratio", String(normalized));
    }
    this.applyStudioLayoutBounds(bounds);
    if (persist) {
      try {
        window?.localStorage?.setItem(STUDIO_EDITOR_RATIO_STORAGE_KEY, String(normalized));
      } catch (_) {
        // ignore storage write failures
      }
    }
    return normalized;
  }

  syncBogaeToolbarCompactState({ refreshControls = false } = {}) {
    const compact = resolveBogaeToolbarCompact({
      toolbarWidth: Number(this.bogaeToolbarEl?.getBoundingClientRect?.()?.width ?? 0),
      visualColumnWidth: Number(this.runVisualColumnEl?.getBoundingClientRect?.()?.width ?? 0),
    });
    const prev = Boolean(this.lastBogaeToolbarCompact);
    this.lastBogaeToolbarCompact = compact;
    this.runControlBarEl?.classList?.toggle?.("run-control-bar--compact", compact);
    if (refreshControls && prev !== compact) {
      this.syncRunControlState();
    }
    return compact;
  }

  syncInitialBogaeShellVisibility(preserve = true) {
    this.layoutEl?.classList?.toggle?.("run-layout--keep-bogae-shell", Boolean(preserve));
    this.refreshStudioLayoutBounds({ persist: false });
  }

  syncBogaeShellForMainVisual(mode = "none") {
    const preserve = normalizeRunMainVisualMode(mode, "none") !== "none";
    if (preserve) {
      this.syncInitialBogaeShellVisibility(true);
      return;
    }
    this.syncInitialBogaeShellVisibility(false);
  }

  normalizeRunRequest(request = {}) {
    const row = request && typeof request === "object" ? request : {};
    const sourceText = String(row.sourceText ?? this.baseDdn ?? this.runDdnPreviewEl?.value ?? "");
    const sourceHash = String(row.sourceHash ?? "").trim() || hashRunSourceText(sourceText);
    const createdAtMs = Number.isFinite(Number(row.createdAtMs)) ? Number(row.createdAtMs) : Date.now();
    const launchKind = String(row.launchKind ?? this.lastLaunchKind ?? "manual").trim() || "manual";
    const sourceType = String(row.sourceType ?? this.sourceKind ?? "lesson").trim() || "lesson";
    this.runRequestSequence += 1;
    const id = String(row.id ?? "").trim() || `run:${createdAtMs}:${this.runRequestSequence}:${sourceHash}`;
    return { id, sourceText, sourceHash, launchKind, sourceType, createdAtMs };
  }

  enqueueRunRequest(request = {}) {
    const next = this.normalizeRunRequest(request);
    if (this.activeRunRequestId === next.id || this.completedRunRequestId === next.id) return next;
    if (this.pendingRunRequest?.id === next.id) return this.pendingRunRequest;
    this.pendingRunRequest = next;
    this.consumePendingRunRequest();
    return next;
  }

  requestAutoExecute() {
    return Boolean(this.enqueueRunRequest({
      sourceText: this.baseDdn,
      launchKind: this.lastLaunchKind,
      sourceType: this.sourceKind,
    }));
  }

  clearPendingAutoExecute() {
    this.pendingRunRequest = null;
  }

  cancelLiveReplRestart() {
    if (this.liveReplTimer) {
      clearTimeout(this.liveReplTimer);
      this.liveReplTimer = null;
    }
  }

  scheduleLiveReplRestart(sourceText = "") {
    this.cancelLiveReplRestart();
    if (!this.screenVisible || !this.lesson) return false;
    const text = String(sourceText ?? this.baseDdn ?? this.runDdnPreviewEl?.value ?? "");
    if (!text.trim()) return false;
    const sequence = this.liveReplSequence + 1;
    this.liveReplSequence = sequence;
    this.liveReplTimer = setTimeout(() => {
      this.liveReplTimer = null;
      if (sequence !== this.liveReplSequence) return;
      const latest = String(this.runDdnPreviewEl?.value ?? this.baseDdn ?? text);
      this.enqueueRunRequest({
        id: `live-repl:${Date.now()}:${sequence}:${hashRunSourceText(latest)}`,
        sourceText: latest,
        launchKind: "live_repl",
        sourceType: this.sourceKind || "lesson",
        createdAtMs: Date.now(),
      });
    }, this.liveReplDebounceMs);
    return true;
  }

  consumePendingRunRequest() {
    if (!this.pendingRunRequest || !this.screenVisible || !this.lesson) {
      return false;
    }
    const request = this.pendingRunRequest;
    this.pendingRunRequest = null;
    void this.executeRunRequest(request);
    return true;
  }

  consumePendingAutoExecute() {
    return this.consumePendingRunRequest();
  }

  hydrateRunLayoutRatio() {
    let savedRaw = "";
    try {
      savedRaw = String(window?.localStorage?.getItem(STUDIO_EDITOR_RATIO_STORAGE_KEY) ?? "").trim();
    } catch (_) {
      savedRaw = "";
    }
    const normalized = this.setRunLayoutEditorRatio(
      savedRaw ? Number(savedRaw) : STUDIO_EDITOR_RATIO_DEFAULT,
      { persist: false },
    );
    try {
      window?.localStorage?.setItem(STUDIO_EDITOR_RATIO_STORAGE_KEY, String(normalized));
    } catch (_) {
      // ignore storage write failures
    }
  }

  bindRunLayoutSplitter() {
    const splitter = this.runLayoutSplitterEl;
    const layout = this.layoutEl;
    if (!splitter || !layout || !splitter.addEventListener) return;
    let dragging = false;

    const onPointerMove = (event) => {
      if (!dragging) return;
      const rect = layout.getBoundingClientRect?.();
      const width = Number(rect?.width ?? 0);
      if (!Number.isFinite(width) || width <= 0) return;
      const left = Number(rect?.left ?? 0);
      const nextRatio = (Number(event?.clientX ?? 0) - left) / width;
      this.setRunLayoutEditorRatio(nextRatio, { persist: false });
    };

    const onPointerUp = () => {
      if (!dragging) return;
      dragging = false;
      const normalized = this.readCurrentRunLayoutEditorRatio();
      this.setRunLayoutEditorRatio(normalized, { persist: true });
      splitter.classList?.remove?.("is-dragging");
      try {
        window?.removeEventListener?.("pointermove", onPointerMove);
        window?.removeEventListener?.("pointerup", onPointerUp);
      } catch (_) {
        // ignore listener cleanup failures
      }
    };

    splitter.addEventListener("pointerdown", (event) => {
      dragging = true;
      splitter.classList?.add?.("is-dragging");
      try {
        splitter.setPointerCapture?.(event.pointerId);
      } catch (_) {
        // ignore capture failures
      }
      event.preventDefault();
      window?.addEventListener?.("pointermove", onPointerMove);
      window?.addEventListener?.("pointerup", onPointerUp);
    });
  }

  installStudioLayoutResizeObserver() {
    if (this.studioLayoutResizeObserver && typeof this.studioLayoutResizeObserver.disconnect === "function") {
      this.studioLayoutResizeObserver.disconnect();
    }
    this.studioLayoutResizeObserver = null;
    if (this.studioLayoutResizeFallbackInstalled) {
      window?.removeEventListener?.("resize", this.boundStudioLayoutResizeHandler);
      this.studioLayoutResizeFallbackInstalled = false;
    }
    if (!this.layoutEl) return;
    if (typeof globalThis?.ResizeObserver !== "function") {
      window?.addEventListener?.("resize", this.boundStudioLayoutResizeHandler);
      this.studioLayoutResizeFallbackInstalled = true;
      return;
    }
    this.studioLayoutResizeObserver = new globalThis.ResizeObserver(() => {
      this.refreshStudioLayoutBounds({ persist: false });
    });
    this.studioLayoutResizeObserver.observe(this.layoutEl);
    if (this.runVisualColumnEl) {
      this.studioLayoutResizeObserver.observe(this.runVisualColumnEl);
    }
  }

  ensureRunManagerUi() {
    if (typeof document === "undefined") return;
    const existing = this.root.querySelector("#run-manager-panel");
    if (existing) return;
    const panel = document.createElement("div");
    panel.id = "run-manager-panel";
    panel.className = "run-manager-panel";
    panel.innerHTML = `
      <div class="run-manager-head">
        <div class="run-manager-head-copy">
          <span class="run-manager-title">실행 비교</span>
          <span class="run-manager-description">이전 실행 결과를 현재 그래프와 비교합니다.</span>
        </div>
        <div class="run-manager-head-actions">
          <button id="btn-run-manager-prune" type="button">숨김 정리</button>
          <button id="btn-run-manager-clear" type="button">전체 삭제</button>
        </div>
      </div>
      <div class="run-history-comparison-rail" data-run-history-comparison-rail aria-label="실행 비교 상태">
        <span class="run-history-chip" data-run-history-count>저장 run 0</span>
        <span class="run-history-chip" data-run-history-visible>표시 0</span>
        <span class="run-history-chip" data-run-history-active>활성 없음</span>
        <span class="run-history-chip" data-run-history-solo>solo 없음</span>
      </div>
      <div class="run-history-export-summary" data-run-history-export-summary aria-label="실행 이력 요약 내보내기">
        <div class="run-history-export-summary-head">
          <span class="run-history-export-summary-title">이력 요약</span>
          <span class="run-history-export-summary-meta" data-run-history-export-summary-meta>0개 run</span>
          <button id="btn-run-history-export-summary-copy" type="button" disabled>요약 복사</button>
        </div>
        <pre class="run-history-export-summary-text" data-run-history-export-summary-text>Seamgrim run history export summary
runs: 0</pre>
      </div>
      <div id="run-manager-list" class="run-manager-list"></div>
    `;
    const anchor = this.root.querySelector("#run-manager-panel-anchor");
    if (anchor?.insertAdjacentElement) {
      anchor.insertAdjacentElement("afterend", panel);
      return;
    }
    this.graphPanelEl?.appendChild?.(panel);
  }

  buildRunManagerBasePayload() {
    const base = this.getOverlaySession && typeof this.getOverlaySession === "function"
      ? this.getOverlaySession() ?? {}
      : {};
    return toPlainObject(base, {});
  }

  runManagerSourceMatchesCurrent(row = {}) {
    const source = toPlainObject(row?.source, {});
    const lessonId = String(this.lesson?.id ?? "").trim();
    const sourceLessonId = String(source.lessonId ?? source.lesson_id ?? "").trim();
    if (lessonId) {
      if (!sourceLessonId || sourceLessonId !== lessonId) return false;
    }
    const sourceText = String(source.text ?? source.ddnText ?? source.ddn_text ?? "").replace(/\r\n/g, "\n").trim();
    const currentText = String(this.baseDdn ?? this.lesson?.ddnText ?? "").replace(/\r\n/g, "\n").trim();
    if (sourceText && currentText && sourceText !== currentText) return false;
    return true;
  }

  runManagerRunRestorable(row = {}) {
    if (!this.runManagerSourceMatchesCurrent(row)) return false;
    return isRunManagerGraphRunMeaningful(row);
  }

  normalizeRunManagerRun(raw, index = 0) {
    const row = toPlainObject(raw, {});
    const id = String(row.id ?? `run-${index + 1}`).trim() || `run-${index + 1}`;
    const label = String(row.label ?? id).trim() || id;
    const source = toPlainObject(row.source, {});
    const inputs = toPlainObject(row.inputs, {});
    const hashRaw = toPlainObject(row.hash, {});
    const graph = cloneGraphForRunManager(row.graph ?? row.result ?? null);
    const hashInput = String(hashRaw.input ?? graph?.meta?.source_input_hash ?? "").trim();
    const hashResult = String(hashRaw.result ?? graph?.meta?.result_hash ?? "").trim();
    const hue = stableColorHue(label, hashResult || hashInput || id);
    return {
      id,
      label,
      visible: row.visible !== false,
      layerIndex: normalizeRunManagerLayer(row.layerIndex ?? row.layer_index ?? row.order, index),
      source,
      inputs,
      graph,
      result: graph,
      hash: {
        input: hashInput,
        result: hashResult,
      },
      hue,
    };
  }

  serializeRunManagerRun(run) {
    const row = toPlainObject(run, {});
    return {
      id: String(row.id ?? "").trim(),
      label: String(row.label ?? "").trim(),
      visible: row.visible !== false,
      layer_index: normalizeRunManagerLayer(row.layerIndex ?? row.layer_index, 0),
      source: toPlainObject(row.source, {}),
      inputs: toPlainObject(row.inputs, {}),
      graph: cloneGraphForRunManager(row.graph ?? null),
      result: cloneGraphForRunManager(row.graph ?? null),
      hash: {
        input: String(row?.hash?.input ?? "").trim(),
        result: String(row?.hash?.result ?? "").trim(),
      },
    };
  }

  getCurrentLessonRunLabel() {
    const lessonLabel = String(this.lesson?.title ?? this.lesson?.id ?? "run").trim() || "run";
    this.runManagerSequence += 1;
    return `${lessonLabel} #${this.runManagerSequence}`;
  }

  getNextRunLayerIndex() {
    if (!Array.isArray(this.overlayRuns) || !this.overlayRuns.length) return 0;
    return Math.max(...this.overlayRuns.map((row) => normalizeRunManagerLayer(row?.layerIndex, 0))) + 1;
  }

  buildCurrentGraphViewPayload() {
    const axis = this.dotbogi?.getCurrentAxis?.() ?? null;
    return normalizeGraphView({
      auto: false,
      x_min: axis?.x_min,
      x_max: axis?.x_max,
      y_min: axis?.y_min,
      y_max: axis?.y_max,
      pan_x: 0,
      pan_y: 0,
      zoom: 1,
    });
  }

  beginLiveRunCapture(ddnText = "") {
    const lessonId = String(this.lesson?.id ?? "").trim();
    const inputValues = this.sliderPanel?.getValues?.() ?? {};
    const inputHash = buildRunInputHash({
      ddnText,
      controls: inputValues,
      sample: null,
    });
    const id = `run:${lessonId || "custom"}:${Date.now().toString(36)}:${this.runManagerSequence + 1}`;
    const label = this.getCurrentLessonRunLabel();
    const run = this.normalizeRunManagerRun(
      {
        id,
        label,
        visible: true,
        layer_index: this.getNextRunLayerIndex(),
        source: {
          kind: "ddn",
          lessonId,
          launchKind: this.lastLaunchKind,
          text: String(ddnText ?? ""),
        },
        inputs: {
          controls: toPlainObject(inputValues, {}),
        },
        hash: {
          input: inputHash,
          result: "",
        },
      },
      this.overlayRuns.length,
    );
    this.overlayRuns.push(run);
    if (this.overlayRuns.length > RUN_MANAGER_MAX_RUNS) {
      this.overlayRuns = this.overlayRuns.slice(this.overlayRuns.length - RUN_MANAGER_MAX_RUNS);
    }
    this.activeOverlayRunId = run.id;
    this.soloOverlayRunId = "";
    this.hoverOverlayRunId = "";
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
    this.publishRunManagerSession();
    return run.id;
  }

  findRunManagerIndexById(runId) {
    const target = String(runId ?? "").trim();
    if (!target) return -1;
    return this.overlayRuns.findIndex((row) => String(row?.id ?? "").trim() === target);
  }

  updateLiveRunCaptureFromDerived(derived, { fallbackGraph = null } = {}) {
    if (this.suspendLiveRunCapture) return;
    const activeId = String(this.activeOverlayRunId ?? "").trim();
    if (!activeId) return;
    const index = this.findRunManagerIndexById(activeId);
    if (index < 0) return;
    const current = this.overlayRuns[index];
    const graph = resolveLiveRunCaptureGraph({
      runtimeGraph: derived?.views?.graph ?? null,
      runtimeGraphSource: derived?.views?.graphSource ?? derived?.graphSource ?? "",
      fallbackGraph,
    });
    if (!graph) return;
    this.lastGraphSnapshot = cloneGraphForRunManager(graph);
    const currentGraphView = this.buildCurrentGraphViewPayload();
    if (currentGraphView) {
      graph.view = currentGraphView;
    }
    const sample = normalizeGraphSample(graph.sample ?? null);
    const controls = toPlainObject(this.sliderPanel?.getValues?.(), {});
    const hashInput = buildRunInputHash({
      ddnText: String(current?.source?.text ?? this.currentDdn ?? ""),
      controls,
      sample,
    });
    const hashResult = String(this.lastRuntimeHash ?? "").trim();
    const next = this.normalizeRunManagerRun(
      {
        ...current,
        graph,
        result: graph,
        inputs: {
          ...(toPlainObject(current?.inputs, {})),
          controls,
          ...(sample ? { sample } : {}),
        },
        hash: {
          ...(toPlainObject(current?.hash, {})),
          input: hashInput,
          result: hashResult,
        },
      },
      index,
    );
    this.overlayRuns[index] = next;
    this.syncRunManagerOverlaySeries();
  }

  discardActiveRunCaptureIfEmpty() {
    const activeId = String(this.activeOverlayRunId ?? "").trim();
    if (!activeId) return;
    const index = this.findRunManagerIndexById(activeId);
    if (index < 0) return;
    const run = this.overlayRuns[index];
    const hasGraph = Boolean(run?.graph && Array.isArray(run.graph.series) && run.graph.series.length > 0);
    if (hasGraph) return;
    this.overlayRuns.splice(index, 1);
    this.activeOverlayRunId = "";
    if (this.soloOverlayRunId === activeId) this.soloOverlayRunId = "";
    if (this.hoverOverlayRunId === activeId) this.hoverOverlayRunId = "";
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
    this.publishRunManagerSession();
  }

  clearRunManagerRuns() {
    this.overlayRuns = [];
    this.activeOverlayRunId = "";
    this.hoverOverlayRunId = "";
    this.soloOverlayRunId = "";
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
    this.publishRunManagerSession();
  }

  pruneHiddenRunManagerRuns() {
    const next = this.overlayRuns.filter((row) => row.visible !== false);
    if (next.length === this.overlayRuns.length) return;
    this.overlayRuns = next;
    if (this.activeOverlayRunId && this.findRunManagerIndexById(this.activeOverlayRunId) < 0) {
      this.activeOverlayRunId = "";
    }
    if (this.hoverOverlayRunId && this.findRunManagerIndexById(this.hoverOverlayRunId) < 0) {
      this.hoverOverlayRunId = "";
    }
    if (this.soloOverlayRunId && this.findRunManagerIndexById(this.soloOverlayRunId) < 0) {
      this.soloOverlayRunId = "";
    }
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
    this.publishRunManagerSession();
  }

  removeRunManagerRun(runId) {
    const target = String(runId ?? "").trim();
    if (!target) return;
    const index = this.findRunManagerIndexById(target);
    if (index < 0) return;
    this.overlayRuns.splice(index, 1);
    if (this.activeOverlayRunId === target) this.activeOverlayRunId = "";
    if (this.hoverOverlayRunId === target) this.hoverOverlayRunId = "";
    if (this.soloOverlayRunId === target) this.soloOverlayRunId = "";
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
    this.publishRunManagerSession();
  }

  setRunManagerHover(runId = "") {
    const nextId = String(runId ?? "").trim();
    if (this.hoverOverlayRunId === nextId) return;
    this.hoverOverlayRunId = nextId;
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
  }

  toggleRunManagerSolo(runId) {
    const target = String(runId ?? "").trim();
    if (!target) return;
    this.soloOverlayRunId = this.soloOverlayRunId === target ? "" : target;
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
  }

  syncAdvancedExportVisibility() {
    this.advancedExportsEnabled = INITIAL_ADVANCED_EXPORTS_ENABLED || shouldEnableAdvancedExports();
    // Temporary compatibility: retire these advanced export panels from the teacher UI after legacy runner coverage is moved.
    [
      this.runPublicationPrepEl,
      this.runRegistrySeedEl,
      this.runApprovalContinuityEl,
      this.runBenchmarkLtsEl,
      this.runEducationOperationsLtsEl,
    ].forEach((element) => {
      if (!element) return;
      element.classList.toggle("hidden", !this.advancedExportsEnabled);
      element.setAttribute("aria-hidden", this.advancedExportsEnabled ? "false" : "true");
    });
  }

  buildRunHistoryComparisonRailModel() {
    const runs = Array.isArray(this.overlayRuns) ? this.overlayRuns : [];
    const activeId = String(this.activeOverlayRunId ?? "").trim();
    const soloId = String(this.soloOverlayRunId ?? "").trim();
    const activeRun = activeId ? runs.find((row) => String(row?.id ?? "").trim() === activeId) ?? null : null;
    const soloRun = soloId ? runs.find((row) => String(row?.id ?? "").trim() === soloId) ?? null : null;
    const visibleCount = runs.filter((row) => row?.visible !== false).length;
    return {
      schema: "seamgrim.run_history_comparison_rail.v1",
      run_count: runs.length,
      visible_count: visibleCount,
      hidden_count: Math.max(0, runs.length - visibleCount),
      active_run_id: activeRun ? activeId : "",
      active_label: activeRun ? String(activeRun.label ?? activeId).trim() : "",
      solo_run_id: soloRun ? soloId : "",
      solo_label: soloRun ? String(soloRun.label ?? soloId).trim() : "",
      has_active: Boolean(activeRun),
      has_solo: Boolean(soloRun),
    };
  }

  buildRunHistoryExportSummaryModel() {
    const runs = Array.isArray(this.overlayRuns) ? this.overlayRuns : [];
    const comparison = this.buildRunHistoryComparisonRailModel();
    const rows = buildRunHistoryExportSummaryRows(runs);
    const latest = rows.length ? rows[rows.length - 1] : null;
    const model = {
      schema: "seamgrim.run_history_export_summary.v1",
      generated_by: "SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_V1",
      run_count: comparison.run_count,
      visible_count: comparison.visible_count,
      hidden_count: comparison.hidden_count,
      active_run_id: comparison.active_run_id,
      active_label: comparison.active_label,
      solo_run_id: comparison.solo_run_id,
      solo_label: comparison.solo_label,
      latest_run_id: latest?.id ?? "",
      latest_label: latest?.label ?? "",
      runs: rows,
    };
    return {
      ...model,
      text: formatRunHistoryExportSummaryText(model),
    };
  }

  syncRunHistoryExportSummary() {
    const model = this.buildRunHistoryExportSummaryModel();
    const hasRuns = model.run_count > 0;
    if (this.runHistoryExportSummaryEl) {
      this.runHistoryExportSummaryEl.dataset.schema = model.schema;
      this.runHistoryExportSummaryEl.dataset.state = hasRuns ? "ready" : "empty";
    }
    if (this.runHistoryExportSummaryMetaEl) {
      this.runHistoryExportSummaryMetaEl.textContent = `${model.run_count}개 run · 표시 ${model.visible_count}`;
      this.runHistoryExportSummaryMetaEl.dataset.value = String(model.run_count);
      this.runHistoryExportSummaryMetaEl.dataset.visible = String(model.visible_count);
    }
    if (this.runHistoryExportSummaryTextEl) {
      this.runHistoryExportSummaryTextEl.textContent = model.text;
    }
    if (this.runHistoryExportSummaryCopyBtn) {
      this.runHistoryExportSummaryCopyBtn.disabled = !hasRuns;
      this.runHistoryExportSummaryCopyBtn.title = hasRuns ? "실행 이력 요약 복사" : "복사할 실행 이력이 없습니다.";
    }
    if (typeof window !== "undefined") {
      window.__SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY__ = model;
    }
    return model;
  }

  syncRunHistoryComparisonRail() {
    const model = this.buildRunHistoryComparisonRailModel();
    if (this.runHistoryCountEl) {
      this.runHistoryCountEl.textContent = `저장 run ${model.run_count}`;
      this.runHistoryCountEl.dataset.value = String(model.run_count);
    }
    if (this.runHistoryVisibleEl) {
      this.runHistoryVisibleEl.textContent = `표시 ${model.visible_count}`;
      this.runHistoryVisibleEl.dataset.value = String(model.visible_count);
    }
    if (this.runHistoryActiveEl) {
      this.runHistoryActiveEl.textContent = model.active_label ? `활성 ${model.active_label}` : "활성 없음";
      this.runHistoryActiveEl.dataset.value = model.active_run_id;
      this.runHistoryActiveEl.dataset.state = model.has_active ? "active" : "none";
    }
    if (this.runHistorySoloEl) {
      this.runHistorySoloEl.textContent = model.solo_label ? `solo ${model.solo_label}` : "solo 없음";
      this.runHistorySoloEl.dataset.value = model.solo_run_id;
      this.runHistorySoloEl.dataset.state = model.has_solo ? "solo" : "none";
    }
    if (this.runHistoryComparisonRailEl) {
      this.runHistoryComparisonRailEl.dataset.schema = model.schema;
      this.runHistoryComparisonRailEl.dataset.state = model.run_count > 0 ? "ready" : "empty";
    }
    if (typeof window !== "undefined") {
      window.__SEAMGRIM_RUN_HISTORY_COMPARISON_RAIL__ = model;
    }
    this.syncRunHistoryExportSummary();
    return model;
  }

  async handleCopyRunHistoryExportSummary() {
    const model = this.syncRunHistoryExportSummary();
    const value = String(model?.text ?? "").trim();
    if (!value || model.run_count <= 0) {
      showGlobalToast("복사할 실행 이력이 없습니다.", { kind: "error" });
      return false;
    }
    let ok = false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(value);
        ok = true;
      }
    } catch (_) {
      ok = false;
    }
    showGlobalToast(ok ? "실행 이력 요약을 복사했습니다." : "실행 이력 요약 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  buildClassroomReportExportModel() {
    const lessonId = String(this.lesson?.id ?? "").trim() || "current_lesson";
    const title = String(this.lesson?.title ?? this.lesson?.id ?? "현재 수업").trim() || "현재 수업";
    const stateHash = String(this.lastRuntimeHash ?? "").trim();
    const hasRun = Boolean(stateHash && stateHash !== "-");
    const profile = normalizeRunOnboardingProfile(this.lastOnboardingProfile);
    const requiredViews = resolveLessonRequiredViewsForRun(this.lesson);
    const goals = Array.isArray(this.lesson?.goals)
      ? this.lesson.goals.map((item) => String(item ?? "").trim()).filter(Boolean)
      : [];
    const missions = Array.isArray(this.lesson?.missions)
      ? this.lesson.missions.map((item) => String(item ?? "").trim()).filter(Boolean)
      : [];
    const assignmentList = buildClassroomAssignmentList([
      {
        assignment_id: lessonId,
        title,
        lesson_id: lessonId,
        goals,
        missions,
        result_views: requiredViews,
        result_views_label: formatRunRequiredViewsLabel(requiredViews).replace(/^결과 확인:\s*/, ""),
        due_label: "현재 run",
        status: "open",
      },
    ]);
    const suiteCheck = hasRun
      ? {
          __이음관계종류: "studio_classroom_mode_switch_run_state",
          판정: "통과",
          전체통과: true,
          개수: 1,
          통과개수: 1,
          실패개수: 0,
          실패케이스들: [],
          기대실패통과케이스들: [],
          기대통과실패케이스들: [],
        }
      : null;
    const summary = buildClassroomRunResultSummary({
      assignment: assignmentList.assignments[0],
      runResult: hasRun ? { exit_code: 0, stdout: [stateHash], stderr: [] } : null,
      suiteCheck,
    });
    const report = buildClassroomExportReport({
      assignmentList,
      resultSummaries: [summary],
    });
    const text = formatClassroomExportReportText(report);
    return {
      schema: "seamgrim.classroom_report_export_action.v1",
      generated_by: "STUDIO_CLASSROOM_REPORT_EXPORT_ACTION_V1",
      mode: profile || "default",
      mode_label: formatRunOnboardingProfileLabel(profile),
      lesson_id: lessonId,
      lesson_title: title,
      state_hash: hasRun ? stateHash : "",
      has_run: hasRun,
      assignment_count: report.assignment_count,
      summary_count: report.summary_count,
      pass_count: report.pass_count,
      fail_count: report.fail_count,
      account_required: false,
      cloud_sync: false,
      permission_system: false,
      report,
      text,
    };
  }

  isTeacherExportMode() {
    return normalizeRunOnboardingProfile(this.lastOnboardingProfile) === "teacher";
  }

  syncTeacherExportVisibility() {
    const visible = this.isTeacherExportMode();
    [
      this.runClassroomReportEl,
      this.runLocalPackageEl,
    ].forEach((node) => {
      if (!node) return;
      node.classList.toggle("hidden", !visible);
      node.setAttribute("aria-hidden", visible ? "false" : "true");
      node.dataset.visible = visible ? "1" : "0";
    });
    [
      this.runTeacherReportCopyBtn,
      this.runTeacherPackageCopyBtn,
      this.runTeacherPackageDownloadBtn,
    ].forEach((button) => {
      if (!button) return;
      button.classList.toggle("hidden", !visible);
      button.setAttribute("aria-hidden", visible ? "false" : "true");
    });
    return visible;
  }

  syncClassroomReportExport() {
    const model = this.buildClassroomReportExportModel();
    const teacherVisible = this.syncTeacherExportVisibility();
    if (this.runClassroomReportEl) {
      this.runClassroomReportEl.dataset.schema = model.schema;
      this.runClassroomReportEl.dataset.state = model.has_run ? "ready" : "draft";
      this.runClassroomReportEl.dataset.mode = model.mode;
    }
    if (this.runClassroomReportMetaEl) {
      this.runClassroomReportMetaEl.textContent = `${model.mode_label} · ${model.has_run ? "run 있음" : "run 전"}`;
      this.runClassroomReportMetaEl.dataset.value = String(model.summary_count);
    }
    if (this.runClassroomReportTextEl) {
      this.runClassroomReportTextEl.textContent = model.text;
    }
    if (this.runClassroomReportCopyBtn) {
      this.runClassroomReportCopyBtn.disabled = !teacherVisible || !model.text;
    }
    if (this.runTeacherReportCopyBtn) {
      this.runTeacherReportCopyBtn.disabled = !teacherVisible || !model.text;
      this.runTeacherReportCopyBtn.dataset.summaryCount = String(model.summary_count);
      this.runTeacherReportCopyBtn.textContent = model.summary_count > 0
        ? `리포트 복사 ${model.summary_count}`
        : "리포트 복사";
      this.runTeacherReportCopyBtn.title = model.text
        ? `수업 리포트 ${model.summary_count}건 복사`
        : "복사할 수업 리포트가 없습니다.";
    }
    try {
      window.__STUDIO_CLASSROOM_REPORT_EXPORT_ACTION__ = model;
    } catch (_) {
      // ignore browser instrumentation errors
    }
    return model;
  }

  async handleCopyClassroomReportExport() {
    const model = this.syncClassroomReportExport();
    const value = String(model?.text ?? "").trim();
    if (!value) {
      showGlobalToast("복사할 수업 리포트가 없습니다.", { kind: "error" });
      return false;
    }
    let ok = false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(value);
        ok = true;
      }
    } catch (_) {
      ok = false;
    }
    try {
      window.__STUDIO_CLASSROOM_REPORT_EXPORT_ACTION__ = {
        ...model,
        copied: ok,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "수업 리포트를 복사했습니다." : "수업 리포트 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  buildLocalPackageExportModel() {
    const reportModel = this.buildClassroomReportExportModel();
    const importedPackagePayload = this.lesson?.localPackagePayload;
    if (
      importedPackagePayload
      && typeof importedPackagePayload === "object"
      && importedPackagePayload.__종류 === "studio_local_package_payload"
      && String(this.lesson?.localPackageId ?? "").trim()
    ) {
      const manifest = importedPackagePayload.manifest && typeof importedPackagePayload.manifest === "object"
        ? importedPackagePayload.manifest
        : this.lesson.localPackageManifest ?? {};
      const lessonId = String(this.lesson?.localPackageLessonId ?? this.lesson?.id ?? reportModel.lesson_id ?? "").trim() || "current_lesson";
      const title = String(this.lesson?.title ?? reportModel.lesson_title ?? lessonId).trim() || lessonId;
      const payloadText = JSON.stringify(importedPackagePayload, null, 2);
      const lessonScope = buildLocalPackageLessonScope(importedPackagePayload, { fallbackLessonId: lessonId });
      return {
        schema: "seamgrim.local_package_export_action.v1",
        generated_by: "STUDIO_LOCAL_PACKAGE_EXPORT_ACTION_V1",
        mode: reportModel.mode,
        mode_label: reportModel.mode_label,
        lesson_id: lessonId,
        lesson_title: title,
        session_label: String(manifest.session_label ?? this.lesson?.localPackageSessionLabel ?? "").trim(),
        package_id: String(manifest.package_id ?? this.lesson?.localPackageId ?? "").trim(),
        lesson_count: Math.max(0, Number(manifest.lesson_count) || 0),
        multi_lesson: lessonScope.multi_lesson,
        lesson_ids: lessonScope.lesson_ids,
        report_count: Math.max(0, Number(manifest.report_count) || 0),
        file_count: Math.max(0, Number(manifest.file_count) || 0),
        account_required: false,
        cloud_sync: false,
        public_registry: false,
        remote_save: false,
        manifest,
        payload: importedPackagePayload,
        index_text: formatStudioLocalPackageIndexText(importedPackagePayload),
        payload_text: payloadText,
      };
    }
    const lessonId = String(this.lesson?.id ?? reportModel.lesson_id ?? "").trim() || "current_lesson";
    const title = String(this.lesson?.title ?? reportModel.lesson_title ?? lessonId).trim() || lessonId;
    const sourceText = String(this.runDdnPreviewEl?.value ?? this.lesson?.source_text ?? this.lesson?.source ?? "");
    const packageId = `studio.local.${lessonId}`.replace(/[^a-zA-Z0-9._-]+/g, "_");
    const sessionLabel = String(this.runLocalPackageSessionInputEl?.value ?? "").trim();
    const rosterTemplateText = buildStudentRosterTemplateText({ lessonId, sessionLabel, packageId });
    const requiredViews = resolveLessonRequiredViewsForRun(this.lesson);
    const goals = Array.isArray(this.lesson?.goals)
      ? this.lesson.goals.map((item) => String(item ?? "").trim()).filter(Boolean)
      : [];
    const missions = Array.isArray(this.lesson?.missions)
      ? this.lesson.missions.map((item) => String(item ?? "").trim()).filter(Boolean)
      : [];
    const lessons = [
      {
        lesson_id: lessonId,
        title,
        description: String(this.lesson?.description ?? "").trim(),
        grade: String(this.lesson?.grade ?? "").trim(),
        subject: String(this.lesson?.subject ?? "").trim(),
        required_views: requiredViews,
        goals,
        missions,
        source_text: sourceText,
      },
    ];
    const baseReports = [
      {
        report_id: `${lessonId}_classroom_report`,
        title: `${title} 수업 리포트`,
        text: reportModel.text,
      },
      {
        report_id: `${lessonId}_teacher_preparation_checklist`,
        title: `${title} 교사용 준비 체크리스트`,
        path: `reports/${buildTeacherPreparationChecklistFileName({ lessonId, sessionLabel, packageId })}`,
        text: buildTeacherPreparationChecklistText({
          lessonTitle: title,
          lessonId,
          sessionLabel,
          packageId,
          requiredViews,
          goals,
          missions,
        }),
      },
      {
        report_id: `${lessonId}_student_roster_template`,
        title: `${title} 학생 명단 양식`,
        path: `reports/${buildStudentRosterTemplateFileName({ lessonId, sessionLabel, packageId })}`,
        mime: "text/tab-separated-values; charset=utf-8",
        text: rosterTemplateText,
      },
    ];
    const draftManifest = buildStudioLocalPackageManifest({
      packageId,
      title: `${title} 교사용 배포 묶음`,
      version: "0.1.0",
      sessionLabel,
      lessons,
      reports: baseReports,
    });
    const draftPayload = buildStudioLocalPackagePayload({ manifest: draftManifest, lessons, reports: baseReports });
    const studentGuideText = buildLocalPackageStudentGuideText({
      lesson_title: title,
      package_id: draftManifest.package_id,
      session_label: draftManifest.session_label || "",
      manifest: draftManifest,
      payload: draftPayload,
    });
    const reports = [
      baseReports[0],
      {
        report_id: `${lessonId}_student_guide`,
        title: `${title} 학생 배포 안내문`,
        path: `reports/${buildLocalPackageStudentGuideFileName({ packageId: draftManifest.package_id, sessionLabel: draftManifest.session_label || "" })}`,
        text: studentGuideText,
      },
      ...baseReports.slice(1),
    ];
    const manifest = buildStudioLocalPackageManifest({
      packageId,
      title: `${title} 교사용 배포 묶음`,
      version: "0.1.0",
      sessionLabel,
      lessons,
      reports,
    });
    const payload = buildStudioLocalPackagePayload({ manifest, lessons, reports });
    const lessonScope = buildLocalPackageLessonScope(payload, { fallbackLessonId: lessonId });
    const indexText = formatStudioLocalPackageIndexText(payload);
    const payloadText = JSON.stringify(payload, null, 2);
    return {
      schema: "seamgrim.local_package_export_action.v1",
      generated_by: "STUDIO_LOCAL_PACKAGE_EXPORT_ACTION_V1",
      mode: reportModel.mode,
      mode_label: reportModel.mode_label,
      lesson_id: lessonId,
      lesson_title: title,
      session_label: manifest.session_label || "",
      package_id: manifest.package_id,
      lesson_count: manifest.lesson_count,
      multi_lesson: lessonScope.multi_lesson,
      lesson_ids: lessonScope.lesson_ids,
      report_count: manifest.report_count,
      file_count: manifest.file_count,
      account_required: false,
      cloud_sync: false,
      public_registry: false,
      remote_save: false,
      manifest,
      payload,
      index_text: indexText,
      payload_text: payloadText,
    };
  }

  syncLocalPackageExport() {
    const model = this.buildLocalPackageExportModel();
    const teacherVisible = this.syncTeacherExportVisibility();
    if (this.runLocalPackageEl) {
      this.runLocalPackageEl.dataset.schema = model.schema;
      this.runLocalPackageEl.dataset.state = model.file_count > 0 ? "ready" : "draft";
      this.runLocalPackageEl.dataset.packageId = model.package_id;
    }
    if (this.runLocalPackageMetaEl) {
      const materialText = Number(model.report_count) > 0 ? ` · 교사용 자료 ${model.report_count}개` : "";
      this.runLocalPackageMetaEl.textContent = `${model.mode_label} · Studio 배포 열기용 · 교과 ${model.lesson_count}개${materialText} · 파일 ${model.file_count}개`;
      this.runLocalPackageMetaEl.dataset.value = String(model.file_count);
    }
    if (this.runLocalPackageGuideEl) {
      const guideText = buildLocalPackageGuideSummaryText(model);
      this.runLocalPackageGuideEl.textContent = guideText;
      this.runLocalPackageGuideEl.dataset.ready = model.file_count > 0 ? "1" : "0";
    }
    if (this.runLocalPackageCodeEl) {
      this.runLocalPackageCodeEl.textContent = model.package_id ? `배포 코드: ${model.package_id}` : "배포 코드: -";
      this.runLocalPackageCodeEl.dataset.ready = model.package_id ? "1" : "0";
      this.runLocalPackageCodeEl.dataset.packageId = model.package_id || "";
    }
    const checklistModel = this.runLocalPackageChecklistEl
      || this.runLocalPackageChecklistCopyBtn
      || this.runLocalPackageChecklistDownloadBtn
      ? this.buildTeacherPreparationChecklistModel()
      : null;
    if (this.runLocalPackageChecklistEl) {
      const sessionText = checklistModel?.session_label ? `차시 ${checklistModel.session_label} · ` : "";
      this.runLocalPackageChecklistEl.textContent = `교사용 준비: ${sessionText}배포 전/수업 중/수업 후 체크리스트`;
      this.runLocalPackageChecklistEl.dataset.ready = model.file_count > 0 ? "1" : "0";
    }
    if (this.runLocalPackageMaterialsEl) {
      const materialsText = buildLocalPackageMaterialsSummaryText(model);
      this.runLocalPackageMaterialsEl.textContent = materialsText;
      this.runLocalPackageMaterialsEl.dataset.reportCount = String(model.report_count);
      this.runLocalPackageMaterialsEl.dataset.lessonCount = String(model.lesson_count);
    }
    if (this.runLocalPackageGuideCopyBtn) {
      this.runLocalPackageGuideCopyBtn.disabled = !teacherVisible || !model.payload_text;
      this.runLocalPackageGuideCopyBtn.dataset.ready = model.file_count > 0 ? "1" : "0";
      this.runLocalPackageGuideCopyBtn.title = model.payload_text
        ? "학생에게 보낼 배포 열기 안내문을 복사합니다."
        : "복사할 학생 안내문이 없습니다.";
    }
    if (this.runLocalPackageGuideDownloadBtn) {
      this.runLocalPackageGuideDownloadBtn.disabled = !teacherVisible || !model.payload_text;
      this.runLocalPackageGuideDownloadBtn.dataset.ready = model.file_count > 0 ? "1" : "0";
      this.runLocalPackageGuideDownloadBtn.title = model.payload_text
        ? "학생에게 보낼 배포 열기 안내문을 파일로 저장합니다."
        : "저장할 학생 안내문이 없습니다.";
    }
    if (this.runLocalPackageCodeCopyBtn) {
      this.runLocalPackageCodeCopyBtn.disabled = !teacherVisible || !model.package_id;
      this.runLocalPackageCodeCopyBtn.dataset.ready = model.package_id ? "1" : "0";
      this.runLocalPackageCodeCopyBtn.dataset.packageId = model.package_id || "";
      this.runLocalPackageCodeCopyBtn.title = model.package_id
        ? "학생에게 전달할 배포 코드만 복사합니다."
        : "복사할 배포 코드가 없습니다.";
    }
    if (this.runLocalPackageChecklistCopyBtn) {
      this.runLocalPackageChecklistCopyBtn.disabled = !teacherVisible || !model.payload_text;
      this.runLocalPackageChecklistCopyBtn.dataset.ready = model.file_count > 0 ? "1" : "0";
      this.runLocalPackageChecklistCopyBtn.title = model.payload_text
        ? "교사용 준비 체크리스트를 복사합니다."
        : "복사할 교사용 준비 체크리스트가 없습니다.";
    }
    if (this.runLocalPackageChecklistDownloadBtn) {
      this.runLocalPackageChecklistDownloadBtn.disabled = !teacherVisible || !model.payload_text;
      this.runLocalPackageChecklistDownloadBtn.dataset.ready = model.file_count > 0 ? "1" : "0";
      this.runLocalPackageChecklistDownloadBtn.title = model.payload_text
        ? "교사용 준비 체크리스트를 파일로 저장합니다."
        : "저장할 교사용 준비 체크리스트가 없습니다.";
    }
    if (this.runLocalPackageTextEl) {
      this.runLocalPackageTextEl.textContent = model.index_text;
    }
    if (this.runLocalPackageCopyBtn) {
      this.runLocalPackageCopyBtn.disabled = !teacherVisible || !model.payload_text;
    }
    if (this.runLocalPackageDownloadBtn) {
      this.runLocalPackageDownloadBtn.disabled = !teacherVisible || !model.payload_text;
      this.runLocalPackageDownloadBtn.dataset.fileCount = String(model.file_count);
      this.runLocalPackageDownloadBtn.textContent = model.file_count > 0
        ? `패키지 저장 ${model.file_count}`
        : "패키지 저장";
      this.runLocalPackageDownloadBtn.title = model.payload_text
        ? `교사용 배포 묶음 JSON 파일 ${model.file_count}개 항목 저장`
        : "저장할 교사용 배포 묶음이 없습니다.";
    }
    if (this.runTeacherPackageCopyBtn) {
      this.runTeacherPackageCopyBtn.disabled = !teacherVisible || !model.payload_text;
      this.runTeacherPackageCopyBtn.dataset.fileCount = String(model.file_count);
      this.runTeacherPackageCopyBtn.textContent = model.file_count > 0
        ? `배포 복사 ${model.file_count}`
        : "배포 복사";
      this.runTeacherPackageCopyBtn.title = model.payload_text
        ? `학생이 Studio의 배포 열기에서 사용할 JSON 배포 파일 ${model.file_count}개 항목 복사`
        : "복사할 교사용 배포 묶음이 없습니다.";
    }
    if (this.runTeacherPackageDownloadBtn) {
      this.runTeacherPackageDownloadBtn.disabled = !teacherVisible || !model.payload_text;
      this.runTeacherPackageDownloadBtn.dataset.fileCount = String(model.file_count);
      this.runTeacherPackageDownloadBtn.textContent = model.file_count > 0
        ? `배포 저장 ${model.file_count}`
        : "배포 저장";
      this.runTeacherPackageDownloadBtn.title = model.payload_text
        ? `학생이 Studio의 배포 열기에서 사용할 JSON 배포 파일 ${model.file_count}개 항목 저장`
        : "저장할 교사용 배포 묶음이 없습니다.";
    }
    try {
      window.__STUDIO_LOCAL_PACKAGE_EXPORT_ACTION__ = model;
    } catch (_) {
      // ignore browser instrumentation errors
    }
    return model;
  }

  async handleCopyLocalPackageExport() {
    const model = this.syncLocalPackageExport();
    const value = String(model?.payload_text ?? "").trim();
    if (!value) {
      showGlobalToast("복사할 교사용 배포 묶음이 없습니다.", { kind: "error" });
      return false;
    }
    let ok = false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(value);
        ok = true;
      }
    } catch (_) {
      ok = false;
    }
    try {
      window.__STUDIO_LOCAL_PACKAGE_EXPORT_ACTION__ = {
        ...model,
        copied: ok,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "교사용 배포 묶음을 복사했습니다." : "교사용 배포 묶음 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  async handleCopyLocalPackageStudentGuide() {
    const model = this.syncLocalPackageExport();
    const value = buildLocalPackageStudentGuideText(model).trim();
    if (!value || !model?.payload_text) {
      showGlobalToast("복사할 학생 안내문이 없습니다.", { kind: "error" });
      return false;
    }
    let ok = false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(value);
        ok = true;
      }
    } catch (_) {
      ok = false;
    }
    const lessonScope = buildLocalPackageLessonScope(model?.payload, { fallbackLessonId: model.lesson_id });
    try {
      window.__STUDIO_LOCAL_PACKAGE_STUDENT_GUIDE_COPY_ACTION__ = {
        schema: "seamgrim.local_package_student_guide_copy_action.v1",
        copied: ok,
        package_id: model.package_id,
        lesson_id: model.lesson_id,
        session_label: String(model?.manifest?.session_label ?? "").trim(),
        multi_lesson: lessonScope.multi_lesson,
        lesson_ids: lessonScope.lesson_ids,
        guide_text: value,
        account_required: false,
        cloud_sync: false,
        public_registry: false,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "학생 안내문을 복사했습니다." : "학생 안내문 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  async handleCopyLocalPackageCode() {
    const model = this.syncLocalPackageExport();
    const value = String(model?.package_id ?? "").trim();
    if (!value) {
      showGlobalToast("복사할 배포 코드가 없습니다.", { kind: "error" });
      return false;
    }
    let ok = false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(value);
        ok = true;
      }
    } catch (_) {
      ok = false;
    }
    const lessonScope = buildLocalPackageLessonScope(model?.payload, { fallbackLessonId: model.lesson_id });
    try {
      window.__STUDIO_LOCAL_PACKAGE_CODE_COPY_ACTION__ = {
        schema: "seamgrim.local_package_code_copy_action.v1",
        copied: ok,
        package_id: value,
        lesson_id: model.lesson_id,
        session_label: String(model?.manifest?.session_label ?? "").trim(),
        multi_lesson: lessonScope.multi_lesson,
        lesson_ids: lessonScope.lesson_ids,
        account_required: false,
        cloud_sync: false,
        public_registry: false,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "배포 코드를 복사했습니다." : "배포 코드 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  handleDownloadLocalPackageStudentGuide() {
    const model = this.syncLocalPackageExport();
    const value = buildLocalPackageStudentGuideText(model).trim();
    if (!value || !model?.payload_text) {
      showGlobalToast("저장할 학생 안내문이 없습니다.", { kind: "error" });
      return false;
    }
    const sessionLabel = String(model?.manifest?.session_label ?? "").trim();
    const fileName = buildLocalPackageStudentGuideFileName({ packageId: model.package_id, sessionLabel });
    let ok = false;
    try {
      savePlainTextToFile(value, fileName);
      ok = true;
    } catch (_) {
      ok = false;
    }
    const lessonScope = buildLocalPackageLessonScope(model?.payload, { fallbackLessonId: model.lesson_id });
    try {
      window.__STUDIO_LOCAL_PACKAGE_STUDENT_GUIDE_DOWNLOAD_ACTION__ = {
        schema: "seamgrim.local_package_student_guide_download_action.v1",
        downloaded: ok,
        file_name: fileName,
        package_id: model.package_id,
        lesson_id: model.lesson_id,
        session_label: sessionLabel,
        multi_lesson: lessonScope.multi_lesson,
        lesson_ids: lessonScope.lesson_ids,
        guide_text: value,
        account_required: false,
        cloud_sync: false,
        public_registry: false,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "학생 안내문을 저장했습니다." : "학생 안내문 저장에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  handleDownloadLocalPackageExport() {
    const model = this.buildLocalPackageExportModel();
    const value = String(model?.payload_text ?? "").trim();
    if (!value) {
      showGlobalToast("저장할 교사용 배포 묶음이 없습니다.", { kind: "error" });
      return false;
    }
    const sessionLabel = String(model?.manifest?.session_label ?? "").trim();
    const sessionSuffix = sessionLabel ? `_${sessionLabel}` : "";
    const fileName = buildSafeJsonDownloadName(`${model?.package_id || "seamgrim-teacher-package"}${sessionSuffix}`, "seamgrim-teacher-package");
    let ok = false;
    try {
      saveJsonTextToFile(value, fileName);
      ok = true;
    } catch (_) {
      ok = false;
    }
    try {
      window.__STUDIO_LOCAL_PACKAGE_DOWNLOAD_ACTION__ = {
        ...model,
        downloaded: ok,
        file_name: fileName,
        session_label: sessionLabel,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "교사용 배포 파일을 저장했습니다." : "교사용 배포 파일 저장에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  buildPublicationPrepExportModel() {
    const packageModel = this.buildLocalPackageExportModel();
    const surface = buildLessonPublicationReviewSurface({
      candidateIds: DEFAULT_LESSON_PUBLICATION_CANDIDATE_IDS,
    });
    const candidateIds = Array.isArray(surface.candidate_ids)
      ? surface.candidate_ids
      : DEFAULT_LESSON_PUBLICATION_CANDIDATE_IDS;
    const selectedCandidate = candidateIds.includes(packageModel.lesson_id)
      ? packageModel.lesson_id
      : candidateIds[0] ?? "";
    const payload = {
      __종류: "studio_publication_prep_export_payload",
      schema: "seamgrim.publication_prep_export_action.v1",
      generated_by: "STUDIO_PUBLICATION_PREP_EXPORT_ACTION_V1",
      lesson_id: packageModel.lesson_id,
      lesson_title: packageModel.lesson_title,
      package_id: packageModel.package_id,
      selected_candidate_id: selectedCandidate,
      candidate_count: candidateIds.length,
      surface_status: surface.status,
      surface_row_count: surface.surface_row_count,
      local_package_manifest: packageModel.manifest,
      publication_review_surface: surface,
      public_upload_claim: false,
      registry_publish_claim: false,
      cloud_sync: false,
      account_required: false,
      permission_system: false,
      remote_save: false,
      active_allowlist_mutation: false,
      publication_snapshot_emit_claim: false,
    };
    const reviewText = formatLessonPublicationReviewSurfaceText(surface);
    const text = [
      "항목\t값",
      `schema\t${payload.schema}`,
      `lesson_id\t${payload.lesson_id}`,
      `package_id\t${payload.package_id}`,
      `selected_candidate_id\t${payload.selected_candidate_id}`,
      `candidate_count\t${payload.candidate_count}`,
      `surface_status\t${payload.surface_status}`,
      "public_upload_claim\tfalse",
      "registry_publish_claim\tfalse",
      "active_allowlist_mutation\tfalse",
      "",
      reviewText,
    ].join("\n");
    return {
      schema: "seamgrim.publication_prep_export_action.v1",
      generated_by: "STUDIO_PUBLICATION_PREP_EXPORT_ACTION_V1",
      mode: packageModel.mode,
      mode_label: packageModel.mode_label,
      lesson_id: packageModel.lesson_id,
      lesson_title: packageModel.lesson_title,
      package_id: packageModel.package_id,
      candidate_count: candidateIds.length,
      selected_candidate_id: selectedCandidate,
      surface_status: surface.status,
      surface_row_count: surface.surface_row_count,
      account_required: false,
      cloud_sync: false,
      public_upload_claim: false,
      registry_publish_claim: false,
      permission_system: false,
      remote_save: false,
      active_allowlist_mutation: false,
      publication_snapshot_emit_claim: false,
      payload,
      text,
      payload_text: JSON.stringify(payload, null, 2),
    };
  }

  syncPublicationPrepExport() {
    const model = this.buildPublicationPrepExportModel();
    if (this.runPublicationPrepEl) {
      this.runPublicationPrepEl.dataset.schema = model.schema;
      this.runPublicationPrepEl.dataset.state = model.candidate_count > 0 ? "ready" : "draft";
      this.runPublicationPrepEl.dataset.candidateCount = String(model.candidate_count);
    }
    if (this.runPublicationPrepMetaEl) {
      this.runPublicationPrepMetaEl.textContent = `${model.mode_label} · 후보 ${model.candidate_count}개`;
      this.runPublicationPrepMetaEl.dataset.value = String(model.candidate_count);
    }
    if (this.runPublicationPrepTextEl) {
      this.runPublicationPrepTextEl.textContent = model.text;
    }
    if (this.runPublicationPrepCopyBtn) {
      this.runPublicationPrepCopyBtn.disabled = !model.payload_text;
    }
    try {
      window.__STUDIO_PUBLICATION_PREP_EXPORT_ACTION__ = model;
    } catch (_) {
      // ignore browser instrumentation errors
    }
    return model;
  }

  async handleCopyPublicationPrepExport() {
    const model = this.syncPublicationPrepExport();
    const value = String(model?.payload_text ?? "").trim();
    if (!value) {
      showGlobalToast("복사할 공개 준비안이 없습니다.", { kind: "error" });
      return false;
    }
    let ok = false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(value);
        ok = true;
      }
    } catch (_) {
      ok = false;
    }
    try {
      window.__STUDIO_PUBLICATION_PREP_EXPORT_ACTION__ = {
        ...model,
        copied: ok,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "공개 준비안을 복사했습니다." : "공개 준비안 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  buildRegistrySeedExportModel() {
    const prepModel = this.buildPublicationPrepExportModel();
    const candidateIds = Array.isArray(prepModel.payload?.publication_review_surface?.candidate_ids)
      ? prepModel.payload.publication_review_surface.candidate_ids
      : DEFAULT_LESSON_PUBLICATION_CANDIDATE_IDS;
    const rows = candidateIds.map((lessonId) => ({
      lesson_id: lessonId,
      registry_id: `studio/lesson/${lessonId}`,
      scope: "나눔",
      catalog_kind: "lesson_catalog",
      visibility: "public_candidate",
      share_kind: "link",
      share_target: "artifact",
      draft_only: true,
      publish_claim: false,
    }));
    const payload = {
      __종류: "studio_registry_share_seed_export_payload",
      schema: "seamgrim.registry_share_seed_export_action.v1",
      generated_by: "STUDIO_REGISTRY_SHARE_SEED_EXPORT_ACTION_V1",
      based_on: prepModel.schema,
      lesson_id: prepModel.lesson_id,
      package_id: prepModel.package_id,
      seed_count: rows.length,
      rows,
      registry_publish_claim: false,
      public_upload_claim: false,
      public_link_creation_claim: false,
      install_enablement_claim: false,
      publication_snapshot_emit_claim: false,
      cloud_sync: false,
      account_required: false,
      permission_system: false,
      remote_save: false,
      active_allowlist_mutation: false,
    };
    const text = [
      "lesson_id\tregistry_id\tdraft_only\tpublish_claim",
      ...rows.map((row) => [
        row.lesson_id,
        row.registry_id,
        row.draft_only ? "true" : "false",
        row.publish_claim ? "true" : "false",
      ].join("\t")),
      "",
      "registry_publish_claim\tfalse",
      "public_upload_claim\tfalse",
      "public_link_creation_claim\tfalse",
      "install_enablement_claim\tfalse",
    ].join("\n");
    return {
      schema: "seamgrim.registry_share_seed_export_action.v1",
      generated_by: "STUDIO_REGISTRY_SHARE_SEED_EXPORT_ACTION_V1",
      mode: prepModel.mode,
      mode_label: prepModel.mode_label,
      lesson_id: prepModel.lesson_id,
      package_id: prepModel.package_id,
      seed_count: rows.length,
      registry_publish_claim: false,
      public_upload_claim: false,
      public_link_creation_claim: false,
      install_enablement_claim: false,
      publication_snapshot_emit_claim: false,
      cloud_sync: false,
      account_required: false,
      permission_system: false,
      remote_save: false,
      active_allowlist_mutation: false,
      payload,
      text,
      payload_text: JSON.stringify(payload, null, 2),
    };
  }

  syncRegistrySeedExport() {
    const model = this.buildRegistrySeedExportModel();
    if (this.runRegistrySeedEl) {
      this.runRegistrySeedEl.dataset.schema = model.schema;
      this.runRegistrySeedEl.dataset.state = model.seed_count > 0 ? "ready" : "draft";
      this.runRegistrySeedEl.dataset.seedCount = String(model.seed_count);
    }
    if (this.runRegistrySeedMetaEl) {
      this.runRegistrySeedMetaEl.textContent = `${model.mode_label} · seed ${model.seed_count}개`;
      this.runRegistrySeedMetaEl.dataset.value = String(model.seed_count);
    }
    if (this.runRegistrySeedTextEl) {
      this.runRegistrySeedTextEl.textContent = model.text;
    }
    if (this.runRegistrySeedCopyBtn) {
      this.runRegistrySeedCopyBtn.disabled = !model.payload_text;
    }
    try {
      window.__STUDIO_REGISTRY_SHARE_SEED_EXPORT_ACTION__ = model;
    } catch (_) {
      // ignore browser instrumentation errors
    }
    return model;
  }

  async handleCopyRegistrySeedExport() {
    const model = this.syncRegistrySeedExport();
    const value = String(model?.payload_text ?? "").trim();
    if (!value) {
      showGlobalToast("복사할 registry seed가 없습니다.", { kind: "error" });
      return false;
    }
    let ok = false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(value);
        ok = true;
      }
    } catch (_) {
      ok = false;
    }
    try {
      window.__STUDIO_REGISTRY_SHARE_SEED_EXPORT_ACTION__ = {
        ...model,
        copied: ok,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "Registry seed를 복사했습니다." : "Registry seed 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  buildApprovalContinuityExportModel() {
    const seedModel = this.buildRegistrySeedExportModel();
    const requiredApprovalPhrase = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다";
    const blockedUntilApproval = [
      "github_release_create",
      "public_upload",
      "registry_publish",
      "cloud_sync",
      "account_setup",
      "artifact_signing",
      "publication_archive_generation",
      "checksum_manifest_generation_for_publication",
      "public_link_create",
      "package_install_enable",
      "publication_snapshot_emit",
      "permission_system_change",
    ];
    const payload = {
      __종류: "studio_release_approval_continuity_export_payload",
      schema: "seamgrim.release_approval_continuity_export_action.v1",
      generated_by: "STUDIO_RELEASE_APPROVAL_CONTINUITY_EXPORT_ACTION_V1",
      based_on: seedModel.schema,
      lesson_id: seedModel.lesson_id,
      package_id: seedModel.package_id,
      seed_count: seedModel.seed_count,
      required_approval_phrase: requiredApprovalPhrase,
      generic_next_dev_request_is_approval: false,
      next_state: "AWAIT_EXPLICIT_RELEASE_APPROVAL",
      blocked_until_approval: blockedUntilApproval,
      release_approval_claim: false,
      release_execution_claim: false,
      public_release_claim: false,
      github_release_claim: false,
      public_upload_claim: false,
      registry_publish_claim: false,
      public_link_creation_claim: false,
      install_enablement_claim: false,
      publication_snapshot_emit_claim: false,
      cloud_sync: false,
      account_required: false,
      permission_system: false,
      remote_save: false,
      active_allowlist_mutation: false,
      registry_seed_payload: seedModel.payload,
    };
    const text = [
      "항목\t값",
      `schema\t${payload.schema}`,
      `next_state\t${payload.next_state}`,
      `required_approval_phrase\t${payload.required_approval_phrase}`,
      `generic_next_dev_request_is_approval\t${payload.generic_next_dev_request_is_approval ? "true" : "false"}`,
      `seed_count\t${payload.seed_count}`,
      "release_approval_claim\tfalse",
      "release_execution_claim\tfalse",
      "public_upload_claim\tfalse",
      "registry_publish_claim\tfalse",
      "",
      "blocked_until_approval",
      ...blockedUntilApproval,
    ].join("\n");
    return {
      schema: "seamgrim.release_approval_continuity_export_action.v1",
      generated_by: "STUDIO_RELEASE_APPROVAL_CONTINUITY_EXPORT_ACTION_V1",
      mode: seedModel.mode,
      mode_label: seedModel.mode_label,
      lesson_id: seedModel.lesson_id,
      package_id: seedModel.package_id,
      seed_count: seedModel.seed_count,
      required_approval_phrase: requiredApprovalPhrase,
      generic_next_dev_request_is_approval: false,
      next_state: "AWAIT_EXPLICIT_RELEASE_APPROVAL",
      blocked_count: blockedUntilApproval.length,
      release_approval_claim: false,
      release_execution_claim: false,
      public_release_claim: false,
      github_release_claim: false,
      public_upload_claim: false,
      registry_publish_claim: false,
      public_link_creation_claim: false,
      install_enablement_claim: false,
      publication_snapshot_emit_claim: false,
      cloud_sync: false,
      account_required: false,
      permission_system: false,
      remote_save: false,
      active_allowlist_mutation: false,
      payload,
      text,
      payload_text: JSON.stringify(payload, null, 2),
    };
  }

  syncApprovalContinuityExport() {
    const model = this.buildApprovalContinuityExportModel();
    if (this.runApprovalContinuityEl) {
      this.runApprovalContinuityEl.dataset.schema = model.schema;
      this.runApprovalContinuityEl.dataset.state = model.next_state;
      this.runApprovalContinuityEl.dataset.blockedCount = String(model.blocked_count);
    }
    if (this.runApprovalContinuityMetaEl) {
      this.runApprovalContinuityMetaEl.textContent = `${model.mode_label} · ${model.next_state}`;
      this.runApprovalContinuityMetaEl.dataset.value = String(model.blocked_count);
    }
    if (this.runApprovalContinuityTextEl) {
      this.runApprovalContinuityTextEl.textContent = model.text;
    }
    if (this.runApprovalContinuityCopyBtn) {
      this.runApprovalContinuityCopyBtn.disabled = !model.payload_text;
    }
    try {
      window.__STUDIO_RELEASE_APPROVAL_CONTINUITY_EXPORT_ACTION__ = model;
    } catch (_) {
      // ignore browser instrumentation errors
    }
    return model;
  }

  async handleCopyApprovalContinuityExport() {
    const model = this.syncApprovalContinuityExport();
    const value = String(model?.payload_text ?? "").trim();
    if (!value) {
      showGlobalToast("복사할 승인 연속성 패킷이 없습니다.", { kind: "error" });
      return false;
    }
    let ok = false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(value);
        ok = true;
      }
    } catch (_) {
      ok = false;
    }
    try {
      window.__STUDIO_RELEASE_APPROVAL_CONTINUITY_EXPORT_ACTION__ = {
        ...model,
        copied: ok,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "승인 연속성 패킷을 복사했습니다." : "승인 연속성 패킷 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  buildBenchmarkLtsExportModel() {
    const continuityModel = this.buildApprovalContinuityExportModel();
    const rows = [
      { id: "approval_continuity", kind: "approval_chain", required: true, ready: true },
      { id: "registry_share_seed", kind: "registry_seed", required: true, ready: continuityModel.seed_count === 15 },
      { id: "browser_smoke_matrix", kind: "browser_smoke", required: true, ready: true },
      { id: "local_packaging", kind: "local_packaging", required: true, ready: true },
      { id: "public_lesson_publication_prep", kind: "publication_prep", required: true, ready: true },
    ];
    const readyCount = rows.filter((row) => row.ready).length;
    const payload = {
      __종류: "studio_benchmark_lts_export_payload",
      schema: "seamgrim.benchmark_lts_matrix_export_action.v1",
      generated_by: "STUDIO_BENCHMARK_LTS_MATRIX_EXPORT_ACTION_V1",
      based_on: continuityModel.schema,
      lesson_id: continuityModel.lesson_id,
      package_id: continuityModel.package_id,
      matrix_entry_count: rows.length,
      ready_entry_count: readyCount,
      matrix_ready: readyCount === rows.length,
      rows,
      deferred_heavy_gates: [
        "full CI profile matrix",
        "real performance benchmark baseline",
        "public release execution",
        "cloud/account integration",
      ],
      benchmark_execution_claim: false,
      performance_baseline_claim: false,
      performance_baseline_publication_claim: false,
      lts_certification_claim: false,
      release_approval_claim: false,
      release_execution_claim: false,
      public_release_claim: false,
      github_release_claim: false,
      public_upload_claim: false,
      registry_publish_claim: false,
      cloud_sync: false,
      account_required: false,
      permission_system: false,
      active_allowlist_mutation: false,
      approval_continuity_payload: continuityModel.payload,
    };
    const text = [
      "matrix_id\tkind\trequired\tready",
      ...rows.map((row) => [
        row.id,
        row.kind,
        row.required ? "true" : "false",
        row.ready ? "true" : "false",
      ].join("\t")),
      "",
      `matrix_ready\t${payload.matrix_ready ? "true" : "false"}`,
      "benchmark_execution_claim\tfalse",
      "performance_baseline_claim\tfalse",
      "lts_certification_claim\tfalse",
      "release_execution_claim\tfalse",
    ].join("\n");
    return {
      schema: "seamgrim.benchmark_lts_matrix_export_action.v1",
      generated_by: "STUDIO_BENCHMARK_LTS_MATRIX_EXPORT_ACTION_V1",
      mode: continuityModel.mode,
      mode_label: continuityModel.mode_label,
      lesson_id: continuityModel.lesson_id,
      package_id: continuityModel.package_id,
      matrix_entry_count: rows.length,
      ready_entry_count: readyCount,
      matrix_ready: readyCount === rows.length,
      benchmark_execution_claim: false,
      performance_baseline_claim: false,
      performance_baseline_publication_claim: false,
      lts_certification_claim: false,
      release_approval_claim: false,
      release_execution_claim: false,
      public_release_claim: false,
      github_release_claim: false,
      public_upload_claim: false,
      registry_publish_claim: false,
      cloud_sync: false,
      account_required: false,
      permission_system: false,
      active_allowlist_mutation: false,
      payload,
      text,
      payload_text: JSON.stringify(payload, null, 2),
    };
  }

  syncBenchmarkLtsExport() {
    const model = this.buildBenchmarkLtsExportModel();
    if (this.runBenchmarkLtsEl) {
      this.runBenchmarkLtsEl.dataset.schema = model.schema;
      this.runBenchmarkLtsEl.dataset.state = model.matrix_ready ? "ready" : "draft";
      this.runBenchmarkLtsEl.dataset.entryCount = String(model.matrix_entry_count);
    }
    if (this.runBenchmarkLtsMetaEl) {
      this.runBenchmarkLtsMetaEl.textContent = `${model.mode_label} · ready ${model.ready_entry_count}/${model.matrix_entry_count}`;
      this.runBenchmarkLtsMetaEl.dataset.value = String(model.ready_entry_count);
    }
    if (this.runBenchmarkLtsTextEl) {
      this.runBenchmarkLtsTextEl.textContent = model.text;
    }
    if (this.runBenchmarkLtsCopyBtn) {
      this.runBenchmarkLtsCopyBtn.disabled = !model.payload_text;
    }
    try {
      window.__STUDIO_BENCHMARK_LTS_MATRIX_EXPORT_ACTION__ = model;
    } catch (_) {
      // ignore browser instrumentation errors
    }
    return model;
  }

  async handleCopyBenchmarkLtsExport() {
    const model = this.syncBenchmarkLtsExport();
    const value = String(model?.payload_text ?? "").trim();
    if (!value) {
      showGlobalToast("복사할 benchmark/LTS matrix가 없습니다.", { kind: "error" });
      return false;
    }
    let ok = false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(value);
        ok = true;
      }
    } catch (_) {
      ok = false;
    }
    try {
      window.__STUDIO_BENCHMARK_LTS_MATRIX_EXPORT_ACTION__ = {
        ...model,
        copied: ok,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "Benchmark/LTS matrix를 복사했습니다." : "Benchmark/LTS matrix 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  buildEducationOperationsLtsExportModel() {
    const classroomModel = this.buildClassroomReportExportModel();
    const packageModel = this.buildLocalPackageExportModel();
    const prepModel = this.buildPublicationPrepExportModel();
    const seedModel = this.buildRegistrySeedExportModel();
    const continuityModel = this.buildApprovalContinuityExportModel();
    const benchmarkModel = this.buildBenchmarkLtsExportModel();
    const rows = [
      { id: "classroom_report", kind: "classroom_report", required: true, ready: classroomModel.summary_count >= 1 },
      { id: "local_package", kind: "local_package", required: true, ready: packageModel.file_count >= 2 },
      { id: "publication_prep", kind: "publication_prep", required: true, ready: prepModel.candidate_count > 0 },
      { id: "registry_share_seed", kind: "registry_seed", required: true, ready: seedModel.seed_count === 15 },
      { id: "release_approval_continuity", kind: "approval_continuity", required: true, ready: continuityModel.next_state === "AWAIT_EXPLICIT_RELEASE_APPROVAL" },
      { id: "benchmark_lts_matrix", kind: "benchmark_lts_matrix", required: true, ready: benchmarkModel.matrix_ready === true },
    ];
    const readyCount = rows.filter((row) => row.ready).length;
    const payload = {
      __종류: "studio_education_operations_lts_export_payload",
      schema: "seamgrim.education_operations_lts_export_action.v1",
      generated_by: "STUDIO_EDUCATION_OPERATIONS_LTS_EXPORT_ACTION_V1",
      based_on: benchmarkModel.schema,
      lesson_id: benchmarkModel.lesson_id,
      package_id: benchmarkModel.package_id,
      operations_entry_count: rows.length,
      ready_entry_count: readyCount,
      operations_ready: readyCount === rows.length,
      rows,
      local_operations_packet_claim: true,
      education_operations_lts_certification_claim: false,
      lts_certification_claim: false,
      benchmark_execution_claim: false,
      performance_baseline_claim: false,
      performance_baseline_publication_claim: false,
      release_approval_claim: false,
      release_execution_claim: false,
      public_release_claim: false,
      github_release_claim: false,
      public_upload_claim: false,
      registry_publish_claim: false,
      public_link_creation_claim: false,
      install_enablement_claim: false,
      publication_snapshot_emit_claim: false,
      archive_generation_claim: false,
      checksum_manifest_generation_claim: false,
      cloud_sync: false,
      account_required: false,
      permission_system: false,
      remote_save: false,
      active_allowlist_mutation: false,
      classroom_report: classroomModel.report,
      local_package_payload: packageModel.payload,
      publication_prep_payload: prepModel.payload,
      registry_seed_payload: seedModel.payload,
      release_approval_continuity_payload: continuityModel.payload,
      benchmark_lts_payload: benchmarkModel.payload,
    };
    const text = [
      "operation_id\tkind\trequired\tready",
      ...rows.map((row) => [
        row.id,
        row.kind,
        row.required ? "true" : "false",
        row.ready ? "true" : "false",
      ].join("\t")),
      "",
      `operations_ready\t${payload.operations_ready ? "true" : "false"}`,
      "local_operations_packet_claim\ttrue",
      "education_operations_lts_certification_claim\tfalse",
      "lts_certification_claim\tfalse",
      "benchmark_execution_claim\tfalse",
      "release_execution_claim\tfalse",
      "public_upload_claim\tfalse",
      "registry_publish_claim\tfalse",
    ].join("\n");
    return {
      schema: "seamgrim.education_operations_lts_export_action.v1",
      generated_by: "STUDIO_EDUCATION_OPERATIONS_LTS_EXPORT_ACTION_V1",
      mode: benchmarkModel.mode,
      mode_label: benchmarkModel.mode_label,
      lesson_id: benchmarkModel.lesson_id,
      package_id: benchmarkModel.package_id,
      operations_entry_count: rows.length,
      ready_entry_count: readyCount,
      operations_ready: readyCount === rows.length,
      local_operations_packet_claim: true,
      education_operations_lts_certification_claim: false,
      lts_certification_claim: false,
      benchmark_execution_claim: false,
      performance_baseline_claim: false,
      performance_baseline_publication_claim: false,
      release_approval_claim: false,
      release_execution_claim: false,
      public_release_claim: false,
      github_release_claim: false,
      public_upload_claim: false,
      registry_publish_claim: false,
      public_link_creation_claim: false,
      install_enablement_claim: false,
      publication_snapshot_emit_claim: false,
      archive_generation_claim: false,
      checksum_manifest_generation_claim: false,
      cloud_sync: false,
      account_required: false,
      permission_system: false,
      remote_save: false,
      active_allowlist_mutation: false,
      payload,
      text,
      payload_text: JSON.stringify(payload, null, 2),
    };
  }

  syncEducationOperationsLtsExport() {
    const model = this.buildEducationOperationsLtsExportModel();
    if (this.runEducationOperationsLtsEl) {
      this.runEducationOperationsLtsEl.dataset.schema = model.schema;
      this.runEducationOperationsLtsEl.dataset.state = model.operations_ready ? "ready" : "draft";
      this.runEducationOperationsLtsEl.dataset.entryCount = String(model.operations_entry_count);
    }
    if (this.runEducationOperationsLtsMetaEl) {
      this.runEducationOperationsLtsMetaEl.textContent = `${model.mode_label} · ready ${model.ready_entry_count}/${model.operations_entry_count}`;
      this.runEducationOperationsLtsMetaEl.dataset.value = String(model.ready_entry_count);
    }
    if (this.runEducationOperationsLtsTextEl) {
      this.runEducationOperationsLtsTextEl.textContent = model.text;
    }
    if (this.runEducationOperationsLtsCopyBtn) {
      this.runEducationOperationsLtsCopyBtn.disabled = !model.payload_text;
    }
    try {
      window.__STUDIO_EDUCATION_OPERATIONS_LTS_EXPORT_ACTION__ = model;
    } catch (_) {
      // ignore browser instrumentation errors
    }
    return model;
  }

  async handleCopyEducationOperationsLtsExport() {
    const model = this.syncEducationOperationsLtsExport();
    const value = String(model?.payload_text ?? "").trim();
    if (!value) {
      showGlobalToast("복사할 운영 LTS 패킷이 없습니다.", { kind: "error" });
      return false;
    }
    let ok = false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(value);
        ok = true;
      }
    } catch (_) {
      ok = false;
    }
    try {
      window.__STUDIO_EDUCATION_OPERATIONS_LTS_EXPORT_ACTION__ = {
        ...model,
        copied: ok,
      };
    } catch (_) {
      // ignore browser instrumentation errors
    }
    showGlobalToast(ok ? "운영 LTS 패킷을 복사했습니다." : "운영 LTS 패킷 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
    return ok;
  }

  renderRunManagerUi() {
    if (!this.runManagerListEl) {
      this.syncRunHistoryComparisonRail();
      return;
    }
    const display = buildRunManagerDisplayState({
      overlayRuns: this.overlayRuns,
      activeOverlayRunId: this.activeOverlayRunId,
      hoverOverlayRunId: this.hoverOverlayRunId,
      soloOverlayRunId: this.soloOverlayRunId,
    });
    if (!display.rows.length) {
      this.runManagerListEl.innerHTML = '<div class="run-manager-empty">저장된 run 없음</div>';
      this.syncRunHistoryComparisonRail();
      return;
    }
    const html = display.rows
      .map((row) => {
        const id = row.id;
        const label = row.label;
        const classes = [
          "run-manager-row",
          row.isActive ? "is-active" : "",
          row.isHovered ? "is-hovered" : "",
          row.isSolo ? "is-solo" : "",
        ]
          .filter(Boolean)
          .join(" ");
        return `
          <div class="${classes}" data-run-row="${escapeHtml(id)}">
            <label class="run-manager-visible">
              <input type="checkbox" data-run-visible="${escapeHtml(id)}" ${row.visible !== false ? "checked" : ""} />
              <span class="run-manager-dot" style="--run-color:${escapeHtml(row.rowColor)};"></span>
            </label>
            <button type="button" class="run-manager-label" data-run-solo="${escapeHtml(id)}" title="solo 토글">${escapeHtml(label)}</button>
            <span class="run-manager-hash" title="${escapeHtml(String(row?.run?.hash?.result || row?.run?.hash?.input || "-"))}">${escapeHtml(row.hashText)}</span>
            <button type="button" class="run-manager-remove" data-run-remove="${escapeHtml(id)}" title="삭제">×</button>
          </div>
        `;
      })
      .join("");
    this.runManagerListEl.innerHTML = html;

    this.runManagerListEl.querySelectorAll("[data-run-visible]").forEach((input) => {
      input.addEventListener("change", (event) => {
        const target = event.currentTarget;
        const runId = String(target?.getAttribute?.("data-run-visible") ?? "").trim();
        const index = this.findRunManagerIndexById(runId);
        if (index < 0) return;
        this.overlayRuns[index].visible = Boolean(target?.checked);
        this.syncRunManagerOverlaySeries();
        this.syncRunHistoryComparisonRail();
        this.publishRunManagerSession();
      });
    });
    this.runManagerListEl.querySelectorAll("[data-run-solo]").forEach((button) => {
      button.addEventListener("click", (event) => {
        const target = event.currentTarget;
        const runId = String(target?.getAttribute?.("data-run-solo") ?? "").trim();
        this.toggleRunManagerSolo(runId);
      });
    });
    this.runManagerListEl.querySelectorAll("[data-run-remove]").forEach((button) => {
      button.addEventListener("click", (event) => {
        const target = event.currentTarget;
        const runId = String(target?.getAttribute?.("data-run-remove") ?? "").trim();
        this.removeRunManagerRun(runId);
      });
    });
    this.runManagerListEl.querySelectorAll("[data-run-row]").forEach((rowEl) => {
      rowEl.addEventListener("mouseenter", () => {
        const runId = String(rowEl?.getAttribute?.("data-run-row") ?? "").trim();
        this.setRunManagerHover(runId);
      });
      rowEl.addEventListener("mouseleave", () => {
        this.setRunManagerHover("");
      });
    });
    this.syncRunHistoryComparisonRail();
  }

  syncRunManagerOverlaySeries() {
    const display = buildRunManagerDisplayState({
      overlayRuns: this.overlayRuns,
      activeOverlayRunId: this.activeOverlayRunId,
      hoverOverlayRunId: this.hoverOverlayRunId,
      soloOverlayRunId: this.soloOverlayRunId,
    });
    const fallbackGraph = cloneGraphForRunManager(this.lastGraphSnapshot ?? null);
    const persistedGraph = cloneGraphForRunManager(display.activeRun?.graph ?? fallbackGraph);
    const hasSolo = Boolean(String(this.soloOverlayRunId ?? "").trim());
    const baseVisible = display.activeRun
      ? display.baseVisible
      : (Boolean(fallbackGraph) && !hasSolo);
    const baseAlpha = display.activeRun ? display.baseAlpha : 1;
    this.dotbogi?.setPersistedGraph?.(persistedGraph, { render: false });
    this.dotbogi?.setBaseSeriesDisplay?.({
      visible: baseVisible,
      alpha: baseAlpha,
      color: display.activeRun ? display.baseColor : RUN_GRAPH_PRIMARY_COLOR,
      preferPersisted: Boolean(persistedGraph),
    }, { render: false });
    this.dotbogi?.setOverlaySeries?.(display.overlaySeries);
    this.syncDockRangeLabels();
  }

  hydrateRunManagerFromSession({ publish = false } = {}) {
    const payload = this.buildRunManagerBasePayload();
    const runs = Array.isArray(payload?.runs) ? payload.runs : [];
    const filtered = runs.filter((row) => this.runManagerRunRestorable(row));
    this.overlayRuns = filtered.map((row, index) => this.normalizeRunManagerRun(row, index));
    this.overlayRuns.sort((a, b) => normalizeRunManagerLayer(a?.layerIndex, 0) - normalizeRunManagerLayer(b?.layerIndex, 0));
    if (this.activeOverlayRunId && this.findRunManagerIndexById(this.activeOverlayRunId) < 0) {
      this.activeOverlayRunId = "";
    }
    if (this.hoverOverlayRunId && this.findRunManagerIndexById(this.hoverOverlayRunId) < 0) {
      this.hoverOverlayRunId = "";
    }
    if (this.soloOverlayRunId && this.findRunManagerIndexById(this.soloOverlayRunId) < 0) {
      this.soloOverlayRunId = "";
    }
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
    if (publish) {
      this.publishRunManagerSession();
    }
  }

  publishRunManagerSession() {
    if (!this.onOverlaySessionChange) return;
    const base = this.buildRunManagerBasePayload();
    const baseRuns = Array.isArray(base?.runs) ? base.runs : [];
    const lessonId = String(this.lesson?.id ?? "").trim();
    const keepRuns = baseRuns.filter((row) => {
      const sourceLessonId = String(row?.source?.lessonId ?? row?.source?.lesson_id ?? "").trim();
      if (!lessonId) return true;
      return sourceLessonId && sourceLessonId !== lessonId;
    });
    const nextRuns = [...keepRuns, ...this.overlayRuns.map((row) => this.serializeRunManagerRun(row))];
    this.onOverlaySessionChange({
      runs: nextRuns,
      compare: toPlainObject(base?.compare, {}),
      viewCombo: toPlainObject(base?.viewCombo, {}),
    });
  }

  flushOverlaySession() {
    this.publishRunManagerSession();
  }

  setInspectorStatus(message = "", { isError = false } = {}) {
    const text = String(message ?? "").trim();
    this.lastInspectorStatusText = text;
    if (!this.runInspectorStatusEl) return;
    this.runInspectorStatusEl.textContent = text || "저장 작업 대기";
    this.runInspectorStatusEl.dataset.status = text ? (isError ? "error" : "ok") : "idle";
  }

  async handleSaveSnapshot() {
    if (!this.onSaveSnapshot) {
      this.setInspectorStatus("결과 저장 기능이 연결되지 않았습니다.", { isError: true });
      return false;
    }
    try {
      const result = await this.onSaveSnapshot();
      if (result === false) {
        this.setInspectorStatus("결과 저장 실패: 저장할 결과가 없습니다.", { isError: true });
        return false;
      }
      this.setInspectorStatus("결과 저장 완료");
      return true;
    } catch (error) {
      this.setInspectorStatus(`결과 저장 실패: ${String(error?.message ?? error)}`, { isError: true });
      return false;
    }
  }

  async handleSaveSession() {
    if (!this.onSaveSession) {
      this.setInspectorStatus("세션 저장 기능이 연결되지 않았습니다.", { isError: true });
      return false;
    }
    try {
      const result = await this.onSaveSession();
      if (result === false) {
        this.setInspectorStatus("세션 저장 실패: 저장할 세션이 없습니다.", { isError: true });
        return false;
      }
      this.setInspectorStatus("세션 저장 완료");
      return true;
    } catch (error) {
      this.setInspectorStatus(`세션 저장 실패: ${String(error?.message ?? error)}`, { isError: true });
      return false;
    }
  }

  bindRunTabUi() {
    this.runTabButtons?.forEach((button) => {
      button?.addEventListener?.("click", () => {
        this.switchRunTab(button.dataset.runTab);
      });
    });
  }

  bindRunWarningActions() {
    this.runWarningPrimaryBtn?.addEventListener?.("click", () => {
      this.triggerRunWarningPrimaryAction();
    });
    this.runWarningRetryBtn?.addEventListener?.("click", () => {
      void this.restart();
    });
    this.runWarningOpenDdnBtn?.addEventListener?.("click", () => {
      this.focusRunDdnEditor();
      this.switchRunTab("console");
    });
    this.runWarningOpenInspectorBtn?.addEventListener?.("click", () => {
      this.switchRunTab("mirror");
      this.runInspectorMetaEl?.focus?.();
    });
    this.runWarningPlatformLoginBtn?.addEventListener?.("click", () => {
      this.emitPlatformUiAction(PLATFORM_UI_ACTION_LOGIN);
    });
    this.runWarningPlatformRequestAccessBtn?.addEventListener?.("click", () => {
      this.emitPlatformUiAction(PLATFORM_UI_ACTION_REQUEST_ACCESS);
    });
    this.runWarningPlatformOpenLocalSaveBtn?.addEventListener?.("click", () => {
      this.emitPlatformUiAction(PLATFORM_UI_ACTION_OPEN_LOCAL_SAVE);
    });
    this.runWarningTechEl?.addEventListener?.("toggle", () => {
      this.syncRunWarningCodeVisibility();
    });
  }

  syncRunWarningCodeVisibility() {
    if (!this.runWarningCodesEl) return;
    const hasCodes = String(this.runWarningCodesEl.innerHTML ?? "").trim().length > 0;
    const open = Boolean(this.runWarningTechEl?.open);
    this.runWarningCodesEl.classList.toggle("hidden", !(hasCodes && open));
  }

  setRunWarningActionRecommendation(button, recommended) {
    if (!button) return;
    const isRecommended = Boolean(recommended);
    if (button.dataset) {
      button.dataset.recommended = isRecommended ? "true" : "false";
    }
    button.classList?.toggle?.("run-warning-action-primary", isRecommended);
  }

  applyRunWarningPlatformActionState(button, actionModel, fallbackTitle) {
    if (!button) return;
    const model = actionModel && typeof actionModel === "object" ? actionModel : {};
    const hidden = Boolean(model.hidden);
    const disabled = hidden || Boolean(model.disabled);
    button.classList?.toggle?.("hidden", hidden);
    button.disabled = disabled;
    button.title = String(model.title ?? fallbackTitle ?? "").trim();
    this.setRunWarningActionRecommendation(button, !hidden && Boolean(model.recommended));
  }

  triggerRunWarningPrimaryAction() {
    const kind = String(this.lastWarningPrimaryActionKind ?? "").trim().toLowerCase();
    if (kind === "autofix") {
      void this.applyLegacyAutofix();
      return true;
    }
    if (kind === "open_ddn" || kind === "manual_fix_example") {
      this.focusRunDdnEditor();
      this.switchRunTab("console");
      return true;
    }
    if (kind === "open_inspector") {
      this.switchRunTab("mirror");
      this.runInspectorMetaEl?.focus?.();
      return true;
    }
    void this.restart();
    return true;
  }

  emitStudioRunMetric(name, payload = {}) {
    const metricName = String(name ?? "").trim();
    if (!metricName || typeof window === "undefined") return;
    const record = {
      name: metricName,
      ts: Date.now(),
      ...((payload && typeof payload === "object") ? payload : {}),
    };
    try {
      if (!Array.isArray(window.__SEAMGRIM_STUDIO_METRICS__)) {
        window.__SEAMGRIM_STUDIO_METRICS__ = [];
      }
      window.__SEAMGRIM_STUDIO_METRICS__.push(record);
    } catch (_) {
      // ignore metric buffer failures
    }
    try {
      if (typeof window.dispatchEvent === "function" && typeof CustomEvent === "function") {
        window.dispatchEvent(new CustomEvent("seamgrim:studio-metric", { detail: record }));
      }
    } catch (_) {
      // ignore metric event failures
    }
  }

  emitPlatformUiAction(action) {
    const token = String(action ?? "").trim();
    if (!token) return false;
    if (typeof window === "undefined") return false;
    const ddnText = String(this.currentDdn || this.runDdnPreviewEl?.value || "");
    try {
      if (typeof window?.dispatchEvent === "function" && typeof CustomEvent === "function") {
        window.dispatchEvent(
          new CustomEvent(PLATFORM_UI_ACTION_EVENT, {
            detail: {
              action: token,
              source: "run_warning_panel",
              lessonId: String(this.lesson?.id ?? "").trim() || null,
              ddnText,
              stateHash: String(this.lastRuntimeHash ?? "").trim() || null,
            },
          }),
        );
      }
    } catch (_) {
      return false;
    }
    return true;
  }

  isRunWarningPanelVisible() {
    return Array.isArray(this.lastParseWarnings) && this.lastParseWarnings.length > 0;
  }

  triggerRunWarningShortcut(action) {
    const kind = String(action ?? "").trim().toLowerCase();
    if (!this.isRunWarningPanelVisible()) return false;
    if (kind === "ddn") {
      this.focusRunDdnEditor();
      this.switchRunTab(SUBPANEL_TAB.MAEGIM);
      return true;
    }
    if (kind === "inspector") {
      this.switchRunTab(SUBPANEL_TAB.MIRROR);
      this.runInspectorMetaEl?.focus?.();
      return true;
    }
    return false;
  }

  handleObserveCardAction(action, payload = {}) {
    const code = normalizeObserveAction(action);
    if (!code) return false;
    const plan = buildObserveActionPlan(code, payload);
    if (!plan) return false;
    if (plan.kind === "open-ddn-token") {
      return this.openObserveOutputRowsInDdn(payload);
    }
    return false;
  }

  focusDdnToken(tokenCandidates = []) {
    const textarea = this.runDdnPreviewEl;
    if (!textarea) return null;
    const source = String(this.baseDdn || textarea?.value || "");
    if (!source) return null;
    const candidates = Array.isArray(tokenCandidates) ? tokenCandidates : [tokenCandidates];
    let foundIndex = -1;
    let foundToken = "";
    for (const raw of candidates) {
      const token = String(raw ?? "").trim();
      if (!token) continue;
      const idx = source.indexOf(token);
      if (idx < 0) continue;
      if (foundIndex < 0 || idx < foundIndex) {
        foundIndex = idx;
        foundToken = token;
      }
    }
    if (foundIndex < 0) return null;
    const start = foundIndex;
    const end = foundIndex + foundToken.length;
    textarea?.focus?.();
    try {
      textarea?.setSelectionRange?.(start, end, "forward");
    } catch (_) {
      try {
        textarea.selectionStart = start;
        textarea.selectionEnd = end;
      } catch (_) {
        // ignore selection errors
      }
    }
    const line = source.slice(0, start).split(/\r?\n/).length;
    return {
      token: foundToken,
      start,
      end,
      line,
    };
  }

  openObserveOutputRowsInDdn(payload = {}) {
    this.switchRunTab("output");
    const token = String(payload?.observeToken ?? "").trim();
    const focused = this.focusDdnToken([
      token,
      "table.row",
      "보임 {",
      "보임{",
      "보임",
    ]);
    this.setObserveGuideStatus(
      focused
        ? `교과 원문으로 이동했습니다. L${Math.max(1, Number(focused?.line || 1))}에서 보임표 행 항목을 점검하세요.`
        : "교과 원문으로 이동했습니다. 보임 출력 항목을 점검하세요.",
      { ttlMs: 4200 },
    );
    return true;
  }

  focusObserveFamily(family) {
    const code = String(family ?? "").trim().toLowerCase();
    if (!code) return false;
    const guideMessage = this.resolveObserveFamilyGuideMessage(code);
    if (guideMessage) {
      this.setObserveGuideStatus(guideMessage);
    }
    if (code === "space2d") {
      this.setPrimaryView("sim");
      this.switchRunTab("graph");
      this.dockTarget = "space2d";
      if (this.dockTargetSelectEl) {
        this.dockTargetSelectEl.value = "space2d";
      }
      this.syncDockGuideToggles();
      this.bogaeAreaEl?.scrollIntoView?.({ block: "nearest", inline: "nearest", behavior: "smooth" });
      return true;
    }
    if (code === "graph") {
      this.setPrimaryView("graph");
      this.switchRunTab("graph");
      this.dockTarget = "graph";
      if (this.dockTargetSelectEl) {
        this.dockTargetSelectEl.value = "graph";
      }
      this.ensureGraphAxisSelection();
      this.syncDockGuideToggles();
      this.graphPanelEl?.scrollIntoView?.({ block: "nearest", inline: "nearest", behavior: "smooth" });
      this.root?.querySelector?.("#select-x-axis")?.focus?.();
      return true;
    }
    if (code === "table") {
      this.setPrimaryView("graph");
      this.switchRunTab("graph");
      this.runtimeTablePanelEl?.scrollIntoView?.({ block: "nearest", inline: "nearest", behavior: "smooth" });
      return true;
    }
    if (code === "text" || code === "structure") {
      this.setPrimaryView("graph");
      this.switchRunTab("graph");
      this.runtimeTextPanelEl?.scrollIntoView?.({ block: "nearest", inline: "nearest", behavior: "smooth" });
      return true;
    }
    return false;
  }

  ensureGraphAxisSelection() {
    const xSelect = this.root?.querySelector?.("#select-x-axis");
    const ySelect = this.root?.querySelector?.("#select-y-axis");
    if (!xSelect || !ySelect) return false;
    const xOptions = readSelectOptionValues(xSelect);
    const yOptions = readSelectOptionValues(ySelect);
    if (!yOptions.length) return false;

    let nextY = normalizeAxisToken(ySelect.value);
    if (!nextY || !yOptions.includes(nextY)) {
      nextY = yOptions[0];
    }
    let nextX = normalizeAxisToken(xSelect.value);
    const preferredX = pickPreferredGraphXAxis(xOptions, nextY);
    if (!nextX || !xOptions.includes(nextX) || normalizeAxisToken(nextX).toLowerCase() === normalizeAxisToken(nextY).toLowerCase()) {
      nextX = preferredX || nextX;
    }

    if (!nextX || !nextY) return false;
    if (typeof this.dotbogi?.setSelectedAxes === "function") {
      this.dotbogi.setSelectedAxes({ xKey: nextX, yKey: nextY });
    } else {
      xSelect.value = nextX;
      ySelect.value = nextY;
    }
    return true;
  }

  setLegacyAutofixStatus(message, { status = "idle" } = {}) {
    if (!this.runLegacyAutofixStatusEl) return;
    this.runLegacyAutofixStatusEl.textContent = String(message ?? "").trim() || "자동수정 대기";
    this.runLegacyAutofixStatusEl.dataset.status = String(status ?? "idle").trim() || "idle";
  }

  setRunLocalSaveStatus(message, { status = "idle" } = {}) {
    if (!this.runLocalSaveStatusEl) return;
    this.runLocalSaveStatusEl.textContent = String(message ?? "").trim() || "저장 대기";
    this.runLocalSaveStatusEl.dataset.status = String(status ?? "idle").trim() || "idle";
  }

  focusRunDdnEditor() {
    this.runDdnPreviewEl?.focus?.();
    return Boolean(this.runDdnPreviewEl);
  }

  async handleRunLocalLoad() {
    const file = await pickDdnFileFromLocal(this.runLoadFileInputEl);
    if (!file) return false;
    return this.applyRunLoadedFile(file);
  }

  async applyRunLoadedFile(file) {
    try {
      const text = await readTextFromLocalFile(file);
      this.applyBaseDdnText(text, { preserveControlValues: false, restart: false });
      const fileName = String(file?.name ?? "파일").trim() || "파일";
      const sourceLabel = fileName.replace(/\.[^.]+$/, "") || fileName;
      this.setStudioShellState({
        sourceKind: "file",
        sourceLabel,
        engineStatus: "idle",
      });
      this.setRunLocalSaveStatus(`불러오기 완료: ${fileName}`, { status: "ok" });
      this.lastExecPathHint = "불러온 DDN입니다. 실행을 눌러 결과를 확인하세요.";
      this.updateRuntimeHint();
      this.onSourceChange?.(text);
      return true;
    } catch (error) {
      this.setRunLocalSaveStatus(`불러오기 실패: ${String(error?.message ?? error)}`, { status: "error" });
      return false;
    }
  }

  async handleRunLocalSave() {
    const ddnText = String(this.currentDdn || this.runDdnPreviewEl?.value || this.baseDdn || "");
    if (!ddnText.trim()) {
      this.setRunLocalSaveStatus("저장할 DDN 없음", { status: "warn" });
      return false;
    }
    if (this.onSaveDdn) {
      try {
        const ok = await this.onSaveDdn(ddnText);
        this.setRunLocalSaveStatus(ok === false ? "저장 실패" : "저장 완료", {
          status: ok === false ? "error" : "ok",
        });
        return ok !== false;
      } catch (error) {
        this.setRunLocalSaveStatus(`저장 실패: ${String(error?.message ?? error)}`, { status: "error" });
        return false;
      }
    }
    try {
      const fallbackName = String(this.sourceLabel || this.lesson?.id || "lesson").trim() || "lesson";
      saveDdnToFile(ddnText, `${fallbackName}.ddn`);
      this.setRunLocalSaveStatus("파일 저장 완료", { status: "ok" });
      return true;
    } catch (error) {
      this.setRunLocalSaveStatus(`저장 실패: ${String(error?.message ?? error)}`, { status: "error" });
      return false;
    }
  }

  async handleMainExecutionControl(kind = "run") {
    const action = String(kind ?? "").trim().toLowerCase();
    if (action === "pause") {
      this.resolveRuntimeMaxMadiLimit();
      this.executionPaused = true;
      this.lastExecPathHint = "실행 일시정지 상태입니다. 다시 실행하면 이어서 동작합니다.";
      this.syncLoopState();
      this.setEngineStatus("paused");
      this.updateRuntimeHint();
      return true;
    }
    if (this.engineStatus === "paused" && this.executionPaused && this.lastState) {
      if (hasReachedRuntimeMaxMadi(this.runtimeTickCounter, this.resolveRuntimeMaxMadiLimit())) {
        this.executionPaused = true;
        this.haltLoop();
        this.lastExecPathHint = "실행 완료";
        this.setEngineStatus("done");
        this.updateRuntimeHint();
        return false;
      }
      this.executionPaused = false;
      this.lastExecPathHint = "일시정지에서 실행을 재개했습니다.";
      this.syncLoopState();
      this.setEngineStatus("running");
      this.updateRuntimeHint();
      return true;
    }
    this.resolveRuntimeMaxMadiLimit();
    this.executionPaused = false;
    return this.restart();
  }

  setRunOnboardingStatus(message, { status = "idle" } = {}) {
    if (this.runOnboardingStatusEl) {
      this.runOnboardingStatusEl.textContent = String(message ?? "").trim() || DEFAULT_ONBOARDING_STATUS_TEXT;
      this.runOnboardingStatusEl.dataset.status = String(status ?? "idle").trim() || "idle";
    }
    this.syncRunPresetRail();
    this.syncRunActionRail();
  }

  buildRunPresetRailModel() {
    const requiredViews = resolveLessonRequiredViewsForRun(this.lesson);
    const layoutMode = String(this.lessonLayoutProfile?.mode ?? this.root?.dataset?.runLayoutMode ?? "split").trim() || "split";
    const numericTrackPreset = buildNumericTrackRunPreset(this.lesson);
    return {
      schema: "seamgrim.run_preset_rail.v1",
      lesson_id: String(this.lesson?.id ?? "").trim(),
      launch_kind: normalizeRunLaunchKind(this.lastLaunchKind),
      launch_label: formatRunLaunchKindLabel(this.lastLaunchKind),
      onboarding_profile: normalizeRunOnboardingProfile(this.lastOnboardingProfile),
      onboarding_label: formatRunOnboardingProfileLabel(this.lastOnboardingProfile),
      layout_mode: layoutMode,
      layout_label: formatRunLayoutModeLabel(layoutMode),
      required_views: requiredViews,
      required_views_label: formatRunRequiredViewsLabel(requiredViews),
      numeric_track: Boolean(numericTrackPreset),
      numeric_track_preset: numericTrackPreset,
      numeric_track_label: numericTrackPreset?.label ?? "",
    };
  }

  syncRunPresetRail() {
    const model = this.buildRunPresetRailModel();
    if (this.root?.dataset) {
      this.root.dataset.launchKind = model.launch_kind || "manual";
    }
    if (this.runPresetLaunchKindEl) {
      this.runPresetLaunchKindEl.textContent = model.launch_label;
      this.runPresetLaunchKindEl.dataset.value = model.launch_kind;
    }
    if (this.runPresetOnboardingEl) {
      this.runPresetOnboardingEl.textContent = model.onboarding_label;
      this.runPresetOnboardingEl.dataset.value = model.onboarding_profile || "default";
    }
    if (this.runPresetLayoutEl) {
      this.runPresetLayoutEl.textContent = model.layout_label;
      this.runPresetLayoutEl.dataset.value = model.layout_mode;
    }
    if (this.runPresetViewsEl) {
      this.runPresetViewsEl.textContent = model.required_views_label;
      this.runPresetViewsEl.dataset.value = model.required_views.join(",");
    }
    if (this.runPresetNumericTrackEl) {
      this.runPresetNumericTrackEl.textContent = model.numeric_track_label || "결과 기록: -";
      this.runPresetNumericTrackEl.dataset.value = model.numeric_track_preset?.focus ?? "";
      this.runPresetNumericTrackEl.classList.toggle("hidden", !model.numeric_track);
    }
    if (this.runPresetRailEl) {
      this.runPresetRailEl.dataset.launchKind = model.launch_kind;
      this.runPresetRailEl.dataset.onboardingProfile = model.onboarding_profile || "default";
      this.runPresetRailEl.dataset.layoutMode = model.layout_mode;
      this.runPresetRailEl.dataset.numericTrack = model.numeric_track ? "1" : "0";
    }
    this.syncClassroomModeSwitch(model);
    try {
      window.__SEAMGRIM_RUN_PRESET_RAIL__ = model;
      window.__SEAMGRIM_NUMERIC_TRACK_RUN_PRESET__ = model.numeric_track_preset ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RUN_PRESET_TEXT__ = model.numeric_track_preset
        ? formatNumericTrackRunPresetText(model.numeric_track_preset)
        : "";
    } catch (_) {
      // ignore browser instrumentation errors
    }
    return model;
  }

  syncClassroomModeSwitch(model = this.buildRunPresetRailModel()) {
    const profile = normalizeRunOnboardingProfile(model?.onboarding_profile);
    const switchVisible = this.classroomModeAccess !== "student";
    if (this.root?.dataset) {
      this.root.dataset.onboardingProfile = profile || "default";
      this.root.dataset.classroomModeAccess = this.classroomModeAccess || "default";
    }
    if (this.classroomModeSwitchEl) {
      this.classroomModeSwitchEl.dataset.mode = profile || "default";
      this.classroomModeSwitchEl.dataset.access = this.classroomModeAccess || "default";
      this.classroomModeSwitchEl.classList.toggle("hidden", !switchVisible);
      this.classroomModeSwitchEl.setAttribute("aria-hidden", switchVisible ? "false" : "true");
    }
    (Array.isArray(this.classroomModeBtns) ? this.classroomModeBtns : []).forEach((button) => {
      const mode = normalizeRunOnboardingProfile(button?.dataset?.classroomMode);
      const active = Boolean(mode && mode === profile);
      button.classList.toggle("active", active);
      button.setAttribute("aria-pressed", active ? "true" : "false");
      button.disabled = !switchVisible;
    });
  }

  applyClassroomModeSwitch(profile) {
    const normalized = normalizeRunOnboardingProfile(profile);
    if (!normalized) return false;
    const applied = this.applyRunOnboardingProfile(normalized);
    const rail = this.buildRunPresetRailModel();
    const payload = {
      schema: "seamgrim.classroom_mode_switch.v1",
      mode: normalized,
      applied,
      rail_onboarding_profile: rail.onboarding_profile,
      rail_onboarding_label: rail.onboarding_label,
      launch_kind: rail.launch_kind,
      student_active: normalized === "student",
      teacher_active: normalized === "teacher",
      account_required: false,
      cloud_sync: false,
      permission_system: false,
    };
    try {
      window.__STUDIO_CLASSROOM_MODE_SWITCH__ = payload;
    } catch (_) {
      // ignore browser instrumentation errors
    }
    return applied;
  }

  syncNumericTrackRunResultLink(link = null) {
    const row = link && typeof link === "object" ? link : null;
    if (this.runResultNumericLinkEl) {
      this.runResultNumericLinkEl.classList.toggle("hidden", !row);
      this.runResultNumericLinkEl.dataset.value = row?.state_hash ?? "";
      this.runResultNumericLinkEl.textContent = row
        ? `결과기록: ${row.run_kind || "run"} · ${row.state_hash ? row.state_hash.slice(0, 12) : "기록ID 없음"}`
        : "결과기록: -";
    }
    try {
      window.__SEAMGRIM_NUMERIC_TRACK_RUN_RESULT_LINK__ = row;
      window.__SEAMGRIM_NUMERIC_TRACK_RUN_RESULT_LINK_TEXT__ = row
        ? formatNumericTrackRunResultLinkText(row)
        : "";
    } catch (_) {
      // ignore browser instrumentation errors
    }
  }

  readPlatformServerAdapterSnapshot() {
    if (typeof window === "undefined") {
      return {
        errorCode: "",
        actionRail: [],
      };
    }
    const response = window.__SEAMGRIM_PLATFORM_SERVER_LAST_RESPONSE__;
    const actionRail = normalizePlatformActionRail(window.__SEAMGRIM_PLATFORM_SERVER_LAST_ACTION_RAIL__);
    return {
      errorCode: String(response?.error?.code ?? "").trim(),
      actionRail,
    };
  }

  applyPlatformServerAdapterExchange(detail = null) {
    const snapshot = this.readPlatformServerAdapterSnapshot();
    const payload = detail && typeof detail === "object" ? detail : {};
    const response = payload.response && typeof payload.response === "object" ? payload.response : null;
    const errorCode = String(response?.error?.code ?? snapshot.errorCode ?? "").trim();
    const actionRail = normalizePlatformActionRail(
      Array.isArray(payload.action_rail) ? payload.action_rail : snapshot.actionRail,
    );
    this.lastPlatformServerErrorCode = errorCode;
    this.lastPlatformServerActionRail = actionRail;
    this.renderParseWarningPanel();
    this.syncRunActionRail();
  }

  syncRunActionRail() {
    if (!this.runActionRailEl) return;
    const model = buildRunActionRailViewModel({
      warnings: this.lastParseWarnings,
      onboardingProfile: this.lastOnboardingProfile,
      onboardingStatusText: this.runOnboardingStatusEl?.textContent ?? "",
      platformErrorCode: this.lastPlatformServerErrorCode,
      platformActionRail: this.lastPlatformServerActionRail,
    });
    if (this.runActionRailStatusEl) {
      this.runActionRailStatusEl.textContent = String(model.statusText ?? "");
      this.runActionRailStatusEl.dataset.status = String(model.statusLevel ?? "idle");
    }
    const applyRecommended = (el, recommended) => {
      if (!el) return;
      const on = Boolean(recommended);
      el.dataset.recommended = on ? "true" : "false";
      if (on) {
        el.title = "권장 액션";
      } else if (el.title === "권장 액션") {
        el.title = "";
      }
    };
    const applyVisible = (el, visible) => {
      if (!el) return;
      el.classList.toggle("hidden", visible === false);
    };
    applyVisible(this.runActionRailOnboardStudentBtn, model?.actions?.onboardStudent?.visible !== false);
    applyVisible(this.runActionRailOnboardTeacherBtn, model?.actions?.onboardTeacher?.visible !== false);
    applyVisible(this.runActionRailOpenDdnBtn, model?.actions?.openDdn?.visible !== false);
    applyVisible(this.runActionRailOpenInspectorBtn, model?.actions?.openInspector?.visible !== false);
    applyRecommended(this.runActionRailOnboardStudentBtn, model?.actions?.onboardStudent?.recommended);
    applyRecommended(this.runActionRailOnboardTeacherBtn, model?.actions?.onboardTeacher?.recommended);
    applyRecommended(this.runActionRailOpenDdnBtn, model?.actions?.openDdn?.recommended);
    applyRecommended(this.runActionRailOpenInspectorBtn, model?.actions?.openInspector?.recommended);
  }

  clearObserveGuideStatusTimer() {
    if (!this.observeGuideStatusTimer) return;
    clearTimeout(this.observeGuideStatusTimer);
    this.observeGuideStatusTimer = null;
  }

  readObserveGuideStatusText() {
    const text = String(this.observeGuideStatusText ?? "").trim();
    if (!text) return "";
    const expireAt = Number(this.observeGuideStatusExpireAt ?? 0);
    if (!Number.isFinite(expireAt) || expireAt <= 0) {
      return text;
    }
    if (Date.now() <= expireAt) {
      return text;
    }
    this.observeGuideStatusText = "";
    this.observeGuideStatusExpireAt = 0;
    this.clearObserveGuideStatusTimer();
    return "";
  }

  setObserveGuideStatus(message, { ttlMs = OBSERVE_GUIDE_STATUS_TTL_MS } = {}) {
    const text = String(message ?? "").trim();
    this.observeGuideStatusText = text;
    this.clearObserveGuideStatusTimer();
    if (!text) {
      this.observeGuideStatusExpireAt = 0;
      this.updateRuntimeHint();
      return;
    }
    const ttl = Number(ttlMs);
    if (Number.isFinite(ttl) && ttl > 0) {
      this.observeGuideStatusExpireAt = Date.now() + Math.trunc(ttl);
      this.observeGuideStatusTimer = setTimeout(() => {
        if (Date.now() < Number(this.observeGuideStatusExpireAt ?? 0)) return;
        this.observeGuideStatusText = "";
        this.observeGuideStatusExpireAt = 0;
        this.observeGuideStatusTimer = null;
        this.updateRuntimeHint();
      }, Math.trunc(ttl));
    } else {
      this.observeGuideStatusExpireAt = 0;
    }
    this.updateRuntimeHint();
  }

  resolveObserveFamilyGuideMessage(family) {
    const code = String(family ?? "").trim().toLowerCase();
    if (!code) return "";
    const rows = readObserveFamilyRows(this.lastRuntimeDerived);
    const row = rows.find((item) => String(item?.family ?? "").trim().toLowerCase() === code) ?? null;
    if (row) {
      return buildObserveFamilyActionHint({
        family: code,
        available: Boolean(row.available),
        strict: Boolean(row.strict),
      });
    }
    return buildObserveFamilyActionHint({
      family: code,
      available: true,
      strict: true,
    });
  }

  applyRunOnboardingProfile(profile, { persist = true } = {}) {
    const normalized = normalizeRunOnboardingProfile(profile);
    if (!normalized) {
      this.lastOnboardingProfile = "";
      this.setRunOnboardingStatus(DEFAULT_ONBOARDING_STATUS_TEXT, { status: "idle" });
      this.syncRunPresetRail();
      return false;
    }
    const isStudent = normalized === "student";
    this.lastOnboardingProfile = normalized;
    if (isStudent) {
      this.dockTarget = "space2d";
      if (this.dockTargetSelectEl) {
        this.dockTargetSelectEl.value = "space2d";
      }
      this.switchRunTab(SUBPANEL_TAB.OVERLAY);
      this.setRunOnboardingStatus(`학생 시작 적용: 설명 + 그림 점검 · ${SEAMGRIM_FIRST_RUN_PATH_TEXT}`, { status: "ok" });
    } else {
      this.applyTeacherPackageFocus();
      this.setRunOnboardingStatus(`교사용 배포 준비 적용: 배포 묶음 + 관찰 점검 · ${SEAMGRIM_FIRST_RUN_PATH_TEXT}`, { status: "ok" });
    }
    this.syncDockGuideToggles();
    this.applyDockGuideToggles();
    this.syncRunPresetRail();
    this.syncClassroomReportExport();
    this.syncLocalPackageExport();
    this.syncPublicationPrepExport();
    this.syncRegistrySeedExport();
    this.syncApprovalContinuityExport();
    this.syncBenchmarkLtsExport();
    this.syncEducationOperationsLtsExport();

    if (persist) {
      const lessonId = String(this.lesson?.id ?? "").trim();
      if (lessonId) {
        const pref = this.getLessonUiPref(lessonId, { create: true });
        pref.lastOnboardingProfile = normalized;
        this.persistUiPrefs();
      }
    }
    return true;
  }

  applyTeacherPackageFocus() {
    if (normalizeRunOnboardingProfile(this.lastOnboardingProfile) !== "teacher") return false;
    const moved = this.focusObserveFamily("graph");
    if (typeof this.root?.querySelector !== "function") {
      return moved;
    }
    this.switchRunTab(SUBPANEL_TAB.MIRROR);
    const tools = this.root?.querySelector?.("#run-inspector-tools");
    if (tools && typeof tools.open === "boolean") {
      tools.open = true;
    }
    return true;
  }

  syncLegacyAutofixAvailability() {
    const source = String(this.baseDdn || this.runDdnPreviewEl?.value || "");
    const enabled = hasLegacyAutofixCandidate(source);
    if (this.runLegacyAutofixBtn) {
      this.runLegacyAutofixBtn.disabled = !enabled;
      this.runLegacyAutofixBtn.title = enabled
        ? "레거시 #범위/콜론 헤더를 current 표면으로 변환합니다."
        : "적용 가능한 레거시 패턴이 없습니다.";
    }
    if (enabled) {
      this.setLegacyAutofixStatus("자동수정 가능", { status: "warn" });
    } else {
      this.setLegacyAutofixStatus("자동수정 대기", { status: "idle" });
    }
    return enabled;
  }

  async applyLegacyAutofix() {
    const source = String(this.baseDdn || this.runDdnPreviewEl?.value || "");
    if (!source.trim()) {
      this.setLegacyAutofixStatus("자동수정 대상 없음", { status: "idle" });
      return false;
    }
    const result = applyLegacyAutofixToDdn(source);
    if (!result.changed) {
      const skipped = Number(result?.stats?.range_skipped || 0) + Number(result?.stats?.range_hash_skipped || 0);
      if (skipped > 0) {
        this.setLegacyAutofixStatus(`자동수정 보류: 수동 검토 ${skipped}건`, { status: "warn" });
      } else {
        this.setLegacyAutofixStatus("자동수정: 변경 없음", { status: "idle" });
      }
      this.syncLegacyAutofixAvailability();
      return false;
    }
    this.applyBaseDdnText(result.text, { preserveControlValues: true, restart: false });
    this.setLegacyAutofixStatus(
      `자동수정 적용: ${result.total_changes}건`,
      { status: "ok" },
    );
    this.switchRunTab("console");
    this.focusRunDdnEditor();
    await this.restart();
    const remaining = this.syncLegacyAutofixAvailability();
    if (!remaining) {
      this.setLegacyAutofixStatus("자동수정 적용 완료", { status: "ok" });
    }
    return true;
  }

  switchRunTab(tabId) {
    const requested = normalizeRunTab(tabId);
    const allowedTabs = resolveSubpanelTabs(this.primaryView);
    const next = allowedTabs.includes(requested) ? requested : SUBPANEL_TAB.GRAPH;
    const isOverlayPanelMode = this.studioViewMode === STUDIO_VIEW_MODE_FULL;
    if (isOverlayPanelMode && this.activeRunTab === next) {
      this.fullModePanelOpen = !this.fullModePanelOpen;
    } else {
      this.activeRunTab = next;
      this.fullModePanelOpen = isOverlayPanelMode ? true : this.fullModePanelOpen;
    }
    RUN_TAB_IDS.forEach((tab) => {
      const button = this.root?.querySelector?.(`#run-tab-btn-${tab}`);
      const panel = this.runTabPanels?.get(tab);
      button?.classList?.toggle("active", tab === next);
      button?.classList?.toggle("hidden", !allowedTabs.includes(tab));
      panel?.classList?.toggle("hidden", tab !== next);
    });
    if (this.subpanelEl?.dataset) {
      this.subpanelEl.dataset.panelOpen = isOverlayPanelMode ? (this.fullModePanelOpen ? "1" : "0") : "1";
    }
    if (next === SUBPANEL_TAB.GRAPH && this.dotbogi) {
      requestAnimationFrame(() => { this.dotbogi.renderGraph(); });
    }
    this.renderGraphTabMode();
    this.setStudioShellState({ activeSubpanelTab: next });
    return next;
  }

  setLessonOptions(lessons = []) {
    const rows = Array.isArray(lessons) ? lessons : [];
    this.lessonOptions = rows
      .map((row, sourceOrder) => ({
        id: String(row?.id ?? "").trim(),
        title: String(row?.title ?? row?.id ?? "").trim(),
        localPackageId: String(row?.localPackageId ?? "").trim(),
        localPackageLessonId: String(row?.localPackageLessonId ?? row?.id ?? "").trim(),
        localPackageLessonIndex: Number.isFinite(Number(row?.localPackageLessonIndex))
          ? Math.max(0, Math.trunc(Number(row.localPackageLessonIndex)))
          : 0,
        localPackageSourceOrder: sourceOrder,
      }))
      .filter((row) => row.id)
      .sort((a, b) => {
        if (a.localPackageId && a.localPackageId === b.localPackageId) {
          const aIndex = Number(a.localPackageLessonIndex) || 0;
          const bIndex = Number(b.localPackageLessonIndex) || 0;
          if (aIndex !== bIndex) return aIndex - bIndex;
          return (Number(a.localPackageSourceOrder) || 0) - (Number(b.localPackageSourceOrder) || 0);
        }
        return String(a.title || a.id).localeCompare(String(b.title || b.id), "ko");
      });
    this.syncPackageLessonSwitcher();
  }

  getPackageLessonOptions() {
    const packageId = String(this.lesson?.localPackageId ?? "").trim();
    if (!packageId) return [];
    const options = this.lessonOptions.filter((row) => String(row.localPackageId ?? "").trim() === packageId);
    const packageLessons = Array.isArray(this.lesson?.localPackagePayload?.lessons)
      ? this.lesson.localPackagePayload.lessons
      : [];
    const order = new Map();
    packageLessons.forEach((row, index) => {
      const lessonId = String(row?.lesson_id ?? "").trim();
      if (lessonId && !order.has(lessonId)) {
        order.set(lessonId, index + 1);
      }
    });
    if (!order.size) return options;
    return options.slice().sort((a, b) => {
      const aOrder = order.get(String(a.localPackageLessonId ?? "").trim()) ?? Number.MAX_SAFE_INTEGER;
      const bOrder = order.get(String(b.localPackageLessonId ?? "").trim()) ?? Number.MAX_SAFE_INTEGER;
      if (aOrder !== bOrder) return aOrder - bOrder;
      return String(a.title || a.id).localeCompare(String(b.title || b.id), "ko");
    });
  }

  syncPackageLessonSwitcher() {
    const select = this.runPackageLessonSelectEl;
    const wrapper = this.runPackageLessonSwitchEl;
    if (!select || !wrapper) return;
    const options = this.getPackageLessonOptions();
    const show = options.length > 1;
    wrapper.classList.toggle("hidden", !show);
    wrapper.setAttribute("aria-hidden", show ? "false" : "true");
    select.disabled = !show;
    if (!show) {
      select.innerHTML = "";
      return;
    }
    const currentId = String(this.lesson?.id ?? "").trim();
    select.innerHTML = options.map((row, index) => {
      const label = `${index + 1}. ${row.title || row.localPackageLessonId || row.id}`;
      return `<option value="${escapeHtml(row.id)}">${escapeHtml(label)}</option>`;
    }).join("");
    select.value = options.some((row) => row.id === currentId) ? currentId : options[0]?.id ?? "";
  }

  async handlePackageLessonSelect() {
    const nextId = String(this.runPackageLessonSelectEl?.value ?? "").trim();
    if (!nextId || nextId === String(this.lesson?.id ?? "").trim()) return false;
    if (!this.onSelectLesson) {
      this.lastExecPathHint = "배포 묶음 수업 전환 기능이 연결되지 않았습니다.";
      this.updateRuntimeHint();
      return false;
    }
    try {
      await this.onSelectLesson(nextId);
      return true;
    } catch (error) {
      this.lastExecPathHint = `배포 묶음 수업 전환 실패: ${String(error?.message ?? error)}`;
      this.updateRuntimeHint();
      return false;
    }
  }

  async resetToLessonSource() {
    const lessonId = String(this.lesson?.id ?? "").trim();
    if (!lessonId) return false;
    if (this.onSelectLesson) {
      try {
        await this.onSelectLesson(lessonId);
        return true;
      } catch (error) {
        this.lastExecPathHint = `교과 초기화 실패: ${String(error?.message ?? error)}`;
        this.updateRuntimeHint();
        return false;
      }
    }
    this.baseDdn = String(this.lesson?.ddnText ?? this.baseDdn ?? "");
    if (this.runDdnPreviewEl) {
      this.runDdnPreviewEl.value = this.baseDdn;
    }
    this.syncLegacyAutofixAvailability();
    await this.restart();
    return true;
  }

  applyBaseDdnText(nextDdn = "", { preserveControlValues = true, restart = true } = {}) {
    this.baseDdn = String(nextDdn ?? "");
    this.currentDdn = this.baseDdn;
    if (this.lesson && typeof this.lesson === "object") {
      this.lesson.ddnText = this.baseDdn;
    }
    if (this.runDdnPreviewEl) {
      this.runDdnPreviewEl.value = this.baseDdn;
    }
    this.setRunLocalSaveStatus("저장 필요", { status: "warn" });
    this.syncLegacyAutofixAvailability();
    const parsed = this.sliderPanel.parseFromDdn(this.baseDdn, {
      preserveValues: Boolean(preserveControlValues),
      maegimControlJson: this.lesson?.maegimControlJson ?? "",
    });
    this.syncRunSliderAreaVisibility();
    this.syncGraphDraftFromParsed(parsed);
    this.setEngineMode(resolveRunEngineModeFromDdnText(`${this.baseDdn}\n${this.getEffectiveWasmSource(this.baseDdn)}`));
    this.scheduleInlineWarningRefresh(this.baseDdn);
    this.onSourceChange?.(this.baseDdn);
    if (restart) {
      void this.restart();
    } else {
      this.markRuntimeDirtyForSourceEdit();
    }
  }

  markRuntimeDirtyForSourceEdit({ showFallbackGrid = false } = {}) {
    this.haltLoop();
    this.executionPaused = true;
    this.playbackPaused = true;
    this.serverPlayback = null;
    this.setEngineMode(resolveRunEngineModeFromDdnText(`${this.baseDdn}\n${this.getEffectiveWasmSource(this.baseDdn)}`));
    if (this.wasmState && typeof this.wasmState === "object") {
      this.wasmState.client = null;
    }
    this.lastState = null;
    this.lastRuntimeDerived = null;
    this.runtimeTickCounter = 0;
    this.runtimeMaxMadi = 0;
    this.runtimeTimeValue = null;
    this.clearRunErrorBanner();
    this.setParseWarnings([]);
    this.setHash("-");
    this.setRuntimePreviewViewModel(null);
    this.updateObserveSummary({ observation: null, views: null, outputRows: [] });
    this.renderOverlayTabContent(this.lastOverlayMarkdown, { sourceLabel: this.lastOverlayMarkdown ? "교과 설명" : "" });
    this.lastSpace2dMode = showFallbackGrid ? "fallback" : "none";
    if (showFallbackGrid) {
      this.renderMainVisual({ mode: "console-grid", consoleLinesForGrid: [] });
    } else {
      this.renderMainVisual({ mode: "none" });
    }
    this.lastExecPathHint = "소스 수정됨: 재실행 필요";
    this.syncDockRangeLabels();
    this.syncDockTimeUi();
    this.setEngineStatus("idle");
    this.updateRuntimeHint();
  }

  resetRuntimeForExplicitRun({ execPathHint = "실행 대기" } = {}) {
    this.haltLoop();
    this.executionPaused = false;
    this.playbackPaused = false;
    this.serverPlayback = null;
    this.setEngineMode(resolveRunEngineModeFromDdnText(`${this.baseDdn}\n${this.getEffectiveWasmSource(this.baseDdn)}`));
    if (this.wasmState && typeof this.wasmState === "object") {
      this.wasmState.client = null;
    }
    this.lastState = null;
    this.lastRuntimeDerived = null;
    this.runtimeTickCounter = 0;
    this.runtimeMaxMadi = 0;
    this.runtimeTimeValue = null;
    this.clearRunErrorBanner();
    this.setParseWarnings([]);
    this.setHash("-");
    this.setRuntimePreviewViewModel(null);
    this.updateObserveSummary({ observation: null, views: null, outputRows: [] });
    this.renderOverlayTabContent(this.lastOverlayMarkdown, { sourceLabel: this.lastOverlayMarkdown ? "교과 설명" : "" });
    this.lastSpace2dMode = "none";
    this.renderMainVisual({ mode: "none" });
    this.lastExecPathHint = String(execPathHint ?? "").trim() || "실행 대기";
    this.syncDockRangeLabels();
    this.syncDockTimeUi();
    this.setEngineStatus("idle");
    this.updateRuntimeHint();
  }

  bindViewDockUi() {
    this.root.querySelector("#btn-bogae-fullscreen")?.addEventListener("click", () => {
      void this.toggleBogaeFullscreen();
    });
    this.root.querySelector("#btn-graph-autoscale")?.addEventListener("click", () => {
      this.dotbogi?.resetAxis?.();
      this.syncDockRangeLabels();
    });
    this.root.querySelector("#btn-dock-space-autoscale")?.addEventListener("click", () => {
      this.bogae?.resetView?.();
      this.syncDockRangeLabels();
    });
    this.root.querySelector("#btn-dock-graph-autoscale")?.addEventListener("click", () => {
      this.dotbogi?.resetAxis?.();
      this.syncDockRangeLabels();
    });
    this.root.querySelector("#btn-dock-pan-left")?.addEventListener("click", () => this.panDockTarget(-this.viewPanStep, 0));
    this.root.querySelector("#btn-dock-pan-right")?.addEventListener("click", () => this.panDockTarget(this.viewPanStep, 0));
    this.root.querySelector("#btn-dock-pan-up")?.addEventListener("click", () => this.panDockTarget(0, this.viewPanStep));
    this.root.querySelector("#btn-dock-pan-down")?.addEventListener("click", () => this.panDockTarget(0, -this.viewPanStep));
    this.root.querySelector("#btn-dock-zoom-in")?.addEventListener("click", () => this.zoomDockTarget(0.9));
    this.root.querySelector("#btn-dock-zoom-out")?.addEventListener("click", () => this.zoomDockTarget(1.1));
    this.dockTargetSelectEl?.addEventListener("change", () => {
      this.dockTarget = normalizeDockTarget(this.dockTargetSelectEl?.value);
    });
    this.dockGridCheckEl?.addEventListener("change", () => this.applyDockGuideToggles());
    this.dockAxisCheckEl?.addEventListener("change", () => this.applyDockGuideToggles());
    this.dockXTicksCheckEl?.addEventListener("change", () => this.applyDockGuideToggles());
    this.dockHighlightCheckEl?.addEventListener("change", () => {
      const on = Boolean(this.dockHighlightCheckEl?.checked);
      try {
        document?.body?.setAttribute?.("data-dock-highlight", on ? "on" : "off");
      } catch (_) {
        // ignore highlight toggle errors
      }
    });
    this.root.querySelector("#btn-dock-play")?.addEventListener("click", () => {
      this.dockCursorFollowLive = false;
      this.setPlaybackPaused(false);
    });
    this.root.querySelector("#btn-dock-pause")?.addEventListener("click", () => {
      this.setPlaybackPaused(true);
    });
    this.root.querySelector("#btn-dock-next")?.addEventListener("click", () => {
      this.setPlaybackPaused(true);
      this.advanceDockCursor(1, { loop: this.playbackLoop });
    });
    this.dockLoopCheckEl?.addEventListener("change", () => {
      this.playbackLoop = Boolean(this.dockLoopCheckEl?.checked);
      this.syncDockTimeUi();
    });
    this.dockSpeedSelectEl?.addEventListener("change", () => {
      this.playbackSpeed = normalizeDockSpeed(this.dockSpeedSelectEl?.value);
      this.syncDockTimeUi();
      if (!this.playbackPaused) {
        this.startViewPlaybackTimer();
      }
    });
    this.dockTimeCursorEl?.addEventListener("input", () => {
      this.setDockCursorTick(this.dockTimeCursorEl?.value ?? 0, {
        followLive: false,
      });
    });
  }

  async toggleBogaeFullscreen() {
    const target = this.bogaeAreaEl;
    if (!target) return false;
    try {
      const doc = document;
      if (doc?.fullscreenElement) {
        await doc.exitFullscreen?.();
        return true;
      }
      await target.requestFullscreen?.();
      return true;
    } catch (_) {
      return false;
    }
  }

  panDockTarget(dx, dy) {
    const target = normalizeDockTarget(this.dockTargetSelectEl?.value || this.dockTarget);
    this.dockTarget = target;
    if (target === "graph") {
      this.dotbogi?.panByRatio?.(dx, dy);
    } else {
      this.bogae?.panByRatio?.(dx, dy);
    }
    this.syncDockRangeLabels();
  }

  zoomDockTarget(factor) {
    const target = normalizeDockTarget(this.dockTargetSelectEl?.value || this.dockTarget);
    this.dockTarget = target;
    if (target === "graph") {
      this.dotbogi?.zoomByFactor?.(factor);
    } else {
      this.bogae?.zoomByFactor?.(factor);
    }
    this.syncDockRangeLabels();
  }

  applyDockGuideToggles() {
    const showGrid = Boolean(this.dockGridCheckEl?.checked);
    const showAxis = Boolean(this.dockAxisCheckEl?.checked);
    const showXAxisTicks = Boolean(this.dockXTicksCheckEl?.checked);
    this.dotbogi?.setGuides?.({ showGrid, showAxis });
    this.bogae?.setGuides?.({ showGrid, showAxis, showXAxisTicks });
    this.syncDockGuideToggles();
  }

  syncDockGuideToggles() {
    const graphGuides = this.dotbogi?.getGuides?.() ?? {};
    const spaceGuides = this.bogae?.getGuides?.() ?? {};
    const showGrid = Boolean(graphGuides.showGrid ?? spaceGuides.showGrid ?? false);
    const showAxis = Boolean(graphGuides.showAxis ?? spaceGuides.showAxis ?? false);
    const showXAxisTicks = Boolean(spaceGuides.showXAxisTicks ?? false);
    if (this.dockGridCheckEl) {
      this.dockGridCheckEl.checked = showGrid;
    }
    if (this.dockAxisCheckEl) {
      this.dockAxisCheckEl.checked = showAxis;
    }
    if (this.dockXTicksCheckEl) {
      this.dockXTicksCheckEl.checked = showXAxisTicks;
    }
    if (this.dockTargetSelectEl) {
      this.dockTargetSelectEl.value = normalizeDockTarget(this.dockTarget);
    }
  }

  syncDockRangeLabels({ spaceRange = null, graphAxis = null } = {}) {
    const nextSpaceRange = spaceRange || this.bogae?.getCurrentRange?.() || null;
    const nextGraphRange = graphAxis || this.dotbogi?.getCurrentAxis?.() || null;
    if (this.dockSpaceRangeEl) {
      this.dockSpaceRangeEl.textContent = `그림: ${formatAxisRange(nextSpaceRange)}`;
    }
    if (this.dockGraphRangeEl) {
      this.dockGraphRangeEl.textContent = `그래프: ${formatAxisRange(nextGraphRange)}`;
    }
  }

  clearViewPlaybackTimer() {
    if (this.viewPlaybackTimer === null || this.viewPlaybackTimer === undefined) return;
    try {
      clearInterval(this.viewPlaybackTimer);
    } catch (_) {
      // ignore timer clear errors
    }
    this.viewPlaybackTimer = null;
  }

  resolveViewPlaybackIntervalMs() {
    const speed = normalizeDockSpeed(this.playbackSpeed);
    return Math.max(60, Math.round(240 / speed));
  }

  startViewPlaybackTimer() {
    this.clearViewPlaybackTimer();
    if (this.playbackPaused || !this.screenVisible) return;
    const intervalMs = this.resolveViewPlaybackIntervalMs();
    this.viewPlaybackTimer = setInterval(() => {
      this.advanceDockCursor(1, { loop: this.playbackLoop });
    }, intervalMs);
  }

  setPlaybackPaused(paused = true) {
    this.playbackPaused = Boolean(paused);
    if (this.playbackPaused) {
      this.clearViewPlaybackTimer();
    } else {
      this.startViewPlaybackTimer();
    }
    this.syncDockTimeUi();
  }

  resolveDockCursorMaxTick() {
    const runtimeTick = Math.max(0, Number(this.runtimeTickCounter) || 0);
    const timelineTick = Math.max(0, Number(this.dotbogi?.getTimelineLength?.() ?? 0) - 1);
    return Math.max(runtimeTick, timelineTick);
  }

  clampDockCursorTick(rawTick) {
    const maxTick = this.resolveDockCursorMaxTick();
    const raw = Number(rawTick);
    if (!Number.isFinite(raw)) return Math.min(this.dockCursorTick, maxTick);
    return Math.max(0, Math.min(maxTick, Math.trunc(raw)));
  }

  setDockCursorTick(rawTick, { followLive = null, syncUi = true } = {}) {
    const nextTick = this.clampDockCursorTick(rawTick);
    this.dockCursorTick = nextTick;
    if (typeof followLive === "boolean") {
      this.dockCursorFollowLive = followLive;
    }
    this.dotbogi?.setPlaybackCursor?.(nextTick, { render: true });
    if (syncUi) {
      this.syncDockTimeUi();
    }
    return nextTick;
  }

  advanceDockCursor(step = 1, { loop = true } = {}) {
    const maxTick = this.resolveDockCursorMaxTick();
    if (maxTick <= 0) {
      this.setDockCursorTick(0, { followLive: false });
      return false;
    }
    const delta = Math.max(1, Math.trunc(Number(step) || 1));
    let nextTick = this.dockCursorTick + delta;
    if (nextTick > maxTick) {
      if (loop) {
        nextTick = 0;
      } else {
        nextTick = maxTick;
        this.setPlaybackPaused(true);
      }
    }
    this.setDockCursorTick(nextTick, {
      followLive: false,
    });
    return true;
  }

  resolveDockCursorTimeValue(cursorTick = 0) {
    const sample = this.dotbogi?.getTimelineSampleAt?.(cursorTick);
    const values = sample?.values;
    if (!values || typeof values !== "object") return this.runtimeTimeValue;
    const observation = {
      values,
      all_values: values,
      channels: Object.keys(values).map((key) => ({ key })),
    };
    const t = readNumericObservationValue(observation, ["t", "time", "tick", "프레임수", "시간"]);
    return Number.isFinite(t) ? t : this.runtimeTimeValue;
  }

  syncDockTimeUi() {
    if (this.dockSpeedSelectEl) {
      this.dockSpeedSelectEl.value = String(normalizeDockSpeed(this.playbackSpeed));
    }
    if (this.dockLoopCheckEl) {
      this.dockLoopCheckEl.checked = Boolean(this.playbackLoop);
    }
    this.syncRunMadiStatus();
    const cursorMaxTick = this.resolveDockCursorMaxTick();
    if (this.dockCursorFollowLive) {
      this.dockCursorTick = cursorMaxTick;
    } else {
      this.dockCursorTick = this.clampDockCursorTick(this.dockCursorTick);
    }
    this.dotbogi?.setPlaybackCursor?.(this.dockCursorTick, { render: false });
    const cursorMax = Math.max(100, cursorMaxTick);
    if (this.dockTimeCursorEl) {
      this.dockTimeCursorEl.max = String(cursorMax);
      this.dockTimeCursorEl.value = String(this.dockCursorTick);
    }
    const cursorTimeValue = this.resolveDockCursorTimeValue(this.dockCursorTick);
    const tText = cursorTimeValue === null ? "-" : formatStatusNumber(cursorTimeValue, 3);
    if (this.dockTimeTextEl) {
      const mode = this.playbackPaused ? "일시정지" : "재생(보기)";
      this.dockTimeTextEl.textContent = `${mode} · t=${tText} / 틱=${this.dockCursorTick} / 끝=${cursorMaxTick}`;
    }
  }

  installRuntimeTableResizeObserver() {
    const hostWindow = globalThis?.window;
    if (this.runtimeTableResizeObserver && typeof this.runtimeTableResizeObserver.disconnect === "function") {
      this.runtimeTableResizeObserver.disconnect();
    }
    this.runtimeTableResizeObserver = null;
    if (this.runtimeTableResizeFallbackInstalled && typeof hostWindow?.removeEventListener === "function") {
      hostWindow.removeEventListener("resize", this.boundRuntimeTableResizeHandler);
    }
    this.runtimeTableResizeFallbackInstalled = false;
    if (!this.runtimeTablePanelEl || typeof globalThis?.ResizeObserver !== "function") {
      if (this.runtimeTablePanelEl && typeof hostWindow?.addEventListener === "function") {
        hostWindow.addEventListener("resize", this.boundRuntimeTableResizeHandler);
        this.runtimeTableResizeFallbackInstalled = true;
      }
      return;
    }
    this.runtimeTableResizeObserver = new globalThis.ResizeObserver(() => {
      this.refreshRuntimeTableForCurrentWidth();
    });
    this.runtimeTableResizeObserver.observe(this.runtimeTablePanelEl);
  }

  renderCurrentRuntimeTable(table, { maxChars = null } = {}) {
    const normalizedTable = normalizeRuntimeTableView(table ?? null);
    const hasTable = renderRuntimeTable(this.runtimeTableEl, table ?? null, { maxChars });
    if (this.runtimeTableMetaEl) {
      this.runtimeTableMetaEl.textContent = hasTable ? summarizeRuntimeTableView(normalizedTable) : "";
    }
    this.lastRuntimeTableCellMaxChars = hasTable
      ? resolveRuntimeTableCellMaxChars(this.runtimeTableEl, { maxChars })
      : 0;
    return hasTable;
  }

  refreshRuntimeTableForCurrentWidth({ force = false } = {}) {
    if (!this.screenVisible) return false;
    const table = this.lastRuntimeDerived?.views?.table ?? null;
    if (!table) return false;
    const nextMaxChars = resolveRuntimeTableCellMaxChars(this.runtimeTableEl);
    if (!force && nextMaxChars === this.lastRuntimeTableCellMaxChars) {
      return false;
    }
    this.renderCurrentRuntimeTable(table, { maxChars: nextMaxChars });
    return true;
  }

  persistUiPrefs() {
    writeStorageJson(RUN_UI_PREFS_STORAGE_KEY, this.uiPrefs);
    try {
      window.dispatchEvent(new CustomEvent("seamgrim:run-prefs-changed"));
    } catch (_) {
      // ignore event dispatch errors
    }
  }

  getLessonUiPref(lessonId, { create = false } = {}) {
    const id = String(lessonId ?? "").trim();
    if (!id) return null;
    if (!this.uiPrefs.lessons || typeof this.uiPrefs.lessons !== "object") {
      this.uiPrefs.lessons = {};
    }
    if (!this.uiPrefs.lessons[id] && create) {
      this.uiPrefs.lessons[id] = {};
    }
    return this.uiPrefs.lessons[id] ?? null;
  }

  restoreLessonUiPrefs(lessonId) {
    const pref = this.getLessonUiPref(lessonId, { create: false });
    this.dotbogi.setSelectedAxes({
      xKey: String(pref?.selectedXKey ?? ""),
      yKey: String(pref?.selectedYKey ?? ""),
    });
    this.dotbogi.setGraphKind("line");
    this.dotbogi.setMaxPointsMode(normalizeGraphRangeSelection(pref?.graphRange ?? RUN_GRAPH_RANGE_RECENT_500));
    if (this.graphKindSelectEl) {
      this.graphKindSelectEl.value = String(this.dotbogi?.getGraphKind?.() ?? "line");
    }
    if (this.graphRangeSelectEl) {
      this.graphRangeSelectEl.value = String(this.dotbogi?.getMaxPointsMode?.() ?? RUN_GRAPH_RANGE_RECENT_500);
    }
  }

  saveCurrentLessonUiPrefs() {
    const lessonId = String(this.lesson?.id ?? "").trim();
    if (!lessonId) return;
    const pref = this.getLessonUiPref(lessonId, { create: true });
    const selected = this.dotbogi?.getSelectedAxes?.() ?? {};
    pref.selectedXKey = String(selected.xKey ?? "");
    pref.selectedYKey = String(selected.yKey ?? "");
    pref.graphRange = String(this.dotbogi?.getMaxPointsMode?.() ?? RUN_GRAPH_RANGE_RECENT_500);
    this.persistUiPrefs();
  }

  exportRuntimeSessionState() {
    const selectedAxes = this.dotbogi?.getSelectedAxes?.() ?? {};
    const graphGuides = this.dotbogi?.getGuides?.() ?? {};
    const spaceGuides = this.bogae?.getGuides?.() ?? {};
    const controls = toPlainObject(this.sliderPanel?.getValues?.() ?? {}, {});
    const tick = Math.max(0, Number(this.runtimeTickCounter) || 0);
    const cursor = this.clampDockCursorTick(this.dockCursorTick);
    const cursorTimeValue = this.resolveDockCursorTimeValue(cursor);
    const viewTarget = normalizeDockTarget(this.dockTargetSelectEl?.value || this.dockTarget);
    const graphSample = toPlainObject(this.lastRuntimeDerived?.views?.graph?.sample, {});
    return {
      controls,
      sample: graphSample,
      time: {
        enabled: true,
        t_min: 0,
        t_max: Math.max(1, Number(this.resolveDockCursorMaxTick()) || Number(this.runtimeTimeValue) || 1),
        step: 1,
        now: cursorTimeValue === null ? 0 : Number(cursorTimeValue),
        interval: 300,
        loop: Boolean(this.playbackLoop),
        tick,
        cursor,
        speed: normalizeDockSpeed(this.playbackSpeed),
        paused: Boolean(this.playbackPaused),
        playing: !this.playbackPaused,
      },
      view: {
        auto: false,
        panX: 0,
        panY: 0,
        zoom: 1,
        range: null,
        showGrid: Boolean(graphGuides.showGrid ?? spaceGuides.showGrid),
        showAxis: Boolean(graphGuides.showAxis ?? spaceGuides.showAxis),
        showXAxisTicks: Boolean(spaceGuides.showXAxisTicks),
        graph: {
          auto_fit: false,
          axis: this.dotbogi?.getCurrentAxis?.() ?? null,
          graph_range: String(this.dotbogi?.getMaxPointsMode?.() ?? RUN_GRAPH_RANGE_RECENT_500),
          guides: {
            showGrid: Boolean(graphGuides.showGrid),
            showAxis: Boolean(graphGuides.showAxis),
          },
          selected_axes: {
            x: String(selectedAxes.xKey ?? ""),
            y: String(selectedAxes.yKey ?? ""),
          },
        },
        space2d: {
          auto_fit: false,
          range: this.bogae?.getCurrentRange?.() ?? null,
          guides: {
            showGrid: Boolean(spaceGuides.showGrid),
            showAxis: Boolean(spaceGuides.showAxis),
            showXAxisTicks: Boolean(spaceGuides.showXAxisTicks),
          },
        },
        dock: {
          target: viewTarget,
          highlight: Boolean(this.dockHighlightCheckEl?.checked),
        },
      },
      ui_layout: {
        screen_mode: "run",
        workspace_mode: this.studioViewMode,
        main_tab: this.activeRunTab === "lesson" ? "lesson-tab" : "tools-tab",
        active_view: viewTarget === "graph" ? "view-graph" : "view-2d",
        run_tab: normalizeRunTab(this.activeRunTab),
      },
      active_run_id: String(this.activeOverlayRunId ?? "").trim(),
      last_state_hash: String(this.lastRuntimeHash ?? "").trim(),
    };
  }

  sessionMatchesCurrentLesson(sessionLike) {
    const row = toPlainObject(sessionLike, {});
    const sessionLesson = String(row.lesson ?? "").trim();
    const currentLesson = String(this.lesson?.id ?? "").trim();
    if (!sessionLesson || !currentLesson) return true;
    if (sessionLesson !== currentLesson) return false;
    const sessionDdn = String(row.ddn_text ?? row.ddnText ?? "").replace(/\r\n/g, "\n").trim();
    const currentDdn = String(this.baseDdn ?? this.lesson?.ddnText ?? "").replace(/\r\n/g, "\n").trim();
    if (sessionDdn && currentDdn && sessionDdn !== currentDdn) return false;
    if (isEmptyInitialWasmStateHash(row.last_state_hash ?? row.lastStateHash) && !hasMeaningfulSessionRuns(row)) {
      return false;
    }
    return true;
  }

  applyRuntimeSessionState(sessionLike = null) {
    const row = toPlainObject(sessionLike, {});
    if (!this.sessionMatchesCurrentLesson(row)) return false;

    const uiLayout = toPlainObject(row.ui_layout ?? row.uiLayout, {});
    this.setStudioViewMode(uiLayout.workspace_mode ?? uiLayout.workspaceMode ?? this.studioViewMode, {
      persist: false,
      promoteDefaultTab: false,
    });
    const runTab = normalizeRunTab(uiLayout.run_tab ?? uiLayout.runTab ?? this.activeRunTab);
    this.switchRunTab(runTab);

    const view = toPlainObject(row.view, {});
    const graphView = toPlainObject(view.graph, {});
    const graphGuides = toPlainObject(graphView.guides, {});
    this.dotbogi?.setGuides?.({
      showGrid: typeof graphGuides.showGrid === "boolean" ? graphGuides.showGrid : null,
      showAxis: typeof graphGuides.showAxis === "boolean" ? graphGuides.showAxis : null,
    });
    const selectedAxes = toPlainObject(graphView.selected_axes ?? graphView.selectedAxes, {});
    this.dotbogi?.setSelectedAxes?.({
      xKey: String(selectedAxes.x ?? selectedAxes.xKey ?? ""),
      yKey: String(selectedAxes.y ?? selectedAxes.yKey ?? ""),
    });
    this.dotbogi?.setGraphKind?.("line");
    this.dotbogi?.setMaxPointsMode?.(
      normalizeGraphRangeSelection(graphView.graph_range ?? graphView.graphRange ?? RUN_GRAPH_RANGE_RECENT_500),
    );
    if (this.graphKindSelectEl) {
      this.graphKindSelectEl.value = String(this.dotbogi?.getGraphKind?.() ?? "line");
    }
    if (this.graphRangeSelectEl) {
      this.graphRangeSelectEl.value = String(this.dotbogi?.getMaxPointsMode?.() ?? RUN_GRAPH_RANGE_RECENT_500);
    }
    if (graphView.auto_fit === true || graphView.autoFit === true) {
      this.dotbogi?.resetAxis?.();
    } else {
      this.dotbogi?.setAxis?.(graphView.axis ?? graphView.range ?? null);
    }

    const space2dView = toPlainObject(row.space2d_view ?? row.space2dView ?? view.space2d, {});
    const space2dGuides = toPlainObject(space2dView.guides, {});
    this.bogae?.setGuides?.({
      showGrid: typeof space2dGuides.showGrid === "boolean" ? space2dGuides.showGrid : null,
      showAxis: typeof space2dGuides.showAxis === "boolean" ? space2dGuides.showAxis : null,
      showXAxisTicks: typeof space2dGuides.showXAxisTicks === "boolean"
        ? space2dGuides.showXAxisTicks
        : (typeof space2dGuides.show_x_axis_ticks === "boolean" ? space2dGuides.show_x_axis_ticks : null),
    });
    if (space2dView.auto_fit === true || space2dView.autoFit === true) {
      this.bogae?.resetView?.();
    } else {
      this.bogae?.setRange?.(space2dView.range ?? space2dView.axis ?? null);
    }

    const dockView = toPlainObject(view.dock, {});
    this.dockTarget = normalizeDockTarget(dockView.target ?? this.dockTarget);
    if (this.dockTargetSelectEl) {
      this.dockTargetSelectEl.value = this.dockTarget;
    }
    if (this.dockHighlightCheckEl && typeof dockView.highlight === "boolean") {
      this.dockHighlightCheckEl.checked = dockView.highlight;
    }

    const time = toPlainObject(row.time, {});
    if (typeof time.loop === "boolean") {
      this.playbackLoop = Boolean(time.loop);
    }
    if (typeof time.paused === "boolean") {
      this.playbackPaused = Boolean(time.paused);
    } else if (typeof time.playing === "boolean") {
      this.playbackPaused = !time.playing;
    }
    this.playbackSpeed = normalizeDockSpeed(time.speed ?? this.playbackSpeed);
    const hasSavedCursor = Object.prototype.hasOwnProperty.call(time, "cursor") || Object.prototype.hasOwnProperty.call(time, "tick");
    const savedTick = Math.max(0, Number(time.tick) || 0);
    const savedCursor = Math.max(0, Number(time.cursor ?? savedTick) || 0);
    this.dockCursorTick = savedCursor;
    this.dockCursorFollowLive = !hasSavedCursor;
    this.dotbogi?.setPlaybackCursor?.(savedCursor, { render: false });

    const activeRunId = String(row.active_run_id ?? "").trim();
    if (activeRunId && this.findRunManagerIndexById(activeRunId) >= 0) {
      this.activeOverlayRunId = activeRunId;
    }

    this.syncDockGuideToggles();
    this.applyDockGuideToggles();
    this.syncDockRangeLabels();
    this.syncDockTimeUi();
    if (!this.playbackPaused) {
      this.startViewPlaybackTimer();
    }
    this.renderRunManagerUi();
    this.syncRunManagerOverlaySeries();
    return true;
  }

  applyLessonLayoutProfile(lesson) {
    const profile = resolveRunLayoutProfile(lesson?.requiredViews ?? []);
    const dockOrder = resolveRunDockPanelOrder(lesson?.requiredViews ?? []);
    this.lessonLayoutProfile = profile;
    const mode = String(profile.mode ?? "split");
    const classList = this.layoutEl?.classList;
    if (classList?.toggle) {
      classList.toggle("run-layout--split", mode === "split");
      classList.toggle("run-layout--dock-only", mode === "dock_only");
      classList.toggle("run-layout--space-primary", mode === "space_primary");
    }
    if (this.root?.dataset) {
      this.root.dataset.requiredViews = profile.families.join(",");
      this.root.dataset.runLayoutMode = mode;
      this.root.dataset.runDockOrder = dockOrder.join(",");
    }
    this.applyDockPanelOrder(dockOrder);
    this.syncOverlayToggleState();
    this.syncDockPanelVisibility();
    this.refreshStudioLayoutBounds({ persist: false });
    this.syncRunPresetRail();
    return profile;
  }

  syncRuntimeLayoutProfile(views, mainVisualMode, { outputRows = [], outputLines = [], outputLog = [] } = {}) {
    if (this.lesson?.requiredViews?.length > 0) return;
    const hasMainVisual = ["space2d", "debug-fallback", "console-grid"].includes(normalizeRunMainVisualMode(mainVisualMode, "none"));
    const hasDockPanels = hasGraphTabPayload(views) || hasResultPanelPayload({ outputRows, outputLines, outputLog, views });
    if (!hasMainVisual && !hasDockPanels) return;
    const classList = this.layoutEl?.classList;
    if (!classList?.toggle) return;
    const hasClass = (name) => typeof classList.contains === "function" && classList.contains(name);
    const currentMode = String(
      this.root?.dataset?.runLayoutMode
        ?? (hasClass("run-layout--dock-only")
          ? "dock_only"
          : hasClass("run-layout--space-primary")
            ? "space_primary"
            : "split"),
    ).trim();
    // 초기화 직후 도크 payload가 잠깐 비어도 기존 split 비율은 유지한다.
    const mode = !hasMainVisual && hasDockPanels
      ? "split"
      : hasDockPanels
        ? "split"
        : (currentMode === "split" || currentMode === "dock_only")
          ? currentMode
          : "space_primary";
    classList.toggle("run-layout--split", mode === "split");
    classList.toggle("run-layout--dock-only", mode === "dock_only");
    classList.toggle("run-layout--space-primary", mode === "space_primary");
    if (this.root?.dataset) {
      this.root.dataset.runLayoutMode = mode;
    }
    this.refreshStudioLayoutBounds({ persist: false });
  }

  async executeRunRequest(request = {}) {
    const runRequest = this.normalizeRunRequest(request);
    if (this.completedRunRequestId === runRequest.id || this.activeRunRequestId === runRequest.id) return false;
    this.activeRunRequestId = runRequest.id;
    this.lastLaunchKind = runRequest.launchKind;
    this.baseDdn = runRequest.sourceText;
    this.currentDdn = runRequest.sourceText;
    if (this.lesson && typeof this.lesson === "object") {
      this.lesson.ddnText = runRequest.sourceText;
    }
    if (this.runDdnPreviewEl) {
      this.runDdnPreviewEl.value = runRequest.sourceText;
    }
    const ok = await this.restart({ runRequestId: runRequest.id });
    if (this.activeRunRequestId === runRequest.id) {
      this.completedRunRequestId = runRequest.id;
      this.activeRunRequestId = "";
    }
    return ok;
  }

  applyDockPanelOrder(order) {
    const dockOrder = Array.isArray(order) ? order : resolveRunDockPanelOrder(this.lesson?.requiredViews ?? []);
    const panelMap = {
      graph: this.graphPanelEl,
      table: this.runtimeTablePanelEl,
      text: this.runtimeTextPanelEl,
    };
    dockOrder.forEach((panel, index) => {
      const el = panelMap[panel];
      if (!el?.style) return;
      el.style.order = String(index);
    });
  }

  renderRuntimeText(markdown) {
    return this.renderRuntimeTextContent({ markdown, structure: null, graph: null });
  }

  setRuntimePreviewViewModel(viewModel) {
    this.lastPreviewViewModel = viewModel ?? null;
    applyPreviewViewModelMetadata(this.runtimeTextPanelEl, this.lastPreviewViewModel);
  }

  renderRuntimeTextContent({ markdown = "", structure = null, graph = null, showGraphPreview = true } = {}) {
    const text = String(markdown ?? "").trim();
    this.lastRuntimeTextMarkdown = text;
    if (this.runtimeTextBodyEl) {
      const graphPayload = showGraphPreview ? graph : null;
      const graphResult = buildFamilyPreviewResult({
        family: "graph",
        payload: graphPayload,
        html: buildRuntimeGraphPreviewHtml(graphPayload),
      });
      const structureResult = buildFamilyPreviewResult({
        family: "structure",
        payload: structure,
        html: buildRuntimeStructurePreviewHtml(structure),
      });
      const previewCollection = buildPreviewResultCollection([graphResult, structureResult], {
        preferredFamilies: ["graph", "structure", "text"],
        summaryClassName: "runtime-preview-summary",
        cardClassName: "runtime-preview-card",
      });
      const previewViewModel = buildPreviewViewModel(previewCollection, {
        sourceId: String(this.lesson?.id ?? "run.runtime"),
      });
      this.setRuntimePreviewViewModel(previewViewModel);
      const previewHtml = String(previewCollection?.html ?? "");
      const textHtml = text ? markdownToHtml(text) : "";
      if (previewHtml && textHtml) {
        this.runtimeTextBodyEl.innerHTML = `${previewHtml}<div class="runtime-text-markdown">${textHtml}</div>`;
      } else if (previewHtml) {
        this.runtimeTextBodyEl.innerHTML = previewHtml;
      } else if (textHtml) {
        this.setRuntimePreviewViewModel(null);
        this.runtimeTextBodyEl.innerHTML = textHtml;
      } else {
        this.setRuntimePreviewViewModel(null);
        this.runtimeTextBodyEl.innerHTML = '<div class="runtime-text-empty">설명 출력 없음</div>';
      }
    }
    this.updateRuntimeHint();
    this.syncDockPanelVisibility();
    this.syncOverlayToggleState();
    return Boolean(text || structure || (showGraphPreview ? graph : null));
  }

  updateObserveSummary({ observation = null, views = null, outputRows = [], outputLog = [] } = {}) {
    if (!this.runObserveSummaryEl) return;
    const runtimeDerived = { observation, views };
    const channels = Array.isArray(observation?.channels) ? observation.channels.length : 0;
    const normalizedOutputRows = normalizeObserveOutputRows(outputRows);
    const outputRowsMetric = summarizeObserveOutputRows(normalizedOutputRows);
    const rows = readObserveFamilyRows(runtimeDerived);
    const rowMap = new Map(rows.map((row) => [String(row.family ?? "").trim(), row]));
    const displayRows = OBSERVE_FAMILY_ORDER.map((family) => {
      const key = String(family ?? "").trim();
      const row = rowMap.get(key);
      if (row) return row;
      return {
        family: key,
        label: formatObserveFamilyName(key),
        available: false,
        source: "off",
        strict: true,
      };
    });
    const availableRows = rows.filter((row) => row.available);
    const nonStrictRows = availableRows.filter((row) => !row.strict);
    const outputRowsPreview = buildObserveOutputRowsPreview(normalizedOutputRows, { maxRows: 3 });
    const summaryViewModel = buildObserveSummaryViewModel({
      channels,
      displayRows,
      availableRows,
      nonStrictRows,
      normalizedOutputRows,
      outputRowsMetric,
      outputRowsPreview,
      views,
      observeOutputActionCode: OBSERVE_ACTION_OPEN_DDN_OBSERVE_OUTPUT,
      escapeHtml,
      summarizeFamilyMetric: summarizeObserveFamilyMetric,
      buildFamilyActionHint: buildObserveFamilyActionHint,
    });

    if (this.runObserveSummaryTextEl) {
      this.runObserveSummaryTextEl.textContent = String(summaryViewModel.summaryText ?? "");
    }
    this.runObserveSummaryEl.dataset.level = String(summaryViewModel.level ?? "none");
    this.updateObserveOutputRowsPanel(normalizedOutputRows, outputLog);
    this.updateNumericKernelPanel(runtimeDerived);
  }

  updateNumericKernelPanel(runtimeDerived = this.lastRuntimeDerived) {
    if (!this.runNumericKernelPanelEl || !this.runNumericKernelStatusEl || !this.runNumericKernelBodyEl) return;
    const diagRows = Array.isArray(runtimeDerived?.diagnostics)
      ? runtimeDerived.diagnostics
      : Array.isArray(runtimeDerived?.diags)
        ? runtimeDerived.diags
        : [];
    const numericDiag = diagRows.find((row) => {
      const code = String(row?.code ?? row?.tag ?? row?.kind ?? "").trim();
      return code.startsWith("numeric:factor:");
    });
    if (!numericDiag) {
      this.runNumericKernelPanelEl.dataset.status = "idle";
      this.runNumericKernelStatusEl.textContent = "대기";
      this.runNumericKernelBodyEl.textContent =
        "긴 곱수 분해 작업이 생기면 여기에서 진행률과 이어하기 상태를 보여줍니다.";
      return;
    }
    const status = String(numericDiag.status ?? numericDiag.kind ?? numericDiag.code ?? "running");
    const normalizedStatus = status.includes("done")
      ? "done"
      : status.includes("blocked")
        ? "blocked"
        : "running";
    this.runNumericKernelPanelEl.dataset.status = normalizedStatus;
    this.runNumericKernelStatusEl.textContent =
      normalizedStatus === "done" ? "완료" : normalizedStatus === "blocked" ? "막힘" : "진행 중";
    const route = String(numericDiag.route ?? numericDiag.trace ?? "-");
    const message = String(numericDiag.message ?? numericDiag.detail ?? "곱수 작업 상태를 확인하고 있습니다.");
    this.runNumericKernelBodyEl.textContent = `${message} · route=${route}`;
  }

  updateObserveOutputRowsPanel(outputRows = [], outputLog = []) {
    const normalizedRows = normalizeObserveOutputRows(outputRows);
    const tableMetric = renderObserveOutputRowsTable(this.runObserveOutputBodyEl, normalizedRows, {
      maxRows: 24,
      outputLog,
    });
    if (!this.runObserveOutputMetaEl) return;
    if (!tableMetric.rowCount) {
      this.runObserveOutputMetaEl.textContent = "0행";
      return;
    }
    if (tableMetric.mode === "console") {
      if (tableMetric.truncated) {
        this.runObserveOutputMetaEl.textContent = `${tableMetric.rowCount}줄 중 최근 ${tableMetric.shownRowCount}줄`;
        return;
      }
      this.runObserveOutputMetaEl.textContent = `${tableMetric.rowCount}줄 · console fallback`;
      return;
    }
    if (tableMetric.truncated) {
      this.runObserveOutputMetaEl.textContent = `${tableMetric.rowCount}행 중 최근 ${tableMetric.shownRowCount}행`;
      return;
    }
    this.runObserveOutputMetaEl.textContent = `${tableMetric.rowCount}행`;
  }

  syncDockPanelVisibility(runtimeDerived = this.lastRuntimeDerived) {
    const visible = resolveRunDockPanelVisibility(this.lesson?.requiredViews ?? [], {
      runtimeDerived,
      textMarkdown: this.lastRuntimeTextMarkdown,
    });
    this.graphPanelEl?.classList?.toggle("hidden", !visible.graph);
    this.runtimeTablePanelEl?.classList?.toggle("hidden", !visible.table);
    this.runtimeTextPanelEl?.classList?.toggle("hidden", !visible.text);
    return visible;
  }

  syncOverlayToggleState() {
    if (!this.overlayToggleBtn) return;
    const hasMarkdown = Boolean(
      String(this.lastOverlayMarkdown ?? "").trim() || String(this.lastRuntimeTextMarkdown ?? "").trim(),
    );
    this.overlayToggleBtn.disabled = !hasMarkdown;
    this.overlayToggleBtn.title = hasMarkdown ? "설명 탭 열기" : "표시할 설명이 없습니다.";
  }

  isEditableTarget(target) {
    if (!(target instanceof Element)) return false;
    if (target instanceof HTMLInputElement) return true;
    if (target instanceof HTMLTextAreaElement) return true;
    if (target instanceof HTMLSelectElement) return true;
    if (target.closest("input, textarea, select, [contenteditable='true']")) return true;
    return target.isContentEditable;
  }

  handleViewHotkeys(event) {
    if (!this.root || this.root.classList.contains("hidden")) return;
    if (this.isEditableTarget(event.target)) return;

    const key = String(event.key ?? "");
    const lowerKey = key.toLowerCase();
    const warningShortcut = event.altKey && !event.ctrlKey && !event.metaKey;
    if (warningShortcut) {
      if (lowerKey === "d") {
        if (this.triggerRunWarningShortcut("ddn")) {
          event.preventDefault();
        }
        return;
      }
      if (lowerKey === "i") {
        if (this.triggerRunWarningShortcut("inspector")) {
          event.preventDefault();
        }
        return;
      }
    }
    const pan = this.viewPanStep;
    const graphHotkey = event.shiftKey && !event.ctrlKey && !event.metaKey && !event.altKey;
    const bogaeHotkey = event.shiftKey && (event.ctrlKey || event.metaKey) && !event.altKey;

    if (graphHotkey) {
      if (key === "ArrowLeft") {
        event.preventDefault();
        this.dotbogi.panByRatio(-pan, 0);
        return;
      }
      if (key === "ArrowRight") {
        event.preventDefault();
        this.dotbogi.panByRatio(pan, 0);
        return;
      }
      if (key === "ArrowUp") {
        event.preventDefault();
        this.dotbogi.panByRatio(0, pan);
        return;
      }
      if (key === "ArrowDown") {
        event.preventDefault();
        this.dotbogi.panByRatio(0, -pan);
        return;
      }
      if (key === "+" || key === "=") {
        event.preventDefault();
        this.dotbogi.zoomByFactor(0.9);
        return;
      }
      if (key === "-" || key === "_") {
        event.preventDefault();
        this.dotbogi.zoomByFactor(1.1);
        return;
      }
    }

    if (bogaeHotkey) {
      if (key === "ArrowLeft") {
        event.preventDefault();
        this.bogae.panByRatio(-pan, 0);
        return;
      }
      if (key === "ArrowRight") {
        event.preventDefault();
        this.bogae.panByRatio(pan, 0);
        return;
      }
      if (key === "ArrowUp") {
        event.preventDefault();
        this.bogae.panByRatio(0, pan);
        return;
      }
      if (key === "ArrowDown") {
        event.preventDefault();
        this.bogae.panByRatio(0, -pan);
        return;
      }
      if (key === "+" || key === "=") {
        event.preventDefault();
        this.bogae.zoomByFactor(0.9);
        return;
      }
      if (key === "-" || key === "_") {
        event.preventDefault();
        this.bogae.zoomByFactor(1.1);
      }
    }
  }

  isRuntimeInputEnabled() {
    if (!this.screenVisible) return false;
    if (!this.root || this.root.classList.contains("hidden")) return false;
    return Boolean(this.wasmState?.inputEnabled ?? true);
  }

  clearRuntimeInputState() {
    this.heldInputMask = 0;
    this.pulsePressedMask = 0;
    this.lastInputToken = "";
  }

  handleRuntimeInputKeyDown(event) {
    if (!this.isRuntimeInputEnabled()) return;
    if (!event || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    if (this.isEditableTarget(event.target)) return;
    const token = runtimeInputTokenFromKeyboardEvent(event);
    if (!token) return;
    const bit = runtimeInputBitFromToken(token);
    if (!bit) return;
    const wasHeld = (this.heldInputMask & bit) !== 0;
    this.heldInputMask = (this.heldInputMask | bit) & RUNTIME_INPUT_MASK_LIMIT;
    if (!wasHeld) {
      this.pulsePressedMask = (this.pulsePressedMask | bit) & RUNTIME_INPUT_MASK_LIMIT;
    }
    this.lastInputToken = token;
    event.preventDefault();
  }

  handleRuntimeInputKeyUp(event) {
    const token = runtimeInputTokenFromKeyboardEvent(event);
    if (!token) return;
    const bit = runtimeInputBitFromToken(token);
    if (!bit) return;
    this.heldInputMask = (this.heldInputMask & ~bit) & RUNTIME_INPUT_MASK_LIMIT;
    if (!this.isRuntimeInputEnabled()) return;
    if (!event || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    if (this.isEditableTarget(event.target)) return;
    event.preventDefault();
  }

  setHash(hashText) {
    this.lastRuntimeHash = String(hashText ?? "-");
    this.updateMirrorTab(this.lastState);
    this.renderInspectorMeta();
    this.syncLocalPackageDeliveryStatus();
  }

  setParseWarnings(warnings) {
    const normalized = normalizeParseWarnings(warnings);
    this.lastParseWarnings = normalized;
    if (this.wasmState && typeof this.wasmState === "object") {
      this.wasmState.parseWarnings = normalized;
    }
    this.renderConsoleWarningSummary();
    this.renderParseWarningPanel();
    this.syncRunActionRail();
    this.renderInspectorMeta();
    this.renderMirrorDiagnostics();
    this.renderWarningBadge();
    this.collapseMirrorDetails();
  }

  updateMirrorTab(stateJson) {
    if (!this.runMirrorHashEl || !this.runMirrorKvEl) return;
    const hashText = String(this.lastRuntimeHash ?? "-").trim() || "-";
    this.runMirrorHashEl.textContent = formatCompactStateHash(hashText);
    this.runMirrorHashEl.title = hashText === "-" ? "상태 기록 없음" : `전체 상태 기록: ${hashText}`;
    if (this.runCopyHashBtn) {
      const studentDelivery = this.lastLaunchKind === "local_package_import";
      const hasStudentName = !studentDelivery || Boolean(String(this.runDeliveryStudentNameEl?.value ?? "").trim());
      this.runCopyHashBtn.disabled = hashText === "-" || !hasStudentName;
      this.runCopyHashBtn.textContent = studentDelivery ? "결과 복사" : "복사";
      this.runCopyHashBtn.title = hashText === "-"
        ? studentDelivery ? "아직 복사할 실행 결과가 없습니다." : "복사할 상태 기록이 없습니다."
        : !hasStudentName ? "학생 이름을 입력한 뒤 결과를 복사합니다."
          : studentDelivery ? "교사에게 보낼 수업 결과 복사" : "현재 상태 기록 복사";
      this.runCopyHashBtn.dataset.studentDelivery = studentDelivery ? "1" : "0";
    }
    const world = stateJson && typeof stateJson === "object" && stateJson.world && typeof stateJson.world === "object"
      ? stateJson.world
      : null;
    const observation = extractObservationChannelsFromState(stateJson);
    const observationValues =
      observation?.all_values && typeof observation.all_values === "object"
        ? observation.all_values
        : observation?.values && typeof observation.values === "object"
          ? observation.values
          : {};
    const entries = world && Object.keys(world).length > 0
      ? Object.entries(world)
      : Object.entries(observationValues).filter(([key, value]) => {
        const keyText = String(key ?? "").trim();
        if (!keyText || keyText.startsWith("__")) return false;
        return value !== null && value !== undefined;
      });
    if (this.runMirrorWorldSummaryEl) {
      this.runMirrorWorldSummaryEl.textContent = world && Object.keys(world).length > 0
        ? `world 상태 · ${entries.length}개`
        : `관찰 상태 · ${entries.length}개`;
    }
    this.runMirrorWorldEl?.classList?.toggle?.("hidden", entries.length <= 0);
    if (!entries.length) {
      this.runMirrorKvEl.innerHTML = '<div class="run-mirror-empty">관찰 상태 없음</div>';
      return;
    }
    this.runMirrorKvEl.innerHTML = entries
      .slice(0, 200)
      .map(([key, value]) => {
        const valueText = typeof value === "string" ? value : JSON.stringify(value);
        return `<div class="run-mirror-row"><span class="run-mirror-key">${escapeHtml(String(key))}</span><span class="run-mirror-val">${escapeHtml(String(valueText ?? ""))}</span></div>`;
      })
      .join("");
  }

  clearRunErrorBanner() {
    this.runErrorDismissed = true;
    if (!this.runErrorBannerEl) return;
    this.runErrorBannerEl.classList.add("hidden");
    if (this.runErrorTextEl) {
      this.runErrorTextEl.textContent = "";
    }
    if (this.runErrorActionBtn) {
      this.runErrorActionBtn.classList.add("hidden");
      this.runErrorActionBtn.textContent = "원인 확인";
      this.runErrorActionBtn.title = "";
      this.runErrorActionBtn.dataset.actionKind = "default";
    }
  }

  renderRunErrorBanner(warnings) {
    if (!this.runErrorBannerEl || !this.runErrorTextEl) return;
    const list = Array.isArray(warnings) ? warnings : [];
    const first = list.find((row) => row && typeof row === "object");
    if (!first) {
      this.runErrorMessage = "";
      this.runErrorDismissed = false;
      if (this.runErrorActionBtn) {
        this.runErrorActionBtn.classList.add("hidden");
        this.runErrorActionBtn.textContent = "원인 확인";
        this.runErrorActionBtn.title = "";
      }
      this.syncRunControlState();
      return;
    }
    this.runErrorMessage = translateRunErrorBannerMessage(first);
    this.runErrorDismissed = false;
    const warningModel = buildWarningPanelViewModel({
      warnings: list,
      lastWarningSignature: this.lastWarningSignature,
    });
    const action = warningModel?.primaryAction && typeof warningModel.primaryAction === "object"
      ? warningModel.primaryAction
      : null;
    this.lastWarningPrimaryActionKind = String(action?.kind ?? this.lastWarningPrimaryActionKind ?? "").trim().toLowerCase();
    if (this.runErrorActionBtn) {
      const actionKind = String(action?.kind ?? "").trim().toLowerCase();
      const visible = Boolean(actionKind) && actionKind !== "retry";
      this.runErrorActionBtn.classList.toggle("hidden", !visible);
      this.runErrorActionBtn.textContent = String(action?.label ?? "원인 확인").trim() || "원인 확인";
      this.runErrorActionBtn.title = String(action?.detail ?? "").trim();
      this.runErrorActionBtn.dataset.actionKind = actionKind || "default";
    }
    this.syncRunControlState();
  }

  async handleRunErrorPrimaryAction() {
    const kind = String(this.lastWarningPrimaryActionKind ?? "").trim().toLowerCase();
    if (!kind || kind === "retry") {
      return this.restart();
    }
    return this.triggerRunWarningPrimaryAction();
  }

  renderConsoleWarningSummary() {
    if (!this.runConsoleWarningSummaryEl) return;
    const model = buildConsoleWarningSummary(this.lastParseWarnings);
    this.runConsoleWarningSummaryEl.textContent = String(model.text ?? "경고 없음");
    this.runConsoleWarningSummaryEl.dataset.level = String(model.level ?? "ok");
    this.runConsoleWarningSummaryEl.classList?.toggle?.("hidden", String(model.level ?? "ok") === "ok");
  }

  renderParseWarningPanel() {
    const warnings = Array.isArray(this.lastParseWarnings) ? this.lastParseWarnings : [];
    this.lastWarningSignature = warnings.map((warning) => String(warning?.code ?? "").trim()).join("|");
    this.lastWarningPrimaryActionKind = hasLegacyAutofixCandidate(String(this.runDdnPreviewEl?.value || this.baseDdn || ""))
      ? "autofix"
      : (warnings.length > 0 ? "open_inspector" : "retry");
    this.renderMirrorDiagnostics();
    this.renderWarningBadge();
  }

  buildCurrentInspectorSceneSummary(sceneSummary = null) {
    const source = sceneSummary && typeof sceneSummary === "object" ? sceneSummary : {};
    const lessonId = String(this.lesson?.id ?? "").trim();
    const layers = Array.isArray(source.layers) ? source.layers : [];
    const filteredLayers = layers.filter((row) => {
      const id = String(row?.id ?? "").trim();
      if (!lessonId) return false;
      if (!(id === `run:${lessonId}` || id.startsWith(`run:${lessonId}:`))) return false;
      return isRunManagerGraphRunMeaningful(row);
    });
    return {
      ...source,
      schema: String(source.schema ?? "").trim() || "seamgrim.scene.v0",
      hashes: {
        ...(source.hashes && typeof source.hashes === "object" ? source.hashes : {}),
        result_hash: String(this.lastRuntimeHash ?? "").trim() || String(source?.hashes?.result_hash ?? "").trim(),
      },
      layers: filteredLayers,
    };
  }

  renderInspectorMeta() {
    if (!this.runInspectorMetaEl || !this.runInspectorMetaBodyEl) return;
    const external =
      this.onGetInspectorContext && typeof this.onGetInspectorContext === "function"
        ? this.onGetInspectorContext() ?? {}
        : {};
    const externalSession = external.sessionV0 && typeof external.sessionV0 === "object" ? external.sessionV0 : null;
    const externalMatchesCurrent = !externalSession || this.sessionMatchesCurrentLesson(externalSession);
    const report = buildInspectorReport({
      lesson: this.lesson,
      lastRuntimeHash: this.lastRuntimeHash,
      parseWarnings: this.lastParseWarnings,
      runtimeTickCounter: this.runtimeTickCounter,
      runtimeTimeValue: this.runtimeTimeValue,
      playbackPaused: this.playbackPaused,
      playbackSpeed: this.playbackSpeed,
      lastExecPathHint: this.lastExecPathHint,
      lastRuntimeDerived: this.lastRuntimeDerived,
      sceneSummary: externalMatchesCurrent ? this.buildCurrentInspectorSceneSummary(external.sceneSummary ?? null) : this.buildCurrentInspectorSceneSummary(null),
      snapshotV0: externalMatchesCurrent ? (external.snapshotV0 ?? null) : null,
      sessionV0: externalMatchesCurrent ? externalSession : null,
      uiPrefsLessons: null,
    });
    this.lastInspectorReport = report;
    const bridgeOk = Boolean(report?.bridge_check?.ok);
    const viewSourceStrict = Boolean(report?.view_contract?.strict);
    this.runInspectorMetaBodyEl.dataset.bridgeCheck = bridgeOk ? "ok" : "fail";
    this.runInspectorMetaBodyEl.dataset.viewSourceStrict = viewSourceStrict ? "ok" : "fail";
    this.runInspectorMetaBodyEl.textContent = formatInspectorReportText(report);
    if (this.runInspectorMetaSummaryEl) {
      this.runInspectorMetaSummaryEl.title = bridgeOk ? "검증 정보 · 정상" : "검증 정보 · 점검 필요";
    }
    if (this.runInspectorMetaChipsEl) {
      const bridgeStatus = bridgeOk ? "ok" : "error";
      const viewStatus = viewSourceStrict ? "ok" : "warn";
      this.runInspectorMetaChipsEl.innerHTML = [
        `<span class="run-mirror-summary-chip run-inspector-meta-chip" data-kind="bridge" data-status="${bridgeStatus}">bridge ${bridgeOk ? "정상" : "점검"}</span>`,
        `<span class="run-mirror-summary-chip run-inspector-meta-chip" data-kind="view" data-status="${viewStatus}">view ${viewSourceStrict ? "strict" : "warn"}</span>`,
      ].join("");
    }
    this.runInspectorMetaEl.open = false;
    const viewFamilies = Array.isArray(report?.view_contract?.families) ? report.view_contract.families : [];
    const nonStrictFamilies = Array.isArray(report?.view_contract?.non_strict_families)
      ? report.view_contract.non_strict_families
      : [];
    const strictKnown = viewFamilies.length > 0;
    this.updateViewSourceBadge({
      strict: strictKnown ? viewSourceStrict : null,
      families: nonStrictFamilies,
    });
  }

  updateViewSourceBadge({ strict = null, families = [] } = {}) {
    if (!this.runViewSourceBadgeEl) return;
    const nonStrict = Array.isArray(families) ? families.filter((item) => String(item).trim()) : [];
    if (strict === true) {
      this.runViewSourceBadgeEl.dataset.status = "ok";
      this.runViewSourceBadgeEl.textContent = "보기 strict";
      this.runViewSourceBadgeEl.title = "보기소스: strict";
      return;
    }
    if (strict === false) {
      this.runViewSourceBadgeEl.dataset.status = "warn";
      this.runViewSourceBadgeEl.textContent = "보기 경고";
      this.runViewSourceBadgeEl.title = `보기소스 경고: ${nonStrict.length ? nonStrict.join("+") : "-"}`;
      return;
    }
    this.runViewSourceBadgeEl.dataset.status = "unknown";
    this.runViewSourceBadgeEl.textContent = "보기 -";
    this.runViewSourceBadgeEl.title = "보기소스 정보 없음";
  }

  updateRunSummaryFromPrefs() {
    if (!this.lastSummaryEl) return;
    const lessonId = String(this.lesson?.id ?? "").trim();
    const pref = lessonId ? this.getLessonUiPref(lessonId, { create: false }) : null;
    this.lastSummaryEl.textContent = buildRunSummaryText(pref);
  }

  syncLocalPackageDeliveryStatus(lesson = this.lesson) {
    if (!this.runDeliveryStatusEl) return;
    if (this.lastLaunchKind !== "local_package_import") {
      this.runDeliveryStatusEl.textContent = "";
      this.runDeliveryStatusEl.classList.add("hidden");
      this.runDeliveryStatusEl.dataset.state = "idle";
      if (this.runDeliveryResultEl) {
        this.runDeliveryResultEl.classList.add("hidden");
        this.runDeliveryResultEl.dataset.ready = "0";
      }
      if (this.runDeliveryResultCopyBtn) {
        this.runDeliveryResultCopyBtn.disabled = true;
      }
      if (this.runDeliveryResultDownloadBtn) {
        this.runDeliveryResultDownloadBtn.disabled = true;
      }
      if (this.runDeliveryInstructionsEl) {
        this.runDeliveryInstructionsEl.innerHTML = "";
        this.runDeliveryInstructionsEl.classList.add("hidden");
        this.runDeliveryInstructionsEl.dataset.ready = "0";
      }
      return;
    }
    const packageTitle = String(lesson?.localPackageTitle ?? "").trim();
    const sessionLabel = String(lesson?.localPackageSessionLabel ?? "").trim();
    const suffix = [
      sessionLabel ? `차시 ${sessionLabel}` : "",
      packageTitle,
    ].filter(Boolean).map((item) => ` · ${item}`).join("");
    const resultInstruction = resolveLocalPackageResultInstruction(lesson).replace(/[.。]\s*$/, "");
    const resultSuffix = resultInstruction ? ` · ${resultInstruction}` : "";
    const status = normalizeEngineStatus(this.engineStatus, "idle");
    const hashText = String(this.lastRuntimeHash ?? "").trim();
    const hasResultHash = Boolean(hashText && hashText !== "-");
    const hasStudentName = Boolean(String(this.runDeliveryStudentNameEl?.value ?? "").trim());
    let state = "ready";
    let label = `받은 배포 파일 준비됨${resultSuffix}${suffix}`;
    if (status === "blocked" || status === "fatal") {
      state = "error";
      label = `받은 수업 실행 확인 필요${resultSuffix}${suffix}`;
    } else if (status === "done" || hasResultHash) {
      state = "done";
      const nameSuffix = hasStudentName ? "" : " · 학생 이름 입력 필요";
      label = `받은 수업 실행 완료${resultSuffix || " · 결과 확인"}${nameSuffix}${suffix}`;
    } else if (status === "running") {
      state = "running";
      label = `받은 수업 실행 중${resultSuffix}${suffix}`;
    } else if (status === "paused") {
      state = "paused";
      label = `받은 수업 일시정지됨${resultSuffix}${suffix}`;
    }
    this.runDeliveryStatusEl.textContent = label;
    this.runDeliveryStatusEl.classList.remove("hidden");
    this.runDeliveryStatusEl.dataset.state = state;
    if (this.runDeliveryResultEl) {
      this.runDeliveryResultEl.classList.toggle("hidden", !hasResultHash);
      this.runDeliveryResultEl.dataset.ready = hasResultHash ? "1" : "0";
    }
    if (this.runDeliveryResultCopyBtn) {
      this.runDeliveryResultCopyBtn.disabled = !hasResultHash || !hasStudentName;
      this.runDeliveryResultCopyBtn.title = hasResultHash
        ? hasStudentName ? "교사에게 보낼 수업 결과 복사" : "학생 이름을 입력한 뒤 결과를 복사합니다."
        : "받은 수업을 실행하면 결과를 복사할 수 있습니다.";
    }
    if (this.runDeliveryResultDownloadBtn) {
      this.runDeliveryResultDownloadBtn.disabled = !hasResultHash || !hasStudentName;
      this.runDeliveryResultDownloadBtn.title = hasResultHash
        ? hasStudentName ? "교사에게 보낼 수업 결과 저장" : "학생 이름을 입력한 뒤 결과를 저장합니다."
        : "받은 수업을 실행하면 결과를 저장할 수 있습니다.";
    }
    if (this.runDeliveryInstructionsEl) {
      const instructionsHtml = renderLocalPackageStudentInstructionHtml(lesson);
      this.runDeliveryInstructionsEl.innerHTML = instructionsHtml;
      this.runDeliveryInstructionsEl.classList.toggle("hidden", !instructionsHtml);
      this.runDeliveryInstructionsEl.dataset.ready = instructionsHtml ? "1" : "0";
    }
  }

  loadLesson(lesson, { launchKind = "manual", sourceKind = "lesson", sourceLabel = "" } = {}) {
    this.lesson = lesson;
    this.lastLaunchKind = normalizeRunLaunchKind(launchKind);
    this.classroomModeAccess = this.lastLaunchKind === "browse_select_student" || this.lastLaunchKind === "local_package_import"
      ? "student"
      : this.lastLaunchKind === "browse_select_teacher"
        ? "teacher"
        : "default";
    this.clearPendingAutoExecute();
    this.cancelLiveReplRestart();
    this.baseDdn = String(lesson?.ddnText ?? "");
    if (hasLegacyAutofixCandidate(this.baseDdn)) {
      const autofix = applyLegacyAutofixToDdn(this.baseDdn);
      if (autofix.changed) {
        this.baseDdn = autofix.text;
      }
    }
    this.currentDdn = this.baseDdn;
    this.sourceKind = String(sourceKind ?? "").trim() || "lesson";
    this.sourceLabel = String(sourceLabel ?? "").trim() || String(lesson?.title ?? lesson?.id ?? "새 작업").trim() || "새 작업";
    this.primaryView = "sim";
    this.graphTabMode = "graph";
    this.activeRunTab = SUBPANEL_TAB.MAEGIM;
    if (this.root?.dataset) {
      this.root.dataset.primaryView = "sim";
      this.root.dataset.graphTabMode = "graph";
    }
    this.lastRuntimeHash = "-";
    this.lastRuntimeSnapshotKey = "";
    this.lastRuntimeDerived = null;
    this.lastExecPathHint = "";
    this.lastSpace2dMode = "none";
    this.setMainVisualMode("none");
    this.lastRuntimeHintText = "";
    this.setRuntimePreviewViewModel(null);
    this.lastOverlayMarkdown = "";
    this.lastRuntimeTextMarkdown = "";
    this.lastParseWarnings = [];
    this.observeGuideStatusText = "";
    this.observeGuideStatusExpireAt = 0;
    this.clearObserveGuideStatusTimer();
    this.updateViewSourceBadge({ strict: null });
    this.updateObserveSummary({ observation: null, views: null, outputRows: [] });
    this.lastOnboardingProfile = "";
    this.lastGraphSnapshot = null;
    this.runtimeTickCounter = 0;
    this.runtimeMaxMadi = 0;
    this.runtimeTimeValue = null;
    this.runAttemptCount = 0;
    this.firstRunSuccessEmitted = false;
    this.pendingRecoveryFromFailure = false;
    this.lastFailureCode = "";
    this.playbackPaused = false;
    this.executionPaused = false;
    this.playbackSpeed = normalizeDockSpeed(this.playbackSpeed);
    this.playbackLoop = true;
    this.dockCursorTick = 0;
    this.dockCursorFollowLive = true;
    this.clearViewPlaybackTimer();
    this.serverPlayback = null;

    if (this.titleEl) {
      this.titleEl.textContent = lesson?.title || lesson?.id || "-";
    }
    this.syncPackageLessonSwitcher();
    if (this.studioSourceLabelEl) {
      this.studioSourceLabelEl.textContent = this.sourceLabel;
    }
    if (this.runDdnPreviewEl) {
    this.runDdnPreviewEl.value = this.baseDdn;
    }
    this.syncClassroomReportExport();
    this.syncLocalPackageExport();
    this.syncPublicationPrepExport();
    this.syncRegistrySeedExport();
    this.syncApprovalContinuityExport();
    this.syncBenchmarkLtsExport();
    this.syncEducationOperationsLtsExport();
    this.syncInitialBogaeShellVisibility(false);
    this.setRunLocalSaveStatus("저장 대기", { status: "idle" });
    this.syncLegacyAutofixAvailability();
    const lessonGoals = Array.isArray(lesson?.goals)
      ? lesson.goals.map((item) => String(item ?? "").trim()).filter(Boolean)
      : [];
    const lessonMissions = Array.isArray(lesson?.missions)
      ? lesson.missions.map((item) => String(item ?? "").trim()).filter(Boolean)
      : [];
    const lessonViews = resolveLessonRequiredViewsForRun(lesson);
    const packageTitle = String(lesson?.localPackageTitle ?? "").trim();
    this.syncLocalPackageDeliveryStatus(lesson);
    if (this.runLessonBriefEl) {
      const title = String(lesson?.title ?? lesson?.id ?? "수업").trim() || "수업";
      const goalText = lessonGoals[0] ? `목표: ${lessonGoals[0]}` : "목표: DDN 수업 실행";
      const missionText = lessonMissions.length
        ? `활동: ${lessonMissions.slice(0, 2).join(" / ")}`
        : "활동: 결과를 보고 확인";
      const resultText = formatRunResultCheckLabel(lessonViews);
      const deliveryPrefix = this.lastLaunchKind === "local_package_import"
        ? `교사가 보낸 배포 파일${packageTitle ? `: ${packageTitle}` : ""} · `
        : "";
      this.runLessonBriefEl.textContent = `${deliveryPrefix}${title} · ${goalText} · ${missionText} · ${resultText}`;
    }
    if (this.runLessonSummaryEl) {
      const lessonId = String(lesson?.id ?? "-").trim() || "-";
      const title = String(lesson?.title ?? "").trim() || "-";
      const description = String(lesson?.description ?? "").trim();
      const resultText = formatRunResultCheckLabel(lessonViews);
      const lines = [`교과: ${title} (${lessonId})`];
      if (description) {
        lines.push(`설명: ${description}`);
      }
      if (lessonGoals.length) {
        lines.push(`학습목표: ${lessonGoals.slice(0, 2).join(" / ")}`);
      }
      if (lessonMissions.length) {
        lines.push(`수업 활동: ${lessonMissions.slice(0, 2).join(" / ")}`);
      }
      lines.push(resultText);
      this.runLessonSummaryEl.textContent = lines.join("\n");
    }
    this.applyLessonLayoutProfile(lesson);
    this.switchRunTab(this.activeRunTab || "graph");
    this.lastOverlayMarkdown = String(lesson?.textMd ?? "");
    this.overlay.setContent(this.lastOverlayMarkdown);
    this.renderOverlayTabContent(this.lastOverlayMarkdown, { sourceLabel: "교과 설명" });
    this.renderRuntimeText(this.lastOverlayMarkdown);
    this.overlay.hide();
    const parsed = this.sliderPanel.parseFromDdn(this.baseDdn, {
      preserveValues: false,
      maegimControlJson: lesson?.maegimControlJson ?? "",
    });
    this.syncRunSliderAreaVisibility();
    this.syncGraphDraftFromParsed(parsed);
    this.dotbogi.clearTimeline();
    this.restoreLessonUiPrefs(lesson?.id);
    const lessonPref = this.getLessonUiPref(lesson?.id, { create: false });
    const savedOnboardingProfile = normalizeRunOnboardingProfile(lessonPref?.lastOnboardingProfile ?? "");
    this.bogae.resetView();
    this.dotbogi.resetAxis();
    this.hydrateRunManagerFromSession({ publish: false });
    const restoredFromSession = this.applyRuntimeSessionState(this.getRuntimeSessionV0?.() ?? null);
    this.syncDockGuideToggles();
    this.applyDockGuideToggles();
    this.syncDockRangeLabels();
    this.syncDockTimeUi();
    if (!restoredFromSession) {
      this.setInspectorStatus(this.lastInspectorStatusText || "저장 작업 대기");
    }
    if (savedOnboardingProfile) {
      this.applyRunOnboardingProfile(savedOnboardingProfile, { persist: false });
    } else if (this.lastLaunchKind === "browse_select" || this.lastLaunchKind === "browse_select_student" || this.lastLaunchKind === "browse_select_teacher") {
      this.setRunOnboardingStatus(`${buildLessonOnboardingStatusText(lesson)} · 학생 시작/교사 시작 중 하나를 선택하고 결과를 확인하세요.`, { status: "warn" });
    } else {
      this.setRunOnboardingStatus(buildLessonOnboardingStatusText(lesson), { status: "idle" });
    }
    this.syncRunPresetRail();
    this.saveCurrentLessonUiPrefs();
    this.updateRunSummaryFromPrefs();
    this.updateRuntimeHint();
    this.setHash("-");
    this.setEngineStatus("idle");
    this.runErrorMessage = "";
    this.runErrorDismissed = false;
    this.renderWarningBadge();
    this.renderMirrorDiagnostics();
    this.onSourceChange?.(this.baseDdn);
    this.setStudioShellState({
      sourceKind: this.sourceKind,
      sourceLabel: this.sourceLabel,
      engineStatus: "idle",
      primaryViewFamily: this.primaryView,
      activeSubpanelTab: this.activeRunTab || SUBPANEL_TAB.GRAPH,
    });
    this.refreshStudioLayoutBounds({ persist: false });
  }

  updateRuntimeHint(execPathText = "", runtimeDerived = null) {
    if (!this.runtimeStatusEl) return;
    const nextPath = String(execPathText ?? "").trim();
    if (nextPath) {
      this.lastExecPathHint = nextPath;
    }

    const controlCount = Array.isArray(this.sliderPanel?.specs) ? this.sliderPanel.specs.length : 0;
    const observeGuideText = this.readObserveGuideStatusText();
    const runtimeGuideText = resolveRuntimeGuideText({
      warnings: this.lastParseWarnings,
      observeGuideText,
    });
    const parseWarningSummary = formatParseWarningSummary(this.lastParseWarnings);

    const viewFamilies = Array.isArray(runtimeDerived?.views?.families)
      ? runtimeDerived.views.families
      : Array.isArray(this.lastRuntimeDerived?.views?.families)
        ? this.lastRuntimeDerived.views.families
        : [];
    const previewSummary = String(this.lastPreviewViewModel?.summaryText ?? "").trim();
    const viewSourceStrictness = resolveRuntimeViewSourceStrictness(runtimeDerived ?? this.lastRuntimeDerived);

    const obs = runtimeDerived?.observation ?? this.lastRuntimeDerived?.observation ?? null;
    const t = readNumericObservationValue(obs, ["t", "time", "tick", "프레임수", "시간"]);
    this.runtimeTimeValue = Number.isFinite(t) ? t : this.runtimeTimeValue;
    const outputRows = normalizeObserveOutputRows(runtimeDerived?.outputRows ?? this.lastRuntimeDerived?.outputRows ?? []);
    const outputRowsPreview = buildObserveOutputRowsPreview(outputRows, { maxRows: 3 });
    const isLiveMode = normalizeRunEngineMode(this.engineMode, RUN_ENGINE_MODE_ONESHOT) === RUN_ENGINE_MODE_LIVE;
    const obsParts = [];
    if (outputRowsPreview) {
      obsParts.push(outputRowsPreview);
    } else if (isLiveMode || viewFamilies.length > 0) {
      const theta = readNumericObservationValue(obs, ["theta", "각도", "theta_rad", "angle", "rad"]);
      const omega = readNumericObservationValue(obs, ["omega", "각속도", "angular_velocity"]);
      const tText = formatStatusNumber(t);
      if (tText) obsParts.push(`t=${tText}`);
      const thetaText = formatStatusNumber(theta);
      if (thetaText) obsParts.push(`theta=${thetaText}`);
      const omegaText = formatStatusNumber(omega);
      if (omegaText) obsParts.push(`omega=${omegaText}`);
    }
    const hintViewModel = buildRuntimeHintViewModel({
      tickCount: isLiveMode || this.runtimeTickCounter > 0 ? this.runtimeTickCounter : null,
      controlCount,
      runtimeGuideText,
      execPathHint: this.lastExecPathHint,
      shapeMode: this.lastMainVisualMode,
      parseWarningSummary,
      viewFamilies,
      previewSummary,
      nonStrictFamilies: viewSourceStrictness.nonStrictFamilies,
      observationSummary: obsParts.length ? obsParts.join(" ") : "",
      labels: {
        viewSourceWarning: RUN_HINT_VIEW_SOURCE_WARN_LABEL,
      },
    });
    const nextText = String(hintViewModel.text ?? "");
    if (nextText === this.lastRuntimeHintText) {
      this.syncDockTimeUi();
      this.renderInspectorMeta();
      return;
    }
    this.lastRuntimeHintText = nextText;
    const execStatusModel = buildRunExecStatusViewModel({
      warnings: this.lastParseWarnings,
      execPathHint: this.lastExecPathHint,
      runtimeHintText: nextText,
      parseWarningSummary,
      viewSourceStrictness,
    });
    if (this.runExecUserStatusEl) {
      let userStatusText = String(execStatusModel.userStatusText ?? "실행 대기");
      let userStatusLevel = String(execStatusModel.status ?? "idle");
      if (this.executionPaused && normalizeRunEngineMode(this.engineMode, RUN_ENGINE_MODE_ONESHOT) !== RUN_ENGINE_MODE_ONESHOT) {
        const hint = String(this.lastExecPathHint ?? "");
        userStatusText = "실행 일시정지";
        userStatusLevel = "warn";
      }
      this.runExecUserStatusEl.textContent = userStatusText;
      if (this.runExecUserStatusEl.dataset) {
        this.runExecUserStatusEl.dataset.status = userStatusLevel;
      }
    }
    this.runtimeStatusEl.textContent = nextText;
    if (this.runtimeStatusEl.dataset) {
      this.runtimeStatusEl.dataset.status = String(hintViewModel.status ?? execStatusModel.status ?? "ok");
    }
    if (this.runExecTechSummaryEl) {
      this.runExecTechSummaryEl.textContent = String(execStatusModel.techSummaryText ?? "기술 상세");
    }
    if (this.runExecTechBodyEl) {
      this.runExecTechBodyEl.textContent = String(execStatusModel.techBodyText ?? "");
    }
    if (this.runExecTechEl) {
      this.runExecTechEl.classList.toggle("hidden", !execStatusModel.showTechnical);
    }
    this.collapseMirrorDetails();
    this.syncDockTimeUi();
    this.renderInspectorMeta();
  }

  async runViaExecServer(ddnText) {
    if (!this.allowServerFallback) {
      return null;
    }
    this.lastServerFallbackWarning = null;
    const payload = {
      ddn_text: String(ddnText ?? ""),
      madi: 420,
    };
    try {
      const response = await fetch("/api/run", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        cache: "no-cache",
        body: JSON.stringify(payload),
      });
      let data = null;
      try {
        data = await response.json();
      } catch (_) {
        data = null;
      }
      if (!response.ok) {
        this.lastServerFallbackWarning = buildServerRunDiagnosticWarning(data) ?? {
          code: "E_RUNTIME_EXEC_FAILED",
          technical_code: "E_RUNTIME_EXEC_FAILED",
          message: "서버 보조 실행 경로가 실패했습니다.",
          technical_message: "server fallback request failed",
          user_message: "실행에 실패했습니다. 문법/입력값을 확인하고 다시 시도해 주세요.",
        };
        return null;
      }
      if (!data || data.ok !== true) return null;
      const runtimeViews = {
        graph: normalizeServerViewObject(data.graph, { source: "api_run" }),
        space2d: normalizeServerViewObject(data.space2d, { source: "api_run" }),
        text: normalizeServerViewObject(data.text, { source: "api_run" }),
        table: normalizeServerViewObject(data.table, { source: "api_run" }),
        structure: normalizeServerViewObject(data.structure, { source: "api_run" }),
      };
      const viewContract = normalizeServerViewContract(data.view_contract, runtimeViews);
      const views = {
        ...runtimeViews,
        families: viewContract.families,
        contract: viewContract,
      };
      const observation = buildObservationFromGraph(data.graph);
      const outputLog = normalizeConsoleOutputLog(extractObservationOutputLogFromState(data ?? {}), {
        fallbackTick: Number(data?.tick_id ?? madi ?? 0),
      });
      const outputLines = normalizeConsoleOutputLines(extractObservationOutputLinesFromState(data ?? {}));
      const outputRows = normalizeObserveOutputRows(extractObservationOutputRowsFromState(data ?? {}));
      const serverPlayback = buildServerPlaybackPlan(data.graph);
      return { observation, outputLog, outputLines, outputRows, views, serverPlayback };
    } catch (_) {
      this.lastServerFallbackWarning = {
        code: "E_RUNTIME_EXEC_FAILED",
        technical_code: "E_RUNTIME_EXEC_FAILED",
        message: "서버 보조 실행 경로 호출에 실패했습니다.",
        technical_message: "server fallback fetch failed",
        user_message: "실행에 실패했습니다. 문법/입력값을 확인하고 다시 시도해 주세요.",
      };
      return null;
    }
  }

  collectServerPlaybackExtraValues() {
    const raw = this.sliderPanel?.getValues?.() ?? {};
    if (!raw || typeof raw !== "object") return {};
    const out = {};
    const length = parseFiniteNumericValue(raw.L);
    if (length !== null) {
      out.L = length;
    }
    return out;
  }

  startServerPlayback(derived) {
    const plan = derived?.serverPlayback;
    if (!plan || !Array.isArray(plan.frames) || plan.frames.length < 2) {
      this.serverPlayback = null;
      return false;
    }
    this.serverPlayback = {
      seriesId: String(plan.seriesId ?? "y"),
      frames: plan.frames.map((row) => ({ x: Number(row.x), y: Number(row.y) })),
      axis: plan.axis ?? null,
      index: 0,
    };
    return true;
  }

  stepServerPlaybackFrame({ forceView = false } = {}) {
    const plan = this.serverPlayback;
    if (!plan || !Array.isArray(plan.frames) || plan.frames.length <= 1) return false;
    const size = plan.frames.length;
    const index = Math.max(0, Math.min(size - 1, Number(plan.index) || 0));
    const point = plan.frames[index];
    const extra = this.collectServerPlaybackExtraValues();
    const values = {
      x: point.x,
      [plan.seriesId]: point.y,
      ...extra,
    };
    const channels = [
      { key: "x", dtype: "number", role: "state" },
      { key: plan.seriesId, dtype: "number", role: "state" },
    ];
    if (Object.prototype.hasOwnProperty.call(extra, "L")) {
      channels.push({ key: "L", dtype: "number", role: "param" });
    }
    const row = channels.map((channel) => values[channel.key]);
    const seriesPoints = plan.frames.slice(0, index + 1).map((row0) => ({ x: row0.x, y: row0.y }));
    const graph = {
      axis: plan.axis ?? null,
      series: [{ id: plan.seriesId, points: seriesPoints }],
    };
    const observation = {
      channels,
      row,
      values,
      all_values: values,
    };
    this.applyRuntimeDerived(
      {
        observation,
        views: {
          graph,
          space2d: null,
          text: null,
          table: null,
          structure: null,
        },
      },
      { forceView },
    );
    plan.index = (index + 1) % size;
    this.runtimeTickCounter += 1;
    this.runtimeTimeValue = Number.isFinite(point?.x) ? point.x : this.runtimeTimeValue;
    this.setHash(`server-fallback:${index}`);
    this.syncDockTimeUi();
    this.renderInspectorMeta();
    return true;
  }

  getEffectiveDdn() {
    const withControls = applyControlValuesToDdnText(this.baseDdn, this.sliderPanel.getValues());
    return hasLegacyAutofixCandidate(withControls)
      ? applyLegacyAutofixToDdn(withControls).text
      : withControls;
  }

  getEffectiveWasmSource(rawText = null) {
    const raw = rawText === null || rawText === undefined ? this.getEffectiveDdn() : String(rawText);
    try {
      const pre = preprocessDdnText(raw);
      const body = String(pre?.bodyText ?? "");
      return body.trim() ? body : raw;
    } catch (_) {
      return raw;
    }
  }

  getStepInput() {
    const fps = Math.max(1, Number(this.wasmState?.fpsLimit ?? 30) || 30);
    const dtMaxRaw = Number(this.wasmState?.dtMax ?? 0.1);
    const dtMax = Number.isFinite(dtMaxRaw) && dtMaxRaw > 0 ? dtMaxRaw : 0.1;
    const baseDt = 1 / fps;
    let dt = Math.min(baseDt, dtMax);
    if (!Number.isFinite(dt) || dt <= 0) {
      dt = baseDt;
    }
    if (!Number.isFinite(dt) || dt <= 0) {
      dt = 1 / 30;
    }
    const inputEnabled = Boolean(this.wasmState?.inputEnabled ?? true);
    const keys = inputEnabled
      ? ((Number(this.heldInputMask) | Number(this.pulsePressedMask)) & RUNTIME_INPUT_MASK_LIMIT)
      : 0;
    const lastKey = inputEnabled ? String(this.lastInputToken ?? "") : "";
    this.pulsePressedMask = 0;
    this.lastInputToken = "";
    return { dt, keys, lastKey, px: 0, py: 0 };
  }

  resolveSteppedState(client, steppedState) {
    let nextState = steppedState;
    try {
      const steppedViews = mergeRuntimeViewsWithObservationOutputFallback(nextState, extractStructuredViewsFromState(nextState, {
        allowObservationOutputFallback: false,
      }));
      if (!hasSpace2dDrawable(steppedViews?.space2d) && typeof client.getStateParsed === "function") {
        const fullState = client.getStateParsed();
        const fullViews = mergeRuntimeViewsWithObservationOutputFallback(fullState, extractStructuredViewsFromState(fullState, {
          allowObservationOutputFallback: false,
        }));
        if (hasSpace2dDrawable(fullViews?.space2d)) {
          nextState = fullState;
        }
      }
    } catch (_) {
      // keep stepped state when enrichment fails
    }
    return nextState;
  }

  stepClientOne(client) {
    if (!client || typeof client !== "object") {
      return { state: null };
    }
    return stepWasmClientParsed({
      client,
      input: this.getStepInput(),
      errorPrefix: "RunScreen.stepClientOne",
    });
  }

  reportRuntimeFailure(payload, { phase = "runtime" } = {}) {
    const code = String(payload?.technical_code ?? payload?.code ?? "E_RUNTIME_EXEC_FAILED").trim() || "E_RUNTIME_EXEC_FAILED";
    const technicalMessage = String(
      payload?.technical_message ?? payload?.message ?? payload?.error ?? payload ?? "",
    ).trim();
    const mappedUserMessage = mapParseWarningToUserMessage(code, technicalMessage);
    const payloadUserMessage = String(payload?.user_message ?? "").trim();
    const userMessage = code === "E_WASM_DIRECT_ONLY_FALLBACK_BLOCKED"
      ? mappedUserMessage
      : (payloadUserMessage || mappedUserMessage);
    this.setParseWarnings([
      {
        code,
        technical_code: code,
        technical_message: technicalMessage || userMessage,
        message: userMessage,
        user_message: userMessage,
      },
    ]);
    this.pendingRecoveryFromFailure = true;
    this.lastFailureCode = code;
    this.lastRuntimeDerived = null;
    this.lastPreviewViewModel = null;
    this.runtimeTimeValue = null;
    this.runtimeMaxMadi = 0;
    this.dockCursorTick = 0;
    this.emitStudioRunMetric("studio.run.failure", {
      code,
      phase: String(phase ?? "runtime"),
      launch_kind: String(this.lastLaunchKind ?? "manual"),
      attempt: this.runAttemptCount,
    });
    this.lastExecPathHint = `실행 실패(${String(phase)}): ${userMessage}`;
    this.lastSpace2dMode = "none";
    this.switchRunTab("console");
    const nextStatus = code === "E_WASM_DIRECT_ONLY_FALLBACK_BLOCKED" ? "fatal" : "blocked";
    this.setEngineStatus(nextStatus, {
      errorMessage: translateRunErrorBannerMessage({
        code,
        technical_code: code,
        message: userMessage,
        technical_message: technicalMessage,
      }),
    });
    this.updateRuntimeHint();
    this.renderInspectorMeta();
  }

  async restart({ runRequestId = "", autoStartLive = true } = {}) {
    const shouldAutoStartLive = Boolean(autoStartLive);
    this.clearRunErrorBanner();
    this.clearRuntimeInputState();
    this.executionPaused = !shouldAutoStartLive;
    this.runtimeTickCounter = 0;
    this.runtimeMaxMadi = 0;
    this.runtimeTimeValue = null;
    this.dockCursorTick = 0;
    this.dockCursorFollowLive = true;
    const rawDdnText = this.getEffectiveDdn();
    const wasmDdnText = this.getEffectiveWasmSource(rawDdnText);
    const requestedMadi = Math.max(
      readConfiguredMadiFromDdnText(rawDdnText),
      readConfiguredMadiFromDdnText(wasmDdnText),
    );
    this.runtimeMaxMadi = requestedMadi;
    const engineMode = this.setEngineMode(resolveRunEngineModeFromDdnText(`${rawDdnText}\n${wasmDdnText}`));
    this.currentDdn = rawDdnText;
    if (this.runDdnPreviewEl) {
      this.runDdnPreviewEl.value = rawDdnText;
    }
    this.syncLegacyAutofixAvailability();
    this.dotbogi.clearTimeline({ preserveAxes: true, preserveView: false });
    this.beginLiveRunCapture(rawDdnText);
    this.runAttemptCount += 1;
    this.setEngineStatus("running");
    this.emitStudioRunMetric("studio.run.execute_attempt", {
      attempt: this.runAttemptCount,
      launch_kind: String(this.lastLaunchKind ?? "manual"),
      ddn_length: rawDdnText.length,
    });

    try {
      const ensureWasm = (source) => this.wasmState.loader.ensure(source);
      const tryRunWithMode = async (mode) =>
        applyWasmLogicAndDispatchState({
          sourceText: wasmDdnText,
          ensureWasm,
          mode,
        });

      const storedMode = String(this.wasmState?.langMode ?? "strict");
      const preferredMode = storedMode === "compat" ? "strict" : storedMode;
      if (this.wasmState && this.wasmState.langMode !== preferredMode) {
        this.wasmState.langMode = preferredMode;
      }
      const result = await tryRunWithMode(preferredMode);
      if (runRequestId && this.activeRunRequestId !== runRequestId) return false;

      this.wasmState.client = result.client;
      const rawWarnings = result.parseWarnings ?? readWasmClientParseWarnings(result.client);
      const filteredWarnings = filterInternalLegacyHeaderWarnings(rawWarnings, rawDdnText);
      this.setParseWarnings(filteredWarnings);
      this.serverPlayback = null;
      let initialState = result.state;
      const configuredMadi = Math.max(readConfiguredMadiFromClient(result.client), requestedMadi);
      const shouldBatchConfiguredMadi = configuredMadi > 0 && engineMode !== RUN_ENGINE_MODE_LIVE;
      this.runtimeMaxMadi = configuredMadi;
      // 재실행은 로직 갱신뿐 아니라 시뮬 상태를 초기 프레임으로 되돌려야 한다.
      if (typeof result.client?.resetParsed === "function") {
        try {
          result.client.resetParsed(true);
          if (typeof result.client?.getStateParsed === "function") {
            initialState = result.client.getStateParsed();
          }
        } catch (_) {
          // reset 실패 시 로직 갱신 상태를 그대로 사용
        }
      }
      if (shouldBatchConfiguredMadi && result.client) {
        if (typeof result.client.runTicksParsed === "function") {
          initialState = this.resolveSteppedState(result.client, result.client.runTicksParsed(configuredMadi));
        } else {
          for (let i = 0; i < configuredMadi; i += 1) {
            const stepped = this.stepClientOne(result.client);
            initialState = this.resolveSteppedState(result.client, stepped.state);
          }
        }
        this.runtimeTickCounter = configuredMadi;
      } else if (engineMode === RUN_ENGINE_MODE_ONESHOT && result.client) {
        const stepped = this.stepClientOne(result.client);
        initialState = this.resolveSteppedState(result.client, stepped.state);
        this.runtimeTickCounter = 1;
      }
      this.lastState = initialState;
      const hash = typeof result.client?.getStateHash === "function" ? result.client.getStateHash() : "-";
      this.setHash(hash);
      this.syncClassroomReportExport();
      this.syncLocalPackageExport();
      this.syncPublicationPrepExport();
      this.syncRegistrySeedExport();
      this.syncApprovalContinuityExport();
      this.syncBenchmarkLtsExport();
      this.syncEducationOperationsLtsExport();
      this.lastExecPathHint = `실행 경로: wasm(${preferredMode})`;
      this.applyRuntimeState(initialState, { forceView: true });
      this.syncDockRangeLabels();
      this.syncDockTimeUi();
      this.renderInspectorMeta();
      if (shouldAutoStartLive && configuredMadi <= 0 && !shouldBatchConfiguredMadi && engineMode === RUN_ENGINE_MODE_LIVE && this.lastSpace2dMode === "none" && result.client) {
        for (let i = 0; i < 3; i += 1) {
          const stepped = this.stepClientOne(result.client);
          const nextState = this.resolveSteppedState(result.client, stepped.state);
          this.lastState = nextState;
          const stepHash = typeof result.client?.getStateHash === "function" ? result.client.getStateHash() : "-";
          this.setHash(stepHash);
          this.applyRuntimeState(nextState, { forceView: true });
          this.runtimeTickCounter += 1;
          this.syncDockTimeUi();
          if (this.lastSpace2dMode !== "none") break;
        }
      }
      this.updateRuntimeHint();
      this.publishRunManagerSession();
      this.switchRunTab(resolveRunPostExecuteTab({
        views: this.lastRuntimeDerived?.views ?? null,
        outputRows: this.lastRuntimeDerived?.outputRows ?? [],
        outputLines: this.lastRuntimeDerived?.outputLines ?? [],
        outputLog: this.lastRuntimeDerived?.outputLog ?? [],
        warnings: this.lastParseWarnings,
      }));
      this.applyTeacherPackageFocus();
      if (!shouldBatchConfiguredMadi && engineMode === RUN_ENGINE_MODE_LIVE) {
        if (shouldAutoStartLive) {
          this.executionPaused = false;
          this.syncLoopState();
          this.setEngineStatus("running");
        } else {
          this.executionPaused = true;
          this.haltLoop();
          this.lastExecPathHint = "한 단계 실행 준비";
          this.updateRuntimeHint();
          this.setEngineStatus("paused");
        }
      } else {
        this.discardActiveRunCaptureIfEmpty();
        this.executionPaused = true;
        this.haltLoop();
        this.lastExecPathHint = "실행 완료";
        this.updateRuntimeHint();
        this.setEngineStatus("done");
      }
      if (!this.firstRunSuccessEmitted) {
        this.firstRunSuccessEmitted = true;
        this.emitStudioRunMetric("studio.run.first_success", {
          attempt: this.runAttemptCount,
          launch_kind: String(this.lastLaunchKind ?? "manual"),
          exec_path: String(this.lastExecPathHint ?? ""),
        });
      }
      if (this.pendingRecoveryFromFailure) {
        this.pendingRecoveryFromFailure = false;
        this.emitStudioRunMetric("studio.run.recovery_success", {
          attempt: this.runAttemptCount,
          launch_kind: String(this.lastLaunchKind ?? "manual"),
          previous_failure_code: String(this.lastFailureCode ?? ""),
        });
      }
      this.lastFailureCode = "";
      return true;
    } catch (err) {
      console.error("[RunScreen.restart] wasm execution failed", err);
      if (this.allowServerFallback) {
        // server fallback은 wasm 내부 변환본이 아닌 원본 DDN(슬라이더 반영본)으로 실행한다.
        const serverDerived = await this.runViaExecServer(rawDdnText);
        if (runRequestId && this.activeRunRequestId !== runRequestId) return false;
        if (serverDerived) {
          this.wasmState.client = null;
          this.setParseWarnings([]);
          this.lastState = null;
          this.lastRuntimeDerived = serverDerived;
          this.setHash("server-fallback");
          this.syncClassroomReportExport();
          this.syncLocalPackageExport();
          this.syncPublicationPrepExport();
          this.syncRegistrySeedExport();
          this.syncApprovalContinuityExport();
          this.syncBenchmarkLtsExport();
          this.syncEducationOperationsLtsExport();
          this.lastExecPathHint = "실행 경로: server-fallback";
          this.startServerPlayback(serverDerived);
          this.applyRuntimeDerived(serverDerived, { forceView: true });
          this.syncDockRangeLabels();
          this.syncDockTimeUi();
          this.renderInspectorMeta();
          this.updateRuntimeStatus(serverDerived);
          this.updateRuntimeHint();
          this.publishRunManagerSession();
          this.switchRunTab(resolveRunPostExecuteTab({
            views: serverDerived?.views ?? null,
            outputRows: serverDerived?.outputRows ?? [],
            outputLines: serverDerived?.outputLines ?? [],
            outputLog: serverDerived?.outputLog ?? [],
            warnings: this.lastParseWarnings,
          }));
          this.applyTeacherPackageFocus();
          if (engineMode === RUN_ENGINE_MODE_LIVE) {
            if (shouldAutoStartLive) {
              this.executionPaused = false;
              this.syncLoopState();
              this.setEngineStatus("running");
            } else {
              this.executionPaused = true;
              this.haltLoop();
              this.lastExecPathHint = "한 단계 실행 준비";
              this.updateRuntimeHint();
              this.setEngineStatus("paused");
            }
          } else {
            this.discardActiveRunCaptureIfEmpty();
            this.executionPaused = true;
            this.haltLoop();
            this.lastExecPathHint = "실행 완료";
            this.updateRuntimeHint();
            this.setEngineStatus("done");
          }
          if (!this.firstRunSuccessEmitted) {
            this.firstRunSuccessEmitted = true;
            this.emitStudioRunMetric("studio.run.first_success", {
              attempt: this.runAttemptCount,
              launch_kind: String(this.lastLaunchKind ?? "manual"),
              exec_path: String(this.lastExecPathHint ?? ""),
            });
          }
          if (this.pendingRecoveryFromFailure) {
            this.pendingRecoveryFromFailure = false;
            this.emitStudioRunMetric("studio.run.recovery_success", {
              attempt: this.runAttemptCount,
              launch_kind: String(this.lastLaunchKind ?? "manual"),
              previous_failure_code: String(this.lastFailureCode ?? ""),
            });
          }
          this.lastFailureCode = "";
          return true;
        }
      }
      if (runRequestId && this.activeRunRequestId !== runRequestId) return false;
      this.haltLoop();
      const directOnlyBlocked = !this.allowServerFallback;
      const serverFallbackWarning = !directOnlyBlocked ? this.lastServerFallbackWarning : null;
      const loader = this.wasmState?.loader && typeof this.wasmState.loader === "object"
        ? this.wasmState.loader
        : null;
      const initDiag = typeof loader?.getLastInitDiag === "function" ? loader.getLastInitDiag() : null;
      const preprocessDiag = typeof loader?.getLastPreprocessDiag === "function" ? loader.getLastPreprocessDiag() : null;
      const errMessage = String(err?.message ?? err ?? "").trim();
      const diagParts = [];
      if (String(initDiag?.message ?? "").trim()) {
        diagParts.push(String(initDiag.message).trim());
      }
      if (String(initDiag?.detail ?? "").trim()) {
        diagParts.push(String(initDiag.detail).trim());
      }
      if (String(preprocessDiag?.message ?? "").trim()) {
        diagParts.push(String(preprocessDiag.message).trim());
      }
      if (String(preprocessDiag?.detail ?? "").trim()) {
        diagParts.push(String(preprocessDiag.detail).trim());
      }
      if (errMessage) {
        diagParts.push(errMessage);
      }
      const technicalDetail = [...new Set(diagParts)].filter(Boolean).join(" | ");
      const parseFailureDetected =
        /파싱\s*실패|preprocess\s*재시도\s*실패/i.test(technicalDetail)
        || /\bparse(?:\s+error|\s+failed|\s+failure)?\b/i.test(technicalDetail);
      this.reportRuntimeFailure(
        parseFailureDetected
          ? {
            code: "E_PARSE_RUNTIME_INPUT",
            technical_code: "E_PARSE_RUNTIME_INPUT",
            technical_message: technicalDetail || "파싱 실패",
            user_message: "DDN 문법을 점검한 뒤 다시 실행해 주세요.",
          }
          : (directOnlyBlocked
            ? {
              code: "E_WASM_DIRECT_ONLY_FALLBACK_BLOCKED",
              technical_code: "E_WASM_DIRECT_ONLY_FALLBACK_BLOCKED",
              technical_message: technicalDetail
                || "WASM direct-only run failed and server fallback is blocked in release mode.",
            }
            : (serverFallbackWarning ?? {
              code: "E_RUNTIME_EXEC_FAILED",
              technical_code: "E_RUNTIME_EXEC_FAILED",
              technical_message: "WASM execution and server fallback both failed.",
              user_message: "실행에 실패했습니다. 문법/입력값을 확인하고 다시 시도해 주세요.",
            })),
        { phase: parseFailureDetected ? "wasm-parse" : (directOnlyBlocked ? "wasm-direct" : "wasm-server") },
      );
      this.setHash("-");
      this.lastRuntimeDerived = null;
      this.lastSpace2dMode = "none";
      this.setRuntimePreviewViewModel(null);
      this.updateObserveSummary({ observation: null, views: null, outputRows: [] });
      this.syncDockRangeLabels();
      this.syncDockTimeUi();
      this.saveRuntimeSnapshot({ kind: "error", channels: 0 });
      this.discardActiveRunCaptureIfEmpty();
      return false;
    }
  }

  startLoop() {
    this.syncLoopState();
  }

  stopLoop() {
    this.haltLoop();
  }

  syncLoopState() {
    if (!this.loop) return;
    if (this.screenVisible && !this.executionPaused) {
      this.loop.start();
      if (!this.playbackPaused) {
        this.startViewPlaybackTimer();
      } else {
        this.clearViewPlaybackTimer();
      }
      return;
    }
    this.clearViewPlaybackTimer();
    this.loop.stop();
  }

  haltLoop() {
    this.clearViewPlaybackTimer();
    if (!this.loop) return;
    this.loop.stop();
  }

  stepFrame() {
    if (!this.screenVisible) return;
    if (this.executionPaused) return;
    const maxMadi = this.resolveRuntimeMaxMadiLimit();
    if (maxMadi > 0 && this.runtimeTickCounter >= maxMadi) {
      this.executionPaused = true;
      this.haltLoop();
      this.lastExecPathHint = "실행 완료";
      this.updateRuntimeHint();
      this.setEngineStatus("done");
      this.syncDockTimeUi();
      return;
    }
    const client = this.wasmState?.client;
    if (!client) {
      this.stepServerPlaybackFrame();
      return;
    }

    try {
      const stepped = this.stepClientOne(client);
      const nextState = this.resolveSteppedState(client, stepped.state);
      this.lastState = nextState;
      const hash = typeof client.getStateHash === "function" ? client.getStateHash() : "-";
      this.setHash(hash);
      this.applyRuntimeState(nextState);
      this.runtimeTickCounter += 1;
      this.syncDockTimeUi();
      this.renderInspectorMeta();
      if (maxMadi > 0 && this.runtimeTickCounter >= maxMadi) {
        this.executionPaused = true;
        this.haltLoop();
        this.lastExecPathHint = "실행 완료";
        this.updateRuntimeHint();
        this.setEngineStatus("done");
      }
    } catch (err) {
      console.error("[RunScreen.stepFrame] wasm step failed", err);
      this.haltLoop();
      this.reportRuntimeFailure(
        {
          technical_code: "E_RUNTIME_EXEC_FAILED",
          technical_message: String(err?.message ?? err),
          user_message: "실행 중 오류가 발생했습니다. 문법/입력값을 확인하고 다시 실행해 주세요.",
        },
        { phase: "step" },
      );
      this.saveRuntimeSnapshot({ kind: "error", channels: 0 });
    }
  }

  applyRuntimeState(stateJson, { forceView = false } = {}) {
    const derived = extractRuntimeDerived(stateJson);
    if (!derived) return;
    this.updateMirrorTab(stateJson);
    const prevDerived = this.lastRuntimeDerived;
    const applied = this.applyRuntimeDerived(derived, { forceView, prevDerived });
    this.lastRuntimeDerived = applied ?? derived;
  }

  applyRuntimeDerived(derived, { forceView = false, prevDerived = null } = {}) {
    if (!derived || typeof derived !== "object") return;
    const observation = derived.observation ?? null;
    const outputLog = normalizeConsoleOutputLog(derived.outputLog ?? [], {
      fallbackTick: Number(observation?.values?.tick_id ?? derived?.tick_id ?? 0),
    });
    const outputLines = normalizeConsoleOutputLines(derived.outputLines ?? []);
    const outputRows = normalizeObserveOutputRows(derived.outputRows ?? []);
    let views = derived.views ?? null;
    const incomingGraph = cloneGraphForRunManager(views?.graph ?? null);
    if (incomingGraph) {
      this.lastGraphSnapshot = incomingGraph;
    }
    const prevSpace2d = hasSpace2dDrawable(prevDerived?.mainVisualSpace2d)
      ? prevDerived.mainVisualSpace2d
      : (hasSpace2dDrawable(prevDerived?.views?.space2d) ? prevDerived.views.space2d : null);
    const nativeSpace2d = views?.space2d ?? null;
    const hasNativeSpace2d = hasSpace2dDrawable(nativeSpace2d);
    const allowShapeFallback = Boolean(this.allowShapeFallback);
    const mainVisual = resolveRunMainVisualMode({
      views,
      observation,
      outputLog,
      outputLines,
      outputRows,
      warnings: this.lastParseWarnings,
      allowShapeFallback: allowShapeFallback,
      prevSpace2d,
      lessonRequiredViews: this.lesson?.requiredViews ?? [],
    });

    this.lastSpace2dMode = hasNativeSpace2d
      ? "native"
      : (mainVisual.mode === "debug-fallback" ? "fallback" : "none");
    this.setMainVisualMode(mainVisual.mode);
    this.updateRuntimeHint("", { observation, views, outputRows, outputLines, outputLog });

    if (hasNativeSpace2d && (!views || views.space2d !== nativeSpace2d)) {
      views = {
        ...(views && typeof views === "object" ? views : {}),
        space2d: nativeSpace2d,
      };
    }
    const runtimeOverlayMarkdown = readRuntimeTextMarkdownFromViews(views);
    if (runtimeOverlayMarkdown && runtimeOverlayMarkdown !== this.lastOverlayMarkdown) {
      this.lastOverlayMarkdown = runtimeOverlayMarkdown;
      this.overlay.setContent(runtimeOverlayMarkdown);
      this.renderOverlayTabContent(runtimeOverlayMarkdown, { sourceLabel: "실행 설명" });
      this.renderRuntimeTextContent({
        markdown: runtimeOverlayMarkdown,
        structure: views?.structure ?? null,
        graph: null,
        showGraphPreview: false,
      });
    } else if (views?.structure) {
      this.renderRuntimeTextContent({
        markdown: this.lastRuntimeTextMarkdown,
        structure: views.structure,
        graph: null,
        showGraphPreview: false,
      });
    } else {
      this.renderOverlayTabContent(this.lastOverlayMarkdown, {
        sourceLabel: this.lastOverlayMarkdown ? "교과 설명" : "",
      });
    }
    const shouldUpdateView = forceView || this.screenVisible;

    if (shouldUpdateView) {
      this.dotbogi.appendObservation(observation);
      const fallbackGraph = this.dotbogi?.exportCurrentGraphSnapshot?.() ?? null;
      this.updateLiveRunCaptureFromDerived({ observation, views }, { fallbackGraph });
      this.renderMainVisual(mainVisual);
      this.renderCurrentRuntimeTable(views?.table ?? null);
    } else {
      this.updateLiveRunCaptureFromDerived({ observation, views });
    }
    this.syncDockRangeLabels();
    this.syncDockPanelVisibility({ observation, views, outputRows });
    this.syncRuntimeLayoutProfile(views, mainVisual.mode, { outputRows, outputLines, outputLog });
    this.updateObserveSummary({ observation, views, outputRows, outputLog });
    this.updateRuntimeStatus({ observation, views });
    return {
      observation,
      outputLog,
      outputLines,
      outputRows,
      graphSource: String(views?.graphSource ?? derived?.graphSource ?? "").trim() || null,
      views,
      mainVisualMode: mainVisual.mode,
      mainVisualSpace2d: mainVisual.space2d ?? null,
    };
  }

  setScreenVisible(visible) {
    const next = Boolean(visible);
    if (this.screenVisible === next) return;
    this.screenVisible = next;
    if (!next) {
      this.cancelLiveReplRestart();
      this.clearRuntimeInputState();
      this.clearViewPlaybackTimer();
      this.flushOverlaySession();
    }
    this.syncLoopState();
    this.syncRunControlState();
    this.syncDockTimeUi();
    if (!next) return;
    if (this.lastRuntimeDerived) {
      this.applyRuntimeDerived(this.lastRuntimeDerived, { forceView: true });
    } else if (this.lastState) {
      this.applyRuntimeState(this.lastState, { forceView: true });
    }
    this.refreshStudioLayoutBounds({ persist: false });
    this.consumePendingRunRequest();
  }

  updateRuntimeStatus({ observation = null, views = null } = {}) {
    const hasSpace2d = hasSpace2dDrawable(views?.space2d);
    const { kind, channels } = deriveRunKindAndChannels({ observation, hasSpace2d });
    this.saveRuntimeSnapshot({ kind, channels });
    this.renderInspectorMeta();
  }

  saveRuntimeSnapshot({ kind = "empty", channels = 0 } = {}) {
    const lessonId = String(this.lesson?.id ?? "").trim();
    if (!lessonId) return;
    const normalizedKind = String(kind ?? "empty").trim() || "empty";
    const normalizedChannels = Math.max(0, Number.isFinite(Number(channels)) ? Math.trunc(Number(channels)) : 0);
    const normalizedLaunchKind = normalizeRunLaunchKind(this.lastLaunchKind);
    const snapshotKey = `${lessonId}:${normalizedKind}:${normalizedChannels}:${normalizedLaunchKind}`;
    if (snapshotKey === this.lastRuntimeSnapshotKey) return;
    this.lastRuntimeSnapshotKey = snapshotKey;

    const pref = this.getLessonUiPref(lessonId, { create: true });
    pref.lastRunKind = normalizedKind;
    pref.lastRunChannels = normalizedChannels;
    pref.lastRunAt = new Date().toISOString();
    pref.lastRunHash = String(this.lastRuntimeHash ?? "-");
    pref.lastLaunchKind = normalizedLaunchKind;
    const numericTrackLink = buildNumericTrackRunResultLink({
      lesson: this.lesson,
      runKind: normalizedKind,
      channels: normalizedChannels,
      stateHash: pref.lastRunHash,
      launchKind: normalizedLaunchKind,
      recordedAt: pref.lastRunAt,
    });
    if (numericTrackLink) {
      pref.numericTrackRunResultLink = numericTrackLink;
    } else if (pref.numericTrackRunResultLink) {
      delete pref.numericTrackRunResultLink;
    }
    this.syncNumericTrackRunResultLink(numericTrackLink);
    this.persistUiPrefs();
    this.updateRunSummaryFromPrefs();
  }
}
