#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function normalizeStdout(stdout) {
  return String(stdout ?? "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("state_hash=") && !line.startsWith("trace_hash="));
}

const root = process.cwd();
const packDir = path.join(root, "pack", "seamgrim_console_rich_markup_v1");
const contract = JSON.parse(await fs.readFile(path.join(packDir, "contract.detjson"), "utf8"));
const runMod = await import(pathToFileURL(path.join(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js")).href);
const raw = contract.expected_cli_stdout[0];

const cli = spawnSync(
  "cargo",
  ["run", "-q", "--manifest-path", "tools/teul-cli/Cargo.toml", "--", "run", path.join(packDir, contract.input)],
  { cwd: root, encoding: "utf8", windowsHide: true },
);
assert((cli.status ?? 1) === 0, `cli failed: ${cli.stderr || cli.stdout}`);
assert(JSON.stringify(normalizeStdout(cli.stdout)) === JSON.stringify(contract.expected_cli_stdout), "cli raw string output mismatch");

const parsed = runMod.parseConsoleRichMarkup(raw);
assert(parsed.plainText === contract.expected_plain_text, "rich markup plain text mismatch");
assert(parsed.rich === true, "rich markup should be detected");
for (const token of contract.expected_html_tokens) {
  assert(String(parsed.html).includes(token), `rich html token missing: ${token}`);
}

const visual = runMod.resolveRunMainVisualMode({
  outputRows: [{ key: "경고", value: raw, syntheticKey: false }],
});
assert(visual.mode === "console-grid", "rich output should use console-grid");
assert(Array.isArray(visual.consoleLinesForGrid), "console grid lines missing");
assert(visual.consoleLinesForGrid.includes(contract.expected_plain_text), "console grid should receive plain text");
assert(!visual.consoleLinesForGrid.includes(raw), "console grid should not receive raw rich markup");
assert(String(visual.consoleHtml ?? "").includes('data-rich="1"'), "console html rich marker missing");
assert(!String(visual.consoleHtml ?? "").includes("\\색{빨강}"), "console html should render markup instead of raw marker");

const unknown = runMod.parseConsoleRichMarkup(contract.unknown_markup_plain_text);
assert(unknown.plainText === contract.unknown_markup_plain_text, "unknown rich markup should remain plain text");
assert(Array.isArray(unknown.warnings) && unknown.warnings.length === 1, "unknown rich markup warning missing");

const report = {
  schema: "ddn.seamgrim_console_rich_markup.report.v1",
  ok: true,
  pack_id: contract.pack_id,
  plain_text: parsed.plainText,
  console_grid_value: visual.consoleLinesForGrid[0],
  raw_string_preserved: normalizeStdout(cli.stdout)[0] === raw,
  html_rich: String(visual.consoleHtml ?? "").includes('data-rich="1"'),
  unknown_markup_preserved: unknown.plainText === contract.unknown_markup_plain_text,
  runtime_truth_owner: "string_value",
};

console.log(JSON.stringify(report, null, 2));
