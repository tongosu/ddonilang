#!/usr/bin/env python
from __future__ import annotations

from contextlib import contextmanager, redirect_stderr, redirect_stdout
import io
import json
import os
import runpy
import subprocess
import sys
import traceback
from pathlib import Path
from tempfile import mkdtemp

ROOT = Path(__file__).resolve().parents[1]


@contextmanager
def persistent_tmpdir(prefix: str):
    yield mkdtemp(prefix=prefix)


def fail(msg: str) -> int:
    print(f"[ci-gate-final-status-line-selftest] fail: {msg}")
    return 1


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(payload, ensure_ascii=False, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


def normalize_cmd(cmd: list[str]) -> list[str]:
    if len(cmd) >= 2 and cmd[0] == sys.executable and cmd[1] != "-S":
        return [cmd[0], "-S", *cmd[1:]]
    return cmd


def _coerce_return_code(code: object) -> int:
    if code is None:
        return 0
    if isinstance(code, int):
        return int(code)
    return 1


def _run_python_inproc(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    script_index = 2 if len(cmd) >= 3 and cmd[1] == "-S" else 1
    script = str(cmd[script_index]).strip()
    argv = [script, *cmd[script_index + 1 :]]
    stdout_buf = io.StringIO()
    stderr_buf = io.StringIO()
    prev_argv = list(sys.argv)
    prev_cwd = Path.cwd()
    returncode = 0
    try:
        sys.argv = argv
        os.chdir(ROOT)
        with redirect_stdout(stdout_buf), redirect_stderr(stderr_buf):
            try:
                runpy.run_path(script, run_name="__main__")
            except SystemExit as exc:
                returncode = _coerce_return_code(exc.code)
            except Exception:
                traceback.print_exc(file=stderr_buf)
                returncode = 1
    finally:
        sys.argv = prev_argv
        os.chdir(prev_cwd)
    return subprocess.CompletedProcess(cmd, returncode, stdout_buf.getvalue(), stderr_buf.getvalue())


def run_cmd(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    normalized = normalize_cmd(cmd)
    if len(normalized) >= 2 and normalized[0] == sys.executable:
        script_index = 2 if len(normalized) >= 3 and normalized[1] == "-S" else 1
        if len(normalized) > script_index and str(normalized[script_index]).strip().endswith(".py"):
            return _run_python_inproc(normalized)
    return subprocess.run(normalized, capture_output=True, text=True, encoding="utf-8", errors="replace")


def main() -> int:
    with persistent_tmpdir(prefix="ci_gate_final_status_line_selftest_") as tmp:
        root = Path(tmp)
        aggregate_status_parse = root / "ci_aggregate_status_line_parse.detjson"
        aggregate_report = root / "ci_aggregate_report.detjson"
        gate_index = root / "ci_gate_report_index.detjson"
        final_status_line = root / "ci_gate_final_status_line.txt"
        final_status_parse = root / "ci_gate_final_status_line_parse.detjson"

        write_json(
            aggregate_status_parse,
            {
                "schema": "ddn.ci.aggregate_gate_status_line_parse.v1",
                "status_line_path": str(root / "ci_aggregate_status_line.txt"),
                "parsed": {
                    "schema": "ddn.ci.aggregate_gate_status_line.v1",
                    "status": "pass",
                    "overall_ok": "1",
                    "seamgrim_failed_steps": "0",
                    "age3_failed_criteria": "0",
                "age4_failed_criteria": "0",
                "age4_proof_ok": "1",
                "age4_proof_failed_criteria": "0",
                "age5_failed_criteria": "0",
                "age5_combined_heavy_full_real_status": "pass",
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
                "ci_sanity_age5_combined_heavy_child_summary_default_fields": "child_defaults",
                "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields": "sync_child_defaults",
                    "oi_failed_packs": "0",
                    "report_path": str(root / "ci_aggregate_report.detjson"),
                    "generated_at_utc": "2026-03-19T00:00:00+00:00",
                    "reason": "-",
                },
            },
        )
        write_json(
            aggregate_report,
            {
                "schema": "ddn.ci.aggregate_gate.report.v1",
                "age4": {
                    "proof_artifact_ok": True,
                    "proof_artifact_failed_criteria": [],
                    "proof_artifact_failed_preview": "-",
                    "proof_artifact_summary_hash": "sha256:selftest",
                },
            },
        )
        write_json(
            gate_index,
            {
                "schema": "ddn.ci.aggregate_gate.index.v1",
                "report_prefix": "selftest",
                "reports": {"aggregate": str(aggregate_report)},
                "steps": [
                    {"name": "seamgrim_ci_gate", "ok": True},
                    {"name": "age3_close", "ok": True},
                ],
            },
        )

        render = run_cmd(
            [
                sys.executable,
                "tools/scripts/render_ci_gate_final_status_line.py",
                "--aggregate-status-parse",
                str(aggregate_status_parse),
                "--gate-index",
                str(gate_index),
                "--out",
                str(final_status_line),
                "--fail-on-bad",
            ]
        )
        if render.returncode != 0:
            return fail(f"render failed: out={render.stdout} err={render.stderr}")

        parse = run_cmd(
            [
                sys.executable,
                "tools/scripts/parse_ci_gate_final_status_line.py",
                "--status-line",
                str(final_status_line),
                "--gate-index",
                str(gate_index),
                "--json-out",
                str(final_status_parse),
                "--fail-on-invalid",
            ]
        )
        if parse.returncode != 0:
            return fail(f"parse failed: out={parse.stdout} err={parse.stderr}")
        if "age4_proof_ok=1" not in parse.stdout:
            return fail(f"parse compact line missing age4_proof_ok: out={parse.stdout}")
        if "age4_proof_failed=0" not in parse.stdout:
            return fail(f"parse compact line missing age4_proof_failed: out={parse.stdout}")
        if "age5_w107_active=54" not in parse.stdout:
            return fail(f"parse compact line missing age5_w107_active: out={parse.stdout}")
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
        if "age5_proof_certificate_v1_consumer_contract_progress=1" not in parse.stdout:
            return fail(
                f"parse compact line missing age5_proof_certificate_v1_consumer_contract_progress: out={parse.stdout}"
            )
        if "age5_proof_certificate_v1_verify_report_digest_contract_completed=1" not in parse.stdout:
            return fail(f"parse compact line missing verify_report_digest completed: out={parse.stdout}")
        if "age5_proof_certificate_v1_verify_report_digest_contract_checks_text=verify_report_digest_contract" not in parse.stdout:
            return fail(f"parse compact line missing verify_report_digest checks_text: out={parse.stdout}")
        if "age5_proof_certificate_v1_verify_report_digest_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing verify_report_digest progress: out={parse.stdout}")
        if "age5_proof_certificate_v1_family_contract_checks_text=signed_contract,consumer_contract,promotion,family" not in parse.stdout:
            return fail(f"parse compact line missing proof_certificate family checks_text: out={parse.stdout}")
        if "age5_proof_certificate_v1_family_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing proof_certificate family progress: out={parse.stdout}")
        if "age5_proof_certificate_family_contract_checks_text=artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family" not in parse.stdout:
            return fail(f"parse compact line missing top-level proof_certificate family checks_text: out={parse.stdout}")
        if "age5_proof_certificate_family_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing top-level proof_certificate family progress: out={parse.stdout}")
        if "age5_proof_certificate_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in parse.stdout:
            return fail(f"parse compact line missing top-level proof_certificate family transport checks_text: out={parse.stdout}")
        if "age5_proof_certificate_family_transport_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing top-level proof_certificate family transport progress: out={parse.stdout}")
        if "age5_proof_family_contract_checks_text=proof_operation_family,proof_certificate_family,proof_family" not in parse.stdout:
            return fail(f"parse compact line missing proof_family checks_text: out={parse.stdout}")
        if "age5_proof_family_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing proof_family progress: out={parse.stdout}")
        if "age5_proof_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in parse.stdout:
            return fail(f"parse compact line missing proof_family transport checks_text: out={parse.stdout}")
        if "age5_proof_family_transport_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing proof_family transport progress: out={parse.stdout}")
        if "age5_lang_surface_family_contract_checks_text=proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family" not in parse.stdout:
            return fail(f"parse compact line missing lang_surface_family checks_text: out={parse.stdout}")
        if "age5_lang_surface_family_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing lang_surface_family progress: out={parse.stdout}")
        if "age5_lang_runtime_family_contract_checks_text=lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family" not in parse.stdout:
            return fail(f"parse compact line missing lang_runtime_family checks_text: out={parse.stdout}")
        if "age5_lang_runtime_family_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing lang_runtime_family progress: out={parse.stdout}")
        if "age5_lang_runtime_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in parse.stdout:
            return fail(f"parse compact line missing lang_runtime_family transport checks_text: out={parse.stdout}")
        if "age5_lang_runtime_family_transport_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing lang_runtime_family transport progress: out={parse.stdout}")
        if "age5_gate0_runtime_family_transport_contract_checks_text=family_contract" not in parse.stdout:
            return fail(f"parse compact line missing gate0_runtime_family transport checks_text: out={parse.stdout}")
        if "age5_gate0_runtime_family_transport_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing gate0_runtime_family transport progress: out={parse.stdout}")
        if "age5_gate0_transport_family_contract_checks_text=lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family" not in parse.stdout:
            return fail(f"parse compact line missing gate0_transport_family transport checks_text: out={parse.stdout}")
        if "age5_gate0_surface_family_contract_checks_text=lang_surface_family,lang_runtime_family,gate0_runtime_family,gate0_family,gate0_transport_family" not in parse.stdout:
            return fail(f"parse compact line missing gate0_surface_family checks_text: out={parse.stdout}")
        if "age5_gate0_surface_family_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing gate0_surface_family progress: out={parse.stdout}")
        if "age5_gate0_surface_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in parse.stdout:
            return fail(f"parse compact line missing gate0_surface_family transport checks_text: out={parse.stdout}")
        if "age5_gate0_surface_family_transport_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing gate0_surface_family transport progress: out={parse.stdout}")
        if "age5_gate0_transport_family_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing gate0_transport_family transport progress: out={parse.stdout}")
        if "age5_lang_surface_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in parse.stdout:
            return fail(f"parse compact line missing lang_surface_family transport checks_text: out={parse.stdout}")
        if "age5_lang_surface_family_transport_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing lang_surface_family transport progress: out={parse.stdout}")
        if "age5_bogae_alias_family_contract_checks_text=shape_alias_contract,alias_family,alias_viewer_family" not in parse.stdout:
            return fail(f"parse compact line missing bogae alias family checks_text: out={parse.stdout}")
        if "age5_bogae_alias_family_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing bogae alias family progress: out={parse.stdout}")
        if "age5_bogae_alias_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in parse.stdout:
            return fail(f"parse compact line missing bogae alias family transport checks_text: out={parse.stdout}")
        if "age5_bogae_alias_family_transport_contract_progress=1" not in parse.stdout:
            return fail(f"parse compact line missing bogae alias family transport progress: out={parse.stdout}")
        parse_doc = json.loads(final_status_parse.read_text(encoding="utf-8"))
        parsed = parse_doc.get("parsed", {})
        if str(parsed.get("age4_proof_failed_preview", "")).strip() != "-":
            return fail(f"parse json missing age4_proof_failed_preview: doc={parse_doc}")
        if str(parsed.get("age5_policy_age4_proof_gate_result_present", "")).strip() != "0":
            return fail(f"parse json missing age5 policy parity default: doc={parse_doc}")
        if str(parsed.get("age5_full_real_w107_golden_index_selftest_active_cases", "")).strip() != "54":
            return fail(f"parse json missing age5 w107 active_cases: doc={parse_doc}")
        if str(parsed.get("age5_full_real_w107_progress_contract_selftest_completed_checks", "")).strip() != "8":
            return fail(f"parse json missing age5 w107 contract completed_checks: doc={parse_doc}")
        if str(parsed.get("age5_full_real_w107_progress_contract_selftest_checks_text", "")).strip() != "golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index":
            return fail(f"parse json missing age5 w107 contract checks_text: doc={parse_doc}")
        if str(parsed.get("age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks", "")).strip() != "5":
            return fail(f"parse json missing age5 age1 contract completed_checks: doc={parse_doc}")
        if str(parsed.get("age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text", "")).strip() != "operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family":
            return fail(f"parse json missing age5 age1 contract checks_text: doc={parse_doc}")
        if str(parsed.get("age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks", "")).strip() != "5":
            return fail(f"parse json missing proof_certificate consumer completed_checks: doc={parse_doc}")
        if str(parsed.get("age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text", "")).strip() != "signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract":
            return fail(f"parse json missing proof_certificate consumer checks_text: doc={parse_doc}")
        if str(parsed.get("age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks", "")).strip() != "1":
            return fail(f"parse json missing proof_certificate digest completed_checks: doc={parse_doc}")
        if str(parsed.get("age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text", "")).strip() != "verify_report_digest_contract":
            return fail(f"parse json missing proof_certificate digest checks_text: doc={parse_doc}")
        if str(parsed.get("age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks", "")).strip() != "4":
            return fail(f"parse json missing proof_certificate family completed_checks: doc={parse_doc}")
        if str(parsed.get("age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text", "")).strip() != "signed_contract,consumer_contract,promotion,family":
            return fail(f"parse json missing proof_certificate family checks_text: doc={parse_doc}")
        if str(parsed.get("age5_full_real_proof_certificate_family_contract_selftest_completed_checks", "")).strip() != "3":
            return fail(f"parse json missing top-level proof_certificate family completed_checks: doc={parse_doc}")
        if str(parsed.get("age5_full_real_proof_certificate_family_contract_selftest_checks_text", "")).strip() != "artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family":
            return fail(f"parse json missing top-level proof_certificate family checks_text: doc={parse_doc}")
        if str(parsed.get("age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks", "")).strip() != "9":
            return fail(f"parse json missing top-level proof_certificate family transport completed_checks: doc={parse_doc}")
        if str(parsed.get("age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail(f"parse json missing top-level proof_certificate family transport checks_text: doc={parse_doc}")
        if str(parsed.get("age5_full_real_proof_family_contract_selftest_completed_checks", "")).strip() != "3":
            return fail(f"parse json missing proof_family completed_checks: doc={parse_doc}")
        if str(parsed.get("age5_full_real_proof_family_contract_selftest_checks_text", "")).strip() != "proof_operation_family,proof_certificate_family,proof_family":
            return fail(f"parse json missing proof_family checks_text: doc={parse_doc}")
        if str(parsed.get("age5_full_real_proof_family_transport_contract_selftest_completed_checks", "")).strip() != "9":
            return fail(f"parse json missing proof_family transport completed_checks: doc={parse_doc}")
        if str(parsed.get("age5_full_real_proof_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail(f"parse json missing proof_family transport checks_text: doc={parse_doc}")
        if str(parsed.get("age5_full_real_lang_surface_family_contract_selftest_completed_checks", "")).strip() != "4":
            return fail(f"parse json missing lang_surface_family completed_checks: doc={parse_doc}")
        if str(parsed.get("age5_full_real_lang_surface_family_contract_selftest_checks_text", "")).strip() != "proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family":
            return fail(f"parse json missing lang_surface_family checks_text: doc={parse_doc}")
        if str(parsed.get("age5_full_real_lang_runtime_family_contract_selftest_completed_checks", "")).strip() != "5":
            return fail(f"parse json missing lang_runtime_family completed_checks: doc={parse_doc}")
        if str(parsed.get("age5_full_real_lang_runtime_family_contract_selftest_checks_text", "")).strip() != "lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family":
            return fail(f"parse json missing lang_runtime_family checks_text: doc={parse_doc}")
        if str(parsed.get("age5_full_real_bogae_alias_family_contract_selftest_completed_checks", "")).strip() != "3":
            return fail(f"parse json missing bogae alias family completed_checks: doc={parse_doc}")
        if str(parsed.get("age5_full_real_bogae_alias_family_contract_selftest_checks_text", "")).strip() != "shape_alias_contract,alias_family,alias_viewer_family":
            return fail(f"parse json missing bogae alias family checks_text: doc={parse_doc}")
        if str(parsed.get("age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks", "")).strip() != "9":
            return fail(f"parse json missing bogae alias family transport completed_checks: doc={parse_doc}")
        if str(parsed.get("age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail(f"parse json missing bogae alias family transport checks_text: doc={parse_doc}")

        check = run_cmd(
            [
                sys.executable,
                "tests/run_ci_gate_final_status_line_check.py",
                "--status-line",
                str(final_status_line),
                "--aggregate-status-parse",
                str(aggregate_status_parse),
                "--gate-index",
                str(gate_index),
                "--require-pass",
            ]
        )
        if check.returncode != 0:
            return fail(f"check failed: out={check.stdout} err={check.stderr}")
        if "age4_proof_ok=1" not in check.stdout:
            return fail(f"check output missing age4_proof_ok: out={check.stdout}")
        if "age5_w107_active=54" not in check.stdout:
            return fail(f"check output missing age5_w107_active: out={check.stdout}")
        if "age5_w107_contract_completed=8" not in check.stdout:
            return fail(f"check output missing age5_w107_contract_completed: out={check.stdout}")
        if "age5_w107_contract_checks_text=golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index" not in check.stdout:
            return fail(f"check output missing age5_w107_contract_checks_text: out={check.stdout}")
        if "age5_age1_immediate_proof_operation_contract_completed=5" not in check.stdout:
            return fail(
                f"check output missing age5_age1_immediate_proof_operation_contract_completed: out={check.stdout}"
            )
        if "age5_age1_immediate_proof_operation_contract_checks_text=operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family" not in check.stdout:
            return fail(
                f"check output missing age5_age1_immediate_proof_operation_contract_checks_text: out={check.stdout}"
            )
        if "age5_proof_certificate_v1_consumer_contract_checks_text=signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract" not in check.stdout:
            return fail(
                f"check output missing proof_certificate consumer checks_text: out={check.stdout}"
            )
        if "age5_proof_certificate_v1_verify_report_digest_contract_checks_text=verify_report_digest_contract" not in check.stdout:
            return fail(f"check output missing proof_certificate digest checks_text: out={check.stdout}")
        if "age5_proof_certificate_v1_family_contract_checks_text=signed_contract,consumer_contract,promotion,family" not in check.stdout:
            return fail(f"check output missing proof_certificate family checks_text: out={check.stdout}")
        if "age5_proof_certificate_family_contract_checks_text=artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family" not in check.stdout:
            return fail(f"check output missing top-level proof_certificate family checks_text: out={check.stdout}")
        if "age5_proof_family_contract_checks_text=proof_operation_family,proof_certificate_family,proof_family" not in check.stdout:
            return fail(f"check output missing proof_family checks_text: out={check.stdout}")
        if "age5_proof_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in check.stdout:
            return fail(f"check output missing proof_family transport checks_text: out={check.stdout}")
        if "age5_lang_surface_family_contract_checks_text=proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family" not in check.stdout:
            return fail(f"check output missing lang_surface_family checks_text: out={check.stdout}")
        if "age5_lang_runtime_family_contract_checks_text=lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family" not in check.stdout:
            return fail(f"check output missing lang_runtime_family checks_text: out={check.stdout}")
        if "age5_lang_runtime_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in check.stdout:
            return fail(f"check output missing lang_runtime_family transport checks_text: out={check.stdout}")
        if "age5_gate0_runtime_family_transport_contract_checks_text=family_contract" not in check.stdout:
            return fail(f"check output missing gate0 runtime family transport checks_text: out={check.stdout}")
        if "age5_gate0_transport_family_contract_checks_text=lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family" not in check.stdout:
            return fail(f"check output missing gate0 transport family checks_text: out={check.stdout}")
        if "age5_bogae_alias_family_contract_checks_text=shape_alias_contract,alias_family,alias_viewer_family" not in check.stdout:
            return fail(f"check output missing bogae alias family checks_text: out={check.stdout}")
        if "age5_bogae_alias_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in check.stdout:
            return fail(f"check output missing bogae alias family transport checks_text: out={check.stdout}")

        broken_line = final_status_line.read_text(encoding="utf-8").strip().replace(
            " age5_full_real_w107_golden_index_selftest_active_cases=54",
            "",
        )
        final_status_line.write_text(broken_line + "\n", encoding="utf-8")
        broken = run_cmd(
            [
                sys.executable,
                "tests/run_ci_gate_final_status_line_check.py",
                "--status-line",
                str(final_status_line),
                "--aggregate-status-parse",
                str(aggregate_status_parse),
                "--gate-index",
                str(gate_index),
            ]
        )
        if broken.returncode == 0:
            return fail("broken age5 w107 token case must fail")

        render = run_cmd(
            [
                sys.executable,
                "tools/scripts/render_ci_gate_final_status_line.py",
                "--aggregate-status-parse",
                str(aggregate_status_parse),
                "--gate-index",
                str(gate_index),
                "--out",
                str(final_status_line),
                "--fail-on-bad",
            ]
        )
        if render.returncode != 0:
            return fail(f"rerender-age1 failed: out={render.stdout} err={render.stderr}")
        broken_line = final_status_line.read_text(encoding="utf-8").strip().replace(
            " age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks=5",
            "",
        )
        final_status_line.write_text(broken_line + "\n", encoding="utf-8")
        broken_age1 = run_cmd(
            [
                sys.executable,
                "tests/run_ci_gate_final_status_line_check.py",
                "--status-line",
                str(final_status_line),
                "--aggregate-status-parse",
                str(aggregate_status_parse),
                "--gate-index",
                str(gate_index),
            ]
        )
        if broken_age1.returncode == 0:
            return fail("broken age1 immediate proof operation contract token case must fail")

    print("[ci-gate-final-status-line-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
