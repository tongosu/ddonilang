#!/usr/bin/env python3
"""Validate GA4_PACKAGE_GAJI_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "가-4_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_ga4_package_gaji_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ga4-package-gaji-reconciliation] FAIL: {message}", file=sys.stderr)
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
        ROOT / "pack" / "gaji_registry_report_provenance_v1" / "expected" / "registry.verify.report.json",
        ROOT / "pack" / "gaji_registry_report_provenance_v1" / "expected" / "registry.audit.verify.report.json",
    ]:
        require_file(path)
    require_tokens(MATRIX, ["| 4마루 나눔마루 | package/gaji 연결 | gaji metadata / lock / package | registry shell pack | 닫힘-동작 |"])
    require_tokens(TRACKER, ["| 4.7 | `가-4` | package/gaji 연결 | 닫힘-동작 |", "가-4_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `가-4` | `gaji_registry_report_provenance_v1`; `roadmap_v2_ga4_package_gaji_reconciliation_v1`; package registry surface |"])
    shared = [
        "GA4_PACKAGE_GAJI_RECONCILIATION_V1",
        "GA4 package/gaji reconciliation 6/6 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 65/90 = 72%",
        "ROADMAP_V2 docs-closed: 4/90 = 4%",
        "ROADMAP_V2 pack evidence 참고값: 67/90 = 74%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_POST_GA4_FRONTIER_REBASE_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 GA4 package/gaji reconciliation", "65/90 = 72%", "67/90 = 74%"])


def check_payload() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_ga4_package_gaji_reconciliation_v1",
        "kind": "roadmap_v2_ga4_package_gaji_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "GA4_PACKAGE_GAJI_RECONCILIATION_V1",
        "roadmap_coordinate": "가-4",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "current_stage_closed": 6,
        "current_stage_total": 6,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 65,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 72,
        "roadmap_v2_docs_closed": 4,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 4,
        "roadmap_v2_pack_evidence_reference_closed": 67,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 74,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "next_item": "ROADMAP_V2_POST_GA4_FRONTIER_REBASE_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("coordinate") != "가-4":
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
        "ROADMAP_V2 행렬 닫힘-동작: 66/90",
        '"roadmap_v2_matrix_behavior_closed": 66',
        '"public_registry_final_claim": true',
        '"network_publish_claim": true',
        '"install_update_remove_execution_claim": true',
        '"trust_signing_claim": true',
        '"cloud_sync_claim": true',
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
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ga4_package_gaji_reconciliation_v1"], timeout=120)
    run([sys.executable, "tests/run_gaji_registry_pack_check.py"], timeout=180)
    run([sys.executable, "tests/run_gaji_registry_report_provenance_check.py"], timeout=180)
    run([sys.executable, "tests/run_gaji_registry_strict_audit_check.py"], timeout=180)
    run([sys.executable, "tests/run_gaji_registry_defaults_check.py"], timeout=180)
    run([sys.executable, "tests/run_seamgrim_package_registry_surface_check.py"], timeout=180)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_payload()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ga4-package-gaji-reconciliation] OK")


if __name__ == "__main__":
    main()
