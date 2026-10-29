#!/usr/bin/env python3
"""Validate Seamgrim Wave2E G6 revision conflict and restore workflow."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def run(cmd: list[str], timeout: int = 120) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def fail(code: str, detail: str = "") -> int:
    print(f"[seamgrim-project-revision-conflict-restore-v0] FAIL {code}{': ' + detail if detail else ''}")
    return 1


def require_text(path: str, needles: list[str]) -> str | None:
    text = (ROOT / path).read_text(encoding="utf-8")
    missing = [needle for needle in needles if needle not in text]
    if missing:
        return f"{path} missing {missing!r}"
    return None


def main() -> int:
    static_checks = {
        "solutions/seamgrim_ui_mvp/ui/index.html": [
            "project-conflict-panel",
            "project-revision-list",
            "btn-project-revisions-refresh",
            "btn-project-conflict-copy-local",
        ],
        "solutions/seamgrim_ui_mvp/ui/app.js": [
            "setProjectConflictPanel",
            "loadProjectRevisions",
            "restoreProjectRevision",
            "expected_parent_revision_id: appState.shell.currentRevisionId",
            "restore_of_revision_id",
            "로컬 꾸러미 보존됨",
        ],
        "tests/seamgrim_project_revision_conflict_restore_runner.mjs": [
            "conflict_detected",
            "local_package_preserved",
            "restore_created_new_revision",
            "restore_parent_is_previous_head",
            "audit_fields_visible",
        ],
    }
    for path, needles in static_checks.items():
        error = require_text(path, needles)
        if error:
            return fail("E_PROJECT_REVISION_STATIC", error)

    checks = [
        ["node", "--check", "solutions/seamgrim_ui_mvp/ui/app.js"],
        ["node", "--check", "tests/seamgrim_project_revision_conflict_restore_runner.mjs"],
        [
            sys.executable,
            "-m",
            "py_compile",
            "solutions/seamgrim_ui_mvp/tools/project_store.py",
            "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py",
        ],
    ]
    for cmd in checks:
        proc = run(cmd, timeout=60)
        if proc.returncode != 0:
            return fail("E_PROJECT_REVISION_SYNTAX", f"{cmd!r}: {proc.stdout.strip()}")

    runner = run(["node", "tests/seamgrim_project_revision_conflict_restore_runner.mjs"], timeout=180)
    if runner.returncode != 0:
        return fail("E_PROJECT_REVISION_RUNNER", runner.stdout.strip())
    last_line = runner.stdout.strip().splitlines()[-1]
    try:
        result = json.loads(last_line)
    except json.JSONDecodeError as exc:
        return fail("E_PROJECT_REVISION_RUNNER_JSON", f"{exc}: {last_line}")
    for key in [
        "conflict_detected",
        "local_package_preserved",
        "restore_created_new_revision",
        "restore_parent_is_previous_head",
        "restore_source_recorded",
        "audit_fields_visible",
    ]:
        if result.get(key) is not True:
            return fail("E_PROJECT_REVISION_RESULT", f"{key}={result.get(key)!r}")

    golden = run(["python", "tests/run_pack_golden.py", "seamgrim_project_revision_conflict_restore_v0"], timeout=120)
    if golden.returncode != 0:
        return fail("E_PROJECT_REVISION_GOLDEN", golden.stdout.strip())
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode != 0 or ssot.stdout.strip():
        return fail("E_PROJECT_REVISION_SSOT_DIRTY", ssot.stdout.strip())
    print("[seamgrim-project-revision-conflict-restore-v0] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
