#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "tests" / "seamgrim_ui_age3_gate_runner.mjs"

try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except Exception:
    pass


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


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


def check_product_contract_tokens() -> int:
    required = [
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "browse.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "components" / "slider_panel.js",
        RUNNER,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_UI_AGE3_MISSING_FILE", repr(missing))

    html = read(required[0])
    app = read(required[1])
    browse = read(required[2])
    run_js = read(required[3])
    styles = read(required[4])
    slider = read(required[5])

    checks: dict[str, list[tuple[str, str]]] = {
        "four_spaces": [
            ("html", 'data-shell-space="learn"'),
            ("html", 'data-shell-space="lab"'),
            ("html", 'data-shell-space="create"'),
            ("html", 'data-shell-space="publish"'),
            ("app", 'const MAIN_TAB_BROWSE = "browse"'),
            ("app", 'const MAIN_TAB_STUDIO = "studio"'),
            ("app", 'const MAIN_TAB_PUBLISH = "publish"'),
        ],
        "current_open_and_lab_actions": [
            ("browse", "수업 열기로 실행 화면을 연다"),
            ("browse", "실험실에서 바꾸기"),
            ("app", "const shellSpace = String(selection?.launchAction ?? \"learn\").trim() === \"lab\" ? \"lab\" : \"learn\";"),
            ("app", "openRunWithLessonWithSource"),
        ],
        "description_single_entry": [
            ("html", 'id="run-tab-btn-overlay"'),
            ("html", 'data-run-tab="overlay"'),
            ("run", "renderOverlayTabContent"),
        ],
        "primary_view_before_console_fallback": [
            ("app", "buildPrimaryViewContract"),
            ("app", "primary_view"),
            ("run", "primaryViewContract"),
            ("run", "primaryViewHeuristicSelection"),
            ("run", "run-observe-console-fallback"),
        ],
        "ddn_edit_run_path": [
            ("html", 'id="run-ddn-preview"'),
            ("html", 'id="btn-run"'),
            ("run", "this.runDdnPreviewEl"),
            ("run", "applyWasmLogicAndDispatchState"),
            ("run", "runViaExecServer"),
            ("slider", "parseFromDdn"),
        ],
    }
    hay = {"html": html, "app": app, "browse": browse, "run": run_js, "styles": styles, "slider": slider}
    missing_tokens: list[str] = []
    for group, tokens in checks.items():
        for label, token in tokens:
            if token not in hay[label]:
                missing_tokens.append(f"{group}:{label}:{token}")
    if missing_tokens:
        return fail("E_SEAMGRIM_UI_AGE3_STALE_CONTRACT", json.dumps(missing_tokens, ensure_ascii=False))

    forbidden = {
        "html": ["data-classroom-mode-switch", 'data-classroom-mode="'],
        "run": ["applyClassroomModeSwitch", "syncClassroomModeSwitch", "__STUDIO_CLASSROOM_MODE_SWITCH__", "classroom_mode_switch"],
        "styles": [".run-classroom-mode-switch", ".run-classroom-mode-btn", "[data-classroom-mode-switch]"],
    }
    hits: list[str] = []
    for label, tokens in forbidden.items():
        for token in tokens:
            if token in hay[label]:
                hits.append(f"{label}:{token}")
    if hits:
        return fail("E_SEAMGRIM_UI_AGE3_RUNTIME_ROLE_TOGGLE", json.dumps(hits, ensure_ascii=False))
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{(proc.stdout or '').strip()}")
    print(f"PASS: {' '.join(cmd)}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_UI_AGE3_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_UI_AGE3_SSOT_DIRTY", proc.stdout.strip())
    print("PASS: git status --short -- docs/ssot")
    return 0


def main() -> int:
    rc = check_product_contract_tokens()
    if rc:
        return rc
    gates = [
        (["node", "--check", "tests/seamgrim_ui_age3_gate_runner.mjs"], "E_SEAMGRIM_UI_AGE3_RUNNER_NODE", 60),
        (["node", "tests/seamgrim_ui_age3_gate_runner.mjs"], "E_SEAMGRIM_UI_AGE3_BROWSER", 300),
        (["python", "tests/run_seamgrim_ui_v2_product_shell_check.py"], "E_SEAMGRIM_UI_AGE3_V2_SHELL", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-ui-age3-gate] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
