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
  extractFunctionBody(appJsText, "openRevisionHistory"),
  extractFunctionBody(appJsText, "compareRevisionWithHead"),
  extractFunctionBody(appJsText, "duplicateCurrentWork"),
].join("\n\n");

const buildHarness = new Function(
  "deps",
  `
const {
  appState,
  window,
  showPlatformToast,
  RouteSlotPolicy,
} = deps;
${functionSource}
return {
  readPlatformRouteSlotsFromLocation,
  openRevisionHistory,
  compareRevisionWithHead,
  duplicateCurrentWork,
};
`,
);

const toasts = [];
const windowMock = {
  location: {
    href: "https://example.test/?work=w-1&revision=r-1&publication=p-1&project=proj-1&lesson=l-1",
  },
};
const appState = {
  shell: {
    currentWorkId: "work-1",
    currentRevisionId: "revision-1",
    currentPublicationId: "publication-1",
  },
};
const RouteSlotPolicy = {
  PLATFORM_ROUTE_PRECEDENCE: ["work", "revision", "publication", "project"],
  LEGACY_FALLBACK_KEYS: ["lesson", "ddn"],
};

const harness = buildHarness({
  appState,
  window: windowMock,
  showPlatformToast: (message) => {
    toasts.push(String(message ?? ""));
  },
  RouteSlotPolicy,
});

const fullSlots = harness.readPlatformRouteSlotsFromLocation();
assertEq(fullSlots.workId, "w-1", "route slots work");
assertEq(fullSlots.revisionId, "r-1", "route slots revision");
assertEq(fullSlots.publicationId, "p-1", "route slots publication");
assertEq(fullSlots.projectId, "proj-1", "route slots project");
assertEq(fullSlots.hasPlatformSlots, true, "route slots hasPlatformSlots");
assertEq(fullSlots.hasLegacySlots, true, "route slots hasLegacySlots");

windowMock.location.href = "https://example.test/?lesson=legacy-only";
const legacyOnlySlots = harness.readPlatformRouteSlotsFromLocation();
assertEq(legacyOnlySlots.hasPlatformSlots, false, "legacy-only route hasPlatformSlots");
assertEq(legacyOnlySlots.hasLegacySlots, true, "legacy-only route hasLegacySlots");

windowMock.location.href = "::::";
const malformedSlots = harness.readPlatformRouteSlotsFromLocation();
assertEq(malformedSlots.hasPlatformSlots, false, "malformed route hasPlatformSlots fallback");
assertEq(malformedSlots.hasLegacySlots, false, "malformed route hasLegacySlots fallback");

const historyOk = harness.openRevisionHistory();
assertEq(historyOk, false, "openRevisionHistory return");
assertIncludes(toasts[toasts.length - 1], "리비전 기록 화면은 준비 중입니다.", "openRevisionHistory toast");

appState.shell.currentRevisionId = "revision-head-check";
const compareOk = harness.compareRevisionWithHead();
assertEq(compareOk, false, "compareRevisionWithHead return");
assertIncludes(toasts[toasts.length - 1], "revision-head-check", "compareRevisionWithHead revision toast");
assertIncludes(toasts[toasts.length - 1], "HEAD", "compareRevisionWithHead head toast");

appState.shell.currentRevisionId = "";
const compareMissingOk = harness.compareRevisionWithHead();
assertEq(compareMissingOk, false, "compareRevisionWithHead missing return");
assertIncludes(toasts[toasts.length - 1], "비교할 리비전이 없습니다.", "compareRevisionWithHead missing toast");

appState.shell.currentWorkId = "work-dup";
const duplicateOk = harness.duplicateCurrentWork();
assertEq(duplicateOk, false, "duplicateCurrentWork return");
assertIncludes(toasts[toasts.length - 1], "작업 복제(work-dup)는 준비 중입니다.", "duplicateCurrentWork toast");

appState.shell.currentWorkId = "";
const duplicateMissingOk = harness.duplicateCurrentWork();
assertEq(duplicateMissingOk, false, "duplicateCurrentWork missing return");
assertIncludes(toasts[toasts.length - 1], "복제할 작업이 없습니다.", "duplicateCurrentWork missing toast");

assert(toasts.length >= 5, "source management toast count");
console.log("seamgrim source management surface runner ok");
