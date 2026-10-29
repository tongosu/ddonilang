#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_view_reaction_authoring_v1"


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
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "editor.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
        ROOT / "tests" / "seamgrim_view_reaction_authoring_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_VIEW_REACTION_AUTHORING_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("gate") != "G5" or contract.get("surface") != "create.lesson_plan_authoring":
        return fail("E_SEAMGRIM_VIEW_REACTION_AUTHORING_CONTRACT", repr(contract))
    if contract.get("camera_policies") != ["fixed", "fit", "follow"]:
        return fail("E_SEAMGRIM_VIEW_REACTION_AUTHORING_CAMERA", repr(contract.get("camera_policies")))
    if contract.get("condition_kinds") != ["zero_crossing", "sign_change", "extremum", "tolerance_entry"]:
        return fail("E_SEAMGRIM_VIEW_REACTION_AUTHORING_CONDITIONS", repr(contract.get("condition_kinds")))
    if contract.get("reaction_actions") != ["highlight", "visibility", "camera_focus"]:
        return fail("E_SEAMGRIM_VIEW_REACTION_AUTHORING_ACTIONS", repr(contract.get("reaction_actions")))
    for key in ("implicit_lww_allowed", "live_online_reaction_claim", "general_world_hook_claim", "state_hash_formula_change", "state_hash_participation", "parser_change", "new_ddn_syntax", "docs_ssot_change"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_VIEW_REACTION_AUTHORING_BOUNDARY", f"{key}={contract.get(key)!r}")
    if contract.get("conflict_diagnostic") != "E_VIEW_AUTHOR_REACTION_CONFLICT":
        return fail("E_SEAMGRIM_VIEW_REACTION_AUTHORING_DIAG", repr(contract))
    return 0


def check_product_sources() -> int:
    editor = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "editor.js")
    for token in [
        "seamgrim.view_reaction_authoring.v1",
        "VIEW_AUTHORING_CAMERA_POLICIES",
        "zero_crossing",
        "sign_change",
        "extremum",
        "tolerance_entry",
        "highlight",
        "visibility",
        "camera_focus",
        "E_VIEW_AUTHOR_REACTION_CONFLICT",
        "implicit_lww_allowed: false",
        "state_hash_participation: false",
        "__SEAMGRIM_VIEW_REACTION_AUTHORING__",
        "data-testid=\"view-reaction-authoring-panel\"",
    ]:
        if token not in editor:
            return fail("E_SEAMGRIM_VIEW_REACTION_AUTHORING_EDITOR", token)
    css = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css")
    for token in ["view-reaction-authoring-panel", "view-reaction-authoring-grid", "view-reaction-artifact"]:
        if token not in css:
            return fail("E_SEAMGRIM_VIEW_REACTION_AUTHORING_CSS", token)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_VIEW_REACTION_AUTHORING_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_VIEW_REACTION_AUTHORING_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_files_and_contract, check_product_sources):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "solutions/seamgrim_ui_mvp/ui/screens/editor.js"], "E_SEAMGRIM_VIEW_REACTION_AUTHORING_EDITOR_NODE", 60),
        (["node", "--check", "tests/seamgrim_view_reaction_authoring_runner.mjs"], "E_SEAMGRIM_VIEW_REACTION_AUTHORING_RUNNER_NODE", 60),
        (["node", "tests/seamgrim_view_reaction_authoring_runner.mjs"], "E_SEAMGRIM_VIEW_REACTION_AUTHORING_BROWSER", 240),
        (["python", "tests/run_pack_golden.py", "seamgrim_view_reaction_authoring_v1"], "E_SEAMGRIM_VIEW_REACTION_AUTHORING_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-view-reaction-authoring-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
