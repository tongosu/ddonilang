#!/usr/bin/env python
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path


def write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def cached_child_report(required_criterion: str) -> dict[str, object]:
    return {
        "schema": "ddn.age5_close_report.v1",
        "overall_ok": True,
        "criteria": [
            {
                "name": required_criterion,
                "ok": True,
                "detail": "cached pass",
            }
        ],
    }


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    invalid_cmd = [
        sys.executable,
        "tests/run_age5_close.py",
        "--combined-heavy-child-timeout-sec",
        "1",
    ]
    invalid_proc = subprocess.run(
        invalid_cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=60,
    )
    if invalid_proc.returncode != 2:
        print("[age5-close-combined-heavy-timeout-selftest] fail: invalid timeout usage rc mismatch")
        return 1
    invalid_stderr = str(invalid_proc.stderr or "")
    if "--combined-heavy-child-timeout-sec requires combined-heavy opt-in" not in invalid_stderr:
        print("[age5-close-combined-heavy-timeout-selftest] fail: invalid timeout usage stderr missing")
        return 1

    with tempfile.TemporaryDirectory(prefix="age5_combined_timeout_") as td:
        temp_root = Path(td)
        report_out = temp_root / "age5_close_report.timeout.detjson"
        runtime_helper_report = temp_root / "age5_close_report.timeout.runtime_helper_negative.detjson"
        group_id_report = temp_root / "age5_close_report.timeout.group_id_summary_negative.detjson"
        write_json(
            runtime_helper_report,
            cached_child_report("age5_ci_profile_core_lang_runtime_helper_negative_optin_pass"),
        )
        write_json(
            group_id_report,
            cached_child_report("age5_ci_profile_core_lang_group_id_summary_negative_optin_pass"),
        )
        sleep_script = temp_root / "sleep_forever.py"
        sleep_script.write_text(
            "import time\n"
            "time.sleep(60)\n",
            encoding="utf-8",
        )
        env = dict(os.environ)
        env["DDN_CI_PROFILE_MATRIX_FULL_REAL_SMOKE_SELFTEST_SCRIPT_OVERRIDE"] = str(sleep_script)
        cmd = [
            sys.executable,
            "tests/run_age5_close.py",
            "--with-combined-heavy-runtime-helper-check",
            "--combined-heavy-child-timeout-sec",
            "1",
            "--report-out",
            str(report_out),
        ]
        proc = subprocess.run(
            cmd,
            cwd=root,
            env=env,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=60,
        )
        if proc.returncode == 0:
            print("[age5-close-combined-heavy-timeout-selftest] fail: combined run unexpectedly passed")
            return 1
        if not report_out.exists():
            print("[age5-close-combined-heavy-timeout-selftest] fail: report missing")
            return 1
        report = json.loads(report_out.read_text(encoding="utf-8"))
        if int(report.get("combined_heavy_child_timeout_sec", 0)) != 1:
            print("[age5-close-combined-heavy-timeout-selftest] fail: timeout field mismatch")
            return 1
        reused = report.get("reused_child_reports")
        if not isinstance(reused, dict):
            print("[age5-close-combined-heavy-timeout-selftest] fail: reused_child_reports missing")
            return 1
        if reused.get("runtime_helper_negative") is not True or reused.get("group_id_summary_negative") is not True:
            print("[age5-close-combined-heavy-timeout-selftest] fail: cached child reports were not reused")
            return 1
        criteria = report.get("criteria")
        if not isinstance(criteria, list):
            print("[age5-close-combined-heavy-timeout-selftest] fail: criteria missing")
            return 1
        by_name = {
            str(row.get("name", "")).strip(): row
            for row in criteria
            if isinstance(row, dict)
        }
        full_real = by_name.get("age5_ci_profile_matrix_full_real_smoke_optin_pass")
        if not isinstance(full_real, dict):
            print("[age5-close-combined-heavy-timeout-selftest] fail: full_real criterion missing")
            return 1
        if bool(full_real.get("ok", False)):
            print("[age5-close-combined-heavy-timeout-selftest] fail: full_real criterion unexpectedly passed")
            return 1
        detail = str(full_real.get("detail", "")).strip()
        if "rc=124" not in detail or "child timeout" not in detail:
            print("[age5-close-combined-heavy-timeout-selftest] fail: timeout detail missing")
            return 1
        print("[age5-close-combined-heavy-timeout-selftest] ok")
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
