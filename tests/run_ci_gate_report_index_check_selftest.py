#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from ci_check_error_codes import GATE_REPORT_INDEX_CODES as CODES

REQUIRED_STEPS_COMMON = (
    "ci_profile_split_contract_check",
    "ci_profile_matrix_gate_selftest",
    "ci_sanity_gate",
    "ci_sync_readiness_report_generate",
    "ci_sync_readiness_report_check",
    "ci_gate_report_index_selftest",
    "ci_gate_report_index_diagnostics_check",
)
REQUIRED_STEPS_SEAMGRIM = (
    "seamgrim_ci_gate_seed_meta_step_check",
    "seamgrim_ci_gate_runtime5_passthrough_check",
    "seamgrim_ci_gate_guideblock_step_check",
    "seamgrim_wasm_cli_diag_parity_check",
)
REQUIRED_STEPS_FULL = REQUIRED_STEPS_COMMON + REQUIRED_STEPS_SEAMGRIM
REQUIRED_STEPS_CORE_LANG = REQUIRED_STEPS_COMMON


def fail(msg: str) -> int:
    print(f"[ci-gate-report-index-selftest] fail: {msg}")
    return 1


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.rstrip() + "\n", encoding="utf-8")


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def run_check(
    index: Path,
    required_steps: tuple[str, ...] = (),
    *,
    sanity_profile: str = "full",
    enforce_profile_step_contract: bool = False,
) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_ci_gate_report_index_check.py",
        "--index",
        str(index),
        "--sanity-profile",
        sanity_profile,
    ]
    if enforce_profile_step_contract:
        cmd.append("--enforce-profile-step-contract")
    for step in required_steps:
        cmd.extend(["--required-step", step])
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def build_index_case(root: Path, case_name: str, sanity_profile: str = "full") -> Path:
    case_dir = root / case_name
    case_dir.mkdir(parents=True, exist_ok=True)

    summary = case_dir / "ci_gate_summary.txt"
    summary_line = case_dir / "ci_gate_summary_line.txt"
    final_status_line = case_dir / "ci_gate_final_status_line.txt"
    final_status_parse = case_dir / "ci_gate_final_status_line_parse.detjson"
    result = case_dir / "ci_gate_result.detjson"
    badge = case_dir / "ci_gate_badge.detjson"
    brief = case_dir / "ci_fail_brief.txt"
    triage = case_dir / "ci_fail_triage.detjson"
    sanity = case_dir / "ci_sanity_gate.detjson"
    sync = case_dir / "ci_sync_readiness.detjson"
    parity = case_dir / "seamgrim_wasm_cli_diag_parity_report.detjson"
    index = case_dir / "ci_gate_report_index.detjson"

    write_text(summary, "[ci-gate-summary] PASS")
    write_text(summary_line, "status=pass reason=ok failed_steps=0")
    write_text(final_status_line, "schema=ddn.ci.final_status.v1 status=pass reason=ok failed_steps=0 aggregate_status=pass overall_ok=1")
    write_json(
        final_status_parse,
        {
            "schema": "ddn.ci.gate_final_status_line_parse.v1",
            "status_line_path": str(final_status_line),
            "parsed": {
                "status": "pass",
                "reason": "ok",
                "failed_steps": "0",
                "aggregate_status": "pass",
                "overall_ok": "1",
            },
        },
    )
    write_json(
        result,
        {
            "schema": "ddn.ci.gate_result.v1",
            "ok": True,
            "status": "pass",
            "reason": "ok",
            "overall_ok": True,
            "aggregate_status": "pass",
            "failed_steps": 0,
            "summary_line_path": str(summary_line),
            "summary_line": summary_line.read_text(encoding="utf-8").strip(),
            "final_status_parse_path": str(final_status_parse),
            "gate_index_path": str(index),
        },
    )
    write_json(
        badge,
        {
            "schema": "ddn.ci.gate_badge.v1",
            "status": "pass",
            "ok": True,
            "label": "ci:pass",
            "result_path": str(result),
        },
    )
    write_text(brief, "status=pass reason=ok failed_steps_count=0")
    write_json(
        triage,
        {
            "schema": "ddn.ci.fail_triage.v1",
            "status": "pass",
            "reason": "ok",
            "summary_report_path_hint_norm": str(summary),
            "artifacts": {
                "summary": {"path": str(summary), "path_norm": str(summary), "exists": True},
                "ci_gate_result_json": {"path": str(result), "path_norm": str(result), "exists": True},
                "ci_gate_badge_json": {"path": str(badge), "path_norm": str(badge), "exists": True},
                "ci_fail_brief_txt": {"path": str(brief), "path_norm": str(brief), "exists": True},
                "ci_fail_triage_json": {"path": str(triage), "path_norm": str(triage), "exists": True},
            },
        },
    )
    write_json(
        sanity,
        {
            "schema": "ddn.ci.sanity_gate.v1",
            "status": "pass",
            "code": "OK",
            "step": "all",
            "profile": sanity_profile,
            "steps": [],
        },
    )
    write_json(
        sync,
        {
            "schema": "ddn.ci.sync_readiness.v1",
            "status": "pass",
            "ok": True,
            "code": "OK",
            "step": "all",
            "sanity_profile": sanity_profile,
            "steps": [],
        },
    )
    write_json(
        parity,
        {"schema": "ddn.seamgrim.wasm_cli_diag_parity.v1", "status": "pass", "ok": True, "code": "OK", "step": "all", "steps": []},
    )

    write_json(
        index,
        {
            "schema": "ddn.ci.aggregate_gate.index.v1",
            "generated_at_utc": "2026-03-04T00:00:00+00:00",
            "report_prefix": "",
            "report_prefix_source": "",
            "report_dir": str(case_dir),
            "ci_sanity_profile": sanity_profile,
            "step_log_dir": "",
            "step_log_failed_only": False,
            "overall_ok": True,
            "reports": {
                "summary": str(summary),
                "summary_line": str(summary_line),
                "final_status_parse_json": str(final_status_parse),
                "ci_gate_result_json": str(result),
                "ci_gate_badge_json": str(badge),
                "ci_fail_brief_txt": str(brief),
                "ci_fail_triage_json": str(triage),
                "ci_sanity_gate": str(sanity),
                "ci_sync_readiness": str(sync),
                "seamgrim_wasm_cli_diag_parity": str(parity),
            },
            "steps": [
                {
                    "name": "ci_profile_split_contract_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_profile_split_contract_check.py"],
                },
                {
                    "name": "ci_profile_matrix_gate_selftest",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_profile_matrix_gate_selftest.py"],
                },
                {"name": "ci_sanity_gate", "returncode": 0, "ok": True, "cmd": ["python", "tests/run_ci_sanity_gate.py"]},
                {
                    "name": "ci_sync_readiness_report_generate",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_sync_readiness_check.py"],
                },
                {
                    "name": "ci_sync_readiness_report_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_sync_readiness_report_check.py"],
                },
                {
                    "name": "seamgrim_wasm_cli_diag_parity_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_seamgrim_wasm_cli_diag_parity_check.py"],
                },
                {
                    "name": "seamgrim_ci_gate_seed_meta_step_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_seamgrim_ci_gate_seed_meta_step_check.py"],
                },
                {
                    "name": "seamgrim_ci_gate_runtime5_passthrough_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_seamgrim_ci_gate_runtime5_passthrough_check.py"],
                },
                {
                    "name": "seamgrim_ci_gate_guideblock_step_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_seamgrim_ci_gate_guideblock_step_check.py"],
                },
                {
                    "name": "ci_gate_report_index_selftest",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_gate_report_index_check_selftest.py"],
                },
                {
                    "name": "ci_gate_report_index_diagnostics_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_gate_report_index_diagnostics_check.py"],
                },
            ],
        },
    )
    return index


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="ci_gate_report_index_selftest_") as td:
        root = Path(td)

        ok_index = build_index_case(root, "ok")
        ok_proc = run_check(
            ok_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if ok_proc.returncode != 0:
            return fail(f"ok case failed: out={ok_proc.stdout} err={ok_proc.stderr}")

        generated_at_missing_index = build_index_case(root, "generated_at_missing")
        generated_at_missing_doc = json.loads(generated_at_missing_index.read_text(encoding="utf-8"))
        generated_at_missing_doc["generated_at_utc"] = ""
        write_json(generated_at_missing_index, generated_at_missing_doc)
        generated_at_missing_proc = run_check(
            generated_at_missing_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if generated_at_missing_proc.returncode == 0:
            return fail("generated_at missing case must fail")
        if f"fail code={CODES['GENERATED_AT_MISSING']}" not in generated_at_missing_proc.stderr:
            return fail(f"generated_at missing code mismatch: err={generated_at_missing_proc.stderr}")

        generated_at_invalid_index = build_index_case(root, "generated_at_invalid")
        generated_at_invalid_doc = json.loads(generated_at_invalid_index.read_text(encoding="utf-8"))
        generated_at_invalid_doc["generated_at_utc"] = "not-a-timestamp"
        write_json(generated_at_invalid_index, generated_at_invalid_doc)
        generated_at_invalid_proc = run_check(
            generated_at_invalid_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if generated_at_invalid_proc.returncode == 0:
            return fail("generated_at invalid case must fail")
        if f"fail code={CODES['GENERATED_AT_INVALID']}" not in generated_at_invalid_proc.stderr:
            return fail(f"generated_at invalid code mismatch: err={generated_at_invalid_proc.stderr}")

        report_dir_missing_index = build_index_case(root, "report_dir_missing")
        report_dir_missing_doc = json.loads(report_dir_missing_index.read_text(encoding="utf-8"))
        report_dir_missing_doc["report_dir"] = ""
        write_json(report_dir_missing_index, report_dir_missing_doc)
        report_dir_missing_proc = run_check(
            report_dir_missing_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if report_dir_missing_proc.returncode == 0:
            return fail("report_dir missing case must fail")
        if f"fail code={CODES['REPORT_DIR_MISSING']}" not in report_dir_missing_proc.stderr:
            return fail(f"report_dir missing code mismatch: err={report_dir_missing_proc.stderr}")

        report_dir_not_found_index = build_index_case(root, "report_dir_not_found")
        report_dir_not_found_doc = json.loads(report_dir_not_found_index.read_text(encoding="utf-8"))
        report_dir_not_found_doc["report_dir"] = str(root / "missing" / "report_dir")
        write_json(report_dir_not_found_index, report_dir_not_found_doc)
        report_dir_not_found_proc = run_check(
            report_dir_not_found_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if report_dir_not_found_proc.returncode == 0:
            return fail("report_dir not found case must fail")
        if f"fail code={CODES['REPORT_DIR_NOT_FOUND']}" not in report_dir_not_found_proc.stderr:
            return fail(f"report_dir not found code mismatch: err={report_dir_not_found_proc.stderr}")

        report_prefix_source_invalid_index = build_index_case(root, "report_prefix_source_invalid")
        report_prefix_source_invalid_doc = json.loads(report_prefix_source_invalid_index.read_text(encoding="utf-8"))
        report_prefix_source_invalid_doc["report_prefix"] = "demo"
        report_prefix_source_invalid_doc["report_prefix_source"] = "manual"
        write_json(report_prefix_source_invalid_index, report_prefix_source_invalid_doc)
        report_prefix_source_invalid_proc = run_check(
            report_prefix_source_invalid_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if report_prefix_source_invalid_proc.returncode == 0:
            return fail("report_prefix_source invalid case must fail")
        if f"fail code={CODES['REPORT_PREFIX_SOURCE_INVALID']}" not in report_prefix_source_invalid_proc.stderr:
            return fail(f"report_prefix_source invalid code mismatch: err={report_prefix_source_invalid_proc.stderr}")

        report_prefix_source_mismatch_index = build_index_case(root, "report_prefix_source_mismatch")
        report_prefix_source_mismatch_doc = json.loads(report_prefix_source_mismatch_index.read_text(encoding="utf-8"))
        report_prefix_source_mismatch_doc["report_prefix"] = "demo"
        report_prefix_source_mismatch_doc["report_prefix_source"] = ""
        write_json(report_prefix_source_mismatch_index, report_prefix_source_mismatch_doc)
        report_prefix_source_mismatch_proc = run_check(
            report_prefix_source_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if report_prefix_source_mismatch_proc.returncode == 0:
            return fail("report_prefix_source mismatch case must fail")
        if (
            f"fail code={CODES['REPORT_PREFIX_SOURCE_MISMATCH']}"
            not in report_prefix_source_mismatch_proc.stderr
        ):
            return fail(f"report_prefix_source mismatch code mismatch: err={report_prefix_source_mismatch_proc.stderr}")

        report_prefix_source_env_empty_index = build_index_case(root, "report_prefix_source_env_empty")
        report_prefix_source_env_empty_doc = json.loads(report_prefix_source_env_empty_index.read_text(encoding="utf-8"))
        report_prefix_source_env_empty_doc["report_prefix"] = "demo"
        report_prefix_source_env_empty_doc["report_prefix_source"] = "env:"
        write_json(report_prefix_source_env_empty_index, report_prefix_source_env_empty_doc)
        report_prefix_source_env_empty_proc = run_check(
            report_prefix_source_env_empty_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if report_prefix_source_env_empty_proc.returncode == 0:
            return fail("report_prefix_source env-empty case must fail")
        if f"fail code={CODES['REPORT_PREFIX_SOURCE_INVALID']}" not in report_prefix_source_env_empty_proc.stderr:
            return fail(
                f"report_prefix_source env-empty code mismatch: err={report_prefix_source_env_empty_proc.stderr}"
            )

        report_prefix_source_env_ok_index = build_index_case(root, "report_prefix_source_env_ok")
        report_prefix_source_env_ok_doc = json.loads(report_prefix_source_env_ok_index.read_text(encoding="utf-8"))
        report_prefix_source_env_ok_doc["report_prefix"] = "demo"
        report_prefix_source_env_ok_doc["report_prefix_source"] = "env:CI_REPORT_PREFIX"
        write_json(report_prefix_source_env_ok_index, report_prefix_source_env_ok_doc)
        report_prefix_source_env_ok_proc = run_check(
            report_prefix_source_env_ok_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if report_prefix_source_env_ok_proc.returncode != 0:
            return fail(
                "report_prefix_source env-ok case should pass: "
                f"out={report_prefix_source_env_ok_proc.stdout} err={report_prefix_source_env_ok_proc.stderr}"
            )

        step_log_dir_type_index = build_index_case(root, "step_log_dir_type")
        step_log_dir_type_doc = json.loads(step_log_dir_type_index.read_text(encoding="utf-8"))
        step_log_dir_type_doc["step_log_dir"] = {"bad": "type"}
        write_json(step_log_dir_type_index, step_log_dir_type_doc)
        step_log_dir_type_proc = run_check(
            step_log_dir_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if step_log_dir_type_proc.returncode == 0:
            return fail("step_log_dir type case must fail")
        if f"fail code={CODES['STEP_LOG_DIR_TYPE']}" not in step_log_dir_type_proc.stderr:
            return fail(f"step_log_dir type code mismatch: err={step_log_dir_type_proc.stderr}")

        step_log_failed_only_type_index = build_index_case(root, "step_log_failed_only_type")
        step_log_failed_only_type_doc = json.loads(step_log_failed_only_type_index.read_text(encoding="utf-8"))
        step_log_failed_only_type_doc["step_log_failed_only"] = "1"
        write_json(step_log_failed_only_type_index, step_log_failed_only_type_doc)
        step_log_failed_only_type_proc = run_check(
            step_log_failed_only_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if step_log_failed_only_type_proc.returncode == 0:
            return fail("step_log_failed_only type case must fail")
        if f"fail code={CODES['STEP_LOG_FAILED_ONLY_TYPE']}" not in step_log_failed_only_type_proc.stderr:
            return fail(
                f"step_log_failed_only type code mismatch: err={step_log_failed_only_type_proc.stderr}"
            )

        step_log_config_mismatch_index = build_index_case(root, "step_log_config_mismatch")
        step_log_config_mismatch_doc = json.loads(step_log_config_mismatch_index.read_text(encoding="utf-8"))
        step_log_config_mismatch_doc["step_log_failed_only"] = True
        step_log_config_mismatch_doc["step_log_dir"] = ""
        write_json(step_log_config_mismatch_index, step_log_config_mismatch_doc)
        step_log_config_mismatch_proc = run_check(
            step_log_config_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if step_log_config_mismatch_proc.returncode == 0:
            return fail("step_log config mismatch case must fail")
        if f"fail code={CODES['STEP_LOG_CONFIG_MISMATCH']}" not in step_log_config_mismatch_proc.stderr:
            return fail(
                f"step_log config mismatch code mismatch: err={step_log_config_mismatch_proc.stderr}"
            )

        step_log_dir_not_found_index = build_index_case(root, "step_log_dir_not_found")
        step_log_dir_not_found_doc = json.loads(step_log_dir_not_found_index.read_text(encoding="utf-8"))
        step_log_dir_not_found_doc["step_log_dir"] = str(root / "missing" / "step_logs")
        write_json(step_log_dir_not_found_index, step_log_dir_not_found_doc)
        step_log_dir_not_found_proc = run_check(
            step_log_dir_not_found_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if step_log_dir_not_found_proc.returncode == 0:
            return fail("step_log_dir not found case must fail")
        if f"fail code={CODES['STEP_LOG_DIR_NOT_FOUND']}" not in step_log_dir_not_found_proc.stderr:
            return fail(
                f"step_log_dir not found code mismatch: err={step_log_dir_not_found_proc.stderr}"
            )

        missing_key_index = build_index_case(root, "missing_key")
        missing_key_doc = json.loads(missing_key_index.read_text(encoding="utf-8"))
        missing_key_doc["reports"].pop("seamgrim_wasm_cli_diag_parity", None)
        write_json(missing_key_index, missing_key_doc)
        missing_key_proc = run_check(
            missing_key_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if missing_key_proc.returncode == 0:
            return fail("missing key case must fail")
        if f"fail code={CODES['REPORT_KEY_MISSING']}" not in missing_key_proc.stderr:
            return fail(f"missing key code mismatch: err={missing_key_proc.stderr}")

        missing_path_index = build_index_case(root, "missing_path")
        missing_path_doc = json.loads(missing_path_index.read_text(encoding="utf-8"))
        missing_path_doc["reports"]["seamgrim_wasm_cli_diag_parity"] = str(root / "missing" / "parity.detjson")
        write_json(missing_path_index, missing_path_doc)
        missing_path_proc = run_check(
            missing_path_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if missing_path_proc.returncode == 0:
            return fail("missing path case must fail")
        if f"fail code={CODES['REPORT_PATH_MISSING']}" not in missing_path_proc.stderr:
            return fail(f"missing path code mismatch: err={missing_path_proc.stderr}")

        bad_schema_index = build_index_case(root, "bad_schema")
        bad_schema_doc = json.loads(bad_schema_index.read_text(encoding="utf-8"))
        parity_path = Path(str(bad_schema_doc["reports"]["seamgrim_wasm_cli_diag_parity"]))
        write_json(parity_path, {"schema": "wrong.schema"})
        bad_schema_proc = run_check(
            bad_schema_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if bad_schema_proc.returncode == 0:
            return fail("bad schema case must fail")
        if f"fail code={CODES['ARTIFACT_SCHEMA_MISMATCH']}" not in bad_schema_proc.stderr:
            return fail(f"bad schema code mismatch: err={bad_schema_proc.stderr}")

        final_parse_parsed_missing_index = build_index_case(root, "final_parse_parsed_missing")
        final_parse_parsed_missing_doc = json.loads(final_parse_parsed_missing_index.read_text(encoding="utf-8"))
        final_parse_parsed_missing_report = Path(str(final_parse_parsed_missing_doc["reports"]["final_status_parse_json"]))
        final_parse_parsed_missing_payload = json.loads(final_parse_parsed_missing_report.read_text(encoding="utf-8"))
        final_parse_parsed_missing_payload["parsed"] = []
        write_json(final_parse_parsed_missing_report, final_parse_parsed_missing_payload)
        final_parse_parsed_missing_proc = run_check(
            final_parse_parsed_missing_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_parsed_missing_proc.returncode == 0:
            return fail("final parse parsed missing case must fail")
        if f"fail code={CODES['FINAL_PARSE_PARSED_MISSING']}" not in final_parse_parsed_missing_proc.stderr:
            return fail(f"final parse parsed missing code mismatch: err={final_parse_parsed_missing_proc.stderr}")

        final_parse_status_line_path_missing_index = build_index_case(root, "final_parse_status_line_path_missing")
        final_parse_status_line_path_missing_doc = json.loads(
            final_parse_status_line_path_missing_index.read_text(encoding="utf-8")
        )
        final_parse_status_line_path_missing_report = Path(
            str(final_parse_status_line_path_missing_doc["reports"]["final_status_parse_json"])
        )
        final_parse_status_line_path_missing_payload = json.loads(
            final_parse_status_line_path_missing_report.read_text(encoding="utf-8")
        )
        final_parse_status_line_path_missing_payload["status_line_path"] = ""
        write_json(final_parse_status_line_path_missing_report, final_parse_status_line_path_missing_payload)
        final_parse_status_line_path_missing_proc = run_check(
            final_parse_status_line_path_missing_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_status_line_path_missing_proc.returncode == 0:
            return fail("final parse status_line_path missing case must fail")
        if (
            f"fail code={CODES['FINAL_PARSE_STATUS_LINE_PATH_MISSING']}"
            not in final_parse_status_line_path_missing_proc.stderr
        ):
            return fail(
                "final parse status_line_path missing code mismatch: "
                f"err={final_parse_status_line_path_missing_proc.stderr}"
            )

        final_parse_status_line_path_not_found_index = build_index_case(root, "final_parse_status_line_path_not_found")
        final_parse_status_line_path_not_found_doc = json.loads(
            final_parse_status_line_path_not_found_index.read_text(encoding="utf-8")
        )
        final_parse_status_line_path_not_found_report = Path(
            str(final_parse_status_line_path_not_found_doc["reports"]["final_status_parse_json"])
        )
        final_parse_status_line_path_not_found_payload = json.loads(
            final_parse_status_line_path_not_found_report.read_text(encoding="utf-8")
        )
        final_parse_status_line_path_not_found_payload["status_line_path"] = str(root / "missing" / "ci_gate_final_status_line.txt")
        write_json(final_parse_status_line_path_not_found_report, final_parse_status_line_path_not_found_payload)
        final_parse_status_line_path_not_found_proc = run_check(
            final_parse_status_line_path_not_found_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_status_line_path_not_found_proc.returncode == 0:
            return fail("final parse status_line_path not found case must fail")
        if (
            f"fail code={CODES['FINAL_PARSE_STATUS_LINE_PATH_NOT_FOUND']}"
            not in final_parse_status_line_path_not_found_proc.stderr
        ):
            return fail(
                "final parse status_line_path not found code mismatch: "
                f"err={final_parse_status_line_path_not_found_proc.stderr}"
            )

        final_parse_status_mismatch_index = build_index_case(root, "final_parse_status_mismatch")
        final_parse_status_mismatch_doc = json.loads(final_parse_status_mismatch_index.read_text(encoding="utf-8"))
        final_parse_status_mismatch_report = Path(str(final_parse_status_mismatch_doc["reports"]["final_status_parse_json"]))
        final_parse_status_mismatch_payload = json.loads(final_parse_status_mismatch_report.read_text(encoding="utf-8"))
        final_parse_status_mismatch_payload["parsed"]["status"] = "fail"
        write_json(final_parse_status_mismatch_report, final_parse_status_mismatch_payload)
        final_parse_status_mismatch_proc = run_check(
            final_parse_status_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_status_mismatch_proc.returncode == 0:
            return fail("final parse status mismatch case must fail")
        if f"fail code={CODES['FINAL_PARSE_STATUS_MISMATCH']}" not in final_parse_status_mismatch_proc.stderr:
            return fail(f"final parse status mismatch code mismatch: err={final_parse_status_mismatch_proc.stderr}")

        final_parse_overall_ok_invalid_index = build_index_case(root, "final_parse_overall_ok_invalid")
        final_parse_overall_ok_invalid_doc = json.loads(final_parse_overall_ok_invalid_index.read_text(encoding="utf-8"))
        final_parse_overall_ok_invalid_report = Path(
            str(final_parse_overall_ok_invalid_doc["reports"]["final_status_parse_json"])
        )
        final_parse_overall_ok_invalid_payload = json.loads(final_parse_overall_ok_invalid_report.read_text(encoding="utf-8"))
        final_parse_overall_ok_invalid_payload["parsed"]["overall_ok"] = "true"
        write_json(final_parse_overall_ok_invalid_report, final_parse_overall_ok_invalid_payload)
        final_parse_overall_ok_invalid_proc = run_check(
            final_parse_overall_ok_invalid_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_overall_ok_invalid_proc.returncode == 0:
            return fail("final parse overall_ok invalid case must fail")
        if f"fail code={CODES['FINAL_PARSE_OVERALL_OK_INVALID']}" not in final_parse_overall_ok_invalid_proc.stderr:
            return fail(f"final parse overall_ok invalid code mismatch: err={final_parse_overall_ok_invalid_proc.stderr}")

        final_parse_overall_ok_mismatch_index = build_index_case(root, "final_parse_overall_ok_mismatch")
        final_parse_overall_ok_mismatch_doc = json.loads(final_parse_overall_ok_mismatch_index.read_text(encoding="utf-8"))
        final_parse_overall_ok_mismatch_report = Path(
            str(final_parse_overall_ok_mismatch_doc["reports"]["final_status_parse_json"])
        )
        final_parse_overall_ok_mismatch_payload = json.loads(
            final_parse_overall_ok_mismatch_report.read_text(encoding="utf-8")
        )
        final_parse_overall_ok_mismatch_payload["parsed"]["overall_ok"] = "0"
        write_json(final_parse_overall_ok_mismatch_report, final_parse_overall_ok_mismatch_payload)
        final_parse_overall_ok_mismatch_proc = run_check(
            final_parse_overall_ok_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_overall_ok_mismatch_proc.returncode == 0:
            return fail("final parse overall_ok mismatch case must fail")
        if (
            f"fail code={CODES['FINAL_PARSE_OVERALL_OK_MISMATCH']}"
            not in final_parse_overall_ok_mismatch_proc.stderr
        ):
            return fail(
                f"final parse overall_ok mismatch code mismatch: err={final_parse_overall_ok_mismatch_proc.stderr}"
            )

        final_parse_aggregate_status_invalid_index = build_index_case(root, "final_parse_aggregate_status_invalid")
        final_parse_aggregate_status_invalid_doc = json.loads(
            final_parse_aggregate_status_invalid_index.read_text(encoding="utf-8")
        )
        final_parse_aggregate_status_invalid_report = Path(
            str(final_parse_aggregate_status_invalid_doc["reports"]["final_status_parse_json"])
        )
        final_parse_aggregate_status_invalid_payload = json.loads(
            final_parse_aggregate_status_invalid_report.read_text(encoding="utf-8")
        )
        final_parse_aggregate_status_invalid_payload["parsed"]["aggregate_status"] = "unknown"
        write_json(final_parse_aggregate_status_invalid_report, final_parse_aggregate_status_invalid_payload)
        final_parse_aggregate_status_invalid_proc = run_check(
            final_parse_aggregate_status_invalid_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_aggregate_status_invalid_proc.returncode == 0:
            return fail("final parse aggregate_status invalid case must fail")
        if (
            f"fail code={CODES['FINAL_PARSE_AGGREGATE_STATUS_INVALID']}"
            not in final_parse_aggregate_status_invalid_proc.stderr
        ):
            return fail(
                "final parse aggregate_status invalid code mismatch: "
                f"err={final_parse_aggregate_status_invalid_proc.stderr}"
            )

        final_parse_failed_steps_type_index = build_index_case(root, "final_parse_failed_steps_type")
        final_parse_failed_steps_type_doc = json.loads(final_parse_failed_steps_type_index.read_text(encoding="utf-8"))
        final_parse_failed_steps_type_report = Path(
            str(final_parse_failed_steps_type_doc["reports"]["final_status_parse_json"])
        )
        final_parse_failed_steps_type_payload = json.loads(final_parse_failed_steps_type_report.read_text(encoding="utf-8"))
        final_parse_failed_steps_type_payload["parsed"]["failed_steps"] = "x"
        write_json(final_parse_failed_steps_type_report, final_parse_failed_steps_type_payload)
        final_parse_failed_steps_type_proc = run_check(
            final_parse_failed_steps_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_failed_steps_type_proc.returncode == 0:
            return fail("final parse failed_steps type case must fail")
        if f"fail code={CODES['FINAL_PARSE_FAILED_STEPS_TYPE']}" not in final_parse_failed_steps_type_proc.stderr:
            return fail(f"final parse failed_steps type code mismatch: err={final_parse_failed_steps_type_proc.stderr}")

        final_parse_failed_steps_mismatch_index = build_index_case(root, "final_parse_failed_steps_mismatch")
        final_parse_failed_steps_mismatch_doc = json.loads(
            final_parse_failed_steps_mismatch_index.read_text(encoding="utf-8")
        )
        final_parse_failed_steps_mismatch_report = Path(
            str(final_parse_failed_steps_mismatch_doc["reports"]["final_status_parse_json"])
        )
        final_parse_failed_steps_mismatch_payload = json.loads(
            final_parse_failed_steps_mismatch_report.read_text(encoding="utf-8")
        )
        final_parse_failed_steps_mismatch_payload["parsed"]["failed_steps"] = "1"
        write_json(final_parse_failed_steps_mismatch_report, final_parse_failed_steps_mismatch_payload)
        final_parse_failed_steps_mismatch_proc = run_check(
            final_parse_failed_steps_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_failed_steps_mismatch_proc.returncode == 0:
            return fail("final parse failed_steps mismatch case must fail")
        if (
            f"fail code={CODES['FINAL_PARSE_FAILED_STEPS_MISMATCH']}"
            not in final_parse_failed_steps_mismatch_proc.stderr
        ):
            return fail(
                f"final parse failed_steps mismatch code mismatch: err={final_parse_failed_steps_mismatch_proc.stderr}"
            )

        missing_required_step_index = build_index_case(root, "missing_required_step")
        missing_required_step_doc = json.loads(missing_required_step_index.read_text(encoding="utf-8"))
        missing_required_step_doc["steps"] = [
            row for row in missing_required_step_doc["steps"] if row.get("name") != "ci_sync_readiness_report_check"
        ]
        write_json(missing_required_step_index, missing_required_step_doc)
        missing_required_step_proc = run_check(
            missing_required_step_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if missing_required_step_proc.returncode == 0:
            return fail("missing required step case must fail")
        if f"fail code={CODES['REQUIRED_STEP_MISSING']}" not in missing_required_step_proc.stderr:
            return fail(f"missing required step code mismatch: err={missing_required_step_proc.stderr}")

        bad_step_shape_index = build_index_case(root, "bad_step_shape")
        bad_step_shape_doc = json.loads(bad_step_shape_index.read_text(encoding="utf-8"))
        bad_step_shape_doc["steps"][0] = "oops"
        write_json(bad_step_shape_index, bad_step_shape_doc)
        bad_step_shape_proc = run_check(
            bad_step_shape_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if bad_step_shape_proc.returncode == 0:
            return fail("bad step shape case must fail")
        if f"fail code={CODES['STEP_ROW_TYPE']}" not in bad_step_shape_proc.stderr:
            return fail(f"bad step shape code mismatch: err={bad_step_shape_proc.stderr}")

        bad_profile_index = build_index_case(root, "bad_profile")
        bad_profile_doc = json.loads(bad_profile_index.read_text(encoding="utf-8"))
        bad_profile_doc["ci_sanity_profile"] = "unknown_profile"
        write_json(bad_profile_index, bad_profile_doc)
        bad_profile_proc = run_check(
            bad_profile_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if bad_profile_proc.returncode == 0:
            return fail("bad profile case must fail")
        if f"fail code={CODES['PROFILE_INVALID']}" not in bad_profile_proc.stderr:
            return fail(f"bad profile code mismatch: err={bad_profile_proc.stderr}")

        profile_mismatch_index = build_index_case(root, "profile_mismatch", sanity_profile="seamgrim")
        profile_mismatch_proc = run_check(
            profile_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if profile_mismatch_proc.returncode == 0:
            return fail("profile mismatch case must fail")
        if f"fail code={CODES['PROFILE_MISMATCH']}" not in profile_mismatch_proc.stderr:
            return fail(f"profile mismatch code mismatch: err={profile_mismatch_proc.stderr}")

        index_overall_ok_type_index = build_index_case(root, "index_overall_ok_type")
        index_overall_ok_type_doc = json.loads(index_overall_ok_type_index.read_text(encoding="utf-8"))
        index_overall_ok_type_doc["overall_ok"] = "true"
        write_json(index_overall_ok_type_index, index_overall_ok_type_doc)
        index_overall_ok_type_proc = run_check(
            index_overall_ok_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if index_overall_ok_type_proc.returncode == 0:
            return fail("index overall_ok type case must fail")
        if f"fail code={CODES['INDEX_OVERALL_OK_TYPE']}" not in index_overall_ok_type_proc.stderr:
            return fail(f"index overall_ok type code mismatch: err={index_overall_ok_type_proc.stderr}")

        index_overall_ok_steps_mismatch_index = build_index_case(root, "index_overall_ok_steps_mismatch")
        index_overall_ok_steps_mismatch_doc = json.loads(index_overall_ok_steps_mismatch_index.read_text(encoding="utf-8"))
        index_overall_ok_steps_mismatch_doc["overall_ok"] = False
        write_json(index_overall_ok_steps_mismatch_index, index_overall_ok_steps_mismatch_doc)
        index_overall_ok_steps_mismatch_proc = run_check(
            index_overall_ok_steps_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if index_overall_ok_steps_mismatch_proc.returncode == 0:
            return fail("index overall_ok steps mismatch case must fail")
        if f"fail code={CODES['INDEX_OVERALL_OK_STEPS_MISMATCH']}" not in index_overall_ok_steps_mismatch_proc.stderr:
            return fail(f"index overall_ok steps mismatch code mismatch: err={index_overall_ok_steps_mismatch_proc.stderr}")

        sanity_profile_mismatch_index = build_index_case(root, "sanity_profile_mismatch")
        sanity_profile_mismatch_doc = json.loads(sanity_profile_mismatch_index.read_text(encoding="utf-8"))
        sanity_report = Path(str(sanity_profile_mismatch_doc["reports"]["ci_sanity_gate"]))
        sanity_report_doc = json.loads(sanity_report.read_text(encoding="utf-8"))
        sanity_report_doc["profile"] = "seamgrim"
        write_json(sanity_report, sanity_report_doc)
        sanity_profile_mismatch_proc = run_check(
            sanity_profile_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if sanity_profile_mismatch_proc.returncode == 0:
            return fail("sanity profile mismatch case must fail")
        if f"fail code={CODES['SANITY_PROFILE_MISMATCH']}" not in sanity_profile_mismatch_proc.stderr:
            return fail(f"sanity profile mismatch code mismatch: err={sanity_profile_mismatch_proc.stderr}")

        sync_profile_mismatch_index = build_index_case(root, "sync_profile_mismatch")
        sync_profile_mismatch_doc = json.loads(sync_profile_mismatch_index.read_text(encoding="utf-8"))
        sync_report = Path(str(sync_profile_mismatch_doc["reports"]["ci_sync_readiness"]))
        sync_report_doc = json.loads(sync_report.read_text(encoding="utf-8"))
        sync_report_doc["sanity_profile"] = "seamgrim"
        write_json(sync_report, sync_report_doc)
        sync_profile_mismatch_proc = run_check(
            sync_profile_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if sync_profile_mismatch_proc.returncode == 0:
            return fail("sync profile mismatch case must fail")
        if f"fail code={CODES['SYNC_PROFILE_MISMATCH']}" not in sync_profile_mismatch_proc.stderr:
            return fail(f"sync profile mismatch code mismatch: err={sync_profile_mismatch_proc.stderr}")

        result_overall_ok_type_index = build_index_case(root, "result_overall_ok_type")
        result_overall_ok_type_doc = json.loads(result_overall_ok_type_index.read_text(encoding="utf-8"))
        result_overall_ok_type_report = Path(str(result_overall_ok_type_doc["reports"]["ci_gate_result_json"]))
        result_overall_ok_type_result = json.loads(result_overall_ok_type_report.read_text(encoding="utf-8"))
        result_overall_ok_type_result["overall_ok"] = "true"
        write_json(result_overall_ok_type_report, result_overall_ok_type_result)
        result_overall_ok_type_proc = run_check(
            result_overall_ok_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_overall_ok_type_proc.returncode == 0:
            return fail("result overall_ok type case must fail")
        if f"fail code={CODES['RESULT_OVERALL_OK_TYPE']}" not in result_overall_ok_type_proc.stderr:
            return fail(f"result overall_ok type code mismatch: err={result_overall_ok_type_proc.stderr}")

        result_overall_ok_mismatch_index = build_index_case(root, "result_overall_ok_mismatch")
        result_overall_ok_mismatch_doc = json.loads(result_overall_ok_mismatch_index.read_text(encoding="utf-8"))
        result_overall_ok_mismatch_report = Path(str(result_overall_ok_mismatch_doc["reports"]["ci_gate_result_json"]))
        result_overall_ok_mismatch_result = json.loads(result_overall_ok_mismatch_report.read_text(encoding="utf-8"))
        result_overall_ok_mismatch_result["overall_ok"] = False
        write_json(result_overall_ok_mismatch_report, result_overall_ok_mismatch_result)
        result_overall_ok_mismatch_proc = run_check(
            result_overall_ok_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_overall_ok_mismatch_proc.returncode == 0:
            return fail("result overall_ok mismatch case must fail")
        if f"fail code={CODES['RESULT_OVERALL_OK_MISMATCH']}" not in result_overall_ok_mismatch_proc.stderr:
            return fail(f"result overall_ok mismatch code mismatch: err={result_overall_ok_mismatch_proc.stderr}")

        result_ok_type_index = build_index_case(root, "result_ok_type")
        result_ok_type_doc = json.loads(result_ok_type_index.read_text(encoding="utf-8"))
        result_ok_type_report = Path(str(result_ok_type_doc["reports"]["ci_gate_result_json"]))
        result_ok_type_result = json.loads(result_ok_type_report.read_text(encoding="utf-8"))
        result_ok_type_result["ok"] = "1"
        write_json(result_ok_type_report, result_ok_type_result)
        result_ok_type_proc = run_check(
            result_ok_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_ok_type_proc.returncode == 0:
            return fail("result ok type case must fail")
        if f"fail code={CODES['RESULT_OK_TYPE']}" not in result_ok_type_proc.stderr:
            return fail(f"result ok type code mismatch: err={result_ok_type_proc.stderr}")

        result_failed_steps_type_index = build_index_case(root, "result_failed_steps_type")
        result_failed_steps_type_doc = json.loads(result_failed_steps_type_index.read_text(encoding="utf-8"))
        result_failed_steps_type_report = Path(str(result_failed_steps_type_doc["reports"]["ci_gate_result_json"]))
        result_failed_steps_type_result = json.loads(result_failed_steps_type_report.read_text(encoding="utf-8"))
        result_failed_steps_type_result["failed_steps"] = "0"
        write_json(result_failed_steps_type_report, result_failed_steps_type_result)
        result_failed_steps_type_proc = run_check(
            result_failed_steps_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_failed_steps_type_proc.returncode == 0:
            return fail("result failed_steps type case must fail")
        if f"fail code={CODES['RESULT_FAILED_STEPS_TYPE']}" not in result_failed_steps_type_proc.stderr:
            return fail(f"result failed_steps type code mismatch: err={result_failed_steps_type_proc.stderr}")

        result_failed_steps_mismatch_index = build_index_case(root, "result_failed_steps_mismatch")
        result_failed_steps_mismatch_doc = json.loads(result_failed_steps_mismatch_index.read_text(encoding="utf-8"))
        result_failed_steps_mismatch_report = Path(str(result_failed_steps_mismatch_doc["reports"]["ci_gate_result_json"]))
        result_failed_steps_mismatch_result = json.loads(result_failed_steps_mismatch_report.read_text(encoding="utf-8"))
        result_failed_steps_mismatch_result["failed_steps"] = 1
        write_json(result_failed_steps_mismatch_report, result_failed_steps_mismatch_result)
        result_failed_steps_mismatch_proc = run_check(
            result_failed_steps_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_failed_steps_mismatch_proc.returncode == 0:
            return fail("result failed_steps mismatch case must fail")
        if f"fail code={CODES['RESULT_FAILED_STEPS_MISMATCH']}" not in result_failed_steps_mismatch_proc.stderr:
            return fail(f"result failed_steps mismatch code mismatch: err={result_failed_steps_mismatch_proc.stderr}")

        result_status_mismatch_index = build_index_case(root, "result_status_mismatch")
        result_status_mismatch_doc = json.loads(result_status_mismatch_index.read_text(encoding="utf-8"))
        result_status_mismatch_report = Path(str(result_status_mismatch_doc["reports"]["ci_gate_result_json"]))
        result_status_mismatch_result = json.loads(result_status_mismatch_report.read_text(encoding="utf-8"))
        result_status_mismatch_result["status"] = "fail"
        write_json(result_status_mismatch_report, result_status_mismatch_result)
        result_status_mismatch_proc = run_check(
            result_status_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_status_mismatch_proc.returncode == 0:
            return fail("result status mismatch case must fail")
        if f"fail code={CODES['RESULT_STATUS_MISMATCH']}" not in result_status_mismatch_proc.stderr:
            return fail(f"result status mismatch code mismatch: err={result_status_mismatch_proc.stderr}")

        result_aggregate_status_invalid_index = build_index_case(root, "result_aggregate_status_invalid")
        result_aggregate_status_invalid_doc = json.loads(result_aggregate_status_invalid_index.read_text(encoding="utf-8"))
        result_aggregate_status_invalid_report = Path(
            str(result_aggregate_status_invalid_doc["reports"]["ci_gate_result_json"])
        )
        result_aggregate_status_invalid_result = json.loads(result_aggregate_status_invalid_report.read_text(encoding="utf-8"))
        result_aggregate_status_invalid_result["aggregate_status"] = "unknown"
        write_json(result_aggregate_status_invalid_report, result_aggregate_status_invalid_result)
        result_aggregate_status_invalid_proc = run_check(
            result_aggregate_status_invalid_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_aggregate_status_invalid_proc.returncode == 0:
            return fail("result aggregate_status invalid case must fail")
        if (
            f"fail code={CODES['RESULT_AGGREGATE_STATUS_INVALID']}"
            not in result_aggregate_status_invalid_proc.stderr
        ):
            return fail(
                f"result aggregate_status invalid code mismatch: err={result_aggregate_status_invalid_proc.stderr}"
            )

        result_aggregate_status_mismatch_index = build_index_case(root, "result_aggregate_status_mismatch")
        result_aggregate_status_mismatch_doc = json.loads(
            result_aggregate_status_mismatch_index.read_text(encoding="utf-8")
        )
        result_aggregate_status_mismatch_report = Path(
            str(result_aggregate_status_mismatch_doc["reports"]["ci_gate_result_json"])
        )
        result_aggregate_status_mismatch_result = json.loads(
            result_aggregate_status_mismatch_report.read_text(encoding="utf-8")
        )
        result_aggregate_status_mismatch_result["aggregate_status"] = "fail"
        write_json(result_aggregate_status_mismatch_report, result_aggregate_status_mismatch_result)
        result_aggregate_status_mismatch_proc = run_check(
            result_aggregate_status_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_aggregate_status_mismatch_proc.returncode == 0:
            return fail("result aggregate_status mismatch case must fail")
        if (
            f"fail code={CODES['RESULT_AGGREGATE_STATUS_MISMATCH']}"
            not in result_aggregate_status_mismatch_proc.stderr
        ):
            return fail(
                f"result aggregate_status mismatch code mismatch: err={result_aggregate_status_mismatch_proc.stderr}"
            )

        result_ok_mismatch_index = build_index_case(root, "result_ok_mismatch")
        result_ok_mismatch_doc = json.loads(result_ok_mismatch_index.read_text(encoding="utf-8"))
        result_ok_mismatch_report = Path(str(result_ok_mismatch_doc["reports"]["ci_gate_result_json"]))
        result_ok_mismatch_result = json.loads(result_ok_mismatch_report.read_text(encoding="utf-8"))
        result_ok_mismatch_result["ok"] = False
        write_json(result_ok_mismatch_report, result_ok_mismatch_result)
        result_ok_mismatch_proc = run_check(
            result_ok_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_ok_mismatch_proc.returncode == 0:
            return fail("result ok mismatch case must fail")
        if f"fail code={CODES['RESULT_OK_MISMATCH']}" not in result_ok_mismatch_proc.stderr:
            return fail(f"result ok mismatch code mismatch: err={result_ok_mismatch_proc.stderr}")

        result_summary_line_path_mismatch_index = build_index_case(root, "result_summary_line_path_mismatch")
        result_summary_line_path_mismatch_doc = json.loads(result_summary_line_path_mismatch_index.read_text(encoding="utf-8"))
        result_summary_line_path_mismatch_report = Path(
            str(result_summary_line_path_mismatch_doc["reports"]["ci_gate_result_json"])
        )
        result_summary_line_path_mismatch_result = json.loads(
            result_summary_line_path_mismatch_report.read_text(encoding="utf-8")
        )
        result_summary_line_path_mismatch_result["summary_line_path"] = str(root / "mismatch" / "summary_line.txt")
        write_json(result_summary_line_path_mismatch_report, result_summary_line_path_mismatch_result)
        result_summary_line_path_mismatch_proc = run_check(
            result_summary_line_path_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_summary_line_path_mismatch_proc.returncode == 0:
            return fail("result summary_line_path mismatch case must fail")
        if f"fail code={CODES['RESULT_SUMMARY_LINE_PATH_MISMATCH']}" not in result_summary_line_path_mismatch_proc.stderr:
            return fail(
                f"result summary_line_path mismatch code mismatch: err={result_summary_line_path_mismatch_proc.stderr}"
            )

        result_summary_line_mismatch_index = build_index_case(root, "result_summary_line_mismatch")
        result_summary_line_mismatch_doc = json.loads(result_summary_line_mismatch_index.read_text(encoding="utf-8"))
        result_summary_line_mismatch_report = Path(str(result_summary_line_mismatch_doc["reports"]["ci_gate_result_json"]))
        result_summary_line_mismatch_result = json.loads(result_summary_line_mismatch_report.read_text(encoding="utf-8"))
        result_summary_line_mismatch_result["summary_line"] = "status=fail reason=mismatch failed_steps=1"
        write_json(result_summary_line_mismatch_report, result_summary_line_mismatch_result)
        result_summary_line_mismatch_proc = run_check(
            result_summary_line_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_summary_line_mismatch_proc.returncode == 0:
            return fail("result summary_line mismatch case must fail")
        if f"fail code={CODES['RESULT_SUMMARY_LINE_MISMATCH']}" not in result_summary_line_mismatch_proc.stderr:
            return fail(f"result summary_line mismatch code mismatch: err={result_summary_line_mismatch_proc.stderr}")

        result_gate_index_path_mismatch_index = build_index_case(root, "result_gate_index_path_mismatch")
        result_gate_index_path_mismatch_doc = json.loads(result_gate_index_path_mismatch_index.read_text(encoding="utf-8"))
        result_gate_index_path_mismatch_report = Path(
            str(result_gate_index_path_mismatch_doc["reports"]["ci_gate_result_json"])
        )
        result_gate_index_path_mismatch_result = json.loads(result_gate_index_path_mismatch_report.read_text(encoding="utf-8"))
        result_gate_index_path_mismatch_result["gate_index_path"] = str(root / "mismatch" / "ci_gate_report_index.detjson")
        write_json(result_gate_index_path_mismatch_report, result_gate_index_path_mismatch_result)
        result_gate_index_path_mismatch_proc = run_check(
            result_gate_index_path_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_gate_index_path_mismatch_proc.returncode == 0:
            return fail("result gate_index_path mismatch case must fail")
        if f"fail code={CODES['RESULT_GATE_INDEX_PATH_MISMATCH']}" not in result_gate_index_path_mismatch_proc.stderr:
            return fail(
                f"result gate_index_path mismatch code mismatch: err={result_gate_index_path_mismatch_proc.stderr}"
            )

        result_final_status_parse_path_mismatch_index = build_index_case(root, "result_final_status_parse_path_mismatch")
        result_final_status_parse_path_mismatch_doc = json.loads(
            result_final_status_parse_path_mismatch_index.read_text(encoding="utf-8")
        )
        result_final_status_parse_path_mismatch_report = Path(
            str(result_final_status_parse_path_mismatch_doc["reports"]["ci_gate_result_json"])
        )
        result_final_status_parse_path_mismatch_result = json.loads(
            result_final_status_parse_path_mismatch_report.read_text(encoding="utf-8")
        )
        result_final_status_parse_path_mismatch_result["final_status_parse_path"] = str(
            root / "mismatch" / "ci_gate_final_status_line_parse.detjson"
        )
        write_json(result_final_status_parse_path_mismatch_report, result_final_status_parse_path_mismatch_result)
        result_final_status_parse_path_mismatch_proc = run_check(
            result_final_status_parse_path_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_final_status_parse_path_mismatch_proc.returncode == 0:
            return fail("result final_status_parse_path mismatch case must fail")
        if (
            f"fail code={CODES['RESULT_FINAL_STATUS_PARSE_PATH_MISMATCH']}"
            not in result_final_status_parse_path_mismatch_proc.stderr
        ):
            return fail(
                "result final_status_parse_path mismatch code mismatch: "
                f"err={result_final_status_parse_path_mismatch_proc.stderr}"
            )

        badge_status_mismatch_index = build_index_case(root, "badge_status_mismatch")
        badge_status_mismatch_doc = json.loads(badge_status_mismatch_index.read_text(encoding="utf-8"))
        badge_status_mismatch_report = Path(str(badge_status_mismatch_doc["reports"]["ci_gate_badge_json"]))
        badge_status_mismatch_badge = json.loads(badge_status_mismatch_report.read_text(encoding="utf-8"))
        badge_status_mismatch_badge["status"] = "fail"
        write_json(badge_status_mismatch_report, badge_status_mismatch_badge)
        badge_status_mismatch_proc = run_check(
            badge_status_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if badge_status_mismatch_proc.returncode == 0:
            return fail("badge status mismatch case must fail")
        if f"fail code={CODES['BADGE_STATUS_MISMATCH']}" not in badge_status_mismatch_proc.stderr:
            return fail(f"badge status mismatch code mismatch: err={badge_status_mismatch_proc.stderr}")

        badge_ok_type_index = build_index_case(root, "badge_ok_type")
        badge_ok_type_doc = json.loads(badge_ok_type_index.read_text(encoding="utf-8"))
        badge_ok_type_report = Path(str(badge_ok_type_doc["reports"]["ci_gate_badge_json"]))
        badge_ok_type_badge = json.loads(badge_ok_type_report.read_text(encoding="utf-8"))
        badge_ok_type_badge["ok"] = "1"
        write_json(badge_ok_type_report, badge_ok_type_badge)
        badge_ok_type_proc = run_check(
            badge_ok_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if badge_ok_type_proc.returncode == 0:
            return fail("badge ok type case must fail")
        if f"fail code={CODES['BADGE_OK_TYPE']}" not in badge_ok_type_proc.stderr:
            return fail(f"badge ok type code mismatch: err={badge_ok_type_proc.stderr}")

        badge_ok_mismatch_index = build_index_case(root, "badge_ok_mismatch")
        badge_ok_mismatch_doc = json.loads(badge_ok_mismatch_index.read_text(encoding="utf-8"))
        badge_ok_mismatch_report = Path(str(badge_ok_mismatch_doc["reports"]["ci_gate_badge_json"]))
        badge_ok_mismatch_badge = json.loads(badge_ok_mismatch_report.read_text(encoding="utf-8"))
        badge_ok_mismatch_badge["ok"] = False
        write_json(badge_ok_mismatch_report, badge_ok_mismatch_badge)
        badge_ok_mismatch_proc = run_check(
            badge_ok_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if badge_ok_mismatch_proc.returncode == 0:
            return fail("badge ok mismatch case must fail")
        if f"fail code={CODES['BADGE_OK_MISMATCH']}" not in badge_ok_mismatch_proc.stderr:
            return fail(f"badge ok mismatch code mismatch: err={badge_ok_mismatch_proc.stderr}")

        badge_result_path_mismatch_index = build_index_case(root, "badge_result_path_mismatch")
        badge_result_path_mismatch_doc = json.loads(badge_result_path_mismatch_index.read_text(encoding="utf-8"))
        badge_result_path_mismatch_report = Path(str(badge_result_path_mismatch_doc["reports"]["ci_gate_badge_json"]))
        badge_result_path_mismatch_badge = json.loads(badge_result_path_mismatch_report.read_text(encoding="utf-8"))
        badge_result_path_mismatch_badge["result_path"] = str(root / "mismatch" / "ci_gate_result.detjson")
        write_json(badge_result_path_mismatch_report, badge_result_path_mismatch_badge)
        badge_result_path_mismatch_proc = run_check(
            badge_result_path_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if badge_result_path_mismatch_proc.returncode == 0:
            return fail("badge result_path mismatch case must fail")
        if f"fail code={CODES['BADGE_RESULT_PATH_MISMATCH']}" not in badge_result_path_mismatch_proc.stderr:
            return fail(f"badge result_path mismatch code mismatch: err={badge_result_path_mismatch_proc.stderr}")

        triage_status_mismatch_index = build_index_case(root, "triage_status_mismatch")
        triage_status_mismatch_doc = json.loads(triage_status_mismatch_index.read_text(encoding="utf-8"))
        triage_status_mismatch_report = Path(str(triage_status_mismatch_doc["reports"]["ci_fail_triage_json"]))
        triage_status_mismatch_triage = json.loads(triage_status_mismatch_report.read_text(encoding="utf-8"))
        triage_status_mismatch_triage["status"] = "fail"
        write_json(triage_status_mismatch_report, triage_status_mismatch_triage)
        triage_status_mismatch_proc = run_check(
            triage_status_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_status_mismatch_proc.returncode == 0:
            return fail("triage status mismatch case must fail")
        if f"fail code={CODES['TRIAGE_STATUS_MISMATCH']}" not in triage_status_mismatch_proc.stderr:
            return fail(f"triage status mismatch code mismatch: err={triage_status_mismatch_proc.stderr}")

        triage_reason_mismatch_index = build_index_case(root, "triage_reason_mismatch")
        triage_reason_mismatch_doc = json.loads(triage_reason_mismatch_index.read_text(encoding="utf-8"))
        triage_reason_mismatch_report = Path(str(triage_reason_mismatch_doc["reports"]["ci_fail_triage_json"]))
        triage_reason_mismatch_triage = json.loads(triage_reason_mismatch_report.read_text(encoding="utf-8"))
        triage_reason_mismatch_triage["reason"] = "different_reason"
        write_json(triage_reason_mismatch_report, triage_reason_mismatch_triage)
        triage_reason_mismatch_proc = run_check(
            triage_reason_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_reason_mismatch_proc.returncode == 0:
            return fail("triage reason mismatch case must fail")
        if f"fail code={CODES['TRIAGE_REASON_MISMATCH']}" not in triage_reason_mismatch_proc.stderr:
            return fail(f"triage reason mismatch code mismatch: err={triage_reason_mismatch_proc.stderr}")

        triage_summary_hint_norm_mismatch_index = build_index_case(root, "triage_summary_hint_norm_mismatch")
        triage_summary_hint_norm_mismatch_doc = json.loads(
            triage_summary_hint_norm_mismatch_index.read_text(encoding="utf-8")
        )
        triage_summary_hint_norm_mismatch_report = Path(
            str(triage_summary_hint_norm_mismatch_doc["reports"]["ci_fail_triage_json"])
        )
        triage_summary_hint_norm_mismatch_triage = json.loads(
            triage_summary_hint_norm_mismatch_report.read_text(encoding="utf-8")
        )
        triage_summary_hint_norm_mismatch_triage["summary_report_path_hint_norm"] = str(
            root / "mismatch" / "ci_gate_summary.txt"
        )
        write_json(triage_summary_hint_norm_mismatch_report, triage_summary_hint_norm_mismatch_triage)
        triage_summary_hint_norm_mismatch_proc = run_check(
            triage_summary_hint_norm_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_summary_hint_norm_mismatch_proc.returncode == 0:
            return fail("triage summary_hint_norm mismatch case must fail")
        if (
            f"fail code={CODES['TRIAGE_SUMMARY_HINT_NORM_MISMATCH']}"
            not in triage_summary_hint_norm_mismatch_proc.stderr
        ):
            return fail(
                "triage summary_hint_norm mismatch code mismatch: "
                f"err={triage_summary_hint_norm_mismatch_proc.stderr}"
            )

        triage_artifacts_missing_index = build_index_case(root, "triage_artifacts_missing")
        triage_artifacts_missing_doc = json.loads(triage_artifacts_missing_index.read_text(encoding="utf-8"))
        triage_artifacts_missing_report = Path(str(triage_artifacts_missing_doc["reports"]["ci_fail_triage_json"]))
        triage_artifacts_missing_triage = json.loads(triage_artifacts_missing_report.read_text(encoding="utf-8"))
        triage_artifacts_missing_triage.pop("artifacts", None)
        write_json(triage_artifacts_missing_report, triage_artifacts_missing_triage)
        triage_artifacts_missing_proc = run_check(
            triage_artifacts_missing_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_artifacts_missing_proc.returncode == 0:
            return fail("triage artifacts missing case must fail")
        if f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}" not in triage_artifacts_missing_proc.stderr:
            return fail(f"triage artifacts missing code mismatch: err={triage_artifacts_missing_proc.stderr}")

        triage_artifact_path_norm_mismatch_index = build_index_case(root, "triage_artifact_path_norm_mismatch")
        triage_artifact_path_norm_mismatch_doc = json.loads(
            triage_artifact_path_norm_mismatch_index.read_text(encoding="utf-8")
        )
        triage_artifact_path_norm_mismatch_report = Path(
            str(triage_artifact_path_norm_mismatch_doc["reports"]["ci_fail_triage_json"])
        )
        triage_artifact_path_norm_mismatch_triage = json.loads(
            triage_artifact_path_norm_mismatch_report.read_text(encoding="utf-8")
        )
        triage_artifact_path_norm_mismatch_triage["artifacts"]["summary"]["path_norm"] = str(
            root / "mismatch" / "ci_gate_summary.txt"
        )
        write_json(triage_artifact_path_norm_mismatch_report, triage_artifact_path_norm_mismatch_triage)
        triage_artifact_path_norm_mismatch_proc = run_check(
            triage_artifact_path_norm_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_artifact_path_norm_mismatch_proc.returncode == 0:
            return fail("triage artifact path_norm mismatch case must fail")
        if (
            f"fail code={CODES['TRIAGE_ARTIFACT_PATH_NORM_MISMATCH']}"
            not in triage_artifact_path_norm_mismatch_proc.stderr
        ):
            return fail(
                "triage artifact path_norm mismatch code mismatch: "
                f"err={triage_artifact_path_norm_mismatch_proc.stderr}"
            )

        triage_artifact_path_mismatch_index = build_index_case(root, "triage_artifact_path_mismatch")
        triage_artifact_path_mismatch_doc = json.loads(triage_artifact_path_mismatch_index.read_text(encoding="utf-8"))
        triage_artifact_path_mismatch_report = Path(str(triage_artifact_path_mismatch_doc["reports"]["ci_fail_triage_json"]))
        triage_artifact_path_mismatch_triage = json.loads(triage_artifact_path_mismatch_report.read_text(encoding="utf-8"))
        triage_artifact_path_mismatch_triage["artifacts"]["summary"]["path"] = str(root / "mismatch" / "ci_gate_summary.txt")
        write_json(triage_artifact_path_mismatch_report, triage_artifact_path_mismatch_triage)
        triage_artifact_path_mismatch_proc = run_check(
            triage_artifact_path_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_artifact_path_mismatch_proc.returncode == 0:
            return fail("triage artifact path mismatch case must fail")
        if f"fail code={CODES['TRIAGE_ARTIFACT_PATH_MISMATCH']}" not in triage_artifact_path_mismatch_proc.stderr:
            return fail(f"triage artifact path mismatch code mismatch: err={triage_artifact_path_mismatch_proc.stderr}")

        triage_artifact_exists_mismatch_index = build_index_case(root, "triage_artifact_exists_mismatch")
        triage_artifact_exists_mismatch_doc = json.loads(
            triage_artifact_exists_mismatch_index.read_text(encoding="utf-8")
        )
        triage_artifact_exists_mismatch_report = Path(
            str(triage_artifact_exists_mismatch_doc["reports"]["ci_fail_triage_json"])
        )
        triage_artifact_exists_mismatch_triage = json.loads(
            triage_artifact_exists_mismatch_report.read_text(encoding="utf-8")
        )
        triage_artifact_exists_mismatch_triage["artifacts"]["summary"]["exists"] = False
        write_json(triage_artifact_exists_mismatch_report, triage_artifact_exists_mismatch_triage)
        triage_artifact_exists_mismatch_proc = run_check(
            triage_artifact_exists_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_artifact_exists_mismatch_proc.returncode == 0:
            return fail("triage artifact exists mismatch case must fail")
        if (
            f"fail code={CODES['TRIAGE_ARTIFACT_EXISTS_MISMATCH']}"
            not in triage_artifact_exists_mismatch_proc.stderr
        ):
            return fail(
                "triage artifact exists mismatch code mismatch: "
                f"err={triage_artifact_exists_mismatch_proc.stderr}"
            )

        cmd_empty_index = build_index_case(root, "cmd_empty")
        cmd_empty_doc = json.loads(cmd_empty_index.read_text(encoding="utf-8"))
        cmd_empty_doc["steps"][0]["cmd"] = []
        write_json(cmd_empty_index, cmd_empty_doc)
        cmd_empty_proc = run_check(
            cmd_empty_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if cmd_empty_proc.returncode == 0:
            return fail("cmd empty case must fail")
        if f"fail code={CODES['STEP_CMD_EMPTY']}" not in cmd_empty_proc.stderr:
            return fail(f"cmd empty code mismatch: err={cmd_empty_proc.stderr}")

        cmd_item_type_index = build_index_case(root, "cmd_item_type")
        cmd_item_type_doc = json.loads(cmd_item_type_index.read_text(encoding="utf-8"))
        cmd_item_type_doc["steps"][0]["cmd"] = ["python", ""]
        write_json(cmd_item_type_index, cmd_item_type_doc)
        cmd_item_type_proc = run_check(
            cmd_item_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if cmd_item_type_proc.returncode == 0:
            return fail("cmd item type case must fail")
        if f"fail code={CODES['STEP_CMD_ITEM_TYPE']}" not in cmd_item_type_proc.stderr:
            return fail(f"cmd item type code mismatch: err={cmd_item_type_proc.stderr}")

        ok_rc_mismatch_index = build_index_case(root, "ok_rc_mismatch")
        ok_rc_mismatch_doc = json.loads(ok_rc_mismatch_index.read_text(encoding="utf-8"))
        ok_rc_mismatch_doc["steps"][0]["ok"] = True
        ok_rc_mismatch_doc["steps"][0]["returncode"] = 1
        write_json(ok_rc_mismatch_index, ok_rc_mismatch_doc)
        ok_rc_mismatch_proc = run_check(
            ok_rc_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if ok_rc_mismatch_proc.returncode == 0:
            return fail("ok/rc mismatch case must fail")
        if f"fail code={CODES['STEP_OK_RC_MISMATCH']}" not in ok_rc_mismatch_proc.stderr:
            return fail(f"ok/rc mismatch code mismatch: err={ok_rc_mismatch_proc.stderr}")

        core_lang_missing_seamgrim_index = build_index_case(
            root,
            "core_lang_missing_seamgrim_steps",
            sanity_profile="core_lang",
        )
        core_lang_doc = json.loads(core_lang_missing_seamgrim_index.read_text(encoding="utf-8"))
        core_lang_doc["steps"] = [
            row
            for row in core_lang_doc["steps"]
            if str(row.get("name", "")) not in set(REQUIRED_STEPS_SEAMGRIM)
        ]
        write_json(core_lang_missing_seamgrim_index, core_lang_doc)
        core_lang_proc = run_check(
            core_lang_missing_seamgrim_index,
            REQUIRED_STEPS_CORE_LANG,
            sanity_profile="core_lang",
            enforce_profile_step_contract=True,
        )
        if core_lang_proc.returncode != 0:
            return fail(f"core_lang profile should allow missing seamgrim steps: out={core_lang_proc.stdout} err={core_lang_proc.stderr}")

        seamgrim_missing_index = build_index_case(
            root,
            "seamgrim_missing_step",
            sanity_profile="seamgrim",
        )
        seamgrim_doc = json.loads(seamgrim_missing_index.read_text(encoding="utf-8"))
        seamgrim_doc["steps"] = [
            row for row in seamgrim_doc["steps"] if str(row.get("name", "")) != "seamgrim_wasm_cli_diag_parity_check"
        ]
        write_json(seamgrim_missing_index, seamgrim_doc)
        seamgrim_missing_proc = run_check(
            seamgrim_missing_index,
            (),
            sanity_profile="seamgrim",
            enforce_profile_step_contract=True,
        )
        if seamgrim_missing_proc.returncode == 0:
            return fail("seamgrim profile missing parity step case must fail")
        if f"fail code={CODES['REQUIRED_STEP_MISSING']}" not in seamgrim_missing_proc.stderr:
            return fail(f"seamgrim profile missing parity step code mismatch: err={seamgrim_missing_proc.stderr}")

    print("[ci-gate-report-index-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
