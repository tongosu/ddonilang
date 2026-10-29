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
  extractFunctionBody(appJsText, "republishCurrent"),
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
  republishCurrent,
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
  PUBLIC_LINK_TARGET_DEFAULT: "artifact",
  PINNED_REVISION_REQUIRED: true,
  REPUBLISH_APPEND_ONLY: true,
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

const shareLinkOk = harness.shareCurrent(ShareKind.LINK);
assertEq(shareLinkOk, false, "share link return");
assertIncludes(toasts[toasts.length - 1], "링크 공유(artifact)는 준비 중입니다.", "share link toast");
assertEq(emittedPayloads[emittedPayloads.length - 1].kind, "link", "share link payload kind");
assertEq(emittedPayloads[emittedPayloads.length - 1].linkTarget, "artifact", "share link payload target");

const publishOk = harness.publishCurrent();
assertEq(publishOk, false, "publish return");
assertIncludes(toasts[toasts.length - 1], "게시 기능은 준비 중입니다.", "publish toast");

appState.shell.currentRevisionId = "";
const publishMissingRevisionOk = harness.publishCurrent();
assertEq(publishMissingRevisionOk, false, "publish missing revision return");
assertIncludes(toasts[toasts.length - 1], "게시 실패: source_revision_id가 필요합니다.", "publish missing revision toast");
appState.shell.currentRevisionId = "revision-1";

PublicationPolicy.SNAPSHOT_IMMUTABLE = false;
const publishPolicyFailOk = harness.publishCurrent();
assertEq(publishPolicyFailOk, false, "publish policy fail return");
assertIncludes(toasts[toasts.length - 1], "게시 스냅샷 정책이 잘못되었습니다.", "publish policy fail toast");
PublicationPolicy.SNAPSHOT_IMMUTABLE = true;

const republishOk = harness.republishCurrent();
assertEq(republishOk, false, "republish return");
assertIncludes(toasts[toasts.length - 1], "새 publication record append-only", "republish append-only toast");

PublicationPolicy.REPUBLISH_APPEND_ONLY = false;
const republishPolicyFailOk = harness.republishCurrent();
assertEq(republishPolicyFailOk, false, "republish policy fail return");
assertIncludes(toasts[toasts.length - 1], "재게시 정책이 잘못되었습니다.", "republish policy fail toast");
PublicationPolicy.REPUBLISH_APPEND_ONLY = true;

appState.shell.currentPublicationId = "";
const republishMissingOk = harness.republishCurrent();
assertEq(republishMissingOk, false, "republish missing publication return");
assertIncludes(toasts[toasts.length - 1], "재게시할 publication이 없습니다.", "republish missing publication toast");

assert(emittedPayloads.length >= 3, "publication snapshot payload emission count");
console.log("seamgrim publication snapshot surface runner ok");
