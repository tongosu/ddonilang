#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_project_snapshot_v0"
SNAPSHOT_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "project_snapshot.js"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
RUNNER = ROOT / "tests" / "seamgrim_project_snapshot_runner.mjs"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    return json.loads(read(path))


def run(cmd: list[str], *, timeout: int = 120) -> subprocess.CompletedProcess[str]:
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
        SNAPSHOT_JS,
        APP_JS,
        RUNNER,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_PROJECT_SNAPSHOT_MISSING", repr(missing))

    contract = read_json(PACK / "contract.detjson")
    if contract.get("pack") != "seamgrim_project_snapshot_v0":
        return fail("E_PROJECT_SNAPSHOT_PACK", repr(contract))
    if contract.get("behavior_closed_claim") is not True:
        return fail("E_PROJECT_SNAPSHOT_CLAIM", repr(contract))
    for key in ("docs_ssot_change", "new_ddn_syntax", "state_hash_formula_change", "server_login_cloud_claim"):
        if contract.get(key) is not False:
            return fail("E_PROJECT_SNAPSHOT_BOUNDARY", f"{key}={contract.get(key)!r}")

    snapshot_js = read(SNAPSHOT_JS)
    app_js = read(APP_JS)
    for token in [
        'PROJECT_SNAPSHOT_SCHEMA = "seamgrim.project_snapshot.v0"',
        "stableProjectSnapshotJson",
        "sha256HexUtf8",
        "buildProjectSnapshotInputFromEditorState",
        "server_executes_ddn_for_save: false",
        "state_hash_participation: false",
    ]:
        if token not in snapshot_js:
            return fail("E_PROJECT_SNAPSHOT_JS_TOKEN", token)
    for token in [
        "buildProjectSnapshotArtifactV0",
        "buildProjectSnapshotInputFromEditorState",
        "buildCurrentProjectSnapshotArtifact",
        "__SEAMGRIM_BUILD_CURRENT_PROJECT_SNAPSHOT__",
        "editorScreen.getDdn()",
        "editorScreen.getLessonPlanText()",
        "buildStudioLocalPackagePayload",
    ]:
        if token not in app_js:
            return fail("E_PROJECT_SNAPSHOT_APP_TOKEN", token)

    node_check = run(["node", "--check", str(RUNNER.relative_to(ROOT))], timeout=60)
    if node_check.returncode != 0:
        return fail("E_PROJECT_SNAPSHOT_NODE_CHECK", node_check.stdout.strip())
    runner = run(["node", str(RUNNER.relative_to(ROOT))], timeout=120)
    if runner.returncode != 0:
        return fail("E_PROJECT_SNAPSHOT_RUNNER", runner.stdout.strip())
    last_line = [line for line in runner.stdout.splitlines() if line.strip()][-1]
    result = json.loads(last_line)
    if result.get("schema") != "seamgrim.project_snapshot.runner_result.v1":
        return fail("E_PROJECT_SNAPSHOT_RUNNER_SCHEMA", repr(result))
    if result.get("edited_value_beats_stale_currentLesson") is not True:
        return fail("E_PROJECT_SNAPSHOT_STALE_EDITOR", repr(result))
    if len(str(result.get("content_sha256", ""))) != 64:
        return fail("E_PROJECT_SNAPSHOT_SHA", repr(result))

    golden = run([sys.executable, "tests/run_pack_golden.py", "seamgrim_project_snapshot_v0"], timeout=180)
    if golden.returncode != 0:
        return fail("E_PROJECT_SNAPSHOT_GOLDEN", golden.stdout.strip())
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode != 0 or ssot.stdout.strip():
        return fail("E_PROJECT_SNAPSHOT_SSOT_DIRTY", ssot.stdout.strip())
    print("[seamgrim-project-snapshot-v0] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
