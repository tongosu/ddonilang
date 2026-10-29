#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CHECK_SCRIPT = ROOT / "tests" / "run_external_intent_seulgi_walk_alignment_check.py"


def fail(msg: str, proc: subprocess.CompletedProcess[str] | None = None) -> int:
    print(f"[external-intent-seulgi-walk-alignment-check-selftest] fail: {msg}")
    if proc is not None:
        if (proc.stdout or "").strip():
            print(proc.stdout.strip())
        if (proc.stderr or "").strip():
            print(proc.stderr.strip())
    return 1


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def main() -> int:
    ok_proc = run([sys.executable, str(CHECK_SCRIPT), "--repo-root", str(ROOT)], ROOT)
    if ok_proc.returncode != 0:
        return fail("default pass case must pass", ok_proc)

    source = CHECK_SCRIPT.read_text(encoding="utf-8")
    target_token = "sam_seulgi_family_contract_selftest_current_probe="
    mutated_token = "sam_seulgi_family_contract_selftest_current_probe_BROKEN="
    if target_token not in source:
        return fail(f"mutation anchor missing: {target_token}")
    mutated = source.replace(target_token, mutated_token, 1)

    with tempfile.TemporaryDirectory(prefix="external_intent_seulgi_walk_alignment_selftest_") as td:
        temp_script = Path(td) / "run_external_intent_seulgi_walk_alignment_check.py"
        temp_script.write_text(mutated, encoding="utf-8")
        bad_proc = run([sys.executable, str(temp_script), "--repo-root", str(ROOT)], ROOT)
        if bad_proc.returncode == 0:
            return fail("mutated ci_sanity token case must fail", bad_proc)
        merged = ((bad_proc.stdout or "") + "\n" + (bad_proc.stderr or "")).strip()
        if "E_ALIGNMENT_CI_SANITY_TOKEN" not in merged:
            return fail("mutated ci_sanity token must report E_ALIGNMENT_CI_SANITY_TOKEN", bad_proc)

    profile_target_block = (
        'SEAMGRIM_PROFILE_REQUIRED_STEPS: tuple[str, ...] = (\n'
        '    "sam_seulgi_family_contract_selftest",\n'
        '    "external_intent_seulgi_walk_alignment_check_selftest",\n'
        ')'
    )
    profile_mutated_block = (
        'SEAMGRIM_PROFILE_REQUIRED_STEPS: tuple[str, ...] = (\n'
        '    "sam_seulgi_family_contract_selftest_BROKEN",\n'
        '    "external_intent_seulgi_walk_alignment_check_selftest",\n'
        ')'
    )
    if profile_target_block not in source:
        return fail(f"mutation anchor missing: {profile_target_block}")
    profile_mutated_source = source.replace(profile_target_block, profile_mutated_block, 1)

    with tempfile.TemporaryDirectory(prefix="external_intent_seulgi_walk_alignment_profile_selftest_") as td:
        temp_script = Path(td) / "run_external_intent_seulgi_walk_alignment_check.py"
        temp_script.write_text(profile_mutated_source, encoding="utf-8")
        bad_proc = run([sys.executable, str(temp_script), "--repo-root", str(ROOT)], ROOT)
        if bad_proc.returncode == 0:
            return fail("mutated seamgrim profile token case must fail", bad_proc)
        merged = ((bad_proc.stdout or "") + "\n" + (bad_proc.stderr or "")).strip()
        if "E_ALIGNMENT_CI_SANITY_SEAMGRIM_PROFILE_TOKEN" not in merged:
            return fail(
                "mutated seamgrim profile token must report E_ALIGNMENT_CI_SANITY_SEAMGRIM_PROFILE_TOKEN",
                bad_proc,
            )

    seamgrim_gate_target_block = (
        'SEAMGRIM_CI_GATE_REQUIRED_TOKENS: tuple[str, ...] = (\n'
        '    \'"sam_seulgi_family_contract_selftest"\',\n'
        '    "tests/run_sam_seulgi_family_contract_selftest.py",\n'
        ')'
    )
    seamgrim_gate_mutated_block = (
        'SEAMGRIM_CI_GATE_REQUIRED_TOKENS: tuple[str, ...] = (\n'
        '    \'"sam_seulgi_family_contract_selftest_BROKEN"\',\n'
        '    "tests/run_sam_seulgi_family_contract_selftest.py",\n'
        ')'
    )
    if seamgrim_gate_target_block not in source:
        return fail(f"mutation anchor missing: {seamgrim_gate_target_block}")
    seamgrim_gate_mutated_source = source.replace(seamgrim_gate_target_block, seamgrim_gate_mutated_block, 1)

    with tempfile.TemporaryDirectory(prefix="external_intent_seulgi_walk_alignment_gate_selftest_") as td:
        temp_script = Path(td) / "run_external_intent_seulgi_walk_alignment_check.py"
        temp_script.write_text(seamgrim_gate_mutated_source, encoding="utf-8")
        bad_proc = run([sys.executable, str(temp_script), "--repo-root", str(ROOT)], ROOT)
        if bad_proc.returncode == 0:
            return fail("mutated seamgrim ci gate token case must fail", bad_proc)
        merged = ((bad_proc.stdout or "") + "\n" + (bad_proc.stderr or "")).strip()
        if "E_ALIGNMENT_SEAMGRIM_CI_GATE_TOKEN" not in merged:
            return fail(
                "mutated seamgrim ci gate token must report E_ALIGNMENT_SEAMGRIM_CI_GATE_TOKEN",
                bad_proc,
            )

    print("[external-intent-seulgi-walk-alignment-check-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
