#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wave2b_quality_gate_v1"


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
        ROOT / "tests" / "seamgrim_wave2b_quality_gate_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_W2B_QUALITY_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("gate") != "G8" or contract.get("viewports") != ["1440x900", "1280x720", "1024x768", "390x844"]:
        return fail("E_SEAMGRIM_W2B_QUALITY_CONTRACT", repr(contract))
    for key in ("state_hash_formula_change", "docs_ssot_change", "new_ddn_syntax", "final_korean_manim_complete_claim"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_W2B_QUALITY_BOUNDARY", f"{key}={contract.get(key)!r}")
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    print(f"PASS: {' '.join(cmd)}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_W2B_QUALITY_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_W2B_QUALITY_SSOT_DIRTY", proc.stdout.strip())
    print("PASS: git status --short -- docs/ssot")
    return 0


def main() -> int:
    rc = check_files_and_contract()
    if rc:
        return rc
    gates = [
        (["node", "--check", "tests/seamgrim_wave2b_quality_gate_runner.mjs"], "E_SEAMGRIM_W2B_QUALITY_NODE", 60),
        (["node", "tests/seamgrim_wave2b_quality_gate_runner.mjs"], "E_SEAMGRIM_W2B_QUALITY_BROWSER", 300),
        (["python", "tests/run_seamgrim_ui_v2_product_shell_check.py"], "E_SEAMGRIM_W2B_QUALITY_UI_V2", 300),
        (["python", "tests/run_seamgrim_primary_view_contract_check.py"], "E_SEAMGRIM_W2B_QUALITY_PRIMARY_VIEW", 300),
        (["python", "tests/run_seamgrim_lab_e1_minimal_check.py"], "E_SEAMGRIM_W2B_QUALITY_LAB_E1", 300),
        (["python", "tests/run_seamgrim_pendulum_view_plan_product_check.py"], "E_SEAMGRIM_W2B_QUALITY_PENDULUM", 300),
        (["python", "tests/run_seamgrim_sealed_trace_reaction_check.py"], "E_SEAMGRIM_W2B_QUALITY_SEALED_TRACE", 300),
        (["python", "tests/run_seamgrim_wave2a_product_ia_check.py"], "E_SEAMGRIM_W2B_QUALITY_W2A_IA", 300),
        (["python", "tests/run_seamgrim_live_trace_reaction_check.py"], "E_SEAMGRIM_W2B_QUALITY_LIVE_TRACE", 300),
        (["python", "tests/run_seamgrim_offline_live_reaction_parity_check.py"], "E_SEAMGRIM_W2B_QUALITY_OFFLINE_PARITY", 300),
        (["python", "tests/run_seamgrim_semantic_cursor_check.py"], "E_SEAMGRIM_W2B_QUALITY_SEMANTIC_CURSOR", 300),
        (["python", "tests/run_lesson_plan_roundtrip_v1_check.py"], "E_SEAMGRIM_W2B_QUALITY_LESSON_ROUNDTRIP", 300),
        (["python", "tests/run_seamgrim_responsive_lesson_workflow_check.py"], "E_SEAMGRIM_W2B_QUALITY_RESPONSIVE", 300),
        (["python", "tests/run_seamgrim_deployment_profile_v0_check.py"], "E_SEAMGRIM_W2B_QUALITY_G3", 300),
        (["python", "tests/run_seamgrim_measurement_components_check.py"], "E_SEAMGRIM_W2B_QUALITY_G4", 300),
        (["python", "tests/run_seamgrim_view_reaction_authoring_check.py"], "E_SEAMGRIM_W2B_QUALITY_G5", 300),
        (["python", "tests/run_seamgrim_lesson_view_plan_roundtrip_check.py"], "E_SEAMGRIM_W2B_QUALITY_G6", 300),
        (["python", "tests/run_seamgrim_wave2b_representative_worlds_check.py"], "E_SEAMGRIM_W2B_QUALITY_G7", 600),
        (
            [
                "python",
                "tests/run_pack_golden.py",
                "seamgrim_deployment_profile_v0_v1",
                "seamgrim_measurement_components_v1",
                "seamgrim_view_reaction_authoring_v1",
                "seamgrim_lesson_view_plan_roundtrip_v1",
                "seamgrim_wave2b_representative_worlds_v1",
                "seamgrim_wave2b_quality_gate_v1",
            ],
            "E_SEAMGRIM_W2B_QUALITY_PACK_GOLDEN",
            300,
        ),
        (["cargo", "check"], "E_SEAMGRIM_W2B_QUALITY_CARGO_CHECK", 600),
        (["cargo", "check", "--manifest-path", "tools/teul-cli/Cargo.toml"], "E_SEAMGRIM_W2B_QUALITY_TEUL_CARGO_CHECK", 600),
        (["python", "tests/run_ci_sanity_gate.py", "--profile", "core_lang"], "E_SEAMGRIM_W2B_QUALITY_CORE_LANG", 600),
        (["git", "diff", "--check"], "E_SEAMGRIM_W2B_QUALITY_DIFF_CHECK", 120),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-wave2b-quality-gate-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
