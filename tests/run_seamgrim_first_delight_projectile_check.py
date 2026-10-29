#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_first_delight_projectile_v1"
RUNNER = ROOT / "tests" / "seamgrim_first_delight_projectile_runner.mjs"
LESSON_ID = "rep_phys_projectile_xy_v1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(cmd: list[str], *, timeout: int = 240) -> subprocess.CompletedProcess[str]:
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


def require_files() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / LESSON_ID / "lesson.ddn",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / LESSON_ID / "meta.toml",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "components" / "slider_panel.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    return fail("E_SEAMGRIM_FIRST_DELIGHT_MISSING", str(missing)) if missing else 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    return fail(code, f"{path.relative_to(ROOT)} missing {missing}") if missing else 0


def check_product_tokens() -> int:
    checks = [
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / LESSON_ID / "meta.toml",
            [
                'first_delight_id = "projectile"',
                'first_delight_controls = ["초기속도x", "초기속도y", "중력가속도"]',
                'view_roles = ["trajectory_trail"]',
            ],
            "E_SEAMGRIM_FIRST_DELIGHT_META",
        ),
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
            ["FIRST_DELIGHT_QUERY_KEY", "FIRST_DELIGHT_PROJECTILE_ID", 'launchKind: "first_delight"'],
            "E_SEAMGRIM_FIRST_DELIGHT_APP",
        ),
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js",
            [
                "resolveFirstDelightControlPresentation",
                'role: "trajectory_trail"',
                'role: "afterimage"',
                "synthesizeTrajectorySpace2dFromObservation",
                'this.lastLaunchKind === "first_delight"',
                "mainViewRoles",
                "engineStatus",
            ],
            "E_SEAMGRIM_FIRST_DELIGHT_RUN",
        ),
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "components" / "slider_panel.js",
            ["visibleControlNames", "controlLabels", "displayLabel", "dataset.controlName"],
            "E_SEAMGRIM_FIRST_DELIGHT_CONTROLS",
        ),
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
            [
                'data-testid="lesson-pause-button"',
                'data-testid="lesson-reset-button"',
                'data-testid="lesson-step-button"',
                'data-testid="lesson-space2d-canvas"',
                'data-testid="lesson-control-panel"',
            ],
            "E_SEAMGRIM_FIRST_DELIGHT_HTML",
        ),
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
            ["#screen-run[data-first-delight]"],
            "E_SEAMGRIM_FIRST_DELIGHT_CSS",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract() -> int:
    payload = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_first_delight_projectile_v1",
        "kind": "seamgrim_first_delight_product_behavior",
        "runtime_claim": False,
        "product_code_change": True,
        "product_behavior_claim": True,
        "closed_by": "SEAMGRIM_FIRST_DELIGHT_PROJECTILE_V1",
        "browser_runner": "tests/seamgrim_first_delight_projectile_runner.mjs",
        "product_lesson_id": LESSON_ID,
        "product_lesson_pack": "edu_seamgrim_rep_phys_projectile_xy_v1",
        "route": "?first_delight=projectile",
        "view_roles_in_state_hash": False,
        "parser_change": False,
        "runtime_semantics_change": False,
        "docs_ssot_change": False,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_SEAMGRIM_FIRST_DELIGHT_CONTRACT", f"{key}={payload.get(key)!r}")
    required_covers = {
        "first_delight_route",
        "three_visible_controls",
        "trajectory_canvas_nonblank",
        "run_pause_step_reset",
        "same_input_same_state_hash",
        "desktop_mobile_layout",
        "cli_check_canon_run_same_source",
    }
    if not required_covers.issubset(set(payload.get("covers", []))):
        return fail("E_SEAMGRIM_FIRST_DELIGHT_COVERS", repr(payload.get("covers")))
    return 0


def check_lesson_sync() -> int:
    mirror = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / LESSON_ID / "lesson.ddn")
    pack = read(ROOT / "pack" / "edu_seamgrim_rep_phys_projectile_xy_v1" / "lesson.ddn")
    if mirror != pack:
        return fail("E_SEAMGRIM_FIRST_DELIGHT_LESSON_SYNC", "active lesson differs from representative pack")
    return 0


def run_gate(cmd: list[str], code: str, *, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_FIRST_DELIGHT_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_FIRST_DELIGHT_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (require_files, check_product_tokens, check_contract, check_lesson_sync):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "tests/seamgrim_first_delight_projectile_runner.mjs"], "E_SEAMGRIM_FIRST_DELIGHT_NODE_CHECK", 60),
        (["node", "tests/seamgrim_first_delight_projectile_runner.mjs"], "E_SEAMGRIM_FIRST_DELIGHT_BROWSER", 150),
        (["python", "tests/run_pack_golden.py", "seamgrim_first_delight_projectile_v1"], "E_SEAMGRIM_FIRST_DELIGHT_GOLDEN", 300),
        (["python", "tests/run_seamgrim_subject_representative_examples_check.py"], "E_SEAMGRIM_FIRST_DELIGHT_CLI_PARITY", 600),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout=timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-first-delight-projectile-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
