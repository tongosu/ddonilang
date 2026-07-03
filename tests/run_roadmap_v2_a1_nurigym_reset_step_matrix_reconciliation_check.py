#!/usr/bin/env python3
"""Validate A1_NURIGYM_RESET_STEP_MATRIX_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "아-1_RECONCILIATION_REPORT_20260609.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_a1_nurigym_reset_step_matrix_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-a1-nurigym-reconciliation] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    payload = json.loads(read(path))
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be JSON object")
    return payload


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def require_tokens(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing {missing}")


def run(args: list[str], *, timeout: float | None = None) -> None:
    proc = subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )
    if proc.returncode != 0:
        print(proc.stdout, end="")
        fail(f"command failed: {' '.join(args)}")


def check_files_and_docs() -> None:
    for path in [
        REPORT,
        MATRIX,
        TRACKER,
        MANIFEST,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        RECONCILIATION,
        ROOT / "docs" / "status" / "roadmap_v2" / "아-1_REPORT_20260604.md",
        ROOT / "tests" / "run_nurigym_dataset_hash_expected_refresh_check.py",
        ROOT / "pack" / "gogae5_w47_nurigym_observation_spec" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_gridmaze_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_cartpole_shared_sync_v1" / "golden.jsonl",
    ]:
        require_file(path)
    shared = [
        "A1_NURIGYM_RESET_STEP_MATRIX_RECONCILIATION_V1",
        "A1 NuriGym reset/step matrix reconciliation 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 58/90 = 64%",
        "ROADMAP_V2 pack evidence 참고값: 60/90 = 67%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_POST_A1_FRONTIER_REBASE_V1",
    ]
    for path in [REPORT, DEV_SUMMARY]:
        require_tokens(path, shared)
    require_tokens(PROJECT_STATUS, ["A1_NURIGYM_RESET_STEP_MATRIX_RECONCILIATION_V1", "58/90 = 64%", "60/90 = 67%", "아-1"])
    require_tokens(CHANGELOG, ["ROADMAP_V2 A1 NuriGym reset/step matrix reconciliation", "ROADMAP_V2_POST_A1_FRONTIER_REBASE_V1"])
    require_tokens(MATRIX, ["| 1마루 첫실행마루 | reset/step 첫실행 | CartPole/GridMaze stub | smoke pack | 닫힘-동작 |"])
    require_tokens(TRACKER, ["| 16.5 | `아-1` | reset/step 첫실행 | 닫힘-동작 |", "아-1_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `아-1` | `gogae5_w47_nurigym_observation_spec`; `nuri_gym_gridmaze_v1`; `nuri_gym_cartpole_shared_sync_v1`; `roadmap_v2_a1_nurigym_reset_step_matrix_reconciliation_v1` |"])


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "work_unit_closed": 5,
        "work_unit_total": 5,
        "work_unit_percent": 100,
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 58,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 64,
        "roadmap_v2_pack_evidence_reference_closed": 60,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 67,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
    }
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}, expected {value!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_a1_nurigym_reset_step_matrix_reconciliation_v1",
        "kind": "roadmap_v2_a1_nurigym_reset_step_matrix_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "nurigym_runtime_change": False,
        "representative_environment_golden_claim": False,
        "python_web_parity_claim": False,
        "dataset_registry_claim": False,
        "training_workflow_claim": False,
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "roadmap_coordinate": "아-1",
        "next_item": "ROADMAP_V2_POST_A1_FRONTIER_REBASE_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    check_payload(CONTRACT)
    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("status") != "behavior_closed":
        fail("reconciliation status mismatch")
    if reconciliation.get("matrix_status_record", {}).get("new_status") != "닫힘-동작":
        fail("matrix status record mismatch")
    for key, value in reconciliation.get("false_claims", {}).items():
        if value is not False:
            fail(f"false claim {key}={value!r}")
    check_payload(RECONCILIATION)


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 59/90",
        '"roadmap_v2_matrix_behavior_closed": 59',
        '"nurigym_runtime_change": true',
        '"representative_environment_golden_claim": true',
        '"python_web_parity_claim": true',
        '"dataset_registry_claim": true',
        '"training_workflow_claim": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
    ]
    for path in [REPORT, PACK / "README.md", CONTRACT, RECONCILIATION]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_a1_nurigym_reset_step_matrix_reconciliation_v1"], timeout=120)
    run([sys.executable, "tests/run_roadmap_v2_post_ha1_frontier_rebase_check.py"], timeout=600)
    run([sys.executable, "tests/run_pack_golden.py", "nuri_gym_gridmaze_v1", "nuri_gym_cartpole_shared_sync_v1", "gogae5_w47_nurigym_observation_spec"], timeout=240)
    run([sys.executable, "tests/run_nurigym_dataset_hash_expected_refresh_check.py"], timeout=240)
    proc = subprocess.run(["git", "status", "--short", "--", "docs/ssot"], cwd=ROOT, text=True, encoding="utf-8", stdout=subprocess.PIPE)
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-a1-nurigym-reconciliation] OK")


if __name__ == "__main__":
    main()
