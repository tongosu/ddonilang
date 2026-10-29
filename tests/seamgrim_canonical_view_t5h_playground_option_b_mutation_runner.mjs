#!/usr/bin/env node

import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const root = path.resolve(import.meta.dirname, "..");
const packRoot = path.join(root, "pack/seamgrim_canonical_view_t5h_readiness_audit_v1");
const contract = JSON.parse(await fs.readFile(path.join(packRoot, "contract.detjson"), "utf8"));
const playHtml = await fs.readFile(path.join(root, "solutions/seamgrim_ui_mvp/ui/play.html"), "utf8");
const legacyModule = await fs.readFile(path.join(root, "solutions/seamgrim_ui_mvp/ui/playground.js"), "utf8");
const tempRoot = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-t5h-playground-option-b-"));

async function runCandidate({ candidateContract = contract, candidateHtml = playHtml, candidateModule = legacyModule }, label) {
  const contractPath = path.join(tempRoot, `${label}.contract.detjson`);
  const htmlPath = path.join(tempRoot, `${label}.html`);
  const modulePath = path.join(tempRoot, `${label}.mjs`);
  await Promise.all([
    fs.writeFile(contractPath, `${JSON.stringify(candidateContract, null, 2)}\n`, "utf8"),
    fs.writeFile(htmlPath, candidateHtml, "utf8"),
    fs.writeFile(modulePath, candidateModule, "utf8"),
  ]);
  const run = spawnSync(process.execPath, ["tests/seamgrim_canonical_view_t5h_playground_surface_identity_audit_runner.mjs"], {
    cwd: root,
    env: {
      ...process.env,
      DDN_T5H_CONTRACT_PATH: contractPath,
      DDN_T5H_PLAY_HTML_PATH: htmlPath,
      DDN_T5H_LEGACY_PLAYGROUND_MODULE_PATH: modulePath,
    },
    encoding: "utf8",
    windowsHide: true,
    timeout: 30_000,
  });
  return run.status;
}

try {
  assert(await runCandidate({}, "baseline") === 0, "baseline Playground OPTION_B boundary is not GREEN");

  const promotedHtml = playHtml.replace(
    'data-full-product-surface-identity="false"',
    'data-full-product-surface-identity="true"',
  );
  assert(promotedHtml !== playHtml, "M1 mutation anchor missing");
  assert(await runCandidate({ candidateHtml: promotedHtml }, "m1_full_surface_promotion") !== 0,
    "M1 full product-surface promotion was not RED");

  const hiddenFallback = legacyModule.replace(
    "allowObservationOutputFallback: false",
    "allowObservationOutputFallback: true",
  );
  assert(hiddenFallback !== legacyModule, "M2 mutation anchor missing");
  assert(await runCandidate({ candidateModule: hiddenFallback }, "m2_hidden_fallback") !== 0,
    "M2 hidden compatibility fallback was not RED");

  const currentSurfacePromotion = structuredClone(contract);
  currentSurfacePromotion.consumer_matrix.console.push("playground");
  assert(await runCandidate({ candidateContract: currentSurfacePromotion }, "m3_current_surface_promotion") !== 0,
    "M3 Playground current-surface promotion was not RED");

  assert(await runCandidate({}, "restored") === 0, "restored Playground OPTION_B boundary is not GREEN");
  console.log(JSON.stringify({
    status: "PASS",
    mutation_count: 3,
    mutations: [
      "M1_PLAYGROUND_FULL_PRODUCT_SURFACE_PROMOTION",
      "M2_PLAYGROUND_HIDDEN_SEMANTIC_FALLBACK",
      "M3_PLAYGROUND_CURRENT_SURFACE_PROMOTION",
    ],
    mutation_residue: 0,
  }));
} finally {
  await fs.rm(tempRoot, { recursive: true, force: true });
}
