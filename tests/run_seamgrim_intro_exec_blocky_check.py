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
PACK = ROOT / "pack" / "seamgrim_intro_exec_blocky_v1"


def fail(message: str) -> int:
    print(f"[seamgrim-intro-exec-blocky] fail: {message}", file=sys.stderr)
    return 1


def sort_json(value):
    if isinstance(value, list):
        return [sort_json(item) for item in value]
    if isinstance(value, dict):
        return {key: sort_json(value[key]) for key in sorted(value)}
    return value


def format_json(value) -> str:
    return json.dumps(sort_json(value), ensure_ascii=False, indent=2) + "\n"


def sha256_text(text: str) -> str:
    return f"sha256:{hashlib.sha256(text.encode('utf-8')).hexdigest()}"


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


def run_cmd(cmd: list[str], *, timeout: int = 240) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )


def teul_cli_text(args: list[str], *, timeout: int = 240) -> str:
    cmd = [*resolve_teul_cli_prefix(), *args]
    proc = run_cmd(cmd, timeout=timeout)
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip()
        raise RuntimeError(f"{' '.join(cmd)}\n{detail}")
    return proc.stdout


def normalize_stdout(text: str) -> list[str]:
    return [
        line.strip()
        for line in str(text).splitlines()
        if line.strip() and not line.strip().startswith("state_hash=") and not line.strip().startswith("trace_hash=")
    ]


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
    const kinds = flatten(blocks);
    return {
      id,
      encoded_ddn: codec.encodeBlocksToDdn(blocks),
      block_kinds: kinds,
      raw_block_count: kinds.filter((kind) => kind === "raw").length,
    };
  }

  const cases = [
    emit("c01_hello_show", [
      block("charim_block", {
        inputs: { items: [block("charim_item_plain", { fields: { name: "인사", type_name: "글", value: "\"hello\"" } })] },
      }),
      block("show", { fields: { expr: "인사" } }),
    ]),
    emit("c02_assign_update", [
      block("charim_block", {
        inputs: { items: [block("charim_item_var", { fields: { name: "점수", type_name: "셈수", value: "0" } })] },
      }),
      block("assign", { fields: { target: "점수", value: "3" } }),
      block("assign", { fields: { target: "점수", value: "점수 + 7" } }),
      block("assign", { fields: { target: "점수", value: "점수 + 5" } }),
      block("show", { fields: { expr: "점수" } }),
    ]),
    emit("c03_if_else", [
      block("charim_block", {
        inputs: {
          items: [
            block("charim_item_var", { fields: { name: "점수", type_name: "셈수", value: "72" } }),
            block("charim_item_var", { fields: { name: "판정", type_name: "글", value: "\"\"" } }),
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
    ]),
    emit("c04_choose_exhaustive", [
      block("charim_block", {
        inputs: { items: [block("charim_item_var", { fields: { name: "점수", type_name: "셈수", value: "72" } })] },
      }),
      block("choose_exhaustive", {
        inputs: {
          branches: [
            block("choose_branch", {
              fields: { cond: "점수 >= 70" },
              inputs: { body: [block("show", { fields: { expr: "\"통과\"" } })] },
            }),
            block("choose_branch", {
              fields: { cond: "점수 < 70" },
              inputs: { body: [block("show", { fields: { expr: "\"보충\"" } })] },
            }),
          ],
        },
      }),
    ]),
    emit("c05_intro_combined", [
      block("charim_block", {
        inputs: {
          items: [
            block("charim_item_var", { fields: { name: "점수", type_name: "셈수", value: "72" } }),
            block("charim_item_var", { fields: { name: "판정", type_name: "글", value: "\"\"" } }),
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
      block("choose_exhaustive", {
        inputs: {
          branches: [
            block("choose_branch", {
              fields: { cond: "판정 == \"통과\"" },
              inputs: { body: [block("show", { fields: { expr: "판정" } })] },
            }),
            block("choose_branch", {
              fields: { cond: "판정 == \"보충\"" },
              inputs: { body: [block("show", { fields: { expr: "판정" } })] },
            }),
          ],
        },
      }),
    ]),
  ];
  console.log(JSON.stringify(cases));
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


def load_contract() -> dict:
    payload = json.loads((PACK / "fixtures" / "cases.detjson").read_text(encoding="utf-8"))
    if payload.get("schema") != "ddn.seamgrim_intro_exec_blocky_cases.v1":
        raise RuntimeError("cases.detjson schema mismatch")
    return payload


def validate_case(case: dict, expected: dict, temp_dir: Path) -> dict:
    case_id = str(case.get("id", "")).strip()
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
    stdout = normalize_stdout(run_out)
    expected_stdout = [str(item) for item in expected.get("expected_stdout", [])]
    if stdout != expected_stdout:
        raise RuntimeError(f"{case_id}: stdout mismatch expected={expected_stdout!r} got={stdout!r}")
    return {
        "id": case_id,
        "block_kinds": block_kinds,
        "raw_block_count": raw_block_count,
        "encoded_hash": sha256_text(encoded),
        "canon_hash": sha256_text(canon),
        "stdout": stdout,
    }


def run_generated_wasm_parity(generated_cases: list[dict], contract_by_id: dict[str, dict], temp_root: Path) -> dict:
    temp_pack = temp_root / "generated_wasm_pack"
    temp_pack.mkdir(parents=True, exist_ok=True)
    cases_dir = temp_pack / "cases"
    cases_dir.mkdir()
    parity_cases = []
    for case in generated_cases:
        case_id = str(case.get("id"))
        case_dir = cases_dir / case_id
        case_dir.mkdir()
        (case_dir / "input.ddn").write_text(str(case.get("encoded_ddn", "")), encoding="utf-8")
        expected_stdout = contract_by_id[case_id].get("expected_stdout", [])
        parity_cases.append(
            {
                "id": case_id,
                "input": f"cases/{case_id}/input.ddn",
                "ticks": 1,
                "expected_cli_stdout": expected_stdout,
            }
        )
    (temp_pack / "contract.detjson").write_text(
        json.dumps(
            {
                "schema": "ddn.seamgrim.wasm_cli_runtime_parity.pack.contract.v1",
                "pack_id": "seamgrim_intro_exec_blocky_generated",
                "state_hash_policy": "report_only",
                "cases": parity_cases,
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    proc = subprocess.run(
        [
            "node",
            "--no-warnings",
            "tests/seamgrim_wasm_cli_runtime_parity_runner.mjs",
            str(temp_pack),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=240,
    )
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip()
        raise RuntimeError(f"generated wasm parity failed: {detail}")
    report = json.loads(proc.stdout)
    if not report.get("ok"):
        raise RuntimeError("generated wasm parity ok=false")
    return {
        "pack_id": report.get("pack_id"),
        "case_count": len(report.get("cases", [])),
        "ok": bool(report.get("ok")),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="ROADMAP_V2 라-1 intro blocky rail checker")
    parser.add_argument("--update", action="store_true", help="expected report 갱신")
    args = parser.parse_args()

    try:
        contract = load_contract()
        required_palette_kinds = [str(item) for item in contract.get("required_palette_kinds", [])]
        cases_contract = {str(item.get("id")): item for item in contract.get("cases", [])}
        generated_cases = generate_cases()
        generated_ids = {str(item.get("id")) for item in generated_cases}
        if generated_ids != set(cases_contract):
            raise RuntimeError(f"generated case id mismatch: {sorted(generated_ids)}")
        all_kinds = {str(kind) for case in generated_cases for kind in case.get("block_kinds", [])}
        for kind in required_palette_kinds:
            if kind not in all_kinds:
                raise RuntimeError(f"required palette kind unused: {kind}")

        with tempfile.TemporaryDirectory(prefix="ddn-intro-blocky-") as tmp:
            temp_dir = Path(tmp)
            rows = [validate_case(case, cases_contract[str(case.get("id"))], temp_dir) for case in generated_cases]
            parity = run_generated_wasm_parity(generated_cases, cases_contract, temp_dir)
        report = {
            "schema": "ddn.seamgrim_intro_exec_blocky_report.v1",
            "case_count": len(rows),
            "cases": rows,
            "raw_block_count_total": sum(int(row["raw_block_count"]) for row in rows),
            "required_palette_kinds": required_palette_kinds,
            "wasm_parity": parity,
        }
        expected_path = PACK / "expected" / "intro_exec_blocky.detjson"
        actual_text = format_json(report)
        if args.update:
            expected_path.parent.mkdir(parents=True, exist_ok=True)
            expected_path.write_text(actual_text, encoding="utf-8")
            print(f"[seamgrim-intro-exec-blocky] updated {expected_path.relative_to(ROOT)}")
            return 0
        expected_text = expected_path.read_text(encoding="utf-8")
        if expected_text != actual_text:
            raise RuntimeError(f"expected mismatch: {expected_path.relative_to(ROOT)}")
    except Exception as exc:
        return fail(str(exc))

    print(f"[seamgrim-intro-exec-blocky] ok cases={len(rows)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
