#!/usr/bin/env python3
"""Validate NA4_STDLIB_REGISTRY_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "나-4_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
AUTH_SAVE_RUNNER = ROOT / "tests" / "seamgrim_auth_save_surface_runner.mjs"
PACK = ROOT / "pack" / "roadmap_v2_na4_stdlib_registry_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-na4-stdlib-registry-reconciliation] FAIL: {message}", file=sys.stderr)
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
        AUTH_SAVE_RUNNER,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        RECONCILIATION,
        ROOT / "pack" / "stdlib_1_v1" / "golden.jsonl",
        ROOT / "pack" / "seamgrim_registry_publish_install_shell_v1" / "contract.detjson",
    ]:
        require_file(path)
    require_tokens(MATRIX, ["| 4마루 나눔마루 | 표준가지 registry | 표준가지 설치/버전/문서 | local registry pack | 닫힘-동작 |"])
    require_tokens(TRACKER, ["| 52.7 | `나-4` | 표준가지 registry | 닫힘-동작 |", "나-4_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `나-4` | `stdlib_1_v1`; stdlib catalog; Seamgrim package registry surface; `seamgrim_registry_publish_install_shell_v1`; `roadmap_v2_na4_stdlib_registry_reconciliation_v1` |"])
    require_tokens(AUTH_SAVE_RUNNER, ["studio: {", "localSaveStatus: \"저장 대기\"", "updateShellStatusRail: () => {}"])
    shared = [
        "NA4_STDLIB_REGISTRY_RECONCILIATION_V1",
        "NA4 stdlib registry reconciliation 7/7 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 66/90 = 73%",
        "ROADMAP_V2 docs-closed: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 68/90 = 76%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_NA5_STDLIB_LTS_ASSESSMENT_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 NA4 stdlib registry reconciliation", "66/90 = 73%", "68/90 = 76%"])


def check_payload() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_na4_stdlib_registry_reconciliation_v1",
        "kind": "roadmap_v2_na4_stdlib_registry_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "NA4_STDLIB_REGISTRY_RECONCILIATION_V1",
        "roadmap_coordinate": "나-4",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "current_stage_closed": 7,
        "current_stage_total": 7,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 66,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 73,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 68,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 76,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "next_item": "ROADMAP_V2_NA5_STDLIB_LTS_ASSESSMENT_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("coordinate") != "나-4":
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
        "ROADMAP_V2 행렬 닫힘-동작: 67/90",
        '"roadmap_v2_matrix_behavior_closed": 67',
        '"public_registry_final_claim": true',
        '"network_registry_sync_claim": true',
        '"trust_signing_claim": true',
        '"cloud_install_update_remove_execution_claim": true',
        '"new_stdlib_surface_claim": true',
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
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_na4_stdlib_registry_reconciliation_v1"], timeout=120)
    run([sys.executable, "tests/run_stdlib_catalog_check.py"], timeout=180)
    run([sys.executable, "tests/run_stdlib_catalog_check_selftest.py"], timeout=180)
    run([sys.executable, "tests/run_stdlib_1_check.py"], timeout=240)
    run([sys.executable, "tests/run_seamgrim_stdlib_1_wasm_check.py"], timeout=180)
    run(["node", "tests/seamgrim_auth_save_surface_runner.mjs"], timeout=180)
    run([sys.executable, "tests/run_seamgrim_platform_mock_interface_contract_check.py"], timeout=180)
    run([sys.executable, "tests/run_seamgrim_package_registry_surface_check.py"], timeout=180)
    run([sys.executable, "tests/run_seamgrim_registry_publish_install_shell_pack_check.py"], timeout=180)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_payload()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-na4-stdlib-registry-reconciliation] OK")


if __name__ == "__main__":
    main()
