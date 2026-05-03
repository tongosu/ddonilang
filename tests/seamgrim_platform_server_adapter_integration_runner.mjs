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

function assertDeepEq(actual, expected, message) {
  const left = JSON.stringify(actual);
  const right = JSON.stringify(expected);
  if (left !== right) {
    throw new Error(`${message}: expected=${right} actual=${left}`);
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

const appJsPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/app.js");
const appJsText = fs.readFileSync(appJsPath, "utf-8");
const platformContractUrl = pathToFileURL(path.resolve(root, "solutions/seamgrim_ui_mvp/ui/platform_contract.js")).href;
const mockAdapterContractUrl = pathToFileURL(
  path.resolve(root, "solutions/seamgrim_ui_mvp/ui/platform_mock_adapter_contract.js"),
).href;
const serverAdapterContractUrl = pathToFileURL(
  path.resolve(root, "solutions/seamgrim_ui_mvp/ui/platform_server_adapter_contract.js"),
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
} = await import(mockAdapterContractUrl);
const {
  PlatformServerAdapterErrorCode,
  PlatformServerAdapterOp,
  PlatformServerAdapterUiAction,
  buildServerAdapterErrorResponse,
  buildServerAdapterRequest,
  mapServerOpToNotReadyErrorCode,
  resolveServerErrorActionRail,
} = await import(serverAdapterContractUrl);

const functionSource = [
  extractFunctionBody(appJsText, "emitPlatformMockAdapterPayload"),
  extractFunctionBody(appJsText, "resolvePlatformServerAdapterEnabled"),
  extractFunctionBody(appJsText, "emitPlatformServerAdapterExchange"),
  extractFunctionBody(appJsText, "maybeEmitPlatformServerAdapterForOp"),
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
  readWindowBoolean,
  readQueryBoolean,
  showPlatformToast,
  saveDdnToFile,
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
  PlatformServerAdapterOp,
  buildServerAdapterErrorResponse,
  buildServerAdapterRequest,
  mapServerOpToNotReadyErrorCode,
  resolveServerErrorActionRail,
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
  currentLesson: { id: "lesson-01" },
  shell: {
    authSession: { id: "auth-01", userId: "user-01" },
    currentWorkId: "work-01",
    currentProjectId: "project-01",
    currentRevisionId: "revision-01",
    currentPublicationId: "publication-01",
    shareMode: Visibility.PRIVATE,
    activeCatalog: CatalogKind.LESSON,
  },
};

const harness = buildHarness({
  appState,
  window: windowMock,
  readWindowBoolean: (key, fallback = false) => {
    if (key === "SEAMGRIM_ENABLE_PLATFORM_SERVER_ADAPTER") return true;
    return fallback;
  },
  readQueryBoolean: (_key, fallback = false) => fallback,
  showPlatformToast: (message) => {
    toasts.push(String(message ?? ""));
  },
  saveDdnToFile: (text, filename) => {
    saves.push({ text, filename });
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
  PlatformServerAdapterOp,
  buildServerAdapterErrorResponse,
  buildServerAdapterRequest,
  mapServerOpToNotReadyErrorCode,
  resolveServerErrorActionRail,
});

assert(typeof harness.saveCurrentWork === "function", "harness saveCurrentWork missing");

harness.saveCurrentWork("server", { ddnText: "x <- 2." });
assertEq(windowMock.__SEAMGRIM_PLATFORM_SERVER_LAST_REQUEST__.op, PlatformServerAdapterOp.SAVE, "server save op");
assertEq(
  windowMock.__SEAMGRIM_PLATFORM_SERVER_LAST_RESPONSE__.error.code,
  PlatformServerAdapterErrorCode.SAVE_NOT_READY,
  "server save error code",
);
assertDeepEq(
  windowMock.__SEAMGRIM_PLATFORM_SERVER_LAST_ACTION_RAIL__,
  [PlatformServerAdapterUiAction.RETRY_LATER, PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE],
  "server save action rail",
);
assertIncludes(toasts[toasts.length - 1], "서버 저장은 준비 중입니다.", "server save toast");

harness.saveCurrentWork("share", { ddnText: "x <- 3." });
assertEq(windowMock.__SEAMGRIM_PLATFORM_SERVER_LAST_REQUEST__.op, PlatformServerAdapterOp.SHARE, "share-save op");
assertEq(
  windowMock.__SEAMGRIM_PLATFORM_SERVER_LAST_RESPONSE__.error.code,
  PlatformServerAdapterErrorCode.SHARE_NOT_READY,
  "share-save error code",
);

harness.restoreRevision("revision-99");
assertEq(
  windowMock.__SEAMGRIM_PLATFORM_SERVER_LAST_REQUEST__.op,
  PlatformServerAdapterOp.RESTORE_REVISION,
  "restore op",
);
assertEq(
  windowMock.__SEAMGRIM_PLATFORM_SERVER_LAST_RESPONSE__.error.code,
  PlatformServerAdapterErrorCode.RESTORE_NOT_READY,
  "restore error code",
);

harness.publishCurrent();
assertEq(windowMock.__SEAMGRIM_PLATFORM_SERVER_LAST_REQUEST__.op, PlatformServerAdapterOp.PUBLISH, "publish op");
assertEq(
  windowMock.__SEAMGRIM_PLATFORM_SERVER_LAST_RESPONSE__.error.code,
  PlatformServerAdapterErrorCode.PUBLISH_NOT_READY,
  "publish error code",
);

harness.installPackage("표준/물리/중력", "0.0.1");
assertEq(
  windowMock.__SEAMGRIM_PLATFORM_SERVER_LAST_REQUEST__.op,
  PlatformServerAdapterOp.INSTALL_PACKAGE,
  "install op",
);
assertEq(
  windowMock.__SEAMGRIM_PLATFORM_SERVER_LAST_RESPONSE__.error.code,
  PlatformServerAdapterErrorCode.INSTALL_NOT_READY,
  "install error code",
);

harness.switchCatalog(CatalogKind.PACKAGE);
assertEq(
  windowMock.__SEAMGRIM_PLATFORM_SERVER_LAST_REQUEST__.op,
  PlatformServerAdapterOp.SWITCH_CATALOG,
  "switch catalog op",
);
assertEq(
  windowMock.__SEAMGRIM_PLATFORM_SERVER_LAST_RESPONSE__.error.code,
  PlatformServerAdapterErrorCode.CATALOG_NOT_READY,
  "switch catalog error code",
);

const saveLocalOk = harness.saveCurrentWork("local", { ddnText: "x <- 1." });
assertEq(saveLocalOk, true, "local save return");
assertEq(saves.length, 1, "local save called once");

console.log("seamgrim platform server adapter integration runner ok");
