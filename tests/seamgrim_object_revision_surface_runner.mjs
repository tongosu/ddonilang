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
const functionSource = extractFunctionBody(appJsText, "restoreRevision");

const buildHarness = new Function(
  "deps",
  `
const {
  appState,
  RevisionPolicy,
  SourceManagementPolicy,
  showPlatformToast,
  emitPlatformMockAdapterPayload,
  buildMockRestoreRevisionPayload,
} = deps;
${functionSource}
return {
  restoreRevision,
};
`,
);

const appState = {
  shell: {
    currentRevisionId: "revision-shell",
  },
};
const toasts = [];
const emittedPayloads = [];
const RevisionPolicy = {
  RESTORE_MODE: "new_revision",
};
const SourceManagementPolicy = {
  REVISION_APPEND_ONLY: true,
  RESTORE_CREATES_NEW_REVISION: true,
  OVERWRITE_FORBIDDEN: true,
};

const harness = buildHarness({
  appState,
  RevisionPolicy,
  SourceManagementPolicy,
  showPlatformToast: (message) => {
    toasts.push(String(message ?? ""));
  },
  emitPlatformMockAdapterPayload: (payload) => {
    emittedPayloads.push(payload);
  },
  buildMockRestoreRevisionPayload: (args = {}) => ({
    schema: "mock",
    op: "restore_revision",
    ...args,
  }),
});

// default: fallback to shell revision
const restoreDefaultOk = harness.restoreRevision("");
assertEq(restoreDefaultOk, false, "restore default return");
assertIncludes(toasts[toasts.length - 1], "리비전 복원은 준비 중입니다. (새 revision으로 복원 예정)", "restore default toast");
assertEq(emittedPayloads[emittedPayloads.length - 1].sourceRevisionId, "revision-shell", "restore default payload source");
assertEq(emittedPayloads[emittedPayloads.length - 1].restoreMode, "new_revision", "restore default payload mode");

// explicit revision should override shell
const restoreExplicitOk = harness.restoreRevision("revision-explicit");
assertEq(restoreExplicitOk, false, "restore explicit return");
assertEq(emittedPayloads[emittedPayloads.length - 1].sourceRevisionId, "revision-explicit", "restore explicit payload source");

// empty source guard
appState.shell.currentRevisionId = null;
const restoreEmptyOk = harness.restoreRevision("");
assertEq(restoreEmptyOk, false, "restore empty return");
assertIncludes(toasts[toasts.length - 1], "복원할 리비전이 없습니다.", "restore empty toast");
assertEq(emittedPayloads[emittedPayloads.length - 1].sourceRevisionId, "", "restore empty payload source");

// policy guard
RevisionPolicy.RESTORE_MODE = "overwrite";
const restorePolicyOk = harness.restoreRevision("revision-policy");
assertEq(restorePolicyOk, false, "restore policy return");
assertIncludes(toasts[toasts.length - 1], "리비전 복원 정책이 잘못되었습니다.", "restore policy toast");
assertEq(emittedPayloads[emittedPayloads.length - 1].restoreMode, "overwrite", "restore policy payload mode");
RevisionPolicy.RESTORE_MODE = "new_revision";

SourceManagementPolicy.REVISION_APPEND_ONLY = false;
const restoreSourcePolicyOk = harness.restoreRevision("revision-source-policy");
assertEq(restoreSourcePolicyOk, false, "restore source policy return");
assertIncludes(
  toasts[toasts.length - 1],
  "소스관리 정책이 잘못되었습니다. (append-only/new-revision/overwrite-forbidden)",
  "restore source policy toast",
);
SourceManagementPolicy.REVISION_APPEND_ONLY = true;

assert(emittedPayloads.length >= 4, "restore payload emission count");
console.log("seamgrim object/revision surface runner ok");
