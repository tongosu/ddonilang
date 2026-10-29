#!/usr/bin/env python3
"""Validate ROADMAP_V2_POST_A1_FRONTIER_REBASE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "POST_A1_FRONTIER_REBASE_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
PACK = ROOT / "pack" / "roadmap_v2_post_a1_frontier_rebase_v1"
CONTRACT = PACK / "contract.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-post-a1-frontier-rebase] FAIL: {message}", file=sys.stderr)
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
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        MATRIX,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        ROOT / "tests" / "run_roadmap_v2_a1_nurigym_reset_step_matrix_reconciliation_check.py",
        ROOT / "tests" / "run_roadmap_v2_a2_final_closure_check.py",
    ]:
        require_file(path)
    shared = [
        "ROADMAP_V2_POST_A1_FRONTIER_REBASE_V1",
        "A2_NURIGYM_REPRESENTATIVE_ENVIRONMENT_MATRIX_RECONCILIATION_V1",
        "ROADMAP_V2 post-A1 frontier rebase 4/4 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 58/90 = 64%",
        "ROADMAP_V2 pack evidence 참고값: 60/90 = 67%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "닫힘-문서",
        "matrix_closure_claim:false",
        "roadmap_matrix_increment:false",
        "a2_matrix_reconciliation_claim:false",
        "docs_ssot_change:false",
    ]
    for path in [REPORT, DEV_SUMMARY]:
        require_tokens(path, shared)
    require_tokens(PROJECT_STATUS, ["ROADMAP_V2_POST_A1_FRONTIER_REBASE_V1", "58/90 = 64%", "60/90 = 67%", "아-2"])
    require_tokens(CHANGELOG, ["ROADMAP_V2 post-A1 frontier rebase", "A2_NURIGYM_REPRESENTATIVE_ENVIRONMENT_MATRIX_RECONCILIATION_V1"])


def check_contract() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_post_a1_frontier_rebase_v1",
        "kind": "roadmap_v2_post_a1_frontier_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "nurigym_runtime_change": False,
        "representative_environment_golden_claim": False,
        "python_web_parity_claim": False,
        "dataset_registry_claim": False,
        "training_workflow_claim": False,
        "matrix_closure_claim": False,
        "roadmap_matrix_increment": False,
        "a2_matrix_reconciliation_claim": False,
        "current_stage_closed": 4,
        "current_stage_total": 4,
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
        "selected_next_work": "A2_NURIGYM_REPRESENTATIVE_ENVIRONMENT_MATRIX_RECONCILIATION_V1",
        "selected_coordinate": "아-2",
        "docs_ssot_change": False,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 59/90",
        '"roadmap_v2_matrix_behavior_closed": 59',
        '"roadmap_matrix_increment": true',
        '"matrix_closure_claim": true',
        '"a2_matrix_reconciliation_claim": true',
        '"nurigym_runtime_change": true',
        '"python_web_parity_claim": true',
        '"dataset_registry_claim": true',
        '"training_workflow_claim": true',
    ]
    for path in [REPORT, PACK / "README.md", CONTRACT]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_post_a1_frontier_rebase_v1"], timeout=120)
    run([sys.executable, "tests/run_roadmap_v2_a1_nurigym_reset_step_matrix_reconciliation_check.py"], timeout=600)
    run([sys.executable, "tests/run_roadmap_v2_a2_final_closure_check.py"], timeout=600)
    proc = subprocess.run(["git", "status", "--short", "--", "docs/ssot"], cwd=ROOT, text=True, encoding="utf-8", stdout=subprocess.PIPE)
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contract()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-post-a1-frontier-rebase] OK")


if __name__ == "__main__":
    main()
