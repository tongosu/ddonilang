#!/usr/bin/env python3
"""Validate LA2_MATRIX_STATUS_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LA2_MATRIX_STATUS_RECONCILIATION_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "라-2_RECONCILIATION_REPORT_20260608.md"
SOURCE_REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "라-2_REPORT_20260604.md"
QUEUE_REPAIR_REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "LA2_QUEUE_GUARD_REPAIR_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_la2_matrix_status_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-la2-reconciliation] FAIL: {message}", file=sys.stderr)
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
        SOURCE_REPORT,
        QUEUE_REPAIR_REPORT,
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
        ROOT / "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1.md",
        ROOT / "ROADMAP_V2_LA2_FINAL_CLOSURE_V1.md",
        ROOT / "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_QUEUE_GUARD_REPAIR_V1.md",
        ROOT / "pack" / "seamgrim_malblock_roundtrip_subset_v1" / "expected" / "malblock_roundtrip_subset.detjson",
        ROOT / "tests" / "run_seamgrim_malblock_roundtrip_subset_check.py",
        ROOT / "tests" / "run_roadmap_v2_la2_final_closure_check.py",
        ROOT / "tests" / "run_seamgrim_malblock_roundtrip_subset_queue_guard_repair_check.py",
    ]:
        require_file(path)
    shared_tokens = [
        "LA2_MATRIX_STATUS_RECONCILIATION_V1",
        "LA2 matrix status reconciliation 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 51/90 = 57%",
        "ROADMAP_V2 pack evidence 참고값: 59/90 = 66%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_POST_LA2_FRONTIER_REBASE_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(PROJECT_STATUS, ["LA2_MATRIX_STATUS_RECONCILIATION_V1", "51/90 = 57%", "59/90 = 66%", "라-2"])
    require_tokens(CHANGELOG, ["LA2 matrix status reconciliation", "ROADMAP_V2_POST_LA2_FRONTIER_REBASE_V1"])
    require_tokens(
        MATRIX,
        [
            "| 2마루 닫힘마루 | subset roundtrip 닫힘 | DDN→block subset, raw/opaque block | roundtrip pack | 닫힘-동작 |",
        ],
    )
    require_tokens(GUIDE, ["#### 라-2 — subset roundtrip 닫힘", "| 현재 상태 | 닫힘-동작 |", "pack 후보 | `seamgrim_malblock_roundtrip_subset_v1`"])
    require_tokens(TRACKER, ["| 7.1 | `라-2` | subset roundtrip 닫힘 | 닫힘-동작 |", "queue guard repair PASS"])
    require_tokens(MANIFEST, ["| `라-2` | `seamgrim_malblock_roundtrip_subset_v1`; `seamgrim_malblock_roundtrip_subset_queue_guard_repair_v1`; `roadmap_v2_la2_matrix_status_reconciliation_v1` |"])


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 51,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 57,
        "roadmap_v2_pack_evidence_reference_closed": 59,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 66,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
    }
    if "work_unit_closed" in progress:
        expected["work_unit_closed"] = 5
        expected["work_unit_total"] = 5
        expected["work_unit_percent"] = 100
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_la2_matrix_status_reconciliation_v1",
        "kind": "roadmap_v2_la2_matrix_status_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "LA2_MATRIX_STATUS_RECONCILIATION_V1",
        "roadmap_coordinate": "라-2",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "malblock_runtime_change": False,
        "malblock_ui_change": False,
        "full_block_editor_ui_integration_claim": False,
        "arbitrary_ddn_grammar_claim": False,
        "condition_hook_roundtrip_claim": False,
        "current_stage": "LA2 matrix status reconciliation",
        "next_item": "ROADMAP_V2_POST_LA2_FRONTIER_REBASE_V1",
        "docs_ssot_change": False,
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


def check_roundtrip_expected() -> None:
    payload = read_json(ROOT / "pack" / "seamgrim_malblock_roundtrip_subset_v1" / "expected" / "malblock_roundtrip_subset.detjson")
    checks = {
        "schema": payload.get("schema") == "ddn.seamgrim_malblock_roundtrip_subset_report.v1",
        "all_canon_equal": payload.get("all_canon_equal") is True,
        "case_count": payload.get("case_count") == 5,
        "supported_case_count": payload.get("supported_case_count") == 4,
        "raw_fallback_case_count": payload.get("raw_fallback_case_count") == 1,
    }
    bad = [key for key, ok in checks.items() if not ok]
    if bad:
        fail(f"roundtrip expected mismatch: {bad}")


def check_forbidden_claims() -> None:
    forbidden = [
        "18/18 = 100%",
        "90/90 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 52/90",
        '"roadmap_v2_matrix_behavior_closed": 52',
        '"malblock_runtime_change": true',
        '"malblock_ui_change": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
        '"full_block_editor_ui_integration_claim": true',
        '"arbitrary_ddn_grammar_claim": true',
        '"condition_hook_roundtrip_claim": true',
    ]
    for path in [DOC, REPORT, PACK / "README.md", CONTRACT, RECONCILIATION]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_la2_matrix_status_reconciliation_v1"], timeout=120)
    run([sys.executable, "tests/run_seamgrim_malblock_roundtrip_subset_check.py"], timeout=180)
    run([sys.executable, "tests/run_roadmap_v2_la2_final_closure_check.py"], timeout=240)
    run([sys.executable, "tests/run_seamgrim_malblock_roundtrip_subset_queue_guard_repair_check.py"], timeout=300)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contracts()
    check_roundtrip_expected()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-la2-reconciliation] OK")


if __name__ == "__main__":
    main()
