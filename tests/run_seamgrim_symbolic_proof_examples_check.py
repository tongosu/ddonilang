#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SAMPLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "samples"
CASES = [
    "12_symbolic_simplify.ddn",
    "13_seum_equivalence_proof.ddn",
    "14_lambda_store_return.ddn",
]


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def run_teul(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["cargo", "run", "-q", "--manifest-path", "tools/teul-cli/Cargo.toml", "--", *args],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
    )


def main() -> int:
    index = json.loads((SAMPLES / "index.json").read_text(encoding="utf-8"))
    ids = {row.get("id") for row in index.get("samples", [])}
    for expected in ["12_symbolic_simplify", "13_seum_equivalence_proof", "14_lambda_store_return"]:
        if expected not in ids:
            return fail("E_SEAMGRIM_SAMPLE_INDEX", expected)
    for case in CASES:
        path = SAMPLES / case
        for command in ["check", "run"]:
            proc = run_teul(command, str(path))
            if proc.returncode != 0:
                return fail("E_SEAMGRIM_SAMPLE_RUN", f"{command} {case}: {proc.stderr.strip()}")
    print("seamgrim symbolic/proof examples check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

