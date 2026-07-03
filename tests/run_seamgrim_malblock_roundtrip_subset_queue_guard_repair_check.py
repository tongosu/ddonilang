#!/usr/bin/env python3
"""Validate SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_QUEUE_GUARD_REPAIR_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_QUEUE_GUARD_REPAIR_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "LA2_QUEUE_GUARD_REPAIR_REPORT_20260608.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "seamgrim_malblock_roundtrip_subset_queue_guard_repair_v1"
CONTRACT = PACK / "contract.detjson"


def fail(message: str) -> None:
    print(f"[seamgrim-malblock-roundtrip-subset-queue-guard-repair] FAIL: {message}", file=sys.stderr)
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


def run(args: list[str], *, timeout: float | None = None) -> str:
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
    return proc.stdout


def check_files() -> None:
    for path in [
        DOC,
        REPORT,
        QUEUE,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        ROOT / "tests" / "run_seamgrim_malblock_roundtrip_subset_check.py",
        ROOT / "tests" / "run_roadmap_v2_la2_final_closure_check.py",
        ROOT / "tests" / "run_roadmap_v2_post_da5_frontier_rebase_check.py",
    ]:
        require_file(path)


def check_docs() -> None:
    shared = [
        "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_QUEUE_GUARD_REPAIR_V1",
        "SEAMGRIM malblock roundtrip subset queue guard repair 4/4 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 50/90 = 56%",
        "ROADMAP_V2 pack evidence 참고값: 59/90 = 66%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "닫힘-문서",
        "runtime_claim:false",
        "product_code_change:false",
        "product_ui_change:false",
        "matrix_closure_claim:false",
        "roadmap_matrix_increment:false",
        "malblock_runtime_change:false",
        "malblock_ui_change:false",
        "queue_guard_repair_claim:true",
        "la2_matrix_reconciliation_claim:false",
        "docs_ssot_change:false",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared)
    require_tokens(PROJECT_STATUS, ["SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_QUEUE_GUARD_REPAIR_V1", "50/90 = 56%", "59/90 = 66%"])
    require_tokens(CHANGELOG, ["SEAMGRIM malblock roundtrip subset queue guard repair", "No automatic next development item is selected."])


def check_contract() -> None:
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_malblock_roundtrip_subset_queue_guard_repair_v1",
        "kind": "seamgrim_malblock_roundtrip_subset_queue_guard_repair",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_QUEUE_GUARD_REPAIR_V1",
        "matrix_closure_claim": False,
        "matrix_closure_tier": "닫힘-문서",
        "roadmap_matrix_increment": False,
        "malblock_runtime_change": False,
        "malblock_ui_change": False,
        "queue_guard_repair_claim": True,
        "la2_matrix_reconciliation_claim": False,
        "current_stage": "SEAMGRIM malblock roundtrip subset queue guard repair",
        "current_stage_closed": 4,
        "current_stage_total": 4,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 50,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 56,
        "roadmap_v2_pack_evidence_reference_closed": 59,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 66,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "queue_required_token": "No automatic next development item is selected.",
        "queue_forbidden_next_item": "1. `ROOT_LOW_RISK_RETIRE_DELETE_V1`",
        "docs_ssot_change": False,
        "requires_docs_ssot_clean": True,
    }
    contract = read_json(CONTRACT)
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")


def check_queue_state() -> None:
    text = read(QUEUE)
    required = [
        "ROADMAP_V2_LA2_FINAL_CLOSURE_V1",
        "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "No automatic next development item is selected.",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        fail(f"queue missing {missing}")
    forbidden = [
        "1. `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1`",
        "1. `ROADMAP_V2_LA2_FINAL_CLOSURE_V1`",
        "1. `ROOT_LOW_RISK_RETIRE_DELETE_V1`",
        "Approval-gated",
    ]
    present = [token for token in forbidden if token in text]
    if present:
        fail(f"queue still contains stale next-item token: {present}")


def check_forbidden_claims() -> None:
    forbidden = [
        "18/18 = 100%",
        "90/90 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 51/90",
        '"roadmap_v2_matrix_behavior_closed": 51',
        '"roadmap_matrix_increment": true',
        '"matrix_closure_claim": true',
        '"malblock_runtime_change": true',
        '"malblock_ui_change": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
        '"la2_matrix_reconciliation_claim": true',
    ]
    for path in [DOC, REPORT, PACK / "README.md", CONTRACT]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "seamgrim_malblock_roundtrip_subset_queue_guard_repair_v1"], timeout=120)
    run([sys.executable, "tests/run_seamgrim_malblock_roundtrip_subset_check.py"], timeout=180)
    run([sys.executable, "tests/run_roadmap_v2_la2_final_closure_check.py"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_post_da5_frontier_rebase_check.py"], timeout=300)
    status = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in status.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{status}")


def main() -> None:
    check_files()
    check_docs()
    check_contract()
    check_queue_state()
    check_forbidden_claims()
    check_gates()
    print("[seamgrim-malblock-roundtrip-subset-queue-guard-repair] OK")


if __name__ == "__main__":
    main()
