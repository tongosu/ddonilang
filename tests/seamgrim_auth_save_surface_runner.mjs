import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

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

const appJsPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/app.js");
const appJsText = fs.readFileSync(appJsPath, "utf-8");

const functionSource = [
  extractFunctionBody(appJsText, "readPlatformRouteSlotsFromLocation"),
  extractFunctionBody(appJsText, "applyPlatformRouteFallback"),
  extractFunctionBody(appJsText, "saveCurrentWork"),
].join("\n\n");

const buildHarness = new Function(
  "deps",
  `
const {
  appState,
  window,
  saveDdnToFile,
  showPlatformToast,
  ObjectKind,
  RouteSlotPolicy,
  emitPlatformMockAdapterPayload,
  buildMockSaveAdapterPayload,
} = deps;
${functionSource}
return {
  readPlatformRouteSlotsFromLocation,
  applyPlatformRouteFallback,
  saveCurrentWork,
};
`,
);

const windowMock = {
  location: {
    href: "https://example.test/?work=w-1&revision=r-1&publication=pub-1&project=p-1&lesson=l-1&ddn=x",
  },
};
const appState = {
  shell: {
    currentWorkId: null,
    currentRevisionId: null,
    currentPublicationId: null,
    currentProjectId: null,
  },
};
const toasts = [];
const saves = [];
const emittedPayloads = [];

const harness = buildHarness({
  appState,
  window: windowMock,
  saveDdnToFile: (text, filename) => {
    saves.push({ text, filename });
  },
  showPlatformToast: (message) => {
    toasts.push(String(message ?? ""));
  },
  ObjectKind: {
    WORKSPACE: "workspace",
    REVISION: "revision",
    ARTIFACT: "artifact",
    PROJECT: "project",
  },
  RouteSlotPolicy: {
    PLATFORM_ROUTE_PRECEDENCE: ["work", "revision", "publication", "project"],
    LEGACY_FALLBACK_KEYS: ["lesson", "ddn"],
  },
  emitPlatformMockAdapterPayload: (payload) => {
    emittedPayloads.push(payload);
    windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__ = payload;
  },
  buildMockSaveAdapterPayload: (args = {}) => ({
    schema: "mock",
    op: "save",
    ...args,
  }),
});

// readPlatformRouteSlotsFromLocation
const fullSlots = harness.readPlatformRouteSlotsFromLocation();
assertEq(fullSlots.workId, "w-1", "route slots work");
assertEq(fullSlots.revisionId, "r-1", "route slots revision");
assertEq(fullSlots.publicationId, "pub-1", "route slots publication");
assertEq(fullSlots.projectId, "p-1", "route slots project");
assertEq(fullSlots.hasPlatformSlots, true, "route slots hasPlatformSlots");
assertEq(fullSlots.hasLegacySlots, true, "route slots hasLegacySlots");

windowMock.location.href = "https://example.test/?lesson=l-2";
const legacyOnlySlots = harness.readPlatformRouteSlotsFromLocation();
assertEq(legacyOnlySlots.hasPlatformSlots, false, "legacy-only route hasPlatformSlots");
assertEq(legacyOnlySlots.hasLegacySlots, true, "legacy-only route hasLegacySlots");

windowMock.location.href = "::::";
const malformedSlots = harness.readPlatformRouteSlotsFromLocation();
assertEq(malformedSlots.hasPlatformSlots, false, "malformed route hasPlatformSlots fallback");
assertEq(malformedSlots.hasLegacySlots, false, "malformed route hasLegacySlots fallback");

// applyPlatformRouteFallback
harness.applyPlatformRouteFallback({
  workId: "work-A",
  revisionId: "rev-A",
  publicationId: "pub-A",
  projectId: "proj-A",
});
assertEq(appState.shell.currentWorkId, "work-A", "fallback work slot");
assertEq(appState.shell.currentRevisionId, "rev-A", "fallback revision slot");
assertEq(appState.shell.currentPublicationId, "pub-A", "fallback publication slot");
assertEq(appState.shell.currentProjectId, "proj-A", "fallback project slot");
assertIncludes(toasts[toasts.length - 1], "workspace, revision, artifact, project", "fallback toast object kinds");
assertIncludes(toasts[toasts.length - 1], "교과 탭으로 이동합니다.", "fallback toast browse message");

harness.applyPlatformRouteFallback({});
assertEq(appState.shell.currentWorkId, null, "fallback empty work slot");
assertEq(appState.shell.currentRevisionId, null, "fallback empty revision slot");
assertEq(appState.shell.currentPublicationId, null, "fallback empty publication slot");
assertEq(appState.shell.currentProjectId, null, "fallback empty project slot");
assertIncludes(toasts[toasts.length - 1], "서버 저장 기능은 준비 중입니다.", "fallback empty toast");

// saveCurrentWork
appState.shell.currentWorkId = "work-S";
appState.shell.currentProjectId = "proj-S";
appState.shell.currentRevisionId = "rev-S";
appState.shell.currentPublicationId = "pub-S";

const saveLocalOk = harness.saveCurrentWork("local", { ddnText: "x <- 1." });
assertEq(saveLocalOk, true, "save local return");
assertEq(saves.length, 1, "save local file count");
assertEq(saves[0].filename, "lesson.ddn", "save local filename");
assertEq(saves[0].text, "x <- 1.", "save local content");
assertEq(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__.target, "local", "save local payload target");

const saveServerOk = harness.saveCurrentWork("server", { ddnText: "x <- 2." });
assertEq(saveServerOk, false, "save server return");
assertIncludes(toasts[toasts.length - 1], "서버 저장은 준비 중입니다.", "save server toast");
assertEq(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__.target, "server", "save server payload target");

const saveShareOk = harness.saveCurrentWork("share", { ddnText: "x <- 3." });
assertEq(saveShareOk, false, "save share return");
assertIncludes(toasts[toasts.length - 1], "공유 링크 생성은 준비 중입니다.", "save share toast");
assertEq(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__.target, "share", "save share payload target");

const saveUnknownOk = harness.saveCurrentWork("remote-x", { ddnText: "x <- 4." });
assertEq(saveUnknownOk, false, "save unknown return");
assertIncludes(toasts[toasts.length - 1], "지원하지 않는 저장 대상입니다:", "save unknown toast prefix");
assertIncludes(toasts[toasts.length - 1], "remote-x", "save unknown toast mode");
assertEq(windowMock.__SEAMGRIM_PLATFORM_MOCK_LAST_PAYLOAD__.target, "remote-x", "save unknown payload target");

assert(emittedPayloads.length >= 4, "payload emission count");
console.log("seamgrim auth/save surface runner ok");
