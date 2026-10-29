#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_semantic_cursor_v1"
LESSON = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_pendulum_wave1_v1"
RUNTIME = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "semantic_cursor_runtime.js"
RUN_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"
RUNNER = ROOT / "tests" / "seamgrim_semantic_cursor_runner.mjs"

EXPECTED_ANCHORS = [
    "obs.theta",
    "obs.angular_velocity",
    "obs.bob",
    "obs.speed",
    "obs.potential_energy",
    "obs.kinetic_energy",
]


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
        LESSON / "lesson.ddn",
        LESSON / "view_plan.detjson",
        RUNTIME,
        RUN_JS,
        RUNNER,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_SEMANTIC_CURSOR_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("pack") != "seamgrim_semantic_cursor_v1":
        return fail("E_SEAMGRIM_SEMANTIC_CURSOR_PACK", repr(contract))
    if contract.get("anchors") != EXPECTED_ANCHORS:
        return fail("E_SEAMGRIM_SEMANTIC_CURSOR_ANCHORS", repr(contract.get("anchors")))
    for key in ("state_hash_participation", "state_hash_formula_change", "parser_change", "runtime_semantics_change", "docs_ssot_change", "world_write"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_SEMANTIC_CURSOR_BOUNDARY", f"{key}={contract.get(key)!r}")
    return 0


def check_product_sources() -> int:
    view_plan = json.loads(read(LESSON / "view_plan.detjson"))
    anchors = [str(row.get("anchor_id")) for row in view_plan.get("anchors", []) if isinstance(row, dict)]
    for anchor in EXPECTED_ANCHORS:
        if anchor not in anchors:
            return fail("E_SEAMGRIM_SEMANTIC_CURSOR_VIEW_PLAN_ANCHOR", f"{anchor} missing from {anchors!r}")
    lesson = read(LESSON / "lesson.ddn")
    for token in [
        "speed <- L * omega",
        "kinetic_energy <- 0.5 * speed * speed",
        "potential_energy <- g * L * (1 - ((theta) cos))",
        "series:kinetic_energy",
        "series:potential_energy",
    ]:
        if token not in lesson:
            return fail("E_SEAMGRIM_SEMANTIC_CURSOR_LESSON_DDN", token)
    runtime = read(RUNTIME)
    for token in [
        "REQUIRED_PENDULUM_ANCHORS",
        "source_ref",
        "provenance",
        "W_SEMANTIC_SOURCE_REF_MISSING",
        "semantic_cursor_selected",
        "state_hash_participation: false",
    ]:
        if token not in runtime:
            return fail("E_SEAMGRIM_SEMANTIC_CURSOR_RUNTIME_TOKEN", token)
    run_js = read(RUN_JS)
    for token in [
        "buildSeamgrimSemanticCursorState",
        "applySemanticCursorToViews",
        "data-semantic-anchor-action",
        "__SEAMGRIM_SEMANTIC_CURSOR__",
        "semanticCursorStateHashParticipation",
        "semanticCursorSpaceShapeCount",
    ]:
        if token not in run_js:
            return fail("E_SEAMGRIM_SEMANTIC_CURSOR_RUN_JS_TOKEN", token)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_SEMANTIC_CURSOR_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_SEMANTIC_CURSOR_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_contract, check_product_sources):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "solutions/seamgrim_ui_mvp/ui/semantic_cursor_runtime.js"], "E_SEAMGRIM_SEMANTIC_CURSOR_RUNTIME_NODE_CHECK", 60),
        (["node", "--check", "tests/seamgrim_semantic_cursor_runner.mjs"], "E_SEAMGRIM_SEMANTIC_CURSOR_RUNNER_NODE_CHECK", 60),
        (["node", "tests/seamgrim_semantic_cursor_runner.mjs"], "E_SEAMGRIM_SEMANTIC_CURSOR_BROWSER", 300),
        (["python", "tests/run_pack_golden.py", "seamgrim_semantic_cursor_v1"], "E_SEAMGRIM_SEMANTIC_CURSOR_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-semantic-cursor-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
