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
  extractFunctionBody(appJsText, "switchCatalog"),
  extractFunctionBody(appJsText, "installPackage"),
].join("\n\n");

const buildHarness = new Function(
  "deps",
  `
const {
  appState,
  CatalogKind,
  showPlatformToast,
  emitPlatformMockAdapterPayload,
  buildMockSwitchCatalogPayload,
  buildMockInstallPackagePayload,
} = deps;
${functionSource}
return {
  switchCatalog,
  installPackage,
};
`,
);

const CatalogKind = {
  LESSON: "lesson_catalog",
  PROJECT: "project_catalog",
  PACKAGE: "package_catalog",
};
const appState = {
  shell: {
    activeCatalog: CatalogKind.LESSON,
  },
};
const toasts = [];
const emittedPayloads = [];
const harness = buildHarness({
  appState,
  CatalogKind,
  showPlatformToast: (message) => {
    toasts.push(String(message ?? ""));
  },
  emitPlatformMockAdapterPayload: (payload) => {
    emittedPayloads.push(payload);
  },
  buildMockSwitchCatalogPayload: (args = {}) => ({
    schema: "mock",
    op: "switch_catalog",
    ...args,
  }),
  buildMockInstallPackagePayload: (args = {}) => ({
    schema: "mock",
    op: "install_package",
    ...args,
  }),
});

// switchCatalog transitions
const switchProjectOk = harness.switchCatalog(CatalogKind.PROJECT);
assertEq(switchProjectOk, undefined, "switch project return");
assertEq(appState.shell.activeCatalog, CatalogKind.PROJECT, "switch project state");
assertEq(emittedPayloads[emittedPayloads.length - 1].catalogKind, CatalogKind.PROJECT, "switch project payload");

const switchUnknownToastCount = toasts.length;
const switchUnknownOk = harness.switchCatalog("unknown-catalog");
assertEq(switchUnknownOk, undefined, "switch unknown return");
assertEq(appState.shell.activeCatalog, CatalogKind.PROJECT, "switch unknown keep state");
assertEq(emittedPayloads[emittedPayloads.length - 1].catalogKind, "unknown-catalog", "switch unknown payload");
assertEq(toasts.length, switchUnknownToastCount, "switch unknown toast count");

const switchPackageOk = harness.switchCatalog(CatalogKind.PACKAGE);
assertEq(switchPackageOk, undefined, "switch package return");
assertEq(appState.shell.activeCatalog, CatalogKind.PROJECT, "switch package keep state");
assertEq(emittedPayloads[emittedPayloads.length - 1].catalogKind, CatalogKind.PACKAGE, "switch package payload");
assertIncludes(toasts[toasts.length - 1], "꾸러미 카탈로그는 준비 중입니다.", "switch package toast");

const switchLessonOk = harness.switchCatalog(CatalogKind.LESSON);
assertEq(switchLessonOk, undefined, "switch lesson return");
assertEq(appState.shell.activeCatalog, CatalogKind.LESSON, "switch lesson state");
assertEq(emittedPayloads[emittedPayloads.length - 1].catalogKind, CatalogKind.LESSON, "switch lesson payload");

// installPackage behavior
appState.shell.activeCatalog = CatalogKind.PROJECT;
const installOk = harness.installPackage("표준/물리/진자", "1.2.3");
assertEq(installOk, false, "install return");
assertEq(emittedPayloads[emittedPayloads.length - 1].packageId, "표준/물리/진자", "install payload packageId");
assertEq(emittedPayloads[emittedPayloads.length - 1].version, "1.2.3", "install payload version");
assertEq(emittedPayloads[emittedPayloads.length - 1].catalogKind, CatalogKind.PROJECT, "install payload catalogKind");
assertIncludes(toasts[toasts.length - 1], "꾸러미 설치(표준/물리/진자@1.2.3)는 준비 중입니다.", "install toast");

const installFallbackOk = harness.installPackage("", "");
assertEq(installFallbackOk, false, "install fallback return");
assertEq(emittedPayloads[emittedPayloads.length - 1].packageId, "-", "install fallback packageId");
assertEq(emittedPayloads[emittedPayloads.length - 1].version, "latest", "install fallback version");
assertIncludes(toasts[toasts.length - 1], "꾸러미 설치(-@latest)는 준비 중입니다.", "install fallback toast");

assert(emittedPayloads.length >= 6, "switch/install payload emission count");
console.log("seamgrim package/registry surface runner ok");

