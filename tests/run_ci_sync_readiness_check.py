#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

SYNC_READINESS_OK = "OK"
SYNC_READINESS_STEP_FAIL = "E_SYNC_READINESS_STEP_FAIL"
SYNC_READINESS_SANITY_CONTRACT_FAIL = "E_SYNC_READINESS_SANITY_CONTRACT_FAIL"
SYNC_READINESS_VALIDATE_ONLY_PATH_MISSING = "E_SYNC_READINESS_VALIDATE_ONLY_PATH_MISSING"

SANITY_REQUIRED_PASS_STEPS = (
    "backup_hygiene_selftest",
    "pipeline_emit_flags_check",
    "pipeline_emit_flags_selftest",
    "ci_profile_split_contract_check",
    "seamgrim_ci_gate_seed_meta_step_check",
    "seamgrim_ci_gate_runtime5_passthrough_check",
    "seamgrim_interface_boundary_contract_check",
    "seamgrim_overlay_session_wired_consistency_check",
    "seamgrim_overlay_session_diag_parity_check",
    "seamgrim_overlay_compare_diag_parity_check",
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
    "seamgrim_wasm_cli_diag_parity_check",
)


def clip(text: str, limit: int = 180) -> str:
    normalized = " ".join(str(text).split())
    if not normalized:
        return "-"
    if len(normalized) <= limit:
        return normalized
    return normalized[: limit - 3] + "..."


def run_step(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8-sig"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def validate_sanity_contract(path: Path) -> tuple[bool, str]:
    doc = load_json(path)
    if not isinstance(doc, dict):
        return False, f"invalid sanity json: {path}"
    if str(doc.get("schema", "")).strip() != "ddn.ci.sanity_gate.v1":
        return False, f"sanity schema mismatch: {doc.get('schema')}"
    if str(doc.get("status", "")).strip() != "pass":
        return False, f"sanity status mismatch: {doc.get('status')}"
    if str(doc.get("code", "")).strip() != "OK":
        return False, f"sanity code mismatch: {doc.get('code')}"
    if str(doc.get("step", "")).strip() != "all":
        return False, f"sanity step mismatch: {doc.get('step')}"

    steps = doc.get("steps")
    if not isinstance(steps, list):
        return False, "sanity steps must be list"
    if len(steps) < len(SANITY_REQUIRED_PASS_STEPS):
        return False, f"sanity step_count too small: {len(steps)}"

    step_index: dict[str, dict] = {}
    for row in steps:
        if not isinstance(row, dict):
            return False, "sanity steps contains non-object row"
        name = str(row.get("step", "")).strip()
        if name:
            step_index[name] = row

    for required_step in SANITY_REQUIRED_PASS_STEPS:
        row = step_index.get(required_step)
        if row is None:
            return False, f"sanity required step missing: {required_step}"
        if not bool(row.get("ok", False)):
            return False, f"sanity required step not ok: {required_step}"
        try:
            rc = int(row.get("returncode", -1))
        except Exception:
            rc = -1
        if rc != 0:
            return False, f"sanity required step rc!=0: {required_step} rc={row.get('returncode')}"

    return True, "ok"


def first_message(stdout: str, stderr: str) -> str:
    for raw in (stderr.splitlines() + stdout.splitlines()):
        line = str(raw).strip()
        if line:
            return clip(line, 220)
    return "-"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run sync-readiness CI checks (pipeline flags/selftests/sanity/aggregate) in one command"
    )
    parser.add_argument("--report-dir", default="build/reports", help="report directory")
    parser.add_argument("--report-prefix", default="dev_sync_readiness", help="report prefix for aggregate gate")
    parser.add_argument("--json-out", default="", help="optional path for sync-readiness report json")
    parser.add_argument(
        "--validate-only-sanity-json",
        default="",
        help="validate-only mode: skip step execution and validate the given ci_sanity_gate json path",
    )
    parser.add_argument(
        "--skip-aggregate",
        action="store_true",
        help="skip aggregate gate run (quick mode)",
    )
    args = parser.parse_args()

    py = sys.executable
    report_dir = Path(args.report_dir)
    report_dir.mkdir(parents=True, exist_ok=True)
    prefix = args.report_prefix.strip() or "dev_sync_readiness"
    sanity_json = report_dir / f"{prefix}.ci_sanity_gate.detjson"
    validate_only_sanity_json = args.validate_only_sanity_json.strip()

    rows: list[dict[str, object]] = []
    all_ok = True
    status_code = SYNC_READINESS_OK
    status_step = "all"
    status_msg = "-"
    started = datetime.now(timezone.utc).isoformat()
    if validate_only_sanity_json:
        validate_path = Path(validate_only_sanity_json)
        tick = time.perf_counter()
        if not validate_path.exists():
            all_ok = False
            contract_msg = f"missing sanity json path: {validate_path}"
            status_code = SYNC_READINESS_VALIDATE_ONLY_PATH_MISSING
            status_step = "validate_only"
            status_msg = contract_msg
            contract_ok = False
        else:
            contract_ok, contract_msg = validate_sanity_contract(validate_path)
            if not contract_ok:
                all_ok = False
                status_code = SYNC_READINESS_SANITY_CONTRACT_FAIL
                status_step = "sanity_gate_contract"
                status_msg = contract_msg
        elapsed_ms = int((time.perf_counter() - tick) * 1000)
        rows.append(
            {
                "name": "sanity_gate_contract",
                "ok": bool(contract_ok),
                "returncode": 0 if contract_ok else 1,
                "elapsed_ms": elapsed_ms,
                "cmd": ["internal", "validate_sanity_contract", str(validate_path)],
                "stdout_head": clip(contract_msg, 220),
                "stderr_head": "-" if contract_ok else clip(contract_msg, 220),
            }
        )
    else:
        steps: list[tuple[str, list[str]]] = [
            ("pipeline_emit_flags_check", [py, "tests/run_ci_pipeline_emit_flags_check.py"]),
            ("pipeline_emit_flags_selftest", [py, "tests/run_ci_pipeline_emit_flags_check_selftest.py"]),
            ("sanity_gate_diagnostics_check", [py, "tests/run_ci_sanity_gate_diagnostics_check.py"]),
            ("sanity_gate", [py, "tests/run_ci_sanity_gate.py", "--json-out", str(sanity_json)]),
        ]
        if not args.skip_aggregate:
            steps.append(
                (
                    "aggregate_gate",
                    [
                        py,
                        "tests/run_ci_aggregate_gate.py",
                        "--report-dir",
                        str(report_dir),
                        "--report-prefix",
                        prefix,
                        "--skip-core-tests",
                        "--fast-fail",
                        "--backup-hygiene",
                        "--clean-prefixed-reports",
                        "--quiet-success-logs",
                        "--compact-step-logs",
                        "--step-log-dir",
                        str(report_dir),
                        "--step-log-failed-only",
                        "--checklist-skip-seed-cli",
                        "--checklist-skip-ui-common",
                    ],
                )
            )

        for name, cmd in steps:
            tick = time.perf_counter()
            proc = run_step(cmd)
            elapsed_ms = int((time.perf_counter() - tick) * 1000)
            stdout = proc.stdout or ""
            stderr = proc.stderr or ""
            if stdout.strip():
                print(stdout, end="")
            if stderr.strip():
                print(stderr, end="", file=sys.stderr)
            ok = proc.returncode == 0
            rows.append(
                {
                    "name": name,
                    "ok": ok,
                    "returncode": int(proc.returncode),
                    "elapsed_ms": elapsed_ms,
                    "cmd": cmd,
                    "stdout_head": clip(stdout, 220),
                    "stderr_head": clip(stderr, 220),
                }
            )
            if not ok:
                all_ok = False
                status_code = SYNC_READINESS_STEP_FAIL
                status_step = name
                status_msg = first_message(stdout, stderr)
                break

        total_elapsed_ms = sum(int(row.get("elapsed_ms", 0)) for row in rows)
        if all_ok:
            tick = time.perf_counter()
            contract_ok, contract_msg = validate_sanity_contract(sanity_json)
            contract_elapsed_ms = int((time.perf_counter() - tick) * 1000)
            contract_row = {
                "name": "sanity_gate_contract",
                "ok": bool(contract_ok),
                "returncode": 0 if contract_ok else 1,
                "elapsed_ms": contract_elapsed_ms,
                "cmd": ["internal", "validate_sanity_contract", str(sanity_json)],
                "stdout_head": clip(contract_msg, 220),
                "stderr_head": "-" if contract_ok else clip(contract_msg, 220),
            }
            rows.append(contract_row)
            total_elapsed_ms += contract_elapsed_ms
            if not contract_ok:
                print(contract_msg, file=sys.stderr)
                all_ok = False
                status_code = SYNC_READINESS_SANITY_CONTRACT_FAIL
                status_step = "sanity_gate_contract"
                status_msg = contract_msg
    total_elapsed_ms = sum(int(row.get("elapsed_ms", 0)) for row in rows)

    payload = {
        "schema": "ddn.ci.sync_readiness.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "started_at_utc": started,
        "status": "pass" if all_ok else "fail",
        "ok": all_ok,
        "code": status_code if not all_ok else SYNC_READINESS_OK,
        "step": status_step if not all_ok else "all",
        "msg": status_msg if not all_ok else "-",
        "report_dir": str(report_dir),
        "report_prefix": prefix,
        "skip_aggregate": bool(args.skip_aggregate),
        "validate_only_sanity_json": validate_only_sanity_json,
        "steps": rows,
        "steps_count": len(rows),
        "total_elapsed_ms": total_elapsed_ms,
    }

    out_path = Path(args.json_out) if args.json_out.strip() else (report_dir / f"{prefix}.ci_sync_readiness.detjson")
    write_json(out_path, payload)
    status = "pass" if all_ok else "fail"
    msg_json = json.dumps(clip(status_msg if not all_ok else "-", 220), ensure_ascii=False)
    print(
        f'ci_sync_readiness_status={status} ok={1 if all_ok else 0} '
        f'code={status_code if not all_ok else SYNC_READINESS_OK} '
        f'step={status_step if not all_ok else "all"} msg={msg_json} '
        f'steps={len(rows)} total_elapsed_ms={total_elapsed_ms} report="{out_path}"'
    )
    return 0 if all_ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
