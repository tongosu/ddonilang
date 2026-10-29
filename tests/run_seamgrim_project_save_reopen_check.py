#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_project_save_reopen_v0"
RUNNER = ROOT / "tests" / "seamgrim_project_save_reopen_runner.mjs"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
INDEX = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    return json.loads(read(path))


def run(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def main() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "golden.jsonl",
        PACK / "input.ddn",
        RUNNER,
        APP_JS,
        INDEX,
        STYLE,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_PROJECT_SAVE_REOPEN_MISSING", repr(missing))
    contract = read_json(PACK / "contract.detjson")
    if contract.get("pack") != "seamgrim_project_save_reopen_v0":
        return fail("E_PROJECT_SAVE_REOPEN_PACK", repr(contract))
    if contract.get("behavior_closed_claim") is not True:
        return fail("E_PROJECT_SAVE_REOPEN_CLAIM", repr(contract))
    for key in ("docs_ssot_change", "new_ddn_syntax", "state_hash_formula_change", "server_login_cloud_claim"):
        if contract.get(key) is not False:
            return fail("E_PROJECT_SAVE_REOPEN_BOUNDARY", f"{key}={contract.get(key)!r}")

    app_text = read(APP_JS)
    index_text = read(INDEX)
    style_text = read(STYLE)
    for token in [
        "btn-project-save",
        "project-name-input",
        "project-edit-link",
        "project-recent-list",
    ]:
        if token not in index_text:
            return fail("E_PROJECT_SAVE_REOPEN_INDEX_TOKEN", token)
    for token in [
        "saveProjectFromEditor",
        "openProjectFromCapabilitySession",
        "requestProjectApi",
        "PROJECT_RECENT_STORAGE_KEY",
        "PROJECT_CAPABILITY_SESSION_KEY",
        "buildCurrentProjectSnapshotArtifact",
        "editorScreen.loadLesson",
    ]:
        if token not in app_text:
            return fail("E_PROJECT_SAVE_REOPEN_APP_TOKEN", token)
    for token in [
        ".project-save-panel",
        ".project-save-name-input",
        ".project-recent-item",
        ".project-save-status[data-state=\"saved\"]",
    ]:
        if token not in style_text:
            return fail("E_PROJECT_SAVE_REOPEN_STYLE_TOKEN", token)

    node_check = run(["node", "--check", str(RUNNER.relative_to(ROOT))], timeout=60)
    if node_check.returncode != 0:
        return fail("E_PROJECT_SAVE_REOPEN_NODE_CHECK", node_check.stdout.strip())
    runner = run(["node", str(RUNNER.relative_to(ROOT))], timeout=240)
    if runner.returncode != 0:
        return fail("E_PROJECT_SAVE_REOPEN_RUNNER", runner.stdout.strip())
    result = json.loads([line for line in runner.stdout.splitlines() if line.strip()][-1])
    for key in (
        "reopened_fragment_scrubbed",
        "reopened_ddn_preserved",
        "reopened_lesson_plan_preserved",
        "run_after_reopen",
    ):
        if result.get(key) is not True:
            return fail("E_PROJECT_SAVE_REOPEN_RESULT", repr(result))

    golden = run([sys.executable, "tests/run_pack_golden.py", "seamgrim_project_save_reopen_v0"], timeout=180)
    if golden.returncode != 0:
        return fail("E_PROJECT_SAVE_REOPEN_GOLDEN", golden.stdout.strip())
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode != 0 or ssot.stdout.strip():
        return fail("E_PROJECT_SAVE_REOPEN_SSOT_DIRTY", ssot.stdout.strip())
    print("[seamgrim-project-save-reopen-v0] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
