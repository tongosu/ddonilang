#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "ganada_ja_seulgi_local_workflow_v1"
RECEIPT = PACK / "receipt.detjson"
PROPOSAL_UI = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "seulgi_proposal_ui.js"
REPLAY_UI = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "seulgi_replay_safe_workflow.js"
RUNNER = ROOT / "tests" / "ganada_ja_seulgi_local_workflow_runner.mjs"


def fail(message: str) -> None:
    print(f"ganada_ja_seulgi_local_workflow_check: FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def run(cmd: list[str], *, timeout: int = 600) -> subprocess.CompletedProcess[str]:
    normalized = [sys.executable if item == "python" else item for item in cmd]
    return subprocess.run(
        normalized,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
        check=False,
    )


def read_json(path: Path) -> dict:
    try:
        payload = json.loads(path.read_text(encoding="utf-8-sig"))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be a JSON object")
    return payload


def check_shape(payload: dict) -> None:
    expected = {
        "schema": "ddn.ganada.ja.seulgi_local_workflow.v1",
        "pack": "ganada_ja_seulgi_local_workflow_v1",
        "cells": ["자-3", "자-5"],
        "status": "product_pass_gate",
        "behavior_closed_claim": False,
        "matrix_upgrade_claim": False,
        "fixture_proposal_import": True,
        "receipt_policy": "approve_reject_local",
        "receipt_replay_policy": "local_append_only",
        "ai_call_claim": False,
        "auto_apply_claim": False,
        "file_write_claim": False,
        "runtime_ast_persistence_claim": False,
        "docs_ssot_change": False,
        "parser_change": False,
        "runtime_change": False,
        "state_hash_change": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"{key}={payload.get(key)!r}, expected {value!r}")
    provenance = payload.get("provenance")
    if not isinstance(provenance, dict):
        fail("provenance must be object")
    for key, value in {
        "lesson_context": "seamgrim_lesson_run_rail_v1",
        "editor_context": "seamgrim_lesson_authoring_flow_v1",
        "proposal_pack": "seulgi_proposal_ui_v1",
        "replay_pack": "seulgi_replay_safe_workflow_v1",
        "product_runner": "tests/ganada_ja_seulgi_local_workflow_runner.mjs",
    }.items():
        if provenance.get(key) != value:
            fail(f"provenance.{key}={provenance.get(key)!r}, expected {value!r}")
    receipts = payload.get("receipts")
    if not isinstance(receipts, list) or [row.get("id") for row in receipts] != ["approve_receipt", "reject_receipt", "replay_log"]:
        fail("receipt row order mismatch")
    states = " ".join(str(row.get("recorded_state", "")) for row in receipts)
    for token in [
        "data-seulgi-approval-receipt=local",
        "data-seulgi-rejection-receipt=local",
        "proposal_receipt_source:seulgi_proposal_ui_v1",
    ]:
        if token not in states:
            fail(f"missing receipt state: {token}")


def check_static_ui_tokens() -> None:
    proposal = PROPOSAL_UI.read_text(encoding="utf-8-sig")
    replay = REPLAY_UI.read_text(encoding="utf-8-sig")
    for token in [
        "receipt_policy:approve_reject_local",
        "seulgiApprovalReceipt",
        "seulgiRejectionReceipt",
        'data-testid="seulgi-fixture-import"',
    ]:
        if token not in proposal:
            fail(f"proposal UI missing token: {token}")
    for token in [
        "proposal_receipt_source:seulgi_proposal_ui_v1",
        "receipt_replay_policy:local_append_only",
        'data-testid="seulgi-replay-log"',
    ]:
        if token not in replay:
            fail(f"replay UI missing token: {token}")


def check_commands() -> None:
    commands = [
        (["python", "tests/run_pack_golden.py", "ganada_ja_seulgi_local_workflow_v1"], "pack golden ok"),
        (["node", "tests/ganada_ja_seulgi_local_workflow_runner.mjs"], "ganada_ja_seulgi_local_workflow: ok"),
        (["python", "tests/run_ganada_ja_seulgi_context_check.py"], "ganada_ja_seulgi_context_check: PASS"),
        (["python", "tests/run_roadmap_v2_ja3_seulgi_proposal_ui_check.py"], "[roadmap-v2-ja3-seulgi-proposal-ui] OK"),
        (["python", "tests/run_roadmap_v2_ja5_replay_safe_ai_workflow_check.py"], "[roadmap-v2-ja5-replay-safe-ai-workflow] OK"),
    ]
    for cmd, marker in commands:
        proc = run(cmd, timeout=1200)
        if proc.returncode != 0:
            fail(f"command failed {cmd}:\n{proc.stdout}")
        if marker not in proc.stdout:
            fail(f"marker {marker!r} missing for {cmd}:\n{proc.stdout}")


def check_boundaries() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed:\n{proc.stdout}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    if not RUNNER.is_file():
        fail(f"missing runner: {RUNNER.relative_to(ROOT)}")
    payload = read_json(RECEIPT)
    check_shape(payload)
    check_static_ui_tokens()
    check_commands()
    check_boundaries()
    print("ganada_ja_seulgi_local_workflow_check: PASS")


if __name__ == "__main__":
    main()
