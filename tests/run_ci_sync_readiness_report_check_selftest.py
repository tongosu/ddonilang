#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from ci_check_error_codes import SYNC_READINESS_REPORT_CODES as CODES


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


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def expect(cond: bool, msg: str, proc: subprocess.CompletedProcess[str] | None = None) -> int:
    if cond:
        return 0
    print(f"check=ci_sync_readiness_report_selftest detail={msg}")
    if proc is not None:
        if (proc.stdout or "").strip():
            print(proc.stdout.strip())
        if (proc.stderr or "").strip():
            print(proc.stderr.strip())
    return 1


def expect_fail_code(proc: subprocess.CompletedProcess[str], code: str, msg: str) -> int:
    return expect(f"fail code={code}" in (proc.stderr or ""), msg, proc)


def run_report_check(
    py: str,
    root: Path,
    report: Path,
    require_pass: bool = False,
    sanity_profile: str = "",
) -> subprocess.CompletedProcess[str]:
    cmd = [py, "tests/run_ci_sync_readiness_report_check.py", "--report", str(report)]
    if require_pass:
        cmd.append("--require-pass")
    if sanity_profile:
        cmd.extend(["--sanity-profile", sanity_profile])
    return run(cmd, root)


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    py = sys.executable

    with tempfile.TemporaryDirectory(prefix="ci_sync_readiness_report_selftest_") as td:
        report_dir = Path(td) / "reports"
        report_dir.mkdir(parents=True, exist_ok=True)

        prefix = "sync_report_selftest"
        run_proc = run(
            [
                py,
                "tests/run_ci_sync_readiness_check.py",
                "--report-dir",
                str(report_dir),
                "--report-prefix",
                prefix,
                "--skip-aggregate",
            ],
            root,
        )
        if expect(run_proc.returncode == 0, "sync_readiness_run_should_pass", run_proc) != 0:
            return 1
        report = report_dir / f"{prefix}.ci_sync_readiness.detjson"
        if expect(report.exists(), "sync_readiness_report_missing", run_proc) != 0:
            return 1
        report_doc = load_json(report)
        if expect(str(report_doc.get("sanity_profile", "")) == "full", "sanity_profile_should_be_full") != 0:
            return 1
        proc_ok = run_report_check(py, root, report, require_pass=True)
        if expect(proc_ok.returncode == 0, "report_check_should_pass", proc_ok) != 0:
            return 1
        proc_profile_mismatch = run_report_check(py, root, report, require_pass=True, sanity_profile="seamgrim")
        if expect(proc_profile_mismatch.returncode != 0, "report_profile_mismatch_should_fail", proc_profile_mismatch) != 0:
            return 1
        if (
            expect_fail_code(
                proc_profile_mismatch,
                CODES["STATUS_OK_MISMATCH"],
                "report_profile_mismatch_fail_code_should_match",
            )
            != 0
        ):
            return 1

        sanity_json = report_dir / f"{prefix}.ci_sanity_gate.detjson"
        validate_prefix = "sync_report_validate"
        validate_proc = run(
            [
                py,
                "tests/run_ci_sync_readiness_check.py",
                "--report-dir",
                str(report_dir),
                "--report-prefix",
                validate_prefix,
                "--validate-only-sanity-json",
                str(sanity_json),
            ],
            root,
        )
        if expect(validate_proc.returncode == 0, "validate_only_should_pass", validate_proc) != 0:
            return 1
        validate_report = report_dir / f"{validate_prefix}.ci_sync_readiness.detjson"
        proc_validate = run_report_check(py, root, validate_report, require_pass=True)
        if expect(proc_validate.returncode == 0, "validate_only_report_check_should_pass", proc_validate) != 0:
            return 1

        bad_code_report = report_dir / "bad_code.ci_sync_readiness.detjson"
        bad_code_doc = load_json(report)
        bad_code_doc["code"] = "BROKEN"
        write_json(bad_code_report, bad_code_doc)
        proc_bad_code = run_report_check(py, root, bad_code_report, require_pass=True)
        if expect(proc_bad_code.returncode != 0, "bad_code_should_fail", proc_bad_code) != 0:
            return 1
        if expect_fail_code(proc_bad_code, CODES["PASS_STATUS_FIELDS"], "bad_code_fail_code_should_match") != 0:
            return 1

        bad_contract_report = report_dir / "bad_contract_row.ci_sync_readiness.detjson"
        bad_contract_doc = load_json(report)
        bad_contract_doc["steps"] = [row for row in bad_contract_doc.get("steps", []) if row.get("name") != "sanity_gate_contract"]
        bad_contract_doc["steps_count"] = len(bad_contract_doc["steps"])
        write_json(bad_contract_report, bad_contract_doc)
        proc_bad_contract = run_report_check(py, root, bad_contract_report, require_pass=True)
        if expect(proc_bad_contract.returncode != 0, "bad_contract_row_should_fail", proc_bad_contract) != 0:
            return 1
        if (
            expect_fail_code(
                proc_bad_contract,
                CODES["MISSING_CONTRACT_ROW"],
                "bad_contract_row_fail_code_should_match",
            )
            != 0
        ):
            return 1

        bad_validate_shape_report = report_dir / "bad_validate_shape.ci_sync_readiness.detjson"
        bad_validate_shape_doc = load_json(validate_report)
        bad_validate_shape_doc["steps"] = [
            {
                "name": "sanity_gate_contract",
                "ok": True,
                "returncode": 0,
                "elapsed_ms": 0,
                "cmd": ["internal", "validate_sanity_contract", "x"],
                "stdout_head": "ok",
                "stderr_head": "-",
            },
            {
                "name": "extra_step",
                "ok": True,
                "returncode": 0,
                "elapsed_ms": 0,
                "cmd": ["python", "x.py"],
                "stdout_head": "-",
                "stderr_head": "-",
            },
        ]
        bad_validate_shape_doc["steps_count"] = 2
        write_json(bad_validate_shape_report, bad_validate_shape_doc)
        proc_bad_validate_shape = run_report_check(py, root, bad_validate_shape_report, require_pass=True)
        if expect(proc_bad_validate_shape.returncode != 0, "bad_validate_shape_should_fail", proc_bad_validate_shape) != 0:
            return 1
        if (
            expect_fail_code(
                proc_bad_validate_shape,
                CODES["VALIDATE_ONLY_SHAPE"],
                "bad_validate_shape_fail_code_should_match",
            )
            != 0
        ):
            return 1

    print("ci sync readiness report check selftest ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
