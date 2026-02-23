import {
  applyWasmLogicAndDispatchState,
  createManagedRafStepLoop,
  stepWasmClientParsed,
} from "../wasm_page_common.js";
import { extractObservationChannelsFromState, extractStructuredViewsFromState } from "../seamgrim_runtime_state.js";
import { applyControlValuesToDdnText } from "../components/control_parser.js";
import { Bogae } from "../components/bogae.js";
import { DotbogiPanel } from "../components/dotbogi.js";
import { SliderPanel } from "../components/slider_panel.js";
import { OverlayDescription } from "../components/overlay.js";

const RUN_UI_PREFS_STORAGE_KEY = "seamgrim.ui.run_prefs.v1";
const PRESET_SLOT_KEYS = Object.freeze(["1", "2", "3"]);

function finiteNumber(value) {
  const n = Number(value);
  return Number.isFinite(n) ? n : null;
}

function normalizeRange(range) {
  if (!range || typeof range !== "object") return null;
  const xMin = finiteNumber(range.x_min ?? range.xMin);
  const xMax = finiteNumber(range.x_max ?? range.xMax);
  const yMin = finiteNumber(range.y_min ?? range.yMin);
  const yMax = finiteNumber(range.y_max ?? range.yMax);
  if ([xMin, xMax, yMin, yMax].some((value) => value === null)) return null;
  if (xMax <= xMin || yMax <= yMin) return null;
  return { x_min: xMin, x_max: xMax, y_min: yMin, y_max: yMax };
}

function formatRangeValue(value) {
  const n = finiteNumber(value);
  if (n === null) return "";
  return String(Math.round(n * 1_000_000) / 1_000_000);
}

function normalizePresetSlot(value) {
  const key = String(value ?? "").trim();
  return PRESET_SLOT_KEYS.includes(key) ? key : PRESET_SLOT_KEYS[0];
}

function createEmptyRangeSlotMap() {
  return {
    1: null,
    2: null,
    3: null,
  };
}

function normalizeRangeSlotMap(raw) {
  const base = createEmptyRangeSlotMap();
  if (!raw || typeof raw !== "object") return base;
  PRESET_SLOT_KEYS.forEach((slot) => {
    base[slot] = normalizeRange(raw[slot] ?? raw[Number(slot)]);
  });
  return base;
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

export class RunScreen {
  constructor({ root, wasmState, onBack, onEditDdn, onOpenAdvanced } = {}) {
    this.root = root;
    this.wasmState = wasmState;
    this.onBack = typeof onBack === "function" ? onBack : () => {};
    this.onEditDdn = typeof onEditDdn === "function" ? onEditDdn : () => {};
    this.onOpenAdvanced = typeof onOpenAdvanced === "function" ? onOpenAdvanced : () => {};

    this.lesson = null;
    this.baseDdn = "";
    this.currentDdn = "";
    this.lastState = null;

    this.loopActive = false;
    this.screenVisible = false;
    this.loop = null;
    this.viewPanStep = 0.08;
    this.lastRuntimeStatus = "";
    this.lastRuntimeHash = "-";
    this.lastRuntimeSnapshotKey = "";
    this.boundKeyHandler = (event) => {
      this.handleViewHotkeys(event);
    };
    this.savedBogaeRanges = createEmptyRangeSlotMap();
    this.savedGraphAxes = createEmptyRangeSlotMap();
    this.selectedBogaePresetSlot = PRESET_SLOT_KEYS[0];
    this.selectedGraphPresetSlot = PRESET_SLOT_KEYS[0];
    this.uiPrefs = {
      axisLock: true,
      lessons: {},
    };
  }

  init() {
    this.titleEl = this.root.querySelector("#run-lesson-title");
    this.lastSummaryEl = this.root.querySelector("#run-last-summary");
    this.statusEl = this.root.querySelector("#run-status");
    this.hashEl = this.root.querySelector("#run-hash");

    this.uiPrefs = {
      axisLock: true,
      lessons: {},
      ...readStorageJson(RUN_UI_PREFS_STORAGE_KEY, {}),
    };
    if (!this.uiPrefs.lessons || typeof this.uiPrefs.lessons !== "object") {
      this.uiPrefs.lessons = {};
    }

    this.bogae = new Bogae({
      canvas: this.root.querySelector("#canvas-bogae"),
      onRangeChange: () => {
        this.syncRangeInputs();
      },
    });
    this.dotbogi = new DotbogiPanel({
      graphCanvas: this.root.querySelector("#canvas-graph"),
      tableEl: this.root.querySelector("#data-table"),
      textEl: this.root.querySelector("#text-content"),
      xAxisSelect: this.root.querySelector("#select-x-axis"),
      yAxisSelect: this.root.querySelector("#select-y-axis"),
      tabButtons: Array.from(this.root.querySelectorAll(".panel-tab")),
      graphResetBtn: this.root.querySelector("#btn-graph-reset"),
      onAxisChange: () => {
        this.syncRangeInputs();
      },
    });
    this.dotbogi.setPreferredAxisLock(Boolean(this.uiPrefs.axisLock ?? true));
    this.overlay = new OverlayDescription(this.root.querySelector("#overlay-description"));
    this.bogaeRangeInputs = {
      xMin: this.root.querySelector("#bogae-x-min"),
      xMax: this.root.querySelector("#bogae-x-max"),
      yMin: this.root.querySelector("#bogae-y-min"),
      yMax: this.root.querySelector("#bogae-y-max"),
    };
    this.graphRangeInputs = {
      xMin: this.root.querySelector("#graph-x-min"),
      xMax: this.root.querySelector("#graph-x-max"),
      yMin: this.root.querySelector("#graph-y-min"),
      yMax: this.root.querySelector("#graph-y-max"),
    };
    this.bogaePresetSlotSelect = this.root.querySelector("#bogae-preset-slot");
    this.graphPresetSlotSelect = this.root.querySelector("#graph-preset-slot");

    this.sliderPanel = new SliderPanel({
      container: this.root.querySelector("#slider-list"),
      statusEl: this.root.querySelector("#slider-status"),
      onCommit: () => {
        void this.restart();
      },
    });

    this.root.querySelector("#btn-back-run")?.addEventListener("click", () => {
      this.onBack();
    });

    this.root.querySelector("#btn-edit-ddn")?.addEventListener("click", () => {
      this.onEditDdn({
        ddnText: this.baseDdn,
        title: this.lesson?.title || this.lesson?.id || "DDN 보기",
      });
    });

    this.root.querySelector("#btn-overlay-toggle")?.addEventListener("click", () => {
      const visible = this.overlay.toggle();
      this.setStatus(visible ? "실행 상태: 설명 오버레이 ON" : "실행 상태: 설명 오버레이 OFF");
    });

    this.root.querySelector("#btn-advanced-run")?.addEventListener("click", () => {
      this.onOpenAdvanced();
    });

    this.root.querySelector("#btn-restart")?.addEventListener("click", () => {
      void this.restart();
    });

    this.root.querySelector("#btn-zoom-in")?.addEventListener("click", () => {
      this.bogae.zoomIn();
    });
    this.root.querySelector("#btn-zoom-out")?.addEventListener("click", () => {
      this.bogae.zoomOut();
    });
    this.root.querySelector("#btn-zoom-reset")?.addEventListener("click", () => {
      this.bogae.resetView();
    });
    this.root.querySelector("#btn-bogae-range-apply")?.addEventListener("click", () => {
      this.applyBogaeRangeFromInputs();
    });
    this.root.querySelector("#btn-bogae-range-reset")?.addEventListener("click", () => {
      this.bogae.resetView();
      this.setStatus("실행 상태: 보개 범위를 자동 맞춤으로 되돌렸습니다.");
      this.syncRangeInputs({ force: true });
    });
    this.root.querySelector("#btn-bogae-range-save")?.addEventListener("click", () => {
      this.saveBogaeRangePreset();
    });
    this.root.querySelector("#btn-bogae-range-load")?.addEventListener("click", () => {
      this.loadBogaeRangePreset();
    });
    this.bogaePresetSlotSelect?.addEventListener("change", () => {
      this.selectedBogaePresetSlot = normalizePresetSlot(this.bogaePresetSlotSelect.value);
      this.bogaePresetSlotSelect.value = this.selectedBogaePresetSlot;
      this.saveCurrentLessonUiPrefs();
    });
    this.root.querySelector("#btn-graph-range-apply")?.addEventListener("click", () => {
      this.applyGraphAxisFromInputs();
    });
    this.root.querySelector("#btn-graph-range-reset")?.addEventListener("click", () => {
      this.dotbogi.resetAxis();
      this.setStatus("실행 상태: 그래프 축을 자동 범위로 되돌렸습니다.");
      this.syncRangeInputs({ force: true });
    });
    this.root.querySelector("#btn-graph-range-save")?.addEventListener("click", () => {
      this.saveGraphAxisPreset();
    });
    this.root.querySelector("#btn-graph-range-load")?.addEventListener("click", () => {
      this.loadGraphAxisPreset();
    });
    this.graphPresetSlotSelect?.addEventListener("change", () => {
      this.selectedGraphPresetSlot = normalizePresetSlot(this.graphPresetSlotSelect.value);
      this.graphPresetSlotSelect.value = this.selectedGraphPresetSlot;
      this.saveCurrentLessonUiPrefs();
    });
    this.root.querySelector("#btn-axis-lock")?.addEventListener("click", () => {
      this.setAxisLock(!this.dotbogi.isPreferredAxisLock(), { persist: true, withStatus: true });
    });
    this.root.querySelector("#select-x-axis")?.addEventListener("change", () => {
      this.saveCurrentLessonUiPrefs();
    });
    this.root.querySelector("#select-y-axis")?.addEventListener("change", () => {
      this.saveCurrentLessonUiPrefs();
    });

    this.loop = createManagedRafStepLoop({
      getFps: () => Math.max(1, Number(this.wasmState?.fpsLimit ?? 30) || 30),
      isActive: () => this.loopActive,
      setActive: (active) => {
        this.loopActive = Boolean(active);
      },
      onStep: () => {
        this.stepFrame();
      },
      onError: (err) => {
        this.setStatus(`실행 상태: 루프 오류 (${String(err?.message ?? err)})`);
      },
    });
    window.addEventListener("keydown", this.boundKeyHandler);
    this.syncPresetSlotSelectors();
    this.updateAxisLockButton();
    this.syncRangeInputs({ force: true });
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

  updateAxisLockButton() {
    const button = this.root?.querySelector("#btn-axis-lock");
    if (!button) return;
    const locked = this.dotbogi?.isPreferredAxisLock?.() ?? true;
    button.textContent = `기본축 고정: ${locked ? "ON" : "OFF"}`;
    button.classList.toggle("active", locked);
    button.title = locked
      ? "교과 기본 x/y축을 고정합니다."
      : "x/y축을 수동으로 선택할 수 있습니다.";
  }

  setAxisLock(locked, { persist = true, withStatus = false } = {}) {
    const next = Boolean(locked);
    this.dotbogi?.setPreferredAxisLock(next);
    this.updateAxisLockButton();
    if (persist) {
      this.uiPrefs.axisLock = next;
      this.saveCurrentLessonUiPrefs();
    }
    if (withStatus) {
      this.setStatus(next ? "실행 상태: 기본축 고정 ON" : "실행 상태: 기본축 고정 OFF");
    }
  }

  restoreLessonUiPrefs(lessonId) {
    const pref = this.getLessonUiPref(lessonId, { create: false });
    this.savedBogaeRanges = normalizeRangeSlotMap(pref?.savedBogaeRanges);
    this.savedGraphAxes = normalizeRangeSlotMap(pref?.savedGraphAxes);
    if (!this.savedBogaeRanges[PRESET_SLOT_KEYS[0]]) {
      this.savedBogaeRanges[PRESET_SLOT_KEYS[0]] = normalizeRange(pref?.savedBogaeRange);
    }
    if (!this.savedGraphAxes[PRESET_SLOT_KEYS[0]]) {
      this.savedGraphAxes[PRESET_SLOT_KEYS[0]] = normalizeRange(pref?.savedGraphAxis);
    }

    this.selectedBogaePresetSlot = normalizePresetSlot(pref?.selectedBogaePresetSlot);
    this.selectedGraphPresetSlot = normalizePresetSlot(pref?.selectedGraphPresetSlot);
    this.syncPresetSlotSelectors();

    if (!this.dotbogi?.isPreferredAxisLock()) {
      this.dotbogi.setSelectedAxes({
        xKey: String(pref?.selectedXKey ?? ""),
        yKey: String(pref?.selectedYKey ?? ""),
      });
    }
  }

  saveCurrentLessonUiPrefs() {
    const lessonId = String(this.lesson?.id ?? "").trim();
    if (!lessonId) return;
    const pref = this.getLessonUiPref(lessonId, { create: true });
    const selected = this.dotbogi?.getSelectedAxes?.() ?? {};
    pref.selectedXKey = String(selected.xKey ?? "");
    pref.selectedYKey = String(selected.yKey ?? "");
    pref.selectedBogaePresetSlot = normalizePresetSlot(this.selectedBogaePresetSlot);
    pref.selectedGraphPresetSlot = normalizePresetSlot(this.selectedGraphPresetSlot);
    pref.savedBogaeRanges = normalizeRangeSlotMap(this.savedBogaeRanges);
    pref.savedGraphAxes = normalizeRangeSlotMap(this.savedGraphAxes);
    pref.savedBogaeRange = normalizeRange(this.savedBogaeRanges[PRESET_SLOT_KEYS[0]]);
    pref.savedGraphAxis = normalizeRange(this.savedGraphAxes[PRESET_SLOT_KEYS[0]]);
    this.persistUiPrefs();
  }

  syncPresetSlotSelectors() {
    if (this.bogaePresetSlotSelect) {
      this.bogaePresetSlotSelect.value = normalizePresetSlot(this.selectedBogaePresetSlot);
      this.selectedBogaePresetSlot = this.bogaePresetSlotSelect.value;
    }
    if (this.graphPresetSlotSelect) {
      this.graphPresetSlotSelect.value = normalizePresetSlot(this.selectedGraphPresetSlot);
      this.selectedGraphPresetSlot = this.graphPresetSlotSelect.value;
    }
  }

  isInputFocused(input) {
    return Boolean(input && document.activeElement === input);
  }

  readRangeInputs(inputs) {
    const xMin = finiteNumber(inputs?.xMin?.value);
    const xMax = finiteNumber(inputs?.xMax?.value);
    const yMin = finiteNumber(inputs?.yMin?.value);
    const yMax = finiteNumber(inputs?.yMax?.value);
    if ([xMin, xMax, yMin, yMax].some((value) => value === null)) return null;
    return normalizeRange({ x_min: xMin, x_max: xMax, y_min: yMin, y_max: yMax });
  }

  writeRangeInputs(inputs, range, { force = false } = {}) {
    const normalized = normalizeRange(range);
    const rows = [
      [inputs?.xMin, normalized?.x_min],
      [inputs?.xMax, normalized?.x_max],
      [inputs?.yMin, normalized?.y_min],
      [inputs?.yMax, normalized?.y_max],
    ];
    rows.forEach(([input, value]) => {
      if (!input) return;
      if (!force && this.isInputFocused(input)) return;
      input.value = normalized ? formatRangeValue(value) : "";
    });
  }

  syncRangeInputs({ force = false } = {}) {
    this.writeRangeInputs(this.bogaeRangeInputs, this.bogae?.getCurrentRange(), { force });
    this.writeRangeInputs(this.graphRangeInputs, this.dotbogi?.getCurrentAxis(), { force });
  }

  applyBogaeRangeFromInputs() {
    const range = this.readRangeInputs(this.bogaeRangeInputs);
    if (!range) {
      this.setStatus("실행 상태: 보개 범위 값이 유효하지 않습니다. (x_max>x_min, y_max>y_min)");
      return;
    }
    const ok = this.bogae.setRange(range);
    if (!ok) {
      this.setStatus("실행 상태: 보개 범위 적용 실패");
      return;
    }
    this.setStatus("실행 상태: 보개 범위를 적용했습니다.");
    this.syncRangeInputs({ force: true });
  }

  applyGraphAxisFromInputs() {
    const axis = this.readRangeInputs(this.graphRangeInputs);
    if (!axis) {
      this.setStatus("실행 상태: 그래프 축 값이 유효하지 않습니다. (x_max>x_min, y_max>y_min)");
      return;
    }
    const ok = this.dotbogi.setAxis(axis);
    if (!ok) {
      this.setStatus("실행 상태: 그래프 축 적용 실패");
      return;
    }
    this.setStatus("실행 상태: 그래프 축 범위를 적용했습니다.");
    this.syncRangeInputs({ force: true });
  }

  saveBogaeRangePreset() {
    const current = normalizeRange(this.bogae?.getCurrentRange());
    if (!current) {
      this.setStatus("실행 상태: 저장할 보개 범위가 없습니다.");
      return;
    }
    const slot = normalizePresetSlot(this.selectedBogaePresetSlot);
    this.savedBogaeRanges[slot] = { ...current };
    this.saveCurrentLessonUiPrefs();
    this.setStatus(`실행 상태: 보개 범위를 슬롯 ${slot}에 저장했습니다.`);
  }

  loadBogaeRangePreset() {
    const slot = normalizePresetSlot(this.selectedBogaePresetSlot);
    const saved = normalizeRange(this.savedBogaeRanges[slot]);
    if (!saved) {
      this.setStatus(`실행 상태: 슬롯 ${slot}에 저장된 보개 범위가 없습니다.`);
      return;
    }
    const ok = this.bogae.setRange(saved);
    if (!ok) {
      this.setStatus("실행 상태: 저장된 보개 범위 적용 실패");
      return;
    }
    this.setStatus(`실행 상태: 슬롯 ${slot} 보개 범위를 불러왔습니다.`);
    this.saveCurrentLessonUiPrefs();
    this.syncRangeInputs({ force: true });
  }

  saveGraphAxisPreset() {
    const current = normalizeRange(this.dotbogi?.getCurrentAxis());
    if (!current) {
      this.setStatus("실행 상태: 저장할 그래프 축 범위가 없습니다.");
      return;
    }
    const slot = normalizePresetSlot(this.selectedGraphPresetSlot);
    this.savedGraphAxes[slot] = { ...current };
    this.saveCurrentLessonUiPrefs();
    this.setStatus(`실행 상태: 그래프 축 범위를 슬롯 ${slot}에 저장했습니다.`);
  }

  loadGraphAxisPreset() {
    const slot = normalizePresetSlot(this.selectedGraphPresetSlot);
    const saved = normalizeRange(this.savedGraphAxes[slot]);
    if (!saved) {
      this.setStatus(`실행 상태: 슬롯 ${slot}에 저장된 그래프 축 범위가 없습니다.`);
      return;
    }
    const ok = this.dotbogi.setAxis(saved);
    if (!ok) {
      this.setStatus("실행 상태: 저장된 그래프 축 범위 적용 실패");
      return;
    }
    this.setStatus(`실행 상태: 슬롯 ${slot} 그래프 축 범위를 불러왔습니다.`);
    this.saveCurrentLessonUiPrefs();
    this.syncRangeInputs({ force: true });
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

  setStatus(message) {
    if (!this.statusEl) return;
    this.statusEl.textContent = String(message ?? "");
  }

  setHash(hashText) {
    this.lastRuntimeHash = String(hashText ?? "-");
    if (!this.hashEl) return;
    this.hashEl.textContent = `state_hash: ${String(hashText ?? "-")}`;
  }

  formatRecentTimeLabel(isoText) {
    const ms = Date.parse(String(isoText ?? ""));
    if (!Number.isFinite(ms)) return "";
    const d = new Date(ms);
    const month = String(d.getMonth() + 1);
    const day = String(d.getDate());
    const hour = String(d.getHours()).padStart(2, "0");
    const minute = String(d.getMinutes()).padStart(2, "0");
    return `${month}/${day} ${hour}:${minute}`;
  }

  buildRunSummaryText(pref) {
    if (!pref || typeof pref !== "object") return "최근 실행: 기록 없음";
    const kind = String(pref.lastRunKind ?? "").trim();
    const channels = Math.max(0, Number.isFinite(Number(pref.lastRunChannels)) ? Math.trunc(Number(pref.lastRunChannels)) : 0);
    const timeLabel = this.formatRecentTimeLabel(pref.lastRunAt);
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
    } else if (String(pref.lastRunStatus ?? "").trim()) {
      label = `최근 실행: ${String(pref.lastRunStatus).trim()}`;
    } else {
      label = "최근 실행: 기록 없음";
    }
    if (shortHash) {
      label = `${label} · hash:${shortHash}`;
    }
    return timeLabel ? `${label} · ${timeLabel}` : label;
  }

  updateRunSummaryFromPrefs() {
    if (!this.lastSummaryEl) return;
    const lessonId = String(this.lesson?.id ?? "").trim();
    const pref = lessonId ? this.getLessonUiPref(lessonId, { create: false }) : null;
    this.lastSummaryEl.textContent = this.buildRunSummaryText(pref);
  }

  loadLesson(lesson) {
    this.lesson = lesson;
    this.baseDdn = String(lesson?.ddnText ?? "");
    this.currentDdn = this.baseDdn;
    this.lastRuntimeStatus = "";
    this.lastRuntimeHash = "-";
    this.lastRuntimeSnapshotKey = "";

    if (this.titleEl) {
      this.titleEl.textContent = lesson?.title || lesson?.id || "-";
    }

    this.overlay.setContent(lesson?.textMd || "");
    this.overlay.hide();
    const parsed = this.sliderPanel.parseFromDdn(this.baseDdn, { preserveValues: false });
    this.dotbogi.setSeedKeys(parsed.axisKeys);
    this.dotbogi.setPreferredXKey(parsed.defaultXAxisKey);
    this.dotbogi.setPreferredYKey(parsed.defaultAxisKey);
    this.dotbogi.clearTimeline();
    this.restoreLessonUiPrefs(lesson?.id);
    this.dotbogi.setText(lesson?.textMd || "");
    this.bogae.resetView();
    this.updateAxisLockButton();
    this.saveCurrentLessonUiPrefs();
    this.updateRunSummaryFromPrefs();
    this.syncRangeInputs({ force: true });
    this.setHash("-");
    void this.restart();
  }

  getEffectiveDdn() {
    return applyControlValuesToDdnText(this.baseDdn, this.sliderPanel.getValues());
  }

  async restart() {
    const ddnText = this.getEffectiveDdn();
    this.currentDdn = ddnText;
    this.dotbogi.clearTimeline();
    this.setStatus("실행 상태: WASM 로딩 중...");

    try {
      const ensureWasm = (source) => this.wasmState.loader.ensure(source);
      const tryRunWithMode = async (mode) =>
        applyWasmLogicAndDispatchState({
          sourceText: ddnText,
          ensureWasm,
          mode,
        });

      let result;
      const preferredMode = String(this.wasmState?.langMode ?? "strict");
      try {
        result = await tryRunWithMode(preferredMode);
      } catch (err) {
        if (preferredMode === "compat") throw err;
        result = await tryRunWithMode("compat");
        this.wasmState.langMode = "compat";
      }

      this.wasmState.client = result.client;
      this.lastState = result.state;
      this.consumeState(result.state);
      this.startLoop();
      const hash = typeof result.client?.getStateHash === "function" ? result.client.getStateHash() : "-";
      this.setHash(hash);
      this.setStatus("실행 상태: WASM 실행 중");
      const observation = extractObservationChannelsFromState(result.state);
      const views = extractStructuredViewsFromState(result.state, { preferPatch: false });
      this.updateRuntimeStatus({ observation, views, force: true });
      this.syncRangeInputs({ force: true });
      return true;
    } catch (err) {
      this.stopLoop();
      this.setStatus(`실행 상태: WASM 실행 실패 (${String(err?.message ?? err)})`);
      this.setHash("-");
      this.saveRuntimeSnapshot({ kind: "error", channels: 0, hasSpace2d: false });
      return false;
    }
  }

  startLoop() {
    if (!this.loop) return;
    if (!this.screenVisible) return;
    this.loop.start();
  }

  stopLoop() {
    if (!this.loop) return;
    this.loop.stop();
  }

  stepFrame() {
    if (!this.screenVisible) return;
    const client = this.wasmState?.client;
    if (!client) return;
    const fps = Math.max(1, Number(this.wasmState?.fpsLimit ?? 30) || 30);
    const dtMax = Number(this.wasmState?.dtMax ?? 0.1);
    const dt = Math.max(0, Math.min(1 / fps, Number.isFinite(dtMax) ? dtMax : 0.1));

    try {
      const stepped = stepWasmClientParsed({
        client,
        input: { dt, keys: 0, lastKey: "", px: 0, py: 0 },
      });
      this.lastState = stepped.state;
      this.consumeState(stepped.state);
      const hash = typeof client.getStateHash === "function" ? client.getStateHash() : "-";
      this.setHash(hash);
    } catch (err) {
      this.stopLoop();
      this.setStatus(`실행 상태: 스텝 실패 (${String(err?.message ?? err)})`);
    }
  }

  consumeState(stateJson) {
    if (!stateJson) return;
    const observation = extractObservationChannelsFromState(stateJson);
    const views = extractStructuredViewsFromState(stateJson, { preferPatch: false });

    this.dotbogi.appendObservation(observation);
    if (views?.table && typeof views.table === "object") {
      this.dotbogi.renderTable(views.table);
    }
    if (views?.text && typeof views.text === "object") {
      const textBody = Array.isArray(views.text.lines)
        ? views.text.lines.join("\n")
        : String(views.text.markdown ?? views.text.text ?? "");
      if (textBody.trim()) {
        this.dotbogi.setText(textBody);
      }
    }

    this.bogae.render(views?.space2d ?? null);
    this.updateRuntimeStatus({ observation, views });
    this.syncRangeInputs();
  }

  setScreenVisible(visible) {
    const next = Boolean(visible);
    if (this.screenVisible === next) return;
    this.screenVisible = next;
    if (!next) {
      this.stopLoop();
      return;
    }
    this.startLoop();
    if (this.lastState) {
      const observation = extractObservationChannelsFromState(this.lastState);
      const views = extractStructuredViewsFromState(this.lastState, { preferPatch: false });
      this.updateRuntimeStatus({ observation, views, force: true });
    }
  }

  hasSpace2dDrawable(space2d) {
    if (!space2d || typeof space2d !== "object") return false;
    const shapes = Array.isArray(space2d.shapes) ? space2d.shapes.length : 0;
    const points = Array.isArray(space2d.points) ? space2d.points.length : 0;
    const drawlist = Array.isArray(space2d.drawlist) ? space2d.drawlist.length : 0;
    return shapes > 0 || points > 0 || drawlist > 0;
  }

  updateRuntimeStatus({ observation = null, views = null, force = false } = {}) {
    const channels = Array.isArray(observation?.channels) ? observation.channels.length : 0;
    const hasSpace2d = this.hasSpace2dDrawable(views?.space2d);
    let next = "";
    let kind = "empty";
    if (!hasSpace2d && channels <= 0) {
      next = "실행 상태: WASM 실행됨 · 출력 없음 (채널=0 · 보개 없음)";
      kind = "empty";
    } else if (!hasSpace2d) {
      next = `실행 상태: WASM 실행됨 · 보개 출력 없음 (채널=${channels} · 그래프/표 확인)`;
      kind = "obs_only";
    } else {
      next = `실행 상태: WASM 실행 중 · 보개 출력 · 채널=${channels}`;
      kind = "space2d";
    }
    if (!force && next === this.lastRuntimeStatus) return;
    this.lastRuntimeStatus = next;
    this.setStatus(next);
    this.saveRuntimeSnapshot({ kind, channels, hasSpace2d });
  }

  saveRuntimeSnapshot({ kind = "empty", channels = 0, hasSpace2d = false } = {}) {
    const lessonId = String(this.lesson?.id ?? "").trim();
    if (!lessonId) return;
    const normalizedKind = String(kind ?? "empty").trim() || "empty";
    const normalizedChannels = Math.max(0, Number.isFinite(Number(channels)) ? Math.trunc(Number(channels)) : 0);
    const snapshotKey = `${lessonId}:${normalizedKind}:${normalizedChannels}:${hasSpace2d ? 1 : 0}`;
    if (snapshotKey === this.lastRuntimeSnapshotKey) return;
    this.lastRuntimeSnapshotKey = snapshotKey;

    const pref = this.getLessonUiPref(lessonId, { create: true });
    pref.lastRunKind = normalizedKind;
    pref.lastRunChannels = normalizedChannels;
    pref.lastRunHasSpace2d = Boolean(hasSpace2d);
    pref.lastRunAt = new Date().toISOString();
    pref.lastRunStatus = String(this.lastRuntimeStatus ?? "");
    pref.lastRunHash = String(this.lastRuntimeHash ?? "-");
    this.persistUiPrefs();
    this.updateRunSummaryFromPrefs();
  }
}
