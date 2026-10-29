#!/usr/bin/env python3
"""Validate LA0_PA0_DOCS_CLOSED_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "라-0_파-0_DOCS_CLOSED_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_la0_pa0_docs_closed_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-la0-pa0-docs-closed-reconciliation] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    payload = json.loads(read(path))
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


def check_docs() -> None:
    for path in [
        MATRIX,
        TRACKER,
        MANIFEST,
        REPORT,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        RECONCILIATION,
        ROOT / "pack" / "seamgrim_malblock_0_v1" / "README.md",
        ROOT / "pack" / "seamgrim_malblock_blocky_design_v1" / "README.md",
        ROOT / "pack" / "jojo_case_card_schema_v1" / "README.md",
    ]:
        require_file(path)
    require_tokens(MATRIX, [
        "| 0마루 씨앗마루 | 말블록 설계 확정 | 말블록/짜임판/팔레트/텍스트섬 | proposal | 닫힘-",
        "| 0마루 씨앗마루 | 케이스 카드 표준 | JOJO_CASE_CARD_V1, view_requirements | case schema | 닫힘-",
    ])
    require_tokens(TRACKER, [
        "| 3 | `라-0` | 말블록 초보자 블록형 설계 확정 | 닫힘-",
        "| 10 | `파-0` | JOJO_CASE_CARD_V1 표준 | 닫힘-",
        "라-0_파-0_DOCS_CLOSED_RECONCILIATION_REPORT_20260609.md",
    ])
    require_tokens(MANIFEST, [
        "| `라-0` |",
        "`seamgrim_malblock_blocky_design_v1`",
        "`roadmap_v2_la0_pa0_docs_closed_reconciliation_v1`",
        "| `파-0` |",
        "`jojo_case_card_schema_v1`",
        "`roadmap_v2_la0_pa0_docs_closed_reconciliation_v1`",
    ])
    shared = [
        "LA0_PA0_DOCS_CLOSED_RECONCILIATION_V1",
        "LA0/PA0 docs-closed reconciliation 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 62/90 = 69%",
        "ROADMAP_V2 docs-closed: 4/90 = 4%",
        "ROADMAP_V2 pack evidence 참고값: 64/90 = 71%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_POST_LA0_PA0_FRONTIER_REBASE_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 LA0/PA0 docs-closed reconciliation", "docs-closed progress to `4/90 = 4%`"])


def check_payload() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_la0_pa0_docs_closed_reconciliation_v1",
        "kind": "roadmap_v2_la0_pa0_docs_closed_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "LA0_PA0_DOCS_CLOSED_RECONCILIATION_V1",
        "matrix_closure_claim": True,
        "roadmap_behavior_increment": False,
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 62,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 69,
        "roadmap_v2_docs_closed": 4,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 4,
        "roadmap_v2_pack_evidence_reference_closed": 64,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 71,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "malblock_product_behavior_claim": False,
        "social_runtime_behavior_claim": False,
        "next_item": "ROADMAP_V2_POST_LA0_PA0_FRONTIER_REBASE_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    if contract.get("behavior_closed_coordinates") != []:
        fail("behavior_closed_coordinates must be empty")
    if contract.get("docs_closed_coordinates") != ["라-0", "파-0"]:
        fail("docs_closed_coordinates mismatch")
    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("status") != "docs_closed_only":
        fail("reconciliation status mismatch")
    for coord in ["라-0", "파-0"]:
        row = reconciliation.get("coordinates", {}).get(coord, {})
        if row.get("new_status") != "닫힘-문서":
            fail(f"{coord} new_status mismatch")
        if row.get("behavior_closed") is not False:
            fail(f"{coord} behavior flag must be false")
    for key, value in reconciliation.get("false_claims", {}).items():
        if value is not False:
            fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 63/90",
        '"roadmap_v2_matrix_behavior_closed": 63',
        '"roadmap_behavior_increment": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
        '"malblock_product_behavior_claim": true',
        '"social_runtime_behavior_claim": true',
    ]
    for path in [REPORT, PACK / "README.md", CONTRACT, RECONCILIATION]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_la0_pa0_docs_closed_reconciliation_v1"], timeout=120)
    run([sys.executable, "tests/run_seamgrim_malblock_design_check.py"], timeout=120)
    run([sys.executable, "tests/run_seamgrim_malblock_blocky_design_check.py"], timeout=120)
    run([sys.executable, "tests/run_jojo_case_card_schema_check.py"], timeout=120)
    run([sys.executable, "tests/run_jojo_case_card_schema_check.py", "--dir", "pack/jojo_case_card_schema_v1/valid"], timeout=120)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_payload()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-la0-pa0-docs-closed-reconciliation] OK")


if __name__ == "__main__":
    main()
