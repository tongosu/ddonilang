#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

from _ci_profile_matrix_full_real_smoke_contract import (
    PROFILE_MATRIX_FULL_REAL_SMOKE_ALLOW_FLAG,
    PROFILE_MATRIX_FULL_REAL_SMOKE_MODE_MARKER,
    PROFILE_MATRIX_FULL_REAL_SMOKE_SELFTEST_SCRIPT_OVERRIDE_ENV_KEY,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STATUS_PASS,
    PROFILE_MATRIX_GATE_SELFTEST_FULL_REAL_FLAG,
    PROFILE_MATRIX_GATE_SELFTEST_FULL_REAL_TOKENS,
    PROFILE_MATRIX_GATE_SELFTEST_REAL_PROFILES,
    PROFILE_MATRIX_GATE_SELFTEST_REAL_PROFILES_FLAG,
    PROFILE_MATRIX_GATE_SELFTEST_SCRIPT,
    PROFILE_MATRIX_GATE_SELFTEST_OK_MARKER,
    PROFILE_MATRIX_GATE_SELFTEST_REAL_PROFILES_MARKER,
    PROFILE_MATRIX_GATE_SELFTEST_SKIPPED_REAL_PROFILES_MARKER,
    PROFILE_MATRIX_GATE_SELFTEST_LIGHTWEIGHT_FALSE_MARKER,
)

FAKE_GATE_PROGRESS_ENV_KEY = "DDN_PROFILE_MATRIX_FULL_REAL_SMOKE_FAKE_GATE_PROGRESS_JSON"


def run(cmd: list[str], cwd: Path, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def emit(proc: subprocess.CompletedProcess[str]) -> tuple[str, str]:
    stdout = (proc.stdout or "").strip()
    stderr = (proc.stderr or "").strip()
    if stdout:
        print(stdout)
    if stderr:
        print(stderr)
    return stdout, stderr


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def write_override_child_progress(stage: str) -> None:
    path_text = str(os.environ.get(FAKE_GATE_PROGRESS_ENV_KEY, "")).strip()
    if not path_text:
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.ci.fake_gate_selftest.progress.v1",
        "stage": stage,
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def print_elapsed_markers(report_path: Path) -> None:
    payload = load_json(report_path)
    if not isinstance(payload, dict):
        print("ci_profile_matrix_full_real_total_elapsed_ms=-")
        print("ci_profile_matrix_full_real_slowest_profile=-")
        print("ci_profile_matrix_full_real_slowest_elapsed_ms=-")
        print("ci_profile_matrix_full_real_profile_status_map=-")
        return
    total_elapsed_ms = str(payload.get("total_elapsed_ms", "-")).strip() or "-"
    steps = payload.get("steps")
    slowest_profile = "-"
    slowest_elapsed_ms = "-"
    elapsed_map = {
        "core_lang": "-",
        "full": "-",
        "seamgrim": "-",
    }
    status_map = {
        "core_lang": "-",
        "full": "-",
        "seamgrim": "-",
    }
    if isinstance(steps, list):
        ranked: list[tuple[int, str]] = []
        for row in steps:
            if not isinstance(row, dict):
                continue
            profile = str(row.get("profile", "")).strip()
            if not profile:
                continue
            try:
                elapsed_ms = int(row.get("elapsed_ms", 0))
            except Exception:
                elapsed_ms = 0
            ranked.append((elapsed_ms, profile))
            elapsed_map[profile] = str(elapsed_ms)
            status_map[profile] = "pass" if bool(row.get("ok", False)) else "fail"
        if ranked:
            ranked.sort(key=lambda item: (-item[0], item[1]))
            slowest_elapsed_ms = str(ranked[0][0])
            slowest_profile = ranked[0][1]
    elapsed_map_text = ",".join(
        f"{name}:{elapsed_map.get(name, '-')}" for name in ("core_lang", "full", "seamgrim")
    )
    status_map_text = ",".join(
        f"{name}:{status_map.get(name, '-')}" for name in ("core_lang", "full", "seamgrim")
    )
    print(f"ci_profile_matrix_full_real_total_elapsed_ms={total_elapsed_ms}")
    print(f"ci_profile_matrix_full_real_slowest_profile={slowest_profile}")
    print(f"ci_profile_matrix_full_real_slowest_elapsed_ms={slowest_elapsed_ms}")
    print(f"ci_profile_matrix_full_real_profile_elapsed_map={elapsed_map_text}")
    print(f"ci_profile_matrix_full_real_profile_status_map={status_map_text}")


def resolve_gate_selftest_script() -> str:
    override = str(os.environ.get(PROFILE_MATRIX_FULL_REAL_SMOKE_SELFTEST_SCRIPT_OVERRIDE_ENV_KEY, "")).strip()
    if override:
        return override
    return PROFILE_MATRIX_GATE_SELFTEST_SCRIPT


def main() -> int:
    parser = argparse.ArgumentParser(description="Run heavy profile-matrix full-real 3-profile smoke")
    parser.add_argument(
        PROFILE_MATRIX_FULL_REAL_SMOKE_ALLOW_FLAG,
        action="store_true",
        help="explicitly allow heavy nightly/manual-only full-real smoke execution",
    )
    parser.add_argument(
        "--step-timeout-sec",
        type=float,
        default=0.0,
        help="optional per-profile timeout passed to profile_matrix_gate in direct smoke mode",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    py = sys.executable

    print(PROFILE_MATRIX_FULL_REAL_SMOKE_MODE_MARKER)
    if not args.allow_heavy_smoke:
        print("ci_profile_matrix_full_real_smoke_status=fail reason=explicit_optin_required")
        return 1

    gate_selftest_script = resolve_gate_selftest_script()
    override_active = bool(
        str(os.environ.get(PROFILE_MATRIX_FULL_REAL_SMOKE_SELFTEST_SCRIPT_OVERRIDE_ENV_KEY, "")).strip()
    )
    if override_active:
        write_override_child_progress("smoke_check_prepare_child")
        proc = run(
            [
                py,
                gate_selftest_script,
                PROFILE_MATRIX_GATE_SELFTEST_FULL_REAL_FLAG,
                PROFILE_MATRIX_GATE_SELFTEST_REAL_PROFILES_FLAG,
                PROFILE_MATRIX_GATE_SELFTEST_REAL_PROFILES,
            ],
            root,
        )
        write_override_child_progress("smoke_check_child_finished")
        stdout, _ = emit(proc)
        if proc.returncode != 0:
            print("ci_profile_matrix_full_real_smoke_status=fail reason=selftest_failed")
            return proc.returncode

        required_markers = tuple(PROFILE_MATRIX_GATE_SELFTEST_FULL_REAL_TOKENS[4:])
        for marker in required_markers:
            if marker not in stdout:
                print(f"ci_profile_matrix_full_real_smoke_status=fail reason=marker_missing marker={marker}")
                return 1
    else:
        with tempfile.TemporaryDirectory(prefix="ci_profile_matrix_full_real_smoke_") as td:
            report_dir = Path(td) / "reports"
            report_dir.mkdir(parents=True, exist_ok=True)
            report_path = report_dir / "profile_matrix_full_real_smoke.detjson"
            env = dict(os.environ)
            env["DDN_CI_PROFILE_GATE_FULL_AGGREGATE"] = "1"
            cmd = [
                py,
                "tests/run_ci_profile_matrix_gate.py",
                "--profiles",
                PROFILE_MATRIX_GATE_SELFTEST_REAL_PROFILES,
                "--report-dir",
                str(report_dir),
                "--report-prefix",
                "profile_matrix_full_real_smoke",
                "--json-out",
                str(report_path),
                "--stop-on-fail",
                "--full-aggregate-gates",
            ]
            if float(args.step_timeout_sec) > 0.0:
                cmd.extend(["--step-timeout-sec", str(float(args.step_timeout_sec))])
            proc = run(cmd, root, env=env)
            stdout, stderr = emit(proc)
            print_elapsed_markers(report_path)
            if proc.returncode != 0:
                merged = "\n".join(part for part in (stdout, stderr) if part)
                if "step timeout after" in merged:
                    print("ci_profile_matrix_full_real_smoke_status=fail reason=selftest_failed")
                    return 124
                print("ci_profile_matrix_full_real_smoke_status=fail reason=selftest_failed")
                return proc.returncode
            if "ci_profile_matrix_status=pass" not in stdout:
                print("ci_profile_matrix_full_real_smoke_status=fail reason=marker_missing marker=ci_profile_matrix_status=pass")
                return 1
            print(PROFILE_MATRIX_GATE_SELFTEST_REAL_PROFILES_MARKER)
            print(PROFILE_MATRIX_GATE_SELFTEST_SKIPPED_REAL_PROFILES_MARKER)
            print(PROFILE_MATRIX_GATE_SELFTEST_LIGHTWEIGHT_FALSE_MARKER)
            print(PROFILE_MATRIX_GATE_SELFTEST_OK_MARKER)

    print(PROFILE_MATRIX_FULL_REAL_SMOKE_STATUS_PASS)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
