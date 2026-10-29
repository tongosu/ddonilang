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
  extractFunctionBody(appJsText, "shareCurrent"),
  extractFunctionBody(appJsText, "publishCurrent"),
].join("\n\n");

const buildHarness = new Function(
  "deps",
  `
const {
  appState,
  showPlatformToast,
  ShareKind,
  ObjectKind,
  Visibility,
  RevisionPolicy,
  PublishPolicy,
  PublicationPolicy,
  emitPlatformMockAdapterPayload,
  buildMockShareAdapterPayload,
  buildMockPublishAdapterPayload,
} = deps;
${functionSource}
return {
  shareCurrent,
  publishCurrent,
};
`,
);

const appState = {
  currentLesson: { id: "lesson-1" },
  shell: {
    currentWorkId: "work-1",
    currentProjectId: "project-1",
    currentRevisionId: "revision-1",
    currentPublicationId: "publication-1",
  },
};
const toasts = [];
const emittedPayloads = [];
const ShareKind = { LINK: "link", CLONE: "clone", PACKAGE: "package" };
const ObjectKind = { LESSON: "lesson", PROJECT: "project" };
const Visibility = { PRIVATE: "private" };
const RevisionPolicy = { SOURCE_REVISION_ID_REQUIRED: true };
const PublishPolicy = { ARTIFACT_TRACKS_DRAFT: false };
const PublicationPolicy = {
  SNAPSHOT_IMMUTABLE: true,
  PINNED_REVISION_REQUIRED: true,
  REPUBLISH_APPEND_ONLY: true,
  PUBLIC_LINK_TARGET_DEFAULT: "artifact",
};

const harness = buildHarness({
  appState,
  showPlatformToast: (message) => {
    toasts.push(String(message ?? ""));
  },
  ShareKind,
  ObjectKind,
  Visibility,
  RevisionPolicy,
  PublishPolicy,
  PublicationPolicy,
  emitPlatformMockAdapterPayload: (payload) => {
    emittedPayloads.push(payload);
  },
  buildMockShareAdapterPayload: (args = {}) => ({
    schema: "mock",
    op: "share",
    ...args,
  }),
  buildMockPublishAdapterPayload: (args = {}) => ({
    schema: "mock",
    op: "publish",
    ...args,
  }),
});

// shareCurrent base kinds
const shareLinkOk = harness.shareCurrent(ShareKind.LINK);
assertEq(shareLinkOk, false, "share link return");
assertIncludes(toasts[toasts.length - 1], "링크 공유(artifact)는 준비 중입니다.", "share link toast");
assertEq(emittedPayloads[emittedPayloads.length - 1].kind, "link", "share link payload kind");
assertEq(emittedPayloads[emittedPayloads.length - 1].objectKind, "lesson", "share link payload objectKind");
assertEq(emittedPayloads[emittedPayloads.length - 1].objectId, "lesson-1", "share link payload objectId");
assertEq(emittedPayloads[emittedPayloads.length - 1].linkTarget, "artifact", "share link payload target");

const shareCloneOk = harness.shareCurrent(ShareKind.CLONE);
assertEq(shareCloneOk, false, "share clone return");
assertIncludes(toasts[toasts.length - 1], "복제 공유는 준비 중입니다.", "share clone toast");

const sharePackageOk = harness.shareCurrent(ShareKind.PACKAGE);
assertEq(sharePackageOk, false, "share package return");
assertIncludes(toasts[toasts.length - 1], "꾸러미 공유는 준비 중입니다.", "share package toast");

const shareUnknownOk = harness.shareCurrent("Unknown-X");
assertEq(shareUnknownOk, false, "share unknown return");
assertIncludes(toasts[toasts.length - 1], "지원하지 않는 공유 방식입니다:", "share unknown toast prefix");
assertIncludes(toasts[toasts.length - 1], "unknown-x", "share unknown toast mode");

// shareCurrent object fallback: lesson -> project -> work -> empty
appState.currentLesson = null;
appState.shell.currentProjectId = "project-2";
harness.shareCurrent(ShareKind.LINK);
assertEq(emittedPayloads[emittedPayloads.length - 1].objectKind, "project", "share fallback objectKind project");
assertEq(emittedPayloads[emittedPayloads.length - 1].objectId, "project-2", "share fallback objectId project");

appState.shell.currentProjectId = null;
appState.shell.currentWorkId = "work-2";
harness.shareCurrent(ShareKind.LINK);
assertEq(emittedPayloads[emittedPayloads.length - 1].objectId, "work-2", "share fallback objectId work");

appState.shell.currentWorkId = null;
harness.shareCurrent(ShareKind.LINK);
assertEq(emittedPayloads[emittedPayloads.length - 1].objectId, "", "share fallback objectId empty");

// publishCurrent base
appState.shell.currentProjectId = "project-1";
appState.shell.currentRevisionId = "revision-1";
appState.shell.currentPublicationId = "publication-1";
const publishOk = harness.publishCurrent();
assertEq(publishOk, false, "publish return");
assertIncludes(toasts[toasts.length - 1], "게시 기능은 준비 중입니다.", "publish toast");
assertEq(emittedPayloads[emittedPayloads.length - 1].op, "publish", "publish payload op");
assertEq(emittedPayloads[emittedPayloads.length - 1].projectId, "project-1", "publish payload projectId");

// publish policy guard: source revision required
RevisionPolicy.SOURCE_REVISION_ID_REQUIRED = false;
const publishSourcePolicyOk = harness.publishCurrent();
assertEq(publishSourcePolicyOk, false, "publish source policy return");
assertIncludes(toasts[toasts.length - 1], "게시 정책이 잘못되었습니다.", "publish source policy toast");
RevisionPolicy.SOURCE_REVISION_ID_REQUIRED = true;

// publish policy guard: artifact draft tracking forbidden
PublishPolicy.ARTIFACT_TRACKS_DRAFT = true;
const publishDraftPolicyOk = harness.publishCurrent();
assertEq(publishDraftPolicyOk, false, "publish draft policy return");
assertIncludes(
  toasts[toasts.length - 1],
  "게시 정책이 잘못되었습니다. (artifact draft 추적 금지)",
  "publish draft policy toast",
);
PublishPolicy.ARTIFACT_TRACKS_DRAFT = false;

PublicationPolicy.SNAPSHOT_IMMUTABLE = false;
const publishSnapshotPolicyOk = harness.publishCurrent();
assertEq(publishSnapshotPolicyOk, false, "publish snapshot policy return");
assertIncludes(toasts[toasts.length - 1], "게시 스냅샷 정책이 잘못되었습니다.", "publish snapshot policy toast");
PublicationPolicy.SNAPSHOT_IMMUTABLE = true;

appState.shell.currentRevisionId = "";
const publishMissingRevisionOk = harness.publishCurrent();
assertEq(publishMissingRevisionOk, false, "publish missing revision return");
assertIncludes(toasts[toasts.length - 1], "게시 실패: source_revision_id가 필요합니다.", "publish missing revision toast");
appState.shell.currentRevisionId = "revision-1";

assert(emittedPayloads.length >= 8, "share/publish payload emission count");
console.log("seamgrim sharing/publishing surface runner ok");
