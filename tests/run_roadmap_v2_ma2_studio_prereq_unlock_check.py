#!/usr/bin/env python3
"""Validate MA2_STUDIO_PREREQ_UNLOCK_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "MA2_STUDIO_PREREQ_UNLOCK_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "마-2_PREREQ_UNLOCK_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "roadmap_v2_ma2_studio_prereq_unlock_v1"
CONTRACT = PACK / "contract.detjson"
UNLOCK = PACK / "unlock.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ma2-prereq-unlock] FAIL: {message}", file=sys.stderr)
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
        UNLOCK,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "run_roadmap_v2_ga2_matrix_status_reconciliation_check.py",
        ROOT / "tests" / "run_roadmap_v2_sa2_sprite_grid2d_final_closure_check.py",
        ROOT / "tests" / "run_roadmap_v2_ta2_matrix_status_reconciliation_check.py",
        ROOT / "tests" / "run_roadmap_v2_da1_final_closure_check.py",
        ROOT / "tests" / "run_roadmap_v2_sa1_rebase_check.py",
    ]:
        require_file(path)

    shared_tokens = [
        "MA2_STUDIO_PREREQ_UNLOCK_V1",
        "MA2 prereq unlock 4/4 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 3/90 = 3%",
        "ROADMAP_V2 pack evidence 참고값: 22/90 = 24%",
        "Studio-local 초장기 계획: 5/18 = 28%",
        "MA2_SEAMGRIM_CURRICULUM_2_PACK_CLOSURE_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 2마루 닫힘마루 | 교과 pack 닫힘 | 차시별 D-PACK + 활동지 연결 | pack/checker PASS | 닫힘-동작 |",
            "| 2마루 닫힘마루 | 대표 문법 pack 닫힘 | 채비/훅/조건/임자/계약 대표 pack | golden/checker PASS | 닫힘-동작 |",
            "| 2마루 닫힘마루 | sprite/grid2d 닫힘 | sprite skin, grid2d game input | sprite/grid pack | 닫힘-동작 |",
            "| 2마루 닫힘마루 | CI/golden gate | golden/checker/CI | CI PASS | 닫힘-동작 |",
        ],
    )
    require_tokens(GUIDE, ["#### 마-2", "| 현재 상태 | 닫힘-동작 |", "| 선행 의존 | 다-1 math_function, 사-1 graph/table, 타-2 |"])
    require_tokens(TRACKER, ["| 17 | `마-2` | 교과 pack 닫힘 | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `마-2` | `seamgrim_curriculum_2_v1`; consumes"])


def check_ma2_status_not_regressed() -> None:
    matrix_line = ""
    for line in read(MATRIX).splitlines():
        if "| 2마루 닫힘마루 | 교과 pack 닫힘 |" in line:
            matrix_line = line
            break
    if not matrix_line:
        fail("missing 마-2 matrix line")
    status_cell = matrix_line.rstrip().split("|")[-2].strip()
    if status_cell not in {"진행", "닫힘-동작"}:
        fail(f"마-2 status must be 진행 or 닫힘-동작: {matrix_line}")
    behavior_closed_rows = [
        "| 2마루 닫힘마루 | 대표 문법 pack 닫힘 | 채비/훅/조건/임자/계약 대표 pack | golden/checker PASS | 닫힘-동작 |",
        "| 2마루 닫힘마루 | sprite/grid2d 닫힘 | sprite skin, grid2d game input | sprite/grid pack | 닫힘-동작 |",
        "| 2마루 닫힘마루 | CI/golden gate | golden/checker/CI | CI PASS | 닫힘-동작 |",
    ]
    text = read(MATRIX)
    if any(row not in text for row in behavior_closed_rows):
        fail("ROADMAP_V2 matrix behavior-closed rows must remain 가-2/사-2/타-2")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "work_unit_closed": 4,
        "work_unit_total": 4,
        "work_unit_percent": 100,
        "current_stage_closed": 4,
        "current_stage_total": 4,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 3,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 3,
        "roadmap_v2_pack_evidence_reference_closed": 22,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 24,
        "studio_local_super_long_closed": 5,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 28,
    }
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_ma2_studio_prereq_unlock_v1",
        "kind": "roadmap_v2_ma2_studio_prereq_unlock",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "MA2_STUDIO_PREREQ_UNLOCK_V1",
        "roadmap_coordinate": "마-2",
        "matrix_closure_claim": False,
        "matrix_closure_tier": "not_closed",
        "ma2_matrix_status": "진행",
        "current_stage": "MA2 prereq unlock",
        "next_item": "MA2_SEAMGRIM_CURRICULUM_2_PACK_CLOSURE_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    check_payload(CONTRACT)

    unlock = read_json(UNLOCK)
    if unlock.get("status") != "docs_closed_startable":
        fail(f"unlock status={unlock.get('status')!r}")
    if unlock.get("ma2_matrix_status") != "진행":
        fail("unlock must record 마-2 as 진행")
    if unlock.get("matrix_closure_claim") is not False:
        fail("unlock must not claim matrix closure")
    check_payload(UNLOCK)
    false_claims = unlock.get("false_claims", {})
    for key, value in false_claims.items():
        if value is not False:
            fail(f"false claim {key}={value!r}")


def check_prerequisite_payload() -> None:
    unlock = read_json(UNLOCK)
    prereqs = unlock.get("prerequisites", {})
    global_coords = [item.get("coordinate") for item in prereqs.get("global_4era", [])]
    if global_coords != ["가-2", "사-2", "타-2"]:
        fail(f"global prereqs={global_coords!r}")
    guide_coords = [item.get("coordinate") for item in prereqs.get("matrix_guide", [])]
    if guide_coords != ["다-1", "사-1", "타-2"]:
        fail(f"guide prereqs={guide_coords!r}")
    if "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1" in json.dumps(prereqs, ensure_ascii=False):
        fail("numeric track consolidation must not be an unlock prerequisite")


def check_forbidden_progress_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, UNLOCK]:
        text = read(path)
        forbidden = ["18/18 = 100%", "90/90 = 100%", "roadmap_v2_matrix_behavior_closed\": 4"]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden progress claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ma2_studio_prereq_unlock_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_ga2_matrix_status_reconciliation_check.py"], timeout=420)
    run([sys.executable, "tests/run_roadmap_v2_sa2_sprite_grid2d_final_closure_check.py"], timeout=420)
    run([sys.executable, "tests/run_roadmap_v2_ta2_matrix_status_reconciliation_check.py"], timeout=420)
    run(
        [
            sys.executable,
            "tests/run_pack_golden.py",
            "math_vector_minimum_first_run_v1",
            "math_calculus_v1",
            "formula_relation_solve_v1",
            "relation_solve_system_2x2_v1",
            "relation_solve_ddn_bridge_v2",
            "relation_solve_wasm_cli_parity_v2",
            "math_numeric_int_v1",
            "math_numeric_diff_v1",
        ],
        timeout=420,
    )
    run([sys.executable, "tests/run_roadmap_v2_sa1_rebase_check.py"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_ma2_status_not_regressed()
    check_contracts()
    check_prerequisite_payload()
    check_forbidden_progress_claims()
    check_gates()
    print("[roadmap-v2-ma2-prereq-unlock] OK")


if __name__ == "__main__":
    main()
