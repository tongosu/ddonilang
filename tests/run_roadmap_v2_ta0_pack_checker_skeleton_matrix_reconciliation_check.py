#!/usr/bin/env python3
"""Validate ROADMAP_V2_TA0_PACK_CHECKER_SKELETON_MATRIX_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROADMAP_V2_TA0_PACK_CHECKER_SKELETON_MATRIX_RECONCILIATION_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "타-0_RECONCILIATION_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_ta0_pack_checker_skeleton_matrix_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ta0-reconciliation] FAIL: {message}", file=sys.stderr)
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


def check_files_and_docs() -> None:
    for path in [
        DOC,
        REPORT,
        MATRIX,
        GUIDE,
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
        ROOT / "ROADMAP_V2_POST_GA1_FRONTIER_REBASE_V1.md",
        ROOT / "tests" / "run_roadmap_v2_post_ga1_frontier_rebase_check.py",
        ROOT / "docs" / "status" / "roadmap_v2" / "타-0_REPORT_20260429.md",
        ROOT / "pack" / "toolchain_pack_0_v1" / "contract.detjson",
        ROOT / "pack" / "toolchain_pack_0_v1" / "expected" / "pack_skeleton.detjson",
        ROOT / "tests" / "run_roadmap_v2_pack_skeleton_check.py",
    ]:
        require_file(path)

    shared_tokens = [
        "ROADMAP_V2_TA0_PACK_CHECKER_SKELETON_MATRIX_RECONCILIATION_V1",
        "TA0 pack/checker skeleton matrix reconciliation 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 44/90 = 49%",
        "ROADMAP_V2 pack evidence 참고값: 59/90 = 66%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_POST_TA0_FRONTIER_REBASE_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(PROJECT_STATUS, ["ROADMAP_V2_TA0_PACK_CHECKER_SKELETON_MATRIX_RECONCILIATION_V1", "44/90 = 49%", "59/90 = 66%"])
    require_tokens(CHANGELOG, ["ROADMAP_V2 TA0 pack/checker skeleton matrix reconciliation", "ROADMAP_V2_POST_TA0_FRONTIER_REBASE_V1"])
    require_tokens(
        MATRIX,
        [
            "| 0마루 씨앗마루 | pack/checker skeleton | pack template, schema index | skeleton docs | 닫힘-동작 |",
            "| 14 | 타-0 | pack/checker skeleton | skeleton contract checker | 닫힘-동작 |",
        ],
    )
    require_tokens(
        GUIDE,
        [
            "#### 타-0",
            "| 현재 상태 | 닫힘-동작 |",
            "| pack 후보 | `toolchain_pack_0_v1`; `roadmap_v2_ta0_pack_checker_skeleton_matrix_reconciliation_v1` |",
        ],
    )
    require_tokens(TRACKER, ["| 2 | `타-0` | pack/checker skeleton 공식화 | 닫힘-동작 |"])
    require_tokens(
        MANIFEST,
        [
            "| `타-0` | `toolchain_pack_0_v1`; `roadmap_v2_ta0_pack_checker_skeleton_matrix_reconciliation_v1` |",
            "python tests/run_roadmap_v2_ta0_pack_checker_skeleton_matrix_reconciliation_check.py",
            "닫힘-동작. ROADMAP_V2 pack/checker skeleton",
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
        "roadmap_v2_matrix_behavior_closed": 44,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 49,
        "roadmap_v2_pack_evidence_reference_closed": 59,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 66,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
    }
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_ta0_pack_checker_skeleton_matrix_reconciliation_v1",
        "kind": "roadmap_v2_ta0_pack_checker_skeleton_matrix_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "pack_runner_reimplementation": False,
        "ci_aggregation_claim": False,
        "wasm_parity_runner_change": False,
        "runtime_semantics_change": False,
        "closed_by": "ROADMAP_V2_TA0_PACK_CHECKER_SKELETON_MATRIX_RECONCILIATION_V1",
        "roadmap_coordinate": "타-0",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "current_stage": "TA0 pack/checker skeleton matrix reconciliation",
        "next_item": "ROADMAP_V2_POST_TA0_FRONTIER_REBASE_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    check_payload(CONTRACT)

    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("status") != "behavior_closed":
        fail(f"reconciliation status={reconciliation.get('status')!r}")
    record = reconciliation.get("matrix_status_record", {})
    if record.get("previous_status") != "진행" or record.get("new_status") != "닫힘-동작":
        fail("missing matrix status record")
    check_payload(RECONCILIATION)
    false_claims = reconciliation.get("false_claims", {})
    for key, value in false_claims.items():
        if value is not False:
            fail(f"false claim {key}={value!r}")
    evidence = reconciliation.get("evidence", {})
    if not isinstance(evidence.get("pack_checker_skeleton"), dict):
        fail("missing evidence axis: pack_checker_skeleton")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 45/90",
        '"roadmap_v2_matrix_behavior_closed": 45',
        "ROADMAP_V2 행렬 닫힘-동작: 90/90",
        "Studio-local 초장기 계획: 18/18",
        '"pack_runner_reimplementation": true',
        '"ci_aggregation_claim": true',
        '"wasm_parity_runner_change": true',
        '"runtime_semantics_change": true',
        '"pack_evidence_reference_inflation": true',
        '"studio_local_progress_inflation": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
    ]
    for path in [DOC, REPORT, PACK / "README.md", CONTRACT, RECONCILIATION]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ta0_pack_checker_skeleton_matrix_reconciliation_v1"], timeout=120)
    run([sys.executable, "tests/run_roadmap_v2_post_ga1_frontier_rebase_check.py"], timeout=600)
    run([sys.executable, "tests/run_roadmap_v2_pack_skeleton_check.py"], timeout=120)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ta0-reconciliation] OK")


if __name__ == "__main__":
    main()
