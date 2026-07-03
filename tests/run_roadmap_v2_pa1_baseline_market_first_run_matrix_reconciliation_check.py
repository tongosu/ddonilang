#!/usr/bin/env python3
"""Validate ROADMAP_V2_PA1_BASELINE_MARKET_FIRST_RUN_MATRIX_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "파-1_RECONCILIATION_REPORT_20260609.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_pa1_baseline_market_first_run_matrix_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-pa1-reconciliation] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    try:
        payload = json.loads(read(path))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be a JSON object")
    return payload


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def require_tokens(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing {missing}")


def run(args: list[str], *, timeout: float | None = None) -> subprocess.CompletedProcess[str]:
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
        encoding = sys.stdout.encoding or "utf-8"
        safe_stdout = proc.stdout.encode(encoding, errors="replace").decode(encoding, errors="replace")
        print(safe_stdout, end="")
        fail(f"command failed: {' '.join(args)}")
    return proc


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
        ROOT / "docs" / "status" / "roadmap_v2" / "파-1_REPORT_20260508.md",
        ROOT / "pack" / "econ_age4_ready_baseline_tax_ceiling_v1" / "golden.jsonl",
        ROOT / "tests" / "run_jojo_case_card_schema_check.py",
        ROOT / "tests" / "run_econ_age4_ready_baseline_tax_ceiling_check.py",
        ROOT / "tests" / "run_roadmap_v2_post_sa1_frontier_rebase_check.py",
    ]:
        require_file(path)

    shared_tokens = [
        "ROADMAP_V2_PA1_BASELINE_MARKET_FIRST_RUN_MATRIX_RECONCILIATION_V1",
        "PA1 matrix reconciliation 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 55/90 = 61%",
        "ROADMAP_V2 pack evidence 참고값: 59/90 = 66%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_POST_PA1_FRONTIER_REBASE_V1",
    ]
    for path in [REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(PROJECT_STATUS, ["ROADMAP_V2_PA1_BASELINE_MARKET_FIRST_RUN_MATRIX_RECONCILIATION_V1", "55/90 = 61%", "59/90 = 66%", "파-1"])
    require_tokens(CHANGELOG, ["ROADMAP_V2 PA1 baseline market first-run matrix reconciliation", "ROADMAP_V2_POST_PA1_FRONTIER_REBASE_V1"])

    require_tokens(
        MATRIX,
        [
            "| 1마루 첫실행마루 | 기준 시장 3개 첫실행 | 기준 시장/세금/가격상한 | AGE4 case smoke | 닫힘-동작 |",
            "| 4 | 파-1 | 기준 시장/세금/가격상한 3개 첫실행 | econ_*_AGE4_READY packs | 닫힘-동작 |",
        ],
    )
    require_tokens(TRACKER, ["| 11 | `파-1` | 기준 시장/세금/가격상한 3개 첫실행 | 닫힘-동작 |", "파-1_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(
        MANIFEST,
        [
            "| `파-1` | `econ_age4_ready_baseline_tax_ceiling_v1`; `roadmap_v2_pa1_baseline_market_first_run_matrix_reconciliation_v1` |",
            "행렬 정합화: `python tests/run_roadmap_v2_pa1_baseline_market_first_run_matrix_reconciliation_check.py`",
        ],
    )


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
        "roadmap_v2_matrix_behavior_closed": 55,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 61,
        "roadmap_v2_pack_evidence_reference_closed": 59,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 66,
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
        "pack": "roadmap_v2_pa1_baseline_market_first_run_matrix_reconciliation_v1",
        "kind": "roadmap_v2_pa1_baseline_market_first_run_matrix_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "econ_runtime_change": False,
        "econ_policy_advice_claim": False,
        "real_world_prediction_claim": False,
        "agent_simulation_execution_claim": False,
        "real_data_integration_claim": False,
        "closed_by": "ROADMAP_V2_PA1_BASELINE_MARKET_FIRST_RUN_MATRIX_RECONCILIATION_V1",
        "roadmap_coordinate": "파-1",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "current_stage": "PA1 matrix reconciliation",
        "next_item": "ROADMAP_V2_POST_PA1_FRONTIER_REBASE_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    check_payload(CONTRACT)

    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("status") != "behavior_closed":
        fail(f"reconciliation status={reconciliation.get('status')!r}")
    if reconciliation.get("matrix_status_record", {}).get("new_status") != "닫힘-동작":
        fail("missing matrix status record")
    check_payload(RECONCILIATION)
    false_claims = reconciliation.get("false_claims", {})
    for key, value in false_claims.items():
        if value is not False:
            fail(f"false claim {key}={value!r}")
    evidence = reconciliation.get("evidence", {})
    for key in ["case_card_schema", "baseline_market_tax_price_ceiling", "pack_golden", "frontier_selection"]:
        if not isinstance(evidence.get(key), dict):
            fail(f"missing evidence axis: {key}")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 56/90",
        '"roadmap_v2_matrix_behavior_closed": 56',
        "ROADMAP_V2 행렬 닫힘-동작: 90/90",
        "Studio-local 초장기 계획: 18/18",
        '"pack_evidence_reference_inflation": true',
        '"studio_local_progress_inflation": true',
        '"new_econ_runtime": true',
        '"econ_policy_advice_claim": true',
        '"real_world_prediction_claim": true',
        '"agent_simulation_execution_claim": true',
        '"real_data_integration_claim": true',
        '"runtime_surface_change": true',
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
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_pa1_baseline_market_first_run_matrix_reconciliation_v1"], timeout=120)
    run([sys.executable, "tests/run_roadmap_v2_post_sa1_frontier_rebase_check.py"], timeout=480)
    run([sys.executable, "tests/run_jojo_case_card_schema_check.py", "--dir", "pack/econ_age4_ready_baseline_tax_ceiling_v1/case_cards"], timeout=120)
    run([sys.executable, "tests/run_econ_age4_ready_baseline_tax_ceiling_check.py"], timeout=240)
    run([sys.executable, "tests/run_pack_golden.py", "econ_age4_ready_baseline_tax_ceiling_v1"], timeout=120)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-pa1-reconciliation] OK")


if __name__ == "__main__":
    main()
