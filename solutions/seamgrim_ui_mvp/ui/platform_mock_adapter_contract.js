import {
  CatalogKind,
  ObjectKind,
  PackageScope,
  PublishPolicy,
  RevisionPolicy,
  Role,
  ShareKind,
  Visibility,
} from "./platform_contract.js";

export const PLATFORM_MOCK_ADAPTER_SCHEMA = "seamgrim.platform.mock_adapter.v1";

export const PlatformMockAdapterOp = Object.freeze({
  SAVE: "save",
  RESTORE_REVISION: "restore_revision",
  SHARE: "share",
  PUBLISH: "publish",
  INSTALL_PACKAGE: "install_package",
  SWITCH_CATALOG: "switch_catalog",
});

function normalizeText(value, fallback = "") {
  const text = String(value ?? "").trim();
  return text || fallback;
}

function normalizeOptionalText(value) {
  const text = normalizeText(value, "");
  return text || null;
}

function normalizeCatalogKind(value) {
  const kind = normalizeText(value, CatalogKind.LESSON);
  if (kind === CatalogKind.LESSON || kind === CatalogKind.PROJECT || kind === CatalogKind.PACKAGE) {
    return kind;
  }
  return CatalogKind.LESSON;
}

function normalizePackageScope(value) {
  const scope = normalizeText(value, PackageScope.PRIVATE);
  if (
    scope === PackageScope.STANDARD ||
    scope === PackageScope.SHARE ||
    scope === PackageScope.PRIVATE ||
    scope === PackageScope.OPEN
  ) {
    return scope;
  }
  return PackageScope.PRIVATE;
}

function normalizeShareKind(value) {
  const kind = normalizeText(value, ShareKind.LINK).toLowerCase();
  if (kind === ShareKind.LINK || kind === ShareKind.CLONE || kind === ShareKind.PACKAGE) {
    return kind;
  }
  return ShareKind.LINK;
}

function normalizeVisibility(value) {
  const visibility = normalizeText(value, Visibility.PRIVATE).toLowerCase();
  if (
    visibility === Visibility.PRIVATE ||
    visibility === Visibility.TEAM ||
    visibility === Visibility.INTERNAL ||
    visibility === Visibility.PUBLIC
  ) {
    return visibility;
  }
  return Visibility.PRIVATE;
}

function normalizeRole(value) {
  const role = normalizeText(value, Role.OWNER).toLowerCase();
  if (role === Role.OWNER || role === Role.EDITOR || role === Role.VIEWER || role === Role.PUBLISHER) {
    return role;
  }
  return Role.OWNER;
}

function normalizeObjectKind(value) {
  const kind = normalizeText(value, ObjectKind.LESSON).toLowerCase();
  if (
    kind === ObjectKind.LESSON ||
    kind === ObjectKind.PROJECT ||
    kind === ObjectKind.PACKAGE ||
    kind === ObjectKind.ARTIFACT ||
    kind === ObjectKind.REVISION ||
    kind === ObjectKind.WORKSPACE
  ) {
    return kind;
  }
  return ObjectKind.LESSON;
}

function sortDeep(value) {
  if (Array.isArray(value)) {
    return value.map((item) => sortDeep(item));
  }
  if (value && typeof value === "object") {
    const sorted = {};
    const keys = Object.keys(value).sort();
    keys.forEach((key) => {
      sorted[key] = sortDeep(value[key]);
    });
    return sorted;
  }
  return value;
}

function baseEnvelope(op) {
  return {
    schema: PLATFORM_MOCK_ADAPTER_SCHEMA,
    op,
  };
}

export function buildMockSaveAdapterPayload({
  target = "local",
  ddnText = "",
  workId = null,
  projectId = null,
  revisionId = null,
  publicationId = null,
} = {}) {
  const normalizedTarget = normalizeText(target, "local").toLowerCase();
  const targetSafe = normalizedTarget === "local" || normalizedTarget === "server" || normalizedTarget === "share"
    ? normalizedTarget
    : "local";
  return {
    ...baseEnvelope(PlatformMockAdapterOp.SAVE),
    target: targetSafe,
    context: {
      work_id: normalizeOptionalText(workId),
      project_id: normalizeOptionalText(projectId),
      revision_id: normalizeOptionalText(revisionId),
      publication_id: normalizeOptionalText(publicationId),
    },
    content: {
      ddn_text: String(ddnText ?? ""),
      text_length: String(ddnText ?? "").length,
    },
  };
}

export function buildMockRestoreRevisionPayload({
  sourceRevisionId = "",
  restoreMode = RevisionPolicy.RESTORE_MODE,
} = {}) {
  return {
    ...baseEnvelope(PlatformMockAdapterOp.RESTORE_REVISION),
    source_revision_id: normalizeText(sourceRevisionId, ""),
    policy: {
      restore_mode: normalizeText(restoreMode, RevisionPolicy.RESTORE_MODE),
      source_revision_required: Boolean(RevisionPolicy.SOURCE_REVISION_ID_REQUIRED),
    },
  };
}

export function buildMockShareAdapterPayload({
  kind = ShareKind.LINK,
  objectKind = ObjectKind.LESSON,
  objectId = "",
  visibility = Visibility.PRIVATE,
  role = Role.OWNER,
  sourceRevisionId = null,
} = {}) {
  return {
    ...baseEnvelope(PlatformMockAdapterOp.SHARE),
    kind: normalizeShareKind(kind),
    object: {
      kind: normalizeObjectKind(objectKind),
      id: normalizeText(objectId, ""),
    },
    policy: {
      visibility: normalizeVisibility(visibility),
      role: normalizeRole(role),
      source_revision_id: normalizeOptionalText(sourceRevisionId),
    },
  };
}

export function buildMockPublishAdapterPayload({
  projectId = null,
  sourceRevisionId = null,
  publicationId = null,
  visibility = Visibility.PRIVATE,
} = {}) {
  return {
    ...baseEnvelope(PlatformMockAdapterOp.PUBLISH),
    context: {
      project_id: normalizeOptionalText(projectId),
      source_revision_id: normalizeOptionalText(sourceRevisionId),
      publication_id: normalizeOptionalText(publicationId),
    },
    policy: {
      visibility: normalizeVisibility(visibility),
      republish_mode: normalizeText(PublishPolicy.REPUBLISH_MODE, "new_artifact"),
      source_revision_required: Boolean(PublishPolicy.SOURCE_REVISION_ID_REQUIRED),
      artifact_tracks_draft: Boolean(PublishPolicy.ARTIFACT_TRACKS_DRAFT),
    },
  };
}

export function buildMockInstallPackagePayload({
  packageId = "",
  version = "latest",
  scope = PackageScope.PRIVATE,
  catalogKind = CatalogKind.LESSON,
} = {}) {
  return {
    ...baseEnvelope(PlatformMockAdapterOp.INSTALL_PACKAGE),
    package: {
      id: normalizeText(packageId, ""),
      version: normalizeText(version, "latest"),
      scope: normalizePackageScope(scope),
    },
    context: {
      catalog_kind: normalizeCatalogKind(catalogKind),
    },
  };
}

export function buildMockSwitchCatalogPayload({
  catalogKind = CatalogKind.LESSON,
} = {}) {
  return {
    ...baseEnvelope(PlatformMockAdapterOp.SWITCH_CATALOG),
    catalog_kind: normalizeCatalogKind(catalogKind),
  };
}

export function stableStringifyMockAdapterPayload(payload) {
  return JSON.stringify(sortDeep(payload));
}

export function parseMockAdapterPayload(jsonText) {
  const parsed = JSON.parse(String(jsonText ?? ""));
  if (!parsed || typeof parsed !== "object") {
    throw new Error("mock_adapter_payload_invalid");
  }
  if (String(parsed.schema ?? "") !== PLATFORM_MOCK_ADAPTER_SCHEMA) {
    throw new Error("mock_adapter_schema_mismatch");
  }
  const op = String(parsed.op ?? "").trim();
  if (!op) {
    throw new Error("mock_adapter_op_missing");
  }
  return parsed;
}

export function roundtripMockAdapterPayload(payload) {
  const canonicalText = stableStringifyMockAdapterPayload(payload);
  return parseMockAdapterPayload(canonicalText);
}
