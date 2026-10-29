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

AGE5_POLICY_FIXTURE = {
    "age5_policy_age4_proof_snapshot_fields_text": "age4_proof_ok=0|age4_proof_failed_criteria=-1|age4_proof_failed_preview=-",
    "age5_policy_age4_proof_snapshot_text": "age4_proof_ok=0|age4_proof_failed_criteria=-1|age4_proof_failed_preview=-",
    "age5_policy_age4_proof_source_snapshot_fields_text": "source-default",
    "age5_policy_age4_proof_gate_result_present": "0",
    "age5_policy_age4_proof_gate_result_parity": "0",
    "age5_policy_age4_proof_final_status_parse_present": "0",
    "age5_policy_age4_proof_final_status_parse_parity": "0",
}
AGE5_W107_PROGRESS_FIXTURE = {
    "age5_full_real_w107_golden_index_selftest_active_cases": "54",
    "age5_full_real_w107_golden_index_selftest_inactive_cases": "1",
    "age5_full_real_w107_golden_index_selftest_index_codes": "34",
    "age5_full_real_w107_golden_index_selftest_current_probe": "-",
    "age5_full_real_w107_golden_index_selftest_last_completed_probe": "validate_pack_pointers",
    "age5_full_real_w107_golden_index_selftest_progress_present": "1",
}
AGE5_W107_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_w107_progress_contract_selftest_completed_checks": "8",
    "age5_full_real_w107_progress_contract_selftest_total_checks": "8",
    "age5_full_real_w107_progress_contract_selftest_checks_text": "golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index",
    "age5_full_real_w107_progress_contract_selftest_current_probe": "-",
    "age5_full_real_w107_progress_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_w107_progress_contract_selftest_progress_present": "1",
}
AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks": "5",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks": "5",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text": "operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe": "-",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe": "proof_operation_family",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present": "1",
}
AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks": "5",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks": "5",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text": "signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe": "-",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe": "signed_contract",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present": "1",
}
AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks": "1",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks": "1",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text": "verify_report_digest_contract",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe": "-",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe": "readme_and_field_contract",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present": "1",
}
AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks": "4",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks": "4",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text": "signed_contract,consumer_contract,promotion,family",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe": "-",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe": "family",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present": "1",
}
AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_proof_certificate_family_contract_selftest_completed_checks": "3",
    "age5_full_real_proof_certificate_family_contract_selftest_total_checks": "3",
    "age5_full_real_proof_certificate_family_contract_selftest_checks_text": "artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family",
    "age5_full_real_proof_certificate_family_contract_selftest_current_probe": "-",
    "age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe": "proof_certificate_family",
    "age5_full_real_proof_certificate_family_contract_selftest_progress_present": "1",
}
AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks": "9",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks": "9",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present": "1",
}
AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_proof_family_contract_selftest_completed_checks": "3",
    "age5_full_real_proof_family_contract_selftest_total_checks": "3",
    "age5_full_real_proof_family_contract_selftest_checks_text": "proof_operation_family,proof_certificate_family,proof_family",
    "age5_full_real_proof_family_contract_selftest_current_probe": "-",
    "age5_full_real_proof_family_contract_selftest_last_completed_probe": "proof_family",
    "age5_full_real_proof_family_contract_selftest_progress_present": "1",
}
AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_proof_family_transport_contract_selftest_completed_checks": "9",
    "age5_full_real_proof_family_transport_contract_selftest_total_checks": "9",
    "age5_full_real_proof_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
    "age5_full_real_proof_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_proof_family_transport_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_proof_family_transport_contract_selftest_progress_present": "1",
}
AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_lang_surface_family_contract_selftest_completed_checks": "4",
    "age5_full_real_lang_surface_family_contract_selftest_total_checks": "4",
    "age5_full_real_lang_surface_family_contract_selftest_checks_text": "proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family",
    "age5_full_real_lang_surface_family_contract_selftest_current_probe": "-",
    "age5_full_real_lang_surface_family_contract_selftest_last_completed_probe": "lang_surface_family",
    "age5_full_real_lang_surface_family_contract_selftest_progress_present": "1",
}
AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_lang_runtime_family_contract_selftest_completed_checks": "5",
    "age5_full_real_lang_runtime_family_contract_selftest_total_checks": "5",
    "age5_full_real_lang_runtime_family_contract_selftest_checks_text": "lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family",
    "age5_full_real_lang_runtime_family_contract_selftest_current_probe": "-",
    "age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe": "lang_runtime_family",
    "age5_full_real_lang_runtime_family_contract_selftest_progress_present": "1",
}
AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_gate0_family_contract_selftest_completed_checks": "5",
    "age5_full_real_gate0_family_contract_selftest_total_checks": "5",
    "age5_full_real_gate0_family_contract_selftest_checks_text": "gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family",
    "age5_full_real_gate0_family_contract_selftest_current_probe": "-",
    "age5_full_real_gate0_family_contract_selftest_last_completed_probe": "gate0_family",
    "age5_full_real_gate0_family_contract_selftest_progress_present": "1",
}
AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_gate0_family_transport_contract_selftest_completed_checks": "9",
    "age5_full_real_gate0_family_transport_contract_selftest_total_checks": "9",
    "age5_full_real_gate0_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
    "age5_full_real_gate0_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_gate0_family_transport_contract_selftest_progress_present": "1",
}
AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_gate0_transport_family_contract_selftest_completed_checks": "4",
    "age5_full_real_gate0_transport_family_contract_selftest_total_checks": "4",
    "age5_full_real_gate0_transport_family_contract_selftest_checks_text": "lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family",
    "age5_full_real_gate0_transport_family_contract_selftest_current_probe": "-",
    "age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe": "gate0_transport_family",
    "age5_full_real_gate0_transport_family_contract_selftest_progress_present": "1",
}
AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks": "9",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks": "9",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present": "1",
}
AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks": "9",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks": "9",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present": "1",
}
AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks": "9",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks": "9",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present": "1",
}
AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks": "1",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks": "1",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text": "family_contract",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe": "family_contract",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present": "1",
}
AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks": "9",
    "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks": "9",
    "age5_full_real_lang_surface_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
    "age5_full_real_lang_surface_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present": "1",
}
AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_bogae_alias_family_contract_selftest_completed_checks": "3",
    "age5_full_real_bogae_alias_family_contract_selftest_total_checks": "3",
    "age5_full_real_bogae_alias_family_contract_selftest_checks_text": "shape_alias_contract,alias_family,alias_viewer_family",
    "age5_full_real_bogae_alias_family_contract_selftest_current_probe": "-",
    "age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe": "alias_viewer_family",
    "age5_full_real_bogae_alias_family_contract_selftest_progress_present": "1",
}
AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks": "9",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks": "9",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present": "1",
}


@contextmanager
def persistent_tmpdir(prefix: str):
    yield mkdtemp(prefix=prefix)


def fail(msg: str) -> int:
    print(f"[ci-gate-result-check-selftest] fail: {msg}")
    return 1


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(payload, ensure_ascii=False, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.rstrip() + "\n", encoding="utf-8")


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
    if len(cmd) >= 2 and cmd[0] == sys.executable and cmd[1] != "-S":
        cmd = [cmd[0], "-S", *cmd[1:]]
    if len(cmd) >= 2 and cmd[0] == sys.executable:
        script_index = 2 if len(cmd) >= 3 and cmd[1] == "-S" else 1
        if len(cmd) > script_index and str(cmd[script_index]).strip().endswith(".py"):
            return _run_python_inproc(cmd)
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def main() -> int:
    with persistent_tmpdir(prefix="ci_gate_result_check_selftest_") as tmp:
        root = Path(tmp)
        final_parse = root / "ci_gate_final_status_line_parse.detjson"
        summary_line = root / "ci_gate_summary_line.txt"
        result = root / "ci_gate_result.detjson"

        write_json(
            final_parse,
            {
                "schema": "ddn.ci.gate_final_status_line_parse.v1",
                "status_line_path": str(root / "ci_gate_final_status_line.txt"),
                "parsed": {
                    "status": "pass",
                    "overall_ok": "1",
                    "failed_steps": "0",
                    "aggregate_status": "pass",
                    "age4_proof_ok": "1",
                    "age4_proof_failed_criteria": "0",
                    "age4_proof_failed_preview": "-",
                    **AGE5_W107_PROGRESS_FIXTURE,
                    **AGE5_W107_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_POLICY_FIXTURE,
                    "reason": "ok",
                },
            },
        )
        write_text(
            summary_line,
            "ci_gate_result_status=pass ok=1 overall_ok=1 failed_steps=0 aggregate_status=pass "
            "age4_proof_ok=1 age4_proof_failed_criteria=0 "
            "age5_w107_active=54 age5_w107_inactive=1 age5_w107_index_codes=34 "
            "age5_w107_current_probe=- age5_w107_last_completed_probe=validate_pack_pointers "
            "age5_w107_progress=1 "
            "age5_w107_contract_completed=8 age5_w107_contract_total=8 "
            "age5_w107_contract_checks_text=golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index "
            "age5_w107_contract_current_probe=- age5_w107_contract_last_completed_probe=report_index "
            "age5_w107_contract_progress=1 "
            "age5_age1_immediate_proof_operation_contract_completed=5 age5_age1_immediate_proof_operation_contract_total=5 "
            "age5_age1_immediate_proof_operation_contract_checks_text=operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family "
            "age5_age1_immediate_proof_operation_contract_current_probe=- age5_age1_immediate_proof_operation_contract_last_completed_probe=proof_operation_family "
            "age5_age1_immediate_proof_operation_contract_progress=1 "
            "age5_proof_certificate_v1_consumer_contract_completed=5 age5_proof_certificate_v1_consumer_contract_total=5 "
            "age5_proof_certificate_v1_consumer_contract_checks_text=signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract "
            "age5_proof_certificate_v1_consumer_contract_current_probe=- age5_proof_certificate_v1_consumer_contract_last_completed_probe=signed_contract "
            "age5_proof_certificate_v1_verify_report_digest_contract_completed=1 age5_proof_certificate_v1_verify_report_digest_contract_total=1 "
            "age5_proof_certificate_v1_verify_report_digest_contract_checks_text=verify_report_digest_contract "
            "age5_proof_certificate_v1_verify_report_digest_contract_current_probe=- age5_proof_certificate_v1_verify_report_digest_contract_last_completed_probe=readme_and_field_contract "
            "age5_proof_certificate_v1_verify_report_digest_contract_progress=1 "
            "age5_proof_certificate_v1_family_contract_completed=4 age5_proof_certificate_v1_family_contract_total=4 "
            "age5_proof_certificate_v1_family_contract_checks_text=signed_contract,consumer_contract,promotion,family "
            "age5_proof_certificate_v1_family_contract_current_probe=- age5_proof_certificate_v1_family_contract_last_completed_probe=family "
            "age5_proof_certificate_v1_family_contract_progress=1 "
            "age5_proof_certificate_family_contract_completed=3 age5_proof_certificate_family_contract_total=3 "
            "age5_proof_certificate_family_contract_checks_text=artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family "
            "age5_proof_certificate_family_contract_current_probe=- age5_proof_certificate_family_contract_last_completed_probe=proof_certificate_family "
            "age5_proof_certificate_family_contract_progress=1 "
            "age5_proof_certificate_family_transport_contract_completed=9 age5_proof_certificate_family_transport_contract_total=9 "
            "age5_proof_certificate_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
            "age5_proof_certificate_family_transport_contract_current_probe=- age5_proof_certificate_family_transport_contract_last_completed_probe=report_index "
            "age5_proof_certificate_family_transport_contract_progress=1 "
            "age5_proof_family_contract_completed=3 age5_proof_family_contract_total=3 "
            "age5_proof_family_contract_checks_text=proof_operation_family,proof_certificate_family,proof_family "
            "age5_proof_family_contract_current_probe=- age5_proof_family_contract_last_completed_probe=proof_family "
            "age5_proof_family_contract_progress=1 "
            "age5_proof_family_transport_contract_completed=9 age5_proof_family_transport_contract_total=9 "
            "age5_proof_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
            "age5_proof_family_transport_contract_current_probe=- age5_proof_family_transport_contract_last_completed_probe=report_index "
            "age5_proof_family_transport_contract_progress=1 "
            "age5_lang_surface_family_contract_completed=4 age5_lang_surface_family_contract_total=4 "
            "age5_lang_surface_family_contract_checks_text=proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family "
            "age5_lang_surface_family_contract_current_probe=- age5_lang_surface_family_contract_last_completed_probe=lang_surface_family "
            "age5_lang_surface_family_contract_progress=1 "
            "age5_lang_runtime_family_contract_completed=5 age5_lang_runtime_family_contract_total=5 "
            "age5_lang_runtime_family_contract_checks_text=lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family "
            "age5_lang_runtime_family_contract_current_probe=- age5_lang_runtime_family_contract_last_completed_probe=lang_runtime_family "
            "age5_lang_runtime_family_contract_progress=1 "
            "age5_gate0_family_contract_completed=5 age5_gate0_family_contract_total=5 "
            "age5_gate0_family_contract_checks_text=gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family "
            "age5_gate0_family_contract_current_probe=- age5_gate0_family_contract_last_completed_probe=gate0_family "
            "age5_gate0_family_contract_progress=1 "
            "age5_gate0_family_transport_contract_completed=9 age5_gate0_family_transport_contract_total=9 "
            "age5_gate0_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
            "age5_gate0_family_transport_contract_current_probe=- age5_gate0_family_transport_contract_last_completed_probe=report_index "
            "age5_gate0_family_transport_contract_progress=1 "
            "age5_gate0_transport_family_contract_completed=4 age5_gate0_transport_family_contract_total=4 "
            "age5_gate0_transport_family_contract_checks_text=lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family "
            "age5_gate0_transport_family_contract_current_probe=- age5_gate0_transport_family_contract_last_completed_probe=gate0_transport_family "
            "age5_gate0_transport_family_contract_progress=1 "
            "age5_gate0_transport_family_transport_contract_completed=9 age5_gate0_transport_family_transport_contract_total=9 "
            "age5_gate0_transport_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
            "age5_gate0_transport_family_transport_contract_current_probe=- age5_gate0_transport_family_transport_contract_last_completed_probe=report_index "
            "age5_gate0_transport_family_transport_contract_progress=1 "
            "age5_gate0_surface_family_transport_contract_completed=9 age5_gate0_surface_family_transport_contract_total=9 "
            "age5_gate0_surface_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
            "age5_gate0_surface_family_transport_contract_current_probe=- age5_gate0_surface_family_transport_contract_last_completed_probe=report_index "
            "age5_gate0_surface_family_transport_contract_progress=1 "
            "age5_lang_runtime_family_transport_contract_completed=9 age5_lang_runtime_family_transport_contract_total=9 "
            "age5_lang_runtime_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
            "age5_lang_runtime_family_transport_contract_current_probe=- age5_lang_runtime_family_transport_contract_last_completed_probe=report_index "
            "age5_lang_runtime_family_transport_contract_progress=1 "
            "age5_gate0_runtime_family_transport_contract_completed=1 age5_gate0_runtime_family_transport_contract_total=1 "
            "age5_gate0_runtime_family_transport_contract_checks_text=family_contract "
            "age5_gate0_runtime_family_transport_contract_current_probe=- age5_gate0_runtime_family_transport_contract_last_completed_probe=family_contract "
            "age5_gate0_runtime_family_transport_contract_progress=1 "
            "age5_lang_surface_family_transport_contract_completed=9 age5_lang_surface_family_transport_contract_total=9 "
            "age5_lang_surface_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
            "age5_lang_surface_family_transport_contract_current_probe=- age5_lang_surface_family_transport_contract_last_completed_probe=report_index "
            "age5_lang_surface_family_transport_contract_progress=1 "
            "age5_bogae_alias_family_contract_completed=3 age5_bogae_alias_family_contract_total=3 "
            "age5_bogae_alias_family_contract_checks_text=shape_alias_contract,alias_family,alias_viewer_family "
            "age5_bogae_alias_family_contract_current_probe=- age5_bogae_alias_family_contract_last_completed_probe=alias_viewer_family "
            "age5_bogae_alias_family_contract_progress=1 "
            "age5_bogae_alias_family_transport_contract_completed=9 age5_bogae_alias_family_transport_contract_total=9 "
            "age5_bogae_alias_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
            "age5_bogae_alias_family_transport_contract_current_probe=- age5_bogae_alias_family_transport_contract_last_completed_probe=report_index "
            "age5_bogae_alias_family_transport_contract_progress=1 "
            "age5_proof_certificate_v1_consumer_contract_progress=1 reason=ok",
        )
        write_json(
            result,
            {
                "schema": "ddn.ci.gate_result.v1",
                "ok": True,
                "status": "pass",
                "overall_ok": True,
                "failed_steps": 0,
                "aggregate_status": "pass",
                "age4_proof_ok": True,
                "age4_proof_failed_criteria": 0,
                "age4_proof_failed_preview": "-",
                **AGE5_W107_PROGRESS_FIXTURE,
                **AGE5_W107_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                    **AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_POLICY_FIXTURE,
                "reason": "ok",
                "summary_line_path": str(summary_line),
                "summary_line": summary_line.read_text(encoding="utf-8").strip(),
                "final_status_parse_path": str(final_parse),
                "gate_index_path": str(root / "ci_gate_report_index.detjson"),
            },
        )

        ok_proc = run_cmd(
            [
                sys.executable,
                "tests/run_ci_gate_result_check.py",
                "--result",
                str(result),
                "--final-status-parse",
                str(final_parse),
                "--summary-line",
                str(summary_line),
                "--require-pass",
            ]
        )
        if ok_proc.returncode != 0:
            return fail(f"pass case failed: out={ok_proc.stdout} err={ok_proc.stderr}")
        if "age4_proof_ok=1" not in ok_proc.stdout:
            return fail(f"pass case output missing age4_proof_ok: out={ok_proc.stdout}")
        if "age4_proof_failed_preview=-" not in ok_proc.stdout:
            return fail(f"pass case output missing age4_proof_failed_preview: out={ok_proc.stdout}")
        if "age5_w107_active=54" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_w107_active: out={ok_proc.stdout}")
        if "age5_w107_progress=1" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_w107_progress: out={ok_proc.stdout}")
        if "age5_w107_contract_completed=8" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_w107_contract_completed: out={ok_proc.stdout}")
        if "age5_w107_contract_checks_text=golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_w107_contract_checks_text: out={ok_proc.stdout}")
        if "age5_w107_contract_progress=1" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_w107_contract_progress: out={ok_proc.stdout}")
        if "age5_age1_immediate_proof_operation_contract_completed=5" not in ok_proc.stdout:
            return fail(
                f"pass case output missing age5_age1_immediate_proof_operation_contract_completed: out={ok_proc.stdout}"
            )
        if "age5_age1_immediate_proof_operation_contract_checks_text=operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family" not in ok_proc.stdout:
            return fail(
                f"pass case output missing age5_age1_immediate_proof_operation_contract_checks_text: out={ok_proc.stdout}"
            )
        if "age5_age1_immediate_proof_operation_contract_progress=1" not in ok_proc.stdout:
            return fail(
                f"pass case output missing age5_age1_immediate_proof_operation_contract_progress: out={ok_proc.stdout}"
            )
        if "age5_proof_certificate_v1_consumer_contract_completed=5" not in ok_proc.stdout:
            return fail(
                f"pass case output missing age5_proof_certificate_v1_consumer_contract_completed: out={ok_proc.stdout}"
            )
        if "age5_proof_certificate_v1_consumer_contract_checks_text=signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract" not in ok_proc.stdout:
            return fail(
                f"pass case output missing age5_proof_certificate_v1_consumer_contract_checks_text: out={ok_proc.stdout}"
            )
        if "age5_proof_certificate_v1_consumer_contract_progress=1" not in ok_proc.stdout:
            return fail(
                f"pass case output missing age5_proof_certificate_v1_consumer_contract_progress: out={ok_proc.stdout}"
            )
        if "age5_proof_certificate_v1_verify_report_digest_contract_checks_text=verify_report_digest_contract" not in ok_proc.stdout:
            return fail(
                f"pass case output missing age5_proof_certificate_v1_verify_report_digest_contract_checks_text: out={ok_proc.stdout}"
            )
        if "age5_proof_certificate_v1_family_contract_checks_text=signed_contract,consumer_contract,promotion,family" not in ok_proc.stdout:
            return fail(
                f"pass case output missing age5_proof_certificate_v1_family_contract_checks_text: out={ok_proc.stdout}"
            )
        if "age5_proof_certificate_family_contract_checks_text=artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family" not in ok_proc.stdout:
            return fail(
                f"pass case output missing age5_proof_certificate_family_contract_checks_text: out={ok_proc.stdout}"
            )
        if "age5_proof_certificate_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in ok_proc.stdout:
            return fail(
                f"pass case output missing age5_proof_certificate_family_transport_contract_checks_text: out={ok_proc.stdout}"
            )
        if "age5_proof_family_contract_checks_text=proof_operation_family,proof_certificate_family,proof_family" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_proof_family_contract_checks_text: out={ok_proc.stdout}")
        if "age5_proof_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_proof_family_transport_contract_checks_text: out={ok_proc.stdout}")
        if "age5_lang_surface_family_contract_checks_text=proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_lang_surface_family_contract_checks_text: out={ok_proc.stdout}")
        if "age5_lang_runtime_family_contract_checks_text=lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_lang_runtime_family_contract_checks_text: out={ok_proc.stdout}")
        if "age5_gate0_family_contract_checks_text=gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_gate0_family_contract_checks_text: out={ok_proc.stdout}")
        if "age5_gate0_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_gate0_family_transport_contract_checks_text: out={ok_proc.stdout}")
        if "age5_gate0_transport_family_contract_checks_text=lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_gate0_transport_family_contract_checks_text: out={ok_proc.stdout}")
        if "age5_gate0_transport_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_gate0_transport_family_transport_contract_checks_text: out={ok_proc.stdout}")
        if "age5_gate0_surface_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_gate0_surface_family_transport_contract_checks_text: out={ok_proc.stdout}")
        if "age5_lang_runtime_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_lang_runtime_family_transport_contract_checks_text: out={ok_proc.stdout}")
        if "age5_gate0_runtime_family_transport_contract_checks_text=family_contract" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_gate0_runtime_family_transport_contract_checks_text: out={ok_proc.stdout}")
        if "age5_lang_surface_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in ok_proc.stdout:
            return fail(f"pass case output missing age5_lang_surface_family_transport_contract_checks_text: out={ok_proc.stdout}")
        if "age5_bogae_alias_family_contract_checks_text=shape_alias_contract,alias_family,alias_viewer_family" not in ok_proc.stdout:
            return fail(
                f"pass case output missing age5_bogae_alias_family_contract_checks_text: out={ok_proc.stdout}"
            )
        if "age5_bogae_alias_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in ok_proc.stdout:
            return fail(
                f"pass case output missing age5_bogae_alias_family_transport_contract_checks_text: out={ok_proc.stdout}"
            )
        if str(json.loads(result.read_text(encoding="utf-8")).get("age5_policy_age4_proof_gate_result_present", "")).strip() != "0":
            return fail("pass case result missing age5 policy gate_result parity")

        broken = json.loads(result.read_text(encoding="utf-8"))
        broken["age4_proof_failed_preview"] = "proof_runtime_error_statehash_preserved"
        write_json(result, broken)
        bad_proc = run_cmd(
            [
                sys.executable,
                "tests/run_ci_gate_result_check.py",
                "--result",
                str(result),
                "--final-status-parse",
                str(final_parse),
                "--summary-line",
                str(summary_line),
            ]
        )
        if bad_proc.returncode == 0:
            return fail("broken age4_proof_failed_preview case must fail")

        restored = {
            "schema": "ddn.ci.gate_result.v1",
            "ok": True,
            "status": "pass",
            "overall_ok": True,
            "failed_steps": 0,
            "aggregate_status": "pass",
            "age4_proof_ok": True,
            "age4_proof_failed_criteria": 0,
            "age4_proof_failed_preview": "-",
            **AGE5_W107_PROGRESS_FIXTURE,
            **AGE5_W107_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_POLICY_FIXTURE,
            "reason": "ok",
            "summary_line_path": str(summary_line),
            "summary_line": summary_line.read_text(encoding="utf-8").strip(),
            "final_status_parse_path": str(final_parse),
            "gate_index_path": str(root / "ci_gate_report_index.detjson"),
        }
        restored["age5_policy_age4_proof_gate_result_parity"] = "1"
        write_json(result, restored)
        bad_policy_proc = run_cmd(
            [
                sys.executable,
                "tests/run_ci_gate_result_check.py",
                "--result",
                str(result),
                "--final-status-parse",
                str(final_parse),
                "--summary-line",
                str(summary_line),
            ]
        )
        if bad_policy_proc.returncode == 0:
            return fail("broken age5 policy parity case must fail")

        restored = {
            "schema": "ddn.ci.gate_result.v1",
            "ok": True,
            "status": "pass",
            "overall_ok": True,
            "failed_steps": 0,
            "aggregate_status": "pass",
            "age4_proof_ok": True,
            "age4_proof_failed_criteria": 0,
            "age4_proof_failed_preview": "-",
            **AGE5_W107_PROGRESS_FIXTURE,
            **AGE5_W107_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_POLICY_FIXTURE,
            "reason": "ok",
            "summary_line_path": str(summary_line),
            "summary_line": summary_line.read_text(encoding="utf-8").strip(),
            "final_status_parse_path": str(final_parse),
            "gate_index_path": str(root / "ci_gate_report_index.detjson"),
        }
        restored["age5_full_real_w107_golden_index_selftest_active_cases"] = "999"
        write_json(result, restored)
        bad_w107_proc = run_cmd(
            [
                sys.executable,
                "tests/run_ci_gate_result_check.py",
                "--result",
                str(result),
                "--final-status-parse",
                str(final_parse),
                "--summary-line",
                str(summary_line),
            ]
        )
        if bad_w107_proc.returncode == 0:
            return fail("broken w107 active_cases case must fail")

        restored = {
            "schema": "ddn.ci.gate_result.v1",
            "ok": True,
            "status": "pass",
            "overall_ok": True,
            "failed_steps": 0,
            "aggregate_status": "pass",
            "age4_proof_ok": True,
            "age4_proof_failed_criteria": 0,
            "age4_proof_failed_preview": "-",
            **AGE5_W107_PROGRESS_FIXTURE,
            **AGE5_W107_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_POLICY_FIXTURE,
            "reason": "ok",
            "summary_line_path": str(summary_line),
            "summary_line": summary_line.read_text(encoding="utf-8").strip(),
            "final_status_parse_path": str(final_parse),
            "gate_index_path": str(root / "ci_gate_report_index.detjson"),
        }
        restored["age5_full_real_w107_progress_contract_selftest_completed_checks"] = "999"
        write_json(result, restored)
        bad_w107_contract_proc = run_cmd(
            [
                sys.executable,
                "tests/run_ci_gate_result_check.py",
                "--result",
                str(result),
                "--final-status-parse",
                str(final_parse),
                "--summary-line",
                str(summary_line),
            ]
        )
        if bad_w107_contract_proc.returncode == 0:
            return fail("broken w107 contract completed_checks case must fail")

        restored = {
            "schema": "ddn.ci.gate_result.v1",
            "ok": True,
            "status": "pass",
            "overall_ok": True,
            "failed_steps": 0,
            "aggregate_status": "pass",
            "age4_proof_ok": True,
            "age4_proof_failed_criteria": 0,
            "age4_proof_failed_preview": "-",
            **AGE5_W107_PROGRESS_FIXTURE,
            **AGE5_W107_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_POLICY_FIXTURE,
            "reason": "ok",
            "summary_line_path": str(summary_line),
            "summary_line": summary_line.read_text(encoding="utf-8").strip(),
            "final_status_parse_path": str(final_parse),
            "gate_index_path": str(root / "ci_gate_report_index.detjson"),
        }
        restored["age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks"] = "999"
        write_json(result, restored)
        bad_age1_contract_proc = run_cmd(
            [
                sys.executable,
                "tests/run_ci_gate_result_check.py",
                "--result",
                str(result),
                "--final-status-parse",
                str(final_parse),
                "--summary-line",
                str(summary_line),
            ]
        )
        if bad_age1_contract_proc.returncode == 0:
            return fail("broken age1 immediate proof operation contract completed_checks case must fail")

        restored = {
            "schema": "ddn.ci.gate_result.v1",
            "ok": True,
            "status": "pass",
            "overall_ok": True,
            "failed_steps": 0,
            "aggregate_status": "pass",
            "age4_proof_ok": True,
            "age4_proof_failed_criteria": 0,
            "age4_proof_failed_preview": "-",
            **AGE5_W107_PROGRESS_FIXTURE,
            **AGE5_W107_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_POLICY_FIXTURE,
            "reason": "ok",
            "summary_line_path": str(summary_line),
            "summary_line": summary_line.read_text(encoding="utf-8").strip(),
            "final_status_parse_path": str(final_parse),
            "gate_index_path": str(root / "ci_gate_report_index.detjson"),
        }
        restored["age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks"] = "999"
        write_json(result, restored)
        bad_consumer_contract_proc = run_cmd(
            [
                sys.executable,
                "tests/run_ci_gate_result_check.py",
                "--result",
                str(result),
                "--final-status-parse",
                str(final_parse),
                "--summary-line",
                str(summary_line),
            ]
        )
        if bad_consumer_contract_proc.returncode == 0:
            return fail("broken proof_certificate_v1 consumer contract completed_checks case must fail")

    print("[ci-gate-result-check-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
