#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


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
    print(f"check=ci_sync_readiness_selftest detail={msg}")
    if proc is not None:
        if (proc.stdout or "").strip():
            print(proc.stdout.strip())
        if (proc.stderr or "").strip():
            print(proc.stderr.strip())
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    py = sys.executable

    with tempfile.TemporaryDirectory(prefix="ci_sync_readiness_selftest_") as td:
        report_dir = Path(td) / "reports"
        report_dir.mkdir(parents=True, exist_ok=True)
        prefix = "sync_selftest"
        expected_default_report = report_dir / f"{prefix}.ci_sync_readiness.detjson"

        quick_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            prefix,
            "--skip-aggregate",
        ]
        quick_proc = run(quick_cmd, cwd=root)
        if expect(quick_proc.returncode == 0, "quick_mode_should_pass", quick_proc) != 0:
            return 1
        if expect(expected_default_report.exists(), "default_report_missing", quick_proc) != 0:
            return 1
        doc = load_json(expected_default_report)
        if expect(str(doc.get("schema", "")) == "ddn.ci.sync_readiness.v1", "schema_mismatch") != 0:
            return 1
        if expect(str(doc.get("status", "")) == "pass", "status_should_be_pass") != 0:
            return 1
        if expect(bool(doc.get("ok", False)), "ok_should_be_true") != 0:
            return 1
        if expect(str(doc.get("code", "")) == "OK", "code_should_be_ok") != 0:
            return 1
        if expect(str(doc.get("sanity_profile", "")) == "full", "sanity_profile_should_be_full") != 0:
            return 1
        if expect(str(doc.get("step", "")) == "all", "step_should_be_all") != 0:
            return 1
        if expect(bool(doc.get("skip_aggregate", False)), "skip_aggregate_should_be_true") != 0:
            return 1
        if expect(int(doc.get("steps_count", 0)) == 5, "steps_count_quick_should_be_5") != 0:
            return 1
        steps = doc.get("steps")
        if expect(isinstance(steps, list), "steps_should_be_list") != 0:
            return 1
        step_names = [str(row.get("name", "")) for row in steps if isinstance(row, dict)]
        if expect(
            step_names
            == [
                "pipeline_emit_flags_check",
                "pipeline_emit_flags_selftest",
                "sanity_gate_diagnostics_check",
                "sanity_gate",
                "sanity_gate_contract",
            ],
            "unexpected_quick_steps",
        ) != 0:
            return 1
        if expect(bool(steps[-1].get("ok", False)), "sanity_gate_contract_should_be_ok") != 0:
            return 1

        custom_json = report_dir / "sync_readiness.custom.detjson"
        custom_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            f"{prefix}_custom",
            "--skip-aggregate",
            "--json-out",
            str(custom_json),
        ]
        custom_proc = run(custom_cmd, cwd=root)
        if expect(custom_proc.returncode == 0, "custom_json_mode_should_pass", custom_proc) != 0:
            return 1
        if expect(custom_json.exists(), "custom_json_report_missing", custom_proc) != 0:
            return 1
        custom_doc = load_json(custom_json)
        if expect(str(custom_doc.get("status", "")) == "pass", "custom_status_should_be_pass") != 0:
            return 1
        if expect(bool(custom_doc.get("ok", False)), "custom_ok_should_be_true") != 0:
            return 1
        if expect(str(custom_doc.get("code", "")) == "OK", "custom_code_should_be_ok") != 0:
            return 1
        if expect(str(custom_doc.get("sanity_profile", "")) == "full", "custom_sanity_profile_should_be_full") != 0:
            return 1
        if expect(int(custom_doc.get("steps_count", 0)) == 5, "custom_steps_count_should_be_5") != 0:
            return 1

        sanity_json = report_dir / f"{prefix}.ci_sanity_gate.detjson"
        validate_ok_json = report_dir / "sync_readiness.validate_ok.detjson"
        validate_ok_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            f"{prefix}_validate_ok",
            "--validate-only-sanity-json",
            str(sanity_json),
            "--json-out",
            str(validate_ok_json),
        ]
        validate_ok_proc = run(validate_ok_cmd, cwd=root)
        if expect(validate_ok_proc.returncode == 0, "validate_only_ok_should_pass", validate_ok_proc) != 0:
            return 1
        validate_ok_doc = load_json(validate_ok_json)
        if expect(str(validate_ok_doc.get("status", "")) == "pass", "validate_only_status_should_be_pass") != 0:
            return 1
        if expect(str(validate_ok_doc.get("code", "")) == "OK", "validate_only_code_should_be_ok") != 0:
            return 1
        if expect(str(validate_ok_doc.get("sanity_profile", "")) == "full", "validate_only_sanity_profile_should_be_full") != 0:
            return 1
        if expect(int(validate_ok_doc.get("steps_count", 0)) == 1, "validate_only_steps_count_should_be_1") != 0:
            return 1
        validate_ok_steps = validate_ok_doc.get("steps")
        if expect(isinstance(validate_ok_steps, list), "validate_only_steps_should_be_list") != 0:
            return 1
        if expect(
            len(validate_ok_steps) == 1 and str(validate_ok_steps[0].get("name", "")) == "sanity_gate_contract",
            "validate_only_step_name_should_be_sanity_gate_contract",
        ) != 0:
            return 1

        bad_sanity_json = report_dir / "sync_readiness.bad_sanity.detjson"
        write_json(
            bad_sanity_json,
            {
                "schema": "ddn.ci.sanity_gate.v1",
                "status": "pass",
                "code": "OK",
                "step": "all",
                "msg": "-",
                "steps": [
                    {
                        "step": "backup_hygiene_selftest",
                        "ok": True,
                        "returncode": 0,
                        "cmd": ["python", "x.py"],
                    }
                ],
            },
        )
        validate_bad_json = report_dir / "sync_readiness.validate_bad.detjson"
        validate_bad_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            f"{prefix}_validate_bad",
            "--validate-only-sanity-json",
            str(bad_sanity_json),
            "--json-out",
            str(validate_bad_json),
        ]
        validate_bad_proc = run(validate_bad_cmd, cwd=root)
        if expect(validate_bad_proc.returncode != 0, "validate_only_bad_should_fail", validate_bad_proc) != 0:
            return 1
        validate_bad_doc = load_json(validate_bad_json)
        if expect(str(validate_bad_doc.get("status", "")) == "fail", "validate_bad_status_should_be_fail") != 0:
            return 1
        if expect(
            str(validate_bad_doc.get("code", "")) == "E_SYNC_READINESS_SANITY_CONTRACT_FAIL",
            "validate_bad_code_should_be_sanity_contract_fail",
        ) != 0:
            return 1
        if expect(
            str(validate_bad_doc.get("step", "")) == "sanity_gate_contract",
            "validate_bad_step_should_be_sanity_gate_contract",
        ) != 0:
            return 1

        missing_validate_json = report_dir / "sync_readiness.validate_missing.detjson"
        missing_validate_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            f"{prefix}_validate_missing",
            "--validate-only-sanity-json",
            str(report_dir / "not_found.ci_sanity_gate.detjson"),
            "--json-out",
            str(missing_validate_json),
        ]
        missing_validate_proc = run(missing_validate_cmd, cwd=root)
        if expect(missing_validate_proc.returncode != 0, "validate_only_missing_should_fail", missing_validate_proc) != 0:
            return 1
        missing_validate_doc = load_json(missing_validate_json)
        if expect(str(missing_validate_doc.get("status", "")) == "fail", "validate_missing_status_should_be_fail") != 0:
            return 1
        if expect(
            str(missing_validate_doc.get("code", "")) == "E_SYNC_READINESS_VALIDATE_ONLY_PATH_MISSING",
            "validate_missing_code_should_be_path_missing",
        ) != 0:
            return 1
        if expect(str(missing_validate_doc.get("step", "")) == "validate_only", "validate_missing_step_should_be_validate_only") != 0:
            return 1

    print("ci sync readiness check selftest ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
