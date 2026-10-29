#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "lesson_plan_roundtrip_v1"
LESSON = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_pendulum_wave1_v1"
PLAN_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "lesson_plan_v0.js"
RUN_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"
EDITOR_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "editor.js"
RUNNER = ROOT / "tests" / "seamgrim_lesson_plan_v0_runner.mjs"

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


def check_static_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        LESSON / "lesson_plan.v0.toml",
        LESSON / "meta.toml",
        PLAN_MODULE,
        RUN_JS,
        EDITOR_JS,
        RUNNER,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_LESSON_PLAN_ROUNDTRIP_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("pack") != "lesson_plan_roundtrip_v1":
        return fail("E_LESSON_PLAN_ROUNDTRIP_PACK", repr(contract))
    if contract.get("activity_kinds") != ["observe", "change", "compare", "explain"]:
        return fail("E_LESSON_PLAN_ROUNDTRIP_KINDS", repr(contract.get("activity_kinds")))
    for key in ("ddn_syntax_change", "parser_runtime_semantics_change", "state_hash_formula_change", "docs_ssot_change"):
        if contract.get(key) is not False:
            return fail("E_LESSON_PLAN_ROUNDTRIP_BOUNDARY", f"{key}={contract.get(key)!r}")
    if 'lesson_plan_path = "solutions/seamgrim_ui_mvp/lessons/rep_phys_pendulum_wave1_v1/lesson_plan.v0.toml"' not in read(LESSON / "meta.toml"):
        return fail("E_LESSON_PLAN_ROUNDTRIP_META", "lesson_plan_path missing")
    plan_text = read(LESSON / "lesson_plan.v0.toml")
    for token in [
        'schema = "seamgrim.lesson_plan.v0"',
        'activity_id = "observe-angle-energy"',
        'activity_id = "change-start-angle"',
        'activity_id = "compare-original-changed"',
        'activity_id = "explain-zero-crossing-energy"',
        'allowed_controls = ["theta0"]',
        'anchor_refs = ["obs.theta", "obs.kinetic_energy", "obs.potential_energy"]',
    ]:
        if token not in plan_text:
            return fail("E_LESSON_PLAN_ROUNDTRIP_PLAN_TOKEN", token)
    return 0


def check_product_tokens() -> int:
    module = read(PLAN_MODULE)
    for token in [
        "parseLessonPlanToml",
        "serializeLessonPlanToml",
        "roundtripLessonPlanToml",
        "LESSON_PLAN_ACTIVITY_KINDS",
    ]:
        if token not in module:
            return fail("E_LESSON_PLAN_ROUNDTRIP_MODULE_TOKEN", token)
    run_js = read(RUN_JS)
    for token in [
        "renderLessonPlanRunner",
        "selectLessonPlanActivity",
        "__SEAMGRIM_LESSON_PLAN__",
        "lessonPlanAllowedControls",
        "lessonPlanActivityKind",
    ]:
        if token not in run_js:
            return fail("E_LESSON_PLAN_ROUNDTRIP_RUN_TOKEN", token)
    editor_js = read(EDITOR_JS)
    for token in [
        "renderLessonPlanAuthoring",
        "data-lesson-plan-code",
        "onSaveLessonPlan",
        "syncLessonPlanFromEditorFields",
    ]:
        if token not in editor_js:
            return fail("E_LESSON_PLAN_ROUNDTRIP_EDITOR_TOKEN", token)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_LESSON_PLAN_ROUNDTRIP_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_LESSON_PLAN_ROUNDTRIP_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_static_contract, check_product_tokens):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "solutions/seamgrim_ui_mvp/ui/lesson_plan_v0.js"], "E_LESSON_PLAN_ROUNDTRIP_MODULE_NODE_CHECK", 60),
        (["node", "--check", "tests/seamgrim_lesson_plan_v0_runner.mjs"], "E_LESSON_PLAN_ROUNDTRIP_RUNNER_NODE_CHECK", 60),
        (["node", "tests/seamgrim_lesson_plan_v0_runner.mjs"], "E_LESSON_PLAN_ROUNDTRIP_BROWSER", 300),
        (["python", "tests/run_pack_golden.py", "lesson_plan_roundtrip_v1"], "E_LESSON_PLAN_ROUNDTRIP_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[lesson-plan-roundtrip-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
