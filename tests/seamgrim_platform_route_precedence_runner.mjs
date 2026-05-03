import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, "..");

function assertEq(actual, expected, message) {
  if (actual !== expected) {
    throw new Error(`${message}: expected=${JSON.stringify(expected)} actual=${JSON.stringify(actual)}`);
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
  extractFunctionBody(appJsText, "shouldApplyPlatformRouteFallback"),
].join("\n\n");

const buildHarness = new Function(
  "deps",
  `
const { window, RouteSlotPolicy } = deps;
${functionSource}
return {
  readPlatformRouteSlotsFromLocation,
  shouldApplyPlatformRouteFallback,
};
`,
);

const windowMock = {
  location: {
    href: "https://example.test/?work=w-1&revision=r-1&publication=p-1&project=proj-1&lesson=l-1&ddn=abc",
  },
};

const harness = buildHarness({
  window: windowMock,
  RouteSlotPolicy: {
    PLATFORM_ROUTE_PRECEDENCE: ["work", "revision", "publication", "project"],
    LEGACY_FALLBACK_KEYS: ["lesson", "ddn"],
  },
});

const mixedSlots = harness.readPlatformRouteSlotsFromLocation();
assertEq(mixedSlots.hasPlatformSlots, true, "mixed route must include platform slots");
assertEq(mixedSlots.hasLegacySlots, true, "mixed route must include legacy slots");
assertEq(
  harness.shouldApplyPlatformRouteFallback(mixedSlots),
  true,
  "platform route must take precedence over legacy route",
);

windowMock.location.href = "https://example.test/?lesson=l-only&ddn=x";
const legacyOnlySlots = harness.readPlatformRouteSlotsFromLocation();
assertEq(legacyOnlySlots.hasPlatformSlots, false, "legacy-only route must not include platform slots");
assertEq(legacyOnlySlots.hasLegacySlots, true, "legacy-only route should include legacy slots");
assertEq(
  harness.shouldApplyPlatformRouteFallback(legacyOnlySlots),
  false,
  "legacy-only route must not trigger platform fallback",
);

windowMock.location.href = "https://example.test/?project=proj-9";
const platformOnlySlots = harness.readPlatformRouteSlotsFromLocation();
assertEq(platformOnlySlots.hasPlatformSlots, true, "platform-only route should include platform slots");
assertEq(platformOnlySlots.hasLegacySlots, false, "platform-only route should not include legacy slots");
assertEq(
  harness.shouldApplyPlatformRouteFallback(platformOnlySlots),
  true,
  "platform-only route should trigger platform fallback",
);

windowMock.location.href = "::::";
const malformedSlots = harness.readPlatformRouteSlotsFromLocation();
assertEq(malformedSlots.hasPlatformSlots, false, "malformed route should fallback to empty platform slots");
assertEq(malformedSlots.hasLegacySlots, false, "malformed route should fallback to empty legacy slots");
assertEq(
  harness.shouldApplyPlatformRouteFallback(malformedSlots),
  false,
  "malformed route must not trigger platform fallback",
);

console.log("seamgrim platform route precedence runner ok");
