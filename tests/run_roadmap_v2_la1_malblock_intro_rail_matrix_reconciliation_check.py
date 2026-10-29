#!/usr/bin/env python3
"""Validate ROADMAP_V2_LA1_MALBLOCK_INTRO_RAIL_MATRIX_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "라-1_RECONCILIATION_REPORT_20260609.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_la1_malblock_intro_rail_matrix_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-la1-reconciliation] FAIL: {message}", file=sys.stderr)
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
        encoding = sys.stdout.encoding or "utf-8"
        safe_stdout = proc.stdout.encode(encoding, errors="replace").decode(encoding, errors="replace")
        print(safe_stdout, end="")
        fail(f"command failed: {' '.join(args)}")
    return proc


def check_files_and_docs() -> None:
    for path in [
        REPORT,
        MATRIX,
        TRACKER,
        MANIFEST,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        RECONCILIATION,
        ROOT / "docs" / "status" / "roadmap_v2" / "라-1_REPORT_20260428.md",
        ROOT / "pack" / "seamgrim_intro_exec_rail_v1" / "expected" / "intro_exec_rail.detjson",
        ROOT / "pack" / "block_editor_screen_intro_exec_v1" / "expected" / "block_editor_screen.detjson",
        ROOT / "pack" / "seamgrim_editor_run_handoff_v1" / "contract.detjson",
        ROOT / "pack" / "seamgrim_editor_run_transaction_v1" / "contract.detjson",
        ROOT / "tests" / "run_seamgrim_intro_exec_rail_check.py",
        ROOT / "tests" / "run_seamgrim_editor_run_handoff_check.py",
        ROOT / "tests" / "run_seamgrim_editor_run_transaction_check.py",
        ROOT / "tests" / "seamgrim_block_editor_runner.mjs",
        ROOT / "tests" / "run_roadmap_v2_na1_post_matrix_frontier_rebase_check.py",
    ]:
        require_file(path)

    shared_tokens = [
        "ROADMAP_V2_LA1_MALBLOCK_INTRO_RAIL_MATRIX_RECONCILIATION_V1",
        "LA1 matrix reconciliation 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 53/90 = 59%",
        "ROADMAP_V2 pack evidence 참고값: 59/90 = 66%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_POST_LA1_FRONTIER_REBASE_V1",
    ]
    for path in [REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(PROJECT_STATUS, ["ROADMAP_V2_LA1_MALBLOCK_INTRO_RAIL_MATRIX_RECONCILIATION_V1", "53/90 = 59%", "59/90 = 66%", "라-1"])
    require_tokens(CHANGELOG, ["ROADMAP_V2 LA1 malblock intro rail matrix reconciliation", "ROADMAP_V2_POST_LA1_FRONTIER_REBASE_V1"])

    require_tokens(
        MATRIX,
        [
            "| 1마루 첫실행마루 | block→DDN 첫실행 | 라-1-01 기본 팔레트, 라-1-02 codegen | generated DDN check/run | 닫힘-동작 |",
            "| 1 | 라-1 | 말블록 기본 팔레트 + block→DDN codegen | seamgrim_malblock_codegen_v1 | 닫힘-동작 |",
        ],
    )
    require_tokens(TRACKER, ["| 6 | `라-1` | 말블록 기본 팔레트 + block->DDN 첫실행 | 닫힘-동작 |", "라-1_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(
        MANIFEST,
        [
            "| `라-1` | `seamgrim_intro_exec_rail_v1`; 하위: `seamgrim_intro_exec_wasm_v1`, `seamgrim_intro_exec_blocky_v1`, `block_editor_screen_intro_exec_v1`, `seamgrim_editor_run_transaction_v1`; 보조: `seamgrim_malblock_codegen_v1`, `seamgrim_editor_run_handoff_v1`; `roadmap_v2_la1_malblock_intro_rail_matrix_reconciliation_v1` |",
            "행렬 정합화: `python tests/run_roadmap_v2_la1_malblock_intro_rail_matrix_reconciliation_check.py`",
        ],
    )


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "work_unit_closed": 5,
        "work_unit_total": 5,
        "work_unit_percent": 100,
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 53,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 59,
        "roadmap_v2_pack_evidence_reference_closed": 59,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 66,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
    }
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}, expected {value!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_la1_malblock_intro_rail_matrix_reconciliation_v1",
        "kind": "roadmap_v2_la1_malblock_intro_rail_matrix_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "malblock_runtime_change": False,
        "malblock_ui_change": False,
        "parser_frontdoor_change": False,
        "closed_by": "ROADMAP_V2_LA1_MALBLOCK_INTRO_RAIL_MATRIX_RECONCILIATION_V1",
        "roadmap_coordinate": "라-1",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "current_stage": "LA1 matrix reconciliation",
        "next_item": "ROADMAP_V2_POST_LA1_FRONTIER_REBASE_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    check_payload(CONTRACT)

    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("status") != "behavior_closed":
        fail(f"reconciliation status={reconciliation.get('status')!r}")
    if reconciliation.get("matrix_status_record", {}).get("new_status") != "닫힘-동작":
        fail("missing matrix status record")
    check_payload(RECONCILIATION)
    false_claims = reconciliation.get("false_claims", {})
    for key, value in false_claims.items():
        if value is not False:
            fail(f"false claim {key}={value!r}")
    evidence = reconciliation.get("evidence", {})
    for key in ["intro_rail", "block_editor_screen", "editor_handoff", "editor_transaction", "downstream_consistency"]:
        if not isinstance(evidence.get(key), dict):
            fail(f"missing evidence axis: {key}")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 54/90",
        '"roadmap_v2_matrix_behavior_closed": 54',
        "ROADMAP_V2 행렬 닫힘-동작: 90/90",
        "Studio-local 초장기 계획: 18/18",
        '"pack_evidence_reference_inflation": true',
        '"studio_local_progress_inflation": true',
        '"new_malblock_runtime": true',
        '"new_block_editor_ui": true',
        '"parser_frontdoor_change": true',
        '"runtime_surface_change": true',
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
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_la1_malblock_intro_rail_matrix_reconciliation_v1"], timeout=120)
    run([sys.executable, "tests/run_roadmap_v2_na1_post_matrix_frontier_rebase_check.py"], timeout=480)
    run([sys.executable, "tests/run_seamgrim_intro_exec_rail_check.py"], timeout=240)
    run([sys.executable, "tests/run_seamgrim_editor_run_handoff_check.py"], timeout=240)
    run([sys.executable, "tests/run_seamgrim_editor_run_transaction_check.py"], timeout=240)
    run(["node", "--no-warnings", "tests/seamgrim_block_editor_runner.mjs", "pack/block_editor_screen_intro_exec_v1"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-la1-reconciliation] OK")


if __name__ == "__main__":
    main()
