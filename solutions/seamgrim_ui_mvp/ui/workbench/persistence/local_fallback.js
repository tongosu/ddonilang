export const LOCAL_FALLBACK_SCHEMA = "seamgrim.workbench.local_fallback_snapshot.v1";
export const APPROVED_WORKBENCH_LOCAL_SCHEMA = "seamgrim.approved_workbench.project_snapshot.v1";

function stableCanonicalJson(value) {
  if (Array.isArray(value)) {
    return `[${value.map((item) => stableCanonicalJson(item)).join(",")}]`;
  }
  if (value && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${stableCanonicalJson(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

function stableHash(text) {
  let hash = 0x811c9dc5;
  for (let i = 0; i < String(text ?? "").length; i += 1) {
    hash ^= String(text).charCodeAt(i);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

export function localProjectStorageKey(projectId) {
  return `seamgrim.approved_workbench.project.${String(projectId || "custom")}.v1`;
}

export function buildLocalFallbackSnapshot(payload = {}) {
  const body = {
    schema: LOCAL_FALLBACK_SCHEMA,
    saved_scope: "local_device_only",
    server_saved: false,
    project_id: String(payload.project_id ?? payload.projectId ?? "custom"),
    active_file_path: String(payload.active_file_path ?? payload.activeFilePath ?? "lesson.ddn"),
    mode: String(payload.mode ?? "ddn-main"),
    view_role: String(payload.view_role ?? payload.viewRole ?? "primary"),
    recipe_version: Number(payload.recipe_version ?? payload.recipeVersion ?? 1),
    files: Array.isArray(payload.files) ? payload.files : [],
    runs: Array.isArray(payload.runs) ? payload.runs : [],
  };
  if (payload.view_workspace && typeof payload.view_workspace === "object") {
    body.view_workspace = payload.view_workspace;
  }
  const canonical = stableCanonicalJson(body);
  return {
    ...body,
    canonical_hash: `candidate:fnv1a32:${stableHash(canonical)}`,
  };
}

export function saveLocalFallbackSnapshot(projectId, payload, storage = globalThis.localStorage) {
  const snapshot = buildLocalFallbackSnapshot(payload);
  storage?.setItem?.(localProjectStorageKey(projectId || snapshot.project_id), stableCanonicalJson(snapshot));
  return {
    schema: "seamgrim.workbench.persistence_status.v1",
    mode: "local_fallback",
    server_saved: false,
    local_saved: true,
    user_label: `이 기기에만 임시 저장 ${snapshot.canonical_hash}`,
    snapshot,
  };
}

export function loadLocalFallbackSnapshot(projectId, storage = globalThis.localStorage) {
  const raw = storage?.getItem?.(localProjectStorageKey(projectId)) ?? "";
  if (!raw) return null;
  const parsed = JSON.parse(raw);
  return migrateApprovedLocalSnapshot(parsed);
}

export function migrateApprovedLocalSnapshot(snapshot = {}) {
  if (snapshot?.schema === LOCAL_FALLBACK_SCHEMA) {
    return buildLocalFallbackSnapshot(snapshot);
  }
  if (snapshot?.schema === APPROVED_WORKBENCH_LOCAL_SCHEMA) {
    return buildLocalFallbackSnapshot({
      ...snapshot,
      server_saved: false,
      saved_scope: "local_device_only",
    });
  }
  return buildLocalFallbackSnapshot(snapshot);
}

export function describePersistenceStatus(status = null) {
  if(status?.last_error)return `${status.last_error} · 기존 저장본은 유지합니다.`;
  if (!status) {
    return "저장 전";
  }
  if (status.mode === "server_restore" && status.user_label) {
    return status.user_label;
  }
  if (status.server_saved) {
    return `서버 저장 완료 ${status.revision_id || ""}`.trim();
  }
  if (status.local_saved) {
    return status.user_label || "이 기기에만 임시 저장";
  }
  return "저장 안 됨";
}

export { stableCanonicalJson };
