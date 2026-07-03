#!/usr/bin/env python3
"""Validate GEO0_AI_QUESTION_CARD_SEED_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "GEO0_AI_QUESTION_CARD_SEED_RECONCILIATION_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "거-0_RECONCILIATION_REPORT_20260608.md"
OLD_REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "거-0_REPORT_20260510.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "seulgi_question_card_schema_v1"
RECONCILIATION = PACK / "roadmap_reconciliation.detjson"
SCHEMA_CHECKER = ROOT / "tests" / "run_seulgi_question_card_schema_check.py"
SEULGI_PACK_CHECKER = ROOT / "tests" / "run_seulgi_v1_pack_check.py"
GATEKEEPER_CHECKER = ROOT / "tests" / "run_seulgi_gatekeeper_pack_check.py"


def fail(message: str) -> None:
    print(f"[roadmap-v2-geo0-question-card-reconciliation] FAIL: {message}", file=sys.stderr)
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
        OLD_REPORT,
        MATRIX,
        GUIDE,
        TRACKER,
        MANIFEST,
        DEV_SUMMARY,
        PACK / "README.md",
        PACK / "valid" / "valid_code_help_question.detjson",
        RECONCILIATION,
        SCHEMA_CHECKER,
        SEULGI_PACK_CHECKER,
        GATEKEEPER_CHECKER,
    ]:
        require_file(path)
    shared_tokens = [
        "GEO0_AI_QUESTION_CARD_SEED_RECONCILIATION_V1",
        "GEO0 AI question card seed reconciliation 4/4 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 32/90 = 36%",
        "ROADMAP_V2 pack evidence 참고값: 53/90 = 59%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "닫힘-문서",
        "GEO1_QUESTION_CARD_SMOKE_V1",
        "parser_preprocessor_claim:false",
        "ai_call_claim:false",
        "patch_generation_claim:false",
        "auto_apply_claim:false",
        "state_hash_owner:false",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(MATRIX, ["| 0마루 씨앗마루 | 질문카드 설계 | ??{...}, prompt_hash, response_hash | question card schema | 닫힘-동작 |"])
    require_tokens(MATRIX, ["| 9 | 거-0 | 질문카드 schema | seulgi_question_card_schema_v1 | 닫힘-동작 |"])
    require_tokens(GUIDE, ["#### 거-0", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `seulgi_question_card_schema_v1`"])
    require_tokens(TRACKER, ["| 13 | `거-0` | 질문카드 schema | 닫힘-동작 |", "거-0_RECONCILIATION_REPORT_20260608.md"])
    require_tokens(MANIFEST, ["| `거-0` | `seulgi_question_card_schema_v1`; reconciliation `roadmap_reconciliation.detjson`"])


def check_status_closed_document() -> None:
    for line in read(MATRIX).splitlines():
        if "| 0마루 씨앗마루 | 질문카드 설계 |" in line:
            if line.rstrip().split("|")[-2].strip() != "닫힘-동작":
                fail(f"거-0 status must be 닫힘-동작: {line}")
            return
    fail("missing 거-0 matrix line")


def check_reconciliation_payload() -> None:
    payload = read_json(RECONCILIATION)
    expected = {
        "schema": "ddn.roadmap_v2.geo0.question_card_seed_reconciliation.v1",
        "work_item": "GEO0_AI_QUESTION_CARD_SEED_RECONCILIATION_V1",
        "primary_coordinate": "거-0",
        "source_pack": "seulgi_question_card_schema_v1",
        "matrix_closure_tier": "닫힘-문서",
        "matrix_behavior_closure_claim": False,
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "question_card_schema_claim": True,
        "valid_fixture_count": 3,
        "invalid_fixture_count": 6,
        "parser_preprocessor_claim": False,
        "ai_call_claim": False,
        "patch_generation_claim": False,
        "auto_apply_claim": False,
        "runtime_ast_persisted": False,
        "state_hash_owner": False,
        "current_stage": "GEO0 AI question card seed reconciliation",
        "current_stage_closed": 4,
        "current_stage_total": 4,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 32,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 36,
        "roadmap_v2_pack_evidence_reference_closed": 53,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 59,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "next_item": "GEO1_QUESTION_CARD_SMOKE_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"reconciliation {key}={payload.get(key)!r}")
    for key, value in payload.get("false_claims", {}).items():
        if value is not False:
            fail(f"false claim {key}={value!r}")


def check_fixture_counts() -> None:
    valid = sorted((PACK / "valid").glob("*.detjson"))
    invalid = sorted((PACK / "invalid").glob("*.detjson"))
    if len(valid) != 3:
        fail(f"valid fixture count mismatch: {len(valid)}")
    if len(invalid) != 6:
        fail(f"invalid fixture count mismatch: {len(invalid)}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, RECONCILIATION, PACK / "README.md"]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "ROADMAP_V2 행렬 닫힘-동작: 33/90",
            "roadmap_v2_matrix_behavior_closed\": 33",
            "Studio-local 초장기 계획: 10/18",
            "parser_preprocessor_claim\": true",
            "ai_call_claim\": true",
            "patch_generation_claim\": true",
            "auto_apply_claim\": true",
            "runtime_ast_persisted\": true",
            "state_hash_owner\": true",
            "product_ui_change\": true",
            "product_code_change\": true",
            "runtime_claim\": true",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_seulgi_question_card_schema_check.py"], timeout=120)
    run(
        [
            sys.executable,
            "tests/run_seulgi_question_card_schema_check.py",
            "--file",
            "pack/seulgi_question_card_schema_v1/valid/valid_code_help_question.detjson",
        ],
        timeout=120,
    )
    run(
        [
            sys.executable,
            "tests/run_seulgi_question_card_schema_check.py",
            "--dir",
            "pack/seulgi_question_card_schema_v1/valid",
        ],
        timeout=120,
    )
    run([sys.executable, "tests/run_seulgi_v1_pack_check.py"], timeout=240)
    run([sys.executable, "tests/run_seulgi_gatekeeper_pack_check.py"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_status_closed_document()
    check_reconciliation_payload()
    check_fixture_counts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-geo0-question-card-reconciliation] OK")


if __name__ == "__main__":
    main()
