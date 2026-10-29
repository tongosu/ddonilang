#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const proc = spawnSync(process.execPath, ["tests/malblock_authoring_ui_browser_runner.mjs"], {
  cwd: root,
  encoding: "utf8",
});

if ((proc.status ?? 1) !== 0) {
  process.stderr.write(proc.stderr || proc.stdout || "malblock first-run runner failed");
  process.exit(proc.status ?? 1);
}
if (!String(proc.stdout || "").includes("malblock_authoring_ui_browser: ok")) {
  process.stderr.write(proc.stdout || "malblock first-run marker missing");
  process.exit(1);
}

console.log("ganada_la_malblock_first_run: ok");
