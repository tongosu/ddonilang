#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "pack" / "creator_gaji_round2_durable_readiness_v1" / "r2_evidence.detjson"
WORKBENCH = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js"
STORE = ROOT / "solutions" / "seamgrim_ui_mvp" / "tools" / "project_store.py"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def main() -> int:
    payload = json.loads(EVIDENCE.read_text(encoding="utf-8"))
    if payload.get("schema") != "ddn.creator_gaji_round2_durable_readiness.evidence.v1":
        return fail("E_R2_EVIDENCE_SCHEMA", repr(payload.get("schema")))
    if payload.get("diagnosis", {}).get("environment_contention_only") is not False:
        return fail("E_R2_DIAGNOSIS", "environment-only diagnosis is not allowed by reproduced product evidence")
    if payload.get("diagnosis", {}).get("product_defect_reproduced_at_least_twice") is not True:
        return fail("E_R2_REPRODUCTION", "product defect repetition missing")
    contract = payload.get("product_contract", {})
    for key in [
        "workbench_api_ready",
        "control_disabled_reason_exposed",
        "persistence_generation_monotonic",
        "save_completion_generation_matches_click",
        "view_server_write_rejected",
        "view_local_wasm_run_enabled",
        "view_local_run_server_append_absent",
        "sqlite_context_connection_closed",
        "aggregate_new_owned_temp_residual_gate",
    ]:
        if contract.get(key) is not True:
            return fail("E_R2_PRODUCT_CONTRACT", f"{key}={contract.get(key)!r}")
    repetitions = payload.get("final_repetitions", {})
    for key, count in {
        "persistence_component": 3,
        "capability_component": 3,
        "durable_aggregate": 2,
        "post_aggregate_components": 2,
    }.items():
        rows = repetitions.get(key, [])
        if len(rows) != count or any(row.get("result") != "PASS" for row in rows):
            return fail("E_R2_REPETITIONS", f"{key}={rows!r}")
    residual = payload.get("final_owned_residual", {})
    if {key: residual.get(key) for key in ("processes", "sockets", "temp_dirs")} != {
        "processes": 0,
        "sockets": 0,
        "temp_dirs": 0,
    }:
        return fail("E_R2_RESIDUAL", repr(residual))
    claims = payload.get("claims", {})
    expected_claims = {
        "r2_complete": True,
        "r0_through_r7_complete": False,
        "overall_goal_complete": False,
        "external_creator_pilot_success_count": 0,
        "docs_ssot_change": False,
    }
    for key, expected in expected_claims.items():
        if claims.get(key) != expected:
            return fail("E_R2_CLAIM", f"{key}={claims.get(key)!r}")
    missing = [rel for rel in payload.get("required_paths", []) if not (ROOT / rel).is_file()]
    if missing:
        return fail("E_R2_REQUIRED_PATH", repr(missing))
    workbench = WORKBENCH.read_text(encoding="utf-8")
    store = STORE.read_text(encoding="utf-8")
    for token in ["workbench_api_ready", "completed_generation", "view_write_forbidden", "beginPersistenceOperation"]:
        if token not in workbench:
            return fail("E_R2_WORKBENCH_TOKEN", token)
    if "button.disabled = viewOnly" in workbench:
        return fail("E_R2_VIEW_RUN_DISABLED", "legacy view-only lifecycle disable remains")
    for token in ["ClosingSqliteConnection", "self.close()"]:
        if token not in store:
            return fail("E_R2_STORE_TOKEN", token)
    ssot = subprocess.run(
        ["git", "status", "--short", "--", "docs/ssot"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=60,
    )
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_R2_SSOT_DIRTY", ssot.stdout.strip())
    print("[creator-gaji-round2-durable-readiness-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
