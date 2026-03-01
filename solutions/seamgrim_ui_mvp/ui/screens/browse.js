const QUALITY_BADGE = Object.freeze({
  recommended: { label: "★ 추천", cls: "badge-recommended" },
  reviewed: { label: "✓ 검수완료", cls: "badge-reviewed" },
  experimental: { label: "실험용", cls: "badge-experimental" },
});
const DEFAULT_FEDERATED_API_CANDIDATES = Object.freeze(["/api/lessons/inventory"]);
const DEFAULT_FEDERATED_FILE_CANDIDATES = Object.freeze([]);
const RUN_UI_PREFS_STORAGE_KEY = "seamgrim.ui.run_prefs.v1";
const BROWSE_UI_PREFS_STORAGE_KEY = "seamgrim.ui.browse_prefs.v1";

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

export class BrowseScreen {
  constructor({
    root,
    onLessonSelect,
    onCreate,
    onOpenAdvanced,
    federatedApiCandidates,
    federatedFileCandidates,
  } = {}) {
    this.root = root;
    this.onLessonSelect = typeof onLessonSelect === "function" ? onLessonSelect : async () => {};
    this.onCreate = typeof onCreate === "function" ? onCreate : () => {};
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
      runStatus: "",
      sort: "recent",
      query: "",
    };
    this.runUiPrefs = {
      lessons: {},
    };
  }

  init() {
    this.tabButtons = Array.from(this.root.querySelectorAll(".browse-tab[data-tab]"));
    this.gradeSelect = this.root.querySelector("#filter-grade");
    this.subjectSelect = this.root.querySelector("#filter-subject");
    this.qualitySelect = this.root.querySelector("#filter-quality");
    this.runStatusSelect = this.root.querySelector("#filter-run-status");
    this.sortSelect = this.root.querySelector("#filter-sort");
    this.queryInput = this.root.querySelector("#filter-query");
    this.grid = this.root.querySelector("#lesson-card-grid");
    this.loadBrowsePrefs();

    this.root.querySelector("#btn-create")?.addEventListener("click", () => {
      this.onCreate();
    });

    this.root.querySelector("#btn-advanced-browse")?.addEventListener("click", () => {
      this.onOpenAdvanced();
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
    this.runStatusSelect?.addEventListener("change", () => {
      this.filter.runStatus = String(this.runStatusSelect.value ?? "");
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
  }

  loadBrowsePrefs() {
    const parsed = readStorageJson(BROWSE_UI_PREFS_STORAGE_KEY, {});
    this.filter.sort = String(parsed?.sort ?? this.filter.sort ?? "recent").trim() || "recent";
    if (this.sortSelect) {
      this.sortSelect.value = this.filter.sort;
    }
  }

  saveBrowsePrefs() {
    writeStorageJson(BROWSE_UI_PREFS_STORAGE_KEY, {
      sort: String(this.filter.sort ?? "recent"),
    });
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
          ddnCandidates: Array.from(new Set(ddnPath.map((item) => String(item)))),
          textCandidates: Array.from(new Set(textPath.map((item) => String(item)))),
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
    const runStatus = String(this.filter.runStatus ?? "").trim();
    const sortMode = String(this.filter.sort ?? "recent").trim();
    const query = String(this.filter.query ?? "").trim().toLowerCase();

    const filtered = this.currentPool().filter((lesson) => {
      if (grade && normalizeGrade(lesson.grade) !== grade) return false;
      if (subject && normalizeSubject(lesson.subject) !== subject) return false;
      if (hasQualityFilter && normalizeQuality(lesson.quality) !== quality) return false;
      if (runStatus) {
        const kind = this.getLessonLastRunKind(lesson.id);
        const isNone = kind === "none";
        const isError = kind === "error";
        const isSuccess = !isNone && !isError;
        if (runStatus === "none" && !isNone) return false;
        if (runStatus === "error" && !isError) return false;
        if (runStatus === "success" && !isSuccess) return false;
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
    }
    return filtered;
  }

  createLessonCard(lesson) {
    const qualityKey = QUALITY_BADGE[lesson.quality] ? lesson.quality : "experimental";
    const badge = QUALITY_BADGE[qualityKey];
    const stateHint = this.buildCardStateHint(lesson.id);
    const runHint = this.buildCardRunHint(lesson.id);
    const runBadge = this.buildRunBadge(lesson.id);
    const runHash = this.getLessonLastRunHash(lesson.id);
    const runHashShort = runHash ? runHash.slice(0, 12) : "";

    const card = document.createElement("button");
    card.type = "button";
    card.className = "lesson-card";
    card.dataset.lessonId = lesson.id;
    card.innerHTML = `
      <span class="quality-badge ${badge.cls}">${badge.label}</span>
      <div class="card-title">${lesson.title || lesson.id}</div>
      <div class="card-meta">${lesson.grade || "all"} · ${lesson.subject || "-"}</div>
      <div class="card-desc">${lesson.description || "설명 없음"}</div>
      <div class="card-badge-row">
        <span class="card-run-badge ${runBadge.cls}">${runBadge.label}</span>
        ${runHashShort
          ? `<button type="button" class="card-hash-copy" data-run-hash="${runHash}" title="state_hash 전체 복사">hash:${runHashShort}</button>`
          : ""}
      </div>
      <div class="card-state-hint">${stateHint}</div>
      <div class="card-run-hint">${runHint}</div>
    `;
    card.querySelector(".card-hash-copy")?.addEventListener("click", async (event) => {
      event.preventDefault();
      event.stopPropagation();
      const hash = String(event.currentTarget?.dataset?.runHash ?? "").trim();
      if (!hash) return;
      const ok = await this.copyToClipboard(hash);
      const button = event.currentTarget;
      if (!(button instanceof HTMLButtonElement)) return;
      const original = button.textContent;
      button.textContent = ok ? "복사됨" : "복사실패";
      window.setTimeout(() => {
        button.textContent = original;
      }, 900);
    });
    card.addEventListener("click", () => {
      void this.onLessonSelect(lesson);
    });
    return card;
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
