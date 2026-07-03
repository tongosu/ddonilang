#!/usr/bin/env python3
"""Validate GA5_GRAMMAR_LTS_DOCS_CLOSED_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "가-5_DOCS_CLOSED_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_ga5_grammar_lts_docs_closed_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ga5-grammar-lts-docs-closed-reconciliation] FAIL: {message}", file=sys.stderr)
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
        ROOT / "pack" / "language_design_priority_rebase_v1" / "contract.detjson",
        ROOT / "pack" / "lang_language_risk_removal_closure_rebase_v1" / "contract.detjson",
        ROOT / "pack" / "gogae9_w98_release_gate" / "README.md",
        ROOT / "pack" / "seamgrim_lesson_loader_basics" / "c01_load" / "lesson_alpha.ddn",
        ROOT / "pack" / "seamgrim_lesson_loader_basics" / "c01_load" / "lesson_beta.ddn",
    ]:
        require_file(path)
    require_tokens(MATRIX, ["| 5마루 단단마루 | LTS 문법선 | breaking-change ledger / compat guide | release gate | 닫힘-문서 |"])
    require_tokens(TRACKER, ["| 4.8 | `가-5` | LTS 문법선 | 닫힘-문서 |", "가-5_DOCS_CLOSED_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(
        MANIFEST,
        [
            "| `가-5` |",
            "`language_design_priority_rebase_v1`",
            "`lang_language_risk_removal_closure_rebase_v1`",
            "grammar manifest operation",
            "formula compatibility gates",
            "`roadmap_v2_ga5_grammar_lts_docs_closed_reconciliation_v1`",
        ],
    )
    shared = [
        "GA5_GRAMMAR_LTS_DOCS_CLOSED_RECONCILIATION_V1",
        "GA5 grammar LTS docs-closed reconciliation 6/6 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 65/90 = 72%",
        "ROADMAP_V2 docs-closed: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 67/90 = 74%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_POST_GA5_FRONTIER_REBASE_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 GA5 grammar LTS docs-closed reconciliation", "65/90 = 72%", "5/90 = 6%"])
    require_tokens(ROOT / "pack" / "gogae9_w98_release_gate" / "README.md", ["draft", "TODO"])


def check_payload() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_ga5_grammar_lts_docs_closed_reconciliation_v1",
        "kind": "roadmap_v2_ga5_grammar_lts_docs_closed_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "GA5_GRAMMAR_LTS_DOCS_CLOSED_RECONCILIATION_V1",
        "roadmap_coordinate": "가-5",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-문서",
        "roadmap_matrix_increment": False,
        "docs_closed_increment": True,
        "current_stage_closed": 6,
        "current_stage_total": 6,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 65,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 72,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 67,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 74,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "next_item": "ROADMAP_V2_POST_GA5_FRONTIER_REBASE_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("coordinate") != "가-5":
        fail("coordinate mismatch")
    if reconciliation.get("new_status") != "닫힘-문서":
        fail("new_status mismatch")
    if reconciliation.get("status") != "docs_closed":
        fail("status mismatch")
    for payload in [contract, reconciliation]:
        for key, value in payload.get("false_claims", {}).items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 66/90",
        '"roadmap_v2_matrix_behavior_closed": 66',
        '"roadmap_matrix_increment": true',
        '"release_gate_execution_claim": true',
        '"full_lts_certification_claim": true',
        '"public_release_claim": true',
        '"parser_runtime_change_claim": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
    ]
    for path in [REPORT, PACK / "README.md", CONTRACT, RECONCILIATION]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")
    release_gate_golden = ROOT / "pack" / "gogae9_w98_release_gate" / "golden.jsonl"
    if release_gate_golden.exists():
        fail("gogae9_w98_release_gate unexpectedly has golden.jsonl; reassess GA5 status")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ga5_grammar_lts_docs_closed_reconciliation_v1"], timeout=120)
    run([sys.executable, "tests/run_language_design_priority_rebase_check.py"], timeout=180)
    run([sys.executable, "tests/run_lang_language_risk_removal_closure_rebase_check.py"], timeout=900)
    run([sys.executable, "tests/run_grammar_manifest_operation_check.py"], timeout=180)
    run([sys.executable, "tests/run_seamgrim_formula_compat_check.py"], timeout=180)
    run([sys.executable, "tests/run_seamgrim_lesson_schema_upgrade_formula_compat_check.py"], timeout=180)
    run([sys.executable, "tests/run_seamgrim_lesson_schema_realign_formula_compat_check.py"], timeout=180)
    run([sys.executable, "tests/run_seamgrim_new_grammar_no_legacy_control_meta_check.py"], timeout=180)
    run(["node", "tests/seamgrim_lesson_loader_runner.mjs"], timeout=180)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_payload()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ga5-grammar-lts-docs-closed-reconciliation] OK")


if __name__ == "__main__":
    main()
