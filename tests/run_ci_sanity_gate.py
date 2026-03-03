#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


FAIL_CODE_RE = re.compile(r"fail code=([A-Z0-9_]+)")

CORE_LANG_PROFILE_STEPS = {
    "backup_hygiene_selftest",
    "pipeline_emit_flags_check",
    "pipeline_emit_flags_selftest",
    "ci_profile_split_contract_check",
    "age5_close_pack_contract_selftest",
    "ci_pack_golden_age5_surface_selftest",
    "ci_pack_golden_guideblock_selftest",
    "ci_pack_golden_exec_policy_selftest",
    "ci_pack_golden_jjaim_flatten_selftest",
    "ci_pack_golden_event_model_selftest",
    "w92_aot_pack_check",
    "w93_universe_pack_check",
    "w94_social_pack_check",
    "w95_cert_pack_check",
    "w96_somssi_pack_check",
    "w97_self_heal_pack_check",
}

SEAMGRIM_PROFILE_STEPS = {
    "ci_profile_split_contract_check",
    "seamgrim_ci_gate_seed_meta_step_check",
    "seamgrim_ci_gate_runtime5_passthrough_check",
    "seamgrim_interface_boundary_contract_check",
    "seamgrim_overlay_session_wired_consistency_check",
    "seamgrim_overlay_session_diag_parity_check",
    "seamgrim_overlay_compare_diag_parity_check",
    "seamgrim_wasm_cli_diag_parity_check",
}


def clip(text: str, limit: int = 180) -> str:
    normalized = " ".join(str(text).split())
    if not normalized:
        return "-"
    if len(normalized) <= limit:
        return normalized
    return normalized[: limit - 3] + "..."


def parse_fail_code(stdout: str, stderr: str, default_code: str) -> str:
    payload = f"{stdout}\n{stderr}"
    match = FAIL_CODE_RE.search(payload)
    if match:
        return match.group(1)
    return default_code


def first_message(stdout: str, stderr: str) -> str:
    for raw in (stderr.splitlines() + stdout.splitlines()):
        line = raw.strip()
        if line:
            return clip(line)
    return "-"


def run_step(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Run CI sanity checks and emit one-line status summary")
    parser.add_argument("--json-out", default="", help="optional path to write sanity result json")
    parser.add_argument(
        "--profile",
        choices=("full", "core_lang", "seamgrim"),
        default="full",
        help="sanity profile selector (default: full)",
    )
    args = parser.parse_args()

    py = sys.executable
    steps = [
        (
            "backup_hygiene_selftest",
            [py, "tests/run_ci_backup_hygiene_selftest.py"],
            "E_CI_SANITY_BACKUP_SELFTEST_FAIL",
        ),
        (
            "pipeline_emit_flags_check",
            [py, "tests/run_ci_pipeline_emit_flags_check.py"],
            "E_CI_SANITY_PIPELINE_FLAGS_FAIL",
        ),
        (
            "pipeline_emit_flags_selftest",
            [py, "tests/run_ci_pipeline_emit_flags_check_selftest.py"],
            "E_CI_SANITY_PIPELINE_FLAGS_SELFTEST_FAIL",
        ),
        (
            "ci_profile_split_contract_check",
            [py, "tests/run_ci_profile_split_contract_check.py"],
            "E_CI_SANITY_PROFILE_SPLIT_CONTRACT_FAIL",
        ),
        (
            "seamgrim_ci_gate_seed_meta_step_check",
            [py, "tests/run_seamgrim_ci_gate_seed_meta_step_check.py"],
            "E_CI_SANITY_SEED_META_STEP_FAIL",
        ),
        (
            "seamgrim_ci_gate_runtime5_passthrough_check",
            [py, "tests/run_seamgrim_ci_gate_runtime5_passthrough_check.py"],
            "E_CI_SANITY_RUNTIME5_PASSTHROUGH_FAIL",
        ),
        (
            "seamgrim_interface_boundary_contract_check",
            [py, "tests/run_seamgrim_interface_boundary_contract_check.py"],
            "E_CI_SANITY_SEAMGRIM_INTERFACE_BOUNDARY_FAIL",
        ),
        (
            "seamgrim_overlay_session_wired_consistency_check",
            [py, "tests/run_seamgrim_overlay_session_wired_consistency_check.py"],
            "E_CI_SANITY_OVERLAY_SESSION_WIRED_CONSISTENCY_FAIL",
        ),
        (
            "seamgrim_overlay_session_diag_parity_check",
            [py, "tests/run_seamgrim_overlay_session_diag_parity_check.py"],
            "E_CI_SANITY_OVERLAY_SESSION_DIAG_PARITY_FAIL",
        ),
        (
            "seamgrim_overlay_compare_diag_parity_check",
            [py, "tests/run_seamgrim_overlay_compare_diag_parity_check.py"],
            "E_CI_SANITY_OVERLAY_COMPARE_DIAG_PARITY_FAIL",
        ),
        (
            "age5_close_pack_contract_selftest",
            [py, "tests/run_age5_close_pack_contract_selftest.py"],
            "E_CI_SANITY_AGE5_CLOSE_PACK_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "ci_pack_golden_age5_surface_selftest",
            [py, "tests/run_pack_golden_age5_surface_selftest.py"],
            "E_CI_SANITY_PACK_GOLDEN_AGE5_SURFACE_SELFTEST_FAIL",
        ),
        (
            "ci_pack_golden_guideblock_selftest",
            [py, "tests/run_pack_golden_guideblock_selftest.py"],
            "E_CI_SANITY_PACK_GOLDEN_GUIDEBLOCK_SELFTEST_FAIL",
        ),
        (
            "ci_pack_golden_exec_policy_selftest",
            [py, "tests/run_pack_golden_exec_policy_selftest.py"],
            "E_CI_SANITY_PACK_GOLDEN_EXEC_POLICY_SELFTEST_FAIL",
        ),
        (
            "ci_pack_golden_jjaim_flatten_selftest",
            [py, "tests/run_pack_golden_jjaim_flatten_selftest.py"],
            "E_CI_SANITY_PACK_GOLDEN_JJAIM_FLATTEN_SELFTEST_FAIL",
        ),
        (
            "ci_pack_golden_event_model_selftest",
            [py, "tests/run_pack_golden_event_model_selftest.py"],
            "E_CI_SANITY_PACK_GOLDEN_EVENT_MODEL_SELFTEST_FAIL",
        ),
        (
            "w92_aot_pack_check",
            [py, "tests/run_w92_aot_pack_check.py"],
            "E_CI_SANITY_W92_AOT_PACK_CHECK_FAIL",
        ),
        (
            "w93_universe_pack_check",
            [py, "tests/run_w93_universe_pack_check.py"],
            "E_CI_SANITY_W93_UNIVERSE_PACK_CHECK_FAIL",
        ),
        (
            "w94_social_pack_check",
            [py, "tests/run_w94_social_pack_check.py"],
            "E_CI_SANITY_W94_SOCIAL_PACK_CHECK_FAIL",
        ),
        (
            "w95_cert_pack_check",
            [py, "tests/run_w95_cert_pack_check.py"],
            "E_CI_SANITY_W95_CERT_PACK_CHECK_FAIL",
        ),
        (
            "w96_somssi_pack_check",
            [py, "tests/run_w96_somssi_pack_check.py"],
            "E_CI_SANITY_W96_SOMSSI_PACK_CHECK_FAIL",
        ),
        (
            "w97_self_heal_pack_check",
            [py, "tests/run_w97_self_heal_pack_check.py"],
            "E_CI_SANITY_W97_SELF_HEAL_PACK_CHECK_FAIL",
        ),
        (
            "seamgrim_wasm_cli_diag_parity_check",
            [py, "tests/run_seamgrim_wasm_cli_diag_parity_check.py"],
            "E_CI_SANITY_WASM_CLI_DIAG_PARITY_FAIL",
        ),
    ]
    if args.profile == "core_lang":
        steps = [row for row in steps if row[0] in CORE_LANG_PROFILE_STEPS]
    elif args.profile == "seamgrim":
        steps = [row for row in steps if row[0] in SEAMGRIM_PROFILE_STEPS]

    rows: list[dict[str, object]] = []
    for step_name, cmd, default_code in steps:
        proc = run_step(cmd)
        stdout = proc.stdout or ""
        stderr = proc.stderr or ""
        if stdout.strip():
            print(stdout, end="")
        if stderr.strip():
            print(stderr, end="", file=sys.stderr)
        ok = proc.returncode == 0
        row = {
            "step": step_name,
            "ok": ok,
            "returncode": int(proc.returncode),
            "cmd": cmd,
        }
        if not ok:
            code = parse_fail_code(stdout, stderr, default_code)
            msg = first_message(stdout, stderr)
            row["code"] = code
            row["msg"] = msg
            rows.append(row)
            payload = {
                "schema": "ddn.ci.sanity_gate.v1",
                "generated_at_utc": datetime.now(timezone.utc).isoformat(),
                "status": "fail",
                "code": code,
                "step": step_name,
                "msg": msg,
                "profile": args.profile,
                "steps": rows,
            }
            if args.json_out.strip():
                out = Path(args.json_out)
                out.parent.mkdir(parents=True, exist_ok=True)
                out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            print(f'ci_sanity_status=fail code={code} step={step_name} msg="{msg}" profile={args.profile}')
            return 1
        rows.append(row)

    payload = {
        "schema": "ddn.ci.sanity_gate.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": "pass",
        "code": "OK",
        "step": "all",
        "msg": "-",
        "profile": args.profile,
        "steps": rows,
    }
    if args.json_out.strip():
        out = Path(args.json_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f'ci_sanity_status=pass code=OK step=all msg="-" profile={args.profile}')
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
