#!/usr/bin/env python3
"""Validate MA3_STUDIO_CLASSROOM_WORKBENCH_PREREQ_REBASE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "MA3_STUDIO_CLASSROOM_WORKBENCH_PREREQ_REBASE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "마-3_PREREQ_REBASE_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "roadmap_v2_ma3_studio_classroom_workbench_prereq_rebase_v1"
CONTRACT = PACK / "contract.detjson"
REBASE = PACK / "rebase.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ma3-prereq-rebase] FAIL: {message}", file=sys.stderr)
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
        ROOT / "tests" / "studio_classroom_mode_browser_runner.mjs",
        ROOT / "tests" / "studio_classroom_report_workflow_runner.mjs",
        ROOT / "tests" / "studio_lesson_authoring_run_integration_runner.mjs",
        ROOT / "tests" / "run_roadmap_v2_ma2_seamgrim_curriculum_2_pack_closure_check.py",
    ]:
        require_file(path)

    shared_tokens = [
        "MA3_STUDIO_CLASSROOM_WORKBENCH_PREREQ_REBASE_V1",
        "MA3 prereq rebase 4/4 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 4/90 = 4%",
        "ROADMAP_V2 pack evidence 참고값: 23/90 = 26%",
        "Studio-local 초장기 계획: 6/18 = 33%",
        "MA3_SEAMGRIM_CURRICULUM_3_CLASSROOM_UI_PACK_CLOSURE_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 2마루 닫힘마루 | 교과 pack 닫힘 | 차시별 D-PACK + 활동지 연결 | pack/checker PASS | 닫힘-동작 |",
            "| 3마루 작업실마루 | 수업용 작업실 | 교사용/학생용 모드 | classroom UI pack |",
        ],
    )
    require_tokens(GUIDE, ["#### 마-3", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `seamgrim_curriculum_3_v1` |"])
    require_tokens(TRACKER, ["| 18 | `마-3` | 수업용 작업실 | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `마-3` | `seamgrim_curriculum_3_v1`; consumes"])


def check_ma3_status_not_regressed() -> None:
    matrix_line = ""
    for line in read(MATRIX).splitlines():
        if "| 3마루 작업실마루 | 수업용 작업실 |" in line:
            matrix_line = line
            break
    if not matrix_line:
        fail("missing 마-3 matrix line")
    status_cell = matrix_line.rstrip().split("|")[-2].strip()
    if status_cell not in {"진행", "닫힘-동작"}:
        fail(f"마-3 status must be 진행 or 닫힘-동작: {matrix_line}")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 4,
        "current_stage_total": 4,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 4,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 4,
        "roadmap_v2_pack_evidence_reference_closed": 23,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 26,
        "studio_local_super_long_closed": 6,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 33,
    }
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_ma3_studio_classroom_workbench_prereq_rebase_v1",
        "kind": "roadmap_v2_ma3_studio_classroom_workbench_prereq_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "MA3_STUDIO_CLASSROOM_WORKBENCH_PREREQ_REBASE_V1",
        "roadmap_coordinate": "마-3",
        "matrix_closure_claim": False,
        "matrix_closure_tier": "not_closed",
        "ma3_matrix_status": "진행",
        "current_stage": "MA3 prereq rebase",
        "next_item": "MA3_SEAMGRIM_CURRICULUM_3_CLASSROOM_UI_PACK_CLOSURE_V1",
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
    if rebase.get("ma3_matrix_status") != "진행":
        fail("rebase must record 마-3 as 진행")
    if rebase.get("matrix_closure_claim") is not False:
        fail("rebase must not claim matrix closure")
    check_payload(REBASE)
    false_claims = rebase.get("false_claims", {})
    for key, value in false_claims.items():
        if value is not False:
            fail(f"false claim {key}={value!r}")


def check_candidate_evidence() -> None:
    rebase = read_json(REBASE)
    candidates = rebase.get("candidate_evidence", [])
    expected = [
        "studio_classroom_mode_v1",
        "studio_classroom_report_workflow_v1",
        "studio_lesson_authoring_run_integration_v1",
    ]
    if [row.get("pack") for row in candidates] != expected:
        fail(f"candidate evidence mismatch: {candidates!r}")


def check_forbidden_progress_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, REBASE]:
        text = read(path)
        forbidden = ["18/18 = 100%", "90/90 = 100%", "roadmap_v2_matrix_behavior_closed\": 5"]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden progress claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ma3_studio_classroom_workbench_prereq_rebase_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_ma2_seamgrim_curriculum_2_pack_closure_check.py"], timeout=900)
    run(["node", "tests/studio_classroom_mode_browser_runner.mjs"], timeout=240)
    run(["node", "tests/studio_classroom_report_workflow_runner.mjs"], timeout=240)
    run(["node", "tests/studio_lesson_authoring_run_integration_runner.mjs"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_ma3_status_not_regressed()
    check_contracts()
    check_candidate_evidence()
    check_forbidden_progress_claims()
    check_gates()
    print("[roadmap-v2-ma3-prereq-rebase] OK")


if __name__ == "__main__":
    main()
