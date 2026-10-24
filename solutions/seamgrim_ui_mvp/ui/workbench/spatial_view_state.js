import { collectSpace2dRangePoints } from '../wasm_page_common.js';

// This cache contains display data only, never runtime state or authored payloads.
export function createSpatialViewState({ maxFrames = 512 } = {}) {
  return { identity: null, maxFrames: Math.max(2, Math.floor(maxFrames) || 512),
    frames: [], range: null, initialRange: null, rawRange: null, mode: 'expand' };
}

function bounds(points) {
  let result = null;
  for (const p of points) {
    if (!Number.isFinite(p.x) || !Number.isFinite(p.y)) continue;
    result = result ? { xMin: Math.min(result.xMin, p.x), xMax: Math.max(result.xMax, p.x),
      yMin: Math.min(result.yMin, p.y), yMax: Math.max(result.yMax, p.y) }
      : { xMin: p.x, xMax: p.x, yMin: p.y, yMax: p.y };
  }
  return result;
}

function explicitCameraRange(camera) {
  const values = ['x_min', 'x_max', 'y_min', 'y_max'].map(key => camera?.[key]);
  if (values.some(value => value == null || value === '')) return null;
  const [xMin, xMax, yMin, yMax] = values.map(Number);
  if (![xMin, xMax, yMin, yMax].every(Number.isFinite) || xMax <= xMin || yMax <= yMin) return null;
  return {xMin, xMax, yMin, yMax};
}

function padded(range) {
  if (!range) return null;
  const dx = Math.max(range.xMax - range.xMin, 1) * 0.1;
  const dy = Math.max(range.yMax - range.yMin, 1) * 0.1;
  return { xMin: range.xMin - dx, xMax: range.xMax + dx,
    yMin: range.yMin - dy, yMax: range.yMax + dy };
}

function combined(a, b) {
  if (!a) return b && { ...b };
  if (!b) return { ...a };
  return { xMin: Math.min(a.xMin, b.xMin), xMax: Math.max(a.xMax, b.xMax),
    yMin: Math.min(a.yMin, b.yMin), yMax: Math.max(a.yMax, b.yMax) };
}

function contains(a, b) {
  return a && b && a.xMin <= b.xMin && a.xMax >= b.xMax
    && a.yMin <= b.yMin && a.yMax >= b.yMax;
}

function tracks(drawlist) {
  const topology = JSON.stringify(drawlist.map(p => String(p?.kind ?? '').trim().toLowerCase()));
  const result = new Map();
  const duplicates = new Set();
  drawlist.forEach((p, index) => {
    const kind = String(p?.kind ?? '').trim().toLowerCase();
    if (kind !== 'circle' && kind !== 'point') return;
    const x = Number(p.x ?? p.cx ?? (kind === 'point' ? p[0] : undefined));
    const y = Number(p.y ?? p.cy ?? (kind === 'point' ? p[1] : undefined));
    if (![x, y].every(Number.isFinite)) return;
    if (kind === 'circle' && !Number.isFinite(Number(p.r))) return;
    const identity = p.id ?? p.primitive_id;
    const key = identity == null ? `index:${index}:${kind}` : `id:${String(identity)}`;
    if (result.has(key)) duplicates.add(key);
    result.set(key, { x, y, signature: JSON.stringify([kind,
      kind === 'circle' ? Number(p.r) : null, identity == null ? topology : null]) });
  });
  // Ambiguous explicit identities cannot establish motion correspondence.
  for (const key of duplicates) result.delete(key);
  return result;
}

function display(state, throughIndex) {
  const overlayDrawlist = [];
  for (let i = 1; i <= throughIndex; i += 1) {
    const previous = state.frames[i - 1];
    const current = state.frames[i];
    if (current.madi !== previous.madi + 1) continue;
    for (const [key, point] of current.tracks) {
      const old = previous.tracks.get(key);
      if (!old || old.signature !== point.signature) continue;
      if (old.x === point.x && old.y === point.y) continue;
      overlayDrawlist.push({ kind: 'line', x1: old.x, y1: old.y,
        x2: point.x, y2: point.y, color: '#688fa8', width: 1, opacity: 0.55 });
    }
  }
  return { viewState: { autoFit: false, range: state.range && { ...state.range },
    zoom: 1, panPx: 0, panPy: 0 }, overlayDrawlist };
}

// space2d must be selected by selectCommittedOutputView before reaching this API.
export function updateSpatialView(state, { runId, sourceHash, frame, space2d }) {
  const identity = JSON.stringify([String(runId ?? ''), String(sourceHash ?? '')]);
  if (state.identity !== identity) {
    Object.assign(state, createSpatialViewState({ maxFrames: state.maxFrames }), { identity });
  }
  if (!frame?.state_hash || !Number.isInteger(Number(frame.madi))
      || (frame.source_hash && String(frame.source_hash) !== String(sourceHash))
      || !Array.isArray(space2d?.drawlist)) return display(state, -1);
  const madi = Number(frame.madi);
  const key = JSON.stringify([madi, String(frame.state_hash)]);
  const existing = state.frames.findIndex(f => f.key === key);
  if (existing >= 0) return display(state, existing);
  // A new value at an earlier boundary is a branch, not another history segment.
  if (state.frames.at(-1)?.madi >= madi) {
    state.frames = state.frames.filter(f => f.madi < madi);
    state.rawRange = state.frames.reduce((result, f) => combined(result, f.range), null);
  }
  const range = bounds(space2d.drawlist.flatMap(collectSpace2dRangePoints));
  // 여백이 아니라 실제 점 범위만 누적한다. 이전 화면 여백을 다시 넓히면
  // 긴 곡선에서 반복 padding이 자료와 무관하게 지수적으로 커진다.
  state.rawRange = combined(state.rawRange, range);
  state.frames.push({ key, madi, range, tracks: tracks(space2d.drawlist) });
  if (state.frames.length > state.maxFrames) state.frames.shift();
  if (!state.initialRange) {
    const cameraRange = explicitCameraRange(space2d.camera);
    state.initialRange = cameraRange || padded(range);
    if (cameraRange) state.mode = 'fixed';
  }
  if (!state.range) state.range = state.initialRange && { ...state.initialRange };
  else if (state.mode === 'expand' && range && !contains(state.range, range)) {
    state.range = padded(state.rawRange);
  }
  return display(state, state.frames.length - 1);
}

export function fitSpatialHistory(state) {
  state.rawRange = state.frames.reduce((result, f) => combined(result, f.range), null);
  state.range = padded(state.rawRange);
  state.mode = 'expand';
  return display(state, state.frames.length - 1);
}

export function resetSpatialRange(state) {
  state.range = state.initialRange && { ...state.initialRange };
  state.mode = 'fixed';
  return display(state, state.frames.length - 1);
}
