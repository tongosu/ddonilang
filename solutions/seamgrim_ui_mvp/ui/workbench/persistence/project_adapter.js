import {
  buildProjectSnapshotArtifactV0,
} from "../../project_snapshot.js";
import {
  PROJECT_API_BASE,
  requestProjectApi,
} from "../../project_service_client.js";

export const WORKBENCH_PROJECT_ADAPTER_SCHEMA = "seamgrim.workbench.project_adapter.v1";
export const WORKBENCH_SOURCE_PROFILE = "v1-core-v25";

// Stored document identity, never inferred from DDN tokens or lesson names.
export function workbenchSourceProfile(project = {}) {
  const snapshot = project.portableSnapshot ?? {};
  const values = [project.sourceProfileIdentity, project.portableMetadata?.source_profile_identity,
    snapshot.metadata?.source_profile_identity].filter((value) => value !== undefined && value !== null && value !== "");
  if (!values.length) return String(project.resolvedRootSourceProfileIdentity ?? "");
  const identity = String(values[0]);
  if (values.some((value) => String(value) !== identity)) return "";
  if (project.resolvedRootSourceProfileIdentity && project.resolvedRootSourceProfileIdentity !== identity) return "";
  return identity;
}

export function selectUnversionedDocumentProfile(project, identity) {
  if (identity !== WORKBENCH_SOURCE_PROFILE) throw new Error("지원하지 않는 문법 판본입니다.");
  if (workbenchSourceProfile(project)) throw new Error("이미 판본이 정해진 문서입니다.");
  const declared = [project.sourceProfileIdentity, project.portableMetadata?.source_profile_identity,
    project.portableSnapshot?.metadata?.source_profile_identity].some((value) => Boolean(value));
  if (declared) throw new Error("기존 판본을 덮어쓸 수 없습니다.");
  project.sourceProfileIdentity = identity;
  project.selectedImportSourceProfileIdentity = identity;
  return project;
}

function asText(value, fallback = "") {
  const text = String(value ?? "").trim();
  return text || fallback;
}

function mainDdnText(files = []) {
  const rows = Array.isArray(files) ? files : [];
  const main = rows.find((file) => file.path === "lesson.ddn") ?? rows[0] ?? {};
  return String(main.text ?? "");
}

export async function buildWorkbenchProjectArtifact({
  projectId = "custom",
  projectName = "",
  files = [],
  controlsSnapshot = null,
  localPackagePayload = null,
  lineage = null,
  metadata = null,
  sourceProfileIdentity = "",
} = {}) {
  if (sourceProfileIdentity && metadata?.source_profile_identity
    && sourceProfileIdentity !== metadata.source_profile_identity) {
    throw new Error("문서의 문법 판본 기록이 서로 다릅니다.");
  }
  return buildProjectSnapshotArtifactV0({
    projectName: asText(projectName, "이름 없는 셈그림"),
    ddnText: mainDdnText(files),
    lessonPlanText: "",
    sourceLessonId: asText(projectId, "custom"),
    controlsSnapshot,
    localPackagePayload,
    lineage,
    metadata: {
      schema: WORKBENCH_PROJECT_ADAPTER_SCHEMA,
      one_file_project: files.length <= 1,
      generated_artifacts_hidden_from_user_files: true,
      ...metadata,
      source_profile_identity: sourceProfileIdentity || metadata?.source_profile_identity || "",
    },
  });
}

export async function createServerProject({
  artifact,
  title = "",
  fetchImpl = globalThis.fetch,
} = {}) {
  return requestProjectApi(PROJECT_API_BASE, {
    method: "POST",
    body: { project_snapshot_artifact: artifact, title },
    fetchImpl,
  });
}

export async function openServerProject({
  projectId = "",
  capability = "",
  fetchImpl = globalThis.fetch,
} = {}) {
  return requestProjectApi(projectId, { capability, fetchImpl });
}

export async function appendServerRevision({
  projectId = "",
  capability = "",
  artifact,
  expectedParentRevisionId = "",
  fetchImpl = globalThis.fetch,
} = {}) {
  return requestProjectApi(`${projectId}/revisions`, {
    method: "POST",
    capability,
    body: {
      project_snapshot_artifact: artifact,
      expected_parent_revision_id: expectedParentRevisionId,
    },
    fetchImpl,
  });
}

export async function listServerRevisions({
  projectId = "",
  capability = "",
  fetchImpl = globalThis.fetch,
} = {}) {
  return requestProjectApi(`${projectId}/revisions`, { capability, fetchImpl });
}

export async function restoreServerRevision({
  projectId = "",
  capability = "",
  sourceRevisionId = "",
  expectedParentRevisionId = "",
  fetchImpl = globalThis.fetch,
} = {}) {
  return requestProjectApi(`${projectId}/restore`, {
    method: "POST",
    capability,
    body: {
      source_revision_id: sourceRevisionId,
      expected_parent_revision_id: expectedParentRevisionId,
    },
    fetchImpl,
  });
}

export async function cloneServerProject({
  projectId = "",
  capability = "",
  title = "",
  fetchImpl = globalThis.fetch,
} = {}) {
  return requestProjectApi(`${projectId}/clone`, {
    method: "POST",
    capability,
    body: { title },
    fetchImpl,
  });
}
