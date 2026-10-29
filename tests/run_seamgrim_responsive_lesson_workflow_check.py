#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_responsive_lesson_workflow_v1"
RUNNER = ROOT / "tests" / "seamgrim_responsive_lesson_workflow_runner.mjs"
RUN_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"
EDITOR_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "editor.js"
STYLE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"

try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except Exception:
    pass


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8-sig")


def run(cmd: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
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


def check_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        RUN_JS,
        EDITOR_JS,
        STYLE,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_RESPONSIVE_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("pack") != "seamgrim_responsive_lesson_workflow_v1":
        return fail("E_SEAMGRIM_RESPONSIVE_PACK", repr(contract))
    if contract.get("viewports") != ["1440x900", "1280x720", "1024x768", "390x844"]:
        return fail("E_SEAMGRIM_RESPONSIVE_VIEWPORTS", repr(contract.get("viewports")))
    if "selected_graph_inside_visible_graph_host" not in contract.get("checks", []):
        return fail("E_SEAMGRIM_RESPONSIVE_GRAPH_VISIBILITY", repr(contract.get("checks")))
    for key in ("docs_ssot_change", "state_hash_formula_change", "parser_runtime_semantics_change", "public_deploy_claim"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_RESPONSIVE_BOUNDARY", f"{key}={contract.get(key)!r}")
    return 0


def check_runner_tokens() -> int:
    runner = read(RUNNER)
    for token in [
        "1440",
        "1024",
        "390",
        "canvasEvidence",
        "liveTraceReactionFiredActions",
        "runLessonPlanActivities",
        "verifyEditorRoundtrip",
        "verifyPublishSpace",
        "first_open_ms",
        "first_compare_semantic_ms",
    ]:
        if token not in runner:
            return fail("E_SEAMGRIM_RESPONSIVE_RUNNER_TOKEN", token)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_RESPONSIVE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_RESPONSIVE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_contract, check_runner_tokens):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "tests/seamgrim_responsive_lesson_workflow_runner.mjs"], "E_SEAMGRIM_RESPONSIVE_NODE_CHECK", 60),
        (["node", "tests/seamgrim_responsive_lesson_workflow_runner.mjs"], "E_SEAMGRIM_RESPONSIVE_BROWSER", 420),
        (["python", "tests/run_pack_golden.py", "seamgrim_responsive_lesson_workflow_v1"], "E_SEAMGRIM_RESPONSIVE_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-responsive-lesson-workflow-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
