import {projectPolylineToViewport} from './space3_projection_math.mjs';
import {rasterizeSpace3Faces} from './space3_face_raster.mjs';

/**
 * Dedicated internal display workflow. readFrame supplies already-adapted
 * {samples,matrix,clipPlanes,yDirection}, or null on invalidation. It does not
 * read/admit public space3 JSON, select axes or manufacture source/run/owner
 * identity. The future checked-carrier adapter owns that boundary.
 */
export function mountSpace3PolylineView(root, {readFrame, subscribeFrame, prepareFrame} = {}) {
  if (!root?.ownerDocument || typeof readFrame !== 'function'
    || (subscribeFrame !== undefined && typeof subscribeFrame !== 'function')
    || (prepareFrame !== undefined && typeof prepareFrame !== 'function'))
    throw new TypeError('공간 표시 영역과 현재 표시 자료 읽기가 필요합니다.');
  const document = root.ownerDocument, window = document.defaultView;
  const container = document.createElement('section');
  container.style.minWidth = '0'; container.style.minHeight = '0'; container.style.height = '100%';
  container.style.display = 'grid'; container.style.gridTemplateRows = 'minmax(0, 1fr) auto';
  const canvas = document.createElement('canvas');
  canvas.style.width = '100%'; canvas.style.height = '100%'; canvas.style.minWidth = '0';
  canvas.setAttribute('role', 'img'); canvas.setAttribute('aria-label', '명시 변환으로 그린 공간 좌표');
  const status = document.createElement('p');
  status.setAttribute('role', 'status'); status.setAttribute('aria-live', 'polite');
  container.append(canvas, status); root.append(container);
  let disposed = false, generation = 0, current = null, unsubscribe = null, observer = null, contextLost = false;
  let resolution = null, resizeFrame = null;
  function scheduleDraw() {
    if (disposed || resizeFrame !== null) return;
    resizeFrame = window.requestAnimationFrame(() => {
      resizeFrame = null;
      draw();
    });
  }
  function observeResolution() {
    resolution?.removeEventListener('change', resolutionChanged);
    resolution = window?.matchMedia?.(`(resolution: ${window.devicePixelRatio}dppx)`) ?? null;
    resolution?.addEventListener('change', resolutionChanged);
  }
  function resolutionChanged() {
    observeResolution();
    scheduleDraw();
  }
  function setStatus(state, message) {
    container.dataset.space3DisplayState = state;
    status.textContent = message;
  }
  function clear() {
    const context = canvas.getContext('2d');
    if (context) {
      context.setTransform(1, 0, 0, 1, 0, 0);
      context.clearRect(0, 0, canvas.width, canvas.height);
    }
  }
  function draw() {
    if (disposed || !current) return;
    clear();
    if (contextLost) {setStatus('context_lost', '그리기 연결이 끊겼습니다. 이전 그림은 표시하지 않습니다.'); return;}
    const rect = canvas.getBoundingClientRect();
    if (!(Number.isFinite(rect.width) && Number.isFinite(rect.height) && rect.width > 0 && rect.height > 0)) {
      setStatus('waiting_viewport', '표시할 화면 영역을 기다립니다.'); return;
    }
    const ratio = window?.devicePixelRatio;
    const dpr = Number.isFinite(ratio) && ratio > 0 ? ratio : 1;
    const width = Math.round(rect.width * dpr), height = Math.round(rect.height * dpr);
    if (!Number.isSafeInteger(width) || !Number.isSafeInteger(height) || width <= 0 || height <= 0) {
      setStatus('unsupported', '현재 화면 크기로 그릴 수 없습니다.'); return;
    }
    try {
      const viewport = {x:0, y:0, width:rect.width, height:rect.height};
      // Rebuild display matrices for this viewport without rereading source or
      // changing the retained explicit camera (e.g. preserve vertical FOV).
      const frame = prepareFrame ? prepareFrame(current, viewport) : current;
      if (frame.renderMode === 'solid_faces') {
        // Calculate privately and publish once. A display resource failure
        // cannot leave a partially filled scene or revive the previous image.
        const rendered = rasterizeSpace3Faces(frame, width, height);
        canvas.width = width; canvas.height = height;
        const context = canvas.getContext('2d');
        if (!context) {setStatus('unsupported', '이 화면의 그리기 기능을 사용할 수 없습니다.'); return;}
        const image = context.createImageData(width, height);
        image.data.set(rendered.pixels);
        context.setTransform(1, 0, 0, 1, 0, 0);
        context.putImageData(image, 0, 0);
        canvas.setAttribute('aria-label', '명시 카메라로 그린 불투명 공간 면');
        const counts = `면 채움 · 보이는 픽셀 ${rendered.paintedPixels}개 · 깊이 가림 적용`;
        if (rendered.limitations.length) setStatus('display_limited', `${counts} · 표시 계산 제한 ${rendered.limitations.length}건. 원관측의 성공이나 결측 판정이 아닙니다.`);
        else if (!frame.faceRanges.length) setStatus('empty_geometry', '공급된 표시 면이 없습니다.');
        else if (!rendered.paintedPixels && rendered.triangleCount)
          setStatus('no_pixel_coverage', '면 자료는 공급됐지만 현재 해상도에서 덮는 픽셀이 없습니다. 자료의 부재나 시야 밖을 뜻하지 않습니다.');
        else if (!rendered.paintedPixels && rendered.zeroProjectedAreaCount)
          setStatus('zero_projected_area', '면 자료는 공급됐지만 현재 화면의 투영 면적이 0입니다. 원래 면의 넓이가 0이라는 뜻은 아닙니다.');
        else if (!rendered.paintedPixels) setStatus('outside_view', '명시한 표시 범위 안에 그릴 면이 없습니다.');
        else setStatus('displayed', `${counts}. 화면의 가림은 원자료의 부재를 뜻하지 않습니다.`);
        return;
      }
      const projected = projectPolylineToViewport(frame.samples, {...frame, viewport});
      canvas.width = width; canvas.height = height;
      const context = canvas.getContext('2d');
      if (!context) {setStatus('unsupported', '이 화면의 그리기 기능을 사용할 수 없습니다.'); return;}
      context.setTransform(width / viewport.width, 0, 0, height / viewport.height, 0, 0);
      context.strokeStyle = '#215b8b'; context.fillStyle = '#215b8b'; context.lineWidth = 1.5;
      for (const segment of projected.segments) {
        context.beginPath(); context.moveTo(...segment.start); context.lineTo(...segment.end); context.stroke();
      }
      for (const point of frame.renderMode === 'face_outlines' ? [] : projected.points) {
        context.beginPath(); context.arc(...point.position, 3, 0, Math.PI * 2); context.fill();
      }
      const counts = frame.renderMode === 'face_outlines'
        ? `면 윤곽선 · 화면 안 선분 ${projected.segments.length}개 · 면 채움과 가림 표현은 미제공`
        : `화면 안 점 ${projected.points.length}개 · 선분 ${projected.segments.length}개`;
      if (projected.limitations.length) setStatus('display_limited', `${counts} · 표시 계산 제한 ${projected.limitations.length}건. 관측값의 결측이나 계산 성공을 뜻하지 않습니다.`);
      else if (!frame.samples.length) setStatus('empty_geometry', '공급된 표시 좌표가 없습니다.');
      else if (!projected.points.length && !projected.segments.length) setStatus('outside_view', '명시한 표시 범위 안에 그릴 좌표가 없습니다.');
      else setStatus('displayed', `${counts}. 화면 좌표는 원관측값이 아닙니다.`);
    } catch (error) {
      clear(); setStatus('unsupported', `표시 자료를 그릴 수 없습니다. ${error?.message ?? String(error)}`);
    } finally {
      const next = canvas.getBoundingClientRect();
      if (Math.round(next.width * dpr) !== width || Math.round(next.height * dpr) !== height)
        scheduleDraw();
    }
  }
  async function refresh() {
    if (disposed) return null;
    const request = ++generation;
    current = null; clear(); setStatus('loading', '현재 표시 자료를 읽는 중입니다. 이전 그림은 폐기했습니다.');
    try {
      const frame = await readFrame();
      if (disposed || request !== generation) return null;
      if (frame == null) {setStatus('not_produced', '현재 공간 표시 자료가 없습니다. · NOT_PRODUCED'); return 'not_produced';}
      // Retain a private numeric snapshot. Resizing never rereads or mutates a
      // producer object and cannot silently adopt subsequent in-place edits.
      current = structuredClone(frame);
      draw();
      return container.dataset.space3DisplayState;
    } catch (error) {
      if (disposed || request !== generation) return null;
      current = null; clear(); setStatus('read_error', `표시 자료를 읽지 못했습니다. ${error?.message ?? String(error)}`);
      return 'read_error';
    }
  }
  function lost(event) {event.preventDefault(); contextLost = true; clear(); setStatus('context_lost', '그리기 연결이 끊겼습니다.');}
  function restored() {
    contextLost = false;
    if (current) draw();
    // No automatic state read/run/save on context restoration.
  }
  canvas.addEventListener('contextlost', lost);
  canvas.addEventListener('contextrestored', restored);
  function dispose() {
    if (disposed) return;
    disposed = true; ++generation; current = null;
    canvas.removeEventListener('contextlost', lost); canvas.removeEventListener('contextrestored', restored);
    observer?.disconnect();
    if (resizeFrame !== null) window.cancelAnimationFrame(resizeFrame);
    resolution?.removeEventListener('change', resolutionChanged);
    clear();
    try {unsubscribe?.();} finally {container.remove();}
  }
  try {
    if (window?.ResizeObserver) {
      observer = new window.ResizeObserver(scheduleDraw);
      observer.observe(canvas);
    }
    observeResolution();
    if (subscribeFrame) {
      unsubscribe = subscribeFrame(refresh);
      if (unsubscribe !== undefined && typeof unsubscribe !== 'function') {
        unsubscribe = null; throw new TypeError('표시 구독 해제 함수가 필요합니다.');
      }
    }
    void refresh();
  } catch (error) {dispose(); throw error;}
  return {refresh, dispose};
}
