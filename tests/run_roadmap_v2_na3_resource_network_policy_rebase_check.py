#!/usr/bin/env python3
"""Validate ROADMAP_V2_NA3_RESOURCE_NETWORK_POLICY_REBASE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROADMAP_V2_NA3_RESOURCE_NETWORK_POLICY_REBASE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "나-3_REBASE_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_na3_resource_network_policy_rebase_v1"
CONTRACT = PACK / "contract.detjson"
REBASE = PACK / "rebase.detjson"
NA3_RECONCILIATION_DOC = ROOT / "ROADMAP_V2_NA3_MATRIX_STATUS_RECONCILIATION_V1.md"


def fail(message: str) -> None:
    print(f"[roadmap-v2-na3-rebase] FAIL: {message}", file=sys.stderr)
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


def na3_reconciled() -> bool:
    return NA3_RECONCILIATION_DOC.is_file()


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
        MATRIX,
        GUIDE,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        REBASE,
        ROOT / "ROADMAP_V2_NA2_MATRIX_STATUS_RECONCILIATION_V1.md",
        ROOT / "tests" / "run_roadmap_v2_na2_matrix_status_reconciliation_check.py",
        ROOT / "docs" / "status" / "STDLIB_PACK_COVERAGE.md",
        ROOT / "docs" / "status" / "STDLIB_IMPL_MATRIX.md",
        ROOT / "pack" / "stdlib_missing_coverage_io_v1" / "golden.jsonl",
        ROOT / "pack" / "age1_container_resource" / "input.ddn",
        ROOT / "pack" / "age1_container_resource" / "expect" / "state_hash.txt",
        ROOT / "pack" / "gogae3_w23_network_sam" / "README.md",
        ROOT / "pack" / "eco_network_flow_smoke" / "README.md",
        ROOT / "pack" / "seamgrim_exec_policy_effect_map_v1" / "golden.jsonl",
        ROOT / "pack" / "social_world_econ_2_v1" / "contract.detjson",
        ROOT / "pack" / "social_world_econ_3_v1" / "contract.detjson",
    ]:
        require_file(path)


def check_docs() -> None:
    shared = [
        "ROADMAP_V2_NA3_RESOURCE_NETWORK_POLICY_REBASE_V1",
        "STD_RESOURCE_NETWORK_POLICY_MINIMUM_V1",
        "NA3 resource/network/policy rebase 4/4 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 38/90 = 42%",
        "ROADMAP_V2 pack evidence 참고값: 58/90 = 64%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "닫힘-문서",
        "matrix_closure_claim:false",
        "roadmap_matrix_increment:false",
        "social_world_ui_as_stdlib_closure:false",
        "policy_advice:false",
        "live_policy_deployment:false",
        "docs_ssot_change:false",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared)
    require_tokens(PROJECT_STATUS, ["ROADMAP_V2_NA3_RESOURCE_NETWORK_POLICY_REBASE_V1", "38/90 = 42%", "58/90 = 64%", "9/18 = 50%"])
    require_tokens(CHANGELOG, ["ROADMAP_V2 NA3 resource/network/policy rebase", "STD_RESOURCE_NETWORK_POLICY_MINIMUM_V1"])
    require_tokens(GUIDE, ["#### 나-3", "| pack 후보 | `std_resource_network_policy_minimum_v1`"])
    guide_text = read(GUIDE)
    allowed_status = ["| 현재 상태 | 진행 |"]
    if na3_reconciled():
        allowed_status.append("| 현재 상태 | 닫힘-동작 |")
    if not any(token in guide_text for token in allowed_status):
        fail("guide status must be 진행, or 닫힘-동작 after NA3 reconciliation")


def check_matrix_not_closed() -> None:
    for line in read(MATRIX).splitlines():
        if "| 3마루 작업실마루 | std_agent/resource/network/policy |" in line:
            if "닫힘-동작" in line and not na3_reconciled():
                fail("나-3 matrix row must not be behavior-closed by this rebase")
            return
    fail("missing 나-3 matrix row")


def check_payloads() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_na3_resource_network_policy_rebase_v1",
        "kind": "roadmap_v2_na3_resource_network_policy_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "ROADMAP_V2_NA3_RESOURCE_NETWORK_POLICY_REBASE_V1",
        "roadmap_coordinate": "나-3",
        "matrix_closure_claim": False,
        "matrix_closure_tier": "닫힘-문서",
        "roadmap_matrix_increment": False,
        "current_stage": "NA3 resource/network/policy rebase",
        "current_stage_closed": 4,
        "current_stage_total": 4,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 38,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 42,
        "roadmap_v2_pack_evidence_reference_closed": 58,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 64,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "selected_next_work": "STD_RESOURCE_NETWORK_POLICY_MINIMUM_V1",
        "rejected_closure_basis": "social_world_ui_as_stdlib_closure",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    rebase = read_json(REBASE)
    if rebase.get("selected_next_work") != "STD_RESOURCE_NETWORK_POLICY_MINIMUM_V1":
        fail("selected next work mismatch")
    matrix_status = rebase.get("matrix_status", {})
    if matrix_status.get("new_guide_status") != "진행":
        fail("guide status must be 진행")
    if matrix_status.get("matrix_behavior_closed") is not False:
        fail("matrix_behavior_closed must be false")
    for axis in ["resource", "network", "policy"]:
        if not isinstance(rebase.get("axis_assessment", {}).get(axis), dict):
            fail(f"missing axis assessment: {axis}")
    policy = rebase.get("axis_assessment", {}).get("policy", {})
    if "seamgrim_exec_policy_effect_map_v1" not in set(policy.get("known_drift", [])):
        fail("policy known drift must include seamgrim_exec_policy_effect_map_v1")
    if policy.get("needs_pass_gate_selection") is not True:
        fail("policy needs_pass_gate_selection must be true")
    for key, value in rebase.get("claims", {}).items():
        if value is not False:
            fail(f"claim {key}={value!r}")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 39/90",
        '"roadmap_v2_matrix_behavior_closed": 39',
        "ROADMAP_V2 행렬 닫힘-동작: 90/90",
        "Studio-local 초장기 계획: 18/18",
        '"matrix_closure_claim": true',
        '"roadmap_matrix_increment": true',
        '"social_world_ui_as_stdlib_closure": true',
        '"policy_advice": true',
        '"live_policy_deployment": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
    ]
    for path in [DOC, REPORT, PACK / "README.md", CONTRACT, REBASE]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_na3_resource_network_policy_rebase_v1"], timeout=120)
    run([sys.executable, "tests/run_pack_golden.py", "stdlib_missing_coverage_io_v1"], timeout=120)
    run([sys.executable, "tests/run_pack_golden.py", "eco_network_flow_smoke"], timeout=240)
    run([sys.executable, "tests/run_pack_golden.py", "social_world_econ_2_v1"], timeout=120)
    run([sys.executable, "tests/run_pack_golden.py", "social_world_econ_3_v1"], timeout=120)
    run([sys.executable, "tests/run_roadmap_v2_na2_matrix_status_reconciliation_check.py"], timeout=300)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files()
    check_docs()
    check_matrix_not_closed()
    check_payloads()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-na3-rebase] OK")


if __name__ == "__main__":
    main()
