#!/usr/bin/env python3
"""Validate GEO5_AI_WORKFLOW_HARDENING_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "GEO5_AI_WORKFLOW_HARDENING_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "거-5_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "question_card_workflow_hardening_v1"
CONTRACT = PACK / "contract.detjson"
HARDENING = PACK / "question_card_workflow_hardening.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "question_card_workflow_hardening.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
UI_RUNNER = ROOT / "tests" / "question_card_workflow_hardening_runner.mjs"
PREREQ_GEO4 = ROOT / "tests" / "run_roadmap_v2_geo4_author_tool_share_check.py"
PREREQ_GEO3 = ROOT / "tests" / "run_roadmap_v2_geo3_dev_assist_ui_check.py"
PREREQ_GEO2 = ROOT / "tests" / "run_roadmap_v2_geo2_ai_output_validation_pack_check.py"
PREREQ_GEO1 = ROOT / "tests" / "run_roadmap_v2_geo1_question_card_smoke_check.py"
PREREQ_TA2 = ROOT / "tests" / "run_roadmap_v2_ta2_matrix_status_reconciliation_check.py"


def fail(message: str) -> None:
    print(f"[roadmap-v2-geo5-ai-workflow-hardening] FAIL: {message}", file=sys.stderr)
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
        HARDENING,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        UI_MODULE,
        APP,
        INDEX_HTML,
        STYLES,
        UI_RUNNER,
        PREREQ_GEO4,
        PREREQ_GEO3,
        PREREQ_GEO2,
        PREREQ_GEO1,
        PREREQ_TA2,
    ]:
        require_file(path)
    shared_tokens = [
        "GEO5_AI_WORKFLOW_HARDENING_V1",
        "GEO5 AI workflow hardening closure 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 37/90 = 41%",
        "ROADMAP_V2 pack evidence 참고값: 58/90 = 64%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_NEXT_FRONTIER_REBASE_V1",
        "release_execution:false",
        "lts_certification:false",
        "auto_apply:false",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(MATRIX, ["| 5마루 단단마루 | AI workflow hardening | approval/replay/audit | AI workflow LTS | 닫힘-동작 |"])
    require_tokens(GUIDE, ["#### 거-5", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `question_card_workflow_hardening_v1` |"])
    require_tokens(
        TRACKER,
        [
            "| 52 | `거-5` | AI workflow hardening | 닫힘-동작 |",
            "| `거-5` | AI workflow hardening | 닫힘-동작 |",
        ],
    )
    require_tokens(MANIFEST, ["| `거-5` | `question_card_workflow_hardening_v1`; UI `question_card_workflow_hardening.js`; runner `question_card_workflow_hardening_runner.mjs` |"])
    require_tokens(DEV_SURFACES, ["question_card_workflow_hardening.js", "__QUESTION_CARD_WORKFLOW_HARDENING__"])
    require_tokens(INDEX_HTML, ["id=\"question-card-workflow-hardening\"", "data-question-card-workflow-hardening"])
    require_tokens(STYLES, [".question-card-workflow-hardening", ".question-hardening-artifacts", ".question-hardening-preview"])


def check_status_closed() -> None:
    for line in read(MATRIX).splitlines():
        if "| 5마루 단단마루 | AI workflow hardening |" in line:
            if line.rstrip().split("|")[-2].strip() != "닫힘-동작":
                fail(f"거-5 status must be 닫힘-동작: {line}")
            return
    fail("missing 거-5 matrix line")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 37,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 41,
        "roadmap_v2_pack_evidence_reference_closed": 58,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 64,
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
        "pack": "question_card_workflow_hardening_v1",
        "kind": "roadmap_v2_geo5_ai_workflow_hardening_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "GEO5_AI_WORKFLOW_HARDENING_V1",
        "roadmap_coordinate": "거-5",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "geo5_matrix_status": "닫힘-동작",
        "requires_geo4_closed_behavior": True,
        "requires_geo3_closed_behavior": True,
        "requires_geo2_closed_behavior": True,
        "requires_geo1_closed_behavior": True,
        "requires_ta2_closed_behavior": True,
        "workflow_hardening_claim": True,
        "approval_gate_claim": True,
        "replay_packet_claim": True,
        "audit_trail_claim": True,
        "rollback_plan_claim": True,
        "lts_gate_claim": True,
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
        "cloud_sync_claim": False,
        "release_execution_claim": False,
        "lts_certification_claim": False,
        "grammar_claim": False,
        "current_stage": "GEO5 AI workflow hardening closure",
        "next_item": "ROADMAP_V2_NEXT_FRONTIER_REBASE_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    check_payload(CONTRACT)
    hardening = read_json(HARDENING)
    if hardening.get("status") != "question_card_workflow_hardening_ready":
        fail(f"hardening status={hardening.get('status')!r}")
    ids = [row.get("id") for row in hardening.get("rows", [])]
    if ids != ["approval_gate", "replay_packet", "audit_trail", "rollback_plan", "lts_gate"]:
        fail(f"hardening rows mismatch: {ids!r}")
    for token in [
        "question_card_workflow_hardening:question_card_workflow_hardening_v1",
        "coordinate:거-5",
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
        "cloud_sync:false",
        "release_execution:false",
        "lts_certification:false",
    ]:
        if token not in str(hardening.get("hardening_text", "")):
            fail(f"hardening text missing {token}")
    check_payload(HARDENING)
    for payload in [contract, hardening]:
        for key, value in payload.get("false_claims", {}).items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, HARDENING, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 38",
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
            "cloud_sync_claim\": true",
            "release_execution_claim\": true",
            "lts_certification_claim\": true",
            "grammar_claim\": true",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "question_card_workflow_hardening_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_geo4_author_tool_share_check.py"], timeout=900)
    run([sys.executable, "tests/run_roadmap_v2_geo3_dev_assist_ui_check.py"], timeout=900)
    run([sys.executable, "tests/run_roadmap_v2_geo2_ai_output_validation_pack_check.py"], timeout=900)
    run([sys.executable, "tests/run_roadmap_v2_geo1_question_card_smoke_check.py"], timeout=900)
    run([sys.executable, "tests/run_roadmap_v2_ta2_matrix_status_reconciliation_check.py"], timeout=700)
    run(["node", "tests/question_card_workflow_hardening_runner.mjs"], timeout=240)
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
    print("[roadmap-v2-geo5-ai-workflow-hardening] OK")


if __name__ == "__main__":
    main()
