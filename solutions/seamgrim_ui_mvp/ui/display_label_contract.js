const DISPLAY_LABELS = Object.freeze({
  id: "식별자",
  name: "이름",
  title: "제목",
  desc: "설명",
  description: "설명",
  source: "출처",
  schema: "꼴",
  status: "상태",
  expected: "기대값",
  actual: "실제값",
  available: "사용 가능",
  strict: "엄격",
  unknown: "알 수 없음",
  off: "꺼짐",
  ok: "정상",
  fail: "실패",
  error: "오류",
  warn: "경고",
  warning: "경고",
  input: "입력",
  output: "출력",
  row: "행",
  rows: "행",
  column: "열",
  columns: "열",
  table: "표",
  "table.row": "보임표 행",
  graph: "그래프",
  structure: "구조",
  space2d: "보개",
  "2d": "보개",
  text: "글",
  overlay: "겹보기",
  resource: "자원",
  resources: "자원",
  fixed64: "수",
  value: "값",
  json: "JSON",
  value_json: "값 JSON",
  handle: "핸들",
  component: "구성요소",
  host: "호스트",
  ext: "확장",
  exts: "확장",
  host_exts: "호스트 확장",
  drawlist: "그림목록",
  draw: "그림",
  view: "보기",
  views: "보기",
  family: "보기 종류",
  families: "보기 종류",
  contract: "계약",
  "view_contract": "보기 계약",
  "bridge_check": "연결 확인",
  "ddn_meta": "DDN 메타",
  "required_views": "필수 보기",
  "state_hash": "상태 해시",
  state: "상태",
  hash: "해시",
  tick: "마디",
  t: "시간",
  playing: "실행 중",
  speed: "속도",
  "source_revision_id": "원본 리비전",
  "source_revision_required": "원본 리비전 필요",
  "observation_output": "관찰 출력",
  "observation-output-lines": "관찰 출력 줄",
  "observation-fallback": "관찰 대체값",
  "resource_value_json": "자원 값 JSON",
  "patch_value_json": "패치 값 JSON",
  "api_run": "API 실행",
  "official": "공식",
  "federated": "통합 목록",
  "scratch": "새 작업",
  "publication": "게시",
  "published": "게시됨",
  "revision": "리비전",
  "lesson": "교과",
  "lesson_id": "교과 식별자",
  "lesson_title": "교과 제목",
  "lesson_desc": "교과 설명",
});

const TOKEN_LABELS = Object.freeze({
  id: "식별자",
  name: "이름",
  title: "제목",
  desc: "설명",
  description: "설명",
  source: "출처",
  schema: "꼴",
  status: "상태",
  expected: "기대값",
  actual: "실제값",
  available: "사용 가능",
  strict: "엄격",
  unknown: "알 수 없음",
  off: "꺼짐",
  input: "입력",
  output: "출력",
  row: "행",
  rows: "행",
  column: "열",
  columns: "열",
  table: "표",
  graph: "그래프",
  structure: "구조",
  space: "공간",
  space2d: "보개",
  text: "글",
  overlay: "겹보기",
  resource: "자원",
  resources: "자원",
  fixed64: "수",
  value: "값",
  json: "JSON",
  handle: "핸들",
  component: "구성요소",
  host: "호스트",
  ext: "확장",
  exts: "확장",
  drawlist: "그림목록",
  draw: "그림",
  view: "보기",
  views: "보기",
  family: "보기 종류",
  families: "보기 종류",
  contract: "계약",
  bridge: "연결",
  check: "확인",
  meta: "메타",
  required: "필수",
  revision: "리비전",
  publication: "게시",
  lesson: "교과",
  state: "상태",
  hash: "해시",
  tick: "마디",
  time: "시간",
  speed: "속도",
});

function hasKorean(text) {
  return /[가-힣]/u.test(text);
}

function splitAsciiLabel(text) {
  return text
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .split(/[\s._:/-]+/u)
    .map((token) => token.trim())
    .filter(Boolean);
}

export function formatDisplayLabel(raw, { fallback = "" } = {}) {
  const text = String(raw ?? "").trim();
  if (!text) return String(fallback ?? "");
  const lower = text.toLowerCase();
  if (Object.prototype.hasOwnProperty.call(DISPLAY_LABELS, text)) return DISPLAY_LABELS[text];
  if (Object.prototype.hasOwnProperty.call(DISPLAY_LABELS, lower)) return DISPLAY_LABELS[lower];
  if (hasKorean(text)) return text;

  const tokens = splitAsciiLabel(text);
  if (!tokens.length) return text;
  const translated = tokens.map((token) => TOKEN_LABELS[token.toLowerCase()] ?? "");
  if (translated.every(Boolean)) {
    return Array.from(new Set(translated)).join(" ");
  }
  return text;
}

export function formatDisplayPairKey(raw) {
  return formatDisplayLabel(raw);
}

export function formatSourceLabel(raw) {
  return formatDisplayLabel(raw, { fallback: "알 수 없음" });
}
