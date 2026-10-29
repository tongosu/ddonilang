#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_local_content_exchange_quality_v1"
RUNNER = ROOT / "tests" / "seamgrim_local_content_exchange_quality_runner.mjs"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def run(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def require_files() -> int:
    required = [
        RUNNER,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_LOCAL_EXCHANGE_QUALITY_MISSING", repr(missing))
    return 0


def check_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_local_content_exchange_quality_v1",
        "gate": "G8",
        "representative_case_count": 5,
        "roundtrip_loss_zero": True,
        "unknown_snapshot_field_preserved": True,
        "edit_hash_inequality": True,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_LOCAL_EXCHANGE_QUALITY_CONTRACT", f"{key}={payload.get(key)!r}")
    return 0


def run_runner() -> int:
    proc = run(["node", str(RUNNER.relative_to(ROOT))])
    if proc.returncode != 0:
        return fail("E_LOCAL_EXCHANGE_QUALITY_RUNNER", proc.stdout.strip())
    if "seamgrim_local_content_exchange_quality: ok" not in proc.stdout:
        return fail("E_LOCAL_EXCHANGE_QUALITY_OK", proc.stdout.strip())
    return 0


def require_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_LOCAL_EXCHANGE_QUALITY_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_LOCAL_EXCHANGE_QUALITY_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (require_files, check_contract, run_runner, require_docs_ssot_clean):
        rc = check()
        if rc:
            return rc
    print("[seamgrim-local-content-exchange-quality-v1] OK roundtrip")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

