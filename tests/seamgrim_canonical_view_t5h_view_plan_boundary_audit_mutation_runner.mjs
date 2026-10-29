#!/usr/bin/env node

import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const root = path.resolve(import.meta.dirname, "..");
const sourcePath = path.join(root, "solutions/seamgrim_ui_mvp/ui/view_plan_runtime.js");
const source = await fs.readFile(sourcePath, "utf8");
const tempRoot = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-t5h-view-plan-boundary-mutation-"));

function runCandidate(candidate, label) {
  const candidatePath = path.join(tempRoot, `${label}.mjs`);
  return fs.writeFile(candidatePath, candidate, "utf8").then(() => {
    const run = spawnSync(process.execPath, ["tests/seamgrim_canonical_view_t5h_view_plan_boundary_audit_runner.mjs"], {
      cwd: root,
      env: { ...process.env, DDN_T5H_VIEW_PLAN_MODULE_PATH: candidatePath, DDN_T5H_VIEW_PLAN_VALIDATE_ONLY: "1" },
      encoding: "utf8",
      windowsHide: true,
      timeout: 30_000,
    });
    return run.status;
  });
}

try {
  assert(await runCandidate(source, "baseline") === 0, "baseline view-plan boundary is not GREEN");

  const canonicalSynthesis = source.replace("canonical_space2d_synthesis: false", "canonical_space2d_synthesis: true");
  assert(canonicalSynthesis !== source, "M1 mutation anchor missing");
  assert(await runCandidate(canonicalSynthesis, "m1_canonical_synthesis") !== 0, "M1 canonical synthesis claim was not RED");

  const authorityPromotion = source.replace(
    'space2dAuthorityClass: presentationSpace2d ? "PRESENTATION_ONLY" : null',
    'space2dAuthorityClass: presentationSpace2d ? "AUTHORITATIVE_STRUCTURED" : null',
  );
  assert(authorityPromotion !== source, "M2 mutation anchor missing");
  assert(await runCandidate(authorityPromotion, "m2_authority_promotion") !== 0, "M2 presentation authority promotion was not RED");

  assert(await runCandidate(source, "restored") === 0, "restored view-plan boundary is not GREEN");
  console.log(JSON.stringify({
    status: "PASS",
    mutation_count: 2,
    mutations: ["M1_CANONICAL_SPACE2D_SYNTHESIS_CLAIM", "M2_PRESENTATION_AUTHORITY_PROMOTION"],
    mutation_residue: 0,
  }));
} finally {
  await fs.rm(tempRoot, { recursive: true, force: true });
}
