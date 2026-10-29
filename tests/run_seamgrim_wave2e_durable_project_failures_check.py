#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wave2e_durable_project_failures_v1"
BASELINE = PACK / "baseline.detjson"
RUNNER = ROOT / "tests" / "seamgrim_wave2e_durable_project_failure_capture_runner.mjs"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def run(cmd: list[str], *, timeout: int = 120) -> subprocess.CompletedProcess[str]:
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


def main() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "golden.jsonl",
        BASELINE,
        RUNNER,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_WAVE2E_DURABLE_FAILURES_MISSING", repr(missing))

    contract = read_json(PACK / "contract.detjson")
    if contract.get("pack") != "seamgrim_wave2e_durable_project_failures_v1":
        return fail("E_WAVE2E_DURABLE_FAILURES_PACK", repr(contract))
    if contract.get("baseline_only") is not True or contract.get("behavior_closed_claim") is not False:
        return fail("E_WAVE2E_DURABLE_FAILURES_CLAIM", repr(contract))
    for key in ("docs_ssot_change", "new_ddn_syntax", "state_hash_formula_change", "server_login_cloud_claim"):
        if contract.get(key) is not False:
            return fail("E_WAVE2E_DURABLE_FAILURES_BOUNDARY", f"{key}={contract.get(key)!r}")

    baseline = read_json(BASELINE)
    if baseline.get("schema") != "seamgrim.wave2e.durable_project_failures.baseline.v1":
        return fail("E_WAVE2E_DURABLE_FAILURES_SCHEMA", repr(baseline.get("schema")))
    if baseline.get("docs_ssot_dirty_at_capture") is not False:
        return fail("E_WAVE2E_DURABLE_FAILURES_SSOT_CAPTURE", repr(baseline.get("docs_ssot_dirty_at_capture")))
    expected_checks = {
        "wave2d_first_view_authoring": "PASS",
        "platform_server_adapter_contract": "FAIL_existing_appState_studio_harness",
        "platform_server_action_rail": "PASS",
        "studio_local_package_export_action_v1": "PASS",
    }
    if baseline.get("baseline_checks") != expected_checks:
        return fail("E_WAVE2E_DURABLE_FAILURES_BASELINE_CHECKS", repr(baseline.get("baseline_checks")))
    vectors = baseline.get("failure_vectors") or {}
    for key in (
        "server_save_share_not_ready",
        "capability_leak_guard_missing",
        "stale_editor_save_guard_missing",
        "conflict_overwrite_guard_missing",
        "view_escalation_guard_missing",
        "restart_persistence_missing",
        "backup_restore_missing",
    ):
        if vectors.get(key) is not True:
            return fail("E_WAVE2E_DURABLE_FAILURES_VECTOR", key)
    buttons = (baseline.get("browser_capture") or {}).get("durable_project_buttons") or []
    if len(buttons) != 4:
        return fail("E_WAVE2E_DURABLE_FAILURES_BUTTON_COUNT", repr(buttons))
    for button in buttons:
        if button.get("disabled") is not True or "준비 중" not in str(button.get("text", "")):
            return fail("E_WAVE2E_DURABLE_FAILURES_BUTTON_STATE", repr(button))
    if (baseline.get("browser_capture") or {}).get("has_runtime_role_switch") is not False:
        return fail("E_WAVE2E_DURABLE_FAILURES_ROLE_SWITCH", repr(baseline.get("browser_capture")))

    proc = run([sys.executable, "tests/run_pack_golden.py", "seamgrim_wave2e_durable_project_failures_v1"], timeout=180)
    if proc.returncode != 0:
        return fail("E_WAVE2E_DURABLE_FAILURES_GOLDEN", proc.stdout.strip())
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode != 0 or ssot.stdout.strip():
        return fail("E_WAVE2E_DURABLE_FAILURES_SSOT_DIRTY", ssot.stdout.strip())
    print("[seamgrim-wave2e-durable-project-failures-v1] OK baseline-recorded")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
