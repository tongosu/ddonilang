#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_approved_workbench_quality_gate_v1"
RUNNER = ROOT / "tests" / "seamgrim_approved_workbench_quality_gate_runner.mjs"


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
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "index.html",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.css",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_QUALITY_MISSING", str(missing))
    return 0


def require_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_approved_workbench_quality_gate_v1",
        "product_behavior_change": False,
        "behavior_closed_claim": True,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
        "public_registry_claim": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_QUALITY_CONTRACT", f"{key}={payload.get(key)!r}")
    if payload.get("viewports") != ["1440x900", "1024x768", "778x912", "390x844"]:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_QUALITY_VIEWPORTS", repr(payload.get("viewports")))
    checks = payload.get("checks")
    if not isinstance(checks, list):
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_QUALITY_CHECKS", repr(checks))
    for required in [
        "ddn_source_accessible",
        "primary_canvas_nonblank",
        "side_open_close_refit",
        "keyboard_focus",
        "console_error_warning_zero",
    ]:
        if required not in checks:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_QUALITY_CHECK_MISSING", required)
    return 0


def run_browser() -> int:
    proc = run(["node", str(RUNNER.relative_to(ROOT))], timeout=240)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_QUALITY_BROWSER", proc.stdout.strip())
    if "seamgrim_approved_workbench_quality_gate: ok" not in proc.stdout:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_QUALITY_BROWSER_OK", proc.stdout.strip())
    return 0


def run_pack_golden() -> int:
    proc = run(["python", "tests/run_pack_golden.py", "seamgrim_approved_workbench_quality_gate_v1"], timeout=120)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_QUALITY_PACK", proc.stdout.strip())
    return 0


def require_docs_ssot_clean() -> int:
    proc = run(["git", "-c", "core.excludesfile=", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_QUALITY_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_QUALITY_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (
        require_files,
        require_contract,
        run_browser,
        run_pack_golden,
        require_docs_ssot_clean,
    ):
        rc = check()
        if rc:
            return rc
    print("[seamgrim-approved-workbench-quality-gate-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
