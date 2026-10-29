import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, "..");

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function assertEq(actual, expected, message) {
  if (actual !== expected) {
    throw new Error(`${message}: expected=${JSON.stringify(expected)} actual=${JSON.stringify(actual)}`);
  }
}

function expectThrow(fn, message, includes = "") {
  let caught = null;
  try {
    fn();
  } catch (error) {
    caught = error;
  }
  if (!caught) {
    throw new Error(`${message}: expected throw`);
  }
  if (includes && !String(caught?.message ?? "").includes(includes)) {
    throw new Error(`${message}: unexpected error message=${String(caught?.message ?? "")}`);
  }
}

const contractPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/platform_server_adapter_contract.js");
const contractUrl = pathToFileURL(contractPath).href;

const {
  PLATFORM_SERVER_ADAPTER_REQUEST_SCHEMA,
  PLATFORM_SERVER_ADAPTER_RESPONSE_SCHEMA,
  PlatformServerAdapterStatus,
  PlatformServerAdapterOp,
  PlatformServerAdapterErrorCode,
  PlatformServerAdapterUiAction,
  mapServerOpToNotReadyErrorCode,
  resolveServerErrorActionRail,
  buildServerAdapterRequest,
  buildServerAdapterErrorResponse,
  stableStringifyServerAdapterEnvelope,
  parseServerAdapterEnvelope,
  roundtripServerAdapterEnvelope,
} = await import(contractUrl);

assert(typeof buildServerAdapterRequest === "function", "export: buildServerAdapterRequest");
assert(typeof buildServerAdapterErrorResponse === "function", "export: buildServerAdapterErrorResponse");
assert(typeof mapServerOpToNotReadyErrorCode === "function", "export: mapServerOpToNotReadyErrorCode");
assert(typeof resolveServerErrorActionRail === "function", "export: resolveServerErrorActionRail");
assert(typeof stableStringifyServerAdapterEnvelope === "function", "export: stableStringifyServerAdapterEnvelope");
assert(typeof parseServerAdapterEnvelope === "function", "export: parseServerAdapterEnvelope");
assert(typeof roundtripServerAdapterEnvelope === "function", "export: roundtripServerAdapterEnvelope");

const request = buildServerAdapterRequest({
  op: PlatformServerAdapterOp.SAVE,
  requestId: " req-1 ",
  authSessionId: " auth-1 ",
  userId: " user-1 ",
  context: { work_id: "work-1" },
  payload: { ddn_text: "x <- 1." },
});
assertEq(request.schema, PLATFORM_SERVER_ADAPTER_REQUEST_SCHEMA, "request schema");
assertEq(request.op, PlatformServerAdapterOp.SAVE, "request op");
assertEq(request.request_id, "req-1", "request id normalize");
assertEq(request.actor_session.auth_session_id, "auth-1", "auth session normalize");
assertEq(request.actor_session.user_id, "user-1", "user id normalize");
assertEq(request.context.work_id, "work-1", "context passthrough");
assertEq(request.payload.ddn_text, "x <- 1.", "payload passthrough");

const response = buildServerAdapterErrorResponse({
  request,
  code: mapServerOpToNotReadyErrorCode(PlatformServerAdapterOp.SAVE),
  message: "서버 저장은 준비 중입니다.",
  retryable: false,
});
assertEq(response.schema, PLATFORM_SERVER_ADAPTER_RESPONSE_SCHEMA, "response schema");
assertEq(response.status, PlatformServerAdapterStatus.ERROR, "response status");
assertEq(response.op, PlatformServerAdapterOp.SAVE, "response op");
assertEq(response.request_id, "req-1", "response request id");
assertEq(response.error.code, PlatformServerAdapterErrorCode.SAVE_NOT_READY, "response error code");

assertEq(
  mapServerOpToNotReadyErrorCode(PlatformServerAdapterOp.RESTORE_REVISION),
  PlatformServerAdapterErrorCode.RESTORE_NOT_READY,
  "map restore code",
);
assertEq(
  mapServerOpToNotReadyErrorCode(PlatformServerAdapterOp.SHARE),
  PlatformServerAdapterErrorCode.SHARE_NOT_READY,
  "map share code",
);
assertEq(
  mapServerOpToNotReadyErrorCode(PlatformServerAdapterOp.PUBLISH),
  PlatformServerAdapterErrorCode.PUBLISH_NOT_READY,
  "map publish code",
);
assertEq(
  mapServerOpToNotReadyErrorCode(PlatformServerAdapterOp.INSTALL_PACKAGE),
  PlatformServerAdapterErrorCode.INSTALL_NOT_READY,
  "map install code",
);
assertEq(
  mapServerOpToNotReadyErrorCode(PlatformServerAdapterOp.SWITCH_CATALOG),
  PlatformServerAdapterErrorCode.CATALOG_NOT_READY,
  "map switch catalog code",
);
assertEq(
  mapServerOpToNotReadyErrorCode("unsupported"),
  PlatformServerAdapterErrorCode.INVALID_REQUEST,
  "map unsupported code",
);
assertEq(
  mapServerOpToNotReadyErrorCode(""),
  PlatformServerAdapterErrorCode.INVALID_REQUEST,
  "map empty code",
);

assertEq(
  resolveServerErrorActionRail(PlatformServerAdapterErrorCode.AUTH_REQUIRED)[0],
  PlatformServerAdapterUiAction.LOGIN,
  "auth action rail first",
);
assertEq(
  resolveServerErrorActionRail(PlatformServerAdapterErrorCode.PERMISSION_DENIED)[0],
  PlatformServerAdapterUiAction.REQUEST_ACCESS,
  "permission action rail first",
);
assertEq(
  resolveServerErrorActionRail(PlatformServerAdapterErrorCode.VALIDATION_FAILED)[0],
  PlatformServerAdapterUiAction.FIX_INPUT,
  "validation action rail first",
);
assertEq(
  resolveServerErrorActionRail(PlatformServerAdapterErrorCode.SAVE_NOT_READY)[0],
  PlatformServerAdapterUiAction.RETRY_LATER,
  "not-ready action rail first",
);

const stableRequest = stableStringifyServerAdapterEnvelope(request);
const stableResponse = stableStringifyServerAdapterEnvelope(response);
const parsedRequest = parseServerAdapterEnvelope(stableRequest);
const parsedResponse = parseServerAdapterEnvelope(stableResponse);
assertEq(parsedRequest.schema, PLATFORM_SERVER_ADAPTER_REQUEST_SCHEMA, "parse request schema");
assertEq(parsedResponse.schema, PLATFORM_SERVER_ADAPTER_RESPONSE_SCHEMA, "parse response schema");
assertEq(
  stableStringifyServerAdapterEnvelope(roundtripServerAdapterEnvelope(request)),
  stableRequest,
  "roundtrip request",
);
assertEq(
  stableStringifyServerAdapterEnvelope(roundtripServerAdapterEnvelope(response)),
  stableResponse,
  "roundtrip response",
);

expectThrow(() => parseServerAdapterEnvelope("{}"), "schema mismatch should throw", "server_adapter_schema_mismatch");
expectThrow(
  () =>
    parseServerAdapterEnvelope(
      JSON.stringify({
        schema: PLATFORM_SERVER_ADAPTER_REQUEST_SCHEMA,
        op: "",
      }),
    ),
  "missing op should throw",
  "server_adapter_op_missing",
);
expectThrow(() => parseServerAdapterEnvelope("not-json"), "invalid json should throw");

console.log("seamgrim platform server adapter contract runner ok");
