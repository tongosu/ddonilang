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

class CustomEventStub {
  constructor(type, init = {}) {
    this.type = String(type ?? "");
    this.detail = init && typeof init === "object" ? init.detail : undefined;
  }
}

const appJsPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/app.js");
const appJsText = fs.readFileSync(appJsPath, "utf-8");

const functionSource = [
  extractFunctionBody(appJsText, "emitPlatformReviewAction"),
  extractFunctionBody(appJsText, "requestReview"),
  extractFunctionBody(appJsText, "approvePublication"),
  extractFunctionBody(appJsText, "rejectPublication"),
].join("\n\n");

const buildHarness = new Function(
  "deps",
  `
const {
  appState,
  window,
  showPlatformToast,
  PLATFORM_REVIEW_ACTION_EVENT,
  CustomEvent,
} = deps;
${functionSource}
return {
  requestReview,
  approvePublication,
  rejectPublication,
};
`,
);

const toasts = [];
const events = [];
const windowMock = {
  dispatchEvent(event) {
    events.push(event);
    return true;
  },
};
const appState = {
  shell: {
    currentWorkId: "work-1",
    currentRevisionId: "revision-1",
    currentPublicationId: "publication-1",
    reviewStatus: "pending",
  },
};

const harness = buildHarness({
  appState,
  window: windowMock,
  showPlatformToast: (message) => {
    toasts.push(String(message ?? ""));
  },
  PLATFORM_REVIEW_ACTION_EVENT: "seamgrim:platform-review-action",
  CustomEvent: CustomEventStub,
});

const requestOk = harness.requestReview();
assertEq(requestOk, false, "requestReview return");
assertEq(appState.shell.reviewStatus, "pending", "requestReview status");
assertIncludes(toasts[toasts.length - 1], "검토 요청은 준비 중입니다.", "requestReview toast");
assertEq(events[events.length - 1].type, "seamgrim:platform-review-action", "requestReview event type");
assertEq(events[events.length - 1].detail.action, "request", "requestReview event action");
assertEq(events[events.length - 1].detail.workId, "work-1", "requestReview event workId");

const approveOk = harness.approvePublication();
assertEq(approveOk, false, "approvePublication return");
assertEq(appState.shell.reviewStatus, "approved", "approvePublication status");
assertIncludes(toasts[toasts.length - 1], "검토 승인 처리는 준비 중입니다.", "approvePublication toast");
assertEq(events[events.length - 1].detail.action, "approve", "approvePublication event action");
assertEq(events[events.length - 1].detail.revisionId, "revision-1", "approvePublication event revisionId");

const rejectOk = harness.rejectPublication();
assertEq(rejectOk, false, "rejectPublication return");
assertEq(appState.shell.reviewStatus, "rejected", "rejectPublication status");
assertIncludes(toasts[toasts.length - 1], "검토 반려 처리는 준비 중입니다.", "rejectPublication toast");
assertEq(events[events.length - 1].detail.action, "reject", "rejectPublication event action");
assertEq(events[events.length - 1].detail.publicationId, "publication-1", "rejectPublication event publicationId");

assert(windowMock.__SEAMGRIM_PLATFORM_REVIEW_LAST_ACTION__, "review last action snapshot missing");
assertEq(windowMock.__SEAMGRIM_PLATFORM_REVIEW_LAST_ACTION__.action, "reject", "review last action final snapshot");
console.log("seamgrim review workflow surface runner ok");
