#!/usr/bin/env node

import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const root = path.resolve(import.meta.dirname, "..");
const sourcePath = path.join(root, "solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js");
const source = await fs.readFile(sourcePath, "utf8");
const tempRoot = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-t5h-status-totality-"));

async function runCandidate(candidate, label) {
  const candidatePath = path.join(tempRoot, `${label}.mjs`);
  await fs.writeFile(candidatePath, candidate, "utf8");
  const run = spawnSync(process.execPath, ["tests/seamgrim_canonical_view_t5h_readiness_audit_runner.mjs"], {
    cwd: root,
    env: { ...process.env, DDN_T5H_CANONICAL_OWNER_MODULE_PATH: candidatePath },
    encoding: "utf8",
    windowsHide: true,
    timeout: 30_000,
  });
  return run.status;
}

try {
  assert(await runCandidate(source, "baseline") === 0, "baseline status totality is not GREEN");

  const unknownPassThrough = source.replace(
    /status: accepted \? raw : "invalid",/,
    "status: raw,",
  );
  assert(unknownPassThrough !== source, "M1 mutation anchor missing");
  assert(await runCandidate(unknownPassThrough, "m1_unknown_pass_through") !== 0,
    "M1 unknown status pass-through was not RED");

  const ambiguousRejected = source.replace(
    /\r?\n\s*"ambiguous",\r?\n\]\);/,
    "\n]);",
  );
  assert(ambiguousRejected !== source, "M2 mutation anchor missing");
  assert(await runCandidate(ambiguousRejected, "m2_ambiguous_rejected") !== 0,
    "M2 accepted ambiguous status rejection was not RED");

  assert(await runCandidate(source, "restored") === 0, "restored status totality is not GREEN");
  console.log(JSON.stringify({
    status: "PASS",
    mutation_count: 2,
    mutations: [
      "M1_UNKNOWN_STATUS_PASS_THROUGH",
      "M2_ACCEPTED_AMBIGUOUS_STATUS_REJECTED",
    ],
    mutation_residue: 0,
  }));
} finally {
  await fs.rm(tempRoot, { recursive: true, force: true });
}
