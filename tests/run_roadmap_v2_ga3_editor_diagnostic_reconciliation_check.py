#!/usr/bin/env python3
"""Validate GA3_EDITOR_DIAGNOSTIC_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "가-3_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_ga3_editor_diagnostic_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ga3-editor-diagnostic-reconciliation] FAIL: {message}", file=sys.stderr)
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
        ROOT / "pack" / "lsp_followon_minimum_v1" / "contract.detjson",
        ROOT / "pack" / "studio_diagnostic_fixit_preview_v1" / "contract.detjson",
        ROOT / "pack" / "studio_diagnostic_fixit_integration_v1" / "contract.detjson",
        ROOT / "pack" / "toolchain_pack_3_v1" / "contract.detjson",
    ]:
        require_file(path)
    require_tokens(MATRIX, ["| 3마루 작업실마루 | 편집기 진단 연결 | LSP/fix-it/diag | LSP minimum pack | 닫힘-동작 |"])
    require_tokens(TRACKER, ["| 4.6 | `가-3` | 편집기 진단 연결 | 닫힘-동작 |", "가-3_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `가-3` | `lsp_followon_minimum_v1`; `studio_diagnostic_fixit_preview_v1`; `studio_diagnostic_fixit_integration_v1`; `toolchain_pack_3_v1`; `roadmap_v2_ga3_editor_diagnostic_reconciliation_v1` |"])
    shared = [
        "GA3_EDITOR_DIAGNOSTIC_RECONCILIATION_V1",
        "GA3 editor diagnostic reconciliation 6/6 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 64/90 = 71%",
        "ROADMAP_V2 docs-closed: 4/90 = 4%",
        "ROADMAP_V2 pack evidence 참고값: 66/90 = 73%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_POST_GA3_FRONTIER_REBASE_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 GA3 editor diagnostic reconciliation", "64/90 = 71%", "66/90 = 73%"])


def check_payload() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_ga3_editor_diagnostic_reconciliation_v1",
        "kind": "roadmap_v2_ga3_editor_diagnostic_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "GA3_EDITOR_DIAGNOSTIC_RECONCILIATION_V1",
        "roadmap_coordinate": "가-3",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "current_stage_closed": 6,
        "current_stage_total": 6,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 64,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 71,
        "roadmap_v2_docs_closed": 4,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 4,
        "roadmap_v2_pack_evidence_reference_closed": 66,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 73,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "next_item": "ROADMAP_V2_POST_GA3_FRONTIER_REBASE_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("coordinate") != "가-3":
        fail("coordinate mismatch")
    if reconciliation.get("new_status") != "닫힘-동작":
        fail("new_status mismatch")
    if reconciliation.get("status") != "behavior_closed":
        fail("status mismatch")
    for payload in [contract, reconciliation]:
        for key, value in payload.get("false_claims", {}).items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 65/90",
        '"roadmap_v2_matrix_behavior_closed": 65',
        '"full_lsp_server_claim": true',
        '"lsp_protocol_expansion_claim": true',
        '"auto_apply_claim": true',
        '"file_write_claim": true',
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


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ga3_editor_diagnostic_reconciliation_v1"], timeout=120)
    run([sys.executable, "tests/run_lsp_followon_minimum_pack_check.py"], timeout=180)
    run([sys.executable, "tests/run_studio_diagnostic_fixit_preview_check.py"], timeout=180)
    run([sys.executable, "tests/run_studio_diagnostic_fixit_integration_check.py"], timeout=180)
    run([sys.executable, "tests/run_roadmap_v2_ta3_diagnostic_ui_lsp_check.py"], timeout=600)
    run(
        [
            sys.executable,
            "tests/run_pack_golden.py",
            "lsp_followon_minimum_v1",
            "studio_diagnostic_fixit_preview_v1",
            "studio_diagnostic_fixit_integration_v1",
            "toolchain_pack_3_v1",
        ],
        timeout=240,
    )
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_payload()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ga3-editor-diagnostic-reconciliation] OK")


if __name__ == "__main__":
    main()
