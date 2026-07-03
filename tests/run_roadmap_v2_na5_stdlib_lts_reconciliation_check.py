#!/usr/bin/env python3
"""Validate NA5_STDLIB_LTS_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "나-5_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_na5_stdlib_lts_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-na5-stdlib-lts-reconciliation] FAIL: {message}", file=sys.stderr)
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
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        RECONCILIATION,
        ROOT / "pack" / "nurigym_shared_sync_priority_tiebreak_v1" / "golden_order_a.txt",
        ROOT / "pack" / "nurigym_shared_sync_priority_tiebreak_v1" / "golden_order_b.txt",
        ROOT / "pack" / "benchmark_baseline_v1" / "contract.detjson",
    ]:
        require_file(path)
    require_tokens(MATRIX, ["| 5마루 단단마루 | 표준 LTS | 호환성, deprecation, benchmark | stdlib LTS suite | 닫힘-동작 |"])
    require_tokens(TRACKER, ["| 52.8 | `나-5` | 표준 LTS | 닫힘-동작 |", "나-5_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `나-5` | `roadmap_v2_na4_stdlib_registry_reconciliation_v1`; `lang_history_alias_stdlib_bridge_v1`; `lang_velocity_verlet_stdlib_surface_acceptance_v1`; `benchmark_baseline_v1`; `nurigym_shared_sync_priority_tiebreak_v1`; `roadmap_v2_na5_stdlib_lts_reconciliation_v1` |"])
    require_tokens(ROOT / "pack" / "nurigym_shared_sync_priority_tiebreak_v1" / "golden_order_a.txt", ["sha256:92170b22fced9bb9e0772e56fdb455183722471780f12792f08dba1029d022c4"])
    require_tokens(ROOT / "pack" / "nurigym_shared_sync_priority_tiebreak_v1" / "golden_order_b.txt", ["sha256:4ee178754010d73fb2480d621bb9f84749c42a4c6b57805434ff259d57cdfdf0"])
    shared = [
        "NA5_STDLIB_LTS_RECONCILIATION_V1",
        "NA5 stdlib LTS reconciliation 8/8 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 67/90 = 74%",
        "ROADMAP_V2 docs-closed: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 69/90 = 77%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_LA4_LESSON_PACKAGE_RECONCILIATION_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 NA5 stdlib LTS reconciliation", "67/90 = 74%", "69/90 = 77%"])


def check_payload() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_na5_stdlib_lts_reconciliation_v1",
        "kind": "roadmap_v2_na5_stdlib_lts_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "NA5_STDLIB_LTS_RECONCILIATION_V1",
        "roadmap_coordinate": "나-5",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "current_stage_closed": 8,
        "current_stage_total": 8,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 67,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 74,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 69,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 77,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "next_item": "ROADMAP_V2_LA4_LESSON_PACKAGE_RECONCILIATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("coordinate") != "나-5":
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
        "ROADMAP_V2 행렬 닫힘-동작: 68/90",
        '"roadmap_v2_matrix_behavior_closed": 68',
        '"lts_certification_claim": true',
        '"public_release_claim": true',
        '"perf_sla_claim": true',
        '"broad_deprecation_removal_claim": true',
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
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_na5_stdlib_lts_reconciliation_v1"], timeout=120)
    run([sys.executable, "tests/run_roadmap_v2_na4_stdlib_registry_reconciliation_check.py"], timeout=600)
    run([sys.executable, "tests/run_lang_history_alias_stdlib_bridge_check.py"], timeout=300)
    run([sys.executable, "tests/run_lang_velocity_verlet_stdlib_surface_acceptance_check.py"], timeout=300)
    run([sys.executable, "tests/run_nurigym_shared_sync_priority_tiebreak_pack_check.py"], timeout=240)
    run([sys.executable, "tests/run_benchmark_baseline_pack_check.py"], timeout=900)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_payload()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-na5-stdlib-lts-reconciliation] OK")


if __name__ == "__main__":
    main()
