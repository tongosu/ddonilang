import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const baseline = "2aa621546af534ec82b6ea89ad0384132b3a4e42";
const ledgerPath = path.join(
  root,
  "pack",
  "ddn_execution_concurrency_w1c2_internal_typed",
  "expected.jsonl",
);
const targetDir = process.env.DDN_W1C2_CARGO_TARGET_DIR;
if (!targetDir) {
  throw new Error("DDN_W1C2_CARGO_TARGET_DIR is required (fail closed)");
}

const rows = fs
  .readFileSync(ledgerPath, "utf8")
  .trim()
  .split(/\r?\n/u)
  .map((line) => JSON.parse(line));
if (rows.length !== 23) {
  throw new Error(`expected 23 evidence rows, got ${rows.length}`);
}
const allowedClasses = new Set([
  "INTERNAL_TYPED_PRODUCT_CASE",
  "PUBLIC_SOURCE_REGRESSION_CASE",
  "PRODUCT_UNSUPPORTED",
]);
const caseIds = new Set();
const testIds = new Set();
for (const row of rows) {
  if (!allowedClasses.has(row.classification)) {
    throw new Error(`invalid classification ${row.classification}`);
  }
  if (caseIds.has(row.case_id) || testIds.has(row.test_id)) {
    throw new Error(`duplicate ledger identity ${row.case_id}/${row.test_id}`);
  }
  caseIds.add(row.case_id);
  testIds.add(row.test_id);
}

function cargo(args) {
  const result = spawnSync("cargo", args, {
    cwd: root,
    env: { ...process.env, CARGO_TARGET_DIR: targetDir },
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  const output = `${result.stdout ?? ""}${result.stderr ?? ""}`;
  if (result.status !== 0) {
    throw new Error(`cargo ${args.join(" ")} failed (${result.status})\n${output}`);
  }
  return output;
}

const listed = cargo([
  "test",
  "-p",
  "ddonirang-tool",
  "--test",
  "ddn_execution_concurrency_w1c2_internal_typed",
  "--",
  "--list",
]);
const compiled = new Set(
  listed
    .split(/\r?\n/u)
    .map((line) => line.match(/^(w1c2_i1_case_[a-zA-Z0-9_]+): test$/u)?.[1])
    .filter(Boolean),
);
for (const testId of testIds) {
  if (!compiled.has(testId)) {
    throw new Error(`ledger test is not compiled: ${testId}`);
  }
}
if (compiled.size !== testIds.size) {
  throw new Error(`compiled/ledger test count mismatch ${compiled.size}/${testIds.size}`);
}

const execution = cargo([
  "test",
  "-p",
  "ddonirang-tool",
  "--test",
  "ddn_execution_concurrency_w1c2_internal_typed",
  "--",
  "--nocapture",
]);
if (!/test result: ok\. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out/u.test(execution)) {
  throw new Error("whole integration binary did not report exact 23/23 execution");
}

const publicIngressPaths = [
  "lang/src/ast.rs",
  "lang/src/parser.rs",
  "lang/src/normalizer.rs",
  "lang/src/currentline.rs",
  "tool/src/preprocess.rs",
  "tool/src/wasm_api.rs",
  "tools/teul-cli/src/lang/ast.rs",
  "tools/teul-cli/src/lang/parser.rs",
  "tools/teul-cli/src/cli",
].map((entry) => path.join(root, entry));
const koreanProviderTokens = ["반응제공자", "반응하면"];
for (const target of publicIngressPaths) {
  const stat = fs.statSync(target);
  const files = stat.isDirectory()
    ? fs.readdirSync(target, { recursive: true })
        .filter((entry) => entry.endsWith(".rs"))
        .map((entry) => path.join(target, entry))
    : [target];
  for (const file of files) {
    const text = fs.readFileSync(file, "utf8");
    for (const token of koreanProviderTokens) {
      if (text.includes(token)) {
        throw new Error(`public provider token ${token} found in ${path.relative(root, file)}`);
      }
    }
  }
}

const ast = fs.readFileSync(path.join(root, "lang/src/ast.rs"), "utf8");
const receiveStart = ast.indexOf("Receive {");
const receiveEnd = ast.indexOf("Send {", receiveStart + "Receive {".length);
const receiveHead = ast.slice(receiveStart, receiveEnd > receiveStart ? receiveEnd : receiveStart + 500);
if (/provider/iu.test(receiveHead)) {
  throw new Error("Stmt::Receive acquired a provider flag");
}

const teulEval = fs.readFileSync(
  path.join(root, "tools/teul-cli/src/runtime/eval.rs"),
  "utf8",
);
if (!teulEval.includes("plan_delivery_occurrence(") ||
    !teulEval.includes("ReactionProviderRegistry::<()>::default()")) {
  throw new Error("teul async route is not bound to shared delivery authority with empty registry");
}
if (teulEval.includes("select_exactly_one_endpoint(&routes)")) {
  throw new Error("teul legacy local route selection fallback remains active");
}

const diff = spawnSync(
  "git",
  [
    "diff",
    "--name-only",
    baseline,
    "--",
    "lang/src/ast.rs",
    "lang/src/parser.rs",
    "lang/src/normalizer.rs",
    "lang/src/currentline.rs",
    "tool/src/preprocess.rs",
    "tool/src/wasm_api.rs",
  ],
  { cwd: root, encoding: "utf8" },
);
if (diff.status !== 0 || diff.stdout.trim() !== "") {
  throw new Error(`public syntax/preprocess/WASM ingress paths changed: ${diff.stdout}`);
}

const counts = Object.fromEntries(
  [...allowedClasses].map((kind) => [kind, rows.filter((row) => row.classification === kind).length]),
);
console.log(
  JSON.stringify({
    status: "PASS",
    cases: rows.length,
    classifications: counts,
    compiled_tests: compiled.size,
    public_provider_source_ingress: 0,
    public_provider_tokens: 0,
    tool_ddnrunner_public_provider_ingress: 0,
    teul_cli_empty_registry_static_scan: "PASS",
  }),
);
