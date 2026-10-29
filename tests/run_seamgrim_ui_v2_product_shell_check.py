#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_ui_v2_product_shell_v1"


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


def check_files_and_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "seamgrim_ui_v2_product_shell_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_UI_V2_SHELL_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("pack") != "seamgrim_ui_v2_product_shell_v1":
        return fail("E_SEAMGRIM_UI_V2_SHELL_PACK", repr(contract))
    if contract.get("product_behavior_claim") is not True:
        return fail("E_SEAMGRIM_UI_V2_SHELL_BEHAVIOR_CLAIM", repr(contract.get("product_behavior_claim")))
    for key in ("parser_change", "runtime_semantics_change", "state_hash_formula_change", "docs_ssot_change"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_UI_V2_SHELL_BOUNDARY", f"{key}={contract.get(key)!r}")
    return 0


def check_product_sources() -> int:
    html = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html")
    browse = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "browse.js")
    run_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js")
    css = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css")
    for token in [
        'data-shell-space="learn"',
        'data-shell-space="lab"',
        'data-shell-space="create"',
        'data-shell-space="publish"',
        'id="btn-run-mobile-ddn-toggle"',
    ]:
        if token not in html:
            return fail("E_SEAMGRIM_UI_V2_SHELL_HTML", token)
    for token in ["수업 열기", "실험실에서 바꾸기", "수업 열기로 실행 화면을 연다"]:
        if token not in browse:
            return fail("E_SEAMGRIM_UI_V2_SHELL_BROWSE", token)
    for blocked in ["학생으로 실행", "교사용 배포 준비", "학생 시작으로 수업을 실행한다"]:
        if blocked in browse:
            return fail("E_SEAMGRIM_UI_V2_SHELL_LEGACY_ROLE_TEXT", blocked)
    for token in ["setMobileDdnOpen", "runMobileDdnToggleBtn", "syncDeploymentProfileState"]:
        if token not in run_js:
            return fail("E_SEAMGRIM_UI_V2_SHELL_RUN_JS", token)
    for source_name, source, blocked_tokens in [
        ("html", html, ["data-classroom-mode-switch", "data-classroom-mode="]),
        ("run_js", run_js, ["applyClassroomModeSwitch", "syncClassroomModeSwitch", "classroomModeAccess", "__STUDIO_CLASSROOM_MODE_SWITCH__", "classroom_mode_switch"]),
        ("css", css, [".run-classroom-mode-switch", ".run-classroom-mode-btn", "[data-classroom-mode-switch]"]),
    ]:
        for token in blocked_tokens:
            if token in source:
                return fail("E_SEAMGRIM_UI_V2_SHELL_RUNTIME_ROLE_TOGGLE", f"{source_name}:{token}")
    for token in [
        ".run-layout.run-layout--studio-basic.run-layout--split",
        'data-mobile-ddn-open="1"]:not([data-first-delight])',
        ".run-mobile-ddn-toggle",
    ]:
        if token not in css:
            return fail("E_SEAMGRIM_UI_V2_SHELL_CSS", token)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_UI_V2_SHELL_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_UI_V2_SHELL_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_files_and_contract, check_product_sources):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "tests/seamgrim_ui_v2_product_shell_runner.mjs"], "E_SEAMGRIM_UI_V2_SHELL_NODE_CHECK", 60),
        (["node", "tests/seamgrim_ui_v2_product_shell_runner.mjs"], "E_SEAMGRIM_UI_V2_SHELL_BROWSER", 240),
        (["python", "tests/run_pack_golden.py", "seamgrim_ui_v2_product_shell_v1"], "E_SEAMGRIM_UI_V2_SHELL_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-ui-v2-product-shell-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
