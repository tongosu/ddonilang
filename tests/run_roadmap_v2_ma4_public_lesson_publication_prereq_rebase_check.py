#!/usr/bin/env python3
"""Validate MA4_PUBLIC_LESSON_PUBLICATION_PREREQ_REBASE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "MA4_PUBLIC_LESSON_PUBLICATION_PREREQ_REBASE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "마-4_PREREQ_REBASE_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "roadmap_v2_ma4_public_lesson_publication_prereq_rebase_v1"
CONTRACT = PACK / "contract.detjson"
REBASE = PACK / "rebase.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ma4-prereq-rebase] FAIL: {message}", file=sys.stderr)
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
        REBASE,
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
        "MA4_PUBLIC_LESSON_PUBLICATION_PREREQ_REBASE_V1",
        "MA4 public lesson publication prereq rebase 4/4 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 24/90 = 27%",
        "Studio-local 초장기 계획: 7/18 = 39%",
        "MA4_SEAMGRIM_CURRICULUM_4_PUBLICATION_PACK_CLOSURE_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 3마루 작업실마루 | 수업용 작업실 | 교사용/학생용 모드 | classroom UI pack | 닫힘-동작 |",
            "| 4마루 나눔마루 | 공개 차시 | 공개 교재/lesson registry | publication pack |",
        ],
    )
    require_tokens(GUIDE, ["#### 마-4", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `seamgrim_curriculum_4_v1` |"])
    require_tokens(TRACKER, ["| 19 | `마-4` | 공개 차시 | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `마-4` | `seamgrim_curriculum_4_v1`; consumes"])


def check_ma4_status_not_regressed() -> None:
    matrix_line = ""
    for line in read(MATRIX).splitlines():
        if "| 4마루 나눔마루 | 공개 차시 |" in line:
            matrix_line = line
            break
    if not matrix_line:
        fail("missing 마-4 matrix line")
    status_cell = matrix_line.rstrip().split("|")[-2].strip()
    if status_cell not in {"진행", "닫힘-동작"}:
        fail(f"마-4 status must be 진행 or 닫힘-동작: {matrix_line}")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 4,
        "current_stage_total": 4,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 5,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 24,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 27,
        "studio_local_super_long_closed": 7,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 39,
    }
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_ma4_public_lesson_publication_prereq_rebase_v1",
        "kind": "roadmap_v2_ma4_public_lesson_publication_prereq_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "MA4_PUBLIC_LESSON_PUBLICATION_PREREQ_REBASE_V1",
        "roadmap_coordinate": "마-4",
        "matrix_closure_claim": False,
        "matrix_closure_tier": "not_closed",
        "ma4_matrix_status": "진행",
        "requires_ma3_closed": True,
        "candidate_evidence_count": 4,
        "requires_publication_runner_evidence": True,
        "current_stage": "MA4 public lesson publication prereq rebase",
        "next_item": "MA4_SEAMGRIM_CURRICULUM_4_PUBLICATION_PACK_CLOSURE_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    check_payload(CONTRACT)

    rebase = read_json(REBASE)
    if rebase.get("status") != "docs_closed_startable":
        fail(f"rebase status={rebase.get('status')!r}")
    if rebase.get("ma4_matrix_status") != "진행":
        fail("rebase must record 마-4 as 진행")
    if rebase.get("matrix_closure_claim") is not False:
        fail("rebase must not claim matrix closure")
    if rebase.get("closure_pack_required") != "seamgrim_curriculum_4_v1":
        fail("rebase must require seamgrim_curriculum_4_v1 for closure")
    check_payload(REBASE)
    false_claims = rebase.get("false_claims", {})
    for key, value in false_claims.items():
        if value is not False:
            fail(f"false claim {key}={value!r}")


def check_candidate_evidence() -> None:
    rebase = read_json(REBASE)
    candidates = rebase.get("candidate_evidence", [])
    expected = [
        "seamgrim_registry_publish_install_shell_v1",
        "studio_public_lesson_publication_prep_v1",
        "studio_lesson_publication_review_surface_v1",
        "studio_local_share_and_packaging_v1",
    ]
    if [row.get("pack") for row in candidates] != expected:
        fail(f"candidate evidence mismatch: {candidates!r}")

    registry_shell = read_json(ROOT / "pack" / "seamgrim_registry_publish_install_shell_v1" / "contract.detjson")
    if registry_shell.get("closure_claim") != "no":
        fail("registry shell evidence must not be a closure claim")
    prep = read_json(ROOT / "pack" / "studio_public_lesson_publication_prep_v1" / "contract.detjson")
    for key in ["public_upload_claim", "registry_publish_claim", "cloud_sync_claim", "account_setup_claim", "permission_system_claim"]:
        if prep.get(key) is not False:
            fail(f"publication prep {key}={prep.get(key)!r}")
    review = read_json(ROOT / "pack" / "studio_lesson_publication_review_surface_v1" / "contract.detjson")
    for key in ["public_upload_claim", "registry_publish_claim", "public_link_creation_claim", "install_enablement_claim"]:
        if review.get(key) is not False:
            fail(f"review surface {key}={review.get(key)!r}")
    local_share = read_json(ROOT / "pack" / "studio_local_share_and_packaging_v1" / "contract.detjson")
    covers = local_share.get("covers", [])
    if "local_only_no_registry_cloud" not in covers:
        fail("local share evidence must stay local-only")


def check_forbidden_progress_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, REBASE]:
        text = read(path)
        forbidden = ["18/18 = 100%", "90/90 = 100%", "roadmap_v2_matrix_behavior_closed\": 6"]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden progress claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ma4_public_lesson_publication_prereq_rebase_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_ma3_seamgrim_curriculum_3_classroom_ui_pack_closure_check.py"], timeout=900)
    run([sys.executable, "tests/run_seamgrim_package_registry_surface_check.py"], timeout=240)
    run([sys.executable, "tests/run_seamgrim_sharing_publishing_surface_check.py"], timeout=240)
    run([sys.executable, "tests/run_seamgrim_publication_snapshot_surface_check.py"], timeout=240)
    run([sys.executable, "tests/run_studio_public_lesson_publication_prep_check.py"], timeout=300)
    run(["node", "tests/studio_lesson_publication_review_surface_runner.mjs"], timeout=240)
    run([sys.executable, "tests/run_studio_lesson_publication_review_surface_check.py"], timeout=300)
    run([sys.executable, "tests/run_studio_local_share_and_packaging_check.py"], timeout=300)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_ma4_status_not_regressed()
    check_contracts()
    check_candidate_evidence()
    check_forbidden_progress_claims()
    check_gates()
    print("[roadmap-v2-ma4-prereq-rebase] OK")


if __name__ == "__main__":
    main()
