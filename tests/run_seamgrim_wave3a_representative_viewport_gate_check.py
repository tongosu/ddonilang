#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "tests" / "seamgrim_wave3a_representative_viewport_gate_runner.mjs"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


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


def check_runner_tokens() -> int:
    if not RUNNER.exists():
        return fail("E_WAVE3A_VIEWPORT_RUNNER_MISSING", str(RUNNER.relative_to(ROOT)))
    text = RUNNER.read_text(encoding="utf-8")
    for token in [
        "1440",
        "1280",
        "1024",
        "390",
        "rep_math_function_line_v1",
        "rep_phys_pendulum_wave1_v1",
        "rep_phys_projectile_xy_v1",
        "rep_phys_damped_oscillation_v1",
        "rep_econ_supply_demand_tax_v1",
        "curriculumArtifactBundleSchema",
        "curriculumModelComparisonSchema",
        "curriculumSemanticSceneSchema",
        "PRIMARY_MODES",
        "unstable primary mode",
        "canvasEvidence",
        "horizontal overflow",
        "DDN source inaccessible",
        "reset tooltip/aria",
    ]:
        if token not in text:
            return fail("E_WAVE3A_VIEWPORT_RUNNER_TOKEN", token)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{proc.stdout.strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_WAVE3A_VIEWPORT_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_WAVE3A_VIEWPORT_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    rc = check_runner_tokens()
    if rc:
        return rc
    gates = [
        (["node", "--check", "tests/seamgrim_wave3a_representative_viewport_gate_runner.mjs"], "E_WAVE3A_VIEWPORT_NODE", 60),
        (["node", "tests/seamgrim_wave3a_representative_viewport_gate_runner.mjs"], "E_WAVE3A_VIEWPORT_BROWSER", 900),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-wave3a-representative-viewport-gate] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
