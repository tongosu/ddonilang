import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import crypto from "node:crypto";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const pack = process.env.DDN_VC06_PACK_OVERRIDE
  ? path.resolve(process.env.DDN_VC06_PACK_OVERRIDE)
  : path.join(root, "pack", "v1_core_text_byte_nfc_rebase_w1");
const manifestPath = path.join(pack, "manifest.detjson");
const wasmDir = process.env.DDN_VC06_WASM_DIR_OVERRIDE
  ? path.resolve(process.env.DDN_VC06_WASM_DIR_OVERRIDE)
  : path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "wasm");

function sha256(buffer) {
  return crypto.createHash("sha256").update(buffer).digest("hex");
}

function requireFile(file, code) {
  if (!fs.existsSync(file) || !fs.statSync(file).isFile()) {
    throw new Error(`${code}:${file}`);
  }
}

function parseStructuredError(error) {
  const text = String(error);
  const start = text.indexOf("{");
  if (start >= 0) {
    try {
      return JSON.parse(text.slice(start));
    } catch {
      // Preserve the exact product error below.
    }
  }
  return { raw_error: text };
}

function readSource(testCase) {
  const fixture = path.join(pack, testCase.path);
  requireFile(fixture, "E_FIXTURE_MISSING");
  const bytes = fs.readFileSync(fixture);
  if (sha256(bytes) !== testCase.fixture_sha256) {
    throw new Error(`E_FIXTURE_HASH_MISMATCH:${testCase.id}`);
  }
  if (testCase.fixture_kind === "raw_utf8") {
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  }
  if (testCase.fixture_kind !== "transport_json") {
    throw new Error(`E_FIXTURE_KIND_UNSUPPORTED:${testCase.id}`);
  }
  const transport = JSON.parse(bytes.toString("utf8"));
  if (transport.encoding === "utf-8-json-string") {
    return String(transport.source_text);
  }
  if (transport.encoding === "hex-bytes") {
    const decoded = Buffer.from(String(transport.bytes_hex), "hex");
    try {
      return new TextDecoder("utf-8", { fatal: true }).decode(decoded);
    } catch {
      return null;
    }
  }
  throw new Error(`E_TRANSPORT_ENCODING_UNSUPPORTED:${testCase.id}`);
}

function runCase(wasm, source, testCase, profile) {
  try {
    return {
      outcome: "ACCEPTED",
      payload: JSON.parse(wasm.wasm_run_summary_with_source_profile_json(
        source,
        testCase.path,
        1,
        profile,
      )),
    };
  } catch (error) {
    return { outcome: "REJECTED", diagnostic: parseStructuredError(error) };
  }
}

function canonCase(wasm, source, profile) {
  try {
    const first = wasm.wasm_canon_ddn_with_source_profile(source, profile);
    try {
      const second = wasm.wasm_canon_ddn_with_source_profile(first, profile);
      return {
        first_outcome: "ACCEPTED",
        first_ddn: first,
        second_outcome: "ACCEPTED",
        second_ddn: second,
        roundtrip_equal: first === second,
      };
    } catch (error) {
      return {
        first_outcome: "ACCEPTED",
        first_ddn: first,
        second_outcome: "REJECTED",
        second_error: String(error),
        roundtrip_equal: false,
      };
    }
  } catch (error) {
    return {
      first_outcome: "REJECTED",
      first_error: String(error),
      second_outcome: "NOT_RUN",
      roundtrip_equal: false,
    };
  }
}

function runtimeCanonReparse(wasm, run, testCase, profile) {
  const canonical = run?.payload?.canonical_ddn;
  if (typeof canonical !== "string") return null;
  const reparse = runCase(wasm, canonical, testCase, profile);
  return {
    canonical_ddn_sha256: sha256(Buffer.from(canonical, "utf8")),
    canonical_ddn: canonical,
    reparse,
    reparse_green: reparse.outcome === "ACCEPTED",
  };
}

async function main() {
  requireFile(manifestPath, "E_MANIFEST_MISSING");
  const manifestBytes = fs.readFileSync(manifestPath);
  const manifest = JSON.parse(manifestBytes.toString("utf8"));
  if (manifest.cases.length !== manifest.required_case_count) {
    throw new Error(`E_CASE_OMISSION:${manifest.cases.length}/${manifest.required_case_count}`);
  }

  const wasmPath = path.join(wasmDir, "ddonirang_tool_bg.wasm");
  const jsPath = path.join(wasmDir, "ddonirang_tool.js");
  const trackedManifestPath = path.join(wasmDir, "ddonirang_tool.manifest.detjson");
  requireFile(wasmPath, "E_TRACKED_WASM_MISSING");
  requireFile(jsPath, "E_TRACKED_WASM_JS_MISSING");
  requireFile(trackedManifestPath, "E_TRACKED_WASM_MANIFEST_MISSING");
  const trackedManifest = JSON.parse(fs.readFileSync(trackedManifestPath, "utf8"));
  const wasmBytes = fs.readFileSync(wasmPath);
  if (sha256(wasmBytes) !== trackedManifest.wasm_sha256) {
    throw new Error("E_STALE_TRACKED_WASM_MANIFEST");
  }

  const wasm = await import(`${pathToFileURL(jsPath).href}?vc06=${Date.now()}`);
  await wasm.default({ module_or_path: wasmBytes });
  const masked = String(process.env.DDN_VC06_MASK_EXPORT ?? "");
  for (const name of [
    "wasm_run_summary_with_source_profile_json",
    "wasm_canon_ddn_with_source_profile",
  ]) {
    if (masked === name || typeof wasm[name] !== "function") {
      throw new Error(`E_REQUIRED_EXPORT_MISSING:${name}`);
    }
  }

  const rows = [];
  for (const testCase of manifest.cases) {
    const source = readSource(testCase);
    const run = source === null
      ? { outcome: "NO_TEXT_SURFACE", diagnostic: { code: "E_INVALID_UTF8_BEFORE_STRING_API" } }
      : runCase(wasm, source, testCase, manifest.source_profile_identity);
    rows.push({
      id: testCase.id,
      fixture_sha256: testCase.fixture_sha256,
      source_sha256: source === null ? null : sha256(Buffer.from(source, "utf8")),
      run,
      canon: testCase.canon_roundtrip && source !== null
        ? canonCase(wasm, source, manifest.source_profile_identity)
        : null,
      runtime_canon_reparse: testCase.canon_roundtrip && source !== null
        ? runtimeCanonReparse(wasm, run, testCase, manifest.source_profile_identity)
        : null,
    });
  }

  const report = {
    schema: "ddn.vc06.tracked_wasm_probe_report.v1",
    manifest_schema: manifest.schema,
    manifest_sha256: sha256(manifestBytes),
    source_profile_identity: manifest.source_profile_identity,
    tracked_wasm_sha256: sha256(wasmBytes),
    tracked_wasm_manifest_sha256: sha256(fs.readFileSync(trackedManifestPath)),
    cases: rows,
  };
  process.stdout.write(`${JSON.stringify(report)}\n`);
}

main().catch((error) => {
  process.stderr.write(`${String(error)}\n`);
  process.exit(1);
});
