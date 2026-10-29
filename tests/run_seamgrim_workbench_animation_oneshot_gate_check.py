#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "tests" / "seamgrim_workbench_animation_oneshot_gate_runner.mjs"
WORKBENCH_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js"
WORKBENCH_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "index.html"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


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


def main() -> int:
    missing = [path for path in [RUNNER, WORKBENCH_JS, WORKBENCH_HTML] if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_G12_MISSING", "\n".join(str(path.relative_to(ROOT)) for path in missing))

    runner = RUNNER.read_text(encoding="utf-8")
    required_runner_tokens = [
        "proveLiveAnimation",
        "proveOneShot",
        "canvasEvidence",
        "autoSequence",
        "manualSequence",
        "pauseResumeSequence",
        "rep_phys_projectile_xy_v1",
        "rep_phys_pendulum_wave1_v1",
        "pixel_hash",
        "semantic_payload_hash",
        "state_hash",
        "lifecycle-step",
        "view-role-side",
    ]
    for token in required_runner_tokens:
        if token not in runner:
            return fail("E_SEAMGRIM_G12_RUNNER_TOKEN", token)

    product = WORKBENCH_JS.read_text(encoding="utf-8")
    for token in [
        "commitNextLiveStep",
        "origin = \"auto\"",
        "origin === \"manual\"",
        "semantic_payload_hash",
        "committedStateHash",
        "lifecycle_pause",
        "lifecycle_resume",
        "lifecycle_step",
    ]:
        if token not in product:
            return fail("E_SEAMGRIM_G12_PRODUCT_TOKEN", token)

    browser = run(["node", str(RUNNER.relative_to(ROOT))], timeout=300)
    if browser.returncode:
        return fail("E_SEAMGRIM_G12_BROWSER", browser.stdout.strip())
    if "seamgrim.workbench_animation_oneshot_gate.runner_result.v1" not in browser.stdout:
        return fail("E_SEAMGRIM_G12_BROWSER_RESULT", browser.stdout.strip())
    if "seamgrim_workbench_animation_oneshot_gate: ok" not in browser.stdout:
        return fail("E_SEAMGRIM_G12_BROWSER_OK", browser.stdout.strip())

    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_G12_SSOT", ssot.stdout.strip())

    print("[seamgrim-workbench-animation-oneshot-gate-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
