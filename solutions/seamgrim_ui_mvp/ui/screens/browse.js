import { showGlobalToast } from "../components/toast.js";
import { FEATURED_SEED_IDS } from "../featured_seed_catalog.js";
import { resolveLessonCardPreviewViewModel } from "../preview_session.js";
import { applyPreviewViewModelMetadata } from "../preview_view_model.js";
import {
  lessonHasPreviewDescriptor,
} from "../view_family_contract.js";

const QUALITY_BADGE = Object.freeze({
  recommended: { label: "★ 추천", cls: "badge-recommended" },
  reviewed: { label: "✓ 검수완료", cls: "badge-reviewed" },
  experimental: { label: "실험용", cls: "badge-experimental" },
});
const LEGACY_WARNING_CODE = "W_LEGACY_RANGE_COMMENT_DEPRECATED";
const DEFAULT_FEDERATED_API_CANDIDATES = Object.freeze(["/api/lessons/inventory"]);
const DEFAULT_FEDERATED_FILE_CANDIDATES = Object.freeze([]);
const RUN_UI_PREFS_STORAGE_KEY = "seamgrim.ui.run_prefs.v1";
const BROWSE_UI_PREFS_STORAGE_KEY = "seamgrim.ui.browse_prefs.v1";
const BROWSE_PRESET_QUERY_KEY = "browsePreset";

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

function normalizeGrade(grade) {
  return String(grade ?? "").trim().toLowerCase();
}

function normalizeQuality(quality) {
  const raw = String(quality ?? "").trim().toLowerCase();
  return QUALITY_BADGE[raw] ? raw : "experimental";
}

function normalizeSource(source) {
  return String(source ?? "").trim().toLowerCase();
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
    onOpenLegacyGuideExample,
    onOpenAdvanced,
    federatedApiCandidates,
    federatedFileCandidates,
  } = {}) {
    this.root = root;
    this.onLessonSelect = typeof onLessonSelect === "function" ? onLessonSelect : async () => {};
    this.onCreate = typeof onCreate === "function" ? onCreate : () => {};
    this.onOpenLegacyGuideExample =
      typeof onOpenLegacyGuideExample === "function" ? onOpenLegacyGuideExample : () => {};
    this.onOpenAdvanced = typeof onOpenAdvanced === "function" ? onOpenAdvanced : () => {};

    this.lessons = [];
    this.searchResults = [];
    this.federatedLoadState = "idle";
    this.activeTab = "official";
    this.federatedApiCandidates = normalizeCandidateList(
      federatedApiCandidates,
      DEFAULT_FEDERATED_API_CANDIDATES,
    );
    this.federatedFileCandidates = normalizeCandidateList(
      federatedFileCandidates,
      DEFAULT_FEDERATED_FILE_CANDIDATES,
    );

    this.filter = {
      grade: "",
      subject: "",
      quality: "",
      seedScope: "",
      runStatus: "",
      runLaunch: "",
      warningStatus: "",
      sort: "recent",
      query: "",
    };
    this.runUiPrefs = {
      lessons: {},
    };
    this.lastBrowsePresetId = "";
    this.lessonPreviewCache = new Map();
  }

  init() {
    this.tabButtons = Array.from(this.root.querySelectorAll(".browse-tab[data-tab]"));
    this.gradeSelect = this.root.querySelector("#filter-grade");
    this.subjectSelect = this.root.querySelector("#filter-subject");
    this.qualitySelect = this.root.querySelector("#filter-quality");
    this.seedScopeSelect = this.root.querySelector("#filter-seed-scope");
    this.runStatusSelect = this.root.querySelector("#filter-run-status");
    this.runLaunchSelect = this.root.querySelector("#filter-run-launch");
    this.warningStatusSelect = this.root.querySelector("#filter-warning-status");
    this.sortSelect = this.root.querySelector("#filter-sort");
    this.queryInput = this.root.querySelector("#filter-query");
    this.presetFeaturedSeedQuickRecentButton = this.root.querySelector("#btn-preset-featured-seed-quick-recent");
    this.copyBrowsePresetLinkButton = this.root.querySelector("#btn-copy-browse-preset-link");
    this.legacyGuideHintEl = this.root.querySelector("#browse-legacy-guide-hint");
    this.grid = this.root.querySelector("#lesson-card-grid");
    this.loadBrowsePrefs();

    this.root.querySelector("#btn-create")?.addEventListener("click", () => {
      this.onCreate();
    });

    this.root.querySelector("#btn-advanced-browse")?.addEventListener("click", () => {
      this.onOpenAdvanced();
    });
    this.presetFeaturedSeedQuickRecentButton?.addEventListener("click", () => {
      this.applyFeaturedSeedQuickRecentPreset();
    });
    this.copyBrowsePresetLinkButton?.addEventListener("click", () => {
      void this.handleCopyBrowsePresetLink();
    });

    this.tabButtons.forEach((button) => {
      button.addEventListener("click", async () => {
        const tab = String(button.dataset.tab ?? "official");
        this.activeTab = tab;
        this.tabButtons.forEach((item) => item.classList.toggle("active", item === button));
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
    this.lastBrowsePresetId = resolveBrowsePresetId(this.filter);
    this.updateFeaturedSeedQuickPresetButton();
    this.updateBrowsePresetCopyButton();
  }

  loadBrowsePrefs() {
    const parsed = readStorageJson(BROWSE_UI_PREFS_STORAGE_KEY, {});
    this.filter.sort = String(parsed?.sort ?? this.filter.sort ?? "recent").trim() || "recent";
    this.filter.seedScope = String(parsed?.seedScope ?? this.filter.seedScope ?? "").trim();
    this.filter.runLaunch = normalizeRunLaunchFilter(parsed?.runLaunch ?? this.filter.runLaunch ?? "");
    if (this.sortSelect) {
      this.sortSelect.value = this.filter.sort;
    }
    if (this.seedScopeSelect) {
      this.seedScopeSelect.value = this.filter.seedScope;
    }
    if (this.runLaunchSelect) {
      this.runLaunchSelect.value = this.filter.runLaunch;
    }
  }

  saveBrowsePrefs() {
    writeStorageJson(BROWSE_UI_PREFS_STORAGE_KEY, {
      sort: String(this.filter.sort ?? "recent"),
      seedScope: String(this.filter.seedScope ?? ""),
      runLaunch: String(this.filter.runLaunch ?? ""),
    });
    this.emitBrowsePresetChanged();
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
  }

  buildCardStateHint(lessonId) {
    const pref = this.runUiPrefs?.lessons?.[String(lessonId ?? "").trim()];
    const x = String(pref?.selectedXKey ?? "").trim();
    const y = String(pref?.selectedYKey ?? "").trim();
    if (x || y) {
      return `축선택 · x:${x || "-"} y:${y || "-"}`;
    }
    return "축선택 · 기본";
  }

  buildCardRunHint(lessonId) {
    const pref = this.runUiPrefs?.lessons?.[String(lessonId ?? "").trim()];
    if (!pref || typeof pref !== "object") return "최근 실행: 기록 없음";
    const kind = String(pref.lastRunKind ?? "").trim();
    const channels = Math.max(0, Number.isFinite(Number(pref.lastRunChannels)) ? Math.trunc(Number(pref.lastRunChannels)) : 0);
    const timeLabel = formatRecentTimeLabel(pref.lastRunAt);
    const hash = String(pref.lastRunHash ?? "").trim();
    const shortHash = hash && hash !== "-" ? hash.slice(0, 12) : "";
    let label = "";
    if (kind === "space2d") {
      label = `최근 실행: 보개 출력 · 채널=${channels}`;
    } else if (kind === "obs_only") {
      label = `최근 실행: 보개 없음 · 채널=${channels}`;
    } else if (kind === "empty") {
      label = "최근 실행: 출력 없음";
    } else if (kind === "error") {
      label = "최근 실행: 실패";
    } else {
      label = "최근 실행: 기록 없음";
    }
    if (shortHash) {
      label = `${label} · hash:${shortHash}`;
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
      return { label: "미실행", cls: "run-badge-none" };
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

  buildFeaturedSeedBadge(lesson) {
    if (!isFeaturedSeedLesson(lesson)) return null;
    return { label: "신규 seed", cls: "badge-featured-seed" };
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
    this.rebuildFilterOptions();
    this.render();
  }

  rebuildFilterOptions() {
    if (this.gradeSelect) {
      const grades = Array.from(new Set(this.lessons.map((lesson) => normalizeGrade(lesson.grade)).filter(Boolean))).sort();
      this.gradeSelect.innerHTML = ['<option value="">학년: 전체</option>', ...grades.map((grade) => `<option value="${grade}">${grade}</option>`)].join("");
      this.gradeSelect.value = this.filter.grade;
    }
    if (this.subjectSelect) {
      const subjects = Array.from(new Set(this.lessons.map((lesson) => normalizeSubject(lesson.subject)).filter(Boolean))).sort();
      this.subjectSelect.innerHTML = ['<option value="">과목: 전체</option>', ...subjects.map((subject) => `<option value="${subject}">${subject}</option>`)].join("");
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

  currentPool() {
    return this.activeTab === "search" ? this.searchResults : this.lessons;
  }

  filteredLessons() {
    const grade = normalizeGrade(this.filter.grade);
    const subject = normalizeSubject(this.filter.subject);
    const quality = normalizeQuality(this.filter.quality || "experimental");
    const hasQualityFilter = String(this.filter.quality ?? "").trim().length > 0;
    const seedScope = String(this.filter.seedScope ?? "").trim();
    const runStatus = String(this.filter.runStatus ?? "").trim();
    const runLaunch = String(this.filter.runLaunch ?? "").trim();
    const warningStatus = String(this.filter.warningStatus ?? "").trim();
    const sortMode = String(this.filter.sort ?? "recent").trim();
    const query = String(this.filter.query ?? "").trim().toLowerCase();

    const filtered = this.currentPool().filter((lesson) => {
      if (grade && normalizeGrade(lesson.grade) !== grade) return false;
      if (subject && normalizeSubject(lesson.subject) !== subject) return false;
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

    if (sortMode === "recent") {
      filtered.sort((a, b) => {
        const aMs = this.getLessonLastRunAtMs(a.id);
        const bMs = this.getLessonLastRunAtMs(b.id);
        if (aMs !== bMs) return bMs - aMs;
        return String(a.title ?? a.id ?? "").localeCompare(String(b.title ?? b.id ?? ""), "ko");
      });
    } else if (sortMode === "legacy_warning") {
      filtered.sort((a, b) => {
        const aCount = this.getLessonLegacyWarningCount(a);
        const bCount = this.getLessonLegacyWarningCount(b);
        if (aCount !== bCount) return bCount - aCount;
        return String(a.title ?? a.id ?? "").localeCompare(String(b.title ?? b.id ?? ""), "ko");
      });
    } else if (sortMode === "featured_seed") {
      filtered.sort((a, b) => {
        const aRank = this.getFeaturedSeedRank(a);
        const bRank = this.getFeaturedSeedRank(b);
        if (aRank !== bRank) return aRank - bRank;
        const aMs = this.getLessonLastRunAtMs(a.id);
        const bMs = this.getLessonLastRunAtMs(b.id);
        if (aMs !== bMs) return bMs - aMs;
        return String(a.title ?? a.id ?? "").localeCompare(String(b.title ?? b.id ?? ""), "ko");
      });
    } else if (sortMode === "featured_seed_quick_recent") {
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

  createLessonCard(lesson) {
    const qualityKey = QUALITY_BADGE[lesson.quality] ? lesson.quality : "experimental";
    const badge = QUALITY_BADGE[qualityKey];
    const stateHint = this.buildCardStateHint(lesson.id);
    const runHint = this.buildCardRunHint(lesson.id);
    const runBadge = this.buildRunBadge(lesson.id);
    const launchBadge = this.buildLaunchBadge(lesson.id);
    const legacyWarningBadge = this.buildLegacyWarningBadge(lesson);
    const featuredSeedBadge = this.buildFeaturedSeedBadge(lesson);
    const runHash = this.getLessonLastRunHash(lesson.id);
    const runHashShort = runHash ? runHash.slice(0, 12) : "";

    const card = document.createElement("button");
    card.type = "button";
    card.className = "lesson-card";
    card.dataset.lessonId = lesson.id;
    card.innerHTML = `
      <div class="card-top-badges">
        <span class="quality-badge ${badge.cls}">${badge.label}</span>
        ${featuredSeedBadge ? `<span class="quality-badge ${featuredSeedBadge.cls}">${featuredSeedBadge.label}</span>` : ""}
      </div>
      <div class="card-title">${lesson.title || lesson.id}</div>
      <div class="card-meta">${lesson.grade || "all"} · ${lesson.subject || "-"}</div>
      <div class="card-desc">${lesson.description || "설명 없음"}</div>
      <div class="card-badge-row">
        <span class="card-run-badge ${runBadge.cls}">${runBadge.label}</span>
        ${launchBadge ? `<span class="card-run-badge ${launchBadge.cls}">${launchBadge.label}</span>` : ""}
        ${legacyWarningBadge
          ? `<button type="button" class="card-warning-badge" title="${legacyWarningBadge.title} · 클릭해 전환 가이드 보기">${legacyWarningBadge.label}</button>`
          : ""}
        ${runHashShort
          ? `<button type="button" class="card-hash-copy" data-run-hash="${runHash}" title="state_hash 전체 복사">hash:${runHashShort}</button>`
          : ""}
      </div>
      <div class="card-state-hint">${stateHint}</div>
      <div class="card-run-hint">${runHint}</div>
    `;
    const shouldShowPreview = lessonHasPreviewDescriptor(lesson);
    if (shouldShowPreview && typeof document?.createElement === "function") {
      const preview = document.createElement("div");
      preview.className = "card-structure-preview hidden";
      preview.textContent = "미리보기 로딩 중…";
      card.appendChild(preview);
      void this.hydrateLessonPreview(preview, lesson);
    }
    card.querySelector(".card-hash-copy")?.addEventListener("click", async (event) => {
      event.preventDefault();
      event.stopPropagation();
      const hash = String(event.currentTarget?.dataset?.runHash ?? "").trim();
      if (!hash) return;
      const ok = await this.copyToClipboard(hash);
      showGlobalToast(ok ? "state_hash를 복사했습니다." : "state_hash 복사에 실패했습니다.", {
        kind: ok ? "success" : "error",
      });
    });
    card.querySelector(".card-warning-badge")?.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      this.showLegacyWarningGuide(lesson);
    });
    card.addEventListener("click", () => {
      void this.onLessonSelect(lesson);
    });
    return card;
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

  render() {
    if (!this.grid) return;
    const lessons = this.filteredLessons();
    this.updateFeaturedSeedQuickPresetButton();
    this.updateBrowsePresetCopyButton();
    this.grid.innerHTML = "";

    if (!lessons.length) {
      const empty = document.createElement("div");
      empty.className = "hint";
      empty.textContent = this.activeTab === "search"
        ? "연합 검색 결과가 없습니다."
        : "조건에 맞는 교과가 없습니다.";
      this.grid.appendChild(empty);
      return;
    }

    lessons.forEach((lesson) => {
      this.grid.appendChild(this.createLessonCard(lesson));
    });
  }
}
