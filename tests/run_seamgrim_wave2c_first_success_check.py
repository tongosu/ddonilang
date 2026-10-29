#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wave2c_first_success_v1"
RUNNER = ROOT / "tests" / "seamgrim_wave2c_first_success_runner.mjs"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
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
        INDEX_HTML,
        EDITOR_JS,
        STYLE,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_W2C_FIRST_SUCCESS_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("pack") != "seamgrim_wave2c_first_success_v1":
        return fail("E_SEAMGRIM_W2C_FIRST_SUCCESS_PACK", repr(contract))
    if contract.get("gates") != ["G2", "G3", "G4"]:
        return fail("E_SEAMGRIM_W2C_FIRST_SUCCESS_GATES", repr(contract.get("gates")))
    if contract.get("worlds") != ["rep_phys_pendulum_wave1_v1", "rep_phys_projectile_xy_v1", "rep_math_function_line_v1"]:
        return fail("E_SEAMGRIM_W2C_FIRST_SUCCESS_WORLDS", repr(contract.get("worlds")))
    for key in ("docs_ssot_change", "new_ddn_syntax", "state_hash_formula_change", "server_login_cloud_claim"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_W2C_FIRST_SUCCESS_BOUNDARY", f"{key}={contract.get(key)!r}")
    for token in [
        "first_lesson_open_from_cards",
        "first_control_change_updates_hash",
        "product_editor_three_layer_diagnostic",
        "mobile_ddn_toggle",
    ]:
        if token not in contract.get("checks", []):
            return fail("E_SEAMGRIM_W2C_FIRST_SUCCESS_CHECK_TOKEN", token)
    return 0


def check_product_tokens() -> int:
    runner = read(RUNNER)
    for token in [
        "card-launch-primary",
        "실험실에서 바꾸기",
        "semanticCursorStateHashParticipation",
        "viewPlanStateHashParticipation",
        "traceReactionStateHashParticipation",
        "editor-readiness-diagnostic",
        "data-diagnostic-layer",
        "btn-editor-fixit-apply",
        "btn-run-mobile-ddn-toggle",
    ]:
        if token not in runner:
            return fail("E_SEAMGRIM_W2C_FIRST_SUCCESS_RUNNER_TOKEN", token)
    html = read(INDEX_HTML)
    editor = read(EDITOR_JS)
    if "editor-readiness-diagnostic" not in html or "buildReadinessDiagnosticLayers" not in editor:
        return fail("E_SEAMGRIM_W2C_FIRST_SUCCESS_DIAGNOSTIC_UI", "three-layer diagnostic DOM/model missing")
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_W2C_FIRST_SUCCESS_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_W2C_FIRST_SUCCESS_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_contract, check_product_tokens):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "tests/seamgrim_wave2c_first_success_runner.mjs"], "E_SEAMGRIM_W2C_FIRST_SUCCESS_NODE_CHECK", 60),
        (["node", "--check", "solutions/seamgrim_ui_mvp/ui/screens/editor.js"], "E_SEAMGRIM_W2C_FIRST_SUCCESS_EDITOR_NODE", 60),
        (["node", "tests/seamgrim_wave2c_first_success_runner.mjs"], "E_SEAMGRIM_W2C_FIRST_SUCCESS_BROWSER", 420),
        (["python", "tests/run_pack_golden.py", "seamgrim_wave2c_first_success_v1"], "E_SEAMGRIM_W2C_FIRST_SUCCESS_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-wave2c-first-success-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
