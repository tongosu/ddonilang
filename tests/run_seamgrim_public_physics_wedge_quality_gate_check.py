#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_public_physics_wedge_v1"


def fail(code: str, message: str) -> int:
    encoding = sys.stdout.encoding or "utf-8"
    safe = str(message).encode(encoding, errors="replace").decode(encoding, errors="replace")
    print(f"{code}: {safe}")
    return 1


def run(cmd: list[str], *, timeout: int = 420) -> subprocess.CompletedProcess[str]:
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


def require_contract() -> int:
    if not PACK.is_dir():
        return fail("E_PUBLIC_WEDGE_PACK_MISSING", str(PACK.relative_to(ROOT)))
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_public_physics_wedge_v1",
        "goal": "public_physics_wedge_default_workbench",
        "behavior_closed_claim": True,
        "external_user_success_claim": False,
        "idc_deployment_claim": False,
        "public_registry_claim": False,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
        "docs_ssot_change": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_PUBLIC_WEDGE_CONTRACT_FIELD", f"{key}={payload.get(key)!r}")
    for rel in payload.get("product_paths", []) + payload.get("required_checkers", []) + payload.get("regression_checkers", []):
        if not (ROOT / rel).exists():
            return fail("E_PUBLIC_WEDGE_REQUIRED_FILE", rel)
    if len(payload.get("required_checkers", [])) != 8:
        return fail("E_PUBLIC_WEDGE_REQUIRED_CHECKERS", repr(payload.get("required_checkers")))
    return 0


def run_pack_golden() -> int:
    proc = run([sys.executable, "tests/run_pack_golden.py", "seamgrim_public_physics_wedge_v1"], timeout=180)
    if proc.returncode != 0:
        return fail("E_PUBLIC_WEDGE_PACK_GOLDEN", proc.stdout.strip())
    return 0


def run_nested_checkers() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    commands = [[sys.executable, rel] for rel in payload["required_checkers"] + payload["regression_checkers"]]
    for cmd in commands:
        proc = run(cmd, timeout=900 if "sync_gate" in cmd[-1] else 420)
        if proc.returncode != 0:
            return fail("E_PUBLIC_WEDGE_NESTED_GATE", f"{cmd}: {proc.stdout.strip()}")
    return 0


def require_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_PUBLIC_WEDGE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_PUBLIC_WEDGE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (require_contract, run_pack_golden, run_nested_checkers, require_docs_ssot_clean):
        rc = check()
        if rc:
            return rc
    print("[seamgrim-public-physics-wedge-quality-gate] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
