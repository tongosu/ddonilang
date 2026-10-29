#!/usr/bin/env python3
"""Validate JA3_SEULGI_PROPOSAL_UI_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "자-3_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "seulgi_proposal_ui_v1"
CONTRACT = PACK / "contract.detjson"
PAYLOAD = PACK / "seulgi_proposal_ui.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "seulgi_proposal_ui.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
UI_RUNNER = ROOT / "tests" / "seulgi_proposal_ui_runner.mjs"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ja3-seulgi-proposal-ui] FAIL: {message}", file=sys.stderr)
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
        PAYLOAD,
        UI_MODULE,
        APP,
        INDEX_HTML,
        STYLES,
        UI_RUNNER,
        ROOT / "tests" / "run_roadmap_v2_ja_seulgi_boundary_reconciliation_check.py",
    ]:
        require_file(path)
    require_tokens(MATRIX, ["| 3마루 작업실마루 | AI 제안 UI | 슬기제안/승인/거절 | AI support UI pack | 닫힘-동작 |"])
    require_tokens(TRACKER, ["| 12.85 | `자-3` | AI 제안 UI | 닫힘-동작 |", "자-3_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `자-3` | `seulgi_proposal_ui_v1`; UI `seulgi_proposal_ui.js`; runner `seulgi_proposal_ui_runner.mjs` |"])
    require_tokens(DEV_SURFACES, ["seulgi_proposal_ui.js", "__SEULGI_PROPOSAL_UI__"])
    require_tokens(INDEX_HTML, ["id=\"seulgi-proposal-ui\"", "data-seulgi-proposal-ui"])
    require_tokens(STYLES, [".seulgi-proposal-ui"])
    shared = [
        "JA3_SEULGI_PROPOSAL_UI_V1",
        "JA3 Seulgi proposal UI 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 74/90 = 82%",
        "ROADMAP_V2 docs-closed: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 76/90 = 84%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_JA5_REPLAY_SAFE_AI_WORKFLOW_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 JA3 Seulgi proposal UI closure", "74/90 = 82%", "76/90 = 84%"])
    total, behavior, docs = count_matrix_statuses()
    if total != 90 or behavior < 74 or docs > 5:
        fail(f"matrix counts mismatch: rows={total} behavior={behavior} docs={docs}")


def check_payload() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seulgi_proposal_ui_v1",
        "kind": "roadmap_v2_ja3_seulgi_proposal_ui",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "JA3_SEULGI_PROPOSAL_UI_V1",
        "roadmap_coordinate": "자-3",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 74,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 82,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 76,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 84,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "requires_ja_boundary_prereq": True,
        "browser_runner": "tests/seulgi_proposal_ui_runner.mjs",
        "ui_module": "solutions/seamgrim_ui_mvp/ui/seulgi_proposal_ui.js",
        "next_item": "ROADMAP_V2_JA5_REPLAY_SAFE_AI_WORKFLOW_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    payload = read_json(PAYLOAD)
    if payload.get("schema") != "ddn.ja.seulgi_proposal_ui.v1":
        fail("payload schema mismatch")
    if payload.get("status") != "seulgi_proposal_ui_ready":
        fail("payload status mismatch")
    row_ids = [row.get("id") for row in payload.get("rows", [])]
    if row_ids != ["proposal", "preview", "approve", "reject", "audit"]:
        fail(f"payload rows mismatch: {row_ids!r}")
    progress = payload.get("progress", {})
    for key in [
        "current_stage_closed",
        "roadmap_v2_matrix_behavior_closed",
        "roadmap_v2_docs_closed",
        "roadmap_v2_pack_evidence_reference_closed",
        "studio_local_super_long_closed",
    ]:
        if progress.get(key) != expected[key]:
            fail(f"payload progress {key}={progress.get(key)!r}")
    for item in [contract, payload]:
        for key, value in item.get("false_claims", {}).items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 75/90",
        '"roadmap_v2_matrix_behavior_closed": 75',
        '"ai_call_claim": true',
        '"auto_apply_claim": true',
        '"file_write_claim": true',
        '"parser_preprocessor_claim": true',
        '"runtime_ast_persisted": true',
        '"state_hash_owner": true',
        '"model_training_claim": true',
        '"account_permission_change_claim": true',
        '"runtime_claim": true',
    ]
    for path in [REPORT, PACK / "README.md", CONTRACT, PAYLOAD, UI_MODULE]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "seulgi_proposal_ui_v1"], timeout=120)
    run(["node", "tests/seulgi_proposal_ui_runner.mjs"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_ja_seulgi_boundary_reconciliation_check.py"], timeout=600)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_payload()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ja3-seulgi-proposal-ui] OK")


if __name__ == "__main__":
    main()
