#!/usr/bin/env python3
"""Validate TA2_MATRIX_STATUS_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "TA2_MATRIX_STATUS_RECONCILIATION_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "타-2_RECONCILIATION_REPORT_20260608.md"
SOURCE_REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "타-2_REPORT_20260503.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "roadmap_v2_ta2_matrix_status_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ta2-reconciliation] FAIL: {message}", file=sys.stderr)
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
        safe_stdout = proc.stdout.encode(encoding, errors="backslashreplace").decode(encoding, errors="replace")
        print(safe_stdout, end="")
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
        ROOT / "pack" / "toolchain_pack_2_v1" / "README.md",
        ROOT / "pack" / "toolchain_pack_2_v1" / "expected" / "work_item_evidence.detjson",
        ROOT / "tests" / "run_roadmap_v2_work_item_evidence_check.py",
        ROOT / "tests" / "run_roadmap_v2_pack_runner_basis_check.py",
        ROOT / "tests" / "run_roadmap_v2_pack_skeleton_check.py",
    ]:
        require_file(path)
    shared_tokens = [
        "TA2_MATRIX_STATUS_RECONCILIATION_V1",
        "TA2 matrix reconciliation 4/4 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 2/90 = 2%",
        "ROADMAP_V2 pack evidence 참고값: 22/90 = 24%",
        "Studio-local 초장기 계획: 5/18 = 28%",
        "GA2_LANGUAGE_REPRESENTATIVE_PACK_CLOSURE_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 2마루 닫힘마루 | CI/golden gate | golden/checker/CI | CI PASS | 닫힘-동작 |",
        ],
    )
    require_tokens(GUIDE, ["| 현재 상태 | 닫힘-동작 |", "pack 후보 | `toolchain_pack_2_v1`"])
    require_tokens(TRACKER, ["| 7 | `타-2` | work item evidence gate 재정렬 | 닫힘-동작 |"])
    require_tokens(
        MANIFEST,
        ["| `타-2` | `toolchain_pack_2_v1`; `roadmap_v2_ta2_matrix_status_reconciliation_v1`; `roadmap_v2_ta2_guide_status_reconciliation_v1` |"],
    )


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
        "roadmap_v2_matrix_behavior_closed": 2,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 2,
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
        "pack": "roadmap_v2_ta2_matrix_status_reconciliation_v1",
        "kind": "roadmap_v2_ta2_matrix_status_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "TA2_MATRIX_STATUS_RECONCILIATION_V1",
        "roadmap_coordinate": "타-2",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "current_stage": "TA2 matrix reconciliation",
        "next_item": "GA2_LANGUAGE_REPRESENTATIVE_PACK_CLOSURE_V1",
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


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ta2_matrix_status_reconciliation_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_work_item_evidence_check.py"], timeout=300)
    run([sys.executable, "tests/run_roadmap_v2_pack_runner_basis_check.py"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_pack_skeleton_check.py"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contracts()
    check_gates()
    print("[roadmap-v2-ta2-reconciliation] OK")


if __name__ == "__main__":
    main()
