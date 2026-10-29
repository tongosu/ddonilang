import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, "..");

const contractModulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/play_diagnostic_contract.js");
const contractModuleUrl = pathToFileURL(contractModulePath).href;
const { mapPlayDiagnosticToUserMessage, normalizeDiagnosticItem } = await import(contractModuleUrl);

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const mappingCases = [
  {
    code: "E_BLOCK_HEADER_COLON_FORBIDDEN",
    technical: "block header has colon",
    expected: "블록 헤더에는 ':'를 쓰지 않습니다. 예: 채비 {}",
  },
  {
    code: "E_BLOCK_HEADER_HASH_FORBIDDEN",
    technical: "hash header not allowed",
    expected: "active 실행선에서는 # 헤더를 사용할 수 없습니다. 설정 {}와 매김 {}를 사용하세요.",
  },
  {
    code: "E_IMPORT_ALIAS_DUPLICATE",
    technical: "duplicate alias",
    expected: "import 별칭이 중복되었습니다. 별칭 이름을 고유하게 바꿔 주세요.",
  },
  {
    code: "E_RECEIVE_OUTSIDE_IMJA",
    technical: "receive outside imja",
    expected: "받기(수신) 구문은 임자 블록 안에서만 사용할 수 있습니다.",
  },
  {
    code: "E_PARSE_UNEXPECTED_TOKEN",
    technical: "unexpected token",
    expected: "예상하지 못한 기호가 있습니다. 문장/기호 배치를 확인해 주세요.",
  },
];

for (const row of mappingCases) {
  const actual = mapPlayDiagnosticToUserMessage(row.code, row.technical);
  assert(actual === row.expected, `playground mapping mismatch: code=${row.code} actual=${actual}`);
}

const normalizedWithUserMessage = normalizeDiagnosticItem({
  technical_code: "E_API_RUN_DDN_TEXT_REQUIRED",
  technical_message: "ddn_text required",
  user_message: "실행할 DDN 본문이 비어 있습니다. 입력을 확인해 주세요.",
  severity: "error",
});
assert(normalizedWithUserMessage?.code === "E_API_RUN_DDN_TEXT_REQUIRED", "normalize: technical_code priority");
assert(
  normalizedWithUserMessage?.message === "실행할 DDN 본문이 비어 있습니다. 입력을 확인해 주세요.",
  "normalize: user_message priority",
);
assert(
  normalizedWithUserMessage?.technical_message === "ddn_text required",
  "normalize: technical_message kept",
);

const normalizedFallback = normalizeDiagnosticItem({
  code: "E_PARSE_UNEXPECTED_TOKEN",
  message: "unexpected token",
});
assert(
  normalizedFallback?.message === "예상하지 못한 기호가 있습니다. 문장/기호 배치를 확인해 주세요.",
  "normalize: fallback mapping from technical message",
);

const normalizedUnknown = normalizeDiagnosticItem({
  technical_code: "E_UNKNOWN_TEST",
  technical_message: "opaque failure",
});
assert(normalizedUnknown?.message === "opaque failure", "normalize: unknown code keeps technical message as user text");

console.log("seamgrim playground diagnostic contract runner ok");
