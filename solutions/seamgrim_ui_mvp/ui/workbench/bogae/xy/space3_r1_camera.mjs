// Display adapter for the adopted R1 camera fields. This does not admit DDN,
// validate a face, verify a seal/binding or select vertices from public elements.
// The checked host/face adapter supplies that exact vertex list separately.
const Q = 4294967296n, MIN = -(1n << 63n), MAX = (1n << 63n) - 1n;
const raw = value => {
  if (typeof value !== 'string' || !/^(?:0|-?[1-9][0-9]*)$/.test(value))
    throw new TypeError('표시 좌표의 정본 rawi64 문자열이 필요합니다.');
  const result = BigInt(value);
  if (result < MIN || result > MAX) throw new RangeError('표시 좌표의 rawi64 범위를 벗어났습니다.');
  return result;
};
function triple(value) {
  if (!Array.isArray(value) || value.length !== 3) throw new TypeError('명시한 세 좌표가 필요합니다.');
  return value.map(raw);
}
const subtract = (a, b) => a.map((n, i) => n - b[i]);
const cross = (a, b) => [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]];
function normalized(value) {
  const numbers = value.map(Number), length = Math.hypot(...numbers);
  if (!(Number.isFinite(length) && length > 0)) throw new RangeError('이 카메라 기저를 화면 수치로 표시할 수 없습니다.');
  return numbers.map(n => n / length);
}

export function prepareSpace3R1Frame(view, vertices, viewport, {geometryMode = 'polyline'} = {}) {
  if (view?.schema !== 'ddn.scene_space3_projection.v1'
    || view.kind !== 'scene' || view.spatial_profile !== 'space3'
    || view.profile !== 'ddn.scene.space3.metres.perspective.r1'
    || view.coordinate_encoding !== 'fixed64_raw_decimal.v1' || view.coordinate_unit !== 'm')
    throw new TypeError('채택된 R1 공간 표시 형식이 아닙니다. 다른 보기로 대신하지 않습니다.');
  const camera = view.camera;
  if (camera?.projection !== 'perspective') throw new TypeError('R1의 명시 원근 카메라가 필요합니다.');
  const position = triple(camera.position), look = triple(camera.look_at), up = triple(camera.up);
  const fovRaw = raw(camera.vertical_fov_degrees_raw);
  if (!(fovRaw > 0n && fovRaw < 180n*Q)) throw new RangeError('세로 시야각은 0도보다 크고 180도보다 작아야 합니다.');
  if (!Array.isArray(vertices)) throw new TypeError('생산자가 공급한 꼭짓점 순서가 필요합니다.');
  if (!['polyline', 'face_outlines', 'solid_faces'].includes(geometryMode))
    throw new TypeError('명시한 내부 표시 형태를 지원하지 않습니다.');
  if (!(Number.isFinite(viewport?.width) && Number.isFinite(viewport?.height)
    && viewport.width > 0 && viewport.height > 0)) throw new RangeError('표시 영역의 유한한 크기가 필요합니다.');

  // Subtract original raw integers before display conversion. A small distance
  // near the rawi64 limit must not disappear through Number(worldPosition).
  const forwardRaw = subtract(look, position);
  const forward = normalized(forwardRaw);
  const right = normalized(cross(forwardRaw, up));
  const upright = cross(right, forward);
  const fov = Number(fovRaw) / Number(Q);
  const fy = 1 / Math.tan(fov*Math.PI/360), fx = fy * (viewport.height/viewport.width);
  if (!(Number.isFinite(fx) && Number.isFinite(fy) && fx > 0 && fy > 0))
    throw new RangeError('시야각과 화면 크기를 이 수치 범위로 표시할 수 없습니다.');
  let ordered = vertices;
  const faceRanges = [];
  if (geometryMode === 'face_outlines' || geometryMode === 'solid_faces') {
    ordered = [];
    for (const face of vertices) {
      if (!Array.isArray(face) || face.length < 3)
        throw new TypeError('각 면의 명시한 꼭짓점 셋 이상이 필요합니다.');
      // Display structure only, not coplanarity/simplicity/owner validation.
      face.forEach(triple);
      faceRanges.push({start:ordered.length,count:face.length});
      for (const vertex of face) ordered.push(vertex);
      if (geometryMode === 'face_outlines') ordered.push(face[0]);
      ordered.push(null);
    }
  }
  const samples = ordered.map(vertex => vertex == null ? null
    : subtract(triple(vertex), position).map(n => Number(n)/Number(Q)));
  // Camera-relative internal coordinates. Only forward half-space and side
  // planes clip geometry; no public near/far distance or automatic fit exists.
  return {samples, matrix:[...right.map(n=>fx*n),0,...upright.map(n=>fy*n),0,0,0,0,0,...forward,0],
    clipPlanes:[[1,0,0,1],[-1,0,0,1],[0,1,0,1],[0,-1,0,1],[0,0,0,1]], yDirection:-1,
    renderMode:geometryMode, faceRanges};
}
