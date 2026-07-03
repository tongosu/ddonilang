#!/usr/bin/env python3
"""Validate ROADMAP_V2_DA4_MATH_PACKAGE_SHARE_FRONTIER_REBASE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROADMAP_V2_DA4_MATH_PACKAGE_SHARE_FRONTIER_REBASE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "다-4_RECONCILIATION_REPORT_20260608.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
PACK = ROOT / "pack" / "roadmap_v2_da4_math_package_share_frontier_rebase_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"
REGISTRY_SHARE_SEED = ROOT / "pack" / "studio_registry_share_seed_v1" / "registry_share_seed.detjson"

REGISTRY_SEED_LANE = ["studio_registry_share_seed_v1", "rep_math_function_line_v1"]
REGISTRY_SURFACE_LANE = [
    "seamgrim_registry_publish_install_shell_v1",
    "run_seamgrim_package_registry_surface_check.py",
    "run_seamgrim_sharing_publishing_surface_check.py",
    "run_seamgrim_publication_snapshot_surface_check.py",
]


def fail(message: str) -> None:
    print(f"[roadmap-v2-da4-math-package-share-frontier-rebase] FAIL: {message}", file=sys.stderr)
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


def check_files() -> None:
    for path in [
        DOC,
        REPORT,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        MATRIX,
        GUIDE,
        TRACKER,
        MANIFEST,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        RECONCILIATION,
        REGISTRY_SHARE_SEED,
        ROOT / "ROADMAP_V2_DA3_SEAMGRIM_MATH_VIEW_FRONTIER_REBASE_V1.md",
        ROOT / "tests" / "run_roadmap_v2_da3_seamgrim_math_view_frontier_rebase_check.py",
        ROOT / "pack" / "studio_registry_share_seed_v1" / "golden.jsonl",
        ROOT / "pack" / "seamgrim_registry_publish_install_shell_v1" / "contract.detjson",
    ]:
        require_file(path)


def check_docs() -> None:
    shared = [
        "ROADMAP_V2_DA4_MATH_PACKAGE_SHARE_FRONTIER_REBASE_V1",
        "ROADMAP_V2_DA5_MATH_LTS_FRONTIER_REBASE_V1",
        "DA4 math package share frontier rebase 6/6 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 49/90 = 54%",
        "ROADMAP_V2 pack evidence 참고값: 59/90 = 66%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "닫힘-동작",
        "runtime_claim:false",
        "product_code_change:false",
        "product_ui_change:false",
        "matrix_closure_claim:true",
        "roadmap_matrix_increment:true",
        "math_runtime_change:false",
        "math_surface_change:false",
        "registry_publish_claim:false",
        "public_upload_claim:false",
        "public_link_creation_claim:false",
        "install_enablement_claim:false",
        "publication_snapshot_emit_claim:false",
        "guide_status_change:false",
        "guide_pack_candidate_change:true",
        "docs_ssot_change:false",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared)
    require_tokens(PROJECT_STATUS, ["ROADMAP_V2_DA4_MATH_PACKAGE_SHARE_FRONTIER_REBASE_V1", "49/90 = 54%", "59/90 = 66%", "다-4"])
    require_tokens(CHANGELOG, ["ROADMAP_V2 DA4 math package share frontier rebase", "ROADMAP_V2_DA5_MATH_LTS_FRONTIER_REBASE_V1"])


def check_payloads() -> None:
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_da4_math_package_share_frontier_rebase_v1",
        "kind": "roadmap_v2_da4_math_package_share_frontier_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "ROADMAP_V2_DA4_MATH_PACKAGE_SHARE_FRONTIER_REBASE_V1",
        "selected_coordinate": "다-4",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "math_runtime_change": False,
        "math_surface_change": False,
        "registry_publish_claim": False,
        "public_upload_claim": False,
        "public_link_creation_claim": False,
        "install_enablement_claim": False,
        "publication_snapshot_emit_claim": False,
        "guide_status_change": False,
        "guide_pack_candidate_change": True,
        "current_stage": "DA4 math package share frontier rebase",
        "current_stage_closed": 6,
        "current_stage_total": 6,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 49,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 54,
        "roadmap_v2_pack_evidence_reference_closed": 59,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 66,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "selected_next_work": "ROADMAP_V2_DA5_MATH_LTS_FRONTIER_REBASE_V1",
        "docs_ssot_change": False,
        "requires_docs_ssot_clean": True,
    }
    contract = read_json(CONTRACT)
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")

    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("schema") != "ddn.roadmap_v2.da4_math_package_share_frontier_rebase.v1":
        fail("reconciliation schema mismatch")
    if reconciliation.get("coordinate") != "다-4":
        fail("reconciliation coordinate mismatch")
    if reconciliation.get("matrix_status_after") != "닫힘-동작":
        fail("reconciliation matrix_status_after mismatch")
    lanes = reconciliation.get("lanes", {})
    if lanes.get("registry_seed") != REGISTRY_SEED_LANE:
        fail("registry_seed lane mismatch")
    if lanes.get("registry_surface") != REGISTRY_SURFACE_LANE:
        fail("registry_surface lane mismatch")


def check_registry_seed_manifest() -> None:
    manifest = read_json(REGISTRY_SHARE_SEED)
    if manifest.get("schema") != "ddn.studio.registry_share_seed.v1":
        fail("registry share seed schema mismatch")
    rows = manifest.get("rows")
    if not isinstance(rows, list):
        fail("registry share seed rows must be a list")
    math_rows = [row for row in rows if row.get("lesson_id") == "rep_math_function_line_v1"]
    if len(math_rows) != 1:
        fail(f"expected one rep_math_function_line_v1 row, got {len(math_rows)}")
    row = math_rows[0]
    expected = {
        "registry_id": "studio/lesson/rep_math_function_line_v1",
        "scope": "나눔",
        "catalog_kind": "lesson_catalog",
        "visibility": "public_candidate",
        "share_kind": "link",
        "share_target": "artifact",
        "draft_only": True,
        "publish_claim": False,
    }
    for key, value in expected.items():
        if row.get(key) != value:
            fail(f"math registry row {key}={row.get(key)!r}, expected {value!r}")
    for flag in [
        "registry_publish_claim",
        "public_upload_claim",
        "public_link_creation_claim",
        "install_enablement_claim",
        "publication_snapshot_emit_claim",
        "github_release_claim",
        "archive_generation_claim",
        "publication_checksum_generation_claim",
        "cloud_sync_claim",
        "account_setup_claim",
        "permission_system_claim",
        "active_allowlist_mutation",
    ]:
        if manifest.get(flag) is not False:
            fail(f"manifest {flag} expected false, got {manifest.get(flag)!r}")


def check_da4_authority_state() -> None:
    require_tokens(
        MATRIX,
        ["| 4마루 나눔마루 | 수학가지 공유 | math package / examples | registry pack | 닫힘-동작 |"],
    )
    require_tokens(
        GUIDE,
        [
            "#### 다-4 — 수학가지 공유",
            "| 현재 상태 | 닫힘-동작 |",
            "`math_symbolic_proof_4_v1`",
            "`studio_registry_share_seed_v1`",
            "`seamgrim_registry_publish_install_shell_v1`",
            "`roadmap_v2_da4_math_package_share_frontier_rebase_v1`",
        ],
    )
    require_tokens(
        TRACKER,
        [
            "| 7.8 | `다-4` | 수학가지 공유 | 닫힘-동작 |",
            "| `다-4` | 수학가지 공유 | 닫힘-동작 |",
        ],
    )
    require_tokens(
        MANIFEST,
        [
            "| `다-4` | registry seed lane:",
            "registry surface lane:",
            "행렬 정합화: `python tests/run_roadmap_v2_da4_math_package_share_frontier_rebase_check.py`",
        ],
    )


def check_forbidden_claims() -> None:
    forbidden = [
        "18/18 = 100%",
        "90/90 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 50/90",
        '"roadmap_v2_matrix_behavior_closed": 50',
        '"math_runtime_change": true',
        '"math_surface_change": true',
        '"registry_publish_claim": true',
        '"public_upload_claim": true',
        '"public_link_creation_claim": true',
        '"install_enablement_claim": true',
        '"publication_snapshot_emit_claim": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
    ]
    for path in [DOC, REPORT, PACK / "README.md", CONTRACT, RECONCILIATION]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_da4_math_package_share_frontier_rebase_v1"], timeout=120)
    run([sys.executable, "tests/run_roadmap_v2_da3_seamgrim_math_view_frontier_rebase_check.py"], timeout=300)
    run([sys.executable, "tests/run_pack_golden.py", "studio_registry_share_seed_v1"], timeout=120)
    run([sys.executable, "tests/run_seamgrim_package_registry_surface_check.py"], timeout=180)
    run([sys.executable, "tests/run_seamgrim_sharing_publishing_surface_check.py"], timeout=180)
    run([sys.executable, "tests/run_seamgrim_publication_snapshot_surface_check.py"], timeout=180)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files()
    check_docs()
    check_payloads()
    check_registry_seed_manifest()
    check_da4_authority_state()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-da4-math-package-share-frontier-rebase] OK")


if __name__ == "__main__":
    main()
