#!/usr/bin/env python3
"""Validate GEO3_DEV_ASSIST_UI_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "GEO3_DEV_ASSIST_UI_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "거-3_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "question_card_dev_assist_v1"
CONTRACT = PACK / "contract.detjson"
ASSIST = PACK / "question_card_dev_assist.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "question_card_dev_assist.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
UI_RUNNER = ROOT / "tests" / "question_card_dev_assist_runner.mjs"
PREREQ_GEO2 = ROOT / "tests" / "run_roadmap_v2_geo2_ai_output_validation_pack_check.py"
PREREQ_GEO1 = ROOT / "tests" / "run_roadmap_v2_geo1_question_card_smoke_check.py"
PREREQ_TA2 = ROOT / "tests" / "run_roadmap_v2_ta2_matrix_status_reconciliation_check.py"


def fail(message: str) -> None:
    print(f"[roadmap-v2-geo3-dev-assist-ui] FAIL: {message}", file=sys.stderr)
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
        PROJECT_STATUS,
        CHANGELOG,
        PACK / "README.md",
        CONTRACT,
        ASSIST,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        UI_MODULE,
        APP,
        INDEX_HTML,
        STYLES,
        UI_RUNNER,
        PREREQ_GEO2,
        PREREQ_GEO1,
        PREREQ_TA2,
    ]:
        require_file(path)
    shared_tokens = [
        "GEO3_DEV_ASSIST_UI_V1",
        "GEO3 dev assist UI closure 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 35/90 = 39%",
        "ROADMAP_V2 pack evidence 참고값: 56/90 = 62%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "GEO4_AUTHOR_TOOL_SHARE_V1",
        "AI call",
        "file_write:false",
        "registry_publish:false",
        "state_hash_owner:false",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(MATRIX, ["| 3마루 작업실마루 | 개발보조 UI | Codex 일감/교재 초안/보고서 | authoring UI pack | 닫힘-동작 |"])
    require_tokens(GUIDE, ["#### 거-3", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `question_card_dev_assist_v1` |"])
    require_tokens(
        TRACKER,
        [
            "| 50 | `거-3` | 개발보조 UI | 닫힘-동작 |",
            "| `거-3` | development assist UI | 닫힘-동작 |",
        ],
    )
    require_tokens(MANIFEST, ["| `거-3` | `question_card_dev_assist_v1`; UI `question_card_dev_assist.js`; runner `question_card_dev_assist_runner.mjs` |"])
    require_tokens(DEV_SURFACES, ["question_card_dev_assist.js", "__QUESTION_CARD_DEV_ASSIST__"])
    require_tokens(INDEX_HTML, ["id=\"question-card-dev-assist\"", "data-question-card-dev-assist"])
    require_tokens(STYLES, [".question-card-dev-assist", ".question-assist-artifacts", ".question-assist-preview"])


def check_status_closed() -> None:
    for line in read(MATRIX).splitlines():
        if "| 3마루 작업실마루 | 개발보조 UI |" in line:
            if line.rstrip().split("|")[-2].strip() != "닫힘-동작":
                fail(f"거-3 status must be 닫힘-동작: {line}")
            return
    fail("missing 거-3 matrix line")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 35,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 39,
        "roadmap_v2_pack_evidence_reference_closed": 56,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 62,
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
        "pack": "question_card_dev_assist_v1",
        "kind": "roadmap_v2_geo3_dev_assist_ui_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "GEO3_DEV_ASSIST_UI_V1",
        "roadmap_coordinate": "거-3",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "geo3_matrix_status": "닫힘-동작",
        "requires_geo2_closed_behavior": True,
        "requires_geo1_closed_behavior": True,
        "requires_ta2_closed_behavior": True,
        "dev_assist_ui_claim": True,
        "codex_work_item_claim": True,
        "lesson_draft_claim": True,
        "report_draft_claim": True,
        "review_queue_claim": True,
        "handoff_receipt_claim": True,
        "ai_call_claim": False,
        "parser_preprocessor_claim": False,
        "auto_apply_claim": False,
        "patch_execution_claim": False,
        "file_write_claim": False,
        "network_call_claim": False,
        "runtime_ast_persisted": False,
        "state_hash_owner": False,
        "registry_publish_claim": False,
        "account_permission_change_claim": False,
        "grammar_claim": False,
        "current_stage": "GEO3 dev assist UI closure",
        "next_item": "GEO4_AUTHOR_TOOL_SHARE_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    check_payload(CONTRACT)
    assist = read_json(ASSIST)
    if assist.get("status") != "question_card_dev_assist_ready":
        fail(f"assist status={assist.get('status')!r}")
    ids = [row.get("id") for row in assist.get("rows", [])]
    if ids != ["codex_work_item", "lesson_draft", "report_draft", "review_queue", "handoff_receipt"]:
        fail(f"assist rows mismatch: {ids!r}")
    for token in [
        "question_card_dev_assist:question_card_dev_assist_v1",
        "coordinate:거-3",
        "ai_call:false",
        "parser_preprocessor:false",
        "auto_apply:false",
        "patch_execution:false",
        "file_write:false",
        "network_call:false",
        "runtime_ast_persisted:false",
        "state_hash_owner:false",
        "registry_publish:false",
        "account_permission_change:false",
    ]:
        if token not in str(assist.get("assist_text", "")):
            fail(f"assist text missing {token}")
    check_payload(ASSIST)
    for payload in [contract, assist]:
        for key, value in payload.get("false_claims", {}).items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, ASSIST, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 36",
            "Studio-local 초장기 계획: 10/18",
            "ai_call_claim\": true",
            "parser_preprocessor_claim\": true",
            "auto_apply_claim\": true",
            "patch_execution_claim\": true",
            "file_write_claim\": true",
            "network_call_claim\": true",
            "runtime_ast_persisted\": true",
            "state_hash_owner\": true",
            "registry_publish_claim\": true",
            "account_permission_change_claim\": true",
            "grammar_claim\": true",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "question_card_dev_assist_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_geo2_ai_output_validation_pack_check.py"], timeout=900)
    run([sys.executable, "tests/run_roadmap_v2_geo1_question_card_smoke_check.py"], timeout=900)
    run([sys.executable, "tests/run_roadmap_v2_ta2_matrix_status_reconciliation_check.py"], timeout=700)
    run(["node", "tests/question_card_dev_assist_runner.mjs"], timeout=240)
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
    print("[roadmap-v2-geo3-dev-assist-ui] OK")


if __name__ == "__main__":
    main()
