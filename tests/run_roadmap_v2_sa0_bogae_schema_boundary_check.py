#!/usr/bin/env python3
"""Validate SA0_BOGAE_SCHEMA_BOUNDARY_RECONCILIATION_V1."""

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
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "사-0_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_sa0_bogae_schema_boundary_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-sa0-bogae-schema-boundary] FAIL: {message}", file=sys.stderr)
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
        ROOT / "pack" / "bogae_api_catalog_v1_basic" / "golden.jsonl",
        ROOT / "pack" / "bogae_backend_parity_console_web_v1" / "expected" / "smoke.detjson",
        ROOT / "tests" / "bogae_alias_family" / "README.md",
        ROOT / "tests" / "bogae_alias_viewer_family" / "README.md",
        ROOT / "tests" / "run_bogae_backend_profile_smoke_check.py",
        ROOT / "tests" / "run_bogae_alias_family_selftest.py",
        ROOT / "tests" / "run_bogae_alias_viewer_family_selftest.py",
    ]:
        require_file(path)
    require_tokens(MATRIX, ["| 0마루 씨앗마루 | family/backend/profile 경계 | graph/table/space/grid/sprite/3D | bogae schema docs | 닫힘-동작 |"])
    sa0_section = section(GUIDE, "#### 사-0 — family/backend/profile 경계")
    if "| 현재 상태 | 닫힘-동작 |" not in sa0_section:
        fail("GUIDE 사-0 status is not 닫힘-동작")
    require_tokens(TRACKER, ["| 7.95 | `사-0` | family/backend/profile 경계 | 닫힘-동작 |", "사-0_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `사-0` | `bogae_api_catalog_v1_basic`; `bogae_backend_parity_console_web_v1`; `bogae_alias_family`; `bogae_alias_viewer_family`; `roadmap_v2_sa0_bogae_schema_boundary_v1` |"])
    shared = [
        "SA0_BOGAE_SCHEMA_BOUNDARY_RECONCILIATION_V1",
        "SA0 Bogae schema boundary 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 80/90 = 89%",
        "ROADMAP_V2 docs-closed: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 82/90 = 91%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "NA0_STDLIB_CANDIDATE_LIST_RECONCILIATION_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 SA0 Bogae schema boundary reconciliation", "80/90 = 89%", "82/90 = 91%"])
    total, behavior, docs = count_matrix_statuses()
    if total != 90 or behavior < 80 or docs < 5:
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
        "roadmap_v2_matrix_behavior_closed": 80,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 89,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 82,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 91,
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
        "pack": "roadmap_v2_sa0_bogae_schema_boundary_v1",
        "kind": "roadmap_v2_sa0_bogae_schema_boundary",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "new_bogae_runtime": False,
        "new_bogae_ui": False,
        "graph_space2d_first_run_claim": False,
        "sprite_grid2d_claim": False,
        "game_preview_claim": False,
        "asset_view_share_claim": False,
        "renderer_hardening_claim": False,
        "state_replay_truth_claim": False,
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "roadmap_coordinate": "사-0",
        "next_item": "NA0_STDLIB_CANDIDATE_LIST_RECONCILIATION_V1",
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
    for key in ["api_catalog", "backend_profile", "alias_family", "viewer_family"]:
        if not isinstance(reconciliation.get("evidence", {}).get(key), dict):
            fail(f"missing evidence axis: {key}")
    for key, value in reconciliation.get("false_claims", {}).items():
        if value is not False:
            fail(f"false claim {key}={value!r}")
    check_payload(RECONCILIATION)


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 81/90",
        '"roadmap_v2_matrix_behavior_closed": 81',
        '"new_bogae_runtime": true',
        '"new_bogae_ui": true',
        '"graph_space2d_first_run_claim": true',
        '"sprite_grid2d_claim": true',
        '"game_preview_claim": true',
        '"asset_view_share_claim": true',
        '"renderer_hardening_claim": true',
        '"state_replay_truth_claim": true',
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
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_sa0_bogae_schema_boundary_v1"], timeout=120)
    run([sys.executable, "tests/run_pack_golden.py", "bogae_api_catalog_v1_basic"], timeout=120)
    run([sys.executable, "tests/run_bogae_backend_profile_smoke_check.py"], timeout=600)
    run([sys.executable, "tests/run_bogae_alias_family_selftest.py"], timeout=120)
    run([sys.executable, "tests/run_bogae_alias_viewer_family_selftest.py"], timeout=120)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-sa0-bogae-schema-boundary] OK")


if __name__ == "__main__":
    main()
