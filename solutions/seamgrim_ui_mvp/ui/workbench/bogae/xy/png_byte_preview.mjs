// UI-local PNG consumption. A byte preview or A-placement drawing is not a
// checked AssetRef, public scene role, ownership/rights grant or admission.
const signature = [137, 80, 78, 71, 13, 10, 26, 10];
const positive = n => Number.isSafeInteger(n) && n > 0;
const release = bitmap => { if (typeof bitmap?.close === 'function') bitmap.close(); };
const rawCoordinate = value => typeof value === 'string' && value.length <= 20 && /^-?(?:0|[1-9]\d*)$/.test(value)
  && BigInt(value) >= -9223372036854775808n && BigInt(value) <= 9223372036854775807n;

function interval(values) {
  if (!Array.isArray(values) || values.length !== 2 || !values.every(rawCoordinate)
    || BigInt(values[0]) >= BigInt(values[1]))
    throw new Error('그림과 평면의 범위는 공급된 fixed64 원자료의 증가 구간이어야 합니다.');
  return [...values];
}

// Internal drawing arguments, not a new producer JSON schema. Plane fields
// reuse the existing plane2 view encoding; image bounds are explicit inputs.
function placement(options, limits) {
  if (options.plane === undefined && options.horizontal === undefined && options.vertical === undefined) return null;
  const plane = options.plane;
  if (plane?.kind !== 'scene' || plane.spatial_profile !== 'plane2'
    || plane.coordinate_encoding !== 'fixed64_raw_decimal.v1')
    throw new Error('그림 배치에 필요한 실제 plane2 좌표 공급을 확인하세요.');
  const width = options.width ?? 640, height = options.height ?? 360;
  if (![width, height].every(positive) || width <= 36 || height <= 36
    || width > limits.maxDimension || height > limits.maxDimension || width > Math.floor(limits.maxPixels / height))
    throw new Error('그림 배치 화면의 크기 한도를 확인하세요.');
  return {width, height, horizontal: interval(options.horizontal), vertical: interval(options.vertical),
    planeHorizontal: interval(plane.horizontal), planeVertical: interval(plane.vertical)};
}

function drawInPlane(context, bitmap, area, camera) {
  const [xmin, xmax] = area.planeHorizontal.map(BigInt), [ymin, ymax] = area.planeVertical.map(BigInt);
  const [left, right] = area.horizontal.map(BigInt), [bottom, top] = area.vertical.map(BigInt);
  const sx = Number(xmax - xmin), sy = Number(ymax - ymin);
  const scale = Math.min((area.width - 36) / sx, (area.height - 36) / sy) * camera.zoom;
  // Subtract in exact integers before screen-only conversion, including near
  // i64 extrema. Neither screen pixels nor this scale become world values.
  const x = (area.width - sx * scale) / 2 + Number(left - xmin) * scale + camera.x;
  const y = (area.height + sy * scale) / 2 - Number(top - ymin) * scale + camera.y;
  context.save();context.beginPath();context.rect(18, 18, area.width - 36, area.height - 36);context.clip();
  // A: full source bitmap fills the authored rectangle, never image auto-fit.
  context.drawImage(bitmap, x, y, Number(right - left) * scale, Number(top - bottom) * scale);
  context.restore();
}

// Internal object-reference association only. The host supplies the original
// Rust view and separately checked bytes. No index/label becomes asset identity
// and this local shape is never serialized as a new projection or save wire.
function mixedScene(plane, layers, maxElements, maxVertices) {
  if (!Array.isArray(plane?.elements)) throw new Error('함께 표시할 실제 장면 요소가 없습니다.');
  if (plane.elements.length > maxElements) throw new Error('함께 표시할 장면 요소 한도를 넘었습니다.');
  let vertices = 0;
  const seen = new Set();
  const point = value => {
    if (!Array.isArray(value) || value.length !== 2 || !value.every(rawCoordinate))
      throw new Error('장면 좌표 원자료를 확인하세요.');
    return [...value];
  };
  const order = plane.elements.map(element => {
    if (element?.kind === 'image') {
      const index = layers.findIndex(layer => layer?.element === element);
      if (index < 0 || seen.has(index)) throw new Error('장면 그림과 별도 공급 바이트의 연결을 확인하세요.');
      seen.add(index);
      const horizontal = interval(element.horizontal), vertical = interval(element.vertical);
      if (horizontal.some((value,i) => value !== layers[index].horizontal?.[i])
        || vertical.some((value,i) => value !== layers[index].vertical?.[i]))
        throw new Error('그림의 실제 장면 범위와 바이트 배치 범위가 다릅니다.');
      return {kind: 'image', index};
    }
    if (element?.kind === 'circle') {
      if (!rawCoordinate(element.radius) || BigInt(element.radius) < 0n)
        throw new Error('원의 반지름 원자료를 확인하세요.');
      return {kind: 'circle', center: point(element.center), radius: element.radius};
    }
    if (element?.kind === 'polygon') {
      if (!Array.isArray(element.vertices) || element.vertices.length < 3)
        throw new Error('다각형 꼭짓점 원자료를 확인하세요.');
      vertices += element.vertices.length;
      if (vertices > maxVertices) throw new Error('함께 표시할 장면 꼭짓점 한도를 넘었습니다.');
      return {kind: 'polygon', vertices: element.vertices.map(point)};
    }
    if (element?.kind === 'text' && typeof element.content === 'string')
      return {kind: 'text', position: point(element.position), content: element.content};
    throw new Error('이 장면 요소의 표시는 지원하지 않습니다.');
  });
  if (seen.size !== layers.length) throw new Error('장면에 없는 그림 바이트는 함께 표시하지 않습니다.');
  return order;
}

function drawPrimitive(context, element, area, camera) {
  const [xmin,xmax] = area.planeHorizontal.map(BigInt), [ymin,ymax] = area.planeVertical.map(BigInt);
  const sx = Number(xmax-xmin), sy = Number(ymax-ymin);
  const scale = Math.min((area.width-36)/sx, (area.height-36)/sy) * camera.zoom;
  const x = raw => (area.width-sx*scale)/2 + Number(BigInt(raw)-xmin)*scale + camera.x;
  const y = raw => (area.height+sy*scale)/2 - Number(BigInt(raw)-ymin)*scale + camera.y;
  context.save();context.beginPath();context.rect(18,18,area.width-36,area.height-36);context.clip();
  context.strokeStyle = '#215b8b';context.fillStyle = '#132018';context.lineWidth = 1.5;
  if (element.kind === 'circle') {
    context.beginPath();context.ellipse(x(element.center[0]),y(element.center[1]),
      Number(element.radius)*scale,Number(element.radius)*scale,0,0,2*Math.PI);context.stroke();
  } else if (element.kind === 'polygon') {
    context.beginPath();element.vertices.forEach((p,index) => index ? context.lineTo(x(p[0]),y(p[1])) : context.moveTo(x(p[0]),y(p[1])));
    context.closePath();context.stroke();
  } else if (element.kind === 'text') context.fillText(element.content,x(element.position[0]),y(element.position[1]));
  context.restore();
}

function dimensions(bytes, limits) {
  if (!(bytes instanceof Uint8Array) || bytes.length > limits.maxBytes)
    throw new Error('PNG 바이트 종류 또는 원본 확인 크기 한도를 확인하세요.');
  if (bytes.length < 33 || signature.some((n, i) => bytes[i] !== n))
    throw new Error('PNG 원본 바이트가 없거나 머리말이 불완전합니다.');
  const data = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  if (data.getUint32(8) !== 13 || String.fromCharCode(...bytes.subarray(12, 16)) !== 'IHDR')
    throw new Error('PNG 첫 크기 정보가 올바르지 않습니다.');
  const width = data.getUint32(16), height = data.getUint32(20);
  if (!positive(width) || !positive(height) || width > limits.maxDimension
    || height > limits.maxDimension || width > Math.floor(limits.maxPixels / height))
    throw new Error('PNG 원본 확인의 해독 크기 한도를 넘었습니다.');
  // APNG can decode to a still first frame in browsers. Do not report that
  // as supported static PNG. Inspect chunk boundaries, never payload strings;
  // native decode retains responsibility for CRC and general PNG validity.
  for (let offset = 8; offset <= bytes.length - 12;) {
    const length = data.getUint32(offset);
    const kind = String.fromCharCode(...bytes.subarray(offset + 4, offset + 8));
    if (['acTL', 'fcTL', 'fdAT'].includes(kind))
      throw new Error('움직이는 PNG(APNG)는 정적 PNG 원본 확인에서 지원하지 않습니다.');
    if (length > bytes.length - offset - 12 || kind === 'IEND') break;
    offset += length + 12;
  }
  // Header inspection bounds allocation; native decode still validates PNG.
  return {width, height};
}

async function nativeDecode(bytes) {
  if (typeof globalThis.createImageBitmap !== 'function')
    throw new Error('이 환경은 PNG 원본 확인 해독을 지원하지 않습니다.');
  return globalThis.createImageBitmap(new Blob([bytes], {type: 'image/png'}));
}

export function mountPngBytePreview(root, {decode = nativeDecode,
  maxBytes = 16 * 1024 * 1024, maxPixels = 16 * 1024 * 1024,
  maxDimension = 16384, maxImages = 256, maxElements = 4096, maxVertices = 65536} = {}) {
  if (!root?.ownerDocument || typeof decode !== 'function'
    || ![maxBytes, maxPixels, maxDimension, maxImages, maxElements, maxVertices].every(positive))
    throw new TypeError('PNG 원본 확인 영역과 양의 크기 한도가 필요합니다.');
  const limits = {maxBytes, maxPixels, maxDimension}, document = root.ownerDocument;
  const owner = document.createElement('figure');owner.dataset.pngBytePreview = 'true';
  owner.style.minWidth = '0';owner.style.maxWidth = '100%';root.append(owner);
  const caption = document.createElement('figcaption');
  caption.textContent = '공급 PNG 원본 확인 · 장면 배치·자산 권한 검증과 별개';owner.append(caption);
  const status = document.createElement('p');status.setAttribute('role', 'status');
  status.setAttribute('aria-live', 'polite');owner.append(status);
  const body = document.createElement('div');owner.append(body);
  const controls = document.createElement('div');controls.setAttribute('role', 'group');
  controls.setAttribute('aria-label', '그림 화면 조작');controls.hidden = true;owner.append(controls);
  const controlListeners = [], buttons = [];
  function control(text, action) {
    const button = document.createElement('button');button.type = 'button';button.textContent = text;
    button.style.minHeight = '44px';button.disabled = true;controls.append(button);
    button.addEventListener('click', action);controlListeners.push([button, action]);buttons.push(button);
  }
  control('그림 확대', () => setCamera({zoom: camera.zoom * 1.25}));
  control('그림 축소', () => setCamera({zoom: camera.zoom / 1.25}));
  control('원래 화면', () => setCamera({x: 0, y: 0, zoom: 1}));
  const cameraStatus = document.createElement('p');controls.append(cameraStatus);
  const help = document.createElement('p');help.textContent = '그림에 초점을 두고 화살표로 이동, +/−로 확대·축소, 0으로 복귀합니다. 화면만 바뀝니다.';controls.append(help);
  let generation = 0, disposed = false, pending = null, current = [], frame = null, detachCanvas = null, detachView = null;
  let camera = {x: 0, y: 0, zoom: 1};
  function paint(context, canvas, plan, bitmaps, scene) {
    context.clearRect(0, 0, canvas.width, canvas.height);
    if (scene) {
      scene.forEach(element => element.kind === 'image'
        ? drawInPlane(context, bitmaps[element.index], plan[element.index].area, camera)
        : drawPrimitive(context, element, plan[0].area, camera));
      return;
    }
    plan.forEach((item, index) => {
      if (item.area) drawInPlane(context, bitmaps[index], item.area, camera);else context.drawImage(bitmaps[index], 0, 0);
    });
  }
  function setCamera(next = {}) {
    if (disposed || !frame?.plan[0].area || !next || typeof next !== 'object') return false;
    const value = {x: next.x ?? camera.x, y: next.y ?? camera.y, zoom: next.zoom ?? camera.zoom};
    if (!Object.values(value).every(Number.isFinite) || value.zoom < 0.125 || value.zoom > 8
      || Math.abs(value.x) > 1000000 || Math.abs(value.y) > 1000000) return false;
    camera = value;
    try { paint(frame.context, frame.canvas, frame.plan, current, frame.scene); }
    catch (error) { retire();outcome('decode_error', `그림 화면을 갱신하지 못했습니다. ${error?.message ?? String(error)}`);return false; }
    cameraStatus.textContent = `화면 확대 ${Math.round(camera.zoom * 100)}% · 원자료와 실행은 바뀌지 않습니다.`;
    buttons[0].disabled = camera.zoom * 1.25 > 8;buttons[1].disabled = camera.zoom / 1.25 < 0.125;
    return true;
  }
  function outcome(kind, text) { owner.dataset.pngPreviewState = kind;status.textContent = text;return kind; }
  function retire() {
    generation++;
    detachView?.();detachView = null;
    if (pending) { const previous = pending;pending = null;previous.detach();previous.releaseDecoded(); }
    detachCanvas?.();detachCanvas = null;frame = null;camera = {x: 0, y: 0, zoom: 1};
    controls.hidden = true;buttons.forEach(button => {button.disabled = true;});cameraStatus.textContent = '';
    body.replaceChildren();
    caption.textContent = '공급 PNG 원본 확인 · 장면 배치·자산 권한 검증과 별개';
    const previous = current;current = [];previous.forEach(release);
  }
  function clear() {
    if (disposed) return;
    retire();outcome('not_produced', '확인할 PNG 원본이 없습니다. 이전 그림은 폐기했습니다.');
  }
  async function show(input, options = {}) {
    if (disposed) return null;
    const {signal} = options;
    retire();const request = generation;
    if (signal?.aborted) return outcome('cancelled', 'PNG 원본 확인을 취소했습니다.');
    const batch = Array.isArray(input);
    let plan, scene = null, sceneRecord = null;
    try {
      if (signal != null && (typeof signal.addEventListener !== 'function' || typeof signal.removeEventListener !== 'function'))
        throw new Error('취소 신호를 확인하세요.');
      if (batch && input.length === 0) return outcome('not_produced', '공급된 그림이 없습니다. 이전 그림은 폐기했습니다.');
      if (batch && input.length > maxImages) throw new Error('한 화면의 그림 개수 한도를 넘었습니다.');
      const sources = batch ? input : [{bytes: input, horizontal: options.horizontal, vertical: options.vertical}];
      if (options.scene === true) {
        if (!batch) throw new Error('함께 표시할 장면은 원자료 요소와 그림 바이트 묶음이 필요합니다.');
        scene = mixedScene(options.plane, sources, maxElements, maxVertices);
        sceneRecord = JSON.stringify(options.plane, null, 2);
      }
      let totalBytes = 0, totalPixels = 0;
      plan = sources.map(layer => {
        const bytes = layer?.bytes, size = dimensions(bytes, limits);
        const area = placement({...options, horizontal: layer.horizontal, vertical: layer.vertical}, limits);
        if (batch && !area) throw new Error('여러 그림은 같은 평면과 각각의 명시 범위가 필요합니다.');
        const pixels = size.width * size.height;
        if (bytes.length > maxBytes - totalBytes || pixels > maxPixels - totalPixels)
          throw new Error('그림 묶음의 전체 바이트 또는 해독 픽셀 한도를 넘었습니다.');
        totalBytes += bytes.length;totalPixels += pixels;
        return {size, area, bytes};
      });
      // Validate the whole batch before allocating copies or starting decode.
      plan = plan.map(item => ({...item, bytes: new Uint8Array(item.bytes)}));
    } catch (error) { return outcome('rejected', error.message); }
    outcome('loading', `공급된 PNG ${plan.length}개를 해독하는 중입니다. 이전 그림은 폐기했습니다.`);
    const abort = () => {
      if (!disposed && request === generation) { retire();outcome('cancelled', 'PNG 원본 확인을 취소했습니다.'); }
    };
    signal?.addEventListener('abort', abort, {once: true});
    const detach = () => signal?.removeEventListener('abort', abort);
    const decoded = [];
    let retainAbort = false;
    const releaseDecoded = () => decoded.splice(0).forEach(release);
    pending = {detach, releaseDecoded};
    try {
      // Decode serially to bound in-flight allocations. Completion timing
      // never reorders supplied layers, and no partial canvas is published.
      for (const item of plan) {
        const bitmap = await decode(item.bytes);
        if (disposed || request !== generation) { release(bitmap);releaseDecoded();return null; }
        if (!bitmap || typeof bitmap.close !== 'function' || bitmap.width !== item.size.width || bitmap.height !== item.size.height) {
          release(bitmap);throw new Error('해독된 PNG 크기가 공급된 머리말과 일치하지 않습니다.');
        }
        decoded.push(bitmap);
      }
      const {size, area} = plan[0];
      const canvas = document.createElement('canvas');canvas.width = area?.width ?? size.width;canvas.height = area?.height ?? size.height;
      canvas.style.maxWidth = '100%';canvas.style.height = 'auto';
      canvas.setAttribute('role', 'img');
      canvas.setAttribute('aria-label', area ? '그림 배치 · 공급된 정확 범위는 원자료에서 확인 · 화면 좌표는 근사'
        : `공급 PNG 원본 ${size.width} × ${size.height} 픽셀 · 세계 좌표 아님`);
      const context = canvas.getContext('2d');
      if (!context) throw new Error('이 환경은 PNG 원본 확인 화면을 지원하지 않습니다.');
      paint(context, canvas, plan, decoded, scene);
      body.append(canvas);
      if (area) {
        const details = document.createElement('details'), summary = document.createElement('summary');
        summary.textContent = '공급된 배치·평면 범위 원자료';details.append(summary);
        const original = document.createElement('pre');original.textContent = sceneRecord ?? JSON.stringify(batch ? {
          coordinate_encoding: 'fixed64_raw_decimal.v1', plane_horizontal: area.planeHorizontal, plane_vertical: area.planeVertical,
          images_in_supplied_order: plan.map(item => ({horizontal: item.area.horizontal, vertical: item.area.vertical}))
        } : {coordinate_encoding: 'fixed64_raw_decimal.v1', image_horizontal: area.horizontal, image_vertical: area.vertical,
          plane_horizontal: area.planeHorizontal, plane_vertical: area.planeVertical}, null, 2);
        original.style.whiteSpace = 'pre-wrap';original.style.overflowWrap = 'anywhere';details.append(original);body.append(details);
      }
      caption.textContent = scene ? '장면·그림 함께 보기 · 별도 공급 바이트 표시이며 자산 권한 검증과 별개'
        : area ? 'A안 그림 배치 · 자산 권한·공개 장면 연결과 별개'
        : '공급 PNG 원본 확인 · 장면 배치·자산 권한 검증과 별개';
      // Keep the complete batch until this owner is retired.
      current.push(...decoded.splice(0));
      if (area) {
        frame = {context, canvas, plan, scene};canvas.tabIndex = 0;
        const keyboard = event => {
          if (disposed || request !== generation || frame?.canvas !== canvas || event.isComposing || event.ctrlKey || event.altKey || event.metaKey) return;
          const next = event.key === 'ArrowLeft' ? {x: camera.x - 16} : event.key === 'ArrowRight' ? {x: camera.x + 16}
            : event.key === 'ArrowUp' ? {y: camera.y - 16} : event.key === 'ArrowDown' ? {y: camera.y + 16}
            : ['+', '='].includes(event.key) ? {zoom: camera.zoom * 1.25}
            : event.key === '-' ? {zoom: camera.zoom / 1.25} : event.key === '0' ? {x: 0, y: 0, zoom: 1} : null;
          if (next) { setCamera(next);event.preventDefault();event.stopPropagation(); }
        };
        canvas.addEventListener('keydown', keyboard);detachCanvas = () => canvas.removeEventListener('keydown', keyboard);
        controls.hidden = false;buttons.forEach(button => {button.disabled = false;});
        cameraStatus.textContent = '화면 확대 100% · 원자료와 실행은 바뀌지 않습니다.';
      }
      retainAbort = Boolean(signal);detachView = retainAbort ? detach : null;
      return outcome('ready', area ? `공급 순서대로 그림 ${plan.length}개를 사각형에 배치했습니다. 화면은 근사이며 원자료는 보존합니다.`
        : `PNG 원본 ${size.width} × ${size.height} 픽셀 · 장면 배치 미연결`);
    } catch (error) {
      releaseDecoded();
      if (disposed || request !== generation) return null;
      body.replaceChildren();return outcome('decode_error', `PNG 원본을 해독하지 못했습니다. ${error?.message ?? String(error)}`);
    } finally {
      if (!retainAbort) detach();if (request === generation) pending = null;
    }
  }
  function dispose() {
    if (disposed) return;retire();disposed = true;
    controlListeners.forEach(([button, action]) => button.removeEventListener('click', action));owner.remove();
  }
  clear();
  return {show, clear, dispose, setCamera};
}
