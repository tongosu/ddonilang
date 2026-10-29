#!/usr/bin/env python
from __future__ import annotations

import json
import io
from contextlib import redirect_stderr, redirect_stdout
import runpy
import subprocess
import sys
import tempfile
from pathlib import Path

from _ci_age5_combined_heavy_contract import (
    build_age5_combined_heavy_child_summary_default_text_transport_fields,
    build_age5_combined_heavy_full_real_source_trace,
)


def fail(msg: str) -> int:
    print(f"[ci-aggregate-status-line-selftest] fail: {msg}")
    return 1


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def run_cmd(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    if len(cmd) < 2 or not str(cmd[1]).endswith(".py"):
        return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")
    script = str(cmd[1])
    argv = [script, *[str(arg) for arg in cmd[2:]]]
    stdout_buf = io.StringIO()
    stderr_buf = io.StringIO()
    old_argv = sys.argv
    returncode = 0
    try:
        sys.argv = argv
        with redirect_stdout(stdout_buf), redirect_stderr(stderr_buf):
            try:
                runpy.run_path(script, run_name="__main__")
            except SystemExit as exc:
                code = exc.code
                if code is None:
                    returncode = 0
                elif isinstance(code, int):
                    returncode = code
                else:
                    returncode = 1
                    stderr_buf.write(str(code))
            except Exception as exc:  # pragma: no cover - defensive fallback
                returncode = 1
                stderr_buf.write(f"{type(exc).__name__}: {exc}")
    finally:
        sys.argv = old_argv
    return subprocess.CompletedProcess(
        args=cmd,
        returncode=returncode,
        stdout=stdout_buf.getvalue(),
        stderr=stderr_buf.getvalue(),
    )


def main() -> int:
    expected_default_transport = build_age5_combined_heavy_child_summary_default_text_transport_fields()
    expected_full_real_source_trace = build_age5_combined_heavy_full_real_source_trace(
        smoke_check_script_exists=True,
        smoke_check_selftest_script_exists=True,
    )
    with tempfile.TemporaryDirectory(prefix="ci_aggregate_status_line_selftest_") as tmp:
        root = Path(tmp)
        aggregate = root / "ci_aggregate_report.detjson"
        status_line = root / "ci_aggregate_status_line.txt"
        parsed = root / "ci_aggregate_status_line_parse.detjson"

        write_json(
            aggregate,
            {
                "schema": "ddn.ci.aggregate_report.v1",
                "generated_at_utc": "2026-02-21T00:00:00+00:00",
                "overall_ok": True,
                "seamgrim": {"ok": True, "failed_steps": []},
                "age3": {"ok": True, "failed_criteria": []},
                "age4": {
                    "ok": True,
                    "failed_criteria": [],
                    "proof_artifact_ok": True,
                    "proof_artifact_failed_criteria": [],
                },
                "age5": {
                    "ok": True,
                    "failed_criteria": [],
                    "age5_combined_heavy_full_real_status": "pass",
                    "full_real_source_trace": expected_full_real_source_trace,
                    "age5_full_real_w107_golden_index_selftest_active_cases": "54",
                    "age5_full_real_w107_golden_index_selftest_inactive_cases": "1",
                    "age5_full_real_w107_golden_index_selftest_index_codes": "34",
                    "age5_full_real_w107_golden_index_selftest_current_probe": "-",
                    "age5_full_real_w107_golden_index_selftest_last_completed_probe": "validate_pack_pointers",
                    "age5_full_real_w107_golden_index_selftest_progress_present": "1",
                    "age5_full_real_w107_progress_contract_selftest_completed_checks": "8",
                    "age5_full_real_w107_progress_contract_selftest_total_checks": "8",
                    "age5_full_real_w107_progress_contract_selftest_checks_text": "golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index",
                    "age5_full_real_w107_progress_contract_selftest_current_probe": "-",
                    "age5_full_real_w107_progress_contract_selftest_last_completed_probe": "report_index",
                    "age5_full_real_w107_progress_contract_selftest_progress_present": "1",
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks": "5",
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks": "5",
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text": "operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family",
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe": "-",
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe": "proof_operation_family",
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present": "1",
                "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks": "5",
                "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks": "5",
                "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text": "signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract",
                "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe": "-",
                "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe": "signed_contract",
                "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present": "1",
                "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks": "1",
                "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks": "1",
                "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text": "verify_report_digest_contract",
                "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe": "-",
                "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe": "readme_and_field_contract",
                "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present": "1",
                "age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks": "4",
                "age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks": "4",
                "age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text": "signed_contract,consumer_contract,promotion,family",
                "age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe": "-",
                "age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe": "family",
                "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present": "1",
                "age5_full_real_proof_certificate_family_contract_selftest_completed_checks": "3",
                "age5_full_real_proof_certificate_family_contract_selftest_total_checks": "3",
                    "age5_full_real_proof_certificate_family_contract_selftest_checks_text": "artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family",
                    "age5_full_real_proof_certificate_family_contract_selftest_current_probe": "-",
                    "age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe": "proof_certificate_family",
                    "age5_full_real_proof_certificate_family_contract_selftest_progress_present": "1",
                    "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks": "9",
                    "age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks": "9",
                    "age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
                    "age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe": "-",
                    "age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe": "report_index",
                    "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present": "1",
                    "age5_full_real_proof_family_contract_selftest_completed_checks": "3",
                    "age5_full_real_proof_family_contract_selftest_total_checks": "3",
                    "age5_full_real_proof_family_contract_selftest_checks_text": "proof_operation_family,proof_certificate_family,proof_family",
                    "age5_full_real_proof_family_contract_selftest_current_probe": "-",
                    "age5_full_real_proof_family_contract_selftest_last_completed_probe": "proof_family",
                    "age5_full_real_proof_family_contract_selftest_progress_present": "1",
                    "age5_full_real_proof_family_transport_contract_selftest_completed_checks": "9",
                    "age5_full_real_proof_family_transport_contract_selftest_total_checks": "9",
                    "age5_full_real_proof_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
                    "age5_full_real_proof_family_transport_contract_selftest_current_probe": "-",
                    "age5_full_real_proof_family_transport_contract_selftest_last_completed_probe": "report_index",
                    "age5_full_real_proof_family_transport_contract_selftest_progress_present": "1",
                    "age5_full_real_lang_surface_family_contract_selftest_completed_checks": "4",
                    "age5_full_real_lang_surface_family_contract_selftest_total_checks": "4",
                    "age5_full_real_lang_surface_family_contract_selftest_checks_text": "proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family",
                    "age5_full_real_lang_surface_family_contract_selftest_current_probe": "-",
                    "age5_full_real_lang_surface_family_contract_selftest_last_completed_probe": "lang_surface_family",
                    "age5_full_real_lang_surface_family_contract_selftest_progress_present": "1",
                    "age5_full_real_lang_runtime_family_contract_selftest_completed_checks": "5",
                    "age5_full_real_lang_runtime_family_contract_selftest_total_checks": "5",
                    "age5_full_real_lang_runtime_family_contract_selftest_checks_text": "lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family",
                    "age5_full_real_lang_runtime_family_contract_selftest_current_probe": "-",
                    "age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe": "lang_runtime_family",
                    "age5_full_real_lang_runtime_family_contract_selftest_progress_present": "1",
                    "age5_full_real_gate0_surface_family_contract_selftest_completed_checks": "5",
                    "age5_full_real_gate0_surface_family_contract_selftest_total_checks": "5",
                    "age5_full_real_gate0_surface_family_contract_selftest_checks_text": "lang_surface_family,lang_runtime_family,gate0_runtime_family,gate0_family,gate0_transport_family",
                    "age5_full_real_gate0_surface_family_contract_selftest_current_probe": "-",
                    "age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe": "gate0_transport_family",
                    "age5_full_real_gate0_surface_family_contract_selftest_progress_present": "1",
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks": "9",
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks": "9",
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe": "-",
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe": "report_index",
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present": "1",
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks": "9",
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks": "9",
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe": "-",
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe": "report_index",
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present": "1",
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks": "1",
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks": "1",
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text": "family_contract",
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe": "-",
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe": "family_contract",
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present": "1",
                    "age5_full_real_gate0_transport_family_contract_selftest_completed_checks": "4",
                    "age5_full_real_gate0_transport_family_contract_selftest_total_checks": "4",
                    "age5_full_real_gate0_transport_family_contract_selftest_checks_text": "lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family",
                    "age5_full_real_gate0_transport_family_contract_selftest_current_probe": "-",
                    "age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe": "gate0_transport_family",
                    "age5_full_real_gate0_transport_family_contract_selftest_progress_present": "1",
                    "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks": "9",
                    "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks": "9",
                    "age5_full_real_lang_surface_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
                    "age5_full_real_lang_surface_family_transport_contract_selftest_current_probe": "-",
                    "age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe": "report_index",
                    "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present": "1",
                    "age5_full_real_bogae_alias_family_contract_selftest_completed_checks": "3",
                    "age5_full_real_bogae_alias_family_contract_selftest_total_checks": "3",
                    "age5_full_real_bogae_alias_family_contract_selftest_checks_text": "shape_alias_contract,alias_family,alias_viewer_family",
                    "age5_full_real_bogae_alias_family_contract_selftest_current_probe": "-",
                    "age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe": "alias_viewer_family",
                    "age5_full_real_bogae_alias_family_contract_selftest_progress_present": "1",
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks": "9",
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks": "9",
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe": "-",
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe": "report_index",
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present": "1",
                    "age5_combined_heavy_runtime_helper_negative_status": "skipped",
                    "age5_combined_heavy_group_id_summary_negative_status": "skipped",
                    **expected_default_transport,
            },
                "oi405_406": {"ok": True, "failed_packs": []},
                "failure_digest": [],
            },
        )

        render = run_cmd(
            [
                sys.executable,
                "tools/scripts/render_ci_aggregate_status_line.py",
                str(aggregate),
                "--out",
                str(status_line),
                "--fail-on-bad",
            ]
        )
        if render.returncode != 0:
            return fail(f"render failed: out={render.stdout} err={render.stderr}")

        parse = run_cmd(
            [
                sys.executable,
                "tools/scripts/parse_ci_aggregate_status_line.py",
                "--status-line",
                str(status_line),
                "--aggregate-report",
                str(aggregate),
                "--json-out",
                str(parsed),
                "--fail-on-invalid",
            ]
        )
        if parse.returncode != 0:
            return fail(f"parse failed: out={parse.stdout} err={parse.stderr}")
        if "age4_failed=0" not in parse.stdout:
            return fail(f"parse compact line missing age4_failed: out={parse.stdout}")
        if "age4_proof_ok=1" not in parse.stdout:
            return fail(f"parse compact line missing age4_proof_ok: out={parse.stdout}")
        if "age4_proof_failed=0" not in parse.stdout:
            return fail(f"parse compact line missing age4_proof_failed: out={parse.stdout}")
        if "age5_failed=0" not in parse.stdout:
            return fail(f"parse compact line missing age5_failed: out={parse.stdout}")
        if "age5_full_real=pass" not in parse.stdout:
            return fail(f"parse compact line missing age5_full_real: out={parse.stdout}")
        if "age5_full_real_source_check=1" not in parse.stdout:
            return fail(f"parse compact line missing age5_full_real_source_check: out={parse.stdout}")
        if "age5_full_real_source_selftest=1" not in parse.stdout:
            return fail(f"parse compact line missing age5_full_real_source_selftest: out={parse.stdout}")
        if "age5_w107_active=54" not in parse.stdout:
            return fail(f"parse compact line missing age5_w107_active: out={parse.stdout}")
        if "age5_w107_inactive=1" not in parse.stdout:
            return fail(f"parse compact line missing age5_w107_inactive: out={parse.stdout}")
        if "age5_w107_index_codes=34" not in parse.stdout:
            return fail(f"parse compact line missing age5_w107_index_codes: out={parse.stdout}")
        if "age5_w107_last_completed_probe=validate_pack_pointers" not in parse.stdout:
            return fail(f"parse compact line missing age5_w107_last_completed_probe: out={parse.stdout}")
        if "age5_w107_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing age5_w107_progress: out={parse.stdout}")
        if "age5_w107_contract_completed=8" not in parse.stdout:
            return fail(f"parse compact line missing age5_w107_contract_completed: out={parse.stdout}")
        if "age5_w107_contract_checks_text=golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index" not in parse.stdout:
            return fail(f"parse compact line missing age5_w107_contract_checks_text: out={parse.stdout}")
        if "age5_w107_contract_last_completed_probe=report_index" not in parse.stdout:
            return fail(f"parse compact line missing age5_w107_contract_last_completed_probe: out={parse.stdout}")
        if "age5_w107_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing age5_w107_contract_progress: out={parse.stdout}")
        if "age5_age1_immediate_proof_operation_contract_completed=5" not in parse.stdout:
            return fail(
                f"parse compact line missing age5_age1_immediate_proof_operation_contract_completed: out={parse.stdout}"
            )
        if "age5_age1_immediate_proof_operation_contract_checks_text=operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family" not in parse.stdout:
            return fail(
                f"parse compact line missing age5_age1_immediate_proof_operation_contract_checks_text: out={parse.stdout}"
            )
        if "age5_age1_immediate_proof_operation_contract_last_completed_probe=proof_operation_family" not in parse.stdout:
            return fail(
                f"parse compact line missing age5_age1_immediate_proof_operation_contract_last_completed_probe: out={parse.stdout}"
            )
        if "age5_age1_immediate_proof_operation_contract_progress=1" not in parse.stdout:
            return fail(
                f"parse compact line missing age5_age1_immediate_proof_operation_contract_progress: out={parse.stdout}"
            )
        if "age5_proof_certificate_v1_consumer_contract_completed=5" not in parse.stdout:
            return fail(
                f"parse compact line missing age5_proof_certificate_v1_consumer_contract_completed: out={parse.stdout}"
            )
        if "age5_proof_certificate_v1_consumer_contract_checks_text=signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract" not in parse.stdout:
            return fail(
                f"parse compact line missing age5_proof_certificate_v1_consumer_contract_checks_text: out={parse.stdout}"
            )
        if "age5_proof_certificate_v1_consumer_contract_last_completed_probe=signed_contract" not in parse.stdout:
            return fail(
                f"parse compact line missing age5_proof_certificate_v1_consumer_contract_last_completed_probe: out={parse.stdout}"
            )
        if "age5_proof_certificate_v1_consumer_contract_progress=1" not in parse.stdout:
            return fail(
                f"parse compact line missing age5_proof_certificate_v1_consumer_contract_progress: out={parse.stdout}"
            )
        if "age5_proof_certificate_v1_verify_report_digest_contract_completed=1" not in parse.stdout:
            return fail(f"parse compact line missing verify_report_digest completed: out={parse.stdout}")
        if "age5_proof_certificate_v1_verify_report_digest_contract_checks_text=verify_report_digest_contract" not in parse.stdout:
            return fail(f"parse compact line missing verify_report_digest checks_text: out={parse.stdout}")
        if "age5_proof_certificate_v1_verify_report_digest_contract_last_completed_probe=readme_and_field_contract" not in parse.stdout:
            return fail(f"parse compact line missing verify_report_digest last_completed_probe: out={parse.stdout}")
        if "age5_proof_certificate_v1_verify_report_digest_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing verify_report_digest progress: out={parse.stdout}")
        if "age5_proof_certificate_v1_family_contract_checks_text=signed_contract,consumer_contract,promotion,family" not in parse.stdout:
            return fail(f"parse compact line missing proof_certificate family checks_text: out={parse.stdout}")
        if "age5_proof_certificate_v1_family_contract_last_completed_probe=family" not in parse.stdout:
            return fail(f"parse compact line missing proof_certificate family last_completed_probe: out={parse.stdout}")
        if "age5_proof_certificate_v1_family_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing proof_certificate family progress: out={parse.stdout}")
        if "age5_proof_certificate_family_contract_checks_text=artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family" not in parse.stdout:
            return fail(f"parse compact line missing proof_certificate top-family checks_text: out={parse.stdout}")
        if "age5_proof_certificate_family_contract_last_completed_probe=proof_certificate_family" not in parse.stdout:
            return fail(f"parse compact line missing proof_certificate top-family last_completed_probe: out={parse.stdout}")
        if "age5_proof_certificate_family_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing proof_certificate top-family progress: out={parse.stdout}")
        if "age5_proof_certificate_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in parse.stdout:
            return fail(f"parse compact line missing proof_certificate top-family transport checks_text: out={parse.stdout}")
        if "age5_proof_certificate_family_transport_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing proof_certificate top-family transport progress: out={parse.stdout}")
        if "age5_proof_family_contract_completed=3" not in parse.stdout:
            return fail(f"parse compact line missing proof_family completed: out={parse.stdout}")
        if "age5_proof_family_contract_checks_text=proof_operation_family,proof_certificate_family,proof_family" not in parse.stdout:
            return fail(f"parse compact line missing proof_family checks_text: out={parse.stdout}")
        if "age5_proof_family_contract_last_completed_probe=proof_family" not in parse.stdout:
            return fail(f"parse compact line missing proof_family last_completed_probe: out={parse.stdout}")
        if "age5_proof_family_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing proof_family progress: out={parse.stdout}")
        if "age5_proof_family_transport_contract_completed=9" not in parse.stdout:
            return fail(f"parse compact line missing proof_family transport completed: out={parse.stdout}")
        if "age5_proof_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in parse.stdout:
            return fail(f"parse compact line missing proof_family transport checks_text: out={parse.stdout}")
        if "age5_proof_family_transport_contract_last_completed_probe=report_index" not in parse.stdout:
            return fail(f"parse compact line missing proof_family transport last_completed_probe: out={parse.stdout}")
        if "age5_proof_family_transport_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing proof_family transport progress: out={parse.stdout}")
        if "age5_lang_surface_family_contract_completed=4" not in parse.stdout:
            return fail(f"parse compact line missing lang_surface_family completed: out={parse.stdout}")
        if "age5_lang_surface_family_contract_checks_text=proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family" not in parse.stdout:
            return fail(f"parse compact line missing lang_surface_family checks_text: out={parse.stdout}")
        if "age5_lang_surface_family_contract_last_completed_probe=lang_surface_family" not in parse.stdout:
            return fail(f"parse compact line missing lang_surface_family last_completed_probe: out={parse.stdout}")
        if "age5_lang_surface_family_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing lang_surface_family progress: out={parse.stdout}")
        if "age5_lang_runtime_family_contract_completed=5" not in parse.stdout:
            return fail(f"parse compact line missing lang_runtime_family completed: out={parse.stdout}")
        if "age5_lang_runtime_family_contract_checks_text=lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family" not in parse.stdout:
            return fail(f"parse compact line missing lang_runtime_family checks_text: out={parse.stdout}")
        if "age5_lang_runtime_family_contract_last_completed_probe=lang_runtime_family" not in parse.stdout:
            return fail(f"parse compact line missing lang_runtime_family last_completed_probe: out={parse.stdout}")
        if "age5_lang_runtime_family_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing lang_runtime_family progress: out={parse.stdout}")
        if "age5_lang_runtime_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in parse.stdout:
            return fail(f"parse compact line missing lang_runtime_family transport checks_text: out={parse.stdout}")
        if "age5_lang_runtime_family_transport_contract_last_completed_probe=report_index" not in parse.stdout:
            return fail(f"parse compact line missing lang_runtime_family transport last_completed_probe: out={parse.stdout}")
        if "age5_lang_runtime_family_transport_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing lang_runtime_family transport progress: out={parse.stdout}")
        if "age5_gate0_runtime_family_transport_contract_checks_text=family_contract" not in parse.stdout:
            return fail(f"parse compact line missing gate0_runtime_family transport checks_text: out={parse.stdout}")
        if "age5_gate0_runtime_family_transport_contract_last_completed_probe=family_contract" not in parse.stdout:
            return fail(f"parse compact line missing gate0_runtime_family transport last_completed_probe: out={parse.stdout}")
        if "age5_gate0_runtime_family_transport_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing gate0_runtime_family transport progress: out={parse.stdout}")
        if "age5_gate0_transport_family_contract_checks_text=lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family" not in parse.stdout:
            return fail(f"parse compact line missing gate0_transport_family transport checks_text: out={parse.stdout}")
        if "age5_gate0_surface_family_contract_checks_text=lang_surface_family,lang_runtime_family,gate0_runtime_family,gate0_family,gate0_transport_family" not in parse.stdout:
            return fail(f"parse compact line missing gate0_surface_family checks_text: out={parse.stdout}")
        if "age5_gate0_surface_family_contract_last_completed_probe=gate0_transport_family" not in parse.stdout:
            return fail(f"parse compact line missing gate0_surface_family last_completed_probe: out={parse.stdout}")
        if "age5_gate0_surface_family_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing gate0_surface_family progress: out={parse.stdout}")
        if "age5_gate0_surface_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in parse.stdout:
            return fail(f"parse compact line missing gate0_surface_family transport checks_text: out={parse.stdout}")
        if "age5_gate0_surface_family_transport_contract_last_completed_probe=report_index" not in parse.stdout:
            return fail(f"parse compact line missing gate0_surface_family transport last_completed_probe: out={parse.stdout}")
        if "age5_gate0_surface_family_transport_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing gate0_surface_family transport progress: out={parse.stdout}")
        if "age5_gate0_transport_family_contract_last_completed_probe=gate0_transport_family" not in parse.stdout:
            return fail(f"parse compact line missing gate0_transport_family transport last_completed_probe: out={parse.stdout}")
        if "age5_gate0_transport_family_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing gate0_transport_family transport progress: out={parse.stdout}")
        if "age5_lang_surface_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in parse.stdout:
            return fail(f"parse compact line missing lang_surface_family transport checks_text: out={parse.stdout}")
        if "age5_lang_surface_family_transport_contract_last_completed_probe=report_index" not in parse.stdout:
            return fail(f"parse compact line missing lang_surface_family transport last_completed_probe: out={parse.stdout}")
        if "age5_lang_surface_family_transport_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing lang_surface_family transport progress: out={parse.stdout}")
        if "age5_bogae_alias_family_contract_completed=3" not in parse.stdout:
            return fail(f"parse compact line missing bogae alias family completed: out={parse.stdout}")
        if "age5_bogae_alias_family_contract_checks_text=shape_alias_contract,alias_family,alias_viewer_family" not in parse.stdout:
            return fail(f"parse compact line missing bogae alias family checks_text: out={parse.stdout}")
        if "age5_bogae_alias_family_contract_last_completed_probe=alias_viewer_family" not in parse.stdout:
            return fail(f"parse compact line missing bogae alias family last_completed_probe: out={parse.stdout}")
        if "age5_bogae_alias_family_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing bogae alias family progress: out={parse.stdout}")
        if "age5_bogae_alias_family_transport_contract_completed=9" not in parse.stdout:
            return fail(f"parse compact line missing bogae alias family transport completed: out={parse.stdout}")
        if "age5_bogae_alias_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in parse.stdout:
            return fail(f"parse compact line missing bogae alias family transport checks_text: out={parse.stdout}")
        if "age5_bogae_alias_family_transport_contract_last_completed_probe=report_index" not in parse.stdout:
            return fail(f"parse compact line missing bogae alias family transport last_completed_probe: out={parse.stdout}")
        if "age5_bogae_alias_family_transport_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing bogae alias family transport progress: out={parse.stdout}")
        if "age5_runtime_helper_negative=skipped" not in parse.stdout:
            return fail(f"parse compact line missing runtime-helper child status: out={parse.stdout}")
        if "age5_group_id_summary_negative=skipped" not in parse.stdout:
            return fail(f"parse compact line missing group-id child status: out={parse.stdout}")
        if (
            f"age5_child_summary_defaults={expected_default_transport['ci_sanity_age5_combined_heavy_child_summary_default_fields']}"
            not in parse.stdout
        ):
            return fail(f"parse compact line missing child-summary defaults: out={parse.stdout}")
        if (
            "age5_sync_child_summary_defaults="
            f"{expected_default_transport['ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields']}"
            not in parse.stdout
        ):
            return fail(f"parse compact line missing sync child-summary defaults: out={parse.stdout}")

        check = run_cmd(
            [
                sys.executable,
                "tests/run_ci_aggregate_status_line_check.py",
                "--status-line",
                str(status_line),
                "--aggregate-report",
                str(aggregate),
                "--require-pass",
            ]
        )
        if check.returncode != 0:
            return fail(f"check failed: out={check.stdout} err={check.stderr}")

        # negative: new child-summary key dropped from status line must fail validation
        line = status_line.read_text(encoding="utf-8").strip()
        broken_line = line.replace(" age5_full_real_w107_golden_index_selftest_active_cases=54", "")
        status_line.write_text(broken_line + "\n", encoding="utf-8")
        broken = run_cmd(
            [
                sys.executable,
                "tests/run_ci_aggregate_status_line_check.py",
                "--status-line",
                str(status_line),
                "--aggregate-report",
                str(aggregate),
            ]
        )
        if broken.returncode == 0:
            return fail("broken status-line case must fail")

        render = run_cmd(
            [
                sys.executable,
                "tools/scripts/render_ci_aggregate_status_line.py",
                str(aggregate),
                "--out",
                str(status_line),
                "--fail-on-bad",
            ]
        )
        if render.returncode != 0:
            return fail(f"rerender-age1 failed: out={render.stdout} err={render.stderr}")
        line = status_line.read_text(encoding="utf-8").strip()
        broken_line = line.replace(
            " age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks=5",
            "",
        )
        status_line.write_text(broken_line + "\n", encoding="utf-8")
        broken_age1 = run_cmd(
            [
                sys.executable,
                "tests/run_ci_aggregate_status_line_check.py",
                "--status-line",
                str(status_line),
                "--aggregate-report",
                str(aggregate),
            ]
        )
        if broken_age1.returncode == 0:
            return fail("broken age1 immediate proof operation contract case must fail")

        render = run_cmd(
            [
                sys.executable,
                "tools/scripts/render_ci_aggregate_status_line.py",
                str(aggregate),
                "--out",
                str(status_line),
                "--fail-on-bad",
            ]
        )
        if render.returncode != 0:
            return fail(f"rerender failed: out={render.stdout} err={render.stderr}")
        line = status_line.read_text(encoding="utf-8").strip()
        broken_line = line.replace(" age5_combined_heavy_runtime_helper_negative_status=skipped", "")
        status_line.write_text(broken_line + "\n", encoding="utf-8")
        broken_child = run_cmd(
            [
                sys.executable,
                "tests/run_ci_aggregate_status_line_check.py",
                "--status-line",
                str(status_line),
                "--aggregate-report",
                str(aggregate),
            ]
        )
        if broken_child.returncode == 0:
            return fail("broken child-status case must fail")

        render = run_cmd(
            [
                sys.executable,
                "tools/scripts/render_ci_aggregate_status_line.py",
                str(aggregate),
                "--out",
                str(status_line),
                "--fail-on-bad",
            ]
        )
        if render.returncode != 0:
            return fail(f"rerender-default failed: out={render.stdout} err={render.stderr}")
        line = status_line.read_text(encoding="utf-8").strip()
        broken_line = line.replace(
            ' ci_sanity_age5_combined_heavy_child_summary_default_fields="'
            + expected_default_transport["ci_sanity_age5_combined_heavy_child_summary_default_fields"]
            + '"',
            "",
        )
        status_line.write_text(broken_line + "\n", encoding="utf-8")
        broken_default = run_cmd(
            [
                sys.executable,
                "tests/run_ci_aggregate_status_line_check.py",
                "--status-line",
                str(status_line),
                "--aggregate-report",
                str(aggregate),
            ]
        )
        if broken_default.returncode == 0:
            return fail("broken default transport case must fail")

        render = run_cmd(
            [
                sys.executable,
                "tools/scripts/render_ci_aggregate_status_line.py",
                str(aggregate),
                "--out",
                str(status_line),
                "--fail-on-bad",
            ]
        )
        if render.returncode != 0:
            return fail(f"rerender-2 failed: out={render.stdout} err={render.stderr}")
        line = status_line.read_text(encoding="utf-8").strip()
        broken_line = line.replace(" age4_proof_ok=1", "")
        status_line.write_text(broken_line + "\n", encoding="utf-8")
        broken_age4_proof = run_cmd(
            [
                sys.executable,
                "tests/run_ci_aggregate_status_line_check.py",
                "--status-line",
                str(status_line),
                "--aggregate-report",
                str(aggregate),
            ]
        )
        if broken_age4_proof.returncode == 0:
            return fail("broken age4 proof status case must fail")

    print("[ci-aggregate-status-line-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
