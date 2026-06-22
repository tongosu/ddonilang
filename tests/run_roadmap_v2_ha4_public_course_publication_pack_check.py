#!/usr/bin/env python3
"""Validate HA4_PUBLIC_COURSE_PUBLICATION_PACK_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "HA4_PUBLIC_COURSE_PUBLICATION_PACK_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "하-4_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "education_curriculum_4_v1"
CONTRACT = PACK / "contract.detjson"
PUBLICATION = PACK / "publication_pack.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "education_publication_pack.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
UI_RUNNER = ROOT / "tests" / "education_publication_pack_runner.mjs"
PREREQ_HA3 = ROOT / "tests" / "run_roadmap_v2_ha3_classroom_ui_pack_check.py"
PREREQ_MA4 = ROOT / "tests" / "run_roadmap_v2_ma4_seamgrim_curriculum_4_publication_pack_closure_check.py"
PREREQ_KA4 = ROOT / "tests" / "run_roadmap_v2_ka4_public_registry_seed_check.py"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ha4-public-course-publication-pack] FAIL: {message}", file=sys.stderr)
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
        PUBLICATION,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        UI_MODULE,
        APP,
        INDEX_HTML,
        STYLES,
        UI_RUNNER,
        PREREQ_HA3,
        PREREQ_MA4,
        PREREQ_KA4,
    ]:
        require_file(path)
    shared_tokens = [
        "HA4_PUBLIC_COURSE_PUBLICATION_PACK_V1",
        "HA4 public course publication pack closure 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 31/90 = 34%",
        "ROADMAP_V2 pack evidence 참고값: 51/90 = 57%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "HA5_EDUCATION_OPERATIONS_LTS_V1",
        "public upload",
        "public_link_creation:false",
        "registry_publish:false",
        "release_execution:false",
        "state_hash_participation:false",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(MATRIX, ["| 4마루 나눔마루 | 공개 강좌/교재 | 5분/1시간/4주 과정 | publication pack | 닫힘-동작 |"])
    require_tokens(GUIDE, ["#### 하-4", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `education_curriculum_4_v1` |"])
    require_tokens(TRACKER, ["| 46 | `하-4` | 공개 강좌/교재 | 닫힘-동작 |", "| `하-4` | education publication pack | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `하-4` | `education_curriculum_4_v1`; UI `education_publication_pack.js`; runner `education_publication_pack_runner.mjs` |"])
    require_tokens(DEV_SURFACES, ["education_publication_pack.js", "__EDUCATION_PUBLICATION_PACK__"])
    require_tokens(INDEX_HTML, ["id=\"education-publication-pack\"", "data-education-publication-pack"])
    require_tokens(STYLES, [".education-publication-pack", ".education-publication-artifacts", ".education-publication-preview"])


def check_status_closed() -> None:
    for line in read(MATRIX).splitlines():
        if "| 4마루 나눔마루 | 공개 강좌/교재 |" in line:
            if line.rstrip().split("|")[-2].strip() != "닫힘-동작":
                fail(f"하-4 status must be 닫힘-동작: {line}")
            return
    fail("missing 하-4 matrix line")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 31,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 34,
        "roadmap_v2_pack_evidence_reference_closed": 51,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 57,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
    }
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "education_curriculum_4_v1",
        "kind": "roadmap_v2_ha4_public_course_publication_pack_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "HA4_PUBLIC_COURSE_PUBLICATION_PACK_V1",
        "roadmap_coordinate": "하-4",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "ha4_matrix_status": "닫힘-동작",
        "requires_ha3_closed": True,
        "requires_ma4_publication_reference": True,
        "requires_ka4_registry_seed_reference": True,
        "education_publication_pack_claim": True,
        "micro_course_claim": True,
        "workshop_course_claim": True,
        "four_week_course_claim": True,
        "publication_bundle_claim": True,
        "share_handoff_claim": True,
        "public_upload_claim": False,
        "public_link_creation_claim": False,
        "registry_publish_claim": False,
        "release_execution_claim": False,
        "artifact_signing_claim": False,
        "account_permission_change_claim": False,
        "state_hash_participation_claim": False,
        "parser_frontdoor_change": False,
        "grammar_claim": False,
        "current_stage": "HA4 public course publication pack closure",
        "next_item": "HA5_EDUCATION_OPERATIONS_LTS_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    check_payload(CONTRACT)
    publication = read_json(PUBLICATION)
    if publication.get("status") != "education_publication_pack_ready":
        fail(f"publication status={publication.get('status')!r}")
    ids = [row.get("id") for row in publication.get("rows", [])]
    if ids != ["micro_course", "workshop_course", "four_week_course", "publication_bundle", "share_handoff"]:
        fail(f"publication rows mismatch: {ids!r}")
    for token in ["coordinate:하-4", "public_upload:false", "public_link_creation:false", "registry_publish:false", "release_execution:false", "state_hash_participation:false"]:
        if token not in str(publication.get("publication_text", "")):
            fail(f"publication text missing {token}")
    check_payload(PUBLICATION)
    for payload in [contract, publication]:
        for key, value in payload.get("false_claims", {}).items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, PUBLICATION, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 32",
            "Studio-local 초장기 계획: 10/18",
            "public_upload_claim\": true",
            "public_link_creation_claim\": true",
            "registry_publish_claim\": true",
            "release_execution_claim\": true",
            "artifact_signing_claim\": true",
            "account_permission_change_claim\": true",
            "state_hash_participation_claim\": true",
            "parser_frontdoor_change\": true",
            "grammar_claim\": true",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "education_curriculum_4_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_ha3_classroom_ui_pack_check.py"], timeout=900)
    run([sys.executable, "tests/run_roadmap_v2_ma4_seamgrim_curriculum_4_publication_pack_closure_check.py"], timeout=700)
    run([sys.executable, "tests/run_roadmap_v2_ka4_public_registry_seed_check.py"], timeout=600)
    run(["node", "tests/education_publication_pack_runner.mjs"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_status_closed()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ha4-public-course-publication-pack] OK")


if __name__ == "__main__":
    main()
