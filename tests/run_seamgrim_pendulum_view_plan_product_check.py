#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_pendulum_view_plan_product_v1"
LESSON = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_pendulum_wave1_v1"


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
        LESSON / "lesson.ddn",
        LESSON / "meta.toml",
        LESSON / "view_plan.detjson",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "view_plan_runtime.js",
        ROOT / "tests" / "seamgrim_pendulum_view_plan_product_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_PENDULUM_PRODUCT_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("binding_mode") != "explicit":
        return fail("E_SEAMGRIM_PENDULUM_PRODUCT_BINDING", repr(contract))
    if contract.get("camera_policies") != ["fixed", "fit", "follow"]:
        return fail("E_SEAMGRIM_PENDULUM_PRODUCT_CAMERA", repr(contract.get("camera_policies")))
    if contract.get("cli_frontdoor_command") != "teul-cli run rep_phys_pendulum_wave1_v1 --madi 2":
        return fail("E_SEAMGRIM_PENDULUM_PRODUCT_CLI_CONTRACT", repr(contract.get("cli_frontdoor_command")))
    for key in ("state_hash_formula_change", "view_plan_state_hash_participation", "parser_change", "runtime_semantics_change", "docs_ssot_change"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_PENDULUM_PRODUCT_BOUNDARY", f"{key}={contract.get(key)!r}")
    return 0


def check_product_sources() -> int:
    view_plan = json.loads(read(LESSON / "view_plan.detjson"))
    if view_plan.get("schema") != "seamgrim.view_plan.v1" or view_plan.get("binding_mode") != "explicit":
        return fail("E_SEAMGRIM_PENDULUM_PRODUCT_VIEW_PLAN", repr(view_plan))
    anchors = {str(row.get("anchor_id")) for row in view_plan.get("anchors", []) if isinstance(row, dict)}
    expected = {"obs.theta", "obs.angular_velocity", "obs.pivot", "obs.bob", "obs.time", "obs.speed", "obs.kinetic_energy", "obs.potential_energy"}
    if anchors != expected:
        return fail("E_SEAMGRIM_PENDULUM_PRODUCT_ANCHORS", repr(anchors))
    policies = [str(row.get("policy_id")) for row in view_plan.get("camera", {}).get("policies", []) if isinstance(row, dict)]
    if policies != ["fixed", "fit", "follow"]:
        return fail("E_SEAMGRIM_PENDULUM_PRODUCT_VIEW_PLAN_CAMERA", repr(policies))
    meta = read(LESSON / "meta.toml")
    for token in ["view_plan_path", "semantic_binding", "angle_measure", "same_tick_graph_cursor"]:
        if token not in meta:
            return fail("E_SEAMGRIM_PENDULUM_PRODUCT_META", token)
    allowlist = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "active_allowlist.detjson")
    if "rep_phys_pendulum_wave1_v1" not in allowlist:
        return fail("E_SEAMGRIM_PENDULUM_PRODUCT_ALLOWLIST", "pendulum lesson not active")
    run_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js")
    for token in [
        "buildSeamgrimViewPlanRuntime",
        "viewPlanCameraPolicy",
        "viewPlanStateHashParticipation",
        "viewPlanAngleMeasureVisible",
        "__SEAMGRIM_VIEW_PLAN_RUNTIME__",
    ]:
        if token not in run_js:
            return fail("E_SEAMGRIM_PENDULUM_PRODUCT_RUN_JS", token)
    runtime = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "view_plan_runtime.js")
    for token in ["measure.angle.theta", "angle_measure_visible", "E_VIEW_PLAN_UNIT_MISMATCH", "world_draw_payload", "state_hash_participation: false"]:
        if token not in runtime:
            return fail("E_SEAMGRIM_PENDULUM_PRODUCT_RUNTIME", token)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_PENDULUM_PRODUCT_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_PENDULUM_PRODUCT_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_files_and_contract, check_product_sources):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "tests/seamgrim_pendulum_view_plan_product_runner.mjs"], "E_SEAMGRIM_PENDULUM_PRODUCT_NODE_CHECK", 60),
        (
            [
                "cargo",
                "run",
                "--manifest-path",
                "tools/teul-cli/Cargo.toml",
                "--",
                "run",
                "solutions/seamgrim_ui_mvp/lessons/rep_phys_pendulum_wave1_v1/lesson.ddn",
                "--madi",
                "2",
            ],
            "E_SEAMGRIM_PENDULUM_PRODUCT_CLI_FRONTDOOR",
            300,
        ),
        (["node", "tests/seamgrim_pendulum_view_plan_product_runner.mjs"], "E_SEAMGRIM_PENDULUM_PRODUCT_BROWSER", 240),
        (["python", "tests/run_pack_golden.py", "seamgrim_pendulum_view_plan_product_v1"], "E_SEAMGRIM_PENDULUM_PRODUCT_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-pendulum-view-plan-product-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
