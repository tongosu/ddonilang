import {
  PlatformServerAdapterErrorCode,
} from "./platform_server_adapter_contract.js";

export const PROJECT_API_BASE = "/api/v1/projects";
export const PROJECT_CAPABILITY_SESSION_KEY = "seamgrim.project.capability.v0";

function asText(value, fallback = "") {
  const text = String(value ?? "").trim();
  return text || fallback;
}

function safeJson(value) {
  try {
    return JSON.stringify(value ?? {});
  } catch (_) {
    return "{}";
  }
}

export function buildProjectCapabilityFragment({ projectId = "", role = "view", capability = "" } = {}) {
  const params = new URLSearchParams();
  params.set("sg_project", asText(projectId));
  params.set("sg_role", asText(role, "view"));
  params.set("sg_cap", asText(capability));
  return `#${params.toString()}`;
}

export function parseProjectCapabilityFragment(hash = "") {
  const source = String(hash ?? "").replace(/^#/, "");
  const params = new URLSearchParams(source);
  const projectId = asText(params.get("sg_project"));
  const role = asText(params.get("sg_role"), "view");
  const capability = asText(params.get("sg_cap"));
  if (!projectId || !capability) return null;
  return {
    project_id: projectId,
    role: role === "edit" ? "edit" : "view",
    capability,
  };
}

export function captureProjectCapabilityFromLocation({
  location = globalThis.location,
  sessionStorage = globalThis.sessionStorage,
  history = globalThis.history,
} = {}) {
  const captured = parseProjectCapabilityFragment(location?.hash ?? "");
  if (!captured) return null;
  try {
    sessionStorage?.setItem?.(PROJECT_CAPABILITY_SESSION_KEY, safeJson(captured));
  } catch (_) {
    // Restricted contexts can still use the returned capability for this page.
  }
  try {
    const cleanUrl = `${location.pathname || "/"}${location.search || ""}`;
    history?.replaceState?.(null, "", cleanUrl);
  } catch (_) {
    // Hash scrub is best-effort in non-browser tests.
  }
  return captured;
}

export function readProjectCapabilitySession({ sessionStorage = globalThis.sessionStorage } = {}) {
  try {
    const raw = sessionStorage?.getItem?.(PROJECT_CAPABILITY_SESSION_KEY) ?? "";
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === "object" && parsed.project_id && parsed.capability) {
      return {
        project_id: asText(parsed.project_id),
        role: asText(parsed.role, "view") === "edit" ? "edit" : "view",
        capability: asText(parsed.capability),
      };
    }
  } catch (_) {
    // Ignore corrupt local session capability.
  }
  return null;
}

export function mapProjectApiStatusToAdapterCode(status, code = "") {
  const numeric = Number(status) || 0;
  const normalizedCode = asText(code);
  if (numeric === 401) return PlatformServerAdapterErrorCode.AUTH_REQUIRED;
  if (numeric === 403) return PlatformServerAdapterErrorCode.PERMISSION_DENIED;
  if (numeric === 409) return PlatformServerAdapterErrorCode.VALIDATION_FAILED;
  if (numeric === 400 || numeric === 413) return PlatformServerAdapterErrorCode.VALIDATION_FAILED;
  if (numeric === 503 || normalizedCode === "E_PROJECT_SERVICE_UNAVAILABLE") {
    return PlatformServerAdapterErrorCode.SAVE_NOT_READY;
  }
  return PlatformServerAdapterErrorCode.INVALID_REQUEST;
}

export async function requestProjectApi(path, {
  method = "GET",
  capability = "",
  body = null,
  fetchImpl = globalThis.fetch,
} = {}) {
  const normalizedPath = String(path ?? "").startsWith("/")
    ? String(path)
    : `${PROJECT_API_BASE}/${String(path ?? "").replace(/^\/+/, "")}`;
  const headers = { "Content-Type": "application/json" };
  const rawCapability = asText(capability);
  if (rawCapability) headers.Authorization = `Bearer ${rawCapability}`;
  const response = await fetchImpl(normalizedPath, {
    method,
    headers,
    body: body === null || body === undefined ? undefined : JSON.stringify(body),
  });
  let payload = null;
  try {
    payload = await response.json();
  } catch (_) {
    payload = {};
  }
  if (!response.ok) {
    const err = new Error(asText(payload?.user_message, "프로젝트 요청 처리에 실패했습니다."));
    err.status = response.status;
    err.payload = payload;
    err.adapterCode = mapProjectApiStatusToAdapterCode(response.status, payload?.code);
    throw err;
  }
  return payload;
}
