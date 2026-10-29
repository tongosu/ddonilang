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

function makeButton() {
  return {
    disabled: false,
    title: "",
    attrs: {},
    setAttribute(key, value) {
      this.attrs[String(key)] = String(value);
    },
  };
}

const appJsPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/app.js");
const appJsText = fs.readFileSync(appJsPath, "utf-8");
const functionSource = [
  extractFunctionBody(appJsText, "resolvePlatformMockMenuEnabled"),
  extractFunctionBody(appJsText, "applyPlatformMockMenuState"),
].join("\n\n");

const buildHarness = new Function(
  "deps",
  `
const {
  readWindowBoolean,
  readQueryBoolean,
} = deps;
${functionSource}
return {
  resolvePlatformMockMenuEnabled,
  applyPlatformMockMenuState,
};
`,
);

const makeResolverHarness = (windowEnabled, queryMenuEnabled, queryAliasEnabled) =>
  buildHarness({
    readWindowBoolean: (key, fallback = false) => {
      if (key === "SEAMGRIM_ENABLE_PLATFORM_MOCK_MENU") return windowEnabled;
      return fallback;
    },
    readQueryBoolean: (key, fallback = false) => {
      if (key === "platform_mock_menu") return queryMenuEnabled;
      if (key === "platform_mock") return queryAliasEnabled;
      return fallback;
    },
  });

const disabledHarness = makeResolverHarness(false, false, false);
assertEq(disabledHarness.resolvePlatformMockMenuEnabled(), false, "resolve menu disabled");

const windowEnabledHarness = makeResolverHarness(true, false, false);
assertEq(windowEnabledHarness.resolvePlatformMockMenuEnabled(), true, "resolve menu enabled by window");

const queryEnabledHarness = makeResolverHarness(false, true, false);
assertEq(queryEnabledHarness.resolvePlatformMockMenuEnabled(), true, "resolve menu enabled by query");

const aliasEnabledHarness = makeResolverHarness(false, false, true);
assertEq(aliasEnabledHarness.resolvePlatformMockMenuEnabled(), true, "resolve menu enabled by alias query");

const buttonsDisabled = [makeButton(), makeButton(), makeButton()];
const applyDisabledOk = disabledHarness.applyPlatformMockMenuState(buttonsDisabled, false);
assertEq(applyDisabledOk, false, "apply disabled return");
buttonsDisabled.forEach((button, index) => {
  assertEq(button.disabled, true, `apply disabled: button disabled ${index}`);
  assertEq(button.attrs["aria-disabled"], "true", `apply disabled: aria-disabled ${index}`);
  assertEq(button.title, "준비 중", `apply disabled: title ${index}`);
});

const buttonsEnabled = [makeButton(), makeButton()];
const applyEnabledOk = disabledHarness.applyPlatformMockMenuState(buttonsEnabled, true);
assertEq(applyEnabledOk, true, "apply enabled return");
buttonsEnabled.forEach((button, index) => {
  assertEq(button.disabled, false, `apply enabled: button enabled ${index}`);
  assertEq(button.attrs["aria-disabled"], "false", `apply enabled: aria-disabled ${index}`);
  assertIncludes(button.title, "플랫폼 mock 표면 활성화", `apply enabled: title ${index}`);
});

const applyInvalidRowsOk = disabledHarness.applyPlatformMockMenuState([null, undefined, {}], true);
assertEq(applyInvalidRowsOk, true, "apply invalid rows return");

console.log("seamgrim platform mock menu mode runner ok");
