#!/usr/bin/env node

import crypto from "node:crypto";
import { existsSync } from "node:fs";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

function sortJson(value) {
  if (Array.isArray(value)) return value.map((item) => sortJson(item));
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.keys(value)
      .sort((a, b) => a.localeCompare(b))
      .map((key) => [key, sortJson(value[key])]),
  );
}

function formatJson(value) {
  return `${JSON.stringify(sortJson(value), null, 2)}\n`;
}

function sha256Text(text) {
  return `sha256:${crypto.createHash("sha256").update(String(text ?? ""), "utf8").digest("hex")}`;
}

function resolveTeulCli(rootDir) {
  const suffix = process.platform === "win32" ? ".exe" : "";
  const candidates = [
    path.join(rootDir, "target", "debug", `teul-cli${suffix}`),
    path.join(rootDir, "target", "release", `teul-cli${suffix}`),
    `I:/home/urihanl/ddn/codex/target/debug/teul-cli${suffix}`,
    `I:/home/urihanl/ddn/codex/target/release/teul-cli${suffix}`,
    `C:/ddn/codex/target/debug/teul-cli${suffix}`,
    `C:/ddn/codex/target/release/teul-cli${suffix}`,
  ];
  return candidates.find((candidate) => {
    try {
      return Boolean(candidate && existsSync(candidate));
    } catch {
      return false;
    }
  }) ?? null;
}

function buildCanonCommand(rootDir, inputPath) {
  const bin = resolveTeulCli(rootDir);
  if (bin) {
    return [bin, "canon", inputPath, "--emit", "ddn"];
  }
  return [
    "cargo",
    "run",
    "--quiet",
    "--manifest-path",
    path.join(rootDir, "tools", "teul-cli", "Cargo.toml"),
    "--",
    "canon",
    inputPath,
    "--emit",
    "ddn",
  ];
}

function runCanonDdn(rootDir, inputPath) {
  const cmd = buildCanonCommand(rootDir, inputPath);
  const proc = spawnSync(cmd[0], cmd.slice(1), {
    cwd: rootDir,
    encoding: "utf8",
    env: process.env,
  });
  if ((proc.status ?? 1) !== 0) {
    const detail = [proc.stderr, proc.stdout].filter(Boolean).join("\n").trim();
    throw new Error(`canon failed: ${cmd.join(" ")}\n${detail}`);
  }
  return String(proc.stdout ?? "");
}

function flattenBlocks(blocks, out = []) {
  for (const block of Array.isArray(blocks) ? blocks : []) {
    out.push(block);
    Object.values(block?.inputs && typeof block.inputs === "object" ? block.inputs : {}).forEach((children) => {
      flattenBlocks(children, out);
    });
  }
  return out;
}

function summarizeFlatJson(value) {
  const nodes = Array.isArray(value?.nodes) ? value.nodes : [];
  const edges = Array.isArray(value?.edges) ? value.edges : [];
  return {
    schema: String(value?.schema ?? ""),
    node_count: nodes.length,
    edge_count: edges.length,
    digest: sha256Text(formatJson(value)),
  };
}

async function main() {
  const args = process.argv.slice(2);
  const update = args.includes("--update");
  const packArg = args.find((item) => !item.startsWith("--")) ?? "pack/block_editor_roundtrip_v1";
  const __dirname = path.dirname(fileURLToPath(import.meta.url));
  const rootDir = path.resolve(__dirname, "..");
  const packDir = path.resolve(rootDir, packArg);

  const codecUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "block_editor", "ddn_block_codec.js"),
  ).href;
  const canonUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "runtime", "wasm_canon_runtime.js"),
  ).href;
  const codecMod = await import(codecUrl);
  const canonMod = await import(canonUrl);

  const sourcePath = path.join(packDir, "fixtures", "source.ddn");
  const sourceText = await fs.readFile(sourcePath, "utf8");
  const wasmBytes = await fs.readFile(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "wasm", "ddonirang_tool_bg.wasm"),
  );
  const canon = await canonMod.createWasmCanon({ cacheBust: 0, initInput: wasmBytes });

  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "ddn-block-roundtrip-"));
  try {
    const canonInputPath = path.join(tempDir, "input.ddn");
    await fs.writeFile(canonInputPath, sourceText, "utf8");
    const canonBefore = runCanonDdn(rootDir, canonInputPath);
    const flatJson = await canon.canonFlatJson(canonBefore);
    const blockPlan = await canon.canonBlockEditorPlan(canonBefore);
    const decoded = codecMod.decodeBlockEditorPlanToBlocks(blockPlan);
    const encodedDdn = codecMod.encodeBlocksToDdn(decoded.blocks);
    const encodedInputPath = path.join(tempDir, "encoded.ddn");
    await fs.writeFile(encodedInputPath, encodedDdn, "utf8");
    const canonAfter = runCanonDdn(rootDir, encodedInputPath);
    const allBlocks = flattenBlocks(decoded.blocks);
    const rawBlocks = allBlocks
      .filter((block) => String(block?.kind ?? "") === "raw")
      .map((block) => ({
        label: String(block?.label ?? ""),
        raw_text: String(block?.rawText ?? "").trim(),
        source: String(block?.source ?? ""),
      }));

    const output = {
      schema: "ddn.block_editor_roundtrip_smoke.v1",
      pack: path.basename(packDir),
      canon_before: canonBefore,
      canon_after: canonAfter,
      canon_before_hash: sha256Text(canonBefore),
      canon_after_hash: sha256Text(canonAfter),
      canon_equal: canonBefore === canonAfter,
      encoded_ddn: String(encodedDdn ?? ""),
      encoded_hash: sha256Text(encodedDdn),
      flat_json: summarizeFlatJson(flatJson),
      block_plan_schema: String(blockPlan?.schema ?? ""),
      top_level_block_kinds: (Array.isArray(decoded.blocks) ? decoded.blocks : []).map((block) =>
        String(block?.kind ?? ""),
      ),
      block_kind_counts: Object.fromEntries(
        [...new Set(allBlocks.map((block) => String(block?.kind ?? "")))]
          .sort((a, b) => a.localeCompare(b))
          .map((kind) => [kind, allBlocks.filter((block) => String(block?.kind ?? "") === kind).length]),
      ),
      raw_block_count: rawBlocks.length,
      raw_blocks: rawBlocks,
      decode_errors: (Array.isArray(decoded.errors) ? decoded.errors : []).map((item) =>
        String(item?.message ?? item),
      ),
    };

    const expectedPath = path.join(packDir, "expected", "block_editor_roundtrip.detjson");
    const actualText = formatJson(output);
    if (update) {
      await fs.mkdir(path.dirname(expectedPath), { recursive: true });
      await fs.writeFile(expectedPath, actualText, "utf8");
      console.log(`block editor roundtrip updated: ${path.relative(rootDir, expectedPath)}`);
      return;
    }
    const expectedText = await fs.readFile(expectedPath, "utf8");
    if (expectedText !== actualText) {
      console.error(`block editor roundtrip mismatch: ${path.relative(rootDir, expectedPath)}`);
      process.exit(1);
    }
    console.log(`block editor roundtrip ok: ${path.relative(rootDir, packDir)}`);
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
}

await main();
