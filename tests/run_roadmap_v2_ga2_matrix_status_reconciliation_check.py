#!/usr/bin/env python3
"""Validate GA2_MATRIX_STATUS_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "GA2_MATRIX_STATUS_RECONCILIATION_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "가-2_RECONCILIATION_REPORT_20260608.md"
SOURCE_REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "가-2_REPORT_20260604.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "roadmap_v2_ga2_matrix_status_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ga2-reconciliation] FAIL: {message}", file=sys.stderr)
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
        SOURCE_REPORT,
        MATRIX,
        GUIDE,
        TRACKER,
        MANIFEST,
        PACK / "README.md",
        CONTRACT,
        RECONCILIATION,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "ROADMAP_V2_GA2_FINAL_CLOSURE_V1.md",
        ROOT / "LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1.md",
        ROOT / "pack" / "lang_core_2_v1" / "expected" / "lang_core_2.detjson",
        ROOT / "tests" / "run_lang_core_2_check.py",
        ROOT / "tests" / "run_lang_core_2_representative_grammar_pack_check.py",
        ROOT / "tests" / "run_roadmap_v2_ga2_final_closure_check.py",
    ]:
        require_file(path)
    shared_tokens = [
        "GA2_MATRIX_STATUS_RECONCILIATION_V1",
        "GA2 matrix reconciliation 4/4 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 3/90 = 3%",
        "ROADMAP_V2 pack evidence 참고값: 22/90 = 24%",
        "Studio-local 초장기 계획: 5/18 = 28%",
        "MA2_STUDIO_PREREQ_UNLOCK_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 2마루 닫힘마루 | 대표 문법 pack 닫힘 | 채비/훅/조건/임자/계약 대표 pack | golden/checker PASS | 닫힘-동작 |",
        ],
    )
    require_tokens(GUIDE, ["| 현재 상태 | 닫힘-동작 |", "pack 후보 | `lang_core_2_v1`"])
    require_tokens(TRACKER, ["| 4.5 | `가-2` | 대표 문법 pack 닫힘 | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `가-2` | `lang_core_2_v1`; `roadmap_v2_ga2_matrix_status_reconciliation_v1` |"])


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
        "pack": "roadmap_v2_ga2_matrix_status_reconciliation_v1",
        "kind": "roadmap_v2_ga2_matrix_status_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "GA2_MATRIX_STATUS_RECONCILIATION_V1",
        "roadmap_coordinate": "가-2",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "current_stage": "GA2 matrix reconciliation",
        "next_item": "MA2_STUDIO_PREREQ_UNLOCK_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
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


def check_surfaces() -> None:
    expected = read_json(ROOT / "pack" / "lang_core_2_v1" / "expected" / "lang_core_2.detjson")
    covered = set(expected.get("covered_surfaces", []))
    missing = sorted({"채비", "훅", "조건", "임자", "계약"} - covered)
    if missing:
        fail(f"missing representative surfaces: {missing}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ga2_matrix_status_reconciliation_v1"], timeout=240)
    run([sys.executable, "tests/run_lang_core_2_check.py"], timeout=300)
    run([sys.executable, "tests/run_lang_core_2_representative_grammar_pack_check.py"], timeout=300)
    run([sys.executable, "tests/run_roadmap_v2_ga2_final_closure_check.py"], timeout=300)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contracts()
    check_surfaces()
    check_gates()
    print("[roadmap-v2-ga2-reconciliation] OK")


if __name__ == "__main__":
    main()

