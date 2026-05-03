export const ObjectKind = Object.freeze({
  LESSON: "lesson",
  PROJECT: "project",
  PACKAGE: "package",
  ARTIFACT: "artifact",
  REVISION: "revision",
  WORKSPACE: "workspace",
});

export function assertIdKindMismatch(idA, kindA, idB, kindB) {
  const leftId = String(idA ?? "").trim();
  const rightId = String(idB ?? "").trim();
  if (!leftId || !rightId) return;
  const leftKind = String(kindA ?? "").trim();
  const rightKind = String(kindB ?? "").trim();
  if (leftId === rightId && leftKind && rightKind && leftKind !== rightKind) {
    throw new Error(`ID 충돌: "${leftId}"가 ${leftKind}와 ${rightKind}에 동시 사용됨`);
  }
}

export const RevisionPolicy = Object.freeze({
  RESTORE_MODE: "new_revision",
  SOURCE_REVISION_ID_REQUIRED: true,
});

export const ShareKind = Object.freeze({
  LINK: "link",
  CLONE: "clone",
  PACKAGE: "package",
});

export const Role = Object.freeze({
  OWNER: "owner",
  EDITOR: "editor",
  VIEWER: "viewer",
  PUBLISHER: "publisher",
});

export const Visibility = Object.freeze({
  PRIVATE: "private",
  TEAM: "team",
  INTERNAL: "internal",
  PUBLIC: "public",
});

export const PublishPolicy = Object.freeze({
  ARTIFACT_TRACKS_DRAFT: false,
  REPUBLISH_MODE: "new_artifact",
  SOURCE_REVISION_ID_REQUIRED: true,
});

export const SourceManagementPolicy = Object.freeze({
  REVISION_APPEND_ONLY: true,
  RESTORE_CREATES_NEW_REVISION: true,
  OVERWRITE_FORBIDDEN: true,
});

export const RouteSlotPolicy = Object.freeze({
  PLATFORM_ROUTE_KEYS: Object.freeze(["work", "revision", "publication", "project"]),
  PLATFORM_ROUTE_PRECEDENCE: Object.freeze(["work", "revision", "publication", "project"]),
  LEGACY_FALLBACK_KEYS: Object.freeze(["lesson", "ddn"]),
});

export const PublicationPolicy = Object.freeze({
  SNAPSHOT_IMMUTABLE: true,
  PUBLIC_LINK_TARGET_DEFAULT: "artifact",
  PINNED_REVISION_REQUIRED: true,
  REPUBLISH_APPEND_ONLY: true,
});

export const PackageScope = Object.freeze({
  STANDARD: "표준",
  SHARE: "나눔",
  PRIVATE: "내",
  OPEN: "벌림",
});

export const CatalogKind = Object.freeze({
  LESSON: "lesson_catalog",
  PACKAGE: "package_catalog",
  PROJECT: "project_catalog",
});

export const PackageMetaKeys = Object.freeze([
  "name",
  "version",
  "scope",
  "description",
  "dependencies",
  "lock_hash",
  "det_tier",
]);
