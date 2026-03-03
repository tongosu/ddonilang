#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from ci_check_error_codes import CI_PROFILE_MATRIX_CODES as CODES


MATRIX_SCHEMA = "ddn.ci.profile_matrix_gate.v1"


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def expect(cond: bool, msg: str, proc: subprocess.CompletedProcess[str] | None = None) -> int:
    if cond:
        return 0
    print(f"check=ci_profile_matrix_gate_selftest detail={msg}")
    if proc is not None:
        if (proc.stdout or "").strip():
            print(proc.stdout.strip())
        if (proc.stderr or "").strip():
            print(proc.stderr.strip())
    return 1


def expect_marker(proc: subprocess.CompletedProcess[str], token: str, msg: str) -> int:
    return expect(token in (proc.stdout or ""), msg, proc)


def run_matrix(
    py: str,
    root: Path,
    report_path: Path,
    profiles: str,
    dry_run: bool,
    stop_on_fail: bool = False,
) -> subprocess.CompletedProcess[str]:
    cmd = [
        py,
        "tests/run_ci_profile_matrix_gate.py",
        "--profiles",
        profiles,
        "--report-dir",
        str(report_path.parent),
        "--report-prefix",
        "profile_matrix_selftest",
        "--json-out",
        str(report_path),
    ]
    if dry_run:
        cmd.append("--dry-run")
    if stop_on_fail:
        cmd.append("--stop-on-fail")
    return run(cmd, root)


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    py = sys.executable

    with tempfile.TemporaryDirectory(prefix="ci_profile_matrix_gate_selftest_") as td:
        report_dir = Path(td) / "reports"
        report_dir.mkdir(parents=True, exist_ok=True)

        dry_report = report_dir / "matrix_dry.detjson"
        proc_dry = run_matrix(
            py=py,
            root=root,
            report_path=dry_report,
            profiles="core_lang,full,seamgrim",
            dry_run=True,
        )
        if expect(proc_dry.returncode == 0, "dry_run_should_pass", proc_dry) != 0:
            return 1
        if expect_marker(proc_dry, "ci_profile_matrix_status=pass", "dry_run_status_marker_missing") != 0:
            return 1
        if expect_marker(proc_dry, "code=OK", "dry_run_code_marker_missing") != 0:
            return 1
        if expect(dry_report.exists(), "dry_run_report_missing", proc_dry) != 0:
            return 1
        dry_doc = load_json(dry_report)
        if expect(str(dry_doc.get("schema", "")) == MATRIX_SCHEMA, "dry_run_schema_mismatch") != 0:
            return 1
        if expect(str(dry_doc.get("status", "")) == "pass", "dry_run_status_mismatch") != 0:
            return 1
        if expect(str(dry_doc.get("code", "")) == "OK", "dry_run_code_mismatch") != 0:
            return 1
        if expect(bool(dry_doc.get("dry_run", False)), "dry_run_flag_mismatch") != 0:
            return 1
        if expect(
            list(dry_doc.get("profiles", [])) == ["core_lang", "full", "seamgrim"],
            "dry_run_profiles_mismatch",
        ) != 0:
            return 1
        if expect(int(len(dry_doc.get("steps", []))) == 3, "dry_run_steps_count_mismatch") != 0:
            return 1

        invalid_report = report_dir / "matrix_invalid.detjson"
        proc_invalid = run_matrix(
            py=py,
            root=root,
            report_path=invalid_report,
            profiles="core_lang,broken",
            dry_run=True,
        )
        if expect(proc_invalid.returncode != 0, "invalid_profile_should_fail", proc_invalid) != 0:
            return 1
        if (
            expect_marker(
                proc_invalid,
                f"code={CODES['PROFILE_INVALID']}",
                "invalid_profile_code_marker_missing",
            )
            != 0
        ):
            return 1
        invalid_doc = load_json(invalid_report)
        if expect(str(invalid_doc.get("status", "")) == "fail", "invalid_profile_status_mismatch") != 0:
            return 1
        if (
            expect(
                str(invalid_doc.get("code", "")) == CODES["PROFILE_INVALID"],
                "invalid_profile_code_mismatch",
            )
            != 0
        ):
            return 1
        if expect(list(invalid_doc.get("invalid_profiles", [])) == ["broken"], "invalid_profile_list_mismatch") != 0:
            return 1

        dedupe_report = report_dir / "matrix_dedupe.detjson"
        proc_dedupe = run_matrix(
            py=py,
            root=root,
            report_path=dedupe_report,
            profiles="full,core_lang,full,seamgrim",
            dry_run=True,
        )
        if expect(proc_dedupe.returncode == 0, "dedupe_case_should_pass", proc_dedupe) != 0:
            return 1
        dedupe_doc = load_json(dedupe_report)
        if expect(
            list(dedupe_doc.get("profiles", [])) == ["full", "core_lang", "seamgrim"],
            "dedupe_profiles_order_mismatch",
        ) != 0:
            return 1

        real_report = report_dir / "matrix_real_core_lang.detjson"
        proc_real = run_matrix(
            py=py,
            root=root,
            report_path=real_report,
            profiles="core_lang",
            dry_run=False,
            stop_on_fail=True,
        )
        if expect(proc_real.returncode == 0, "real_core_lang_should_pass", proc_real) != 0:
            return 1
        if expect_marker(proc_real, "ci_profile_core_lang_status=pass", "real_core_lang_profile_marker_missing") != 0:
            return 1
        if expect_marker(proc_real, "ci_profile_matrix_status=pass", "real_core_lang_matrix_marker_missing") != 0:
            return 1
        real_doc = load_json(real_report)
        if expect(bool(real_doc.get("ok", False)), "real_core_lang_ok_mismatch") != 0:
            return 1
        if expect(str(real_doc.get("step", "")) == "all", "real_core_lang_step_mismatch") != 0:
            return 1
        if expect(int(len(real_doc.get("steps", []))) == 1, "real_core_lang_steps_count_mismatch") != 0:
            return 1

    print("ci profile matrix gate selftest ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
