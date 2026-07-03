import { showGlobalToast } from "../components/toast.js";
import { FEATURED_SEED_IDS } from "../featured_seed_catalog.js";
import {
  buildFirstRunBadgeLabel,
  buildFirstRunHintText,
  resolveFirstRunStepByTarget,
} from "../first_run_catalog.js";
import { resolveLessonCardPreviewViewModel } from "../preview_session.js";
import { applyPreviewViewModelMetadata } from "../preview_view_model.js";
import {
  buildStudioLocalPackageManifest,
  buildStudioLocalPackagePayload,
  formatStudioLocalPackageIndexText,
  formatStudioLocalPackageError,
  validateStudioLocalPackagePayload,
} from "../studio_local_share_package.js";
import {
  lessonHasPreviewDescriptor,
} from "../view_family_contract.js";
import {
  NUMERIC_TRACK_LESSON_IDS,
  buildNumericTrackLessonPreview,
  buildNumericTrackReportExport,
  buildNumericTrackResultCompare,
  buildNumericTrackResultCompareExport,
  buildNumericTrackResultCompareHistory,
  buildNumericTrackResultCompareHistoryExport,
  buildNumericTrackResultCompareHistoryReport,
  buildNumericTrackResultCompareHistoryReportExport,
  buildNumericTrackResultCompareHistoryReportTable,
  buildNumericTrackResultCompareHistoryReportTableExport,
  buildNumericTrackResultCompareHistoryReportTableSummary,
  buildNumericTrackResultCompareHistoryReportTableSummaryExport,
  buildNumericTrackResultCompareHistoryReportTableStatus,
  buildNumericTrackResultCompareHistoryReportTableStatusBadge,
  buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11y,
  buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yExport,
  buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatus,
  buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExport,
  buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExportSummary,
  buildNumericTrackResultCompareHistoryReportTableStatusBadgeExport,
  buildNumericTrackResultCompareHistoryReportTableStatusExport,
  buildNumericReportWorkflowConsolidation,
  buildNumericTrackResultReopenTarget,
  buildNumericTrackResultSummaryExport,
  buildNumericTrackResultHistorySnapshot,
  buildNumericTrackResultTimelineView,
  formatNumericTrackLessonPreviewText,
  formatNumericTrackReportExportText,
  formatNumericTrackResultCompareExportText,
  formatNumericTrackResultCompareHistoryExportText,
  formatNumericTrackResultCompareHistoryReportText,
  formatNumericTrackResultCompareHistoryReportExportText,
  formatNumericTrackResultCompareHistoryReportTableText,
  formatNumericTrackResultCompareHistoryReportTableExportText,
  formatNumericTrackResultCompareHistoryReportTableSummaryText,
  formatNumericTrackResultCompareHistoryReportTableSummaryExportText,
  formatNumericTrackResultCompareHistoryReportTableStatusText,
  formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yText,
  formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yExportText,
  formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusText,
  formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExportText,
  formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExportSummaryText,
  formatNumericTrackResultCompareHistoryReportTableStatusBadgeText,
  formatNumericTrackResultCompareHistoryReportTableStatusBadgeExportText,
  formatNumericTrackResultCompareHistoryReportTableStatusExportText,
  formatNumericReportWorkflowConsolidationText,
  formatNumericTrackResultCompareHistoryText,
  formatNumericTrackResultCompareText,
  formatNumericTrackResultHistoryText,
  formatNumericTrackResultReopenTargetText,
  formatNumericTrackResultSummaryExportText,
  formatNumericTrackResultTimelineViewText,
  isNumericTrackLesson,
  normalizeNumericTrackRunResultLink,
} from "../numeric_curriculum_track.js";

function shouldShowLegacyBrowseControls() {
  try {
    if (document?.body?.classList?.contains("dev-surfaces-enabled") === true) return true;
    const host = String(globalThis?.location?.hostname ?? "").trim().toLowerCase();
    const localHost = !host || host === "localhost" || host === "127.0.0.1" || host === "::1";
    if (!localHost) return false;
    const params = new URLSearchParams(globalThis?.location?.search ?? "");
    const raw = String(params.get("devSurfaces") ?? "").trim().toLowerCase();
    return raw === "1" || raw === "true" || raw === "yes" || globalThis?.SEAMGRIM_DEV_SURFACES === true;
  } catch (_) {
    return false;
  }
}

function insertHtmlBefore(parent, anchor, html) {
  if (!parent || !html) return;
  const template = document.createElement("template");
  template.innerHTML = String(html).trim();
  parent.insertBefore(template.content, anchor || null);
}

function buildSafeJsonDownloadName(value, fallback = "seamgrim-course-package") {
  const raw = String(value ?? "").trim() || fallback;
  const withoutExtension = raw.replace(/\.json$/i, "");
  const safe = withoutExtension
    .replace(/[^a-zA-Z0-9._-]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .slice(0, 120) || fallback;
  return `${safe}.json`;
}

function saveJsonTextToFile(text, filename = "seamgrim-course-package.json") {
  const blob = new Blob([String(text ?? "")], { type: "application/json;charset=utf-8" });
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = buildSafeJsonDownloadName(filename);
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(link.href);
  return link.download;
}

function ensureDevBrowseControls(root) {
  if (!root || !shouldShowLegacyBrowseControls()) return;
  const browseTabs = root.querySelector(".browse-tabs");
  if (browseTabs && !root.querySelector("#btn-filter-numeric-track")) {
    insertHtmlBefore(
      browseTabs,
      root.querySelector("#btn-create"),
      `
        <button id="btn-filter-numeric-track" class="ghost" type="button">그래프·표 수업</button>
        <button id="btn-filter-numeric-track-results" class="ghost" type="button">수치 결과</button>
        <button id="btn-preset-featured-seed-quick-recent" class="ghost" type="button">Alt+6 최근 보기</button>
        <button id="btn-copy-numeric-track-result-summary" class="ghost" type="button">수치 결과 요약 복사</button>
        <button id="btn-toggle-numeric-track-result-timeline" class="ghost" type="button">수치 결과 흐름</button>
        <button id="btn-copy-browse-preset-link" class="ghost" type="button">프리셋 링크 복사</button>
        <button class="browse-tab" type="button" disabled title="AGE5">커뮤니티</button>
      `,
    );
  }
  const filters = root.querySelector(".browse-filters");
  if (filters && !root.querySelector("#filter-quality")) {
    insertHtmlBefore(
      filters,
      root.querySelector("#filter-query"),
      `
        <select id="filter-quality">
          <option value="">품질: 전체</option>
          <option value="recommended">품질: 추천</option>
          <option value="reviewed">품질: 검수완료</option>
          <option value="experimental">품질: 실험용</option>
        </select>
        <select id="filter-seed-scope">
          <option value="">seed: 전체</option>
          <option value="featured_seed">seed: 신규 10종만</option>
          <option value="seed_only">seed: 전체</option>
        </select>
        <select id="filter-run-status">
          <option value="">최근 실행: 전체</option>
          <option value="success">최근 실행: 성공</option>
          <option value="error">최근 실행: 실패</option>
          <option value="none">최근 실행: 미실행</option>
        </select>
        <select id="filter-run-launch">
          <option value="">실행 경로: 전체</option>
          <option value="featured_seed_quick">실행 경로: Alt+6</option>
          <option value="browse_select">실행 경로: 탐색 선택</option>
          <option value="editor_run">실행 경로: 편집 실행</option>
          <option value="manual">실행 경로: 수동/기타</option>
          <option value="none">실행 경로: 기록 없음</option>
        </select>
        <select id="filter-warning-status">
          <option value="">구식 범위주석: 전체</option>
          <option value="has_legacy_warning">구식 범위주석: 있음</option>
          <option value="clean">구식 범위주석: 없음</option>
        </select>
        <select id="filter-launch-profile">
          <option value="">실행 시작: 기본</option>
          <option value="student">실행 시작: 학생</option>
          <option value="teacher">실행 시작: 교사</option>
        </select>
        <select id="filter-sort">
          <option value="teacher">정렬: 교과 우선</option>
          <option value="recent">정렬: 최근 실행순</option>
          <option value="featured_seed_quick_recent">정렬: Alt+6 최근 우선</option>
          <option value="featured_seed">정렬: 신규 seed 우선</option>
          <option value="default">정렬: 기본</option>
          <option value="legacy_warning">정렬: 구식 범위주석 많은순</option>
        </select>
      `,
    );
  }
}

const QUALITY_BADGE = Object.freeze({
  recommended: { label: "★ 추천", cls: "badge-recommended" },
  reviewed: { label: "✓ 검수완료", cls: "badge-reviewed" },
  experimental: { label: "교과", cls: "badge-experimental" },
});
const TEACHER_CATALOG_SUBJECT_RANK = Object.freeze({
  physics: 0,
  math: 1,
  econ: 2,
  science: 3,
  biology: 4,
  ecology: 5,
  engineering: 6,
  game: 20,
  cs: 30,
});
const TEACHER_DEFAULT_COURSE_SUBJECTS = Object.freeze([
  "physics",
  "math",
  "econ",
  "science",
]);
const TEACHER_CATALOG_LESSON_RANK = Object.freeze({
  rep_physics_velocity_history_v1: 0,
  rep_phys_projectile_xy_v1: 1,
  rep_math_function_line_v1: 2,
  rep_econ_growth_compound_v1: 3,
  rep_econ_supply_demand_tax_v1: 4,
  rep_science_phase_change_timeline_v1: 5,
  rep_grid_game_state_drop_v1: 20,
});
const LEGACY_WARNING_CODE = "W_LEGACY_RANGE_COMMENT_DEPRECATED";
const PROJECT_PREFIX = "solutions/seamgrim_ui_mvp/";
const DEFAULT_FEDERATED_API_CANDIDATES = Object.freeze(["/api/lessons/inventory"]);
const DEFAULT_FEDERATED_FILE_CANDIDATES = Object.freeze([]);
const DEFAULT_SAMPLE_CANDIDATES = Object.freeze(["/samples/index.json"]);
const RUN_UI_PREFS_STORAGE_KEY = "seamgrim.ui.run_prefs.v1";
const BROWSE_UI_PREFS_STORAGE_KEY = "seamgrim.ui.browse_prefs.v1";
const BROWSE_PRESET_QUERY_KEY = "browsePreset";
const PREVIEW_CONCURRENCY_LIMIT = 3;

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

function normalizeSubject(subject) {
  const raw = String(subject ?? "").trim().toLowerCase();
  if (!raw) return "";
  if (raw === "economy") return "econ";
  return raw;
}

function isTeacherDefaultCourseSubject(subject) {
  return TEACHER_DEFAULT_COURSE_SUBJECTS.includes(normalizeSubject(subject));
}

function normalizeGrade(grade) {
  return String(grade ?? "").trim().toLowerCase();
}

function formatSubjectLabel(subject) {
  const key = normalizeSubject(subject);
  const labels = {
    physics: "물리",
    math: "수학",
    econ: "경제",
    science: "과학",
    cs: "컴퓨터과학",
  };
  return labels[key] || String(subject ?? "").trim() || "-";
}

function buildSubjectCountRows(lessons) {
  if (!Array.isArray(lessons) || lessons.length === 0) return [];
  const counts = new Map();
  lessons.forEach((lesson) => {
    const subject = normalizeSubject(lesson?.subject);
    const label = formatSubjectLabel(subject);
    if (!label || label === "-") return;
    const current = counts.get(subject) || { subject, label, count: 0 };
    current.count += 1;
    counts.set(subject, current);
  });
  const preferredOrder = ["physics", "math", "econ", "science", "cs"];
  const orderedLabels = [
    ...preferredOrder.filter((subject) => counts.has(subject)),
    ...Array.from(counts.keys()).filter((subject) => !preferredOrder.includes(subject)).sort(),
  ];
  return orderedLabels.map((subject) => counts.get(subject)).filter(Boolean);
}

function buildSubjectCountSummary(lessons) {
  const rows = buildSubjectCountRows(lessons).map((row) => `${row.label} ${row.count}개`);
  return rows.length ? `대표 과목: ${rows.join(" · ")}` : "";
}

function formatRequiredViewLabel(view) {
  const key = String(view ?? "").trim().toLowerCase();
  const labels = {
    graph: "그래프",
    table: "표",
    text: "설명",
    space2d: "그림",
    console_grid: "콘솔 격자",
    grid2d: "2D 격자",
    grid: "격자",
  };
  return labels[key] || key;
}

function getLessonRequiredViews(lesson) {
  return Array.isArray(lesson?.requiredViews)
    ? lesson.requiredViews
    : Array.isArray(lesson?.required_views)
      ? lesson.required_views
      : [];
}

function buildCourseSurfaceText(lesson) {
  const views = getLessonRequiredViews(lesson);
  const viewText = views
    .map(formatRequiredViewLabel)
    .filter(Boolean)
    .slice(0, 3)
    .join(" · ");
  const suffix = viewText ? ` · ${viewText}` : "";
  return `DDN 교과 실행${suffix}`;
}

function buildCourseResultText(lesson) {
  const viewText = getLessonRequiredViews(lesson)
    .map(formatRequiredViewLabel)
    .filter(Boolean)
    .slice(0, 4)
    .join(" · ");
  return viewText ? `결과 확인: ${viewText}` : "결과 확인: 실행 화면";
}

function buildCourseDeliveryText(lesson) {
  const grade = formatGradeLabel(lesson?.grade);
  const subject = formatSubjectLabel(lesson?.subject);
  const course = [grade, subject].filter((item) => item && item !== "-").join(" · ");
  const prefix = course ? `${course} 수업` : "교과 수업";
  return `${prefix} · 학생 실행 · 교사용 배포`;
}

function buildCourseReadinessText(lesson) {
  const views = getLessonRequiredViews(lesson);
  const goalCount = buildCourseGoalTexts(lesson).length;
  const missionCount = buildCourseMissionTexts(lesson).length;
  if (views.length > 0 && goalCount > 0 && missionCount > 0) {
    return "수업 준비 완료 · 활동 · 결과 보기 · 배포";
  }
  if (views.length > 0) return "수업 준비 확인 필요 · 결과 보기 준비";
  return "수업 준비 확인 필요";
}

function buildCourseGoalTexts(lesson) {
  const rows = Array.isArray(lesson?.goals) ? lesson.goals : [];
  return rows
    .map((item) => String(item ?? "").trim())
    .filter(Boolean)
    .slice(0, 2);
}

function buildCourseMissionTexts(lesson) {
  const rows = Array.isArray(lesson?.missions) ? lesson.missions : [];
  const missions = rows
    .map((item) => String(item ?? "").trim())
    .filter(Boolean)
    .slice(0, 2);
  if (missions.length > 0) return missions;
  const views = getLessonRequiredViews(lesson);
  const viewText = views.map(formatRequiredViewLabel).filter(Boolean).slice(0, 3).join(", ");
  return [
    "학생 시작으로 수업을 실행한다",
    viewText ? `${viewText} 결과를 보고 수업 내용을 확인한다` : "결과 화면을 보고 수업 내용을 확인한다",
  ];
}

function buildCourseActivityText(lesson) {
  const missions = buildCourseMissionTexts(lesson);
  const firstMission = missions.find((item) => String(item ?? "").trim());
  if (firstMission) return `오늘 활동: ${firstMission}`;
  const views = getLessonRequiredViews(lesson);
  const viewText = views.map(formatRequiredViewLabel).filter(Boolean).slice(0, 2).join(", ");
  return viewText ? `오늘 활동: ${viewText} 결과를 함께 확인` : "오늘 활동: 실행 결과 함께 확인";
}

function formatGradeLabel(grade) {
  const key = normalizeGrade(grade);
  const labels = {
    elementary: "초등",
    middle: "중등",
    high: "고등",
    college: "대학",
    all: "전체",
  };
  return labels[key] || String(grade ?? "").trim() || "전체";
}

function normalizeQuality(quality) {
  const raw = String(quality ?? "").trim().toLowerCase();
  return QUALITY_BADGE[raw] ? raw : "experimental";
}

function normalizeSource(source) {
  return String(source ?? "").trim().toLowerCase();
}

function normalizeStringList(value) {
  return Array.isArray(value)
    ? value.map((item) => String(item ?? "").trim()).filter(Boolean)
    : [];
}

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function renderDetailList(title, rows) {
  const items = normalizeStringList(rows);
  if (items.length === 0) return "";
  return `
    <section class="detail-curriculum-section">
      <div class="detail-curriculum-title">${escapeHtml(title)}</div>
      <ul class="detail-curriculum-list">
        ${items.map((item) => `<li>${escapeHtml(item)}</li>`).join("")}
      </ul>
    </section>
  `;
}

function buildDetailStudentPreviewRows(lesson) {
  const title = String(lesson?.title ?? lesson?.id ?? "수업").trim() || "수업";
  const goalRows = buildCourseGoalTexts(lesson);
  const missionRows = buildCourseMissionTexts(lesson);
  return [
    `제목: ${title}`,
    goalRows.length ? `목표: ${goalRows[0]}` : "목표: 수업 실행 결과 확인",
    missionRows.length ? `활동: ${missionRows[0]}` : "활동: 결과 화면을 보고 수업 내용을 확인한다",
    buildCourseResultText(lesson),
    "실행 버튼: 받은 수업 실행",
  ];
}

function renderDetailStudentPreviewSection(lesson) {
  return renderDetailList("학생 화면 미리보기", buildDetailStudentPreviewRows(lesson));
}

function clipDdnPreviewText(text) {
  const normalized = String(text ?? "").trim();
  if (!normalized) return "";
  const lines = normalized.split(/\r?\n/);
  const clipped = lines.slice(0, 28).join("\n");
  return lines.length > 28 ? `${clipped}\n...` : clipped;
}

function renderDetailDdnPreviewSection() {
  return `
    <section class="detail-curriculum-section detail-ddn-source" data-detail-ddn-preview data-detail-ddn-preview-status="loading">
      <div class="detail-ddn-source-head">
        <div class="detail-curriculum-title">DDN 원문</div>
        <button class="ghost detail-ddn-source-copy" type="button" data-detail-ddn-copy disabled>원문 복사</button>
      </div>
      <pre class="detail-ddn-source-code" data-detail-ddn-preview-code>DDN 원문을 불러오는 중입니다.</pre>
      <div class="detail-ddn-source-path" data-detail-ddn-preview-path></div>
    </section>
  `;
}

function buildDetailRunReadinessRows(lesson) {
  const views = getLessonRequiredViews(lesson);
  const viewText = views.map(formatRequiredViewLabel).filter(Boolean).slice(0, 4).join(", ");
  const missionRows = buildCourseMissionTexts(lesson);
  return [
    `수업 준비 상태: ${buildCourseReadinessText(lesson)}`,
    "학생으로 실행하면 이 수업이 바로 학생 화면에서 열립니다",
    viewText ? `결과 확인: ${viewText}` : "결과 확인: 기본 실행 화면",
    missionRows.length ? `첫 활동: ${missionRows[0]}` : "첫 활동: 그래프와 표를 보고 수업 내용을 확인하기",
    "교사용 배포 준비에서 같은 수업과 활동을 학생에게 보낼 수 있습니다",
  ];
}

function renderDetailRunReadinessSection(lesson) {
  const rows = buildDetailRunReadinessRows(lesson);
  return `
    <section class="detail-curriculum-section detail-run-readiness" data-detail-run-readiness>
      <div class="detail-curriculum-title">실행 전 확인</div>
      <ol class="detail-run-readiness-list">
        ${rows.map((row) => `<li>${escapeHtml(row)}</li>`).join("")}
      </ol>
    </section>
  `;
}

function renderDetailCourseFlowSection(lesson, courseSequenceText = "") {
  const activityText = buildCourseActivityText(lesson).replace(/^오늘 활동:\s*/, "");
  const resultText = buildCourseResultText(lesson).replace(/^결과 확인:\s*/, "");
  const deliveryText = buildCourseDeliveryText(lesson);
  const sequenceText = String(courseSequenceText ?? "").trim();
  return `
    <section class="detail-curriculum-section detail-course-flow" data-detail-course-flow>
      <div class="detail-curriculum-title">수업 흐름</div>
      ${sequenceText ? `<div class="detail-course-sequence" data-detail-course-sequence>${escapeHtml(sequenceText)}</div>` : ""}
      <div class="detail-course-flow-grid">
        <div class="detail-course-flow-item">
          <span>학생 활동</span>
          <strong>${escapeHtml(activityText || "수업 실행")}</strong>
        </div>
        <div class="detail-course-flow-item">
          <span>결과 확인</span>
          <strong>${escapeHtml(resultText || "실행 화면")}</strong>
        </div>
        <div class="detail-course-flow-item">
          <span>배포</span>
          <strong>${escapeHtml(deliveryText)}</strong>
        </div>
      </div>
    </section>
  `;
}

function mergeLessonPools(...pools) {
  const merged = new Map();
  pools.forEach((pool) => {
    if (!Array.isArray(pool)) return;
    pool.forEach((lesson) => {
      const id = String(lesson?.id ?? "").trim();
      if (!id || merged.has(id)) return;
      merged.set(id, lesson);
    });
  });
  return Array.from(merged.values());
}

function setElementDatasetValue(element, key, value) {
  if (!element || !element.dataset) return;
  element.dataset[key] = String(value ?? "");
}

function isFeaturedSeedLesson(lesson) {
  const lessonId = String(lesson?.id ?? "").trim();
  if (!lessonId || !FEATURED_SEED_IDS.includes(lessonId)) return false;
  return normalizeSource(lesson?.source) === "seed";
}

function normalizeCandidateList(value, fallback = []) {
  const rows = Array.isArray(value) ? value : Array.isArray(fallback) ? fallback : [];
  const seen = new Set();
  const out = [];
  rows.forEach((row) => {
    const candidate = String(row ?? "").trim();
    if (!candidate || seen.has(candidate)) return;
    seen.add(candidate);
    out.push(candidate);
  });
  return out;
}

function lessonMatchesQuery(lesson, query) {
  if (!query) return true;
  const hay = [lesson.id, lesson.title, lesson.subject, lesson.description, lesson.source]
    .map((item) => String(item ?? "").toLowerCase())
    .join(" ");
  return hay.includes(query.toLowerCase());
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

function normalizeLaunchKind(raw) {
  const kind = String(raw ?? "").trim().toLowerCase();
  if (!kind) return "";
  if (kind === "featured_seed_quick") return "featured_seed_quick";
  if (kind === "browse_select") return "browse_select";
  if (kind === "browse_select_student") return "browse_select";
  if (kind === "browse_select_teacher") return "browse_select";
  if (kind === "editor_run") return "editor_run";
  if (kind === "manual") return "manual";
  return "manual";
}

function normalizeRunLaunchFilter(raw) {
  const kind = String(raw ?? "").trim().toLowerCase();
  if (!kind) return "";
  if (kind === "none") return "none";
  if (kind === "featured_seed_quick") return "featured_seed_quick";
  if (kind === "browse_select") return "browse_select";
  if (kind === "editor_run") return "editor_run";
  if (kind === "manual") return "manual";
  return "";
}

function normalizeLaunchProfile(raw) {
  const profile = String(raw ?? "").trim().toLowerCase();
  if (profile === "student") return "student";
  if (profile === "teacher") return "teacher";
  return "";
}

function resolveBrowsePresetId(filter) {
  const runLaunch = normalizeRunLaunchFilter(filter?.runLaunch ?? "");
  const sort = String(filter?.sort ?? "").trim().toLowerCase();
  if (runLaunch === "featured_seed_quick" && sort === "featured_seed_quick_recent") {
    return "featured_seed_quick_recent";
  }
  return "";
}

function buildBrowsePresetShareUrl(presetId = "") {
  try {
    const href = String(window?.location?.href ?? "").trim();
    if (!href) return "";
    const url = new URL(href);
    const normalized = String(presetId ?? "").trim().toLowerCase();
    if (normalized) {
      url.searchParams.set(BROWSE_PRESET_QUERY_KEY, normalized);
    } else {
      url.searchParams.delete(BROWSE_PRESET_QUERY_KEY);
    }
    return url.toString();
  } catch (_) {
    return "";
  }
}

export class BrowseScreen {
  constructor({
    root,
    onLessonSelect,
    onCreate,
    onOpenLocalPackageFile,
    onOpenLegacyGuideExample,
    onOpenAdvanced,
    federatedApiCandidates,
    federatedFileCandidates,
    sampleCandidates,
    featuredSeedEnabled,
  } = {}) {
    this.root = root;
    this.onLessonSelect = typeof onLessonSelect === "function" ? onLessonSelect : async () => {};
    this.onCreate = typeof onCreate === "function" ? onCreate : () => {};
    this.onOpenLocalPackageFile =
      typeof onOpenLocalPackageFile === "function" ? onOpenLocalPackageFile : async () => {};
    this.onOpenLegacyGuideExample =
      typeof onOpenLegacyGuideExample === "function" ? onOpenLegacyGuideExample : () => {};
    this.onOpenAdvanced = typeof onOpenAdvanced === "function" ? onOpenAdvanced : () => {};

    this.lessons = [];
    this.sampleResults = [];
    this.searchResults = [];
    this.federatedLoadState = "idle";
    this.sampleLoadState = "idle";
    this.activeTab = "official";
    this.federatedApiCandidates = normalizeCandidateList(
      federatedApiCandidates,
      DEFAULT_FEDERATED_API_CANDIDATES,
    );
    this.federatedFileCandidates = normalizeCandidateList(
      federatedFileCandidates,
      DEFAULT_FEDERATED_FILE_CANDIDATES,
    );
    this.sampleCandidates = normalizeCandidateList(
      sampleCandidates,
      DEFAULT_SAMPLE_CANDIDATES,
    );
    this.featuredSeedEnabled = featuredSeedEnabled !== false;

    this.filter = {
      grade: "",
      subject: "",
      quality: "",
      numericTrack: false,
      numericTrackResults: false,
      numericTrackTimeline: false,
      seedScope: "",
      runStatus: "",
      runLaunch: "",
      launchProfile: "",
      warningStatus: "",
      sort: "teacher",
      query: "",
    };
    this.runUiPrefs = {
      lessons: {},
    };
    this.lastBrowsePresetId = "";
    this.lessonPreviewCache = new Map();
    this.previewQueue = [];
    this.activePreviewCount = 0;
    this.previewObserver = null;
    this.previewPayloadByElement = new WeakMap();
    this.detailLesson = null;
    this.hasAutoSelectedCourseDetail = false;
    this.detailDdnPreviewCache = new Map();
    this.detailDdnPreviewSeq = 0;
  }

  init() {
    ensureDevBrowseControls(this.root);
    this.tabButtons = Array.from(this.root.querySelectorAll(".browse-tab[data-tab]"));
    this.gradeSelect = this.root.querySelector("#filter-grade");
    this.subjectSelect = this.root.querySelector("#filter-subject");
    this.qualitySelect = this.root.querySelector("#filter-quality");
    this.seedScopeSelect = this.root.querySelector("#filter-seed-scope");
    this.runStatusSelect = this.root.querySelector("#filter-run-status");
    this.runLaunchSelect = this.root.querySelector("#filter-run-launch");
    this.launchProfileSelect = this.root.querySelector("#filter-launch-profile");
    this.warningStatusSelect = this.root.querySelector("#filter-warning-status");
    this.sortSelect = this.root.querySelector("#filter-sort");
    this.queryInput = this.root.querySelector("#filter-query");
    this.presetFeaturedSeedQuickRecentButton = this.root.querySelector("#btn-preset-featured-seed-quick-recent");
    this.numericTrackButton = this.root.querySelector("#btn-filter-numeric-track");
    this.numericTrackResultsButton = this.root.querySelector("#btn-filter-numeric-track-results");
    this.copyNumericTrackResultSummaryButton = this.root.querySelector("#btn-copy-numeric-track-result-summary");
    this.numericTrackTimelineButton = this.root.querySelector("#btn-toggle-numeric-track-result-timeline");
    this.numericTrackTimelinePanel = this.root.querySelector("#numeric-track-result-timeline-panel");
    this.numericTrackTimelineList = this.root.querySelector("#numeric-track-result-timeline-list");
    this.numericTrackCompareButton = this.root.querySelector("#btn-compare-numeric-track-results");
    this.numericTrackComparePanel = this.root.querySelector("#numeric-track-result-compare-panel");
    this.numericTrackCompareHistoryButton = this.root.querySelector("#btn-show-numeric-track-result-compare-history");
    this.numericTrackCompareHistoryPanel = this.root.querySelector("#numeric-track-result-compare-history-panel");
    this.copyBrowsePresetLinkButton = this.root.querySelector("#btn-copy-browse-preset-link");
    this.openLocalPackageButton = this.root.querySelector("#btn-open-local-package");
    this.localPackageFileInput = this.root.querySelector("#input-local-package-file");
    this.localPackageImportStatusEl = this.root.querySelector("[data-local-package-import-status]");
    this.pasteLocalPackageButton = this.root.querySelector("#btn-paste-local-package");
    this.copyCoursePackageButton = this.root.querySelector("#btn-copy-course-package");
    this.downloadCoursePackageButton = this.root.querySelector("#btn-download-course-package");
    this.coursePackageExportStatusEl = this.root.querySelector("[data-course-package-export-status]");
    this.localPackagePastePanel = this.root.querySelector("[data-local-package-paste-panel]");
    this.localPackagePasteText = this.root.querySelector("#local-package-paste-text");
    this.localPackagePastePreview = this.root.querySelector("[data-local-package-paste-preview]");
    this.localPackagePasteOpenButton = this.root.querySelector("#btn-local-package-paste-open");
    this.localPackagePasteCancelButton = this.root.querySelector("#btn-local-package-paste-cancel");
    this.legacyGuideHintEl = this.root.querySelector("#browse-legacy-guide-hint");
    this.grid = this.root.querySelector("#lesson-card-grid");
    this.courseSummaryEl = this.root.querySelector("[data-course-catalog-summary]");
    if (!this.courseSummaryEl && this.grid && typeof document?.createElement === "function") {
      this.courseSummaryEl = document.createElement("div");
      this.courseSummaryEl.className = "course-catalog-summary";
      this.courseSummaryEl.dataset.courseCatalogSummary = "";
      this.grid.parentElement?.insertBefore(this.courseSummaryEl, this.grid);
    }
    this.courseSubjectShortcutsEl = this.root.querySelector("[data-course-subject-shortcuts]");
    if (!this.courseSubjectShortcutsEl && this.courseSummaryEl && typeof document?.createElement === "function") {
      this.courseSubjectShortcutsEl = document.createElement("div");
      this.courseSubjectShortcutsEl.className = "course-subject-shortcuts hidden";
      this.courseSubjectShortcutsEl.dataset.courseSubjectShortcuts = "";
      this.courseSummaryEl.parentElement?.insertBefore(
        this.courseSubjectShortcutsEl,
        this.courseSummaryEl.nextSibling,
      );
    }
    this.detailPanelEl = this.root.querySelector("#catalog-detail-panel");
    this.detailSubjectBadgeEl = this.root.querySelector("#detail-subject-badge");
    this.detailTitleEl = this.root.querySelector("#detail-title");
    this.detailDescEl = this.root.querySelector("#detail-desc");
    this.detailKeywordsEl = this.root.querySelector("#detail-keywords");
    this.detailCurriculumEl = this.root.querySelector("#detail-curriculum");
    this.detailOpenBtn = this.root.querySelector("#btn-open-in-studio");
    this.detailTeacherOpenBtn = this.root.querySelector("#btn-open-in-studio-teacher");
    this.detailCopyNumericTrackReportBtn = this.root.querySelector("#btn-copy-numeric-track-report");
    this.detailCloseBtn = this.root.querySelector("#btn-detail-close");
    this.loadBrowsePrefs();
    this.applyFeaturedSeedVisibilityPolicy();
    this.applySimplifiedCatalogFilters();

    this.root.querySelector("#btn-create")?.addEventListener("click", () => {
      this.onCreate();
    });
    this.openLocalPackageButton?.addEventListener("click", () => {
      if (this.localPackageFileInput) this.localPackageFileInput.value = "";
      if (this.localPackagePasteText) this.localPackagePasteText.value = "";
      this.clearLocalPackagePastePreview();
      this.toggleLocalPackagePastePanel(false);
      this.setLocalPackagePasteStatus("idle", "");
      this.localPackageFileInput?.click();
    });
    this.pasteLocalPackageButton?.addEventListener("click", () => {
      this.setLocalPackagePasteStatus("idle", "");
      this.toggleLocalPackagePastePanel(true);
      this.updateLocalPackagePastePreview();
    });
    this.copyCoursePackageButton?.addEventListener("click", () => {
      void this.handleCopyCoursePackage();
    });
    this.downloadCoursePackageButton?.addEventListener("click", () => {
      void this.handleDownloadCoursePackage();
    });
    this.localPackagePasteCancelButton?.addEventListener("click", () => {
      this.toggleLocalPackagePastePanel(false);
    });
    this.localPackagePasteText?.addEventListener("input", () => {
      this.clearLocalPackagePasteErrorStatus();
      this.updateLocalPackagePastePreview();
    });
    this.localPackagePasteText?.addEventListener("keydown", async (event) => {
      if (event.key === "Escape") {
        event.preventDefault();
        this.toggleLocalPackagePastePanel(false);
        this.pasteLocalPackageButton?.focus?.();
        return;
      }
      if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
        event.preventDefault();
        await this.handleOpenLocalPackagePaste();
      }
    });
    this.localPackagePasteOpenButton?.addEventListener("click", async () => {
      await this.handleOpenLocalPackagePaste();
    });
    this.localPackageFileInput?.addEventListener("change", async () => {
      const file = this.localPackageFileInput?.files?.[0] ?? null;
      if (!file) return;
      const originalLabel = this.openLocalPackageButton?.textContent ?? "배포 열기";
      if (this.openLocalPackageButton) {
        this.openLocalPackageButton.disabled = true;
        this.openLocalPackageButton.textContent = "배포 여는 중...";
      }
      try {
        await this.onOpenLocalPackageFile(file);
      } finally {
        if (this.localPackageFileInput) this.localPackageFileInput.value = "";
        if (this.openLocalPackageButton) {
          this.openLocalPackageButton.disabled = false;
          this.openLocalPackageButton.textContent = originalLabel;
        }
      }
    });

    this.root.querySelector("#btn-advanced-browse")?.addEventListener("click", () => {
      this.onOpenAdvanced();
    });
    this.courseSubjectShortcutsEl?.addEventListener("click", (event) => {
      const button = event.target?.closest?.("[data-course-subject-filter]");
      if (!button || !this.courseSubjectShortcutsEl?.contains(button)) return;
      const subject = String(button.dataset.courseSubjectFilter ?? "").trim();
      this.filter.subject = subject === "__all__" ? "" : subject;
      if (this.subjectSelect) this.subjectSelect.value = this.filter.subject;
      this.render();
    });
    this.presetFeaturedSeedQuickRecentButton?.addEventListener("click", () => {
      this.applyFeaturedSeedQuickRecentPreset();
    });
    this.copyBrowsePresetLinkButton?.addEventListener("click", () => {
      void this.handleCopyBrowsePresetLink();
    });
    this.numericTrackButton?.addEventListener("click", () => {
      this.toggleNumericTrackFilter();
    });
    this.numericTrackResultsButton?.addEventListener("click", () => {
      this.toggleNumericTrackResultsFilter();
    });
    this.copyNumericTrackResultSummaryButton?.addEventListener("click", () => {
      void this.handleCopyNumericTrackResultSummary();
    });
    this.numericTrackTimelineButton?.addEventListener("click", () => {
      this.toggleNumericTrackResultTimeline();
    });
    this.numericTrackCompareButton?.addEventListener("click", () => {
      this.showNumericTrackResultCompare();
    });
    this.numericTrackCompareHistoryButton?.addEventListener("click", () => {
      this.showNumericTrackResultCompareHistory();
    });
    this.detailCloseBtn?.addEventListener("click", () => {
      this.hideLessonDetail();
    });
    this.detailOpenBtn?.addEventListener("click", () => {
      if (!this.detailLesson) return;
      void this.onLessonSelect({
        ...this.detailLesson,
        launchProfile: "student",
      }, { autoExecute: true });
    });
    this.detailTeacherOpenBtn?.addEventListener("click", () => {
      if (!this.detailLesson) return;
      void this.onLessonSelect({
        ...this.detailLesson,
        launchProfile: "teacher",
      }, { autoExecute: true });
    });
    this.detailCopyNumericTrackReportBtn?.addEventListener("click", () => {
      void this.handleCopyNumericTrackReport();
    });

    this.tabButtons.forEach((button) => {
      button.addEventListener("click", async () => {
        const tab = String(button.dataset.tab ?? "official");
        this.activeTab = tab;
        this.tabButtons.forEach((item) => item.classList.toggle("active", item === button));
        if (tab === "examples") {
          await this.loadSampleResults();
        }
        if (tab === "search") {
          await this.loadFederatedResults();
        }
        this.render();
      });
    });

    this.gradeSelect?.addEventListener("change", () => {
      this.filter.grade = String(this.gradeSelect.value ?? "");
      this.render();
    });
    this.subjectSelect?.addEventListener("change", () => {
      this.filter.subject = String(this.subjectSelect.value ?? "");
      this.render();
    });
    this.qualitySelect?.addEventListener("change", () => {
      this.filter.quality = String(this.qualitySelect.value ?? "");
      this.render();
    });
    this.seedScopeSelect?.addEventListener("change", () => {
      this.filter.seedScope = String(this.seedScopeSelect.value ?? "");
      this.saveBrowsePrefs();
      this.render();
    });
    this.runStatusSelect?.addEventListener("change", () => {
      this.filter.runStatus = String(this.runStatusSelect.value ?? "");
      this.render();
    });
    this.runLaunchSelect?.addEventListener("change", () => {
      this.filter.runLaunch = normalizeRunLaunchFilter(this.runLaunchSelect.value);
      this.saveBrowsePrefs();
      this.render();
    });
    this.launchProfileSelect?.addEventListener("change", () => {
      this.filter.launchProfile = normalizeLaunchProfile(this.launchProfileSelect.value);
      this.saveBrowsePrefs();
      this.render();
    });
    this.warningStatusSelect?.addEventListener("change", () => {
      this.filter.warningStatus = String(this.warningStatusSelect.value ?? "");
      this.render();
    });
    this.sortSelect?.addEventListener("change", () => {
      this.filter.sort = String(this.sortSelect.value ?? "recent");
      this.saveBrowsePrefs();
      this.render();
    });
    this.queryInput?.addEventListener("input", () => {
      this.filter.query = String(this.queryInput.value ?? "").trim();
      this.render();
    });

    window.addEventListener("seamgrim:run-prefs-changed", () => {
      this.refreshRunUiPrefs();
      this.render();
    });
    window.addEventListener("storage", (event) => {
      if (event.key !== RUN_UI_PREFS_STORAGE_KEY) return;
      this.refreshRunUiPrefs();
      this.render();
    });
    window.addEventListener("storage", (event) => {
      if (event.key !== BROWSE_UI_PREFS_STORAGE_KEY) return;
      this.loadBrowsePrefs();
      this.render();
    });

    this.refreshRunUiPrefs();
    if (this.sortSelect) {
      this.sortSelect.value = this.filter.sort;
    }
    if (this.seedScopeSelect) {
      this.seedScopeSelect.value = this.filter.seedScope;
    }
    if (this.runLaunchSelect) {
      this.runLaunchSelect.value = this.filter.runLaunch;
    }
    if (this.launchProfileSelect) {
      this.launchProfileSelect.value = this.filter.launchProfile;
    }
    this.lastBrowsePresetId = resolveBrowsePresetId(this.filter);
    this.updateFeaturedSeedQuickPresetButton();
    this.updateNumericTrackButton();
    this.updateNumericTrackResultsButton();
    this.updateNumericTrackResultSummaryCopyButton();
    this.updateNumericTrackResultTimelinePanel();
    this.updateNumericTrackResultCompareButton();
    this.updateNumericTrackResultCompareHistoryButton();
    this.updateBrowsePresetCopyButton();
    this.publishNumericTrackResultHistorySnapshot();
  }

  toggleLocalPackagePastePanel(open = false) {
    if (!this.localPackagePastePanel) return false;
    const visible = Boolean(open);
    this.localPackagePastePanel.classList.toggle("hidden", !visible);
    this.localPackagePastePanel.dataset.open = visible ? "1" : "0";
    this.localPackagePastePanel.setAttribute("aria-hidden", visible ? "false" : "true");
    this.pasteLocalPackageButton?.setAttribute("aria-expanded", visible ? "true" : "false");
    if (visible) {
      this.updateLocalPackagePastePreview();
      this.localPackagePasteText?.focus?.();
    } else {
      this.clearLocalPackagePasteErrorStatus();
      this.clearLocalPackagePastePreview();
    }
    return visible;
  }

  setLocalPackagePastePreview(state = "idle", message = "") {
    if (!this.localPackagePastePreview) return;
    const text = String(message ?? "").trim();
    this.localPackagePastePreview.dataset.state = String(state ?? "idle").trim() || "idle";
    this.localPackagePastePreview.textContent = text;
    this.localPackagePastePreview.classList.toggle("hidden", !text);
  }

  clearLocalPackagePastePreview() {
    this.setLocalPackagePastePreview("idle", "");
  }

  updateLocalPackagePastePreview() {
    const text = String(this.localPackagePasteText?.value ?? "").trim();
    if (!text) {
      this.clearLocalPackagePastePreview();
      return false;
    }
    let payload = null;
    try {
      payload = JSON.parse(text);
    } catch (_) {
      this.setLocalPackagePastePreview("pending", "JSON 형식을 확인한 뒤 배포를 열어 주세요.");
      return false;
    }
    const validation = validateStudioLocalPackagePayload(payload);
    if (!validation.valid) {
      this.setLocalPackagePastePreview("error", `배포 형식을 확인해 주세요: ${formatStudioLocalPackageError(new Error(validation.error))}`);
      return false;
    }
    const manifest = payload && typeof payload === "object" && payload.manifest && typeof payload.manifest === "object"
      ? payload.manifest
      : {};
    const title = String(manifest.title ?? payload?.title ?? "").trim() || "제목 없는 배포";
    const session = String(manifest.session_label ?? payload?.session_label ?? "").trim();
    const firstLesson = Array.isArray(payload?.lessons) ? payload.lessons[0] : null;
    const lessonTitle = String(firstLesson?.title ?? firstLesson?.lesson_id ?? "").trim();
    const lessonCountRaw = Number(manifest.lesson_count);
    const lessonCount = Number.isFinite(lessonCountRaw)
      ? lessonCountRaw
      : Array.isArray(payload?.lessons)
        ? payload.lessons.length
        : 0;
    const reportCountRaw = Number(manifest.report_count);
    const reportCount = Number.isFinite(reportCountRaw)
      ? reportCountRaw
      : Array.isArray(payload?.reports)
        ? payload.reports.length
        : 0;
    const studentMaterialSummary = Array.isArray(manifest.student_materials_summary)
      ? manifest.student_materials_summary.map((item) => String(item ?? "").trim()).filter(Boolean)
      : [];
    const studentReportSummary = studentMaterialSummary.find((item) => item.startsWith("학생 자료 "));
    const lessonLabel = lessonCount > 1
      ? lessonTitle
        ? `첫 수업 ${lessonTitle}`
        : "첫 수업 미확인"
      : lessonTitle
        ? `실행할 수업 ${lessonTitle}`
        : "실행할 수업 미확인";
    const parts = [
      `배포 확인: ${title}`,
      lessonLabel,
      ...(lessonCount > 1 ? ["배포 수업 선택 가능"] : []),
      session ? `차시 ${session}` : "차시 미지정",
      `교과 ${lessonCount}개`,
    ];
    if (studentReportSummary) {
      parts.push(studentReportSummary);
    } else if (reportCount > 0) {
      parts.push(`자료 ${reportCount}개`);
    }
    this.setLocalPackagePastePreview("ok", parts.join(" · "));
    return true;
  }

  setLocalPackagePasteStatus(state = "idle", message = "") {
    if (!this.localPackageImportStatusEl) return;
    const text = String(message ?? "").trim();
    this.localPackageImportStatusEl.dataset.state = String(state ?? "idle").trim() || "idle";
    this.localPackageImportStatusEl.textContent = text;
    this.localPackageImportStatusEl.classList.toggle("hidden", !text);
  }

  setCoursePackageExportStatus(state = "idle", message = "") {
    if (!this.coursePackageExportStatusEl) return;
    const text = String(message ?? "").trim();
    this.coursePackageExportStatusEl.dataset.state = String(state ?? "idle").trim() || "idle";
    this.coursePackageExportStatusEl.textContent = text;
    this.coursePackageExportStatusEl.classList.toggle("hidden", !text);
  }

  clearLocalPackagePasteErrorStatus() {
    if (String(this.localPackageImportStatusEl?.dataset?.state ?? "") !== "error") return;
    this.setLocalPackagePasteStatus("idle", "");
  }

  async fetchLessonDdnTextForPackage(lesson) {
    const lessonId = String(lesson?.id ?? "").trim();
    if (!lessonId) throw new Error("교과 ID가 없습니다.");
    const cached = this.detailDdnPreviewCache.get(lessonId);
    const cachedText = String(cached?.fullText ?? cached?.text ?? "").trim();
    if (cachedText) return cachedText;
    const candidates = this.resolveDetailDdnPreviewCandidates(lesson);
    for (const candidate of candidates) {
      try {
        const response = await fetch(candidate, { cache: "no-cache" });
        if (!response.ok) continue;
        const raw = await response.text();
        if (!String(raw ?? "").trim()) continue;
        this.detailDdnPreviewCache.set(lessonId, {
          text: clipDdnPreviewText(raw),
          fullText: raw,
          path: candidate,
        });
        return raw;
      } catch (_) {
        // try next candidate
      }
    }
    throw new Error(`DDN 원문을 불러오지 못했습니다: ${lessonId}`);
  }

  async buildCoursePackageExportModel(lessons = null) {
    const rows = Array.isArray(lessons) ? lessons : this.filteredLessons({ subject: "" });
    const courseLessons = rows.filter((lesson) => String(lesson?.id ?? "").trim());
    if (!courseLessons.length) {
      throw new Error("대표 교과가 없습니다.");
    }
    const packageLessons = [];
    for (const lesson of courseLessons) {
      const sourceText = await this.fetchLessonDdnTextForPackage(lesson);
      packageLessons.push({
        lesson_id: String(lesson.id ?? "").trim(),
        title: String(lesson.title ?? lesson.id ?? "").trim(),
        description: String(lesson.description ?? "").trim(),
        grade: String(lesson.grade ?? "").trim(),
        subject: String(lesson.subject ?? "").trim(),
        required_views: getLessonRequiredViews(lesson),
        goals: buildCourseGoalTexts(lesson),
        missions: buildCourseMissionTexts(lesson),
        source_text: sourceText,
      });
    }
    const title = `대표 교과 ${packageLessons.length}개 배포 묶음`;
    const guideText = [
      "셈그림 대표 교과 배포 안내",
      "",
      `교과 수: ${packageLessons.length}`,
      ...packageLessons.map((lesson, index) => `${index + 1}. ${lesson.title} (${lesson.lesson_id})`),
      "",
      "학생 안내:",
      "1. 셈그림 Studio에서 배포 열기 또는 붙여넣기 열기로 이 JSON을 엽니다.",
      "2. 배포 수업 선택에서 오늘 수업을 고릅니다.",
      "3. 받은 수업 실행을 눌러 그래프, 표, 설명을 확인합니다.",
      "4. 결과 확인 뒤 이름을 입력하고 결과 복사 또는 결과 저장으로 제출합니다.",
    ].join("\n");
    const checklistText = [
      "셈그림 대표 교과 교사용 준비 체크리스트",
      "",
      `배포 묶음: ${title}`,
      `교과 수: ${packageLessons.length}`,
      "",
      "수업 전:",
      "- 대표 묶음 JSON을 학생에게 전달합니다.",
      "- 학생에게 배포 열기 또는 붙여넣기 열기를 안내합니다.",
      "- 배포 수업 선택에서 오늘 수업을 고르게 합니다.",
      "- 학생 명단 양식에 이름을 준비합니다.",
      "",
      "수업 중:",
      "- 받은 수업 실행 후 그래프, 표, 설명을 확인하게 합니다.",
      "- 결과 확인 뒤 이름을 입력하고 결과 복사 또는 결과 저장으로 제출하게 합니다.",
      "",
      "포함 교과:",
      ...packageLessons.map((lesson, index) => `${index + 1}. ${lesson.title} (${lesson.lesson_id})`),
    ].join("\n");
    const rosterTemplateText = [
      "# 셈그림 대표 교과 학생 명단",
      "# 배포 묶음\tstudio.local.course.representative.v1",
      "학생\t차시\t수업\t수업 코드\t배포 묶음\t배포 코드",
      ...packageLessons.map((lesson) => (
        `\t\t${lesson.title ?? ""}\t${lesson.lesson_id ?? ""}\t${title}\tstudio.local.course.representative.v1`
      )),
    ].join("\n");
    const reports = [
      {
        report_id: "representative_course_student_guide",
        title: "대표 교과 학생 배포 안내문",
        text: guideText,
      },
      {
        report_id: "representative_course_teacher_preparation_checklist",
        title: "대표 교과 교사용 준비 체크리스트",
        text: checklistText,
      },
      {
        report_id: "representative_course_student_roster_template",
        title: "대표 교과 학생 명단 양식",
        path: "reports/representative_course_student_roster_template.tsv",
        mime: "text/tab-separated-values; charset=utf-8",
        text: rosterTemplateText,
      },
    ];
    const studentMaterialCount = reports.filter((report) => {
      const label = String(report?.title ?? report?.report_id ?? "");
      return label.includes("학생");
    }).length;
    const manifest = buildStudioLocalPackageManifest({
      packageId: "studio.local.course.representative.v1",
      title,
      version: "0.1.0",
      lessons: packageLessons,
      reports,
    });
    const payload = buildStudioLocalPackagePayload({ manifest, lessons: packageLessons, reports });
    const payloadText = JSON.stringify(payload, null, 2);
    const indexText = formatStudioLocalPackageIndexText(payload);
    const lessonIds = packageLessons
      .map((lesson) => String(lesson?.lesson_id ?? lesson?.id ?? "").trim())
      .filter(Boolean);
    return {
      schema: "seamgrim.course_package_export_action.v1",
      generated_by: "STUDIO_COURSE_PACKAGE_EXPORT_ACTION_V1",
      title,
      lesson_count: packageLessons.length,
      multi_lesson: lessonIds.length > 1,
      lesson_ids: lessonIds,
      report_count: reports.length,
      student_material_count: studentMaterialCount,
      package_id: manifest.package_id,
      manifest,
      payload,
      payload_text: payloadText,
      index_text: indexText,
    };
  }

  async handleCopyCoursePackage() {
    const button = this.copyCoursePackageButton;
    const originalLabel = button?.textContent ?? "대표 묶음 복사";
    if (button) {
      button.disabled = true;
      button.textContent = "묶음 준비 중...";
    }
    this.setCoursePackageExportStatus("loading", "대표 교과 DDN 원문을 모아 배포 묶음을 만드는 중입니다.");
    try {
      const model = await this.buildCoursePackageExportModel();
      const ok = await this.copyToClipboard(model.payload_text);
      try {
        window.__STUDIO_COURSE_PACKAGE_EXPORT_ACTION__ = {
          ...model,
          copied: ok,
        };
      } catch (_) {
        // ignore browser instrumentation errors
      }
      this.setCoursePackageExportStatus(
        ok ? "ok" : "error",
        ok
          ? `대표 교과 배포 묶음을 복사했습니다: 교과 ${model.lesson_count}개 · 학생 자료 ${model.student_material_count}개 · 전체 자료 ${model.report_count}개`
          : "대표 교과 배포 묶음 복사에 실패했습니다.",
      );
      showGlobalToast(ok ? "대표 교과 배포 묶음을 복사했습니다." : "대표 교과 배포 묶음 복사에 실패했습니다.", {
        kind: ok ? "success" : "error",
      });
      return ok;
    } catch (error) {
      const message = String(error?.message ?? error);
      try {
        window.__STUDIO_COURSE_PACKAGE_EXPORT_ACTION__ = {
          schema: "seamgrim.course_package_export_action.v1",
          copied: false,
          error: message,
        };
      } catch (_) {
        // ignore browser instrumentation errors
      }
      this.setCoursePackageExportStatus("error", `대표 교과 배포 묶음 생성 실패: ${message}`);
      showGlobalToast(`대표 교과 배포 묶음 생성 실패: ${message}`, { kind: "error" });
      return false;
    } finally {
      if (button) {
        button.disabled = false;
        button.textContent = originalLabel;
      }
    }
  }

  async handleDownloadCoursePackage() {
    const button = this.downloadCoursePackageButton;
    const originalLabel = button?.textContent ?? "대표 묶음 저장";
    if (button) {
      button.disabled = true;
      button.textContent = "묶음 저장 중...";
    }
    this.setCoursePackageExportStatus("loading", "대표 교과 배포 묶음 파일을 만드는 중입니다.");
    try {
      const model = await this.buildCoursePackageExportModel();
      const fileName = buildSafeJsonDownloadName(`${model.package_id || "studio.local.course.representative.v1"}.json`);
      const savedName = saveJsonTextToFile(model.payload_text, fileName);
      try {
        window.__STUDIO_COURSE_PACKAGE_DOWNLOAD_ACTION__ = {
          ...model,
          downloaded: true,
          file_name: savedName,
        };
      } catch (_) {
        // ignore browser instrumentation errors
      }
      this.setCoursePackageExportStatus(
        "ok",
        `대표 교과 배포 묶음을 저장했습니다: 교과 ${model.lesson_count}개 · 학생 자료 ${model.student_material_count}개 · 전체 자료 ${model.report_count}개 · ${savedName}`,
      );
      showGlobalToast("대표 교과 배포 묶음을 저장했습니다.", { kind: "success" });
      return true;
    } catch (error) {
      const message = String(error?.message ?? error);
      try {
        window.__STUDIO_COURSE_PACKAGE_DOWNLOAD_ACTION__ = {
          schema: "seamgrim.course_package_download_action.v1",
          downloaded: false,
          error: message,
        };
      } catch (_) {
        // ignore browser instrumentation errors
      }
      this.setCoursePackageExportStatus("error", `대표 교과 배포 묶음 저장 실패: ${message}`);
      showGlobalToast(`대표 교과 배포 묶음 저장 실패: ${message}`, { kind: "error" });
      return false;
    } finally {
      if (button) {
        button.disabled = false;
        button.textContent = originalLabel;
      }
    }
  }

  async handleOpenLocalPackagePaste() {
    const text = String(this.localPackagePasteText?.value ?? "").trim();
    if (!text) {
      this.setLocalPackagePasteStatus("error", "붙여넣을 배포 JSON이 없습니다.");
      this.localPackagePasteText?.focus?.();
      return false;
    }
    const fileLike = {
      name: "pasted-teacher-package.json",
      async text() {
        return text;
      },
    };
    const originalLabel = this.localPackagePasteOpenButton?.textContent ?? "붙여넣은 배포 열기";
    if (this.localPackagePasteOpenButton) {
      this.localPackagePasteOpenButton.disabled = true;
      this.localPackagePasteOpenButton.textContent = "배포 여는 중...";
    }
    try {
      const ok = await this.onOpenLocalPackageFile(fileLike);
      if (ok) {
        if (this.localPackagePasteText) this.localPackagePasteText.value = "";
        this.clearLocalPackagePastePreview();
        this.toggleLocalPackagePastePanel(false);
      }
      return ok;
    } finally {
      if (this.localPackagePasteOpenButton) {
        this.localPackagePasteOpenButton.disabled = false;
        this.localPackagePasteOpenButton.textContent = originalLabel;
      }
    }
  }

  removeSelectOption(selectEl, value) {
    if (!selectEl) return;
    const option = selectEl.querySelector(`option[value="${value}"]`);
    if (option) option.remove();
  }

  applyFeaturedSeedVisibilityPolicy() {
    if (this.featuredSeedEnabled) return;
    this.presetFeaturedSeedQuickRecentButton?.classList.add("hidden");
    this.removeSelectOption(this.seedScopeSelect, "featured_seed");
    this.removeSelectOption(this.seedScopeSelect, "seed_only");
    this.removeSelectOption(this.runLaunchSelect, "featured_seed_quick");
    this.removeSelectOption(this.sortSelect, "featured_seed");
    this.removeSelectOption(this.sortSelect, "featured_seed_quick_recent");
    if (this.filter.seedScope === "featured_seed") this.filter.seedScope = "";
    if (this.filter.seedScope === "seed_only") this.filter.seedScope = "";
    if (this.filter.runLaunch === "featured_seed_quick") this.filter.runLaunch = "";
    if (this.filter.sort === "featured_seed" || this.filter.sort === "featured_seed_quick_recent") {
      this.filter.sort = "recent";
    }
  }

  applySimplifiedCatalogFilters() {
    const showLegacyControls = shouldShowLegacyBrowseControls();
    if (!showLegacyControls) {
      this.filter.quality = "";
      this.filter.seedScope = "";
      this.filter.runStatus = "";
      this.filter.runLaunch = "";
      this.filter.launchProfile = "";
      this.filter.warningStatus = "";
      this.filter.sort = "teacher";
      [
        [this.qualitySelect, ""],
        [this.seedScopeSelect, ""],
        [this.runStatusSelect, ""],
        [this.runLaunchSelect, ""],
        [this.launchProfileSelect, ""],
        [this.warningStatusSelect, ""],
        [this.sortSelect, "teacher"],
      ].forEach(([node, value]) => {
        if (node) node.value = value;
      });
    }
    const hiddenTargets = [
      this.qualitySelect,
      this.seedScopeSelect,
      this.runStatusSelect,
      this.runLaunchSelect,
      this.launchProfileSelect,
      this.warningStatusSelect,
      this.sortSelect,
    ];
    hiddenTargets.forEach((node) => {
      if (!node || !node.classList?.add) return;
      node.classList.add("catalog-filter-hidden");
      node.setAttribute?.("aria-hidden", "true");
      node.tabIndex = -1;
    });
    // Temporary compatibility: keep legacy result/preset controls available for dev-surface runner coverage.
    if (!showLegacyControls) {
      [
        this.numericTrackButton,
        this.numericTrackResultsButton,
        this.copyNumericTrackResultSummaryButton,
        this.numericTrackTimelineButton,
        this.copyBrowsePresetLinkButton,
        this.root.querySelector(".browse-tab[disabled][title='AGE5']"),
      ].forEach((node) => {
        if (!node || !node.classList?.add) return;
        node.classList.add("hidden");
        node.setAttribute?.("aria-hidden", "true");
        node.tabIndex = -1;
      });
    }
  }

  loadBrowsePrefs() {
    const parsed = readStorageJson(BROWSE_UI_PREFS_STORAGE_KEY, {});
    const savedSort = String(parsed?.sort ?? this.filter.sort ?? "teacher").trim() || "teacher";
    this.filter.sort = savedSort === "recent" ? "teacher" : savedSort;
    this.filter.seedScope = String(parsed?.seedScope ?? this.filter.seedScope ?? "").trim();
    this.filter.runLaunch = normalizeRunLaunchFilter(parsed?.runLaunch ?? this.filter.runLaunch ?? "");
    this.filter.launchProfile = normalizeLaunchProfile(
      parsed?.launchProfile ?? this.filter.launchProfile ?? "",
    );
    if (this.sortSelect) {
      this.sortSelect.value = this.filter.sort;
    }
    if (this.seedScopeSelect) {
      this.seedScopeSelect.value = this.filter.seedScope;
    }
    if (this.runLaunchSelect) {
      this.runLaunchSelect.value = this.filter.runLaunch;
    }
    if (this.launchProfileSelect) {
      this.launchProfileSelect.value = this.filter.launchProfile;
    }
  }

  saveBrowsePrefs() {
    writeStorageJson(BROWSE_UI_PREFS_STORAGE_KEY, {
      sort: String(this.filter.sort ?? "recent"),
      seedScope: String(this.filter.seedScope ?? ""),
      runLaunch: String(this.filter.runLaunch ?? ""),
      launchProfile: String(this.filter.launchProfile ?? ""),
    });
    this.emitBrowsePresetChanged();
  }

  toggleNumericTrackFilter() {
    this.filter.numericTrack = !this.filter.numericTrack;
    this.updateNumericTrackButton();
    this.render();
  }

  toggleNumericTrackResultsFilter() {
    this.filter.numericTrackResults = !this.filter.numericTrackResults;
    this.updateNumericTrackResultsButton();
    this.render();
  }

  toggleNumericTrackResultTimeline() {
    this.filter.numericTrackTimeline = !this.filter.numericTrackTimeline;
    this.updateNumericTrackResultTimelinePanel();
  }

  updateNumericTrackButton() {
    const button = this.numericTrackButton;
    if (!button) return;
    const count = this.currentPool().filter((lesson) => isNumericTrackLesson(lesson)).length;
    const active = Boolean(this.filter.numericTrack);
    button.classList.toggle("active", active);
    button.dataset.active = active ? "1" : "0";
    button.dataset.count = String(count);
    button.title = count > 0
      ? `그래프·표 교과 ${count}개를 표시합니다.`
      : "현재 catalog에 그래프·표 교과가 없습니다.";
  }

  updateNumericTrackResultsButton() {
    const button = this.numericTrackResultsButton;
    if (!button) return;
    const count = this.currentPool()
      .filter((lesson) => this.getLessonNumericTrackRunResultLink(lesson?.id))
      .length;
    const active = Boolean(this.filter.numericTrackResults);
    button.classList.toggle("active", active);
    button.dataset.active = active ? "1" : "0";
    button.dataset.count = String(count);
    button.disabled = count <= 0;
    button.title = count > 0
      ? `저장된 수치 실행 결과 ${count}건만 표시합니다.`
      : "저장된 수치 실행 결과가 없습니다.";
  }

  updateNumericTrackResultSummaryCopyButton() {
    const button = this.copyNumericTrackResultSummaryButton;
    if (!button) return;
    const snapshot = this.buildNumericTrackResultHistorySnapshot();
    const count = Math.max(0, Number.isFinite(Number(snapshot?.result_count)) ? Math.trunc(Number(snapshot.result_count)) : 0);
    button.disabled = count <= 0;
    button.dataset.count = String(count);
    button.title = count > 0
      ? `저장된 수치 결과 요약 ${count}건을 복사합니다.`
      : "복사할 수치 결과 요약이 없습니다.";
  }

  updateNumericTrackResultTimelinePanel() {
    const button = this.numericTrackTimelineButton;
    const panel = this.numericTrackTimelinePanel;
    const list = this.numericTrackTimelineList;
    const snapshot = this.buildNumericTrackResultHistorySnapshot();
    const timeline = buildNumericTrackResultTimelineView(snapshot);
    const count = Math.max(0, Number.isFinite(Number(timeline?.result_count)) ? Math.trunc(Number(timeline.result_count)) : 0);
    if (button) {
      const active = Boolean(this.filter.numericTrackTimeline);
      button.classList.toggle("active", active);
      button.dataset.active = active ? "1" : "0";
      button.dataset.count = String(count);
      button.disabled = count <= 0;
      button.title = count > 0
        ? `저장된 수치 결과 timeline ${count}건을 표시합니다.`
        : "표시할 수치 결과 timeline이 없습니다.";
    }
    if (panel) {
      panel.classList.toggle("hidden", !this.filter.numericTrackTimeline || count <= 0);
      panel.dataset.count = String(count);
      panel.dataset.schema = timeline.schema;
    }
    if (list) {
      const rows = Array.isArray(timeline.rows) ? timeline.rows : [];
      list.innerHTML = rows.length > 0
        ? rows.map((row) => `
            <li class="numeric-track-timeline-row" data-lesson-id="${escapeHtml(row.lesson_id)}">
              <span class="numeric-track-timeline-time">${escapeHtml(row.recorded_at || "-")}</span>
              <span class="numeric-track-timeline-title">${escapeHtml(row.title || row.lesson_id)}</span>
              <span class="numeric-track-timeline-meta">${escapeHtml(row.preset_focus || "-")} · ${escapeHtml(row.run_kind || "-")} · ch:${escapeHtml(String(row.channels ?? 0))}</span>
              <span class="numeric-track-timeline-hash">${escapeHtml(row.state_hash_short || "")}</span>
              <button class="ghost numeric-track-timeline-reopen" type="button" data-lesson-id="${escapeHtml(row.lesson_id)}">다시 열기</button>
            </li>
          `).join("")
        : `<li class="numeric-track-timeline-row empty">저장된 수치 결과가 없습니다.</li>`;
      list.querySelectorAll(".numeric-track-timeline-reopen")?.forEach((button) => {
        button.addEventListener("click", (event) => {
          event.preventDefault();
          event.stopPropagation();
          this.reopenNumericTrackTimelineResult(String(button.dataset.lessonId ?? ""));
        });
      });
    }
    this.updateNumericTrackResultCompareButton(timeline);
    this.updateNumericTrackResultCompareHistoryButton(timeline);
  }

  updateNumericTrackResultCompareButton(timelineView = null) {
    const button = this.numericTrackCompareButton;
    const timeline = timelineView || buildNumericTrackResultTimelineView(this.buildNumericTrackResultHistorySnapshot());
    const count = Math.max(0, Number.isFinite(Number(timeline?.result_count)) ? Math.trunc(Number(timeline.result_count)) : 0);
    if (button) {
      button.disabled = count < 2;
      button.dataset.count = String(count);
      button.title = count >= 2
        ? "최근 수치 결과 2건의 metadata를 비교합니다."
        : "비교하려면 저장된 수치 결과가 2건 이상 필요합니다.";
    }
    if (count < 2 && this.numericTrackComparePanel) {
      this.numericTrackComparePanel.classList.add("hidden");
      this.numericTrackComparePanel.textContent = "";
    }
  }

  updateNumericTrackResultCompareHistoryButton(timelineView = null) {
    const button = this.numericTrackCompareHistoryButton;
    const timeline = timelineView || buildNumericTrackResultTimelineView(this.buildNumericTrackResultHistorySnapshot());
    const count = Math.max(0, Number.isFinite(Number(timeline?.result_count)) ? Math.trunc(Number(timeline.result_count)) : 0);
    if (button) {
      button.disabled = count < 2;
      button.dataset.count = String(count);
      button.title = count >= 2
        ? `저장된 수치 결과의 인접 비교 ${count - 1}건을 표시합니다.`
        : "비교 이력을 만들려면 저장된 수치 결과가 2건 이상 필요합니다.";
    }
    if (count < 2 && this.numericTrackCompareHistoryPanel) {
      this.numericTrackCompareHistoryPanel.classList.add("hidden");
      this.numericTrackCompareHistoryPanel.textContent = "";
    }
  }

  publishNumericTrackResultCompare(compare = null) {
    try {
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE__ = compare;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_TEXT__ = compare
        ? formatNumericTrackResultCompareText(compare)
        : "";
      const compareExport = buildNumericTrackResultCompareExport(compare);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_EXPORT__ = compareExport;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_EXPORT_TEXT__ = compareExport
        ? formatNumericTrackResultCompareExportText(compareExport)
        : "";
    } catch (_) {
      // ignore browser instrumentation errors
    }
  }

  publishNumericTrackResultCompareHistory(history = null) {
    try {
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY__ = history;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_TEXT__ = history
        ? formatNumericTrackResultCompareHistoryText(history)
        : "";
      const historyExport = buildNumericTrackResultCompareHistoryExport(history);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_EXPORT__ = historyExport;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_EXPORT_TEXT__ = historyExport
        ? formatNumericTrackResultCompareHistoryExportText(historyExport)
        : "";
      const historyReport = buildNumericTrackResultCompareHistoryReport(history);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT__ = historyReport;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TEXT__ = historyReport
        ? formatNumericTrackResultCompareHistoryReportText(historyReport)
        : "";
      const historyReportExport = buildNumericTrackResultCompareHistoryReportExport(historyReport);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_EXPORT__ = historyReportExport;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_EXPORT_TEXT__ = historyReportExport
        ? formatNumericTrackResultCompareHistoryReportExportText(historyReportExport)
        : "";
      const historyReportTable = buildNumericTrackResultCompareHistoryReportTable(historyReport);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE__ = historyReportTable;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_TEXT__ = historyReportTable
        ? formatNumericTrackResultCompareHistoryReportTableText(historyReportTable)
        : "";
      const historyReportTableExport = buildNumericTrackResultCompareHistoryReportTableExport(historyReportTable);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT__ = historyReportTableExport;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT_TEXT__ = historyReportTableExport
        ? formatNumericTrackResultCompareHistoryReportTableExportText(historyReportTableExport)
        : "";
      const historyReportTableSummary = buildNumericTrackResultCompareHistoryReportTableSummary(historyReportTable);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY__ = historyReportTableSummary;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_TEXT__ = historyReportTableSummary
        ? formatNumericTrackResultCompareHistoryReportTableSummaryText(historyReportTableSummary)
        : "";
      const historyReportTableSummaryExport = buildNumericTrackResultCompareHistoryReportTableSummaryExport(historyReportTableSummary);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_EXPORT__ = historyReportTableSummaryExport;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_EXPORT_TEXT__ = historyReportTableSummaryExport
        ? formatNumericTrackResultCompareHistoryReportTableSummaryExportText(historyReportTableSummaryExport)
        : "";
      const historyReportTableStatus = buildNumericTrackResultCompareHistoryReportTableStatus(historyReportTableSummary);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS__ = historyReportTableStatus;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_TEXT__ = historyReportTableStatus
        ? formatNumericTrackResultCompareHistoryReportTableStatusText(historyReportTableStatus)
        : "";
      const historyReportTableStatusExport = buildNumericTrackResultCompareHistoryReportTableStatusExport(historyReportTableStatus);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_EXPORT__ = historyReportTableStatusExport;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_EXPORT_TEXT__ = historyReportTableStatusExport
        ? formatNumericTrackResultCompareHistoryReportTableStatusExportText(historyReportTableStatusExport)
        : "";
      const historyReportTableStatusBadge = buildNumericTrackResultCompareHistoryReportTableStatusBadge(historyReportTableStatus);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE__ = historyReportTableStatusBadge;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_TEXT__ = historyReportTableStatusBadge
        ? formatNumericTrackResultCompareHistoryReportTableStatusBadgeText(historyReportTableStatusBadge)
        : "";
      const historyReportTableStatusBadgeA11y = buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11y(historyReportTableStatusBadge);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y__ = historyReportTableStatusBadgeA11y;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_TEXT__ = historyReportTableStatusBadgeA11y
        ? formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yText(historyReportTableStatusBadgeA11y)
        : "";
      const historyReportTableStatusBadgeA11yExport = buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yExport(historyReportTableStatusBadgeA11y);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_EXPORT__ = historyReportTableStatusBadgeA11yExport;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_EXPORT_TEXT__ = historyReportTableStatusBadgeA11yExport
        ? formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yExportText(historyReportTableStatusBadgeA11yExport)
        : "";
      const historyReportTableStatusBadgeA11yStatus = buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatus(historyReportTableStatusBadgeA11yExport);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS__ = historyReportTableStatusBadgeA11yStatus;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_TEXT__ = historyReportTableStatusBadgeA11yStatus
        ? formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusText(historyReportTableStatusBadgeA11yStatus)
        : "";
      const historyReportTableStatusBadgeA11yStatusExport =
        buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExport(historyReportTableStatusBadgeA11yStatus);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT__ =
        historyReportTableStatusBadgeA11yStatusExport;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_TEXT__ =
        historyReportTableStatusBadgeA11yStatusExport
          ? formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExportText(historyReportTableStatusBadgeA11yStatusExport)
          : "";
      const historyReportTableStatusBadgeA11yStatusExportSummary =
        buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExportSummary(historyReportTableStatusBadgeA11yStatusExport);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY__ =
        historyReportTableStatusBadgeA11yStatusExportSummary;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY_TEXT__ =
        historyReportTableStatusBadgeA11yStatusExportSummary
          ? formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExportSummaryText(historyReportTableStatusBadgeA11yStatusExportSummary)
          : "";
      const numericReportWorkflowConsolidation = buildNumericReportWorkflowConsolidation(history);
      window.__SEAMGRIM_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION__ = numericReportWorkflowConsolidation;
      window.__SEAMGRIM_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_TEXT__ = numericReportWorkflowConsolidation
        ? formatNumericReportWorkflowConsolidationText(numericReportWorkflowConsolidation)
        : "";
      const historyReportTableStatusBadgeExport = buildNumericTrackResultCompareHistoryReportTableStatusBadgeExport(historyReportTableStatusBadge);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_EXPORT__ = historyReportTableStatusBadgeExport;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_EXPORT_TEXT__ = historyReportTableStatusBadgeExport
        ? formatNumericTrackResultCompareHistoryReportTableStatusBadgeExportText(historyReportTableStatusBadgeExport)
        : "";
    } catch (_) {
      // ignore browser instrumentation errors
    }
  }

  async handleCopyNumericTrackResultCompareHistoryExport(history = null) {
    const historyExport = buildNumericTrackResultCompareHistoryExport(history);
    const text = historyExport ? formatNumericTrackResultCompareHistoryExportText(historyExport) : "";
    if (!text) return;
    this.publishNumericTrackResultCompareHistory(history);
    const ok = await this.copyToClipboard(text);
    showGlobalToast(ok ? "수치 결과 비교 이력을 복사했습니다." : "수치 결과 비교 이력 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
  }

  async handleCopyNumericTrackResultCompareHistoryReportExport(report = null) {
    const reportExport = buildNumericTrackResultCompareHistoryReportExport(report);
    const text = reportExport ? formatNumericTrackResultCompareHistoryReportExportText(reportExport) : "";
    if (!text) return;
    const ok = await this.copyToClipboard(text);
    showGlobalToast(ok ? "수치 결과 비교 이력 보고서를 복사했습니다." : "수치 결과 비교 이력 보고서 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
  }

  async handleCopyNumericTrackResultCompareHistoryReportTableExport(table = null) {
    const tableExport = buildNumericTrackResultCompareHistoryReportTableExport(table);
    const text = tableExport ? formatNumericTrackResultCompareHistoryReportTableExportText(tableExport) : "";
    if (!text) return;
    const ok = await this.copyToClipboard(text);
    showGlobalToast(ok ? "수치 결과 비교 이력 보고서 표를 복사했습니다." : "수치 결과 비교 이력 보고서 표 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
  }

  async handleCopyNumericTrackResultCompareHistoryReportTableSummaryExport(summary = null) {
    const summaryExport = buildNumericTrackResultCompareHistoryReportTableSummaryExport(summary);
    const text = summaryExport ? formatNumericTrackResultCompareHistoryReportTableSummaryExportText(summaryExport) : "";
    if (!text) return;
    const ok = await this.copyToClipboard(text);
    showGlobalToast(ok ? "수치 결과 비교 이력 보고서 표 요약을 복사했습니다." : "수치 결과 비교 이력 보고서 표 요약 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
  }

  async handleCopyNumericTrackResultCompareHistoryReportTableStatusExport(status = null) {
    const statusExport = buildNumericTrackResultCompareHistoryReportTableStatusExport(status);
    const text = statusExport ? formatNumericTrackResultCompareHistoryReportTableStatusExportText(statusExport) : "";
    if (!text) return;
    const ok = await this.copyToClipboard(text);
    showGlobalToast(ok ? "수치 결과 비교 이력 보고서 표 상태를 복사했습니다." : "수치 결과 비교 이력 보고서 표 상태 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
  }

  async handleCopyNumericTrackResultCompareHistoryReportTableStatusBadgeExport(badge = null) {
    const badgeExport = buildNumericTrackResultCompareHistoryReportTableStatusBadgeExport(badge);
    const text = badgeExport ? formatNumericTrackResultCompareHistoryReportTableStatusBadgeExportText(badgeExport) : "";
    if (!text) return;
    const ok = await this.copyToClipboard(text);
    showGlobalToast(ok ? "Numeric status badge export copied." : "Numeric status badge export copy failed.", {
      kind: ok ? "success" : "error",
    });
  }

  async handleCopyNumericTrackResultCompareHistoryReportTableStatusBadgeA11yExport(a11y = null) {
    const a11yExport = buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yExport(a11y);
    const text = a11yExport ? formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yExportText(a11yExport) : "";
    if (!text) return;
    const ok = await this.copyToClipboard(text);
    showGlobalToast(ok ? "Numeric status badge a11y export copied." : "Numeric status badge a11y export copy failed.", {
      kind: ok ? "success" : "error",
    });
  }

  async handleCopyNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExport(status = null) {
    const statusExport = buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExport(status);
    const text = statusExport ? formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExportText(statusExport) : "";
    if (!text) return;
    const ok = await this.copyToClipboard(text);
    showGlobalToast(ok ? "Numeric status badge a11y status export copied." : "Numeric status badge a11y status export copy failed.", {
      kind: ok ? "success" : "error",
    });
  }

  async handleCopyNumericReportWorkflowConsolidation(history = null) {
    const workflow = buildNumericReportWorkflowConsolidation(history);
    const text = workflow ? formatNumericReportWorkflowConsolidationText(workflow) : "";
    if (!text) return;
    const ok = await this.copyToClipboard(text);
    showGlobalToast(ok ? "수치 보고 workflow를 복사했습니다." : "수치 보고 workflow 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
  }

  async handleCopyNumericTrackResultCompareExport(compare = null) {
    const compareExport = buildNumericTrackResultCompareExport(compare);
    const text = compareExport ? formatNumericTrackResultCompareExportText(compareExport) : "";
    if (!text) return;
    this.publishNumericTrackResultCompare(compare);
    const ok = await this.copyToClipboard(text);
    showGlobalToast(ok ? "수치 결과 비교를 복사했습니다." : "수치 결과 비교 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
  }

  showNumericTrackResultCompare() {
    const timeline = buildNumericTrackResultTimelineView(this.buildNumericTrackResultHistorySnapshot());
    const compare = buildNumericTrackResultCompare(timeline);
    this.publishNumericTrackResultCompare(compare);
    if (!compare || !this.numericTrackComparePanel) return;
    const latest = compare.latest || {};
    const previous = compare.previous || {};
    this.numericTrackComparePanel.classList.remove("hidden");
    this.numericTrackComparePanel.dataset.schema = compare.schema;
    this.numericTrackComparePanel.dataset.compareKind = compare.compare_kind;
    this.numericTrackComparePanel.innerHTML = `
      <div class="numeric-track-compare-head">
        <strong>최근 2개 수치 결과 비교</strong>
        <span>${escapeHtml(compare.compare_claim)} · replay:${compare.replay_claim === true ? "true" : "false"}</span>
        <button id="btn-copy-numeric-track-result-compare-export" class="ghost" type="button">비교 복사</button>
      </div>
      <div class="numeric-track-compare-grid">
        <div class="numeric-track-compare-card">
          <span class="numeric-track-compare-label">최신</span>
          <strong>${escapeHtml(latest.title || latest.lesson_id || "-")}</strong>
          <span>${escapeHtml(latest.preset_focus || "-")} · ${escapeHtml(latest.run_kind || "-")} · ch:${escapeHtml(String(latest.channels ?? 0))}</span>
          <code>${escapeHtml(latest.state_hash_short || "")}</code>
        </div>
        <div class="numeric-track-compare-card">
          <span class="numeric-track-compare-label">이전</span>
          <strong>${escapeHtml(previous.title || previous.lesson_id || "-")}</strong>
          <span>${escapeHtml(previous.preset_focus || "-")} · ${escapeHtml(previous.run_kind || "-")} · ch:${escapeHtml(String(previous.channels ?? 0))}</span>
          <code>${escapeHtml(previous.state_hash_short || "")}</code>
        </div>
      </div>
      <div class="numeric-track-compare-summary">
        <span>같은 교과: ${compare.same_lesson ? "예" : "아니오"}</span>
        <span>같은 초점: ${compare.same_focus ? "예" : "아니오"}</span>
        <span>실행종류 변화: ${compare.same_run_kind ? "없음" : "있음"}</span>
        <span>채널 차이: ${escapeHtml(String(compare.channel_delta ?? 0))}</span>
        <span>hash 변화: ${compare.state_hash_changed ? "있음" : "없음"}</span>
      </div>
    `;
    this.numericTrackComparePanel
      .querySelector("#btn-copy-numeric-track-result-compare-export")
      ?.addEventListener("click", () => {
        void this.handleCopyNumericTrackResultCompareExport(compare);
      });
  }

  showNumericTrackResultCompareHistory() {
    const timeline = buildNumericTrackResultTimelineView(this.buildNumericTrackResultHistorySnapshot());
    const history = buildNumericTrackResultCompareHistory(timeline);
    this.publishNumericTrackResultCompareHistory(history);
    if (!history || !this.numericTrackCompareHistoryPanel) return;
    const pairs = Array.isArray(history.pairs) ? history.pairs : [];
    const report = buildNumericTrackResultCompareHistoryReport(history);
    const reportTable = buildNumericTrackResultCompareHistoryReportTable(report);
    const reportTableSummary = buildNumericTrackResultCompareHistoryReportTableSummary(reportTable);
    const reportTableStatus = buildNumericTrackResultCompareHistoryReportTableStatus(reportTableSummary);
    const reportTableStatusBadge = buildNumericTrackResultCompareHistoryReportTableStatusBadge(reportTableStatus);
    const reportTableStatusBadgeA11y = buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11y(reportTableStatusBadge);
    const reportTableStatusBadgeA11yExport =
      buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yExport(reportTableStatusBadgeA11y);
    const reportTableStatusBadgeA11yStatus =
      buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatus(reportTableStatusBadgeA11yExport);
    const reportTableStatusBadgeA11yStatusExport =
      buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExport(reportTableStatusBadgeA11yStatus);
    const reportTableStatusBadgeA11yStatusExportSummary =
      buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExportSummary(reportTableStatusBadgeA11yStatusExport);
    const workflowConsolidation = buildNumericReportWorkflowConsolidation(history);
    const reportTableRows = Array.isArray(reportTable?.rows) ? reportTable.rows : [];
    this.numericTrackCompareHistoryPanel.classList.remove("hidden");
    this.numericTrackCompareHistoryPanel.dataset.schema = history.schema;
    this.numericTrackCompareHistoryPanel.dataset.pairCount = String(pairs.length);
    this.numericTrackCompareHistoryPanel.dataset.reportSchema = report?.schema ?? "";
    this.numericTrackCompareHistoryPanel.dataset.reportTableSchema = reportTable?.schema ?? "";
    this.numericTrackCompareHistoryPanel.setAttribute("data-workflow-schema", workflowConsolidation?.schema ?? "");
    this.numericTrackCompareHistoryPanel.innerHTML = `
      <div class="numeric-track-compare-history-head">
        <strong>수치 결과 비교 이력</strong>
        <span>${escapeHtml(history.compare_claim)} · replay:${history.replay_claim === true ? "true" : "false"} · pairs:${escapeHtml(String(pairs.length))}</span>
        <button id="btn-copy-numeric-track-result-compare-history-export" class="ghost" type="button">이력 복사</button>
      </div>
      <div class="numeric-report-workflow-consolidation" data-status="${escapeHtml(workflowConsolidation?.status || "")}" data-tone="${escapeHtml(workflowConsolidation?.tone || "")}">
        <strong>보고 workflow</strong>
        <span>좌표 ${escapeHtml(workflowConsolidation?.primary_coordinate || "마-3")}</span>
        <span>단계 ${escapeHtml(String(workflowConsolidation?.ready_stage_count ?? 0))}/${escapeHtml(String(workflowConsolidation?.stage_count ?? 0))}</span>
        <span>상태 ${escapeHtml(workflowConsolidation?.status || "workflow_incomplete")}</span>
        <span>rows ${escapeHtml(String(workflowConsolidation?.row_count ?? 0))}</span>
        <button id="btn-copy-numeric-report-workflow-consolidation" class="ghost" type="button">workflow 복사</button>
      </div>
      <div class="numeric-track-compare-history-report-summary">
        <span>hash 변화 ${escapeHtml(String(report?.state_hash_changed_count ?? 0))}</span>
        <span>실행종류 변화 ${escapeHtml(String(report?.run_kind_changed_count ?? 0))}</span>
        <span>채널 변화합 ${escapeHtml(String(report?.channel_delta_total ?? 0))}</span>
        <span>채널 변화절대합 ${escapeHtml(String(report?.channel_delta_abs_total ?? 0))}</span>
        <button id="btn-copy-numeric-track-result-compare-history-report-export" class="ghost" type="button">보고서 복사</button>
      </div>
      <div class="numeric-track-compare-history-report-table" data-row-count="${escapeHtml(String(reportTableRows.length))}">
        <div class="numeric-track-compare-history-report-table-head">
          <strong>보고서 표</strong>
          <span>rows:${escapeHtml(String(reportTableRows.length))} · columns:${escapeHtml(String(reportTable?.column_count ?? 0))}</span>
          <button id="btn-copy-numeric-track-result-compare-history-report-table-export" class="ghost" type="button">표 복사</button>
        </div>
        <div class="numeric-track-compare-history-report-table-summary">
          <span>교과 ${escapeHtml(String(reportTableSummary?.lesson_count ?? 0))}</span>
          <span>hash 변화 ${escapeHtml(String(reportTableSummary?.state_hash_changed_count ?? 0))}</span>
          <span>실행종류 변화 ${escapeHtml(String(reportTableSummary?.run_kind_changed_count ?? 0))}</span>
          <span>채널 변화절대합 ${escapeHtml(String(reportTableSummary?.channel_delta_abs_total ?? 0))}</span>
          <button id="btn-copy-numeric-track-result-compare-history-report-table-summary-export" class="ghost" type="button">요약 복사</button>
        </div>
        <div class="numeric-track-compare-history-report-table-status" data-status="${escapeHtml(reportTableStatus?.status || "")}">
          <span class="numeric-track-compare-history-report-table-status-badge" data-tone="${escapeHtml(reportTableStatusBadge?.tone || "")}" data-a11y-schema="${escapeHtml(reportTableStatusBadgeA11y?.schema || "")}" role="${escapeHtml(reportTableStatusBadgeA11y?.role || "status")}" aria-label="${escapeHtml(reportTableStatusBadgeA11y?.aria_label || "")}" title="${escapeHtml(reportTableStatusBadgeA11y?.title || "")}">${escapeHtml(reportTableStatusBadge?.label || reportTableStatus?.status || "변화없음")}</span>
          <span>상태 ${escapeHtml(reportTableStatus?.status || "변화없음")}</span>
          <span>이유 ${escapeHtml(String(reportTableStatus?.status_reasons?.length ?? 0))}</span>
          <span>claim ${escapeHtml(reportTableStatus?.status_claim || "")}</span>
          <button id="btn-copy-numeric-track-result-compare-history-report-table-status-export" class="ghost" type="button">상태 복사</button>
          <button id="btn-copy-numeric-track-result-compare-history-report-table-status-badge-export" class="ghost" type="button" aria-label="copy numeric status badge metadata export" title="copy numeric status badge metadata export">badge copy</button>
          <button id="btn-copy-numeric-track-result-compare-history-report-table-status-badge-a11y-export" class="ghost" type="button" aria-label="copy numeric status badge accessibility metadata export" title="copy numeric status badge accessibility metadata export">a11y copy</button>
          <span class="numeric-track-compare-history-report-table-status-badge-a11y-status" data-tone="${escapeHtml(reportTableStatusBadgeA11yStatus?.tone || "")}" data-status="${escapeHtml(reportTableStatusBadgeA11yStatus?.status || "")}">a11y ${escapeHtml(reportTableStatusBadgeA11yStatus?.status || "a11y_incomplete")}</span>
          <button id="btn-copy-numeric-track-result-compare-history-report-table-status-badge-a11y-status-export" class="ghost" type="button" aria-label="copy numeric status badge accessibility status metadata export" title="copy numeric status badge accessibility status metadata export">a11y status copy</button>
          <span class="numeric-track-compare-history-report-table-status-badge-a11y-status-export-summary" data-tone="${escapeHtml(reportTableStatusBadgeA11yStatusExportSummary?.tone || "")}" data-status="${escapeHtml(reportTableStatusBadgeA11yStatusExportSummary?.status || "")}">summary ${escapeHtml(reportTableStatusBadgeA11yStatusExportSummary?.status || "summary_incomplete")}</span>
        </div>
        <table>
          <thead>
            <tr>
              <th>idx</th>
              <th>latest</th>
              <th>previous</th>
              <th>run</th>
              <th>channel</th>
              <th>hash</th>
            </tr>
          </thead>
          <tbody>
            ${reportTableRows.length > 0
              ? reportTableRows.map((row) => `
                  <tr data-index="${escapeHtml(String(row.index ?? 0))}">
                    <td>${escapeHtml(String(row.index ?? 0))}</td>
                    <td>${escapeHtml(row.latest_lesson_id || "-")}</td>
                    <td>${escapeHtml(row.previous_lesson_id || "-")}</td>
                    <td>${row.same_run_kind ? "같음" : "다름"}</td>
                    <td>${escapeHtml(String(row.channel_delta ?? 0))}</td>
                    <td>${row.state_hash_changed ? "변화" : "동일"}</td>
                  </tr>
                `).join("")
              : `<tr><td colspan="6">표로 표시할 비교 행이 없습니다.</td></tr>`}
          </tbody>
        </table>
      </div>
      <ol class="numeric-track-compare-history-list">
        ${pairs.length > 0
          ? pairs.map((pair) => `
              <li class="numeric-track-compare-history-row" data-index="${escapeHtml(String(pair.index ?? 0))}">
                <div class="numeric-track-compare-history-meta">
                  <strong>${escapeHtml(pair.latest_lesson_id || "-")} ↔ ${escapeHtml(pair.previous_lesson_id || "-")}</strong>
                  <span>${escapeHtml(pair.latest_recorded_at || "-")} / ${escapeHtml(pair.previous_recorded_at || "-")}</span>
                </div>
                <div class="numeric-track-compare-history-summary">
                  <span>교과:${pair.same_lesson ? "같음" : "다름"}</span>
                  <span>초점:${pair.same_focus ? "같음" : "다름"}</span>
                  <span>실행:${pair.same_run_kind ? "같음" : "다름"}</span>
                  <span>채널차:${escapeHtml(String(pair.channel_delta ?? 0))}</span>
                  <span>hash:${pair.state_hash_changed ? "변화" : "동일"}</span>
                </div>
              </li>
            `).join("")
          : `<li class="numeric-track-compare-history-row empty">비교할 인접 결과가 없습니다.</li>`}
      </ol>
    `;
    this.numericTrackCompareHistoryPanel
      .querySelector("#btn-copy-numeric-track-result-compare-history-export")
      ?.addEventListener("click", () => {
        void this.handleCopyNumericTrackResultCompareHistoryExport(history);
      });
    this.numericTrackCompareHistoryPanel
      .querySelector("#btn-copy-numeric-report-workflow-consolidation")
      ?.addEventListener("click", () => {
        void this.handleCopyNumericReportWorkflowConsolidation(history);
      });
    this.numericTrackCompareHistoryPanel
      .querySelector("#btn-copy-numeric-track-result-compare-history-report-export")
      ?.addEventListener("click", () => {
        void this.handleCopyNumericTrackResultCompareHistoryReportExport(report);
      });
    this.numericTrackCompareHistoryPanel
      .querySelector("#btn-copy-numeric-track-result-compare-history-report-table-export")
      ?.addEventListener("click", () => {
        void this.handleCopyNumericTrackResultCompareHistoryReportTableExport(reportTable);
      });
    this.numericTrackCompareHistoryPanel
      .querySelector("#btn-copy-numeric-track-result-compare-history-report-table-summary-export")
      ?.addEventListener("click", () => {
        void this.handleCopyNumericTrackResultCompareHistoryReportTableSummaryExport(reportTableSummary);
      });
    this.numericTrackCompareHistoryPanel
      .querySelector("#btn-copy-numeric-track-result-compare-history-report-table-status-export")
      ?.addEventListener("click", () => {
        void this.handleCopyNumericTrackResultCompareHistoryReportTableStatusExport(reportTableStatus);
      });
    this.numericTrackCompareHistoryPanel
      .querySelector("#btn-copy-numeric-track-result-compare-history-report-table-status-badge-export")
      ?.addEventListener("click", () => {
        void this.handleCopyNumericTrackResultCompareHistoryReportTableStatusBadgeExport(reportTableStatusBadge);
      });
    this.numericTrackCompareHistoryPanel
      .querySelector("#btn-copy-numeric-track-result-compare-history-report-table-status-badge-a11y-export")
      ?.addEventListener("click", () => {
        void this.handleCopyNumericTrackResultCompareHistoryReportTableStatusBadgeA11yExport(reportTableStatusBadgeA11y);
      });
    this.numericTrackCompareHistoryPanel
      .querySelector("#btn-copy-numeric-track-result-compare-history-report-table-status-badge-a11y-status-export")
      ?.addEventListener("click", () => {
        void this.handleCopyNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExport(reportTableStatusBadgeA11yStatus);
      });
  }

  findLessonById(lessonId) {
    const targetId = String(lessonId ?? "").trim();
    if (!targetId) return null;
    const pools = [this.currentPool(), this.lessons, this.searchResults, this.sampleResults];
    for (const pool of pools) {
      const found = Array.isArray(pool)
        ? pool.find((lesson) => String(lesson?.id ?? "").trim() === targetId)
        : null;
      if (found) return found;
    }
    return null;
  }

  publishNumericTrackReopenTarget(target = null) {
    try {
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_REOPEN_TARGET__ = target;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_REOPEN_TARGET_TEXT__ = target
        ? formatNumericTrackResultReopenTargetText(target)
        : "";
    } catch (_) {
      // ignore browser instrumentation errors
    }
  }

  reopenNumericTrackTimelineResult(lessonId) {
    const timeline = buildNumericTrackResultTimelineView(this.buildNumericTrackResultHistorySnapshot());
    const row = Array.isArray(timeline.rows)
      ? timeline.rows.find((item) => String(item?.lesson_id ?? "") === String(lessonId ?? ""))
      : null;
    const target = buildNumericTrackResultReopenTarget(row);
    if (!target) return;
    const lesson = this.findLessonById(target.lesson_id);
    if (!lesson) return;
    this.showLessonDetail(lesson);
    if (this.detailPanelEl) {
      setElementDatasetValue(this.detailPanelEl, "numericTrackReopen", "1");
      setElementDatasetValue(this.detailPanelEl, "reopenLessonId", target.lesson_id);
    }
    this.publishNumericTrackReopenTarget(target);
  }

  refreshRunUiPrefs() {
    try {
      const raw = window?.localStorage?.getItem(RUN_UI_PREFS_STORAGE_KEY);
      const parsed = raw ? JSON.parse(raw) : {};
      this.runUiPrefs = {
        lessons: parsed?.lessons && typeof parsed.lessons === "object" ? parsed.lessons : {},
      };
    } catch (_) {
      this.runUiPrefs = { lessons: {} };
    }
    this.publishNumericTrackResultHistorySnapshot();
  }

  buildNumericTrackResultHistorySnapshot() {
    return buildNumericTrackResultHistorySnapshot({
      lessons: this.currentPool(),
      runPrefs: this.runUiPrefs,
    });
  }

  publishNumericTrackResultHistorySnapshot() {
    try {
      const snapshot = this.buildNumericTrackResultHistorySnapshot();
      const summary = buildNumericTrackResultSummaryExport(snapshot);
      const timeline = buildNumericTrackResultTimelineView(snapshot);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_HISTORY_FILTER__ = snapshot;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_HISTORY_FILTER_TEXT__ =
        formatNumericTrackResultHistoryText(snapshot);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_SUMMARY_EXPORT__ = summary;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_SUMMARY_EXPORT_TEXT__ =
        formatNumericTrackResultSummaryExportText(summary);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_TIMELINE_VIEW__ = timeline;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_TIMELINE_VIEW_TEXT__ =
        formatNumericTrackResultTimelineViewText(timeline);
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_EXPORT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_EXPORT__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_EXPORT_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_EXPORT_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_EXPORT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_EXPORT__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_EXPORT_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_EXPORT_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_EXPORT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_EXPORT__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_EXPORT_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_EXPORT_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_EXPORT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_EXPORT__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_EXPORT_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_EXPORT_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_EXPORT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_EXPORT__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_EXPORT_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_EXPORT_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_EXPORT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_EXPORT__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_EXPORT_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_EXPORT_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY_TEXT__ ?? "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_EXPORT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_EXPORT__ ?? null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_EXPORT_TEXT__ =
        window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_EXPORT_TEXT__ ?? "";
    } catch (_) {
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_HISTORY_FILTER__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_HISTORY_FILTER_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_SUMMARY_EXPORT__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_SUMMARY_EXPORT_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_TIMELINE_VIEW__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_TIMELINE_VIEW_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_EXPORT__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_EXPORT_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_EXPORT__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_EXPORT_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_EXPORT__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_EXPORT_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_EXPORT__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_EXPORT_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_EXPORT__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_EXPORT_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_EXPORT__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_EXPORT_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_EXPORT__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_EXPORT_TEXT__ = "";
    }
  }

  buildCardStateHint(lessonId) {
    const pref = this.runUiPrefs?.lessons?.[String(lessonId ?? "").trim()];
    const x = String(pref?.selectedXKey ?? "").trim();
    const y = String(pref?.selectedYKey ?? "").trim();
    if (x || y) {
      return `보기 설정 · 가로:${x || "-"} 세로:${y || "-"}`;
    }
    return "보기 설정 · 기본";
  }

  buildCardRunHint(lessonId) {
    const pref = this.runUiPrefs?.lessons?.[String(lessonId ?? "").trim()];
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

  getLessonLastRunKind(lessonId) {
    const pref = this.runUiPrefs?.lessons?.[String(lessonId ?? "").trim()];
    if (!pref || typeof pref !== "object") return "none";
    const kind = String(pref.lastRunKind ?? "").trim();
    if (!kind) return "none";
    return kind;
  }

  getLessonLastRunAtMs(lessonId) {
    const pref = this.runUiPrefs?.lessons?.[String(lessonId ?? "").trim()];
    const ms = Date.parse(String(pref?.lastRunAt ?? ""));
    return Number.isFinite(ms) ? ms : 0;
  }

  getLessonLastRunHash(lessonId) {
    const pref = this.runUiPrefs?.lessons?.[String(lessonId ?? "").trim()];
    const hash = String(pref?.lastRunHash ?? "").trim();
    return hash && hash !== "-" ? hash : "";
  }

  getLessonNumericTrackRunResultLink(lessonId) {
    const pref = this.runUiPrefs?.lessons?.[String(lessonId ?? "").trim()];
    return normalizeNumericTrackRunResultLink(pref?.numericTrackRunResultLink);
  }

  buildNumericTrackResultHint(lessonId) {
    const link = this.getLessonNumericTrackRunResultLink(lessonId);
    if (!link) return "";
    const shortHash = link.state_hash ? link.state_hash.slice(0, 12) : "";
    const focus = link.preset_focus || "수치";
    const hashText = shortHash ? ` · 기록ID:${shortHash}` : "";
    return `결과 기록 · ${focus}${hashText}`;
  }

  getLessonLastLaunchKind(lessonId) {
    const pref = this.runUiPrefs?.lessons?.[String(lessonId ?? "").trim()];
    return normalizeLaunchKind(pref?.lastLaunchKind);
  }

  buildRunBadge(lessonId) {
    const kind = this.getLessonLastRunKind(lessonId);
    if (kind === "error") {
      return { label: "최근실패", cls: "run-badge-error" };
    }
    if (kind === "none") {
      return { label: "시작 전", cls: "run-badge-none" };
    }
    if (kind === "empty") {
      return { label: "출력없음", cls: "run-badge-empty" };
    }
    return { label: "최근성공", cls: "run-badge-success" };
  }

  buildLaunchBadge(lessonId) {
    const launchKind = this.getLessonLastLaunchKind(lessonId);
    if (launchKind === "featured_seed_quick") {
      return { label: "Alt+6 실행", cls: "launch-badge-featured-seed-quick" };
    }
    return null;
  }

  getFeaturedSeedRank(lesson) {
    if (isFeaturedSeedLesson(lesson)) return 0;
    if (normalizeSource(lesson?.source) === "seed") return 1;
    return 2;
  }

  getTeacherCatalogRank(lesson) {
    const id = String(lesson?.id ?? "").trim();
    if (Object.prototype.hasOwnProperty.call(TEACHER_CATALOG_LESSON_RANK, id)) {
      return TEACHER_CATALOG_LESSON_RANK[id];
    }
    if (normalizeSource(lesson?.source) === "seed") return 100;
    const subject = normalizeSubject(lesson?.subject);
    const subjectRank = Object.prototype.hasOwnProperty.call(TEACHER_CATALOG_SUBJECT_RANK, subject)
      ? TEACHER_CATALOG_SUBJECT_RANK[subject]
      : 50;
    const quality = normalizeQuality(lesson?.quality);
    const qualityOffset = quality === "recommended" ? 0 : quality === "reviewed" ? 1 : 2;
    return subjectRank * 10 + qualityOffset;
  }

  buildFeaturedSeedBadge(lesson) {
    if (!isFeaturedSeedLesson(lesson)) return null;
    return { label: "신규 seed", cls: "badge-featured-seed" };
  }

  buildFirstRunBadge(lesson) {
    const step = resolveFirstRunStepByTarget({
      id: lesson?.id,
      source: lesson?.source,
      firstRunPath: lesson?.firstRunPath,
    });
    if (!step) return null;
    return { label: buildFirstRunBadgeLabel(step), cls: "badge-recommended" };
  }

  buildFirstRunHint(lesson) {
    const step = resolveFirstRunStepByTarget({
      id: lesson?.id,
      source: lesson?.source,
      firstRunPath: lesson?.firstRunPath,
    });
    return buildFirstRunHintText(step);
  }

  buildLegacyWarningBadge(lesson) {
    const count = Math.max(0, Math.trunc(Number(lesson?.maegimControlWarningCount) || 0));
    const codes = Array.isArray(lesson?.maegimControlWarningCodes) ? lesson.maegimControlWarningCodes : [];
    if (count <= 0 || !codes.includes(LEGACY_WARNING_CODE)) return null;
    const source = String(lesson?.maegimControlWarningSource ?? "").trim();
    const sourceSuffix = source ? ` · ${source}` : "";
    return {
      label: `구식범위주석 ${count}건`,
      title: `legacy // 범위(...) 경고 ${count}건${sourceSuffix}`,
    };
  }

  getLessonLegacyWarningCount(lesson) {
    return Math.max(0, Math.trunc(Number(lesson?.maegimControlWarningCount) || 0));
  }

  getLessonLegacyWarningNames(lesson) {
    return Array.isArray(lesson?.maegimControlWarningNames)
      ? lesson.maegimControlWarningNames
          .map((item) => String(item ?? "").trim())
          .filter(Boolean)
      : [];
  }

  getLessonLegacyWarningExamples(lesson) {
    return Array.isArray(lesson?.maegimControlWarningExamples)
      ? lesson.maegimControlWarningExamples
          .map((item) => String(item ?? "").trim())
          .filter(Boolean)
      : [];
  }

  buildLegacyWarningGuideMessage(lesson) {
    const count = this.getLessonLegacyWarningCount(lesson);
    if (count <= 0) return "";
    const title = String(lesson?.title ?? lesson?.id ?? "lesson").trim() || "lesson";
    const names = this.getLessonLegacyWarningNames(lesson);
    const namesLine = names.length > 0 ? `대상 항목: ${names.join(", ")}` : "";
    const examples = this.getLessonLegacyWarningExamples(lesson);
    const sampleName = names[0] || "g";
    const exampleLines = examples.length > 0 ? ["예시:", ...examples.slice(0, 2)] : [`예: ${sampleName}:수 = (9.8) 매김 { 범위: 1..20. 간격: 0.1. }.`];
    return [
      `${title}: 구식 범위주석 ${count}건`,
      namesLine,
      "`// 범위(...)` 대신 `채비` 항목을 `(값) 매김 { 범위: ... 간격: ... }.` 로 옮기세요.",
      "주의: 전환 뒤에는 원래 `// 범위(...)`가 붙은 선언 줄을 지우거나 아래 `매김 {}` 줄로 교체해야 합니다.",
      ...exampleLines,
    ]
      .filter(Boolean)
      .join("\n");
  }

  openLegacyGuideExample(lesson) {
    const examples = this.getLessonLegacyWarningExamples(lesson);
    const example = examples[0] || "";
    if (!example) return;
    this.onOpenLegacyGuideExample({
      lesson,
      example,
      examples,
      warningNames: this.getLessonLegacyWarningNames(lesson),
    });
  }

  showLegacyWarningGuide(lesson) {
    const message = this.buildLegacyWarningGuideMessage(lesson);
    if (!message || !this.legacyGuideHintEl) return;
    this.legacyGuideHintEl.textContent = message;
    this.legacyGuideHintEl.classList.remove("hidden");
    if (typeof document?.createElement === "function") {
      const openButton = document.createElement("button");
      openButton.type = "button";
      openButton.className = "ghost";
      openButton.textContent = "예시 열기";
      openButton.addEventListener("click", (event) => {
        event.preventDefault();
        event.stopPropagation();
        this.openLegacyGuideExample(lesson);
      });
      this.legacyGuideHintEl.appendChild(openButton);
    }
  }

  setLessons(lessons = []) {
    this.lessons = Array.isArray(lessons)
      ? lessons.map((lesson) => ({
          ...lesson,
          subject: normalizeSubject(lesson.subject),
          grade: normalizeGrade(lesson.grade),
          quality: normalizeQuality(lesson.quality),
        }))
      : [];
    this.hasAutoSelectedCourseDetail = false;
    this.rebuildFilterOptions();
    this.render();
  }

  rebuildFilterOptions() {
    if (this.gradeSelect) {
      const grades = Array.from(new Set(this.lessons.map((lesson) => normalizeGrade(lesson.grade)).filter(Boolean))).sort();
      this.gradeSelect.innerHTML = [
        '<option value="">학년: 전체</option>',
        ...grades.map((grade) => `<option value="${grade}">${formatGradeLabel(grade)}</option>`),
      ].join("");
      this.gradeSelect.value = this.filter.grade;
    }
    if (this.subjectSelect) {
      const subjects = Array.from(new Set(this.lessons.map((lesson) => normalizeSubject(lesson.subject)).filter(Boolean))).sort();
      this.subjectSelect.innerHTML = [
        '<option value="">과목: 대표 교과</option>',
        ...subjects.map((subject) => `<option value="${subject}">${formatSubjectLabel(subject)}</option>`),
      ].join("");
      this.subjectSelect.value = this.filter.subject;
    }
    if (this.qualitySelect) {
      const qualityLabels = {
        recommended: "추천",
        reviewed: "검수완료",
        experimental: "실험용",
      };
      const qualities = Array.from(
        new Set(this.lessons.map((lesson) => normalizeQuality(lesson.quality)).filter(Boolean)),
      ).sort();
      this.qualitySelect.innerHTML = [
        '<option value="">품질: 전체</option>',
        ...qualities.map((quality) => {
          const label = qualityLabels[quality] ?? quality;
          return `<option value="${quality}">품질: ${label}</option>`;
        }),
      ].join("");
      this.qualitySelect.value = this.filter.quality;
    }
  }

  toFederatedLessonItems(payload) {
    const rows = Array.isArray(payload?.lessons)
      ? payload.lessons
      : Array.isArray(payload?.samples)
        ? payload.samples
      : Array.isArray(payload)
        ? payload
        : [];
    return rows
      .map((row) => {
        const id = String(row?.id ?? row?.lesson_id ?? "").trim();
        if (!id) return null;
        const ddnPath = []
          .concat(row?.ddn_path ?? [])
          .concat(row?.ddnCandidates ?? [])
          .concat(row?.lesson_ddn_path ?? [])
          .filter(Boolean);
        const textPath = []
          .concat(row?.text_path ?? [])
          .concat(row?.textCandidates ?? [])
          .concat(row?.text_md_path ?? [])
          .filter(Boolean);
        const graphPath = []
          .concat(row?.graph_path ?? [])
          .concat(row?.graphCandidates ?? [])
          .concat(row?.graph_json_path ?? [])
          .filter(Boolean);
        const structurePath = []
          .concat(row?.structure_path ?? [])
          .concat(row?.structureCandidates ?? [])
          .filter(Boolean);
        const metaPath = []
          .concat(row?.meta_path ?? [])
          .concat(row?.metaCandidates ?? [])
          .filter(Boolean);
        return {
          id,
          title: String(row?.title ?? row?.name ?? row?.lesson_id ?? id).trim(),
          description: String(row?.description ?? ""),
          grade: normalizeGrade(row?.grade),
          subject: normalizeSubject(row?.subject),
          quality: normalizeQuality(row?.quality),
          source: String(row?.source ?? "federated"),
          firstRunPath: String(row?.first_run_path ?? row?.firstRunPath ?? "").trim(),
          tags: normalizeStringList(row?.tags),
          maegimControlWarningCount: Math.max(0, Math.trunc(Number(row?.maegim_control_warning_count ?? row?.maegimControlWarningCount) || 0)),
          maegimControlWarningCodes: Array.isArray(row?.maegim_control_warning_codes)
            ? row.maegim_control_warning_codes.map((item) => String(item))
            : Array.isArray(row?.maegimControlWarningCodes)
              ? row.maegimControlWarningCodes.map((item) => String(item))
              : [],
          maegimControlWarningNames: Array.isArray(row?.maegim_control_warning_names)
            ? row.maegim_control_warning_names.map((item) => String(item))
            : Array.isArray(row?.maegimControlWarningNames)
              ? row.maegimControlWarningNames.map((item) => String(item))
              : [],
          maegimControlWarningExamples: Array.isArray(row?.maegim_control_warning_examples)
            ? row.maegim_control_warning_examples.map((item) => String(item))
            : Array.isArray(row?.maegimControlWarningExamples)
              ? row.maegimControlWarningExamples.map((item) => String(item))
              : [],
          maegimControlWarningSource: String(row?.maegim_control_warning_source ?? row?.maegimControlWarningSource ?? ""),
          ddnCandidates: Array.from(new Set(ddnPath.map((item) => String(item)))),
          textCandidates: Array.from(new Set(textPath.map((item) => String(item)))),
          graphCandidates: Array.from(new Set(graphPath.map((item) => String(item)))),
          structureCandidates: Array.from(new Set(structurePath.map((item) => String(item)))),
          metaCandidates: Array.from(new Set(metaPath.map((item) => String(item)))),
        };
      })
      .filter(Boolean);
  }

  async tryLoadFederatedCandidate(candidate) {
    try {
      const response = await fetch(candidate, { cache: "no-cache" });
      if (!response.ok) return false;
      const json = await response.json();
      this.searchResults = this.toFederatedLessonItems(json);
      this.federatedLoadState = "loaded";
      return true;
    } catch (_) {
      return false;
    }
  }

  async tryLoadSampleCandidate(candidate) {
    try {
      const response = await fetch(candidate, { cache: "no-cache" });
      if (!response.ok) return false;
      const json = await response.json();
      this.sampleResults = this.toFederatedLessonItems(json);
      this.sampleLoadState = "loaded";
      return true;
    } catch (_) {
      return false;
    }
  }

  async loadFederatedResults() {
    if (this.federatedLoadState === "loaded" || this.federatedLoadState === "unavailable") {
      return;
    }
    for (const candidate of this.federatedApiCandidates) {
      if (await this.tryLoadFederatedCandidate(candidate)) {
        return;
      }
    }

    const uniqueCandidates = [];
    const seen = new Set();
    for (const candidate of this.federatedFileCandidates) {
      try {
        const resolved = new URL(candidate, window.location.href).href;
        if (seen.has(resolved)) continue;
        seen.add(resolved);
        uniqueCandidates.push(resolved);
      } catch (_) {
        if (seen.has(candidate)) continue;
        seen.add(candidate);
        uniqueCandidates.push(candidate);
      }
    }

    for (const candidate of uniqueCandidates) {
      if (await this.tryLoadFederatedCandidate(candidate)) {
        return;
      }
    }
    this.searchResults = [];
    this.federatedLoadState = "unavailable";
  }

  async loadSampleResults() {
    if (this.sampleLoadState === "loaded" || this.sampleLoadState === "unavailable") {
      return;
    }
    for (const candidate of this.sampleCandidates) {
      if (await this.tryLoadSampleCandidate(candidate)) {
        return;
      }
    }
    this.sampleResults = [];
    this.sampleLoadState = "unavailable";
  }

  currentPool() {
    if (this.activeTab === "search") return mergeLessonPools(this.lessons, this.searchResults);
    if (this.activeTab === "examples") return this.sampleResults;
    return this.lessons;
  }

  filteredLessons(filterOverride = {}) {
    const filter = { ...this.filter, ...(filterOverride && typeof filterOverride === "object" ? filterOverride : {}) };
    const grade = normalizeGrade(filter.grade);
    const subject = normalizeSubject(filter.subject);
    const quality = normalizeQuality(filter.quality || "experimental");
    const hasQualityFilter = String(filter.quality ?? "").trim().length > 0;
    const seedScope = String(filter.seedScope ?? "").trim();
    const runStatus = String(filter.runStatus ?? "").trim();
    const runLaunch = String(filter.runLaunch ?? "").trim();
    const warningStatus = String(filter.warningStatus ?? "").trim();
    const sortMode = String(filter.sort ?? "teacher").trim();
    const effectiveSortMode = warningStatus && sortMode === "teacher" ? "legacy_warning" : sortMode;
    const query = String(filter.query ?? "").trim().toLowerCase();
    const numericTrackOnly = Boolean(filter.numericTrack);
    const numericTrackResultsOnly = Boolean(filter.numericTrackResults);
    const defaultCourseOnly = this.activeTab === "official"
      && !subject
      && !query
      && !numericTrackOnly
      && !numericTrackResultsOnly
      && !shouldShowLegacyBrowseControls();

    const filtered = this.currentPool().filter((lesson) => {
      if (numericTrackOnly && !isNumericTrackLesson(lesson)) return false;
      if (numericTrackResultsOnly && !this.getLessonNumericTrackRunResultLink(lesson?.id)) return false;
      if (grade && normalizeGrade(lesson.grade) !== grade) return false;
      if (subject && normalizeSubject(lesson.subject) !== subject) return false;
      if (defaultCourseOnly && !isTeacherDefaultCourseSubject(lesson.subject)) return false;
      if (hasQualityFilter && normalizeQuality(lesson.quality) !== quality) return false;
      if (seedScope === "featured_seed" && !isFeaturedSeedLesson(lesson)) return false;
      if (seedScope === "seed_only" && normalizeSource(lesson?.source) !== "seed") return false;
      if (runStatus) {
        const kind = this.getLessonLastRunKind(lesson.id);
        const isNone = kind === "none";
        const isError = kind === "error";
        const isSuccess = !isNone && !isError;
        if (runStatus === "none" && !isNone) return false;
        if (runStatus === "error" && !isError) return false;
        if (runStatus === "success" && !isSuccess) return false;
      }
      if (runLaunch) {
        const launchKind = this.getLessonLastLaunchKind(lesson.id);
        if (runLaunch === "none" && launchKind) return false;
        if (runLaunch === "featured_seed_quick" && launchKind !== "featured_seed_quick") return false;
        if (runLaunch === "browse_select" && launchKind !== "browse_select") return false;
        if (runLaunch === "editor_run" && launchKind !== "editor_run") return false;
        if (runLaunch === "manual" && launchKind !== "manual") return false;
      }
      if (warningStatus) {
        const hasLegacyWarning = this.getLessonLegacyWarningCount(lesson) > 0;
        if (warningStatus === "has_legacy_warning" && !hasLegacyWarning) return false;
        if (warningStatus === "clean" && hasLegacyWarning) return false;
      }
      if (!lessonMatchesQuery(lesson, query)) return false;
      return true;
    });

    if (effectiveSortMode === "teacher") {
      filtered.sort((a, b) => {
        const aRank = this.getTeacherCatalogRank(a);
        const bRank = this.getTeacherCatalogRank(b);
        if (aRank !== bRank) return aRank - bRank;
        return String(a.title ?? a.id ?? "").localeCompare(String(b.title ?? b.id ?? ""), "ko");
      });
    } else if (effectiveSortMode === "recent") {
      filtered.sort((a, b) => {
        const aMs = this.getLessonLastRunAtMs(a.id);
        const bMs = this.getLessonLastRunAtMs(b.id);
        if (aMs !== bMs) return bMs - aMs;
        return String(a.title ?? a.id ?? "").localeCompare(String(b.title ?? b.id ?? ""), "ko");
      });
    } else if (effectiveSortMode === "legacy_warning") {
      filtered.sort((a, b) => {
        const aCount = this.getLessonLegacyWarningCount(a);
        const bCount = this.getLessonLegacyWarningCount(b);
        if (aCount !== bCount) return bCount - aCount;
        return String(a.title ?? a.id ?? "").localeCompare(String(b.title ?? b.id ?? ""), "ko");
      });
    } else if (effectiveSortMode === "featured_seed") {
      filtered.sort((a, b) => {
        const aRank = this.getFeaturedSeedRank(a);
        const bRank = this.getFeaturedSeedRank(b);
        if (aRank !== bRank) return aRank - bRank;
        const aMs = this.getLessonLastRunAtMs(a.id);
        const bMs = this.getLessonLastRunAtMs(b.id);
        if (aMs !== bMs) return bMs - aMs;
        return String(a.title ?? a.id ?? "").localeCompare(String(b.title ?? b.id ?? ""), "ko");
      });
    } else if (effectiveSortMode === "featured_seed_quick_recent") {
      filtered.sort((a, b) => {
        const aLaunchQuick = this.getLessonLastLaunchKind(a.id) === "featured_seed_quick" ? 0 : 1;
        const bLaunchQuick = this.getLessonLastLaunchKind(b.id) === "featured_seed_quick" ? 0 : 1;
        if (aLaunchQuick !== bLaunchQuick) return aLaunchQuick - bLaunchQuick;
        const aMs = this.getLessonLastRunAtMs(a.id);
        const bMs = this.getLessonLastRunAtMs(b.id);
        if (aMs !== bMs) return bMs - aMs;
        return String(a.title ?? a.id ?? "").localeCompare(String(b.title ?? b.id ?? ""), "ko");
      });
    }
    return filtered;
  }

  applyFeaturedSeedQuickRecentPreset() {
    if (!this.featuredSeedEnabled) return;
    this.filter.runLaunch = "featured_seed_quick";
    this.filter.sort = "featured_seed_quick_recent";
    if (this.runLaunchSelect) {
      this.runLaunchSelect.value = this.filter.runLaunch;
    }
    if (this.sortSelect) {
      this.sortSelect.value = this.filter.sort;
    }
    this.saveBrowsePrefs();
    this.render();
  }

  applyBrowsePreset(presetId = "") {
    if (!this.featuredSeedEnabled) return false;
    const normalized = String(presetId ?? "").trim().toLowerCase();
    if (normalized === "featured_seed_quick_recent") {
      this.applyFeaturedSeedQuickRecentPreset();
      return true;
    }
    return false;
  }

  getActiveBrowsePresetId() {
    return resolveBrowsePresetId(this.filter);
  }

  emitBrowsePresetChanged({ force = false } = {}) {
    const presetId = this.getActiveBrowsePresetId();
    if (!force && presetId === this.lastBrowsePresetId) return;
    this.lastBrowsePresetId = presetId;
    try {
      window.dispatchEvent(new CustomEvent("seamgrim:browse-preset-changed", {
        detail: {
          presetId,
        },
      }));
    } catch (_) {
      // ignore event dispatch errors
    }
  }

  updateFeaturedSeedQuickPresetButton() {
    const button = this.presetFeaturedSeedQuickRecentButton;
    if (!button) return;
    if (!this.featuredSeedEnabled) {
      button.classList.add("hidden");
      return;
    }
    const pool = this.currentPool();
    const quickCount = Array.isArray(pool)
      ? pool.filter((lesson) => this.getLessonLastLaunchKind(lesson?.id) === "featured_seed_quick").length
      : 0;
    const active = this.filter.runLaunch === "featured_seed_quick" && this.filter.sort === "featured_seed_quick_recent";
    button.classList.toggle("active", active);
    button.disabled = quickCount <= 0;
    button.title = quickCount > 0
      ? `Alt+6 quick-launch 기록 ${quickCount}건을 최근순으로 정렬합니다.`
      : "Alt+6 quick-launch 실행 기록이 없습니다.";
  }

  getBrowsePresetShareUrl() {
    const presetId = this.getActiveBrowsePresetId();
    return buildBrowsePresetShareUrl(presetId);
  }

  updateBrowsePresetCopyButton() {
    const button = this.copyBrowsePresetLinkButton;
    if (!button) return;
    const presetId = this.getActiveBrowsePresetId();
    if (!presetId) {
      button.disabled = true;
      button.title = "활성 preset이 없어서 복사할 링크가 없습니다.";
      return;
    }
    const shareUrl = this.getBrowsePresetShareUrl();
    button.disabled = !shareUrl;
    button.title = shareUrl || "preset 링크를 만들지 못했습니다.";
  }

  async handleCopyBrowsePresetLink() {
    const button = this.copyBrowsePresetLinkButton;
    if (!button || button.disabled) return;
    const shareUrl = this.getBrowsePresetShareUrl();
    if (!shareUrl) return;
    const ok = await this.copyToClipboard(shareUrl);
    showGlobalToast(ok ? "프리셋 링크를 복사했습니다." : "프리셋 링크 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
  }

  async handleCopyNumericTrackResultSummary() {
    const button = this.copyNumericTrackResultSummaryButton;
    if (!button || button.disabled) return;
    const snapshot = this.buildNumericTrackResultHistorySnapshot();
    const summary = buildNumericTrackResultSummaryExport(snapshot);
    const text = formatNumericTrackResultSummaryExportText(summary);
    const ok = await this.copyToClipboard(text);
    showGlobalToast(ok ? "수치 결과 요약을 복사했습니다." : "수치 결과 요약 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
  }

  getCourseSequenceText(lesson, index = null) {
    if (this.activeTab !== "official") return "";
    let sequenceIndex = typeof index === "number" && Number.isFinite(index) ? index : -1;
    if (sequenceIndex < 0) {
      const lessonId = String(lesson?.id ?? "").trim();
      if (!lessonId) return "";
      sequenceIndex = this.filteredLessons().findIndex((row) => String(row?.id ?? "").trim() === lessonId);
    }
    return sequenceIndex >= 0 ? `추천 순서 ${sequenceIndex + 1}` : "";
  }

  createLessonCard(lesson, index = 0) {
    const showLegacyControls = shouldShowLegacyBrowseControls();
    const qualityKey = QUALITY_BADGE[lesson.quality] ? lesson.quality : "experimental";
    const badge = QUALITY_BADGE[qualityKey];
    const firstRunBadge = this.buildFirstRunBadge(lesson);
    const firstRunHint = this.buildFirstRunHint(lesson);
    const stateHint = this.buildCardStateHint(lesson.id);
    const runHint = this.buildCardRunHint(lesson.id);
    const runBadge = this.buildRunBadge(lesson.id);
    const launchBadge = this.buildLaunchBadge(lesson.id);
    const numericTrackResultHint = this.buildNumericTrackResultHint(lesson.id);
    const legacyWarningBadge = this.buildLegacyWarningBadge(lesson);
    const featuredSeedBadge = this.buildFeaturedSeedBadge(lesson);
    const numericTrackResultBadge = numericTrackResultHint
      ? { label: "결과기록", cls: "badge-numeric-track-result" }
      : null;
    const sampleBadge = String(lesson?.source ?? "").trim().toLowerCase() === "sample"
      ? { label: "예제", cls: "badge-reviewed" }
      : null;
    const runHash = this.getLessonLastRunHash(lesson.id);
    const runHashShort = runHash ? runHash.slice(0, 12) : "";
    const gradeLabel = formatGradeLabel(lesson.grade);
    const subjectLabel = formatSubjectLabel(lesson.subject);
    const courseSurfaceText = buildCourseSurfaceText(lesson);
    const courseResultText = buildCourseResultText(lesson);
    const courseDeliveryText = buildCourseDeliveryText(lesson);
    const courseReadinessText = buildCourseReadinessText(lesson);
    const courseActivityText = buildCourseActivityText(lesson);
    const courseGoals = buildCourseGoalTexts(lesson);
    const courseMissions = buildCourseMissionTexts(lesson);
    const courseSequenceText = this.getCourseSequenceText(lesson, index);

    const card = document.createElement("article");
    card.className = "lesson-card";
    card.tabIndex = 0;
    card.setAttribute("role", "button");
    card.setAttribute("aria-label", `${String(lesson.title || lesson.id || "교과").trim()} 상세 보기`);
    card.dataset.lessonId = lesson.id;
    if (courseSequenceText) card.dataset.courseSequence = courseSequenceText;
    card.innerHTML = `
      <div class="card-top-badges">
        <span class="quality-badge ${badge.cls}">${badge.label}</span>
        ${sampleBadge ? `<span class="quality-badge ${sampleBadge.cls}">${sampleBadge.label}</span>` : ""}
        ${firstRunBadge ? `<span class="quality-badge ${firstRunBadge.cls}">${firstRunBadge.label}</span>` : ""}
        ${featuredSeedBadge ? `<span class="quality-badge ${featuredSeedBadge.cls}">${featuredSeedBadge.label}</span>` : ""}
        ${numericTrackResultBadge ? `<span class="quality-badge ${numericTrackResultBadge.cls}">${numericTrackResultBadge.label}</span>` : ""}
      </div>
      ${courseSequenceText ? `<div class="card-course-sequence" data-course-sequence>${escapeHtml(courseSequenceText)}</div>` : ""}
      <div class="card-title">${lesson.title || lesson.id}</div>
      <div class="card-meta">${gradeLabel} · ${subjectLabel}</div>
      <div class="card-desc">${lesson.description || "설명 없음"}</div>
      <div class="card-course-surface" data-course-surface>${escapeHtml(courseSurfaceText)}</div>
      <div class="card-course-result" data-course-result>${escapeHtml(courseResultText)}</div>
      <div class="card-course-delivery" data-course-delivery>${escapeHtml(courseDeliveryText)}</div>
      <div class="card-course-readiness" data-course-readiness>${escapeHtml(courseReadinessText)}</div>
      <div class="card-course-activity" data-course-activity>${escapeHtml(courseActivityText)}</div>
      ${courseGoals.length
        ? `<ul class="card-course-goals" data-course-goals>
          ${courseGoals.map((goal) => `<li>${escapeHtml(goal)}</li>`).join("")}
        </ul>`
        : ""}
      ${courseMissions.length
        ? `<div class="card-course-missions" data-course-missions>
          <span>수업 활동</span>
          <ul>
            ${courseMissions.map((mission) => `<li>${escapeHtml(mission)}</li>`).join("")}
          </ul>
        </div>`
        : ""}
      ${showLegacyControls && firstRunHint ? `<div class="card-state-hint">${firstRunHint}</div>` : ""}
      ${showLegacyControls
        ? `<div class="card-badge-row">
          <span class="card-run-badge ${runBadge.cls}">${runBadge.label}</span>
          ${launchBadge ? `<span class="card-run-badge ${launchBadge.cls}">${launchBadge.label}</span>` : ""}
          ${legacyWarningBadge
            ? `<button type="button" class="card-warning-badge" title="${legacyWarningBadge.title} · 클릭해 전환 가이드 보기">${legacyWarningBadge.label}</button>`
            : ""}
          ${runHashShort
            ? `<button type="button" class="card-hash-copy" data-run-hash="${runHash}" title="실행 기록 ID 복사">기록ID:${runHashShort}</button>`
            : ""}
        </div>
        <div class="card-state-hint">${stateHint}</div>
        ${numericTrackResultHint ? `<div class="card-state-hint">${numericTrackResultHint}</div>` : ""}
        <div class="card-run-hint">${runHint}</div>`
        : ""}
      <div class="card-launch-actions">
        <button type="button" class="card-launch-primary card-launch-btn" data-launch-profile="student" title="학생 시작 프로필로 실행">▶ 학생 시작</button>
        <button type="button" class="ghost card-launch-btn" data-launch-profile="teacher" title="교사용 배포 준비 프로필로 실행">교사용 배포</button>
      </div>
    `;
    const shouldShowPreview = lessonHasPreviewDescriptor(lesson);
    if (shouldShowPreview && typeof document?.createElement === "function") {
      const preview = document.createElement("div");
      preview.className = "card-structure-preview hidden";
      preview.textContent = "미리보기 로딩 중…";
      card.appendChild(preview);
      this.enqueueLessonPreview(card, preview, lesson);
    }
    card.querySelector(".card-hash-copy")?.addEventListener("click", async (event) => {
      event.preventDefault();
      event.stopPropagation();
      const hash = String(event.currentTarget?.dataset?.runHash ?? "").trim();
      if (!hash) return;
      const ok = await this.copyToClipboard(hash);
      showGlobalToast(ok ? "실행 기록 ID를 복사했습니다." : "실행 기록 ID 복사에 실패했습니다.", {
        kind: ok ? "success" : "error",
      });
    });
    card.querySelector(".card-warning-badge")?.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      this.showLegacyWarningGuide(lesson);
    });
    card.querySelectorAll(".card-launch-btn")?.forEach((button) => {
      button.addEventListener("click", (event) => {
        event.preventDefault();
        event.stopPropagation();
        const launchProfile = normalizeLaunchProfile(button?.dataset?.launchProfile ?? "");
        void this.onLessonSelect({
          ...lesson,
          launchProfile,
        }, { autoExecute: true });
      });
    });
    card.addEventListener("click", () => {
      if (this.detailPanelEl) {
        this.showLessonDetail(lesson);
        return;
      }
      void this.onLessonSelect(lesson, { autoExecute: true });
    });
    card.addEventListener("keydown", (event) => {
      if (event.key !== "Enter" && event.key !== " ") return;
      event.preventDefault();
      if (this.detailPanelEl) {
        this.showLessonDetail(lesson);
        return;
      }
      void this.onLessonSelect(lesson, { autoExecute: true });
    });
    return card;
  }

  showLessonDetail(lesson) {
    this.detailLesson = lesson && typeof lesson === "object" ? lesson : null;
    if (!this.detailPanelEl || !this.detailLesson) {
      return;
    }
    const courseSequenceText = this.getCourseSequenceText(this.detailLesson);
    const subject = String(this.detailLesson.subject ?? "").trim() || "-";
    const grade = String(this.detailLesson.grade ?? "").trim() || "-";
    if (this.detailSubjectBadgeEl) {
      this.detailSubjectBadgeEl.textContent = `${formatSubjectLabel(subject)} · ${formatGradeLabel(grade)}`;
    }
    if (this.detailTitleEl) {
      this.detailTitleEl.textContent = String(this.detailLesson.title ?? this.detailLesson.id ?? "").trim() || "교과";
    }
    if (this.detailDescEl) {
      this.detailDescEl.textContent = String(this.detailLesson.description ?? "").trim() || "설명 없음";
    }
    if (this.detailKeywordsEl) {
      const curriculumMeta = this.detailLesson.curriculumMeta && typeof this.detailLesson.curriculumMeta === "object"
        ? this.detailLesson.curriculumMeta
        : null;
      const keywords = Array.isArray(this.detailLesson.keywords) ? this.detailLesson.keywords : curriculumMeta?.coreConcepts ?? [];
      this.detailKeywordsEl.innerHTML = keywords.length
        ? keywords.map((keyword) => `<span class="detail-keyword">${escapeHtml(String(keyword ?? "").trim())}</span>`).join("")
        : "";
    }
    if (this.detailCurriculumEl) {
      const curriculumMeta = this.detailLesson.curriculumMeta && typeof this.detailLesson.curriculumMeta === "object"
        ? this.detailLesson.curriculumMeta
        : null;
      const numericPreview = buildNumericTrackLessonPreview(this.detailLesson);
      this.updateNumericTrackReportButton(numericPreview);
      const numericSections = numericPreview
        ? [
            renderDetailList("그래프·표 수업", [
              numericPreview.summary,
              numericPreview.module_labels?.length ? `모듈: ${numericPreview.module_labels.join(", ")}` : "",
            ]),
            renderDetailList("수치 근거", numericPreview.evidence_packs),
          ]
        : [];
      if (this.detailPanelEl) {
        setElementDatasetValue(this.detailPanelEl, "numericTrack", numericPreview ? "1" : "0");
        setElementDatasetValue(this.detailPanelEl, "numericTrackReopen", "0");
        setElementDatasetValue(this.detailPanelEl, "reopenLessonId", "");
      }
      try {
        window.__SEAMGRIM_NUMERIC_TRACK_DETAIL_PREVIEW__ = numericPreview;
        window.__SEAMGRIM_NUMERIC_TRACK_DETAIL_PREVIEW_TEXT__ = numericPreview
          ? formatNumericTrackLessonPreviewText(numericPreview)
          : "";
      } catch (_) {
        // ignore browser instrumentation errors
      }
      if (!curriculumMeta) {
        const ddnPreviewSection = renderDetailDdnPreviewSection();
        const goalRows = buildCourseGoalTexts(this.detailLesson);
        const missionRows = buildCourseMissionTexts(this.detailLesson);
        const viewRows = Array.isArray(this.detailLesson.requiredViews) && this.detailLesson.requiredViews.length
          ? [`결과 확인: ${this.detailLesson.requiredViews.map(formatRequiredViewLabel).join(", ")}`]
          : [];
        this.detailCurriculumEl.innerHTML = [
          renderDetailStudentPreviewSection(this.detailLesson),
          renderDetailRunReadinessSection(this.detailLesson),
          renderDetailCourseFlowSection(this.detailLesson, courseSequenceText),
          renderDetailList("학습목표", goalRows),
          renderDetailList("수업 활동", missionRows),
          renderDetailList("결과 확인", viewRows),
          ...numericSections,
          ddnPreviewSection,
        ].join("");
      } else {
        const ddnPreviewSection = renderDetailDdnPreviewSection();
        const titleRows = [
          curriculumMeta.unit ? `단원: ${curriculumMeta.unit}` : "",
          curriculumMeta.lesson ? `차시: ${curriculumMeta.lesson}` : "",
          curriculumMeta.difficulty ? `난이도: ${curriculumMeta.difficulty}` : "",
          curriculumMeta.requiredViews?.length ? `결과 확인: ${curriculumMeta.requiredViews.map(formatRequiredViewLabel).join(", ")}` : "",
        ].filter(Boolean);
        const refRows = [
          curriculumMeta.teacherNotesRef ? `교사용: ${curriculumMeta.teacherNotesRef}` : "",
          curriculumMeta.studentSheetRef ? `학생용: ${curriculumMeta.studentSheetRef}` : "",
        ].filter(Boolean);
        this.detailCurriculumEl.innerHTML = [
          renderDetailStudentPreviewSection(this.detailLesson),
          renderDetailRunReadinessSection(this.detailLesson),
          renderDetailCourseFlowSection(this.detailLesson, courseSequenceText),
          renderDetailList("차시 정보", titleRows),
          renderDetailList("학습목표", curriculumMeta.learningGoals),
          renderDetailList("핵심개념", curriculumMeta.coreConcepts),
          renderDetailList("자료", refRows),
          ...numericSections,
          ddnPreviewSection,
        ].join("");
      }
      void this.hydrateDetailDdnPreview(this.detailLesson);
      this.wireDetailDdnCopyButton();
    }
    this.detailPanelEl.classList.remove("hidden");
    this.updateSelectedLessonCard();
  }

  hideLessonDetail() {
    this.detailLesson = null;
    this.updateNumericTrackReportButton(null);
    if (this.detailPanelEl) {
      setElementDatasetValue(this.detailPanelEl, "numericTrack", "0");
      setElementDatasetValue(this.detailPanelEl, "numericTrackReopen", "0");
      setElementDatasetValue(this.detailPanelEl, "reopenLessonId", "");
    }
    try {
      window.__SEAMGRIM_NUMERIC_TRACK_DETAIL_PREVIEW__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_DETAIL_PREVIEW_TEXT__ = "";
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_REOPEN_TARGET__ = null;
      window.__SEAMGRIM_NUMERIC_TRACK_RESULT_REOPEN_TARGET_TEXT__ = "";
    } catch (_) {
      // ignore browser instrumentation errors
    }
    this.detailPanelEl?.classList?.add?.("hidden");
    this.updateSelectedLessonCard();
  }

  resolveDetailDdnPreviewCandidates(lesson) {
    const rawCandidates = normalizeCandidateList([
      ...(Array.isArray(lesson?.ddnCandidates) ? lesson.ddnCandidates : []),
      ...(Array.isArray(lesson?.ddn_path) ? lesson.ddn_path : []),
      ...(Array.isArray(lesson?.lesson_ddn_path) ? lesson.lesson_ddn_path : []),
    ]);
    const expanded = [];
    rawCandidates.forEach((candidate) => {
      const normalized = String(candidate ?? "").trim().replace(/\\/g, "/").replace(/^\/+/, "");
      if (!normalized) return;
      if (/^https?:\/\//i.test(candidate) || String(candidate).startsWith("/")) {
        expanded.push(candidate);
        return;
      }
      const stripped = normalized.startsWith(PROJECT_PREFIX)
        ? normalized.slice(PROJECT_PREFIX.length)
        : normalized;
      expanded.push(`/${PROJECT_PREFIX}${stripped}`);
      expanded.push(`/${stripped}`);
      expanded.push(candidate);
      expanded.push(`../${stripped}`);
    });
    return normalizeCandidateList(expanded);
  }

  updateDetailDdnPreview(status, text = "", path = "") {
    const section = this.detailCurriculumEl?.querySelector?.("[data-detail-ddn-preview]");
    const code = section?.querySelector?.("[data-detail-ddn-preview-code]");
    const pathNode = section?.querySelector?.("[data-detail-ddn-preview-path]");
    const copyButton = section?.querySelector?.("[data-detail-ddn-copy]");
    if (!section || !code) return;
    const normalizedStatus = String(status || "unknown");
    section.dataset.detailDdnPreviewStatus = normalizedStatus;
    code.textContent = text || "DDN 원문을 표시할 수 없습니다.";
    if (pathNode) pathNode.textContent = path ? `source: ${path}` : "";
    if (copyButton) {
      copyButton.disabled = normalizedStatus !== "loaded";
      copyButton.dataset.ready = normalizedStatus === "loaded" ? "1" : "0";
    }
  }

  wireDetailDdnCopyButton() {
    const copyButton = this.detailCurriculumEl?.querySelector?.("[data-detail-ddn-copy]");
    if (!copyButton || copyButton.dataset.bound === "1") return;
    copyButton.dataset.bound = "1";
    copyButton.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      void this.handleCopyDetailDdnPreview();
    });
  }

  async handleCopyDetailDdnPreview() {
    const lessonId = String(this.detailLesson?.id ?? "").trim();
    const cached = lessonId ? this.detailDdnPreviewCache.get(lessonId) : null;
    const text = String(cached?.fullText ?? cached?.text ?? "").trim();
    if (!text) {
      showGlobalToast("복사할 DDN 원문이 없습니다.", { kind: "error" });
      return;
    }
    const ok = await this.copyToClipboard(text);
    showGlobalToast(ok ? "DDN 원문을 복사했습니다." : "DDN 원문 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
  }

  async hydrateDetailDdnPreview(lesson) {
    const lessonId = String(lesson?.id ?? "").trim();
    const token = ++this.detailDdnPreviewSeq;
    const candidates = this.resolveDetailDdnPreviewCandidates(lesson);
    if (!lessonId || candidates.length <= 0) {
      this.updateDetailDdnPreview("missing", "DDN 원문 경로가 없습니다.");
      return;
    }
    const cached = this.detailDdnPreviewCache.get(lessonId);
    if (cached) {
      this.updateDetailDdnPreview("loaded", cached.text, cached.path);
      return;
    }
    for (const candidate of candidates) {
      try {
        const response = await fetch(candidate, { cache: "no-cache" });
        if (!response.ok) continue;
        const raw = await response.text();
        const previewText = clipDdnPreviewText(raw);
        if (!previewText) continue;
        if (token !== this.detailDdnPreviewSeq || String(this.detailLesson?.id ?? "") !== lessonId) return;
        const entry = { text: previewText, fullText: raw, path: candidate };
        this.detailDdnPreviewCache.set(lessonId, entry);
        this.updateDetailDdnPreview("loaded", entry.text, entry.path);
        return;
      } catch (_) {
        // try next candidate
      }
    }
    if (token !== this.detailDdnPreviewSeq || String(this.detailLesson?.id ?? "") !== lessonId) return;
    this.updateDetailDdnPreview("error", "DDN 원문을 불러오지 못했습니다.", candidates[0] || "");
  }

  updateSelectedLessonCard() {
    if (!this.grid) return;
    const selectedId = String(this.detailLesson?.id ?? "").trim();
    this.grid.querySelectorAll(".lesson-card").forEach((card) => {
      const active = Boolean(selectedId) && String(card?.dataset?.lessonId ?? "") === selectedId;
      card.classList.toggle("lesson-card-selected", active);
      card.setAttribute("aria-pressed", active ? "true" : "false");
    });
  }

  maybeAutoShowCourseDetail(lessons) {
    if (this.hasAutoSelectedCourseDetail || !this.detailPanelEl) return;
    if (this.activeTab !== "official") return;
    if (!Array.isArray(lessons) || lessons.length <= 0) return;
    this.hasAutoSelectedCourseDetail = true;
    this.showLessonDetail(lessons[0]);
  }

  buildNumericTrackReportExport() {
    return buildNumericTrackReportExport(this.lessons);
  }

  formatNumericTrackReportExportText() {
    return formatNumericTrackReportExportText(this.buildNumericTrackReportExport());
  }

  updateNumericTrackReportButton(numericPreview) {
    const button = this.detailCopyNumericTrackReportBtn;
    if (!button) return;
    const active = Boolean(numericPreview);
    button.classList.toggle("hidden", !active);
    button.disabled = !active;
    button.dataset.active = active ? "1" : "0";
    button.title = active
      ? "현재 그래프·표 교과 보고서를 복사합니다."
      : "그래프·표 교과를 선택하면 보고서를 복사할 수 있습니다.";
  }

  async handleCopyNumericTrackReport() {
    const text = this.formatNumericTrackReportExportText();
    const ok = await this.copyToClipboard(text);
    showGlobalToast(ok ? "그래프·표 수업 보고서를 복사했습니다." : "그래프·표 수업 보고서 복사에 실패했습니다.", {
      kind: ok ? "success" : "error",
    });
  }

  ensurePreviewObserver() {
    if (this.previewObserver || typeof IntersectionObserver === "undefined") return this.previewObserver;
    this.previewObserver = new IntersectionObserver((entries) => {
      entries.forEach((entry) => {
        if (!entry?.isIntersecting) return;
        const payload = this.previewPayloadByElement.get(entry.target);
        if (!payload) return;
        this.previewObserver?.unobserve?.(entry.target);
        this.previewPayloadByElement.delete(entry.target);
        this.previewQueue.push(payload);
      });
      this.drainPreviewQueue();
    }, { rootMargin: "100px" });
    return this.previewObserver;
  }

  enqueueLessonPreview(card, preview, lesson) {
    if (!card || !preview || !lesson) return;
    const payload = { preview, lesson };
    const observer = this.ensurePreviewObserver();
    if (!observer) {
      this.previewQueue.push(payload);
      this.drainPreviewQueue();
      return;
    }
    this.previewPayloadByElement.set(card, payload);
    observer.observe(card);
  }

  drainPreviewQueue() {
    while (this.activePreviewCount < PREVIEW_CONCURRENCY_LIMIT && this.previewQueue.length > 0) {
      const next = this.previewQueue.shift();
      if (!next?.preview || !next?.lesson) continue;
      this.activePreviewCount += 1;
      this.hydrateLessonPreview(next.preview, next.lesson)
        .finally(() => {
          this.activePreviewCount = Math.max(0, this.activePreviewCount - 1);
          this.drainPreviewQueue();
        });
    }
  }

  async hydrateLessonPreview(container, lesson) {
    if (!container || !lesson) return;
    const viewModel = await resolveLessonCardPreviewViewModel(lesson, {
      cache: this.lessonPreviewCache,
      fetchImpl: globalThis.fetch,
    });
    if (viewModel?.html) {
      container.innerHTML = viewModel.html;
      applyPreviewViewModelMetadata(container, viewModel);
      container.classList.remove("hidden");
      return;
    }
    applyPreviewViewModelMetadata(container, null);
    container.classList.add("hidden");
  }

  async copyToClipboard(text) {
    const value = String(text ?? "");
    if (!value) return false;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(value);
        return true;
      }
    } catch (_) {
      // fallback below
    }
    try {
      const textarea = document.createElement("textarea");
      textarea.value = value;
      textarea.setAttribute("readonly", "true");
      textarea.style.position = "fixed";
      textarea.style.left = "-9999px";
      document.body.appendChild(textarea);
      textarea.select();
      const ok = document.execCommand("copy");
      document.body.removeChild(textarea);
      return Boolean(ok);
    } catch (_) {
      return false;
    }
  }

  updateCourseCatalogSummary(lessons) {
    if (!this.courseSummaryEl) return;
    const show = this.activeTab === "official" && Array.isArray(lessons) && lessons.length > 0;
    this.courseSummaryEl.classList.toggle("hidden", !show);
    if (!show) {
      this.courseSummaryEl.textContent = "";
      this.courseSummaryEl.dataset.courseCatalogTotal = "0";
      this.courseSummaryEl.dataset.courseCatalogVisible = "0";
      this.updateCourseSubjectShortcuts([]);
      return;
    }
    const subjects = Array.from(new Set(lessons.map((lesson) => formatSubjectLabel(lesson.subject)).filter(Boolean)));
    const subjectText = subjects.slice(0, 4).join(" · ");
    const suffix = subjectText ? ` · ${subjectText}` : "";
    const totalCount = this.currentPool().length;
    const filteredText = totalCount > lessons.length
      ? ` · 전체 ${totalCount}개 중 교사용 대표 ${lessons.length}개 표시`
      : "";
    const subjectCountText = buildSubjectCountSummary(lessons);
    const subjectCountSuffix = subjectCountText ? ` · ${subjectCountText}` : "";
    this.courseSummaryEl.dataset.courseCatalogTotal = String(totalCount);
    this.courseSummaryEl.dataset.courseCatalogVisible = String(lessons.length);
    this.courseSummaryEl.textContent =
      `${lessons.length}개 대표 교과${suffix}${filteredText}${subjectCountSuffix} · DDN 실행 · 학생 시작 · 교사용 배포 가능 · 전체 검색으로 나머지 교과 확인`;
    this.updateCourseSubjectShortcuts(this.filteredLessons({ subject: "" }));
  }

  updateCourseSubjectShortcuts(lessons) {
    if (!this.courseSubjectShortcutsEl) return;
    const show = this.activeTab === "official" && Array.isArray(lessons) && lessons.length > 0;
    this.courseSubjectShortcutsEl.classList.toggle("hidden", !show);
    if (!show) {
      this.courseSubjectShortcutsEl.innerHTML = "";
      return;
    }
    const rows = buildSubjectCountRows(lessons);
    const activeSubject = normalizeSubject(this.filter.subject);
    const allActive = !activeSubject;
    this.courseSubjectShortcutsEl.innerHTML = `
      <span class="course-subject-shortcuts-label">과목 바로 보기</span>
      <button type="button" class="course-subject-shortcut${allActive ? " active" : ""}" data-course-subject-filter="__all__">전체</button>
      ${rows.map((row) => `
        <button
          type="button"
          class="course-subject-shortcut${activeSubject === row.subject ? " active" : ""}"
          data-course-subject-filter="${escapeHtml(row.subject)}"
        >${escapeHtml(row.label)} ${escapeHtml(String(row.count))}</button>
      `).join("")}
    `;
  }

  render() {
    if (!this.grid) return;
    const lessons = this.filteredLessons();
    this.updateCourseCatalogSummary(lessons);
    this.updateFeaturedSeedQuickPresetButton();
    this.updateNumericTrackButton();
    this.updateNumericTrackResultsButton();
    this.updateNumericTrackResultSummaryCopyButton();
    this.updateNumericTrackResultTimelinePanel();
    this.updateNumericTrackResultCompareButton();
    this.updateBrowsePresetCopyButton();
    this.publishNumericTrackResultHistorySnapshot();
    this.grid.innerHTML = "";
    this.previewQueue = [];
    this.activePreviewCount = 0;
    if (this.detailLesson && !lessons.some((row) => String(row?.id ?? "") === String(this.detailLesson?.id ?? ""))) {
      this.hideLessonDetail();
    }

    if (!lessons.length) {
      const empty = document.createElement("div");
      empty.className = "hint";
      empty.textContent = this.activeTab === "search"
        ? "전체 검색 결과가 없습니다."
        : this.activeTab === "examples"
          ? "등록된 예제가 없습니다."
          : "조건에 맞는 교과가 없습니다.";
      this.grid.appendChild(empty);
      this.hideLessonDetail();
      return;
    }

    lessons.forEach((lesson, index) => {
      this.grid.appendChild(this.createLessonCard(lesson, index));
    });
    this.maybeAutoShowCourseDetail(lessons);
    this.updateSelectedLessonCard();
  }
}
