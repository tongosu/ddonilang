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

const adapterContractPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/platform_mock_adapter_contract.js");
const adapterContractUrl = pathToFileURL(adapterContractPath).href;
const platformContractPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/platform_contract.js");
const platformContractUrl = pathToFileURL(platformContractPath).href;

const {
  CatalogKind,
  ObjectKind,
  PackageScope,
  ShareKind,
  Visibility,
} = await import(platformContractUrl);

const {
  PLATFORM_MOCK_ADAPTER_SCHEMA,
  PlatformMockAdapterOp,
  buildMockSaveAdapterPayload,
  buildMockRestoreRevisionPayload,
  buildMockShareAdapterPayload,
  buildMockPublishAdapterPayload,
  buildMockInstallPackagePayload,
  buildMockSwitchCatalogPayload,
  stableStringifyMockAdapterPayload,
  parseMockAdapterPayload,
  roundtripMockAdapterPayload,
} = await import(adapterContractUrl);

assert(typeof buildMockSaveAdapterPayload === "function", "export: buildMockSaveAdapterPayload");
assert(typeof buildMockRestoreRevisionPayload === "function", "export: buildMockRestoreRevisionPayload");
assert(typeof buildMockShareAdapterPayload === "function", "export: buildMockShareAdapterPayload");
assert(typeof buildMockPublishAdapterPayload === "function", "export: buildMockPublishAdapterPayload");
assert(typeof buildMockInstallPackagePayload === "function", "export: buildMockInstallPackagePayload");
assert(typeof buildMockSwitchCatalogPayload === "function", "export: buildMockSwitchCatalogPayload");
assert(typeof stableStringifyMockAdapterPayload === "function", "export: stableStringifyMockAdapterPayload");
assert(typeof parseMockAdapterPayload === "function", "export: parseMockAdapterPayload");
assert(typeof roundtripMockAdapterPayload === "function", "export: roundtripMockAdapterPayload");

const savePayload = buildMockSaveAdapterPayload({
  target: "unsupported",
  ddnText: "space2d { point(0,0). }\n",
  workId: " work-1 ",
  projectId: "",
  revisionId: null,
  publicationId: " pub-1 ",
});
assertEq(savePayload.schema, PLATFORM_MOCK_ADAPTER_SCHEMA, "save schema");
assertEq(savePayload.op, PlatformMockAdapterOp.SAVE, "save op");
assertEq(savePayload.target, "local", "save target normalize");
assertEq(savePayload.context.work_id, "work-1", "save work id normalize");
assertEq(savePayload.context.project_id, null, "save project id normalize");
assertEq(savePayload.context.revision_id, null, "save revision id normalize");
assertEq(savePayload.context.publication_id, "pub-1", "save publication id normalize");
assertEq(savePayload.content.text_length, "space2d { point(0,0). }\n".length, "save text length");

const restorePayload = buildMockRestoreRevisionPayload({});
assertEq(restorePayload.schema, PLATFORM_MOCK_ADAPTER_SCHEMA, "restore schema");
assertEq(restorePayload.op, PlatformMockAdapterOp.RESTORE_REVISION, "restore op");
assertEq(restorePayload.source_revision_id, "", "restore source id default");
assertEq(restorePayload.policy.restore_mode, "new_revision", "restore mode policy");
assertEq(restorePayload.policy.source_revision_required, true, "restore source required");

const sharePayload = buildMockShareAdapterPayload({
  kind: "unknown",
  objectKind: "wrong-kind",
  objectId: " lesson-42 ",
  visibility: "invalid",
  sourceRevisionId: "  ",
});
assertEq(sharePayload.op, PlatformMockAdapterOp.SHARE, "share op");
assertEq(sharePayload.kind, ShareKind.LINK, "share kind normalize");
assertEq(sharePayload.object.kind, ObjectKind.LESSON, "share object kind normalize");
assertEq(sharePayload.object.id, "lesson-42", "share object id normalize");
assertEq(sharePayload.policy.visibility, Visibility.PRIVATE, "share visibility normalize");
assertEq(sharePayload.policy.source_revision_id, null, "share source revision normalize");

const publishPayload = buildMockPublishAdapterPayload({
  projectId: " proj-1 ",
  sourceRevisionId: " rev-1 ",
  visibility: "wrong",
});
assertEq(publishPayload.op, PlatformMockAdapterOp.PUBLISH, "publish op");
assertEq(publishPayload.context.project_id, "proj-1", "publish project normalize");
assertEq(publishPayload.context.source_revision_id, "rev-1", "publish source revision normalize");
assertEq(publishPayload.policy.visibility, Visibility.PRIVATE, "publish visibility normalize");
assertEq(publishPayload.policy.republish_mode, "new_artifact", "publish mode");
assertEq(publishPayload.policy.source_revision_required, true, "publish source required");
assertEq(publishPayload.policy.artifact_tracks_draft, false, "publish artifact draft policy");

const installPayload = buildMockInstallPackagePayload({
  packageId: " math/basic ",
  version: "",
  scope: "bad-scope",
  catalogKind: "unknown-catalog",
});
assertEq(installPayload.op, PlatformMockAdapterOp.INSTALL_PACKAGE, "install op");
assertEq(installPayload.package.id, "math/basic", "install package id normalize");
assertEq(installPayload.package.version, "latest", "install version fallback");
assertEq(installPayload.package.scope, PackageScope.PRIVATE, "install scope normalize");
assertEq(installPayload.context.catalog_kind, CatalogKind.LESSON, "install catalog normalize");

const switchPayload = buildMockSwitchCatalogPayload({ catalogKind: "broken-catalog" });
assertEq(switchPayload.op, PlatformMockAdapterOp.SWITCH_CATALOG, "switch op");
assertEq(switchPayload.catalog_kind, CatalogKind.LESSON, "switch catalog normalize");

const payloadCases = [
  savePayload,
  restorePayload,
  sharePayload,
  publishPayload,
  installPayload,
  switchPayload,
];

for (const payload of payloadCases) {
  const stable = stableStringifyMockAdapterPayload(payload);
  const parsed = parseMockAdapterPayload(stable);
  const roundtrip = roundtripMockAdapterPayload(payload);
  assertEq(stableStringifyMockAdapterPayload(parsed), stable, `stable parse roundtrip: ${payload.op}`);
  assertEq(stableStringifyMockAdapterPayload(roundtrip), stable, `roundtrip helper: ${payload.op}`);
}

expectThrow(() => parseMockAdapterPayload("{}"), "parse schema mismatch", "mock_adapter_schema_mismatch");
expectThrow(
  () => parseMockAdapterPayload(JSON.stringify({ schema: PLATFORM_MOCK_ADAPTER_SCHEMA })),
  "parse missing op",
  "mock_adapter_op_missing",
);
expectThrow(() => parseMockAdapterPayload("not-json"), "parse invalid json");

console.log("seamgrim platform mock adapter roundtrip runner ok");
