#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from _teul_cli_freshness import build_teul_cli_cmd as shared_build_teul_cli_cmd


def fail(code: str, msg: str) -> int:
    print(f"[w89-pack-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(path: Path) -> dict:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise ValueError(f"missing file: {path}")
    except Exception as exc:
        raise ValueError(f"invalid json: {path} ({exc})")
    if not isinstance(data, dict):
        raise ValueError(f"json root must be object: {path}")
    return data


def teul_cli_candidates(root: Path) -> list[Path]:
    suffix = ".exe" if os.name == "nt" else ""
    return [
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        root / "target" / "debug" / f"teul-cli{suffix}",
    ]


def build_teul_cli_cmd(root: Path, args: list[str]) -> list[str]:
    return shared_build_teul_cli_cmd(
        root,
        args,
        candidates=teul_cli_candidates(root),
        include_which=False,
        manifest_path=root / "tools" / "teul-cli" / "Cargo.toml",
    )


def run_teul_cli(root: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
    env = dict(os.environ)
    env.setdefault("RUST_MIN_STACK", str(64 * 1024 * 1024))
    return subprocess.run(
        build_teul_cli_cmd(root, args),
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
    )


def prefixed(stdout: str, prefix: str) -> str:
    for line in stdout.splitlines():
        line = line.strip()
        if line.startswith(prefix):
            return line[len(prefix):].strip()
    return ""


def main() -> int:
    parser = argparse.ArgumentParser(description="W89 self-evolving-code pack checker")
    parser.add_argument("--pack", default="pack/gogae9_w89_self_evolving_code")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    pack = root / args.pack
    required = [
        pack / "README.md",
        pack / "intent.md",
        pack / "input.ddn",
        pack / "seed.ddn",
        pack / "evolve_spec.json",
        pack / "golden.detjson",
        pack / "golden.jsonl",
    ]
    missing = [str(path).replace("\\", "/") for path in required if not path.exists()]
    if missing:
        return fail("E_W89_PACK_FILE_MISSING", ",".join(missing))

    readme = (pack / "README.md").read_text(encoding="utf-8")
    for token in (
        "Pack ID: `pack/gogae9_w89_self_evolving_code`",
        "teul-cli evolve run|emit",
        "evolve_spec.json",
        "generated.ddn",
        "evolve_meta.json",
    ):
        if token not in readme:
            return fail("E_W89_README_TOKEN_MISSING", token)

    spec = load_json(pack / "evolve_spec.json")
    if spec.get("schema") != "ddn.gogae9.w89.evolve_spec.v1":
        return fail("E_W89_SPEC_SCHEMA", f"schema={spec.get('schema')}")
    mutation_ops = spec.get("mutation_ops")
    if not isinstance(mutation_ops, list) or len(mutation_ops) < 5:
        return fail("E_W89_MUTATION_OP_COUNT", f"mutation_ops={mutation_ops}")
    for op in ("constant_delta", "operator_replace", "statement_insert", "statement_delete", "subtree_swap"):
        if op not in mutation_ops:
            return fail("E_W89_MUTATION_OP_MISSING", op)

    golden = load_json(pack / "golden.detjson")
    if golden.get("schema") != "ddn.gogae9.w89.evolve_report.v1":
        return fail("E_W89_GOLDEN_SCHEMA", f"schema={golden.get('schema')}")
    if not golden.get("overall_pass"):
        return fail("E_W89_GOLDEN_NOT_PASS", "overall_pass must be true")

    with tempfile.TemporaryDirectory(prefix="w89_evolve_check_") as td:
        out_a = Path(td) / "a"
        out_b = Path(td) / "b"
        results: list[tuple[subprocess.CompletedProcess[str], dict]] = []
        for out in (out_a, out_b):
            proc = run_teul_cli(root, ["evolve", "run", "--pack", str(pack), "--seed", "1234", "--out", str(out)])
            if proc.returncode != 0:
                return fail("E_W89_CLI_RUN_FAIL", ((proc.stdout or "") + "\n" + (proc.stderr or "")).strip())
            meta_path = out / "evolve_meta.json"
            generated_path = out / "generated.ddn"
            if not meta_path.exists() or not generated_path.exists():
                return fail("E_W89_OUTPUT_MISSING", f"meta={meta_path.exists()} generated={generated_path.exists()}")
            meta = load_json(meta_path)
            canon = run_teul_cli(root, ["canon", str(generated_path), "--check"])
            if canon.returncode != 0:
                return fail("E_W89_GENERATED_CANON_CHECK_FAIL", ((canon.stdout or "") + "\n" + (canon.stderr or "")).strip())
            results.append((proc, meta))

        meta_a = results[0][1]
        meta_b = results[1][1]
        for key in ("best_program_canon_hash", "final_state_hash", "best_score", "best_value"):
            if meta_a.get(key) != meta_b.get(key):
                return fail("E_W89_NONDETERMINISM", f"{key}: {meta_a.get(key)} != {meta_b.get(key)}")
            if str(golden.get(key, "")).strip() and meta_a.get(key) != golden.get(key):
                return fail("E_W89_GOLDEN_MISMATCH", f"{key}: {meta_a.get(key)} != {golden.get(key)}")
        if prefixed(results[0][0].stdout, "best_program_canon_hash=") != golden.get("best_program_canon_hash"):
            return fail("E_W89_STDOUT_HASH_MISMATCH", results[0][0].stdout.strip())

    pack_golden = run_teul_cli(root, ["canon", str(pack / "input.ddn"), "--check"])
    if pack_golden.returncode != 0:
        return fail("E_W89_INPUT_CANON_CHECK_FAIL", ((pack_golden.stdout or "") + "\n" + (pack_golden.stderr or "")).strip())

    print("[w89-pack-check] ok")
    print(f"best_program_canon_hash={golden.get('best_program_canon_hash')}")
    print(f"final_state_hash={golden.get('final_state_hash')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
