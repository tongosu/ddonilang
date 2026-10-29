#!/usr/bin/env python3
"""Validate Seamgrim Wave2E G5 view-link share and clone workflow."""

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
    print(f"[seamgrim-project-share-clone-v0] FAIL {code}{': ' + detail if detail else ''}")
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
            "project-view-link",
            "btn-project-clone",
            "내 버전 만들기",
        ],
        "solutions/seamgrim_ui_mvp/ui/app.js": [
            "updateProjectViewLink",
            "cloneProjectFromSession",
            "view_capability",
            "보기 링크에서는 원본을 저장할 수 없습니다",
            "window.__SEAMGRIM_LAST_PROJECT_CLONE__",
        ],
        "solutions/seamgrim_ui_mvp/tools/project_store.py": [
            "origin_content_sha256",
            "origin_kind",
        ],
        "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py": [
            "\"origin_revision_id\"",
            "\"origin_content_sha256\"",
            "action == \"clone\"",
        ],
        "tests/seamgrim_project_share_clone_runner.mjs": [
            "view_readonly",
            "clone_project_distinct",
            "origin_head_unchanged",
            "clone_lineage_recorded",
        ],
    }
    for path, needles in static_checks.items():
        error = require_text(path, needles)
        if error:
            return fail("E_PROJECT_SHARE_CLONE_STATIC", error)

    check = run(["node", "--check", "solutions/seamgrim_ui_mvp/ui/app.js"], timeout=60)
    if check.returncode != 0:
        return fail("E_PROJECT_SHARE_CLONE_APP_SYNTAX", check.stdout.strip())
    runner_check = run(["node", "--check", "tests/seamgrim_project_share_clone_runner.mjs"], timeout=60)
    if runner_check.returncode != 0:
        return fail("E_PROJECT_SHARE_CLONE_RUNNER_SYNTAX", runner_check.stdout.strip())
    py_check = run([
        sys.executable,
        "-m",
        "py_compile",
        "solutions/seamgrim_ui_mvp/tools/project_store.py",
        "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py",
    ], timeout=60)
    if py_check.returncode != 0:
        return fail("E_PROJECT_SHARE_CLONE_PY_SYNTAX", py_check.stdout.strip())

    runner = run(["node", "tests/seamgrim_project_share_clone_runner.mjs"], timeout=180)
    if runner.returncode != 0:
        return fail("E_PROJECT_SHARE_CLONE_RUNNER", runner.stdout.strip())
    last_line = runner.stdout.strip().splitlines()[-1]
    try:
      result = json.loads(last_line)
    except json.JSONDecodeError as exc:
      return fail("E_PROJECT_SHARE_CLONE_RUNNER_JSON", f"{exc}: {last_line}")
    expected_true = [
        "view_readonly",
        "clone_project_distinct",
        "origin_head_unchanged",
        "origin_content_unchanged",
        "clone_lineage_recorded",
    ]
    for key in expected_true:
        if result.get(key) is not True:
            return fail("E_PROJECT_SHARE_CLONE_RESULT", f"{key}={result.get(key)!r}")

    golden = run(["python", "tests/run_pack_golden.py", "seamgrim_project_share_clone_v0"], timeout=120)
    if golden.returncode != 0:
        return fail("E_PROJECT_SHARE_CLONE_GOLDEN", golden.stdout.strip())
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode != 0 or ssot.stdout.strip():
        return fail("E_PROJECT_SHARE_CLONE_SSOT_DIRTY", ssot.stdout.strip())
    print("[seamgrim-project-share-clone-v0] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
