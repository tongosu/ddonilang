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
const tempRoot = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-t5h-projection-nonmutation-"));

function runCandidate(candidate, label) {
  const candidatePath = path.join(tempRoot, `${label}.mjs`);
  return fs.writeFile(candidatePath, candidate, "utf8").then(() => {
    const run = spawnSync(process.execPath, ["tests/seamgrim_canonical_view_t5h_projection_nonmutation_audit_runner.mjs"], {
      cwd: root,
      env: { ...process.env, DDN_T5H_CANONICAL_OWNER_MODULE_PATH: candidatePath, DDN_T5H_PROJECTION_NONMUTATION_VALIDATE_ONLY: "1" },
      encoding: "utf8",
      windowsHide: true,
      timeout: 30_000,
    });
    return run.status;
  });
}

try {
  assert(await runCandidate(source, "baseline") === 0, "baseline canonical projection is not GREEN");

  const inputMutation = source.replace(
    "const entries = Array.isArray(projection?.entries) ? projection.entries : [];",
    "const entries = Array.isArray(projection?.entries) ? projection.entries : []; entries.push({});",
  );
  assert(inputMutation !== source, "M1 mutation anchor missing");
  assert(await runCandidate(inputMutation, "m1_input_mutation") !== 0, "M1 authoritative input mutation was not RED");

  const outputAlias = source.replace(
    /if \(value && typeof value === "object"\) \{\r?\n\s*return Object\.fromEntries\(/,
    'if (value && typeof value === "object") {\n    return value;\n    return Object.fromEntries(',
  );
  assert(outputAlias !== source, "M2 mutation anchor missing");
  assert(await runCandidate(outputAlias, "m2_output_alias") !== 0, "M2 output/input alias was not RED");

  assert(await runCandidate(source, "restored") === 0, "restored canonical projection is not GREEN");
  console.log(JSON.stringify({
    status: "PASS",
    mutation_count: 2,
    mutations: ["M1_AUTHORITATIVE_INPUT_MUTATION", "M2_OUTPUT_INPUT_OBJECT_ALIAS"],
    mutation_residue: 0,
  }));
} finally {
  await fs.rm(tempRoot, { recursive: true, force: true });
}
