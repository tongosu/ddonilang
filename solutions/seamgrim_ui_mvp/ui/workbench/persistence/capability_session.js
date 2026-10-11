import {
  buildProjectCapabilityFragment,
  captureProjectCapabilityFromLocation,
  PROJECT_CAPABILITY_SESSION_KEY,
  parseProjectCapabilityFragment,
  readProjectCapabilitySession,
} from "../../project_service_client.js";

export const WORKBENCH_CAPABILITY_SCOPE = "seamgrim.workbench.capability_session.v1";

export function captureWorkbenchCapabilitySession(options = {}) {
  const captured = captureProjectCapabilityFromLocation(options);
  if (!captured) return null;
  return {
    schema: WORKBENCH_CAPABILITY_SCOPE,
    source: "url_fragment",
    project_id: captured.project_id,
    role: captured.role,
    capability: captured.capability,
  };
}

export function readWorkbenchCapabilitySession(options = {}) {
  const session = readProjectCapabilitySession(options);
  if (!session) return null;
  return {
    schema: WORKBENCH_CAPABILITY_SCOPE,
    source: "session_storage",
    project_id: session.project_id,
    role: session.role,
    capability: session.capability,
  };
}

export function writeWorkbenchCapabilitySession({
  projectId = "",
  role = "edit",
  capability = "",
  sessionStorage = globalThis.sessionStorage,
} = {}) {
  const row = {
    project_id: String(projectId || ""),
    role: String(role || "view") === "edit" ? "edit" : "view",
    capability: String(capability || ""),
  };
  if (!row.project_id || !row.capability) return null;
  try {
    sessionStorage?.setItem?.(PROJECT_CAPABILITY_SESSION_KEY, JSON.stringify(row));
  } catch (_) {
    // Restricted contexts can still keep the returned in-memory session.
  }
  return {
    schema: WORKBENCH_CAPABILITY_SCOPE,
    source: "session_storage",
    project_id: row.project_id,
    role: row.role,
    capability: row.capability,
  };
}

export function clearWorkbenchCapabilitySession({
  sessionStorage = globalThis.sessionStorage,
} = {}) {
  try {
    sessionStorage?.removeItem?.(PROJECT_CAPABILITY_SESSION_KEY);
  } catch (_) {
    // Restricted contexts can still clear the in-memory state at the caller.
  }
}

export function buildWorkbenchCapabilityLink({ baseUrl = "", projectId = "", role = "view", capability = "" } = {}) {
  const fragment = buildProjectCapabilityFragment({ projectId, role, capability });
  return `${String(baseUrl || "").split("#")[0]}${fragment}`;
}

export function parseWorkbenchCapabilityFragment(hash = "") {
  const parsed = parseProjectCapabilityFragment(hash);
  if (!parsed) return null;
  return {
    schema: WORKBENCH_CAPABILITY_SCOPE,
    source: "url_fragment",
    project_id: parsed.project_id,
    role: parsed.role,
    capability: parsed.capability,
  };
}
