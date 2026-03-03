#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


MATRIX_SCHEMA = "ddn.ci.profile_matrix_gate.v1"
MATRIX_OK = "OK"
MATRIX_PROFILE_INVALID = "E_CI_PROFILE_MATRIX_PROFILE_INVALID"
MATRIX_STEP_FAIL = "E_CI_PROFILE_MATRIX_STEP_FAIL"

VALID_PROFILES = ("core_lang", "full", "seamgrim")
PROFILE_GATE_SCRIPTS = {
    "core_lang": "tests/run_ci_profile_core_lang_gate.py",
    "full": "tests/run_ci_profile_full_gate.py",
    "seamgrim": "tests/run_ci_profile_seamgrim_gate.py",
}
PROFILE_PASS_MARKERS = {
    "core_lang": "ci_profile_core_lang_status=pass",
    "full": "ci_profile_full_status=pass",
    "seamgrim": "ci_profile_seamgrim_status=pass",
}


def clip(text: str, limit: int = 180) -> str:
    normalized = " ".join(str(text).split())
    if not normalized:
        return "-"
    if len(normalized) <= limit:
        return normalized
    return normalized[: limit - 3] + "..."


def parse_profiles(raw: str) -> tuple[list[str], list[str]]:
    seen: set[str] = set()
    ordered: list[str] = []
    invalid: list[str] = []
    for token in str(raw).split(","):
        name = token.strip()
        if not name:
            continue
        if name not in VALID_PROFILES:
            if name not in invalid:
                invalid.append(name)
            continue
        if name in seen:
            continue
        seen.add(name)
        ordered.append(name)
    return ordered, invalid


def run_step(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="Run CI profile gates as a single matrix entrypoint")
    parser.add_argument(
        "--profiles",
        default="core_lang,full,seamgrim",
        help="comma-separated profiles (core_lang,full,seamgrim)",
    )
    parser.add_argument("--report-dir", default="build/reports", help="report directory")
    parser.add_argument("--report-prefix", default="dev_ci_profile_matrix", help="report file prefix")
    parser.add_argument("--json-out", default="", help="optional explicit matrix report path")
    parser.add_argument("--dry-run", action="store_true", help="print planned matrix steps only")
    parser.add_argument("--stop-on-fail", action="store_true", help="stop matrix on first failed profile step")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    profiles, invalid_profiles = parse_profiles(args.profiles)
    if not profiles:
        invalid_profiles = invalid_profiles or ["(empty)"]

    report_dir = Path(args.report_dir)
    report_dir.mkdir(parents=True, exist_ok=True)
    out_path = Path(args.json_out) if args.json_out.strip() else (report_dir / f"{args.report_prefix}.ci_profile_matrix_gate.detjson")

    rows: list[dict[str, object]] = []
    status = "pass"
    code = MATRIX_OK
    step = "all"
    msg = "-"

    if invalid_profiles:
        status = "fail"
        code = MATRIX_PROFILE_INVALID
        step = "profile_list"
        msg = f"invalid profiles: {','.join(invalid_profiles)}"
    else:
        for profile in profiles:
            script_rel = PROFILE_GATE_SCRIPTS[profile]
            script = root / script_rel
            cmd = [py, script_rel]
            if not script.exists():
                row = {
                    "profile": profile,
                    "script": script_rel,
                    "ok": False,
                    "returncode": 127,
                    "cmd": cmd,
                    "stdout_head": "-",
                    "stderr_head": f"missing script: {script}",
                }
                rows.append(row)
                status = "fail"
                code = MATRIX_STEP_FAIL
                step = profile
                msg = f"missing script: {script_rel}"
                if args.stop_on_fail:
                    break
                continue

            if args.dry_run:
                row = {
                    "profile": profile,
                    "script": script_rel,
                    "ok": True,
                    "returncode": 0,
                    "cmd": cmd,
                    "stdout_head": "[dry-run]",
                    "stderr_head": "-",
                }
                rows.append(row)
                continue

            proc = run_step(cmd, root)
            stdout = proc.stdout or ""
            stderr = proc.stderr or ""
            if stdout.strip():
                print(stdout, end="" if stdout.endswith("\n") else "\n")
            if stderr.strip():
                print(stderr, end="" if stderr.endswith("\n") else "\n", file=sys.stderr)
            marker = PROFILE_PASS_MARKERS[profile]
            ok = proc.returncode == 0 and marker in stdout
            row = {
                "profile": profile,
                "script": script_rel,
                "ok": ok,
                "returncode": int(proc.returncode),
                "cmd": cmd,
                "stdout_head": clip(stdout),
                "stderr_head": clip(stderr),
            }
            rows.append(row)
            if not ok:
                status = "fail"
                code = MATRIX_STEP_FAIL
                step = profile
                msg = f"profile step failed: {profile}"
                if args.stop_on_fail:
                    break

    payload = {
        "schema": MATRIX_SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": status,
        "ok": status == "pass",
        "code": code,
        "step": step,
        "msg": msg,
        "profiles": profiles,
        "invalid_profiles": invalid_profiles,
        "steps": rows,
        "dry_run": bool(args.dry_run),
    }
    write_json(out_path, payload)

    print(
        "ci_profile_matrix_status={} code={} step={} profiles={} msg=\"{}\" report=\"{}\"".format(
            status,
            code,
            step,
            ",".join(profiles) if profiles else "-",
            msg,
            out_path,
        )
    )
    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
