#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "ganada_ja_seulgi_context_v1"
CONTEXT = PACK / "context.detjson"
PROPOSAL_UI = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "seulgi_proposal_ui.js"
REPLAY_UI = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "seulgi_replay_safe_workflow.js"
RUNNER = ROOT / "tests" / "ganada_ja_seulgi_context_runner.mjs"


def fail(message: str) -> None:
    print(f"ganada_ja_seulgi_context_check: FAIL: {message}", file=sys.stderr)
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
        "schema": "ddn.ganada.ja.seulgi_context.v1",
        "pack": "ganada_ja_seulgi_context_v1",
        "cells": ["자-3", "자-5"],
        "status": "product_pass_gate",
        "behavior_closed_claim": False,
        "matrix_upgrade_claim": False,
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
    context = payload.get("context")
    if not isinstance(context, dict):
        fail("context must be object")
    for key, value in {
        "lesson_context": "seamgrim_lesson_run_rail_v1",
        "editor_context": "seamgrim_lesson_authoring_flow_v1",
        "replay_context": "lesson_editor_fixture",
        "fixture_proposal_import": True,
        "product_runner": "tests/ganada_ja_seulgi_context_runner.mjs",
    }.items():
        if context.get(key) != value:
            fail(f"context.{key}={context.get(key)!r}, expected {value!r}")


def check_static_ui_tokens() -> None:
    proposal_text = PROPOSAL_UI.read_text(encoding="utf-8-sig")
    replay_text = REPLAY_UI.read_text(encoding="utf-8-sig")
    for token in [
        "lesson_context:seamgrim_lesson_run_rail_v1",
        "editor_context:seamgrim_lesson_authoring_flow_v1",
        "fixture_proposal_import:true",
        'data-testid="seulgi-preview"',
        'data-testid="seulgi-fixture-import"',
    ]:
        if token not in proposal_text:
            fail(f"proposal UI missing token: {token}")
    for token in [
        "proposal_receipt_source:seulgi_proposal_ui_v1",
        "replay_context:lesson_editor_fixture",
        "receipt_replay_policy:local_append_only",
        'data-testid", "seulgi-replay-panel"',
    ]:
        if token not in replay_text:
            fail(f"replay UI missing token: {token}")


def check_commands() -> None:
    commands = [
        (["python", "tests/run_pack_golden.py", "ganada_ja_seulgi_context_v1"], "pack golden ok"),
        (["node", "tests/seulgi_proposal_ui_runner.mjs"], "seulgi_proposal_ui: ok"),
        (["node", "tests/seulgi_replay_safe_workflow_runner.mjs"], "seulgi_replay_safe_workflow: ok"),
        (["node", "tests/ganada_ja_seulgi_context_runner.mjs"], "ganada_ja_seulgi_context: ok"),
    ]
    for cmd, marker in commands:
        proc = run(cmd, timeout=900)
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
    payload = read_json(CONTEXT)
    check_shape(payload)
    check_static_ui_tokens()
    check_commands()
    check_boundaries()
    print("ganada_ja_seulgi_context_check: PASS")


if __name__ == "__main__":
    main()
