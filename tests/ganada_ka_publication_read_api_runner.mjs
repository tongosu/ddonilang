#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const proc = spawnSync(process.execPath, ["tests/ttonimaru_publication_read_api_runner.mjs"], {
  cwd: root,
  encoding: "utf8",
});

if ((proc.status ?? 1) !== 0) {
  process.stderr.write(proc.stderr || proc.stdout || "publication read API runner failed");
  process.exit(proc.status ?? 1);
}
if (!String(proc.stdout || "").includes("ttonimaru_publication_read_api: ok")) {
  process.stderr.write(proc.stdout || "publication read API marker missing");
  process.exit(1);
}

console.log("ganada_ka_publication_read_api: ok");
