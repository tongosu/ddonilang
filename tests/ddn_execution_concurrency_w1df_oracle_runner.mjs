import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { Worker, isMainThread, parentPort, workerData } from "node:worker_threads";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const packRoot = path.join(repoRoot, "pack", "future_grammar_atlas_owner_closed_delta_r2_w1df");
const oraclePath = path.join(packRoot, "logical_oracle.jsonl");
const generatorPath = path.join(packRoot, "build_oracle.mjs");

function sha256(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

function recomputeSemanticDigest(row) {
  const caseId = row.case_id.replace(/^FGA2-/u, "");
  return `sha256:${sha256(Buffer.from(`fga2-w1df\0${caseId}\0${row.normalized_relation_graph}`, "utf8"))}`;
}

function canonicalWorkerRows(rows) {
  return rows
    .map((row) => ({
      case_id: row.case_id,
      diagnostic: row.expected_diagnostic_family,
      digest: recomputeSemanticDigest(row),
      logical_oracle_class: row.logical_oracle_class,
      evidence_class: row.evidence_class,
    }))
    .sort((a, b) => a.case_id.localeCompare(b.case_id, "en", { numeric: true }));
}

if (!isMainThread) {
  parentPort.postMessage(canonicalWorkerRows(workerData.rows));
} else {
const before = fs.readFileSync(oraclePath);
const generated = spawnSync(process.execPath, [generatorPath], {
  cwd: repoRoot,
  encoding: "utf8",
  windowsHide: true,
});
assert.equal(generated.status, 0, `oracle regeneration: ${generated.stderr}`);
const after = fs.readFileSync(oraclePath);
assert.equal(sha256(after), sha256(before), "oracle regeneration must be byte-identical");

const rows = after
  .toString("utf8")
  .trim()
  .split(/\r?\n/u)
  .map((line) => JSON.parse(line));

if (process.env.DDN_W1DF_FAULT_INJECT === "worker_digest_drift") {
  rows[0].expected_semantic_digests.workers_8 = "sha256:fault-injected-worker-drift";
}

assert.equal(rows.length, 65, "65 unique logical fixtures");
assert.equal(new Set(rows.map((row) => row.case_id)).size, 65, "case identity uniqueness");
assert.equal(
  new Set(rows.map((row) => row.owner_delta_id)).size,
  65,
  "owner delta identity uniqueness",
);

const groupCounts = new Map();
for (const row of rows) {
  groupCounts.set(row.group, (groupCounts.get(row.group) ?? 0) + 1);
  assert.deepEqual(row.worker_permutations, [1, 2, 8], `${row.case_id}: worker matrix`);
  assert.deepEqual(
    row.same_tag_permutations,
    ["forward", "reverse", "shuffled"],
    `${row.case_id}: same-tag matrix`,
  );
  const digests = row.expected_semantic_digests;
  assert.equal(digests.workers_1, digests.workers_2, `${row.case_id}: workers 1/2`);
  assert.equal(digests.workers_1, digests.workers_8, `${row.case_id}: workers 1/8`);
  assert.equal(row.identity_receipts.stable_semantic_identity, digests.workers_1);
  assert.equal(row.identity_receipts.excludes_physical_order, true);
  assert.equal(row.resource_unit_ledger.host_bytes_semantic, false);
  assert.equal(row.patch_outbox_effect_candidates.partial_commit_forbidden, true);
  assert.equal(row.stable_public_wire, "OPEN");
  assert.equal(row.logical_oracle_class, "LOGICAL_ORACLE_ONLY");
  assert.equal(Object.hasOwn(row, "product_status"), false, `${row.case_id}: stale product_status`);
  assert.equal(JSON.stringify(row).includes("PRODUCT_PASS"), false, `${row.case_id}: stale PRODUCT_PASS label`);
}

assert.deepEqual(
  Object.fromEntries([...groupCounts].sort()),
  {
    "FGA2-G01": 16,
    "FGA2-G02": 12,
    "FGA2-G03": 15,
    "FGA2-G04": 3,
    "FGA2-G05": 10,
    "FGA2-G06": 7,
    "FGA2-G07": 2,
  },
  "seven-group cardinality",
);

const publicSourceCases = rows
  .filter((row) => row.evidence_class === "PUBLIC_SOURCE_PRODUCT_CASE")
  .map((row) => row.case_id)
  .sort();
assert.deepEqual(publicSourceCases, ["FGA2-N10", "FGA2-P09", "FGA2-P10", "FGA2-X20"]);

const internalTypedCases = rows
  .filter((row) => row.evidence_class === "INTERNAL_TYPED_PRODUCT_CASE")
  .map((row) => row.case_id)
  .sort();
assert.deepEqual(internalTypedCases, ["FGA2-N11", "FGA2-P11", "FGA2-X14"]);

for (const row of rows.filter((item) => item.evidence_class !== "PRODUCT_UNSUPPORTED")) {
  assert.notEqual(row.evidence_frontdoor, "NONE", `${row.case_id}: evidence frontdoor`);
  assert.equal(row.evidence_test_ids.length > 0, true, `${row.case_id}: evidence test IDs`);
  assert.match(row.evidence_fixed_head, /^[0-9a-f]{40}$/u, `${row.case_id}: evidence fixed HEAD`);
  assert.notEqual(row.evidence_scope, "NO_CURRENT_PRODUCT_EVIDENCE", `${row.case_id}: evidence scope`);
}

assert.equal(
  rows.filter((row) => row.evidence_class === "PRODUCT_UNSUPPORTED").length,
  58,
  "unsupported logical meaning remains explicitly open",
);

function permuteRows(sourceRows, permutation) {
  if (permutation === "forward") return [...sourceRows];
  if (permutation === "reverse") return [...sourceRows].reverse();
  return [...sourceRows].sort((a, b) =>
    sha256(Buffer.from(`same-tag-shuffle\0${a.case_id}`, "utf8")).localeCompare(
      sha256(Buffer.from(`same-tag-shuffle\0${b.case_id}`, "utf8")),
    ),
  );
}

async function runWorkerMatrix(workerCount, permutation) {
  const buckets = Array.from({ length: workerCount }, () => []);
  for (const [index, row] of permuteRows(rows, permutation).entries()) {
    buckets[index % workerCount].push(row);
  }
  const batches = await Promise.all(
    buckets.map(
      (workerRows) =>
        new Promise((resolve, reject) => {
          const worker = new Worker(new URL(import.meta.url), {
            workerData: { rows: workerRows },
          });
          worker.once("message", resolve);
          worker.once("error", reject);
          worker.once("exit", (code) => {
            if (code !== 0) reject(new Error(`oracle worker exited ${code}`));
          });
        }),
    ),
  );
  return batches
    .flat()
    .sort((a, b) => a.case_id.localeCompare(b.case_id, "en", { numeric: true }));
}

const matrixDigests = [];
for (const workerCount of [1, 2, 8]) {
  for (const permutation of ["forward", "reverse", "shuffled"]) {
    const result = await runWorkerMatrix(workerCount, permutation);
    for (const row of result) {
      const expected = rows.find((candidate) => candidate.case_id === row.case_id);
      assert.equal(row.digest, expected.expected_semantic_digests.workers_1);
    }
    matrixDigests.push(sha256(Buffer.from(JSON.stringify(result), "utf8")));
  }
}
assert.equal(new Set(matrixDigests).size, 1, "worker/permutation semantic result convergence");

console.log(JSON.stringify({
  schema: "ddn.execution_concurrency.w1df.logical_oracle_check.v1",
  cases: rows.length,
  groups: Object.fromEntries([...groupCounts].sort()),
  logical_oracle_cases: rows.length,
  public_source_product_cases: publicSourceCases.length,
  internal_typed_product_cases: internalTypedCases.length,
  product_unsupported_cases: 58,
  workers: [1, 2, 8],
  same_tag_permutations: ["forward", "reverse", "shuffled"],
  worker_matrix_runs: matrixDigests.length,
  logical_worker_matrix_digest: matrixDigests[0],
  logical_metadata_digest: sha256(after),
  w1c2_public_surface: "OPEN",
  rendezvous_runtime: "OPEN",
  rd207_stable_public_wire: "OPEN",
}));
}
