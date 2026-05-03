#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]

SOURCE = """
채비 {
  x:수 <- 1.
}.

고르기:
  x < 10 인 경우 {
    x 보여주기.
  }
  모든 경우 다룸.
"""


def resolve_teul_cli() -> list[str]:
    return [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        str(ROOT / "tools" / "teul-cli" / "Cargo.toml"),
        "--",
    ]


def canon_text(source: str) -> str:
    with tempfile.TemporaryDirectory(prefix="ddn-choose-exhaustive-") as tmp:
        input_path = Path(tmp) / "input.ddn"
        input_path.write_text(source, encoding="utf-8")
        cmd = [*resolve_teul_cli(), "canon", str(input_path), "--emit", "ddn"]
        proc = subprocess.run(
            cmd,
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=180,
        )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or f"returncode={proc.returncode}"
        raise RuntimeError(detail)
    return proc.stdout


def main() -> int:
    script = r"""
const fs = require("fs");
const path = require("path");
const { pathToFileURL } = require("url");

async function main() {
  const root = process.cwd();
  const codecUrl = pathToFileURL(
    path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "block_editor", "ddn_block_codec.js"),
  ).href;
  const canonUrl = pathToFileURL(
    path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "runtime", "wasm_canon_runtime.js"),
  ).href;
  const codec = await import(codecUrl);
  const canonMod = await import(canonUrl);
  const wasmBytes = fs.readFileSync(path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "wasm", "ddonirang_tool_bg.wasm"));
  const canon = await canonMod.createWasmCanon({ cacheBust: 0, initInput: wasmBytes });
  const plan = await canon.canonBlockEditorPlan(__SOURCE_JSON__);
  const decoded = codec.decodeBlockEditorPlanToBlocks(plan);
  const encoded = codec.encodeBlocksToDdn(decoded.blocks);
  const allBlocks = [];
  function flatten(blocks) {
    for (const block of Array.isArray(blocks) ? blocks : []) {
      allBlocks.push(block);
      Object.values(block?.inputs && typeof block.inputs === "object" ? block.inputs : {}).forEach(flatten);
    }
  }
  flatten(decoded.blocks);
  const rawCount = allBlocks.filter((block) => String(block?.kind ?? "") === "raw").length;
  console.log(JSON.stringify({
    rawCount,
    encoded,
    planSchema: String(plan?.schema ?? ""),
    firstKind: String(decoded.blocks?.[0]?.kind ?? ""),
    blockKinds: allBlocks.map((block) => String(block?.kind ?? "")),
  }));
}

main().catch((err) => {
  console.error(String((err && err.stack) || err));
  process.exit(1);
});
""".replace("__SOURCE_JSON__", json.dumps(SOURCE))
    proc = subprocess.run(
        ["node", "-"],
        cwd=ROOT,
        input=script,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=120,
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or f"returncode={proc.returncode}"
        print(f"[block-editor-choose-exhaustive] fail: {detail}")
        return 1
    payload = json.loads(proc.stdout)
    encoded = str(payload.get("encoded", ""))
    if int(payload.get("rawCount", -1)) != 0:
        print(f"[block-editor-choose-exhaustive] fail: raw_count={payload.get('rawCount')}")
        return 1
    block_kinds = [str(kind) for kind in payload.get("blockKinds", [])]
    if "choose_exhaustive" not in block_kinds:
        print(f"[block-editor-choose-exhaustive] fail: kinds={','.join(block_kinds)}")
        return 1
    if str(payload.get("planSchema", "")) != "ddn.block_editor_plan.v1":
        print(f"[block-editor-choose-exhaustive] fail: schema={payload.get('planSchema')}")
        return 1
    if "모든 경우 다룸." not in encoded:
        print("[block-editor-choose-exhaustive] fail: missing exhaustive marker")
        return 1
    if "아니면:" in encoded:
        print("[block-editor-choose-exhaustive] fail: unexpected else branch")
        return 1
    try:
        before = canon_text(SOURCE)
        after = canon_text(encoded)
    except RuntimeError as exc:
        print(f"[block-editor-choose-exhaustive] fail: canon:{exc}")
        return 1
    if before != after:
        print("[block-editor-choose-exhaustive] fail: canon mismatch")
        return 1
    print("[block-editor-choose-exhaustive] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
