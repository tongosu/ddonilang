export const PLATFORM_SERVER_ADAPTER_REQUEST_SCHEMA = "seamgrim.platform.server_adapter.request.v1";
export const PLATFORM_SERVER_ADAPTER_RESPONSE_SCHEMA = "seamgrim.platform.server_adapter.response.v1";

export const PlatformServerAdapterStatus = Object.freeze({
  OK: "ok",
  ERROR: "error",
});

export const PlatformServerAdapterOp = Object.freeze({
  SAVE: "save",
  RESTORE_REVISION: "restore_revision",
  SHARE: "share",
  PUBLISH: "publish",
  INSTALL_PACKAGE: "install_package",
  SWITCH_CATALOG: "switch_catalog",
});

export const PlatformServerAdapterErrorCode = Object.freeze({
  INVALID_REQUEST: "E_PLATFORM_INVALID_REQUEST",
  AUTH_REQUIRED: "E_PLATFORM_AUTH_REQUIRED",
  PERMISSION_DENIED: "E_PLATFORM_PERMISSION_DENIED",
  VALIDATION_FAILED: "E_PLATFORM_VALIDATION_FAILED",
  SAVE_NOT_READY: "E_PLATFORM_SAVE_NOT_READY",
  RESTORE_NOT_READY: "E_PLATFORM_RESTORE_NOT_READY",
  SHARE_NOT_READY: "E_PLATFORM_SHARE_NOT_READY",
  PUBLISH_NOT_READY: "E_PLATFORM_PUBLISH_NOT_READY",
  INSTALL_NOT_READY: "E_PLATFORM_INSTALL_NOT_READY",
  CATALOG_NOT_READY: "E_PLATFORM_CATALOG_NOT_READY",
});

export const PlatformServerAdapterUiAction = Object.freeze({
  LOGIN: "login",
  REQUEST_ACCESS: "request_access",
  FIX_INPUT: "fix_input",
  RETRY_LATER: "retry_later",
  OPEN_LOCAL_SAVE: "open_local_save",
});

const OP_NOT_READY_ERROR_CODE = Object.freeze({
  [PlatformServerAdapterOp.SAVE]: PlatformServerAdapterErrorCode.SAVE_NOT_READY,
  [PlatformServerAdapterOp.RESTORE_REVISION]: PlatformServerAdapterErrorCode.RESTORE_NOT_READY,
  [PlatformServerAdapterOp.SHARE]: PlatformServerAdapterErrorCode.SHARE_NOT_READY,
  [PlatformServerAdapterOp.PUBLISH]: PlatformServerAdapterErrorCode.PUBLISH_NOT_READY,
  [PlatformServerAdapterOp.INSTALL_PACKAGE]: PlatformServerAdapterErrorCode.INSTALL_NOT_READY,
  [PlatformServerAdapterOp.SWITCH_CATALOG]: PlatformServerAdapterErrorCode.CATALOG_NOT_READY,
});

function normalizeText(value, fallback = "") {
  const text = String(value ?? "").trim();
  return text || fallback;
}

function normalizeOptionalText(value) {
  const text = normalizeText(value, "");
  return text || null;
}

function normalizeOp(op) {
  const value = normalizeText(op, "");
  if (
    value === PlatformServerAdapterOp.SAVE ||
    value === PlatformServerAdapterOp.RESTORE_REVISION ||
    value === PlatformServerAdapterOp.SHARE ||
    value === PlatformServerAdapterOp.PUBLISH ||
    value === PlatformServerAdapterOp.INSTALL_PACKAGE ||
    value === PlatformServerAdapterOp.SWITCH_CATALOG
  ) {
    return value;
  }
  return "";
}

function normalizeErrorCode(code) {
  const value = normalizeText(code, "");
  const known = Object.values(PlatformServerAdapterErrorCode);
  if (known.includes(value)) {
    return value;
  }
  return PlatformServerAdapterErrorCode.INVALID_REQUEST;
}

function sortDeep(value) {
  if (Array.isArray(value)) {
    return value.map((item) => sortDeep(item));
  }
  if (value && typeof value === "object") {
    const out = {};
    const keys = Object.keys(value).sort();
    keys.forEach((key) => {
      out[key] = sortDeep(value[key]);
    });
    return out;
  }
  return value;
}

export function mapServerOpToNotReadyErrorCode(op) {
  const normalized = normalizeOp(op);
  if (!normalized) {
    return PlatformServerAdapterErrorCode.INVALID_REQUEST;
  }
  return OP_NOT_READY_ERROR_CODE[normalized] ?? PlatformServerAdapterErrorCode.INVALID_REQUEST;
}

export function resolveServerErrorActionRail(code) {
  const normalized = normalizeErrorCode(code);
  if (normalized === PlatformServerAdapterErrorCode.AUTH_REQUIRED) {
    return [PlatformServerAdapterUiAction.LOGIN, PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE];
  }
  if (normalized === PlatformServerAdapterErrorCode.PERMISSION_DENIED) {
    return [PlatformServerAdapterUiAction.REQUEST_ACCESS, PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE];
  }
  if (
    normalized === PlatformServerAdapterErrorCode.INVALID_REQUEST ||
    normalized === PlatformServerAdapterErrorCode.VALIDATION_FAILED
  ) {
    return [PlatformServerAdapterUiAction.FIX_INPUT, PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE];
  }
  return [PlatformServerAdapterUiAction.RETRY_LATER, PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE];
}

export function buildServerAdapterRequest({
  op = "",
  requestId = null,
  authSessionId = null,
  userId = null,
  context = null,
  payload = null,
} = {}) {
  const normalizedOp = normalizeOp(op);
  return {
    schema: PLATFORM_SERVER_ADAPTER_REQUEST_SCHEMA,
    op: normalizedOp,
    request_id: normalizeOptionalText(requestId),
    actor_session: {
      auth_session_id: normalizeOptionalText(authSessionId),
      user_id: normalizeOptionalText(userId),
    },
    context: context && typeof context === "object" ? context : {},
    payload: payload && typeof payload === "object" ? payload : {},
  };
}

export function buildServerAdapterErrorResponse({
  request = null,
  code = PlatformServerAdapterErrorCode.INVALID_REQUEST,
  message = "",
  retryable = false,
} = {}) {
  const req = request && typeof request === "object" ? request : {};
  const normalizedOp = normalizeOp(req.op ?? "");
  return {
    schema: PLATFORM_SERVER_ADAPTER_RESPONSE_SCHEMA,
    status: PlatformServerAdapterStatus.ERROR,
    op: normalizedOp,
    request_id: normalizeOptionalText(req.request_id),
    error: {
      code: normalizeErrorCode(code),
      message: normalizeText(message, "platform server adapter error"),
      retryable: Boolean(retryable),
    },
  };
}

export function stableStringifyServerAdapterEnvelope(value) {
  return JSON.stringify(sortDeep(value));
}

export function parseServerAdapterEnvelope(jsonText) {
  const parsed = JSON.parse(String(jsonText ?? ""));
  if (!parsed || typeof parsed !== "object") {
    throw new Error("server_adapter_envelope_invalid");
  }
  const schema = String(parsed.schema ?? "");
  if (
    schema !== PLATFORM_SERVER_ADAPTER_REQUEST_SCHEMA &&
    schema !== PLATFORM_SERVER_ADAPTER_RESPONSE_SCHEMA
  ) {
    throw new Error("server_adapter_schema_mismatch");
  }
  const op = normalizeOp(parsed.op ?? "");
  if (!op) {
    throw new Error("server_adapter_op_missing");
  }
  return parsed;
}

export function roundtripServerAdapterEnvelope(envelope) {
  const canonical = stableStringifyServerAdapterEnvelope(envelope);
  return parseServerAdapterEnvelope(canonical);
}
