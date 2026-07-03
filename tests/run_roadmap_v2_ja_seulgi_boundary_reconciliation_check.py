#!/usr/bin/env python3
"""Validate JA_SEULGI_BOUNDARY_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "자-0-1-2_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_ja_seulgi_boundary_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ja-seulgi-boundary-reconciliation] FAIL: {message}", file=sys.stderr)
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
        ROOT / "pack" / "seulgi_v1" / "contract.detjson",
        ROOT / "pack" / "seulgi_gatekeeper_v1" / "contract.detjson",
        ROOT / "pack" / "sam_ai_ordering_v1" / "contract.detjson",
    ]:
        require_file(path)
    require_tokens(
        MATRIX,
        [
            "| 0마루 씨앗마루 | AI 경계 문서 | 슬기/배움틀/코어 3층 | boundary docs | 닫힘-동작 |",
            "| 1마루 첫실행마루 | SeulgiIntent smoke | intent schema, Gatekeeper skeleton | intent pack | 닫힘-동작 |",
            "| 2마루 닫힘마루 | Gatekeeper/InputSnapshot 닫힘 | world-affecting AI injection | AI boundary pack | 닫힘-동작 |",
        ],
    )
    require_tokens(
        TRACKER,
        [
            "| 12.6 | `자-0` | AI 경계 문서 | 닫힘-동작 |",
            "| 12.7 | `자-1` | SeulgiIntent smoke | 닫힘-동작 |",
            "| 12.8 | `자-2` | Gatekeeper/InputSnapshot 닫힘 | 닫힘-동작 |",
            "자-0-1-2_RECONCILIATION_REPORT_20260609.md",
        ],
    )
    require_tokens(
        MANIFEST,
        [
            "| `자-0` | `seulgi_v1`; `external_intent_boundary_v1`; `roadmap_v2_ja_seulgi_boundary_reconciliation_v1`",
            "| `자-1` | `seulgi_v1`; `roadmap_v2_ja_seulgi_boundary_reconciliation_v1` |",
            "| `자-2` | `seulgi_gatekeeper_v1`; `sam_ai_ordering_v1`; `roadmap_v2_ja_seulgi_boundary_reconciliation_v1` |",
        ],
    )
    shared = [
        "JA_SEULGI_BOUNDARY_RECONCILIATION_V1",
        "JA Seulgi boundary reconciliation 7/7 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 61/90 = 68%",
        "ROADMAP_V2 docs-closed: 2/90 = 2%",
        "ROADMAP_V2 pack evidence 참고값: 63/90 = 70%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_POST_JA2_FRONTIER_REBASE_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(
        CHANGELOG,
        [
            "ROADMAP_V2 JA Seulgi boundary reconciliation",
            "61/90 = 68%",
            "63/90 = 70%",
        ],
    )


def check_payload() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_ja_seulgi_boundary_reconciliation_v1",
        "kind": "roadmap_v2_ja_seulgi_boundary_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "JA_SEULGI_BOUNDARY_RECONCILIATION_V1",
        "matrix_closure_claim": True,
        "roadmap_matrix_behavior_increment": 2,
        "roadmap_docs_closed_increment": 1,
        "current_stage_closed": 7,
        "current_stage_total": 7,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 61,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 68,
        "roadmap_v2_docs_closed": 2,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 2,
        "roadmap_v2_pack_evidence_reference_closed": 63,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 70,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "model_artifact_share_claim": False,
        "production_ai_path_claim": False,
        "replay_safe_ai_lts_claim": False,
        "auto_apply_claim": False,
        "state_hash_owner": False,
        "next_item": "ROADMAP_V2_POST_JA2_FRONTIER_REBASE_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    if contract.get("docs_closed_coordinates") != ["자-0"]:
        fail("docs_closed_coordinates mismatch")
    if contract.get("behavior_closed_coordinates") != ["자-1", "자-2"]:
        fail("behavior_closed_coordinates mismatch")

    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("status") != "mixed_docs_and_behavior_closed":
        fail("reconciliation status mismatch")
    coords = reconciliation.get("coordinates")
    if not isinstance(coords, dict):
        fail("coordinates must be object")
    if coords.get("자-0", {}).get("new_status") != "닫힘-문서":
        fail("자-0 status mismatch")
    if coords.get("자-0", {}).get("behavior_closed") is not False:
        fail("자-0 behavior flag mismatch")
    for coord in ["자-1", "자-2"]:
        if coords.get(coord, {}).get("new_status") != "닫힘-동작":
            fail(f"{coord} status mismatch")
        if coords.get(coord, {}).get("behavior_closed") is not True:
            fail(f"{coord} behavior flag mismatch")
    for key, value in reconciliation.get("false_claims", {}).items():
        if value is not False:
            fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 62/90",
        '"roadmap_v2_matrix_behavior_closed": 62',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
        '"model_artifact_share_claim": true',
        '"production_ai_path_claim": true',
        '"replay_safe_ai_lts_claim": true',
        '"auto_apply_claim": true',
        '"state_hash_owner": true',
    ]
    for path in [REPORT, PACK / "README.md", CONTRACT, RECONCILIATION]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ja_seulgi_boundary_reconciliation_v1"], timeout=120)
    run([sys.executable, "tests/run_seulgi_v1_pack_check.py"], timeout=120)
    run([sys.executable, "tests/run_seulgi_gatekeeper_pack_check.py"], timeout=120)
    run([sys.executable, "tests/run_sam_ai_ordering_pack_check.py"], timeout=120)
    run([sys.executable, "tests/run_sam_seulgi_family_contract_selftest.py"], timeout=120)
    run([sys.executable, "tests/run_external_intent_seulgi_walk_alignment_check.py"], timeout=120)
    run([sys.executable, "tests/run_seamgrim_ci_gate_sam_seulgi_family_step_check.py"], timeout=120)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_payload()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ja-seulgi-boundary-reconciliation] OK")


if __name__ == "__main__":
    main()
