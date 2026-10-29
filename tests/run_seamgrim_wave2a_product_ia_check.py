#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wave2a_product_ia_v1"


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


def check_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "seamgrim_wave2a_product_ia_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_WAVE2A_IA_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("pack") != "seamgrim_wave2a_product_ia_v1":
        return fail("E_SEAMGRIM_WAVE2A_IA_PACK", repr(contract))
    if contract.get("product_behavior_claim") is not True:
        return fail("E_SEAMGRIM_WAVE2A_IA_BEHAVIOR", repr(contract.get("product_behavior_claim")))
    for key in ("parser_change", "runtime_semantics_change", "state_hash_formula_change", "docs_ssot_change"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_WAVE2A_IA_BOUNDARY", f"{key}={contract.get(key)!r}")
    return 0


def check_sources() -> int:
    html = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html")
    app = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js")
    css = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css")
    editor = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "editor.js")
    run_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js")
    for token in [
        'id="screen-publish"',
        'id="publish-panel"',
        'data-publish-panel',
        'id="btn-editor-world-code"',
        'id="btn-editor-lesson-plan"',
        'id="btn-editor-expand"',
        'data-lesson-plan-authoring',
    ]:
        if token not in html:
            return fail("E_SEAMGRIM_WAVE2A_IA_HTML", token)
    if '["learn", "lab", "create", "publish"]' not in app:
        return fail("E_SEAMGRIM_WAVE2A_IA_APP", "publish shell space not accepted")
    for token in ["MAIN_TAB_PUBLISH", 'byId("screen-publish")', "publishRoot: byId(\"screen-publish\")"]:
        if token not in app:
            return fail("E_SEAMGRIM_WAVE2A_IA_APP", token)
    if ".browse-space-publish" in app:
        return fail("E_SEAMGRIM_WAVE2A_IA_BROWSE_PUBLISH_LEAK", "app still toggles browse-space-publish")
    for token in ["#screen-publish", ".browse-publish-panel", "min-height: 420px", ".editor-mode-tabs"]:
        if token not in css:
            return fail("E_SEAMGRIM_WAVE2A_IA_CSS", token)
    if ".browse-space-publish" in css:
        return fail("E_SEAMGRIM_WAVE2A_IA_BROWSE_PUBLISH_CSS", "css still has browse-space-publish")
    for token in ["setEditorMode", "setExpanded", "editor-expanded", "lesson-plan"]:
        if token not in editor:
            return fail("E_SEAMGRIM_WAVE2A_IA_EDITOR", token)
    blocked_sources = {
        "html": (html, ["data-classroom-mode-switch", "data-classroom-mode="]),
        "run_js": (run_js, ["applyClassroomModeSwitch", "syncClassroomModeSwitch", "classroomModeAccess", "__STUDIO_CLASSROOM_MODE_SWITCH__", "classroom_mode_switch"]),
        "css": (css, [".run-classroom-mode-switch", ".run-classroom-mode-btn", "[data-classroom-mode-switch]"]),
    }
    for source_name, (source, blocked_tokens) in blocked_sources.items():
        for token in blocked_tokens:
            if token in source:
                return fail("E_SEAMGRIM_WAVE2A_IA_RUNTIME_ROLE_TOGGLE", f"{source_name}:{token}")
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_WAVE2A_IA_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_WAVE2A_IA_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_contract, check_sources):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "tests/seamgrim_wave2a_product_ia_runner.mjs"], "E_SEAMGRIM_WAVE2A_IA_NODE_CHECK", 60),
        (["node", "tests/seamgrim_wave2a_product_ia_runner.mjs"], "E_SEAMGRIM_WAVE2A_IA_BROWSER", 240),
        (["python", "tests/run_pack_golden.py", "seamgrim_wave2a_product_ia_v1"], "E_SEAMGRIM_WAVE2A_IA_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-wave2a-product-ia-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
