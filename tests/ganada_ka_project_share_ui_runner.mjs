#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const proc = spawnSync(process.execPath, ["tests/ttonimaru_project_share_ui_runner.mjs"], {
  cwd: root,
  encoding: "utf8",
});

if ((proc.status ?? 1) !== 0) {
  process.stderr.write(proc.stderr || proc.stdout || "project share UI runner failed");
  process.exit(proc.status ?? 1);
}
if (!String(proc.stdout || "").includes("ttonimaru_project_share_ui: ok")) {
  process.stderr.write(proc.stdout || "project share UI marker missing");
  process.exit(1);
}

console.log("ganada_ka_project_share_ui: ok");
