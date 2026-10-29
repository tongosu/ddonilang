#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const proc = spawnSync(process.execPath, ["tests/studio_malblock_workbench_integration_runner.mjs"], {
  cwd: root,
  encoding: "utf8",
});

if ((proc.status ?? 1) !== 0) {
  process.stderr.write(proc.stderr || proc.stdout || "malblock workbench runner failed");
  process.exit(proc.status ?? 1);
}
if (!String(proc.stdout || "").includes("studio_malblock_workbench_integration: ok")) {
  process.stderr.write(proc.stdout || "malblock workbench marker missing");
  process.exit(1);
}

console.log("ganada_la_malblock_workbench: ok");
