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

const contractPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/platform_server_adapter_contract.js");
const contractUrl = pathToFileURL(contractPath).href;
const {
  PlatformServerAdapterErrorCode,
  PlatformServerAdapterUiAction,
  resolveServerErrorActionRail,
} = await import(contractUrl);

assertDeepEq(
  resolveServerErrorActionRail(PlatformServerAdapterErrorCode.AUTH_REQUIRED),
  [PlatformServerAdapterUiAction.LOGIN, PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE],
  "auth action rail",
);
assertDeepEq(
  resolveServerErrorActionRail(PlatformServerAdapterErrorCode.PERMISSION_DENIED),
  [PlatformServerAdapterUiAction.REQUEST_ACCESS, PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE],
  "permission action rail",
);
assertDeepEq(
  resolveServerErrorActionRail(PlatformServerAdapterErrorCode.VALIDATION_FAILED),
  [PlatformServerAdapterUiAction.FIX_INPUT, PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE],
  "validation action rail",
);
assertDeepEq(
  resolveServerErrorActionRail(PlatformServerAdapterErrorCode.SAVE_NOT_READY),
  [PlatformServerAdapterUiAction.RETRY_LATER, PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE],
  "not-ready action rail",
);

const appJsPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/app.js");
const appJsText = fs.readFileSync(appJsPath, "utf-8");
const emitFnSource = extractFunctionBody(appJsText, "emitPlatformServerAdapterExchange");

const buildHarness = new Function(
  "deps",
  `
const { window, resolveServerErrorActionRail, CustomEvent } = deps;
${emitFnSource}
return { emitPlatformServerAdapterExchange };
`,
);

const dispatchedEvents = [];
const windowMock = {
  dispatchEvent(event) {
    dispatchedEvents.push(event);
    return true;
  },
};
class MockCustomEvent {
  constructor(type, init = {}) {
    this.type = type;
    this.detail = init?.detail ?? null;
  }
}
const harness = buildHarness({
  window: windowMock,
  resolveServerErrorActionRail,
  CustomEvent: MockCustomEvent,
});

harness.emitPlatformServerAdapterExchange(
  { op: "share" },
  { error: { code: PlatformServerAdapterErrorCode.PERMISSION_DENIED } },
);
assertEq(
  windowMock.__SEAMGRIM_PLATFORM_SERVER_LAST_REQUEST__.op,
  "share",
  "request snapshot set",
);
assertEq(
  windowMock.__SEAMGRIM_PLATFORM_SERVER_LAST_RESPONSE__.error.code,
  PlatformServerAdapterErrorCode.PERMISSION_DENIED,
  "response snapshot set",
);
assertDeepEq(
  windowMock.__SEAMGRIM_PLATFORM_SERVER_LAST_ACTION_RAIL__,
  [PlatformServerAdapterUiAction.REQUEST_ACCESS, PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE],
  "action rail snapshot set",
);
assertEq(dispatchedEvents.length, 1, "exchange event should be dispatched once");
assertEq(
  dispatchedEvents[0].type,
  "seamgrim:platform-server-adapter-exchange",
  "exchange event name",
);
assertEq(
  dispatchedEvents[0].detail?.response?.error?.code,
  PlatformServerAdapterErrorCode.PERMISSION_DENIED,
  "exchange event response code",
);
assertDeepEq(
  dispatchedEvents[0].detail?.action_rail,
  [PlatformServerAdapterUiAction.REQUEST_ACCESS, PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE],
  "exchange event action rail",
);

console.log("seamgrim platform server action rail runner ok");
