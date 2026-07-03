#!/usr/bin/env python3
"""Validate STD_RESOURCE_NETWORK_POLICY_MINIMUM_V1 evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STD_RESOURCE_NETWORK_POLICY_MINIMUM_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "나-3_MINIMUM_REPORT_20260608.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
STDLIB_MATRIX = ROOT / "docs" / "status" / "STDLIB_IMPL_MATRIX.md"
PACK = ROOT / "pack" / "std_resource_network_policy_minimum_v1"
CONTRACT = PACK / "contract.detjson"
NA3_RECONCILIATION_DOC = ROOT / "ROADMAP_V2_NA3_MATRIX_STATUS_RECONCILIATION_V1.md"

SUPPORT_PACKS = [
    "stdlib_missing_coverage_io_v1",
    "eco_network_flow_smoke",
    "social_world_econ_2_v1",
    "social_world_econ_3_v1",
]

EXPECTED_STDOUT = [
    "STD_RESOURCE_NETWORK_POLICY_MINIMUM_V1",
    "std resource network policy minimum sealed",
    "자원#fade0e6558ad1c1c",
    "resource_gate: 자원 handle deterministic",
    "network_gate: eco_network_flow_smoke PASS",
    "policy_gate: social_world_econ_2_v1/social_world_econ_3_v1 PASS",
    "drift_excluded: seamgrim_exec_policy_effect_map_v1/open_decl_policy/open_deny_policy",
    "stage: 5/5 = 100%",
    "roadmap matrix behavior: 38/90 = 42%",
    "pack evidence reference: 59/90 = 66%",
    "studio local super-long: 9/18 = 50%",
    "status: pack-evidence only, not na3 matrix-closed",
    "next: ROADMAP_V2_NA3_MATRIX_STATUS_RECONCILIATION_V1",
]


def fail(message: str) -> None:
    print(f"[std-resource-network-policy-minimum] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def require_tokens(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing {missing}")


def na3_reconciled() -> bool:
    return NA3_RECONCILIATION_DOC.is_file()


def read_json(path: Path) -> dict:
    try:
        payload = json.loads(read(path))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be a JSON object")
    return payload


def read_jsonl(path: Path) -> list[dict]:
    rows: list[dict] = []
    for lineno, line in enumerate(read(path).splitlines(), start=1):
        if not line.strip():
            continue
        try:
            row = json.loads(line)
        except json.JSONDecodeError as exc:
            fail(f"{path.relative_to(ROOT)}:{lineno}: invalid JSONL: {exc}")
        if not isinstance(row, dict):
            fail(f"{path.relative_to(ROOT)}:{lineno}: row must be object")
        rows.append(row)
    return rows


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
        GUIDE,
        MATRIX,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        STDLIB_MATRIX,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        ROOT / "pack" / "age1_container_resource" / "input.ddn",
        ROOT / "pack" / "age1_container_resource" / "expect" / "state_hash.txt",
    ]:
        require_file(path)
    for pack in SUPPORT_PACKS:
        require_file(ROOT / "pack" / pack / "golden.jsonl")


def check_docs() -> None:
    shared = [
        "STD_RESOURCE_NETWORK_POLICY_MINIMUM_V1",
        "STD resource/network/policy minimum 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 38/90 = 42%",
        "ROADMAP_V2 pack evidence 참고값: 59/90 = 66%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "matrix_closure_claim:false",
        "roadmap_matrix_increment:false",
        "policy_advice:false",
        "live_policy_deployment:false",
        "docs_ssot_change:false",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared)
    require_tokens(PROJECT_STATUS, ["STD_RESOURCE_NETWORK_POLICY_MINIMUM_V1", "59/90 = 66%"])
    require_tokens(CHANGELOG, ["STD resource/network/policy minimum", "ROADMAP_V2_NA3_MATRIX_STATUS_RECONCILIATION_V1"])
    require_tokens(GUIDE, ["#### 나-3", "| pack 후보 | `std_resource_network_policy_minimum_v1`"])
    guide_text = read(GUIDE)
    allowed_status = ["| 현재 상태 | 진행 |"]
    if na3_reconciled():
        allowed_status.append("| 현재 상태 | 닫힘-동작 |")
    if not any(token in guide_text for token in allowed_status):
        fail("guide status must be 진행, or 닫힘-동작 after NA3 reconciliation")
    require_tokens(STDLIB_MATRIX, ["std_resource_network_policy_minimum_v1", "network_flow", "policy_ghost"])


def check_matrix_not_closed() -> None:
    for line in read(MATRIX).splitlines():
        if "| 3마루 작업실마루 | std_agent/resource/network/policy |" in line:
            if "닫힘-동작" in line and not na3_reconciled():
                fail("나-3 matrix row must not be behavior-closed by this minimum pack")
            return
    fail("missing 나-3 matrix row")


def check_contract() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.std_resource_network_policy_minimum.pack.contract.v1",
        "pack_id": "std_resource_network_policy_minimum_v1",
        "roadmap_coordinate": "나-3",
        "evidence_tier": "pack_evidence_only",
        "matrix_closure_claim": False,
        "roadmap_matrix_increment": False,
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "public_surface_added": False,
        "current_stage": "STD resource/network/policy minimum",
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 38,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 42,
        "roadmap_v2_pack_evidence_reference_closed": 59,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 66,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "next_item": "ROADMAP_V2_NA3_MATRIX_STATUS_RECONCILIATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    axes = contract.get("axes", {})
    for axis in ["resource", "network", "policy"]:
        if not isinstance(axes.get(axis), dict):
            fail(f"missing axis contract: {axis}")
    drift = set(contract.get("excluded_drift_gates") or [])
    for pack in ["seamgrim_exec_policy_effect_map_v1", "open_decl_policy", "open_deny_policy"]:
        if pack not in drift:
            fail(f"excluded drift gate missing: {pack}")
    for key, value in contract.get("false_claims", {}).items():
        if value is not False:
            fail(f"false claim {key}={value!r}")


def check_golden() -> None:
    rows = read_jsonl(PACK / "golden.jsonl")
    if len(rows) != 1:
        fail("minimum pack must have exactly one golden row")
    if rows[0].get("stdout") != EXPECTED_STDOUT:
        fail("minimum pack stdout mismatch")
    source = read(PACK / "input.ddn")
    for token in ["자원", "eco_network_flow_smoke", "social_world_econ_3_v1", "ROADMAP_V2_NA3_MATRIX_STATUS_RECONCILIATION_V1"]:
        if token not in source:
            fail(f"minimum input missing token: {token}")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 39/90",
        '"roadmap_v2_matrix_behavior_closed": 39',
        '"matrix_closure_claim": true',
        '"roadmap_matrix_increment": true',
        '"runtime_claim": true',
        '"product_code_change": true',
        '"product_ui_change": true',
        '"policy_advice": true',
        '"live_policy_deployment": true',
        '"social_world_ui_as_stdlib_closure": true',
    ]
    for path in [DOC, REPORT, PACK / "README.md", CONTRACT]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "std_resource_network_policy_minimum_v1"], timeout=120)
    for pack in SUPPORT_PACKS:
        run([sys.executable, "tests/run_pack_golden.py", pack], timeout=300)
    run([sys.executable, "tests/run_roadmap_v2_na3_resource_network_policy_rebase_check.py"], timeout=600)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files()
    check_docs()
    check_matrix_not_closed()
    check_contract()
    check_golden()
    check_forbidden_claims()
    check_gates()
    print("[std-resource-network-policy-minimum] OK")


if __name__ == "__main__":
    main()
