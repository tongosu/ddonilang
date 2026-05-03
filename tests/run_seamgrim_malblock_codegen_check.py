#!/usr/bin/env python
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_malblock_codegen_v1"


def fail(message: str) -> int:
    print(f"[seamgrim-malblock-codegen] fail: {message}", file=sys.stderr)
    return 1


def sha256_text(text: str) -> str:
    return f"sha256:{hashlib.sha256(text.encode('utf-8')).hexdigest()}"


def sort_json(value):
    if isinstance(value, list):
        return [sort_json(item) for item in value]
    if isinstance(value, dict):
        return {key: sort_json(value[key]) for key in sorted(value)}
    return value


def format_json(value) -> str:
    return json.dumps(sort_json(value), ensure_ascii=False, indent=2) + "\n"


def resolve_teul_cli_prefix() -> list[str]:
    suffix = ".exe" if sys.platform.startswith("win") else ""
    candidates = [
        ROOT / "target" / "debug" / f"teul-cli{suffix}",
        ROOT / "target" / "release" / f"teul-cli{suffix}",
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"I:/home/urihanl/ddn/codex/target/release/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/release/teul-cli{suffix}"),
    ]
    for candidate in candidates:
        if candidate.exists():
            return [str(candidate)]
    return [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        str(ROOT / "tools" / "teul-cli" / "Cargo.toml"),
        "--",
    ]


def run_cmd(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )


def teul_cli_text(args: list[str], *, timeout: int = 180) -> str:
    cmd = [*resolve_teul_cli_prefix(), *args]
    proc = run_cmd(cmd, timeout=timeout)
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip()
        raise RuntimeError(f"{' '.join(cmd)}\n{detail}")
    return proc.stdout


def generate_cases() -> list[dict]:
    script = r"""
const { pathToFileURL } = require("url");
const path = require("path");

async function main() {
  const root = process.cwd();
  const codec = await import(pathToFileURL(path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "block_editor", "ddn_block_codec.js")).href);
  const paletteMod = await import(pathToFileURL(path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "block_editor", "seamgrim_palette.js")).href);
  const palette = paletteMod.SEAMGRIM_PALETTE;

  function def(kind) {
    const hit = paletteMod.findPaletteBlock(palette, kind);
    if (!hit) throw new Error(`palette block missing: ${kind}`);
    return hit.block;
  }
  function block(kind, overrides = {}) {
    return codec.instantiateBlock(def(kind), overrides);
  }
  function flatten(blocks, out = []) {
    for (const item of Array.isArray(blocks) ? blocks : []) {
      out.push(String(item?.kind ?? ""));
      Object.values(item?.inputs && typeof item.inputs === "object" ? item.inputs : {}).forEach((children) => flatten(children, out));
    }
    return out;
  }
  function emit(id, blocks) {
    return {
      id,
      encoded_ddn: codec.encodeBlocksToDdn(blocks),
      block_kinds: flatten(blocks),
      raw_block_count: flatten(blocks).filter((kind) => kind === "raw").length,
    };
  }

  const charimVariableShow = [
    block("charim_block", {
      inputs: {
        items: [
          block("charim_item_var", {
            fields: { name: "점수", type_name: "셈수", value: "72" },
          }),
        ],
      },
    }),
    block("show", { fields: { expr: "점수" } }),
  ];

  const ifElseShow = [
    block("charim_block", {
      inputs: {
        items: [
          block("charim_item_var", {
            fields: { name: "점수", type_name: "셈수", value: "72" },
          }),
          block("charim_item_var", {
            fields: { name: "판정", type_name: "글", value: "\"\"" },
          }),
        ],
      },
    }),
    block("if_else", {
      fields: { cond: "점수 >= 70" },
      inputs: {
        then: [block("assign", { fields: { target: "판정", value: "\"통과\"" } })],
        else: [block("assign", { fields: { target: "판정", value: "\"보충\"" } })],
      },
    }),
    block("show", { fields: { expr: "판정" } }),
  ];

  const chooseExhaustiveShow = [
    block("charim_block", {
      inputs: {
        items: [
          block("charim_item_var", {
            fields: { name: "x", type_name: "셈수", value: "1" },
          }),
        ],
      },
    }),
    block("choose_exhaustive", {
      inputs: {
        branches: [
          block("choose_branch", {
            fields: { cond: "x < 10" },
            inputs: { body: [block("show", { fields: { expr: "\"작음\"" } })] },
          }),
        ],
      },
    }),
  ];

  console.log(JSON.stringify([
    emit("charim_variable_show", charimVariableShow),
    emit("if_else_show", ifElseShow),
    emit("choose_exhaustive_show", chooseExhaustiveShow),
  ]));
}

main().catch((err) => {
  console.error(String((err && err.stack) || err));
  process.exit(1);
});
"""
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
        detail = (proc.stderr or proc.stdout or "").strip()
        raise RuntimeError(detail)
    return json.loads(proc.stdout)


def load_case_contract() -> dict[str, dict]:
    payload = json.loads((PACK / "fixtures" / "cases.detjson").read_text(encoding="utf-8"))
    if payload.get("schema") != "ddn.seamgrim_malblock_codegen_cases.v1":
        raise RuntimeError("cases.detjson schema mismatch")
    return {str(item.get("id")): item for item in payload.get("cases", [])}


def validate_case(case: dict, contract: dict, temp_dir: Path) -> dict:
    case_id = str(case.get("id", "")).strip()
    if not case_id:
        raise RuntimeError("generated case missing id")
    expected = contract.get(case_id)
    if not expected:
        raise RuntimeError(f"unexpected generated case: {case_id}")

    encoded = str(case.get("encoded_ddn", ""))
    block_kinds = [str(kind) for kind in case.get("block_kinds", [])]
    raw_block_count = int(case.get("raw_block_count", -1))
    if raw_block_count != 0:
        raise RuntimeError(f"{case_id}: raw_block_count={raw_block_count}")
    for kind in expected.get("required_block_kinds", []):
        if str(kind) not in block_kinds:
            raise RuntimeError(f"{case_id}: required block kind missing: {kind}")

    source_path = temp_dir / f"{case_id}.ddn"
    source_path.write_text(encoded, encoding="utf-8")
    canon = teul_cli_text(["canon", str(source_path), "--emit", "ddn"])
    run_out = teul_cli_text(["run", str(source_path), "--madi", "1"])
    for needle in expected.get("expected_stdout_contains", []):
        if str(needle) not in run_out:
            raise RuntimeError(f"{case_id}: run stdout missing {needle!r}")

    return {
        "id": case_id,
        "block_kinds": block_kinds,
        "raw_block_count": raw_block_count,
        "encoded_hash": sha256_text(encoded),
        "canon_hash": sha256_text(canon),
        "run_stdout_contains": [str(item) for item in expected.get("expected_stdout_contains", [])],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="ROADMAP_V2 라-1 말블록 codegen evidence checker")
    parser.add_argument("--update", action="store_true", help="expected report 갱신")
    args = parser.parse_args()

    try:
        contract = load_case_contract()
        generated_cases = generate_cases()
        with tempfile.TemporaryDirectory(prefix="ddn-malblock-codegen-") as tmp:
            rows = [validate_case(case, contract, Path(tmp)) for case in generated_cases]
        missing = sorted(set(contract) - {row["id"] for row in rows})
        if missing:
            raise RuntimeError(f"missing generated cases: {', '.join(missing)}")
        report = {
            "schema": "ddn.seamgrim_malblock_codegen_report.v1",
            "cases": rows,
        }
        expected_path = PACK / "expected" / "malblock_codegen.detjson"
        actual_text = format_json(report)
        if args.update:
            expected_path.parent.mkdir(parents=True, exist_ok=True)
            expected_path.write_text(actual_text, encoding="utf-8")
            print(f"[seamgrim-malblock-codegen] updated {expected_path.relative_to(ROOT)}")
            return 0
        expected_text = expected_path.read_text(encoding="utf-8")
        if expected_text != actual_text:
            raise RuntimeError(f"expected mismatch: {expected_path.relative_to(ROOT)}")
    except Exception as exc:
        return fail(str(exc))

    print(f"[seamgrim-malblock-codegen] ok cases={len(rows)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
