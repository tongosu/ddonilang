#!/usr/bin/env python3
"""Validate ROADMAP_V2_POST_TA1_FRONTIER_REBASE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROADMAP_V2_POST_TA1_FRONTIER_REBASE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "POST_TA1_FRONTIER_REBASE_REPORT_20260608.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
PACK = ROOT / "pack" / "roadmap_v2_post_ta1_frontier_rebase_v1"
CONTRACT = PACK / "contract.detjson"
NEXT_FRONTIER = PACK / "next_frontier.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-post-ta1-frontier-rebase] FAIL: {message}", file=sys.stderr)
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
        print(proc.stdout, end="")
        fail(f"command failed: {' '.join(args)}")
    return proc


def check_files() -> None:
    for path in [
        DOC,
        REPORT,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        MATRIX,
        GUIDE,
        TRACKER,
        MANIFEST,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        NEXT_FRONTIER,
        ROOT / "ROADMAP_V2_TA1_PACK_RUNNER_BASIS_MATRIX_RECONCILIATION_V1.md",
        ROOT / "tests" / "run_roadmap_v2_ta1_pack_runner_basis_matrix_reconciliation_check.py",
        ROOT / "docs" / "status" / "roadmap_v2" / "타-2_RECONCILIATION_REPORT_20260608.md",
        ROOT / "tests" / "run_roadmap_v2_ta2_matrix_status_reconciliation_check.py",
    ]:
        require_file(path)


def check_docs() -> None:
    shared = [
        "ROADMAP_V2_POST_TA1_FRONTIER_REBASE_V1",
        "ROADMAP_V2_TA2_GUIDE_STATUS_RECONCILIATION_V1",
        "ROADMAP_V2 post-TA1 frontier rebase 4/4 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 45/90 = 50%",
        "ROADMAP_V2 pack evidence 참고값: 59/90 = 66%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "닫힘-문서",
        "runtime_claim:false",
        "product_code_change:false",
        "product_ui_change:false",
        "matrix_closure_claim:false",
        "roadmap_matrix_increment:false",
        "guide_status_change:false",
        "ci_reimplementation:false",
        "pack_runner_reimplementation:false",
        "ci_aggregation_claim:false",
        "wasm_parity_runner_change:false",
        "docs_ssot_change:false",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared)
    require_tokens(PROJECT_STATUS, ["ROADMAP_V2_POST_TA1_FRONTIER_REBASE_V1", "45/90 = 50%", "59/90 = 66%", "타-2"])
    require_tokens(CHANGELOG, ["ROADMAP_V2 post-TA1 frontier rebase", "ROADMAP_V2_TA2_GUIDE_STATUS_RECONCILIATION_V1"])


def check_payloads() -> None:
    contract = read_json(CONTRACT)
    expected_contract = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_post_ta1_frontier_rebase_v1",
        "kind": "roadmap_v2_post_ta1_frontier_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "ROADMAP_V2_POST_TA1_FRONTIER_REBASE_V1",
        "matrix_closure_claim": False,
        "matrix_closure_tier": "닫힘-문서",
        "roadmap_matrix_increment": False,
        "guide_status_change": False,
        "current_stage": "ROADMAP_V2 post-TA1 frontier rebase",
        "current_stage_closed": 4,
        "current_stage_total": 4,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 45,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 50,
        "roadmap_v2_pack_evidence_reference_closed": 59,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 66,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "selected_next_work": "ROADMAP_V2_TA2_GUIDE_STATUS_RECONCILIATION_V1",
        "selected_coordinate": "타-2",
        "rejected_work": "TOOLCHAIN_CI_REIMPLEMENTATION",
        "ci_reimplementation": False,
        "pack_runner_reimplementation": False,
        "ci_aggregation_claim": False,
        "wasm_parity_runner_change": False,
        "requires_docs_ssot_clean": True,
    }
    for key, expected in expected_contract.items():
        if contract.get(key) != expected:
            fail(f"contract {key}={contract.get(key)!r}, expected {expected!r}")

    frontier = read_json(NEXT_FRONTIER)
    if frontier.get("schema") != "ddn.roadmap_v2.post_ta1_frontier_rebase.v1":
        fail("next_frontier schema mismatch")
    if frontier.get("selected_next_work") != "ROADMAP_V2_TA2_GUIDE_STATUS_RECONCILIATION_V1":
        fail("next_frontier selected_next_work mismatch")
    if frontier.get("selected_coordinate") != "타-2":
        fail("next_frontier selected_coordinate mismatch")
    progress = frontier.get("progress")
    if not isinstance(progress, dict):
        fail("next_frontier progress must be object")
    for key in [
        "current_stage_closed",
        "current_stage_total",
        "current_stage_percent",
        "roadmap_v2_matrix_behavior_closed",
        "roadmap_v2_matrix_behavior_total",
        "roadmap_v2_matrix_behavior_percent",
        "roadmap_v2_pack_evidence_reference_closed",
        "roadmap_v2_pack_evidence_reference_total",
        "roadmap_v2_pack_evidence_reference_percent",
        "studio_local_super_long_closed",
        "studio_local_super_long_total",
        "studio_local_super_long_percent",
    ]:
        if progress.get(key) != expected_contract[key]:
            fail(f"next_frontier progress {key}={progress.get(key)!r}")
    claims = frontier.get("claims")
    if not isinstance(claims, dict):
        fail("next_frontier claims must be object")
    for key in [
        "runtime_claim",
        "product_code_change",
        "product_ui_change",
        "matrix_closure_claim",
        "roadmap_matrix_increment",
        "guide_status_change",
        "ci_reimplementation",
        "pack_runner_reimplementation",
        "ci_aggregation_claim",
        "wasm_parity_runner_change",
        "docs_ssot_change",
    ]:
        if claims.get(key) is not False:
            fail(f"next_frontier claim {key}={claims.get(key)!r}")


def check_ta2_drift_basis() -> None:
    require_tokens(
        MATRIX,
        ["| 2마루 닫힘마루 | CI/golden gate | golden/checker/CI | CI PASS | 닫힘-동작 |"],
    )
    require_tokens(TRACKER, ["| 7 | `타-2` | work item evidence gate 재정렬 | 닫힘-동작 |"])
    require_tokens(
        MANIFEST,
        ["| `타-2` | `toolchain_pack_2_v1`; `roadmap_v2_ta2_matrix_status_reconciliation_v1`; `roadmap_v2_ta2_guide_status_reconciliation_v1` |"],
    )
    guide = read(GUIDE)
    if "#### 타-2" not in guide:
        fail("missing 타-2 guide card")
    if "| 현재 상태 | 진행 |" not in guide and "| 현재 상태 | 닫힘-동작 |" not in guide:
        fail("타-2 guide card should remain 진행 or be reconciled by next work")


def check_forbidden_claims() -> None:
    forbidden = [
        "18/18 = 100%",
        "90/90 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 46/90",
        '"roadmap_v2_matrix_behavior_closed": 46',
        '"roadmap_matrix_increment": true',
        '"matrix_closure_claim": true',
        '"guide_status_change": true',
        '"ci_reimplementation": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
        '"pack_runner_reimplementation": true',
        '"ci_aggregation_claim": true',
        '"wasm_parity_runner_change": true',
    ]
    for path in [DOC, REPORT, PACK / "README.md", CONTRACT, NEXT_FRONTIER]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_post_ta1_frontier_rebase_v1"], timeout=120)
    run([sys.executable, "tests/run_roadmap_v2_ta1_pack_runner_basis_matrix_reconciliation_check.py"], timeout=600)
    run([sys.executable, "tests/run_roadmap_v2_pack_runner_basis_check.py"], timeout=120)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files()
    check_docs()
    check_payloads()
    check_ta2_drift_basis()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-post-ta1-frontier-rebase] OK")


if __name__ == "__main__":
    main()
