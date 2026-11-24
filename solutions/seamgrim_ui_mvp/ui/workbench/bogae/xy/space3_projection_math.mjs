// Internal display arithmetic, not a space3 wire format or a camera provider.
// A future checked adapter must supply the matrix, clip half-spaces and axis
// orientation explicitly. No camera, bounds, unit or missing coordinate is
// inferred here. Floating-point results are screen positions, never evidence.
const vector = (value, size) => (Array.isArray(value) || ArrayBuffer.isView(value))
  && value.length === size && Array.from(value).every(Number.isFinite);
const dot = (a, b) => a.reduce((sum, n, i) => sum + n * b[i], 0);
const mix = (a, b, t) => a.map((n, i) => (1 - t) * n + t * b[i]);

function intersectionFraction(a, b) {
  // Opposite signs. Scaling avoids overflow in a-b for large finite values.
  const scale = Math.max(Math.abs(a), Math.abs(b));
  const left = Math.abs(a) / scale, right = Math.abs(b) / scale;
  return left / (left + right);
}

function clipSegment(a, b, planes) {
  let enter = 0, leave = 1;
  for (const plane of planes) {
    const left = dot(plane, a), right = dot(plane, b);
    if (!Number.isFinite(left) || !Number.isFinite(right)) return {limitation: 'non_finite_clip_distance'};
    if (left < 0 && right < 0) return null;
    if (left < 0) enter = Math.max(enter, intersectionFraction(left, right));
    else if (right < 0) leave = Math.min(leave, intersectionFraction(left, right));
    if (enter > leave) return null;
  }
  // A zero-width parameter interval has no drawable line. This also avoids
  // inventing a collapsed interior line when extreme magnitudes round two
  // distinct crossings to the same floating-point parameter.
  if (enter === leave) return {limitation: 'zero_width_clip_interval'};
  return [mix(a, b, enter), mix(a, b, leave)];
}

/**
 * Project one already-admitted internal polyline for drawing.
 * matrix is row-major and multiplies column vectors [x,y,z,1]; that is an
 * internal calculation convention, not a declaration of public camera units,
 * handedness, projection enum or depth range. Each explicit clip plane keeps
 * dot(plane, clipPosition) >= 0. yDirection is +1 or -1, never inferred.
 *
 * Null/non-finite/malformed samples break the path. Adjacent supplied samples
 * alone form segments; clipping must never reconnect across a missing sample.
 * index/fromIndex/toIndex are array positions for drawing, not object/sample
 * identities. The caller retains all canonical values and provenance.
 */
export function projectPolylineToViewport(samples, {matrix, clipPlanes, viewport, yDirection} = {}) {
  if (!Array.isArray(samples)) throw new TypeError('표시할 좌표열이 필요합니다.');
  if (!vector(matrix, 16)) throw new TypeError('유한한 명시 변환 행렬이 필요합니다.');
  if (!Array.isArray(clipPlanes) || clipPlanes.length === 0
    || !clipPlanes.every(plane => vector(plane, 4) && Array.from(plane).some(n => n !== 0)))
    throw new TypeError('유효한 명시 자름 평면들이 필요합니다.');
  if (!viewport || !['x', 'y', 'width', 'height'].every(key => Number.isFinite(viewport[key]))
    || viewport.width <= 0 || viewport.height <= 0 || ![1, -1].includes(yDirection))
    throw new TypeError('유한한 화면 영역과 명시 세로 방향이 필요합니다.');

  const rows = Array.from({length: 4}, (_, i) => Array.from(matrix).slice(i * 4, i * 4 + 4));
  const planes = clipPlanes.map(plane => Array.from(plane));
  const limitations = [];
  const transformed = samples.map((point, index) => {
    if (!vector(point, 3)) {
      if (point != null) limitations.push({kind:'point', index, reason:'invalid_coordinate'});
      return null;
    }
    const homogeneous = [...point, 1];
    const result = rows.map(row => dot(row, homogeneous));
    if (result.every(Number.isFinite)) return result;
    limitations.push({kind:'point', index, reason:'non_finite_transform'});
    return null;
  });
  function screen(point) {
    // No arbitrary epsilon or division of a point on/behind the camera.
    if (!(point[3] > 0)) return null;
    const x = viewport.x + viewport.width / 2 + (point[0] / point[3]) * viewport.width / 2;
    const y = viewport.y + viewport.height / 2 + yDirection * (point[1] / point[3]) * viewport.height / 2;
    return Number.isFinite(x) && Number.isFinite(y) ? [x, y] : null;
  }
  const points = [], segments = [];
  for (let i = 0; i < transformed.length; i++) {
    const point = transformed[i];
    if (!point) continue;
    const inside = planes.every(plane => {
      const distance = dot(plane, point);
      return Number.isFinite(distance) && distance >= 0;
    });
    const visible = inside ? screen(point) : null;
    if (visible) points.push({index: i, position: visible});
    else if (inside) limitations.push({kind:'point', index:i, reason:'non_projectable_screen_position'});
    const previous = transformed[i - 1];
    if (!previous) continue;
    const clipped = clipSegment(previous, point, planes);
    if (!clipped) continue;
    if (clipped.limitation) {
      limitations.push({kind:'segment', fromIndex:i-1, toIndex:i, reason:clipped.limitation});
      continue;
    }
    const start = screen(clipped[0]), end = screen(clipped[1]);
    if (start && end) segments.push({fromIndex: i - 1, toIndex: i, start, end});
    else limitations.push({kind:'segment', fromIndex:i-1, toIndex:i, reason:'non_projectable_screen_position'});
  }
  // Presentation limitations must not be mistaken for a successful empty
  // observation or suppressed producer diagnostics by the future adapter.
  return {points, segments, limitations};
}
