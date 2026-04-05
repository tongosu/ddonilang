/**
 * Playground — 또니랑 언어 인터랙티브 실행 환경
 *
 * URL 형식: /play.html#v=1&src=<base64-encoded DDN>
 *
 * 구현된 결정:
 *   D2 — 탭 전환 시 동기화 (텍스트↔블록). 여기서는 에디터 기준:
 *        dirty-check, beforeunload 경고, localStorage 자동저장, 초안 복원 배너
 *   D4 — 수식 입력 tip chip (에디터 포커스 시 표시)
 *   D5 — 단위 참조 (ddn.units.json 기반, tip chip 하단)
 *   D7 — 설정 패널: expert_mode, show_tip, fps, mirror_full (seamgrim.view_config)
 */

import {
  createWasmLoader,
  applyWasmLogicFromSource,
  stepWasmClientParsed,
  readWasmClientParseWarnings,
  createManagedRafStepLoop,
} from "./wasm_page_common.js";
import {
  normalizePlayTab,
  resolvePlayActiveTab,
  resolvePlayTabVisibility,
} from "./play_output_contract.js";
import {
  resolvePlayLaunchRequest,
} from "./play_source_contract.js";
import { createWasmCanon } from "./runtime/index.js";
import { preprocessDdnText } from "./runtime/ddn_preprocess.js";
import {
  buildGraphPreviewHtml,
} from "./components/graph_preview.js";
import { buildStructurePreviewHtml } from "./components/structure_preview.js";
import {
  buildTablePreviewHtml,
} from "./components/table_preview.js";
import { Bogae } from "./components/bogae.js";
import { markdownToHtml } from "./components/markdown.js";
import {
  buildFamilyPreviewResult,
  buildPreviewResultCollection,
} from "./preview_result_contract.js";
import {
  applyPreviewViewModelMetadata,
  buildPreviewViewModel,
} from "./preview_view_model.js";
import { parseLessonDdnMetaHeader } from "./lesson_loader_contract.js";
import { normalizeViewFamilyList } from "./view_family_contract.js";
import {
  flattenMirrorInputEntries,
  normalizeWasmStatePayload,
  extractObservationChannelsFromState,
  extractObservationOutputRowsFromState,
  extractStructuredViewsFromState,
} from "./seamgrim_runtime_state.js";

// ─── 상수 ─────────────────────────────────────────────────────────────────────

const STORAGE_KEY_DRAFT   = "seamgrim.block_editor.draft";
const STORAGE_KEY_CONFIG  = "seamgrim.view_config";
const DRAFT_DEBOUNCE_MS   = 1000;
const MIRROR_DEFAULT_MAX  = 20;
const GRAMMAR_TEXT_MAX    = 120000;

// ─── D5: 단위 목록 (ddn.units.json 기반 hardcode) ─────────────────────────────

const DDN_UNITS = {
  길이:   ["@m", "@km", "@cm", "@mm", "@inch", "@ft"],
  시간:   ["@s", "@ms", "@min", "@h"],
  질량:   ["@kg", "@g"],
  온도:   ["@K", "@C", "@F"],
  속도:   ["@m/s", "@kmh"],
  가속도: ["@m/s^2"],
  각도:   ["@rad"],
  통화:   ["@KRW", "@USD"],
};

// ─── 예제 코드 ────────────────────────────────────────────────────────────────

const EXAMPLES = {
  hello: `(매마디)마다 {
  보임 {
    안녕: "세계".
  }.
}.
`,
  counter: `채비 {
  시작값:수 <- 0.
  최대:수 <- 10.
}.

(시작)할때 {
  n <- 시작값.
}.

(매마디)마다 {
  n <- n + 1.
  보임 {
    n: n.
  }.
}.
`,
  fibonacci: `(시작)할때 {
  a <- 0.
  b <- 1.
}.

(매마디)마다 {
  다음 <- a + b.
  a <- b.
  b <- 다음.
  보임 {
    피보나치: a.
  }.
}.
`,
  stream_slice: `(시작)할때 {
  기록 <- (5) 흐름만들기.
  n <- 0.
}.

(매마디)마다 {
  n <- n + 1.
  기록 <- (기록, n) 흐름넣기.
  최근셋 <- (기록, 3) 흐름잘라보기.
  보임 {
    n: n.
    최근셋: 최근셋.
  }.
}.
`,
  pendulum: `채비 {
  g:수 <- 9.8.
  L:수 <- 1.0.
  dt:수 <- 0.02.
}.

(시작)할때 {
  theta <- 0.5.
  omega <- 0.
  t <- 0.
}.

(매마디)마다 {
  alpha <- (0 - (g / L)) * theta.
  omega <- omega + alpha * dt.
  theta <- theta + omega * dt.
  t <- t + dt.
  보임 {
    t: t.
    theta: theta.
  }.
}.
`,
  prep_slider: `채비 {
  k:수 <- (1.0) 매김 { 범위: 0.1..5. 간격: 0.1. }.
  b:수 <- (0.1) 매김 { 범위: 0..1. 간격: 0.01. }.
  x0:수 <- (1.0) 매김 { 범위: -3..3. 간격: 0.1. }.
  dt:수 <- 0.02.
}.

(시작)할때 {
  x <- x0.
  v <- 0.
  t <- 0.
}.

(매마디)마다 {
  a <- (0 - k) * x - b * v.
  v <- v + a * dt.
  x <- x + v * dt.
  t <- t + dt.
  보임 {
    t: t.
    x: x.
    v: v.
  }.
}.
`,
  shape_overlay: `(시작)할때 {
  t <- 0.
}.

(매마디)마다 {
  t <- t + 0.1.
  x <- t.
  y <- 0.3.

  모양 {
    선(0, 0, x, y, 색="#4fc3f7", 굵기=0.03).
    원(x, y, 0.08, 색="#22c55e", 선색="#16a34a", 굵기=0.02).
    점(x, y, 크기=0.06, 색="#facc15").
  }.

  보개마당 {
    토막("hud") {
      자막(글="겹보기 자막", 자리=(0.1, 0.9)).
    }.
  }.

  보임 {
    t: t.
    x: x.
    y: y.
  }.
}.
`,
};

// ─── URL 해시 유틸 ────────────────────────────────────────────────────────────

function encodeSourceToHash(src) {
  try {
    const bytes = new TextEncoder().encode(src);
    const b64 = btoa(String.fromCharCode(...bytes));
    return `#v=1&src=${encodeURIComponent(b64)}`;
  } catch (_) {
    return "";
  }
}

function decodeSourceFromHash(hash) {
  try {
    const params = new URLSearchParams(hash.replace(/^#/, ""));
    const b64 = params.get("src");
    if (!b64) return null;
    const decoded = atob(decodeURIComponent(b64));
    const bytes = Uint8Array.from(decoded, (c) => c.charCodeAt(0));
    return new TextDecoder().decode(bytes);
  } catch (_) {
    return null;
  }
}

// ─── D7: View config (localStorage) ──────────────────────────────────────────

const DEFAULT_VIEW_CONFIG = {
  block_editor: {
    expert_mode:   false,
    show_tip:      true,
    fps:           30,
    mirror_full:   false,
  },
};

function loadViewConfig() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY_CONFIG);
    if (!raw) return structuredClone(DEFAULT_VIEW_CONFIG);
    const parsed = JSON.parse(raw);
    // deep merge with defaults
    return {
      block_editor: {
        ...DEFAULT_VIEW_CONFIG.block_editor,
        ...(parsed?.block_editor ?? {}),
      },
    };
  } catch (_) {
    return structuredClone(DEFAULT_VIEW_CONFIG);
  }
}

function saveViewConfig(cfg) {
  try {
    localStorage.setItem(STORAGE_KEY_CONFIG, JSON.stringify(cfg));
  } catch (_) {}
}

// ─── D2: Draft 자동저장 ───────────────────────────────────────────────────────

let draftTimer = null;

function scheduleDraftSave(text) {
  clearTimeout(draftTimer);
  draftTimer = setTimeout(() => {
    try { localStorage.setItem(STORAGE_KEY_DRAFT, text); } catch (_) {}
  }, DRAFT_DEBOUNCE_MS);
}

function clearDraft() {
  clearTimeout(draftTimer);
  try { localStorage.removeItem(STORAGE_KEY_DRAFT); } catch (_) {}
}

function loadDraft() {
  try { return localStorage.getItem(STORAGE_KEY_DRAFT) ?? null; } catch (_) { return null; }
}

// ─── DOM refs ─────────────────────────────────────────────────────────────────

const $ = (id) => document.getElementById(id);

const elEditor          = $("play-editor");
const elExampleSel      = $("play-example-select");
const elSourceLabel     = $("play-source-label");
const elBtnRun          = $("btn-play-run");
const elBtnStep         = $("btn-play-step");
const elBtnStep10       = $("btn-play-step10");
const elBtnPause        = $("btn-play-pause");
const elBtnReset        = $("btn-play-reset");
const elBtnShare        = $("btn-play-share");
const elBtnSettings     = $("btn-play-settings");
const elTickCount       = $("play-tick-count");
const elStatusMsg       = $("play-status-msg");
const elStatusHash      = $("play-status-hash");
const elDirtyBadge      = $("play-dirty-badge");
const elDraftBanner     = $("play-draft-banner");
const elBtnDraftRestore = $("btn-draft-restore");
const elBtnDraftDiscard = $("btn-draft-discard");
const elTipChip         = $("play-tip-chip");
const elTipUnits        = $("play-tip-units");
const elDiagEmpty       = $("play-diag-empty");
const elDiagList        = $("play-diag-list");
const elObsEmpty        = $("play-obs-empty");
const elObsBody         = $("play-obs-body");
const elMirror          = $("play-mirror-content");
const elMirrorBogaeCanvas = $("play-mirror-bogae-canvas");
const elMirrorBogaeStatus = $("play-mirror-bogae-status");
const elMirrorBogaeOverlay = $("play-mirror-bogae-overlay");
const elSettingsOverlay = $("play-settings-overlay");
const elSettingExpert   = $("setting-expert-mode");
const elSettingShowTip  = $("setting-show-tip");
const elSettingFps      = $("setting-fps");
const elSettingMirrorFull = $("setting-mirror-full");
const elBtnSettingsClose  = $("btn-settings-close");
const elBtnGrammarAnalyze = $("btn-play-grammar-analyze");
const elGrammarStatus     = $("play-grammar-status");
const elGrammarBuild      = $("play-grammar-build");
const elGrammarUiPre      = $("play-grammar-ui-pre");
const elGrammarWasmPre    = $("play-grammar-wasm-pre");
const elGrammarFlat       = $("play-grammar-flat");
const elGrammarMaegim     = $("play-grammar-maegim");
const elGrammarAlrim      = $("play-grammar-alrim");
const elGrammarBlock      = $("play-grammar-block");
const playTabButtons = Array.from(document.querySelectorAll(".play-tab"));
const playTabPanels = Array.from(document.querySelectorAll(".play-tab-panel"));

let activePlayTab = "diag";
let currentRequestedViewFamilies = [];
let canonRuntime = null;
let grammarAnalyzeSeq = 0;
let playBogae = null;
let lastMirrorSpace2d = null;
let lastMirrorOverlayText = null;
let lastMirrorViewStack = null;

// ─── 출력 탭 전환 ─────────────────────────────────────────────────────────────

function setPlayTab(tab) {
  activePlayTab = normalizePlayTab(tab);
  playTabButtons.forEach((btn) => {
    btn.classList.toggle("active", btn.dataset.tab === activePlayTab);
  });
  playTabPanels.forEach((panel) => {
    panel.classList.toggle("active", panel.id === `play-panel-${activePlayTab}`);
  });
  if (activePlayTab === "mirror") {
    renderMirrorBogae(lastMirrorSpace2d);
    renderMirrorOverlayText(lastMirrorOverlayText, lastMirrorViewStack);
  }
}

function syncPlayOutputTabs(summary, { preserveCurrent = false } = {}) {
  const visible = resolvePlayTabVisibility(summary);
  playTabButtons.forEach((btn) => {
    const tab = String(btn.dataset.tab ?? "");
    const enabled = Boolean(visible[tab]);
    btn.disabled = !enabled;
    btn.title = enabled
      ? ""
      : tab === "obs"
        ? "보임{} 출력이 있을 때 열립니다."
        : tab === "mirror"
          ? "실행 상태가 있을 때 열립니다."
          : "";
  });
  setPlayTab(resolvePlayActiveTab(activePlayTab, summary, { preserveCurrent }));
}

playTabButtons.forEach((btn) => {
  btn.addEventListener("click", () => {
    if (btn.disabled) return;
    setPlayTab(btn.dataset.tab);
  });
});

// ─── 런타임 상태 ─────────────────────────────────────────────────────────────

let viewConfig = loadViewConfig();

const loader = createWasmLoader({
  modulePath: "./wasm/ddonirang_tool.js",
  wrapperPath: "./wasm_ddn_wrapper.js",
  setStatus: (lines) => setStatus(Array.isArray(lines) ? lines.join(" ") : String(lines)),
});

let client     = null;
let tickCount  = 0;
let loopActive = false;
let isDirty    = false;

const loop = createManagedRafStepLoop({
  getFps:    () => viewConfig.block_editor.fps || 30,
  isActive:  () => loopActive,
  setActive: (v) => { loopActive = Boolean(v); },
  onStep:    () => stepFrame(),
  onError:   (_err) => {},
});

// ─── UI 헬퍼 ─────────────────────────────────────────────────────────────────

function setStatus(msg, { isError = false } = {}) {
  if (elStatusMsg) {
    elStatusMsg.textContent = String(msg ?? "");
    elStatusMsg.className   = isError ? "status-error" : "";
  }
}

function setSourceLabel(text) {
  if (!elSourceLabel) return;
  const label = String(text ?? "").trim() || "source: -";
  elSourceLabel.textContent = label;
  elSourceLabel.title = label;
}

function setTickCount(n) {
  tickCount = n;
  if (elTickCount) elTickCount.textContent = `틱: ${n}`;
}

function setRunning(running) {
  loopActive = running;
  if (elBtnRun)    { elBtnRun.disabled = running; }
  if (elBtnStep)     elBtnStep.disabled = running;
  if (elBtnStep10)   elBtnStep10.disabled = running;
  if (elBtnPause)  { elBtnPause.disabled = !running; }
}

// ─── D2: Dirty 표시 ───────────────────────────────────────────────────────────

function markDirty() {
  isDirty = true;
  elDirtyBadge?.classList.add("visible");
}

function clearDirty() {
  isDirty = false;
  elDirtyBadge?.classList.remove("visible");
  clearDraft();
}

// ─── D4: Tip chip 표시 ───────────────────────────────────────────────────────

function showTip(show) {
  if (!viewConfig.block_editor.show_tip) {
    elTipChip?.classList.remove("visible");
    return;
  }
  elTipChip?.classList.toggle("visible", show);
}

// ─── D5: 단위 참조 렌더 ───────────────────────────────────────────────────────

function renderUnitReference() {
  if (!elTipUnits) return;
  const parts = Object.entries(DDN_UNITS).map(([cat, units]) =>
    `<span style="color:var(--text-muted,#888)">${cat}:</span> ${units.map((u) =>
      `<span class="tip-item" style="font-size:0.7rem">${escHtml(u)}</span>`
    ).join(" ")}`
  );
  elTipUnits.innerHTML = `<div style="display:flex;flex-wrap:wrap;gap:8px;align-items:baseline">${parts.join("&nbsp;&nbsp;")}</div>`;
}

// ─── 문법 실험실 ──────────────────────────────────────────────────────────────

function setGrammarStatus(text, { isError = false } = {}) {
  if (!elGrammarStatus) return;
  elGrammarStatus.textContent = String(text ?? "");
  elGrammarStatus.style.color = isError ? "#f48" : "";
}

function setGrammarPanelText(el, value) {
  if (!el) return;
  el.textContent = String(value ?? "-");
}

function jsonPreview(value, { maxLen = GRAMMAR_TEXT_MAX } = {}) {
  let text = "";
  try {
    text = JSON.stringify(value, null, 2);
  } catch (_) {
    text = String(value ?? "");
  }
  if (text.length <= maxLen) return text;
  return `${text.slice(0, maxLen)}\n... (truncated ${text.length - maxLen} chars)`;
}

function textPreview(value, { maxLen = GRAMMAR_TEXT_MAX } = {}) {
  const text = String(value ?? "");
  if (text.length <= maxLen) return text;
  return `${text.slice(0, maxLen)}\n... (truncated ${text.length - maxLen} chars)`;
}

function formatGrammarError(label, err) {
  return `[${label} ERROR]\n${String(err?.message ?? err)}`;
}

function resetGrammarPanels() {
  setGrammarStatus("대기");
  setGrammarPanelText(elGrammarUiPre, "-");
  setGrammarPanelText(elGrammarWasmPre, "-");
  setGrammarPanelText(elGrammarFlat, "-");
  setGrammarPanelText(elGrammarMaegim, "-");
  setGrammarPanelText(elGrammarAlrim, "-");
  setGrammarPanelText(elGrammarBlock, "-");
}

async function ensureCanonRuntime() {
  if (canonRuntime) return canonRuntime;
  canonRuntime = await createWasmCanon({
    wasmUrl: "./wasm/ddonirang_tool.js",
    cacheBust: "play-grammar-lab",
  });
  const buildInfo = typeof canonRuntime?.getBuildInfo === "function"
    ? String(canonRuntime.getBuildInfo() ?? "").trim()
    : "";
  if (elGrammarBuild) {
    elGrammarBuild.textContent = buildInfo ? buildInfo : "wasm canon ready";
    elGrammarBuild.title = elGrammarBuild.textContent;
  }
  return canonRuntime;
}

async function runGrammarAnalysis(sourceText) {
  const source = String(sourceText ?? "");
  const seq = ++grammarAnalyzeSeq;
  setGrammarStatus("분석 중…");

  const uiPre = preprocessDdnText(source);
  const uiPrePayload = {
    schema: "ddn.play.grammar_lab.ui_preprocess.v1",
    pragmas: Array.isArray(uiPre?.pragmas) ? uiPre.pragmas : [],
    diags: normalizePreprocessDiagnostics(uiPre?.diags),
    body_text: String(uiPre?.bodyText ?? ""),
  };
  setGrammarPanelText(elGrammarUiPre, jsonPreview(uiPrePayload));

  let runtime = null;
  try {
    runtime = await ensureCanonRuntime();
  } catch (err) {
    if (seq !== grammarAnalyzeSeq) return;
    setGrammarStatus("WASM canon 초기화 실패", { isError: true });
    setGrammarPanelText(elGrammarWasmPre, formatGrammarError("wasm_preprocess_source", err));
    setGrammarPanelText(elGrammarFlat, formatGrammarError("flat", err));
    setGrammarPanelText(elGrammarMaegim, formatGrammarError("maegim", err));
    setGrammarPanelText(elGrammarAlrim, formatGrammarError("alrim", err));
    setGrammarPanelText(elGrammarBlock, formatGrammarError("block_editor", err));
    return;
  }

  const wasmPrePromise = runtime.preprocessSource(source)
    .then((result) => ({ ok: true, text: textPreview(result) }))
    .catch((err) => ({ ok: false, text: formatGrammarError("wasm_preprocess_source", err) }));
  const flatPromise = runtime.canonFlatJson(source)
    .then((result) => ({ ok: true, text: jsonPreview(result) }))
    .catch((err) => ({ ok: false, text: formatGrammarError("flat", err) }));
  const maegimPromise = runtime.canonMaegimPlan(source)
    .then((result) => ({ ok: true, text: jsonPreview(result) }))
    .catch((err) => ({ ok: false, text: formatGrammarError("maegim", err) }));
  const alrimPromise = runtime.canonAlrimPlan(source)
    .then((result) => ({ ok: true, text: jsonPreview(result) }))
    .catch((err) => ({ ok: false, text: formatGrammarError("alrim", err) }));
  const blockPromise = runtime.canonBlockEditorPlan(source)
    .then((result) => ({ ok: true, text: jsonPreview(result) }))
    .catch((err) => ({ ok: false, text: formatGrammarError("block_editor", err) }));

  const [wasmPre, flat, maegim, alrim, block] = await Promise.all([
    wasmPrePromise,
    flatPromise,
    maegimPromise,
    alrimPromise,
    blockPromise,
  ]);

  if (seq !== grammarAnalyzeSeq) return;

  setGrammarPanelText(elGrammarWasmPre, wasmPre.text);
  setGrammarPanelText(elGrammarFlat, flat.text);
  setGrammarPanelText(elGrammarMaegim, maegim.text);
  setGrammarPanelText(elGrammarAlrim, alrim.text);
  setGrammarPanelText(elGrammarBlock, block.text);

  const summary = [wasmPre, flat, maegim, alrim, block];
  const failCount = summary.filter((entry) => !entry.ok).length;
  if (failCount > 0) {
    setGrammarStatus(`분석 완료 (오류 ${failCount}개)`, { isError: true });
  } else {
    setGrammarStatus("분석 완료");
  }
}

// ─── 진단 렌더 ────────────────────────────────────────────────────────────────

function normalizeDiagnosticSeverity(warning) {
  const raw = String(warning?.severity ?? warning?.level ?? "").trim().toLowerCase();
  if (raw === "error" || raw === "err" || raw === "fatal") return "error";
  if (raw === "warn" || raw === "warning") return "warn";
  if (raw === "info" || raw === "information") return "info";
  const code = String(warning?.code ?? warning?.kind ?? "").trim().toUpperCase();
  if (code.startsWith("E_")) return "error";
  if (code.startsWith("W_")) return "warn";
  return "info";
}

function diagnosticHasError(warning) {
  return normalizeDiagnosticSeverity(warning) === "error";
}

function readDiagnosticLineInfo(warning) {
  const spanLine = warning?.span?.start_line;
  if (spanLine != null) return String(spanLine);
  const locLine = warning?.loc?.line1;
  if (locLine != null) return String(locLine);
  const where = String(warning?.where ?? "");
  const match = where.match(/line\s*:\s*(\d+)/i);
  return match ? String(match[1]) : "";
}

function normalizePreprocessDiagnostics(diags) {
  if (!Array.isArray(diags) || diags.length === 0) return [];
  return diags
    .filter((row) => row && typeof row === "object")
    .map((row) => ({
      code: String(row.code ?? "W_PREPROCESS"),
      message: String(row.message ?? row.details ?? ""),
      severity: normalizeDiagnosticSeverity(row),
      where: String(row.where ?? ""),
      loc: row.loc && typeof row.loc === "object" ? row.loc : undefined,
    }));
}

function renderDiagnostics(warnings) {
  if (!elDiagList || !elDiagEmpty) return;
  elDiagList.innerHTML = "";
  if (!warnings || warnings.length === 0) {
    elDiagEmpty.style.display = "";
    return { count: 0, hasError: false };
  }
  elDiagEmpty.style.display = "none";
  let hasError = false;
  for (const w of warnings) {
    const li   = document.createElement("li");
    const code = String(w?.code ?? w?.kind ?? "");
    const msg  = String(w?.message ?? w?.msg ?? w ?? "");
    const sev  = normalizeDiagnosticSeverity(w);
    li.className = sev === "error" ? "diag-error" : sev === "warn" ? "diag-warn" : "diag-info";
    if (sev === "error") hasError = true;
    const line = readDiagnosticLineInfo(w);
    const lineInfo = line ? ` [${line}줄]` : "";
    li.textContent = `${code}${lineInfo}: ${msg}`;
    elDiagList.appendChild(li);
  }
  return { count: warnings.length, hasError };
}

// ─── 출력(관측) 렌더 ──────────────────────────────────────────────────────────

function renderObservation(state) {
  if (!elObsBody || !elObsEmpty) return;
  const obs      = extractObservationChannelsFromState(state);
  const channels = Array.isArray(obs?.channels) ? obs.channels : [];
  const values   = obs?.values ?? obs?.all_values ?? {};
  const outputRows = extractObservationOutputRowsFromState(state);

  if (outputRows.length === 0 && channels.length === 0) {
    elObsEmpty.style.display = "";
    elObsBody.innerHTML = "";
    return { rowCount: 0 };
  }
  elObsEmpty.style.display = "none";
  elObsBody.innerHTML = "";
  // 출력 탭은 보임{} 기반 table.row가 있으면 이를 우선 표시한다.
  if (outputRows.length > 0) {
    for (const row of outputRows) {
      const tr  = document.createElement("tr");
      const tdK = document.createElement("td");
      const tdV = document.createElement("td");
      tdK.textContent = String(row?.key ?? "");
      tdV.textContent = String(row?.value ?? "");
      tr.appendChild(tdK);
      tr.appendChild(tdV);
      elObsBody.appendChild(tr);
    }
    return { rowCount: outputRows.length };
  }

  for (const ch of channels) {
    const key = typeof ch === "string" ? ch : (ch?.key ?? "");
    if (!key) continue;
    const tr  = document.createElement("tr");
    const tdK = document.createElement("td");
    const tdV = document.createElement("td");
    tdK.textContent = key;
    tdV.textContent = formatValue(values[key]);
    tr.appendChild(tdK);
    tr.appendChild(tdV);
    elObsBody.appendChild(tr);
  }
  return { rowCount: channels.length };
}

function formatValue(val) {
  if (val === null || val === undefined) return "—";
  if (typeof val === "number") return Number.isFinite(val) ? val.toPrecision(6).replace(/\.?0+$/, "") : String(val);
  if (typeof val === "string") return val;
  if (typeof val === "object") {
    if ("f64" in val) return formatValue(val.f64);
    if ("num" in val) return formatValue(val.num);
    return JSON.stringify(val);
  }
  return String(val);
}

function formatMirrorJsonPreview(raw, { maxLen = 96 } = {}) {
  const source = String(raw ?? "");
  let text = source;
  try {
    const parsed = JSON.parse(source);
    text = JSON.stringify(parsed);
  } catch (_) {
    // raw 문자열 그대로 사용
  }
  if (text.length <= maxLen) return text;
  return `${text.slice(0, maxLen)}…`;
}

function summarizeSpace2dView(space2d) {
  if (!space2d || typeof space2d !== "object") return null;
  const points = Array.isArray(space2d.points) ? space2d.points : [];
  const shapes = Array.isArray(space2d.shapes) ? space2d.shapes : [];
  const drawlist = Array.isArray(space2d.drawlist) ? space2d.drawlist : [];
  if (points.length === 0 && shapes.length === 0 && drawlist.length === 0) return null;

  const shapeKindCount = new Map();
  const pushKind = (item) => {
    const kind = String(item?.kind ?? item?.type ?? "").trim().toLowerCase();
    if (!kind) return;
    shapeKindCount.set(kind, (shapeKindCount.get(kind) ?? 0) + 1);
  };
  shapes.forEach((item) => pushKind(item));
  drawlist.forEach((item) => pushKind(item));

  return {
    points: points.length,
    shapes: shapes.length,
    drawlist: drawlist.length,
    kinds: Array.from(shapeKindCount.entries())
      .sort(([a], [b]) => a.localeCompare(b, "ko"))
      .map(([kind, count]) => `${kind}:${count}`),
  };
}

function hasSpace2dDrawable(space2d) {
  if (!space2d || typeof space2d !== "object") return false;
  const points = Array.isArray(space2d.points) ? space2d.points.length : 0;
  const shapes = Array.isArray(space2d.shapes) ? space2d.shapes.length : 0;
  const drawlist = Array.isArray(space2d.drawlist) ? space2d.drawlist.length : 0;
  return points > 0 || shapes > 0 || drawlist > 0;
}

function ensurePlayBogae() {
  if (playBogae) return playBogae;
  if (!elMirrorBogaeCanvas) return null;
  playBogae = new Bogae({ canvas: elMirrorBogaeCanvas });
  return playBogae;
}

function setMirrorBogaeStatus(space2d) {
  if (!elMirrorBogaeStatus) return;
  const summary = summarizeSpace2dView(space2d);
  if (!summary) {
    elMirrorBogaeStatus.textContent = "보개 출력 없음";
    return;
  }
  elMirrorBogaeStatus.textContent = `점 ${summary.points} · 모양 ${summary.shapes} · drawlist ${summary.drawlist}`;
}

function renderMirrorBogae(space2d) {
  const bogae = ensurePlayBogae();
  if (!bogae) return;
  const drawableSpace2d = hasSpace2dDrawable(space2d) ? space2d : null;
  lastMirrorSpace2d = drawableSpace2d;
  bogae.render(drawableSpace2d);
  setMirrorBogaeStatus(drawableSpace2d);
}

function resetMirrorBogae() {
  lastMirrorSpace2d = null;
  if (playBogae) {
    playBogae.resetView();
    playBogae.render(null);
  } else {
    setMirrorBogaeStatus(null);
  }
  renderMirrorOverlayText(null, null);
}

function summarizeOverlayTextView(textView) {
  if (!textView || typeof textView !== "object") return null;
  const markdown = String(textView.markdown ?? textView.text ?? "").trim();
  if (!markdown) return null;
  const id = String(textView.id ?? "").trim();
  const x = Number(textView.x);
  const y = Number(textView.y);
  return {
    id,
    markdown,
    hasPosition: Number.isFinite(x) && Number.isFinite(y),
    x: Number.isFinite(x) ? x : null,
    y: Number.isFinite(y) ? y : null,
  };
}

function summarizeOverlayStack(viewStack) {
  const overlays = Array.isArray(viewStack?.overlays) ? viewStack.overlays : [];
  if (overlays.length === 0) return null;
  return {
    count: overlays.length,
    families: Array.from(new Set(overlays.map((entry) => String(entry?.family ?? "").trim()).filter(Boolean))),
    roles: Array.from(new Set(overlays.map((entry) => String(entry?.role ?? "").trim()).filter(Boolean))),
  };
}

function hasOverlayTextFamily(viewStack) {
  const overlays = Array.isArray(viewStack?.overlays) ? viewStack.overlays : [];
  return overlays.some((entry) => String(entry?.family ?? "").trim().toLowerCase() === "text");
}

function setMirrorOverlayHidden() {
  if (!elMirrorBogaeOverlay) return;
  elMirrorBogaeOverlay.classList.remove("visible");
  elMirrorBogaeOverlay.innerHTML = "";
}

function renderMirrorOverlayText(overlayText, viewStack) {
  lastMirrorOverlayText = overlayText ?? null;
  lastMirrorViewStack = viewStack ?? null;
  if (!elMirrorBogaeOverlay) return;
  if (!overlayText || !overlayText.markdown) {
    setMirrorOverlayHidden();
    return;
  }
  if (!overlayText.hasPosition && !hasOverlayTextFamily(viewStack)) {
    setMirrorOverlayHidden();
    return;
  }

  const bodyHtml = markdownToHtml(overlayText.markdown);
  const x = Number(overlayText.x);
  const y = Number(overlayText.y);
  const hasNormalizedPos = overlayText.hasPosition
    && Number.isFinite(x)
    && Number.isFinite(y)
    && x >= 0
    && x <= 1
    && y >= 0
    && y <= 1;

  let leftStyle = "10px";
  let topStyle = "10px";
  let transformStyle = "none";
  if (hasNormalizedPos) {
    const clampedX = Math.max(0, Math.min(1, x));
    const clampedY = Math.max(0, Math.min(1, y));
    leftStyle = `${(clampedX * 100).toFixed(1)}%`;
    topStyle = `${((1 - clampedY) * 100).toFixed(1)}%`;
    transformStyle = "translate(-50%, -50%)";
  }

  elMirrorBogaeOverlay.innerHTML = `<div class="overlay-chip" style="left:${leftStyle}; top:${topStyle}; transform:${transformStyle};">${bodyHtml}</div>`;
  elMirrorBogaeOverlay.classList.add("visible");
}

function stripMaegimAnnotations(text) {
  let result = "";
  let i = 0;
  const src = String(text ?? "");
  while (i < src.length) {
    const maegimIdx = src.indexOf("매김", i);
    if (maegimIdx === -1) {
      result += src.slice(i);
      break;
    }
    let j = maegimIdx + "매김".length;
    while (j < src.length && " \t\n\r".includes(src[j])) j += 1;
    if (j < src.length && src[j] === "{") {
      result += src.slice(i, maegimIdx).replace(/[ \t]+$/u, "");
      let depth = 0;
      let k = j;
      while (k < src.length) {
        if (src[k] === "{") {
          depth += 1;
          k += 1;
          continue;
        }
        if (src[k] === "}") {
          depth -= 1;
          k += 1;
          if (depth === 0) {
            while (k < src.length && " \t\n\r".includes(src[k])) k += 1;
            if (k < src.length && src[k] === ".") k += 1;
            break;
          }
          continue;
        }
        k += 1;
      }
      result += ".";
      i = k;
      continue;
    }
    result += src.slice(i, j);
    i = j;
  }
  return result;
}

function getEffectiveWasmSource(rawText) {
  const raw = String(rawText ?? "");
  try {
    const pre = preprocessDdnText(raw);
    const preprocessWarnings = normalizePreprocessDiagnostics(pre?.diags);
    const body = stripMaegimAnnotations(String(pre?.bodyText ?? ""));
    return {
      sourceText: body.trim() ? body : raw,
      preprocessWarnings,
    };
  } catch (err) {
    return {
      sourceText: raw,
      preprocessWarnings: [{
        code: "W_PREPROCESS_FAIL",
        message: String(err?.message ?? err ?? "전처리 실패"),
        severity: "warn",
      }],
    };
  }
}

function extractRequestedViewFamilies(sourceText) {
  try {
    const ddnMeta = parseLessonDdnMetaHeader(String(sourceText ?? ""));
    return normalizeViewFamilyList(ddnMeta?.requiredViews ?? ddnMeta?.required_views ?? []);
  } catch (_) {
    return [];
  }
}

// ─── 거울(mirror) 렌더 ───────────────────────────────────────────────────────

function renderMirror(state, { warnings = [], requestedViewFamilies = [] } = {}) {
  if (!elMirror) return { hasContent: false };
  const norm = normalizeWasmStatePayload(state);
  const structuredViews = extractStructuredViewsFromState(state, { preferPatch: false });
  renderMirrorBogae(structuredViews?.space2d);
  const maxEntries = viewConfig.block_editor.mirror_full ? 200 : MIRROR_DEFAULT_MAX;
  const resolvedViewFamilies = normalizeViewFamilyList(structuredViews?.families ?? []);
  const declaredFamilies = normalizeViewFamilyList(requestedViewFamilies);
  const viewFamilies = resolvedViewFamilies.length > 0 ? resolvedViewFamilies : declaredFamilies;
  const missingFamilies = declaredFamilies.filter((family) => !resolvedViewFamilies.includes(family));
  const graphResult = buildFamilyPreviewResult({
    family: "graph",
    payload: structuredViews?.graph,
    html: buildGraphPreviewHtml(structuredViews?.graph, { width: 240, height: 150, maxSeries: 3 }),
  });
  const tableResult = buildFamilyPreviewResult({
    family: "table",
    payload: structuredViews?.table,
    html: buildTablePreviewHtml(structuredViews?.table, {
      maxRows: 4,
      maxCols: 3,
      containerClass: "runtime-table-preview",
      tableClass: "runtime-table-preview-table",
      titleClass: "runtime-table-preview-title",
      metaClass: "runtime-table-preview-meta",
    }),
  });
  const structureResult = buildFamilyPreviewResult({
    family: "structure",
    payload: structuredViews?.structure,
    html: buildStructurePreviewHtml(structuredViews?.structure, { width: 240, height: 150, maxNodes: 6 }),
  });
  const graphSummary = graphResult?.summary ?? null;
  const tableSummary = tableResult?.summary ?? null;
  const structureSummary = structureResult?.summary ?? null;
  const space2dSummary = summarizeSpace2dView(structuredViews?.space2d);
  const overlayStackSummary = summarizeOverlayStack(structuredViews?.viewStack);
  const overlayTextSummary = summarizeOverlayTextView(structuredViews?.text);
  renderMirrorOverlayText(overlayTextSummary, structuredViews?.viewStack);
  const previewCollection = buildPreviewResultCollection([graphResult, tableResult, structureResult], {
    preferredFamilies: viewFamilies.length > 0 ? viewFamilies : undefined,
    summaryClassName: "runtime-preview-summary",
    cardClassName: "runtime-preview-card",
  });
  const previewViewModel = buildPreviewViewModel(previewCollection, { sourceId: "play.mirror" });
  const previewCollectionHtml = String(previewCollection?.html ?? "");

  const sections = [
    { title: "메타", items: [
      ["틱",    norm.tick_id ?? tickCount],
      ["해시",  norm.state_hash ? norm.state_hash.slice(0, 16) + "…" : "—"],
      ["진단",  warnings.length + "건"],
      ["요청 보기", declaredFamilies.length ? declaredFamilies.join(", ") : "—"],
      ["보기",  viewFamilies.length ? viewFamilies.join(", ") : "—"],
      ["미해석 보기", missingFamilies.length ? missingFamilies.join(", ") : "—"],
      ["겹보기 레이어", overlayStackSummary ? `${overlayStackSummary.count}개` : "0개"],
      ["대표 보기", previewViewModel?.summaryText || "—"],
    ]},
  ];

  const rf = norm?.resources?.fixed64 ?? {};
  const rfEntries = Object.entries(rf).slice(0, maxEntries);
  if (rfEntries.length > 0) {
    sections.push({ title: "수 상태", items: rfEntries.map(([k, v]) => [k, formatValue(v)]) });
  }

  const inputEntries = flattenMirrorInputEntries(norm?.input, { maxEntries });
  if (inputEntries.length > 0) {
    sections.push({ title: "입력", items: inputEntries.map(([k, v]) => [k, formatValue(v)]) });
  }

  const rj = norm?.resources?.json ?? {};
  const rjEntries = Object.entries(rj).slice(0, maxEntries);
  if (rjEntries.length > 0) {
    const maxJsonLen = viewConfig.block_editor.mirror_full ? 240 : 96;
    sections.push({
      title: "json 상태",
      items: rjEntries.map(([k, v]) => [k, formatMirrorJsonPreview(v, { maxLen: maxJsonLen })]),
    });
  }

  const rv = norm?.resources?.value ?? {};
  const rvEntries = Object.entries(rv).slice(0, maxEntries);
  if (rvEntries.length > 0) {
    sections.push({ title: "값 자원", items: rvEntries.map(([k, v]) => [k, formatValue(v)]) });
  }

  if (structureSummary) {
    const items = [
      ["노드", `${structureSummary.nodeCount}개`],
      ["간선", `${structureSummary.edgeCount}개`],
    ];
    if (structureSummary.title) {
      items.unshift(["제목", structureSummary.title]);
    }
    if (structureSummary.nodeSamples.length > 0) {
      items.push(["노드 예", structureSummary.nodeSamples.join(", ")]);
    }
    if (structureSummary.edgeSamples.length > 0) {
      items.push(["간선 예", structureSummary.edgeSamples.join(", ")]);
    }
    sections.push({ title: "구조", items });
  }

  if (graphSummary) {
    const items = [
      ["계열", `${graphSummary.seriesCount}개`],
      ["점", `${graphSummary.pointCount}개`],
    ];
    if (graphSummary.title) {
      items.unshift(["제목", graphSummary.title]);
    }
    if (graphSummary.seriesLabels.length > 0) {
      items.push(["계열 예", graphSummary.seriesLabels.join(", ")]);
    }
    sections.push({ title: "그래프", items });
  }

  if (tableSummary) {
    const items = [
      ["열", `${tableSummary.columnCount}개`],
      ["행", `${tableSummary.rowCount}개`],
    ];
    if (tableSummary.title) {
      items.unshift(["제목", tableSummary.title]);
    }
    if (tableSummary.columns.length > 0) {
      items.push(["열 예", tableSummary.columns.join(", ")]);
    }
    sections.push({ title: "표", items });
  }

  if (space2dSummary) {
    const items = [
      ["점", `${space2dSummary.points}개`],
      ["모양", `${space2dSummary.shapes}개`],
      ["drawlist", `${space2dSummary.drawlist}개`],
    ];
    if (space2dSummary.kinds.length > 0) {
      items.push(["모양 종류", space2dSummary.kinds.join(", ")]);
    }
    sections.push({ title: "모양(space2d)", items });
  }

  if (overlayStackSummary || overlayTextSummary) {
    const items = [];
    if (overlayStackSummary) {
      items.push(["레이어 수", `${overlayStackSummary.count}개`]);
      if (overlayStackSummary.families.length > 0) {
        items.push(["레이어 보기", overlayStackSummary.families.join(", ")]);
      }
      if (overlayStackSummary.roles.length > 0) {
        items.push(["레이어 역할", overlayStackSummary.roles.join(", ")]);
      }
    }
    if (overlayTextSummary) {
      if (overlayTextSummary.id) {
        items.push(["자막 id", overlayTextSummary.id]);
      }
      items.push(["자막", overlayTextSummary.markdown]);
      if (overlayTextSummary.hasPosition) {
        items.push(["자막 위치", `${formatValue(overlayTextSummary.x)}, ${formatValue(overlayTextSummary.y)}`]);
      }
    }
    sections.push({ title: "겹보기(overlay)", items });
  }

  // expert_mode: json/handle/component resources도 표시
  if (viewConfig.block_editor.expert_mode) {
    const rh = norm?.resources?.handle ?? {};
    const rhEntries = Object.entries(rh).slice(0, 10);
    if (rhEntries.length > 0) {
      sections.push({ title: "handle 자원", items: rhEntries.map(([k, v]) => [k, String(v)]) });
    }
    const rc = norm?.resources?.component ?? {};
    const rcEntries = Object.entries(rc).slice(0, 10);
    if (rcEntries.length > 0) {
      sections.push({ title: "component 자원", items: rcEntries.map(([k, v]) => [k, JSON.stringify(v).slice(0, 60)]) });
    }
  }

  const sectionHtml = sections.map(({ title, items }) => `
    <div class="mirror-section">${escHtml(title)}</div>
    ${items.map(([k, v]) => `
      <div class="mirror-kv">
        <span class="mirror-key">${escHtml(String(k))}</span>
        <span class="mirror-sep">=</span>
        <span class="mirror-val">${escHtml(String(v ?? "—"))}</span>
      </div>`).join("")}
  `).join("");
  applyPreviewViewModelMetadata(elMirror, previewViewModel);
  elMirror.innerHTML = `${previewCollectionHtml}${sectionHtml}`;

  if (elStatusHash) {
    elStatusHash.textContent = norm.state_hash ? `hash: ${norm.state_hash.slice(0, 12)}` : "";
  }
  return { hasContent: sections.length > 0 };
}

function escHtml(str) {
  return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

// ─── 소스 적용 (초기화 + tick 0) ─────────────────────────────────────────────

async function applySource(src) {
  setStatus("로딩 중…");
  client = null;
  currentRequestedViewFamilies = extractRequestedViewFamilies(src);
  setTickCount(0);
  try {
    const sourceInfo = getEffectiveWasmSource(src);
    const wasmSource = String(sourceInfo?.sourceText ?? "");
    const preprocessWarnings = Array.isArray(sourceInfo?.preprocessWarnings)
      ? sourceInfo.preprocessWarnings
      : [];
    const result = await applyWasmLogicFromSource({
      sourceText: wasmSource,
      ensureWasm: (text) => loader.ensure(text),
    });
    client = result.client;
    let state = result.state;
    if (typeof client?.resetParsed === "function") {
      try {
        client.resetParsed(false);
        if (typeof client?.getStateParsed === "function") {
          state = client.getStateParsed();
        }
      } catch (_) {
        // reset 실패 시 updateLogic 직후 상태를 그대로 사용
      }
    }
    const warnings = [...preprocessWarnings, ...(result.parseWarnings ?? [])];
    const diag = renderDiagnostics(warnings) ?? { count: 0, hasError: false };
    const obs = renderObservation(state) ?? { rowCount: 0 };
    const mirror = renderMirror(state, { warnings, requestedViewFamilies: currentRequestedViewFamilies }) ?? { hasContent: false };
    syncPlayOutputTabs({
      hasError: diag.hasError,
      hasDiagnostics: diag.count > 0,
      hasObservation: obs.rowCount > 0,
      hasMirror: mirror.hasContent,
    });
    const hasError = warnings.some((w) => diagnosticHasError(w));
    setStatus(hasError ? `파서 오류 ${warnings.length}건` : "준비", { isError: hasError });
    setTickCount(0);
    return !hasError;
  } catch (err) {
    setStatus(`오류: ${String(err?.message ?? err)}`, { isError: true });
    const diag = renderDiagnostics([{ code: "E_RUNTIME", message: String(err?.message ?? err), severity: "error" }]);
    syncPlayOutputTabs({
      hasError: true,
      hasDiagnostics: (diag?.count ?? 0) > 0,
      hasObservation: false,
      hasMirror: false,
    });
    return false;
  }
}

// ─── 1틱 실행 ─────────────────────────────────────────────────────────────────

function stepFrame() {
  if (!client) return false;
  try {
    const result   = stepWasmClientParsed({ client });
    const warnings = readWasmClientParseWarnings(client);
    setTickCount(tickCount + 1);
    const diag = renderDiagnostics(warnings) ?? { count: 0, hasError: false };
    const obs = renderObservation(result.state) ?? { rowCount: 0 };
    const mirror = renderMirror(result.state, { warnings, requestedViewFamilies: currentRequestedViewFamilies }) ?? { hasContent: false };
    syncPlayOutputTabs({
      hasError: diag.hasError,
      hasDiagnostics: diag.count > 0,
      hasObservation: obs.rowCount > 0,
      hasMirror: mirror.hasContent,
    });
    return true;
  } catch (err) {
    stopLoop();
    setStatus(`런타임 오류: ${String(err?.message ?? err)}`, { isError: true });
    const diag = renderDiagnostics([{ code: "E_RUNTIME", message: String(err?.message ?? err), severity: "error" }]);
    syncPlayOutputTabs({
      hasError: true,
      hasDiagnostics: (diag?.count ?? 0) > 0,
      hasObservation: false,
      hasMirror: false,
    });
    return false;
  }
}

// ─── 루프 제어 ────────────────────────────────────────────────────────────────

function stopLoop() {
  loop.stop();
  setRunning(false);
}

function startLoop() {
  loop.start();
  setRunning(true);
}

// ─── 버튼: 실행/스텝/일시정지/초기화 ─────────────────────────────────────────

elBtnGrammarAnalyze?.addEventListener("click", async () => {
  await runGrammarAnalysis(elEditor?.value ?? "");
});

elBtnRun.addEventListener("click", async () => {
  const ok = await applySource(elEditor.value);
  if (ok) {
    clearDirty();
    await runGrammarAnalysis(elEditor.value);
    const stepped = stepFrame();
    if (stepped) {
      startLoop();
    }
  }
});

elBtnStep.addEventListener("click", async () => {
  if (!client) {
    const ok = await applySource(elEditor.value);
    if (!ok) return;
    clearDirty();
    await runGrammarAnalysis(elEditor.value);
  }
  stepFrame();
});

elBtnStep10.addEventListener("click", async () => {
  if (!client) {
    const ok = await applySource(elEditor.value);
    if (!ok) return;
    clearDirty();
    await runGrammarAnalysis(elEditor.value);
  }
  for (let i = 0; i < 10; i++) stepFrame();
});

elBtnPause.addEventListener("click", () => {
  stopLoop();
});

elBtnReset.addEventListener("click", () => {
  stopLoop();
  loader.reset();
  client = null;
  currentRequestedViewFamilies = [];
  setTickCount(0);
  setStatus("초기화됨");
  renderDiagnostics([]);
  if (elObsBody) elObsBody.innerHTML = "";
  if (elObsEmpty) elObsEmpty.style.display = "";
  if (elMirror) {
    applyPreviewViewModelMetadata(elMirror, null);
    elMirror.innerHTML = "";
  }
  resetMirrorBogae();
  if (elStatusHash) elStatusHash.textContent = "";
  syncPlayOutputTabs({
    hasError: false,
    hasDiagnostics: false,
    hasObservation: false,
    hasMirror: false,
  });
  resetGrammarPanels();
});

// ─── 버튼: 예제 선택 ─────────────────────────────────────────────────────────

elExampleSel.addEventListener("change", async () => {
  const key = elExampleSel.value;
  if (!key || !EXAMPLES[key]) return;
  stopLoop();
  client = null;
  elEditor.value = EXAMPLES[key];
  setSourceLabel(`source: example:${key}`);
  elExampleSel.value = "";
  setTickCount(0);
  const ok = await applySource(elEditor.value);
  if (ok) {
    clearDirty();
    stepFrame();
    await runGrammarAnalysis(elEditor.value);
    setStatus("예제 실행됨 — ▶로 계속 실행하거나 1틱/10틱으로 살펴보세요");
  }
});

// ─── 버튼: URL 공유 ───────────────────────────────────────────────────────────

elBtnShare.addEventListener("click", () => {
  const hash = encodeSourceToHash(elEditor.value);
  const url  = `${location.origin}${location.pathname}${location.search}${hash}`;
  navigator.clipboard.writeText(url).then(
    () => setStatus("URL 복사됨!"),
    () => prompt("이 URL을 복사하세요:", url),
  );
});

// ─── D2: 에디터 변경 → dirty + 자동저장 ──────────────────────────────────────

elEditor.addEventListener("input", () => {
  if (!loopActive) client = null;
  markDirty();
  scheduleDraftSave(elEditor.value);
  setGrammarStatus("수정됨 — 문법 분석 필요");
});

// ─── D4: 에디터 포커스 → tip chip ─────────────────────────────────────────────

elEditor.addEventListener("focus", () => showTip(true));
elEditor.addEventListener("blur",  () => {
  // 약간 지연 — 칩 내부 클릭 시 즉시 사라지지 않도록
  setTimeout(() => showTip(false), 200);
});

// ─── D2: beforeunload 경고 ────────────────────────────────────────────────────

window.addEventListener("beforeunload", (e) => {
  if (isDirty) {
    e.preventDefault();
    e.returnValue = "저장되지 않은 변경이 있습니다. 페이지를 떠나시겠습니까?";
  }
});

// ─── D2: Draft 복원 배너 ─────────────────────────────────────────────────────

elBtnDraftRestore?.addEventListener("click", () => {
  const draft = loadDraft();
  if (draft) {
    stopLoop();
    client = null;
    elEditor.value = draft;
    setSourceLabel("source: draft");
    markDirty();
    runGrammarAnalysis(elEditor.value).catch(() => {});
    setStatus("초안 복원됨 — ▶ 실행을 눌러 시작하세요");
  }
  elDraftBanner?.classList.remove("visible");
});

elBtnDraftDiscard?.addEventListener("click", () => {
  clearDraft();
  setSourceLabel("source: example:counter");
  elDraftBanner?.classList.remove("visible");
});

// ─── D7: 설정 패널 ───────────────────────────────────────────────────────────

function applyViewConfigToUi() {
  const bc = viewConfig.block_editor;
  if (elSettingExpert)    elSettingExpert.checked    = Boolean(bc.expert_mode);
  if (elSettingShowTip)   elSettingShowTip.checked   = Boolean(bc.show_tip);
  if (elSettingFps)       elSettingFps.value         = String(bc.fps ?? 30);
  if (elSettingMirrorFull) elSettingMirrorFull.checked = Boolean(bc.mirror_full);
}

function readViewConfigFromUi() {
  viewConfig.block_editor.expert_mode  = elSettingExpert?.checked   ?? false;
  viewConfig.block_editor.show_tip     = elSettingShowTip?.checked  ?? true;
  viewConfig.block_editor.fps          = Math.max(1, Math.min(120, Number(elSettingFps?.value) || 30));
  viewConfig.block_editor.mirror_full  = elSettingMirrorFull?.checked ?? false;
  saveViewConfig(viewConfig);
}

elBtnSettings?.addEventListener("click", () => {
  applyViewConfigToUi();
  elSettingsOverlay?.classList.add("visible");
});

elBtnSettingsClose?.addEventListener("click", () => {
  readViewConfigFromUi();
  elSettingsOverlay?.classList.remove("visible");
  // tip 표시 상태 즉시 반영
  if (!viewConfig.block_editor.show_tip) showTip(false);
});

elSettingsOverlay?.addEventListener("click", (e) => {
  if (e.target === elSettingsOverlay) {
    readViewConfigFromUi();
    elSettingsOverlay.classList.remove("visible");
  }
});

// ─── 초기 로드 ────────────────────────────────────────────────────────────────

async function init() {
  // D5: 단위 참조 렌더
  renderUnitReference();
  resetGrammarPanels();
  resetMirrorBogae();

  // D7: 설정값 반영
  applyViewConfigToUi();
  syncPlayOutputTabs({
    hasError: false,
    hasDiagnostics: false,
    hasObservation: false,
    hasMirror: false,
  });

  // URL 해시 우선
  const fromHash = decodeSourceFromHash(location.hash);
  if (fromHash) {
    elEditor.value = fromHash;
    setSourceLabel("source: url-hash");
    setStatus("URL에서 코드 복원됨 — ▶ 실행을 눌러 시작하세요");
    await runGrammarAnalysis(elEditor.value);
    return;
  }

  const launchRequest = resolvePlayLaunchRequest(window.location);
  if (launchRequest.kind === "lesson" && launchRequest.lesson?.requested) {
    const lessonRequest = launchRequest.lesson;
    let loadedText = null;
    let loadedUrl = "";
    for (const candidate of lessonRequest.candidates) {
      try {
        const response = await fetch(candidate, { cache: "no-cache" });
        if (!response.ok) continue;
        loadedText = await response.text();
        loadedUrl = candidate;
        break;
      } catch (_) {
        // continue
      }
    }
    if (loadedText) {
      stopLoop();
      client = null;
      elEditor.value = loadedText;
      setSourceLabel(`source: ${lessonRequest.sourceLabel}`);
      setStatus(`lesson 로드됨 — ${loadedUrl} — ▶ 실행을 눌러 시작하세요`);
      await runGrammarAnalysis(elEditor.value);
      return;
    }
    setSourceLabel(`source: ${lessonRequest.sourceLabel} (load failed)`);
    setStatus(`lesson 로드 실패 — ${lessonRequest.lessonPath}`, { isError: true });
    return;
  } else if (launchRequest.kind === "example" && launchRequest.example?.requested) {
    const exampleRequest = launchRequest.example;
    const key = exampleRequest.exampleKey;
    const source = EXAMPLES[key];
    if (source) {
      stopLoop();
      client = null;
      elEditor.value = source;
      setSourceLabel(`source: ${exampleRequest.sourceLabel}`);
      setStatus("example 로드됨 — ▶ 실행을 눌러 시작하세요");
      await runGrammarAnalysis(elEditor.value);
      return;
    }
    setSourceLabel(`source: ${exampleRequest.sourceLabel} (not found)`);
    setStatus(`example 로드 실패 — ${key}`, { isError: true });
    return;
  }

  // D2: draft 확인
  const draft = loadDraft();
  if (draft && draft !== EXAMPLES.counter) {
    elEditor.value = EXAMPLES.counter; // 기본값으로 표시
    setSourceLabel("source: draft-pending");
    elDraftBanner?.classList.add("visible");
    setStatus("저장되지 않은 초안이 있습니다 — 복원하시겠습니까?");
    await runGrammarAnalysis(elEditor.value);
    return;
  }

  // 기본 예제
  elEditor.value = EXAMPLES.counter;
  setSourceLabel("source: example:counter");
  setStatus("예제 코드 로드됨 — ▶ 실행을 눌러 시작하세요");
  await runGrammarAnalysis(elEditor.value);
}

init();
