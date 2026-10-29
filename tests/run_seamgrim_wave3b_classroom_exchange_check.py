#!/usr/bin/env python3
"""Gate for Wave3B G5 classroom exchange lesson bundle workflow."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wave3b_classroom_exchange_v1"


def run(cmd: list[str]) -> str:
    proc = subprocess.run(cmd, cwd=ROOT, text=True, encoding="utf-8", errors="replace", capture_output=True)
    if proc.returncode != 0:
        detail = ((proc.stdout or "") + (proc.stderr or "")).strip()
        raise SystemExit(f"E_WAVE3B_G5_COMMAND: {' '.join(cmd)}\n{detail}")
    return (proc.stdout + proc.stderr).strip()


def load_contract() -> dict:
    path = PACK / "contract.detjson"
    if not path.exists():
        raise SystemExit(f"E_WAVE3B_G5_MISSING_CONTRACT: {path}")
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> None:
    contract = load_contract()
    required = {
        "product_behavior_claim": True,
        "classroom_exchange_schema": "seamgrim.classroom_exchange.v0",
        "kind": "lesson_bundle",
        "export_import_roundtrip": True,
        "profile_hash_in_payload_hash": True,
        "tamper_detection": True,
        "oversize_rejection": True,
        "no_runtime_role_toggle": True,
        "docs_ssot_change": False,
        "parser_change": False,
        "state_hash_formula_change": False,
    }
    for key, expected in required.items():
        actual = contract.get(key)
        if actual != expected:
            raise SystemExit(f"E_WAVE3B_G5_CONTRACT_{key}: expected {expected!r}, got {actual!r}")

    runner = ROOT / "tests" / "seamgrim_wave3b_classroom_exchange_runner.mjs"
    for script in [
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_local_share_package.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "browse.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
        runner,
    ]:
        run(["node", "--check", str(script.relative_to(ROOT))])

    run(["python", "tests/run_pack_golden.py", "seamgrim_wave3b_classroom_exchange_v1"])
    output = run(["node", str(runner.relative_to(ROOT))])
    if "seamgrim_wave3b_classroom_exchange: ok" not in output:
        raise SystemExit(f"E_WAVE3B_G5_RUNNER_OUTPUT: {output}")
    print("[seamgrim-wave3b-classroom-exchange] OK behavior-closed")


if __name__ == "__main__":
    main()
