#!/usr/bin/env python3
"""Validate MA5_CURRICULUM_LTS_PREREQ_REBASE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "MA5_CURRICULUM_LTS_PREREQ_REBASE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "마-5_PREREQ_REBASE_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "roadmap_v2_ma5_curriculum_lts_prereq_rebase_v1"
CONTRACT = PACK / "contract.detjson"
REBASE = PACK / "rebase.detjson"
CLOSURE_CHECK = ROOT / "tests" / "run_roadmap_v2_ma5_seamgrim_curriculum_5_lts_pack_closure_check.py"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ma5-prereq-rebase] FAIL: {message}", file=sys.stderr)
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
        PACK / "README.md",
        CONTRACT,
        REBASE,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "run_roadmap_v2_ma4_seamgrim_curriculum_4_publication_pack_closure_check.py",
        ROOT / "tests" / "run_studio_education_operations_lts_check.py",
        ROOT / "tests" / "run_studio_benchmark_lts_matrix_check.py",
        ROOT / "tests" / "run_studio_ma3_regression_gate_matrix_check.py",
        ROOT / "tests" / "studio_local_release_rehearsal_check_runner.mjs",
    ]:
        require_file(path)

    shared_tokens = [
        "MA5_CURRICULUM_LTS_PREREQ_REBASE_V1",
        "MA5 curriculum LTS prereq rebase 4/4 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 6/90 = 7%",
        "ROADMAP_V2 pack evidence 참고값: 25/90 = 28%",
        "Studio-local 초장기 계획: 8/18 = 44%",
        "MA5_SEAMGRIM_CURRICULUM_5_LTS_PACK_CLOSURE_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 4마루 나눔마루 | 공개 차시 | 공개 교재/lesson registry | publication pack | 닫힘-동작 |",
            "| 5마루 단단마루 | 교과 LTS | 학년/단원/버전 관리 | curriculum LTS | 진행 |",
        ],
    )
    require_tokens(GUIDE, ["#### 마-5", "| 현재 상태 | 진행 |", "| pack 후보 | `seamgrim_curriculum_5_v1` |"])
    require_tokens(TRACKER, ["| 20 | `마-5` | 교과 LTS 선행 rebase | 진행 |", "| `마-5` | Studio curriculum LTS prereq rebase | 진행 |"])
    require_tokens(MANIFEST, ["| `마-5` | `roadmap_v2_ma5_curriculum_lts_prereq_rebase_v1`; candidate anchors:"])


def check_ma5_status_startable_not_closed() -> None:
    matrix_line = ""
    for line in read(MATRIX).splitlines():
        if "| 5마루 단단마루 | 교과 LTS |" in line:
            matrix_line = line
            break
    if not matrix_line:
        fail("missing 마-5 matrix line")
    status_cell = matrix_line.rstrip().split("|")[-2].strip()
    if status_cell != "진행":
        fail(f"마-5 status must be 진행: {matrix_line}")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 4,
        "current_stage_total": 4,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 6,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 7,
        "roadmap_v2_pack_evidence_reference_closed": 25,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 28,
        "studio_local_super_long_closed": 8,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 44,
    }
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_ma5_curriculum_lts_prereq_rebase_v1",
        "kind": "roadmap_v2_ma5_curriculum_lts_prereq_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "MA5_CURRICULUM_LTS_PREREQ_REBASE_V1",
        "roadmap_coordinate": "마-5",
        "matrix_closure_claim": False,
        "matrix_closure_tier": "not_closed",
        "ma5_matrix_status": "진행",
        "requires_ma4_closed": True,
        "candidate_evidence_count": 4,
        "requires_lts_readiness_evidence": True,
        "current_stage": "MA5 curriculum LTS prereq rebase",
        "next_item": "MA5_SEAMGRIM_CURRICULUM_5_LTS_PACK_CLOSURE_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    check_payload(CONTRACT)

    rebase = read_json(REBASE)
    if rebase.get("status") != "docs_closed_startable":
        fail(f"rebase status={rebase.get('status')!r}")
    if rebase.get("ma5_matrix_status") != "진행":
        fail("rebase must record 마-5 as 진행")
    if rebase.get("matrix_closure_claim") is not False:
        fail("rebase must not claim matrix closure")
    if rebase.get("closure_pack_required") != "seamgrim_curriculum_5_v1":
        fail("rebase must require seamgrim_curriculum_5_v1 for closure")
    check_payload(REBASE)
    false_claims = rebase.get("false_claims", {})
    for key, value in false_claims.items():
        if value is not False:
            fail(f"false claim {key}={value!r}")


def check_candidate_evidence() -> None:
    rebase = read_json(REBASE)
    candidates = rebase.get("candidate_evidence", [])
    expected = [
        "studio_education_operations_lts_v1",
        "studio_benchmark_lts_matrix_v1",
        "studio_ma3_regression_gate_matrix_v1",
        "studio_local_release_rehearsal_check_v1",
    ]
    if [row.get("pack") for row in candidates] != expected:
        fail(f"candidate evidence mismatch: {candidates!r}")

    source_paths = [
        ROOT / "pack" / "studio_education_operations_lts_v1" / "contract.detjson",
        ROOT / "pack" / "studio_benchmark_lts_matrix_v1" / "contract.detjson",
        ROOT / "pack" / "studio_ma3_regression_gate_matrix_v1" / "contract.detjson",
        ROOT / "pack" / "studio_local_release_rehearsal_check_v1" / "contract.detjson",
    ]
    forbidden_true_keys = [
        "lts_certification_claim",
        "benchmark_execution_claim",
        "performance_baseline_claim",
        "performance_baseline_publication_claim",
        "release_approval_claim",
        "release_execution_claim",
        "public_release_claim",
        "github_release_claim",
        "public_upload_claim",
        "registry_publish_claim",
        "cloud_sync_claim",
        "account_setup_claim",
        "permission_system_claim",
    ]
    for path in source_paths:
        payload = read_json(path)
        for key in forbidden_true_keys:
            if key in payload and payload.get(key) is not False:
                fail(f"{path.relative_to(ROOT)} {key}={payload.get(key)!r}")


def check_forbidden_progress_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, REBASE]:
        text = read(path)
        forbidden = ["18/18 = 100%", "90/90 = 100%", "roadmap_v2_matrix_behavior_closed\": 7"]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden progress claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ma5_curriculum_lts_prereq_rebase_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_ma4_seamgrim_curriculum_4_publication_pack_closure_check.py"], timeout=900)
    run([sys.executable, "tests/run_roadmap_v2_ma5_lts_candidate_progress_boundary_check.py"], timeout=420)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    if CLOSURE_CHECK.is_file():
        run([sys.executable, str(CLOSURE_CHECK.relative_to(ROOT))], timeout=900)
        print("[roadmap-v2-ma5-prereq-rebase] OK (superseded by ma5 lts closure)")
        return
    check_files_and_docs()
    check_ma5_status_startable_not_closed()
    check_contracts()
    check_candidate_evidence()
    check_forbidden_progress_claims()
    check_gates()
    print("[roadmap-v2-ma5-prereq-rebase] OK")


if __name__ == "__main__":
    main()
