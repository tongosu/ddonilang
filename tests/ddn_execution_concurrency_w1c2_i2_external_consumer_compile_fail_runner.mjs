import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { basename, dirname, relative, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const targetDir = process.env.DDN_W1C2_I2_EXTERNAL_CARGO_TARGET_DIR;
const tempParent = process.env.DDN_W1C2_I2_EXTERNAL_CONSUMER_TEMP_ROOT;
if (!targetDir || !tempParent) {
  throw new Error(
    "DDN_W1C2_I2_EXTERNAL_CARGO_TARGET_DIR and DDN_W1C2_I2_EXTERNAL_CONSUMER_TEMP_ROOT are required (fail closed)",
  );
}

const resolvedParent = resolve(tempParent);
if (!basename(resolvedParent).startsWith("ddn-w1c2-i2-external-consumer-")) {
  throw new Error("external consumer temp root must use the task-owned ddn-w1c2-i2-external-consumer-* prefix");
}
if (!relative(root, resolvedParent).startsWith("..")) {
  throw new Error("external consumer temp root must be outside the repository");
}

const runtimePath = resolve(root, "tool/src/ddn_runtime.rs");
const libPath = resolve(root, "tool/src/lib.rs");
const runtimeText = readFileSync(runtimePath, "utf8");
const libText = readFileSync(libPath, "utf8");
const builderNames = [
  "with_internal_reaction_provider_registry",
  "with_internal_provider_reachability",
  "with_internal_reaction_runtime",
];
const externalRustProviderInjectionApiCount = builderNames.filter((name) =>
  new RegExp(`\\bpub\\s+fn\\s+${name}\\b`, "u").test(runtimeText),
).length;
const crateRootInjectionReexportCount = /pub\s+use\s+ddn_runtime::DdnRunner\s*;/u.test(libText) ? 1 : 0;

mkdirSync(dirname(resolvedParent), { recursive: true });
mkdirSync(resolvedParent, { recursive: true });
const consumerRoot = mkdtempSync(resolve(resolvedParent, "case-"));
const toolDependencyPath = resolve(root, "tool").replaceAll("\\", "/");
try {
  mkdirSync(resolve(consumerRoot, "src"), { recursive: true });
  writeFileSync(
    resolve(consumerRoot, "Cargo.toml"),
    `[package]\nname = "w1c2_i2_external_consumer_probe"\nversion = "0.0.0"\nedition = "2021"\n\n[dependencies]\nddonirang-tool = { path = "${toolDependencyPath}" }\n`,
    "utf8",
  );
  writeFileSync(
    resolve(consumerRoot, "src/main.rs"),
    `use ddonirang_tool::ddn_runtime::DdnRunner;\n\nfn main() {\n    let _forbidden_builder = DdnRunner::with_internal_reaction_provider_registry;\n}\n`,
    "utf8",
  );
  const result = spawnSync("cargo", ["check", "--offline"], {
    cwd: consumerRoot,
    env: { ...process.env, CARGO_TARGET_DIR: resolve(targetDir) },
    encoding: "utf8",
    windowsHide: true,
  });
  const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
  const privateMethodCompileFail =
    result.status !== 0 &&
    /(?:associated function|method) `with_internal_reaction_provider_registry` is private/u.test(output) &&
    /E0624/u.test(output);
  if (!privateMethodCompileFail) {
    throw new Error(`external consumer must fail specifically at the private injection builder\n${output}`);
  }
  if (externalRustProviderInjectionApiCount !== 0 || crateRootInjectionReexportCount !== 0) {
    throw new Error("external provider injection ingress count must remain zero");
  }
  console.log(
    JSON.stringify({
      status: "PASS",
      external_consumer_compile_fail: true,
      diagnostic: "E0624_private_injection_builder",
      external_rust_provider_injection_api_count: externalRustProviderInjectionApiCount,
      crate_root_injection_reexport_count: crateRootInjectionReexportCount,
    }),
  );
} finally {
  rmSync(consumerRoot, { recursive: true, force: true });
  if (resolvedParent.startsWith(dirname(resolvedParent)) && basename(resolvedParent).startsWith("ddn-w1c2-i2-external-consumer-")) {
    rmSync(resolvedParent, { recursive: true, force: true });
  }
}
