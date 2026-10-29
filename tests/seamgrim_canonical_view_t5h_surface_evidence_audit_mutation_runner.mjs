#!/usr/bin/env node

import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const root = path.resolve(import.meta.dirname, "..");
const sourcePath = path.join(root, "pack/seamgrim_canonical_view_t5h_readiness_audit_v1/surface_evidence_matrix.detjson");
const baseline = JSON.parse(await fs.readFile(sourcePath, "utf8"));
const tempRoot = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-t5h-surface-evidence-mutation-"));

function runCandidate(candidate, label) {
  const candidatePath = path.join(tempRoot, `${label}.detjson`);
  return fs.writeFile(candidatePath, `${JSON.stringify(candidate, null, 2)}\n`, "utf8").then(() => {
    const run = spawnSync(process.execPath, ["tests/seamgrim_canonical_view_t5h_surface_evidence_audit_runner.mjs"], {
      cwd: root,
      env: {
        ...process.env,
        DDN_T5H_SURFACE_EVIDENCE_MATRIX: candidatePath,
        DDN_T5H_SURFACE_AUDIT_VALIDATE_ONLY: "1",
      },
      encoding: "utf8",
      windowsHide: true,
      timeout: 30_000,
    });
    return run.status;
  });
}

try {
  assert(await runCandidate(structuredClone(baseline), "baseline") === 0, "baseline surface evidence matrix is not GREEN");

  const missingCell = structuredClone(baseline);
  missingCell.rows[0].evidence.splice(1, 1);
  assert(await runCandidate(missingCell, "m1_missing_surface_cell") !== 0, "M1 missing surface cell was not RED");

  const promotedPlayground = structuredClone(baseline);
  promotedPlayground.rows[0].evidence.push({
    surface: "playground",
    classification: "PRODUCT_FUNCTION_BYTE_PARITY",
    executable_id: "T5B_GOLDEN",
    required_marker: "run_workbench_console_byte_parity: true",
  });
  promotedPlayground.expected_classification_counts.PRODUCT_FUNCTION_BYTE_PARITY += 1;
  promotedPlayground.claim_boundaries.matrix_cell_count += 1;
  assert(await runCandidate(promotedPlayground, "m2_playground_silent_promotion") !== 0, "M2 Playground silent promotion was not RED");

  const missingNonConsumer = structuredClone(baseline);
  missingNonConsumer.non_consumer_evidence.keys.shift();
  missingNonConsumer.claim_boundaries.excluded_non_consumer_cells -= 1;
  assert(await runCandidate(missingNonConsumer, "m3_run_non_consumer_omitted") !== 0,
    "M3 Run non-consumer omission was not RED");

  assert(await runCandidate(structuredClone(baseline), "restored") === 0, "restored surface evidence matrix is not GREEN");
  console.log(JSON.stringify({
    status: "PASS",
    mutation_count: 3,
    mutations: [
      "M1_MISSING_APPLICABLE_SURFACE_CELL",
      "M2_PLAYGROUND_LEGACY_COMPATIBILITY_SILENT_PROMOTION",
      "M3_RUN_NON_CONSUMER_OMITTED"
    ],
    mutation_residue: 0,
  }));
} finally {
  await fs.rm(tempRoot, { recursive: true, force: true });
}
