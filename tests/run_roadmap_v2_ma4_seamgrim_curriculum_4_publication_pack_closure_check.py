#!/usr/bin/env python3
"""Validate MA4_SEAMGRIM_CURRICULUM_4_PUBLICATION_PACK_CLOSURE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "MA4_SEAMGRIM_CURRICULUM_4_PUBLICATION_PACK_CLOSURE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "마-4_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "seamgrim_curriculum_4_v1"
CONTRACT = PACK / "contract.detjson"
PUBLICATION_PACK = PACK / "publication_pack.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ma4-publication-closure] FAIL: {message}", file=sys.stderr)
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
        PUBLICATION_PACK,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "run_roadmap_v2_ma3_seamgrim_curriculum_3_classroom_ui_pack_closure_check.py",
        ROOT / "tests" / "run_seamgrim_package_registry_surface_check.py",
        ROOT / "tests" / "run_seamgrim_sharing_publishing_surface_check.py",
        ROOT / "tests" / "run_seamgrim_publication_snapshot_surface_check.py",
        ROOT / "tests" / "run_studio_public_lesson_publication_prep_check.py",
        ROOT / "tests" / "studio_lesson_publication_review_surface_runner.mjs",
        ROOT / "tests" / "run_studio_lesson_publication_review_surface_check.py",
        ROOT / "tests" / "run_studio_local_share_and_packaging_check.py",
    ]:
        require_file(path)

    shared_tokens = [
        "MA4_SEAMGRIM_CURRICULUM_4_PUBLICATION_PACK_CLOSURE_V1",
        "MA4 publication pack closure 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 6/90 = 7%",
        "ROADMAP_V2 pack evidence 참고값: 25/90 = 28%",
        "Studio-local 초장기 계획: 8/18 = 44%",
        "MA5_CURRICULUM_LTS_PREREQ_REBASE_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 3마루 작업실마루 | 수업용 작업실 | 교사용/학생용 모드 | classroom UI pack | 닫힘-동작 |",
            "| 4마루 나눔마루 | 공개 차시 | 공개 교재/lesson registry | publication pack | 닫힘-동작 |",
        ],
    )
    require_tokens(GUIDE, ["#### 마-4", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `seamgrim_curriculum_4_v1` |"])
    require_tokens(TRACKER, ["| 19 | `마-4` | 공개 차시 | 닫힘-동작 |", "| `마-4` | Studio public lesson publication pack closure | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `마-4` | `seamgrim_curriculum_4_v1`; consumes"])


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 6,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 7,
        "roadmap_v2_pack_evidence_reference_closed": 25,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 28,
        "studio_local_super_long_closed": 8,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 44,
    }
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_curriculum_4_v1",
        "kind": "roadmap_v2_ma4_publication_pack_closure",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "MA4_SEAMGRIM_CURRICULUM_4_PUBLICATION_PACK_CLOSURE_V1",
        "roadmap_coordinate": "마-4",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "ma4_matrix_status": "닫힘-동작",
        "requires_ma3_closed": True,
        "publication_candidate_count": 4,
        "requires_publication_runner_evidence": True,
        "current_stage": "MA4 publication pack closure",
        "next_item": "MA5_CURRICULUM_LTS_PREREQ_REBASE_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    check_payload(CONTRACT)

    closure = read_json(PUBLICATION_PACK)
    if closure.get("status") != "behavior_closed":
        fail(f"publication_pack status={closure.get('status')!r}")
    if closure.get("ma4_matrix_status") != "닫힘-동작":
        fail("publication_pack must record 마-4 as 닫힘-동작")
    if closure.get("matrix_closure_claim") is not True:
        fail("publication_pack must claim matrix closure")
    check_payload(PUBLICATION_PACK)
    false_claims = closure.get("false_claims", {})
    for key, value in false_claims.items():
        if value is not False:
            fail(f"false claim {key}={value!r}")


def check_publication_evidence() -> None:
    closure = read_json(PUBLICATION_PACK)
    pack = closure.get("publication_pack", {})
    expected_pack = {
        "pack_id": "seamgrim_curriculum_4_v1",
        "requires_ma3_pack": "seamgrim_curriculum_3_v1",
        "candidate_count": 4,
        "boundary": "local_publication_review_share",
    }
    for key, value in expected_pack.items():
        if pack.get(key) != value:
            fail(f"publication_pack {key}={pack.get(key)!r}")
    candidates = closure.get("publication_candidates", [])
    expected = [
        "seamgrim_registry_publish_install_shell_v1",
        "studio_public_lesson_publication_prep_v1",
        "studio_lesson_publication_review_surface_v1",
        "studio_local_share_and_packaging_v1",
    ]
    if [row.get("pack") for row in candidates] != expected:
        fail(f"publication candidates mismatch: {candidates!r}")

    registry_shell = read_json(ROOT / "pack" / "seamgrim_registry_publish_install_shell_v1" / "contract.detjson")
    if registry_shell.get("closure_claim") != "no":
        fail("registry shell source must not claim closure")
    prep = read_json(ROOT / "pack" / "studio_public_lesson_publication_prep_v1" / "contract.detjson")
    review = read_json(ROOT / "pack" / "studio_lesson_publication_review_surface_v1" / "contract.detjson")
    for source_name, source in [("publication prep", prep), ("review surface", review)]:
        for key in ["public_upload_claim", "registry_publish_claim", "cloud_sync_claim", "account_setup_claim", "permission_system_claim"]:
            if source.get(key) is not False:
                fail(f"{source_name} {key}={source.get(key)!r}")
    local_share = read_json(ROOT / "pack" / "studio_local_share_and_packaging_v1" / "contract.detjson")
    if "local_only_no_registry_cloud" not in local_share.get("covers", []):
        fail("local share evidence must stay local-only")


def check_forbidden_progress_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, PUBLICATION_PACK]:
        text = read(path)
        forbidden = ["18/18 = 100%", "90/90 = 100%"]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden progress claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "seamgrim_curriculum_4_v1"], timeout=240)
    run([sys.executable, "tests/run_pack_golden.py", "seamgrim_curriculum_3_v1"], timeout=240)
    run([sys.executable, "tests/run_seamgrim_package_registry_surface_check.py"], timeout=240)
    run([sys.executable, "tests/run_seamgrim_sharing_publishing_surface_check.py"], timeout=240)
    run([sys.executable, "tests/run_seamgrim_publication_snapshot_surface_check.py"], timeout=240)
    run([sys.executable, "tests/run_pack_golden.py", "studio_public_lesson_publication_prep_v1"], timeout=240)
    run(["node", "tests/studio_lesson_publication_review_surface_runner.mjs"], timeout=240)
    run([sys.executable, "tests/run_studio_local_share_and_packaging_check.py"], timeout=300)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contracts()
    check_publication_evidence()
    check_forbidden_progress_claims()
    check_gates()
    print("[roadmap-v2-ma4-publication-closure] OK")


if __name__ == "__main__":
    main()
