#!/usr/bin/env python3
"""Validate NA0_STDLIB_CANDIDATE_LIST_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "나-0_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_na0_stdlib_candidate_list_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-na0-stdlib-candidate-list] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    try:
        payload = json.loads(read(path))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be JSON object")
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


def section(path: Path, heading: str) -> str:
    text = read(path)
    start = text.find(heading)
    if start < 0:
        fail(f"{path.relative_to(ROOT)} missing section {heading}")
    next_heading = text.find("\n#### ", start + 1)
    if next_heading < 0:
        return text[start:]
    return text[start:next_heading]


def count_matrix_statuses() -> tuple[int, int, int]:
    rows = []
    for line in read(MATRIX).splitlines():
        if not line.startswith("| ") or "마루" not in line or line.startswith("| 마루"):
            continue
        cols = [col.strip() for col in line.strip().strip("|").split("|")]
        if len(cols) == 5 and cols[0] and cols[0][0] in "012345" and "마루" in cols[0]:
            rows.append(cols)
    return (
        len(rows),
        sum(1 for row in rows if row[-1] == "닫힘-동작"),
        sum(1 for row in rows if row[-1] == "닫힘-문서"),
    )


def check_docs() -> None:
    for path in [
        MATRIX,
        GUIDE,
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
        ROOT / "docs" / "status" / "STDLIB_IMPL_MATRIX.md",
        ROOT / "docs" / "status" / "STDLIB_PACK_COVERAGE.md",
        ROOT / "pack" / "stdlib_text_basics" / "golden.jsonl",
        ROOT / "pack" / "stdlib_charim_basics" / "golden.jsonl",
        ROOT / "pack" / "stdlib_range_basics" / "golden.jsonl",
        ROOT / "pack" / "stdlib_math_basics" / "golden.jsonl",
        ROOT / "pack" / "stdlib_map_basics" / "golden.jsonl",
        ROOT / "pack" / "stdlib_1_v1" / "golden.jsonl",
        ROOT / "tests" / "run_stdlib_catalog_check.py",
        ROOT / "tests" / "run_stdlib_1_check.py",
    ]:
        require_file(path)
    require_tokens(MATRIX, ["| 0마루 씨앗마루 | 표준가지 후보 목록 확정 | std_core/text/time/random/unit/color/grid/input/resource/network/policy | stdlib proposal | 닫힘-동작 |"])
    na0_section = section(GUIDE, "#### 나-0 — 표준가지 후보 목록 확정")
    if "| 현재 상태 | 닫힘-동작 |" not in na0_section:
        fail("GUIDE 나-0 status is not 닫힘-동작")
    require_tokens(TRACKER, ["| 8.9 | `나-0` | 표준가지 후보 목록 확정 | 닫힘-동작 |", "나-0_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `나-0` | stdlib catalog; stdlib core candidate packs; `stdlib_1_v1`; `roadmap_v2_na0_stdlib_candidate_list_v1` |"])
    shared = [
        "NA0_STDLIB_CANDIDATE_LIST_RECONCILIATION_V1",
        "NA0 Stdlib candidate list 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 81/90 = 90%",
        "ROADMAP_V2 docs-closed: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 83/90 = 92%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "DA0_MATH_PROOF_SCOPE_RECONCILIATION_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 NA0 stdlib candidate list reconciliation", "81/90 = 90%", "83/90 = 92%"])
    total, behavior, docs = count_matrix_statuses()
    if total != 90 or behavior != 81 or docs != 5:
        fail(f"matrix counts mismatch: rows={total} behavior={behavior} docs={docs}")


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
        "roadmap_v2_matrix_behavior_closed": 81,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 90,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 83,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 92,
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
        "pack": "roadmap_v2_na0_stdlib_candidate_list_v1",
        "kind": "roadmap_v2_na0_stdlib_candidate_list_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "new_stdlib_surface": False,
        "stdlib_runtime_change": False,
        "parser_frontdoor_change": False,
        "std_core_grid_input_first_run_claim": False,
        "unit_random_event_claim": False,
        "resource_network_policy_claim": False,
        "registry_claim": False,
        "lts_claim": False,
        "closed_by": "NA0_STDLIB_CANDIDATE_LIST_RECONCILIATION_V1",
        "roadmap_coordinate": "나-0",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "next_item": "DA0_MATH_PROOF_SCOPE_RECONCILIATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    check_payload(CONTRACT)
    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("status") != "behavior_closed":
        fail("reconciliation status mismatch")
    if reconciliation.get("matrix_status_record", {}).get("new_status") != "닫힘-동작":
        fail("matrix status record mismatch")
    for key in ["candidate_catalog", "core_candidate_packs", "umbrella_regression"]:
        if not isinstance(reconciliation.get("evidence", {}).get(key), dict):
            fail(f"missing evidence axis: {key}")
    for key, value in reconciliation.get("false_claims", {}).items():
        if value is not False:
            fail(f"false claim {key}={value!r}")
    check_payload(RECONCILIATION)


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 82/90",
        '"roadmap_v2_matrix_behavior_closed": 82',
        '"new_stdlib_surface": true',
        '"stdlib_runtime_change": true',
        '"parser_frontdoor_change": true',
        '"std_core_grid_input_first_run_claim": true',
        '"unit_random_event_claim": true',
        '"resource_network_policy_claim": true',
        '"registry_claim": true',
        '"lts_claim": true',
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
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_na0_stdlib_candidate_list_v1"], timeout=120)
    run([sys.executable, "tests/run_stdlib_catalog_check.py"], timeout=240)
    run([sys.executable, "tests/run_stdlib_1_check.py"], timeout=240)
    run(
        [
            sys.executable,
            "tests/run_pack_golden.py",
            "stdlib_text_basics",
            "stdlib_charim_basics",
            "stdlib_range_basics",
            "stdlib_math_basics",
            "stdlib_map_basics",
            "stdlib_1_v1",
        ],
        timeout=240,
    )
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-na0-stdlib-candidate-list] OK")


if __name__ == "__main__":
    main()
