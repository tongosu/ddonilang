import fs from "node:fs";
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

function assertIncludes(text, expected, message) {
  const source = String(text ?? "");
  if (!source.includes(expected)) {
    throw new Error(`${message}: expected substring=${JSON.stringify(expected)} actual=${JSON.stringify(source)}`);
  }
}

function extractFunctionBody(source, name) {
  const marker = `function ${name}(`;
  const start = source.indexOf(marker);
  if (start < 0) {
    throw new Error(`function_missing:${name}`);
  }
  const parenStart = source.indexOf("(", start);
  if (parenStart < 0) {
    throw new Error(`function_paren_missing:${name}`);
  }
  let parenDepth = 0;
  let parenEnd = -1;
  for (let i = parenStart; i < source.length; i += 1) {
    const ch = source[i];
    if (ch === "(") parenDepth += 1;
    else if (ch === ")") {
      parenDepth -= 1;
      if (parenDepth === 0) {
        parenEnd = i;
        break;
      }
    }
  }
  if (parenEnd < 0) {
    throw new Error(`function_paren_unclosed:${name}`);
  }
  const openBrace = source.indexOf("{", parenEnd);
  if (openBrace < 0) {
    throw new Error(`function_open_brace_missing:${name}`);
  }
  let depth = 0;
  for (let i = openBrace; i < source.length; i += 1) {
    const ch = source[i];
    if (ch === "{") depth += 1;
    else if (ch === "}") {
      depth -= 1;
      if (depth === 0) {
        return source.slice(start, i + 1);
      }
    }
  }
  throw new Error(`function_unclosed:${name}`);
}

function sortDeep(value) {
  if (Array.isArray(value)) return value.map((item) => sortDeep(item));
  if (value && typeof value === "object") {
    const out = {};
    for (const key of Object.keys(value).sort()) {
      out[key] = sortDeep(value[key]);
    }
    return out;
  }
  return value;
}

function stableStringify(value) {
  return JSON.stringify(sortDeep(value));
}

const appJsPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/app.js");
const appJsText = fs.readFileSync(appJsPath, "utf-8");
const platformContractUrl = pathToFileURL(path.resolve(root, "solutions/seamgrim_ui_mvp/ui/platform_contract.js")).href;
const adapterContractUrl = pathToFileURL(
  path.resolve(root, "solutions/seamgrim_ui_mvp/ui/platform_mock_adapter_contract.js"),
).href;

const {
  CatalogKind,
  ObjectKind,
  PublishPolicy,
  PublicationPolicy,
  RevisionPolicy,
  ShareKind,
  SourceManagementPolicy,
  Visibility,
} = await import(platformContractUrl);

const {
  buildMockInstallPackagePayload,
  buildMockPublishAdapterPayload,
  buildMockRestoreRevisionPayload,
  buildMockSaveAdapterPayload,
  buildMockShareAdapterPayload,
  buildMockSwitchCatalogPayload,
} = await import(adapterContractUrl);

const functionSource = [
  extractFunctionBody(appJsText, "emitPlatformMockAdapterPayload"),
  extractFunctionBody(appJsText, "switchCatalog"),
  extractFunctionBody(appJsText, "saveCurrentWork"),
  extractFunctionBody(appJsText, "restoreRevision"),
  extractFunctionBody(appJsText, "shareCurrent"),
  extractFunctionBody(appJsText, "publishCurrent"),
  extractFunctionBody(appJsText, "installPackage"),
].join("\n\n");

const buildHarness = new Function(
  "deps",
  `
const {
  appState,
  window,
  saveDdnToFile,
  showPlatformToast,
  CatalogKind,
  ObjectKind,
  PublishPolicy,
  PublicationPolicy,
  RevisionPolicy,
  ShareKind,
  SourceManagementPolicy,
  Visibility,
  buildMockInstallPackagePayload,
  buildMockPublishAdapterPayload,
  buildMockRestoreRevisionPayload,
  buildMockSaveAdapterPayload,
  buildMockShareAdapterPayload,
  buildMockSwitchCatalogPayload,
} = deps;
${functionSource}
return {
  switchCatalog,
  saveCurrentWork,
  restoreRevision,
  shareCurrent,
  publishCurrent,
  installPackage,
};
`,
);

const toasts = [];
const saves = [];
const windowMock = {};
const appState = {
  currentLesson: { id: "lesson-alpha" },
  shell: {
    currentWorkId: "work-01",
    currentProjectId: "project-01",
    currentRevisionId: "revision-01",
    currentPublicationId: "publication-01",
    activeCatalog: CatalogKind.LESSON,
  },
};

const harness = buildHarness({
  appState,
  window: windowMock,
  saveDdnToFile: (text, filename) => {
    saves.push({ text, filename });
  },
  showPlatformToast: (message) => {
    toasts.push(String(message ?? ""));
  },
  CatalogKind,
  ObjectKind,
  PublishPolicy,
  PublicationPolicy,
  RevisionPolicy,
  ShareKind,
  SourceManagementPolicy,
  Visibility,
  buildMockInstallPackagePayload,
  buildMockPublishAdapterPayload,
  buildMockRestoreRevisionPayload,
  buildMockSaveAdapterPayload,
  buildMockShareAdapterPayload,
  buildMockSwitchCatalogPayload,
});

assert(typeof harness.saveCurrentWork === "function", "harness saveCurrentWork missing");

// switchCatalog (unknown: state unchanged, no toast)
const switchUnknownToastCount = toasts.length;
const switchUnknownOk = harness.switchCatalog("unknown-catalog");
assertEq(switchUnknownOk, undefined, "switchCatalog unknown return");
assertEq(appState.shell.activeCatalog, CatalogKind.LESSON, "switchCatalog unknown should keep activeCatalog");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(buildMockSwitchCatalogPayload({ catalogKind: "unknown-catalog" })),
  "snapshot switchCatalog unknown payload(normalized)",
);
assertEq(toasts.length, switchUnknownToastCount, "switchCatalog unknown should not emit toast");

// switchCatalog (project/lesson transitions)
const switchProjectOk = harness.switchCatalog(CatalogKind.PROJECT);
assertEq(switchProjectOk, undefined, "switchCatalog project return");
assertEq(appState.shell.activeCatalog, CatalogKind.PROJECT, "switchCatalog project should update activeCatalog");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(buildMockSwitchCatalogPayload({ catalogKind: CatalogKind.PROJECT })),
  "snapshot switchCatalog project payload",
);
const switchLessonOk = harness.switchCatalog(CatalogKind.LESSON);
assertEq(switchLessonOk, undefined, "switchCatalog lesson return");
assertEq(appState.shell.activeCatalog, CatalogKind.LESSON, "switchCatalog lesson should update activeCatalog");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(buildMockSwitchCatalogPayload({ catalogKind: CatalogKind.LESSON })),
  "snapshot switchCatalog lesson payload",
);

// switchCatalog (package: toast + no state change)
const switchPackagePrevCatalog = appState.shell.activeCatalog;
harness.switchCatalog(CatalogKind.PACKAGE);
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(buildMockSwitchCatalogPayload({ catalogKind: CatalogKind.PACKAGE })),
  "snapshot switchCatalog payload",
);
assertEq(
  appState.shell.activeCatalog,
  switchPackagePrevCatalog,
  "switchCatalog package should keep activeCatalog",
);
assertIncludes(toasts[toasts.length - 1], "꾸러미 카탈로그는 준비 중입니다.", "switchCatalog package toast");

// saveCurrentWork
const saveLocalOk = harness.saveCurrentWork("local", { ddnText: "x <- 1." });
assertEq(saveLocalOk, true, "saveCurrentWork local return");
assertEq(saves.length, 1, "save call count");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(
    buildMockSaveAdapterPayload({
      target: "local",
      ddnText: "x <- 1.",
      workId: appState.shell.currentWorkId,
      projectId: appState.shell.currentProjectId,
      revisionId: appState.shell.currentRevisionId,
      publicationId: appState.shell.currentPublicationId,
    }),
  ),
  "snapshot saveCurrentWork payload",
);

const saveServerOk = harness.saveCurrentWork("server", { ddnText: "x <- 2." });
assertEq(saveServerOk, false, "saveCurrentWork server return");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(
    buildMockSaveAdapterPayload({
      target: "server",
      ddnText: "x <- 2.",
      workId: appState.shell.currentWorkId,
      projectId: appState.shell.currentProjectId,
      revisionId: appState.shell.currentRevisionId,
      publicationId: appState.shell.currentPublicationId,
    }),
  ),
  "snapshot saveCurrentWork server payload",
);
assertIncludes(toasts[toasts.length - 1], "서버 저장은 준비 중입니다.", "saveCurrentWork server toast");

const saveShareOk = harness.saveCurrentWork("share", { ddnText: "x <- 3." });
assertEq(saveShareOk, false, "saveCurrentWork share return");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(
    buildMockSaveAdapterPayload({
      target: "share",
      ddnText: "x <- 3.",
      workId: appState.shell.currentWorkId,
      projectId: appState.shell.currentProjectId,
      revisionId: appState.shell.currentRevisionId,
      publicationId: appState.shell.currentPublicationId,
    }),
  ),
  "snapshot saveCurrentWork share payload",
);
assertIncludes(toasts[toasts.length - 1], "공유 링크 생성은 준비 중입니다.", "saveCurrentWork share toast");

const saveUnknownOk = harness.saveCurrentWork("remote-x", { ddnText: "x <- 4." });
assertEq(saveUnknownOk, false, "saveCurrentWork unknown return");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(
    buildMockSaveAdapterPayload({
      target: "remote-x",
      ddnText: "x <- 4.",
      workId: appState.shell.currentWorkId,
      projectId: appState.shell.currentProjectId,
      revisionId: appState.shell.currentRevisionId,
      publicationId: appState.shell.currentPublicationId,
    }),
  ),
  "snapshot saveCurrentWork unknown payload(normalized)",
);
assertIncludes(toasts[toasts.length - 1], "지원하지 않는 저장 대상입니다:", "saveCurrentWork unknown toast prefix");
assertIncludes(toasts[toasts.length - 1], "remote-x", "saveCurrentWork unknown toast mode");

// restoreRevision
const restoreDefaultOk = harness.restoreRevision("");
assertEq(restoreDefaultOk, false, "restoreRevision default return");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(
    buildMockRestoreRevisionPayload({
      sourceRevisionId: appState.shell.currentRevisionId,
      restoreMode: RevisionPolicy.RESTORE_MODE,
    }),
  ),
  "snapshot restoreRevision payload",
);

const restoreExplicitOk = harness.restoreRevision("revision-explicit-01");
assertEq(restoreExplicitOk, false, "restoreRevision explicit return");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(
    buildMockRestoreRevisionPayload({
      sourceRevisionId: "revision-explicit-01",
      restoreMode: RevisionPolicy.RESTORE_MODE,
    }),
  ),
  "snapshot restoreRevision explicit payload",
);

// shareCurrent
const shareLessonOk = harness.shareCurrent(ShareKind.CLONE);
assertEq(shareLessonOk, false, "shareCurrent lesson return");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(
    buildMockShareAdapterPayload({
      kind: ShareKind.CLONE,
      objectKind: ObjectKind.LESSON,
      objectId: appState.currentLesson.id,
      visibility: Visibility.PRIVATE,
      sourceRevisionId: appState.shell.currentRevisionId,
      linkTarget: PublicationPolicy.PUBLIC_LINK_TARGET_DEFAULT,
    }),
  ),
  "snapshot shareCurrent payload",
);

appState.currentLesson = null;
const shareProjectOk = harness.shareCurrent(ShareKind.PACKAGE);
assertEq(shareProjectOk, false, "shareCurrent project fallback return");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(
    buildMockShareAdapterPayload({
      kind: ShareKind.PACKAGE,
      objectKind: ObjectKind.PROJECT,
      objectId: appState.shell.currentProjectId,
      visibility: Visibility.PRIVATE,
      sourceRevisionId: appState.shell.currentRevisionId,
      linkTarget: PublicationPolicy.PUBLIC_LINK_TARGET_DEFAULT,
    }),
  ),
  "snapshot shareCurrent project fallback payload",
);

appState.shell.currentProjectId = null;
appState.shell.currentWorkId = "work-02";
const shareWorkOk = harness.shareCurrent(ShareKind.LINK);
assertEq(shareWorkOk, false, "shareCurrent work fallback return");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(
    buildMockShareAdapterPayload({
      kind: ShareKind.LINK,
      objectKind: ObjectKind.PROJECT,
      objectId: appState.shell.currentWorkId,
      visibility: Visibility.PRIVATE,
      sourceRevisionId: appState.shell.currentRevisionId,
      linkTarget: PublicationPolicy.PUBLIC_LINK_TARGET_DEFAULT,
    }),
  ),
  "snapshot shareCurrent work fallback payload",
);

const shareUnknownOk = harness.shareCurrent("Unknown-X");
assertEq(shareUnknownOk, false, "shareCurrent unknown return");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(
    buildMockShareAdapterPayload({
      kind: "unknown-x",
      objectKind: ObjectKind.PROJECT,
      objectId: appState.shell.currentWorkId,
      visibility: Visibility.PRIVATE,
      sourceRevisionId: appState.shell.currentRevisionId,
      linkTarget: PublicationPolicy.PUBLIC_LINK_TARGET_DEFAULT,
    }),
  ),
  "snapshot shareCurrent unknown payload(normalized)",
);
assertIncludes(toasts[toasts.length - 1], "지원하지 않는 공유 방식입니다:", "shareCurrent unknown toast prefix");
assertIncludes(toasts[toasts.length - 1], "unknown-x", "shareCurrent unknown toast mode");

appState.shell.currentWorkId = null;
const shareEmptyIdOk = harness.shareCurrent(ShareKind.LINK);
assertEq(shareEmptyIdOk, false, "shareCurrent empty-id fallback return");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(
    buildMockShareAdapterPayload({
      kind: ShareKind.LINK,
      objectKind: ObjectKind.PROJECT,
      objectId: "",
      visibility: Visibility.PRIVATE,
      sourceRevisionId: appState.shell.currentRevisionId,
      linkTarget: PublicationPolicy.PUBLIC_LINK_TARGET_DEFAULT,
    }),
  ),
  "snapshot shareCurrent empty-id fallback payload",
);
appState.shell.currentWorkId = "work-02";

// publishCurrent
const publishOk = harness.publishCurrent();
assertEq(publishOk, false, "publishCurrent return");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(
    buildMockPublishAdapterPayload({
      projectId: appState.shell.currentProjectId,
      sourceRevisionId: appState.shell.currentRevisionId,
      publicationId: appState.shell.currentPublicationId,
      visibility: Visibility.PRIVATE,
    }),
  ),
  "snapshot publishCurrent payload",
);

appState.shell.currentProjectId = "  ";
appState.shell.currentRevisionId = null;
appState.shell.currentPublicationId = undefined;
const publishNormalizedOk = harness.publishCurrent();
assertEq(publishNormalizedOk, false, "publishCurrent normalized context return");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(
    buildMockPublishAdapterPayload({
      projectId: appState.shell.currentProjectId,
      sourceRevisionId: appState.shell.currentRevisionId,
      publicationId: appState.shell.currentPublicationId,
      visibility: Visibility.PRIVATE,
    }),
  ),
  "snapshot publishCurrent normalized context payload",
);

appState.shell.currentProjectId = "project-01";
appState.shell.currentRevisionId = "revision-01";
appState.shell.currentPublicationId = "publication-01";

// installPackage
appState.shell.activeCatalog = CatalogKind.PROJECT;
const installOk = harness.installPackage("표준/물리/진자", "1.2.3");
assertEq(installOk, false, "installPackage return");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(
    buildMockInstallPackagePayload({
      packageId: "표준/물리/진자",
      version: "1.2.3",
      catalogKind: CatalogKind.PROJECT,
    }),
  ),
  "snapshot installPackage payload",
);

appState.shell.activeCatalog = "invalid-catalog";
const installInvalidCatalogOk = harness.installPackage("표준/물리/운동", "2.0.0");
assertEq(installInvalidCatalogOk, false, "installPackage invalid catalog return");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(
    buildMockInstallPackagePayload({
      packageId: "표준/물리/운동",
      version: "2.0.0",
      catalogKind: appState.shell.activeCatalog,
    }),
  ),
  "snapshot installPackage invalid catalog payload(normalized)",
);

// overwrite order (last call wins): switchCatalog -> installPackage
appState.shell.activeCatalog = CatalogKind.LESSON;
harness.switchCatalog(CatalogKind.PROJECT);
const afterSwitchPayload = stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__);
assertEq(
  afterSwitchPayload,
  stableStringify(buildMockSwitchCatalogPayload({ catalogKind: CatalogKind.PROJECT })),
  "snapshot overwrite order switch payload",
);

const installAfterSwitchOk = harness.installPackage("표준/화학/기본", "3.0.0");
assertEq(installAfterSwitchOk, false, "installPackage after switch return");
assertEq(
  stableStringify(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__),
  stableStringify(
    buildMockInstallPackagePayload({
      packageId: "표준/화학/기본",
      version: "3.0.0",
      catalogKind: appState.shell.activeCatalog,
    }),
  ),
  "snapshot overwrite order final payload",
);

assert(toasts.length >= 1, "toast trace should exist");
assert(saves.length === 1, "local save should be the only file write path");
console.log("seamgrim platform mock payload snapshot runner ok");
