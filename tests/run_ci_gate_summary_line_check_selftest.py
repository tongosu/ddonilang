#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[ci-gate-summary-line-check-selftest] fail: {msg}")
    return 1


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.rstrip() + "\n", encoding="utf-8")


def run_cmd(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="ci_gate_summary_line_check_selftest_") as tmp:
        root = Path(tmp)
        summary_line = root / "ci_gate_summary_line.txt"
        result_parse = root / "ci_gate_result_parse.detjson"
        compact = (
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
            "age5_lang_runtime_family_transport_contract_completed=9 age5_lang_runtime_family_transport_contract_total=9 "
            "age5_lang_runtime_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
            "age5_lang_runtime_family_transport_contract_current_probe=- age5_lang_runtime_family_transport_contract_last_completed_probe=report_index "
            "age5_lang_runtime_family_transport_contract_progress=1 "
            "age5_gate0_runtime_family_transport_contract_completed=1 age5_gate0_runtime_family_transport_contract_total=1 "
            "age5_gate0_runtime_family_transport_contract_checks_text=family_contract "
            "age5_gate0_runtime_family_transport_contract_current_probe=- age5_gate0_runtime_family_transport_contract_last_completed_probe=family_contract "
            "age5_gate0_runtime_family_transport_contract_progress=1 "
            "age5_gate0_transport_family_contract_completed=4 age5_gate0_transport_family_contract_total=4 "
            "age5_gate0_transport_family_contract_checks_text=lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family "
            "age5_gate0_transport_family_contract_current_probe=- age5_gate0_transport_family_contract_last_completed_probe=gate0_transport_family "
            "age5_gate0_transport_family_contract_progress=1 "
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
            "age5_proof_certificate_v1_consumer_contract_progress=1 reason=ok"
        )
        write_text(summary_line, compact)
        write_json(
            result_parse,
            {
                "schema": "ddn.ci.gate_result_parse.v1",
                "result_path": str(root / "ci_gate_result.detjson"),
                "parsed": {
                    "status": "pass",
                    "ok": True,
                    "overall_ok": True,
                    "failed_steps": 0,
                    "aggregate_status": "pass",
                },
                "compact_line": compact,
            },
        )

        ok_proc = run_cmd(
            [
                sys.executable,
                "tests/run_ci_gate_summary_line_check.py",
                "--summary-line",
                str(summary_line),
                "--ci-gate-result-parse",
                str(result_parse),
                "--require-pass",
            ]
        )
        if ok_proc.returncode != 0:
            return fail(f"pass case failed: out={ok_proc.stdout} err={ok_proc.stderr}")
        if "age5_w107_contract_checks_text=golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index" not in ok_proc.stdout:
            return fail(f"pass case output missing checks_text token: out={ok_proc.stdout}")
        if "age5_age1_immediate_proof_operation_contract_checks_text=operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family" not in ok_proc.stdout:
            return fail(f"pass case output missing age1 checks_text token: out={ok_proc.stdout}")
        if "age5_proof_certificate_v1_consumer_contract_checks_text=signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract" not in ok_proc.stdout:
            return fail(f"pass case output missing proof_certificate consumer checks_text token: out={ok_proc.stdout}")
        if "age5_proof_certificate_v1_verify_report_digest_contract_checks_text=verify_report_digest_contract" not in ok_proc.stdout:
            return fail(f"pass case output missing proof_certificate digest checks_text token: out={ok_proc.stdout}")
        if "age5_proof_certificate_v1_family_contract_checks_text=signed_contract,consumer_contract,promotion,family" not in ok_proc.stdout:
            return fail(f"pass case output missing proof_certificate family checks_text token: out={ok_proc.stdout}")
        if "age5_proof_certificate_family_contract_checks_text=artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family" not in ok_proc.stdout:
            return fail(f"pass case output missing top-level proof_certificate family checks_text token: out={ok_proc.stdout}")
        if "age5_proof_certificate_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in ok_proc.stdout:
            return fail(f"pass case output missing top-level proof_certificate family transport checks_text token: out={ok_proc.stdout}")
        if "age5_proof_family_contract_checks_text=proof_operation_family,proof_certificate_family,proof_family" not in ok_proc.stdout:
            return fail(f"pass case output missing proof_family checks_text token: out={ok_proc.stdout}")
        if "age5_proof_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in ok_proc.stdout:
            return fail(f"pass case output missing proof_family transport checks_text token: out={ok_proc.stdout}")
        if "age5_lang_surface_family_contract_checks_text=proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family" not in ok_proc.stdout:
            return fail(f"pass case output missing lang_surface_family checks_text token: out={ok_proc.stdout}")
        if "age5_lang_runtime_family_contract_checks_text=lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family" not in ok_proc.stdout:
            return fail(f"pass case output missing lang_runtime_family checks_text token: out={ok_proc.stdout}")
        if "age5_gate0_family_contract_checks_text=gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family" not in ok_proc.stdout:
            return fail(f"pass case output missing gate0_family checks_text token: out={ok_proc.stdout}")
        if "age5_lang_runtime_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in ok_proc.stdout:
            return fail(f"pass case output missing lang_runtime_family transport checks_text token: out={ok_proc.stdout}")
        if "age5_gate0_runtime_family_transport_contract_checks_text=family_contract" not in ok_proc.stdout:
            return fail(f"pass case output missing gate0_runtime_family transport checks_text token: out={ok_proc.stdout}")
        if "age5_gate0_transport_family_contract_checks_text=lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family" not in ok_proc.stdout:
            return fail(f"pass case output missing gate0_transport_family transport checks_text token: out={ok_proc.stdout}")
        if "age5_lang_surface_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in ok_proc.stdout:
            return fail(f"pass case output missing lang_surface_family transport checks_text token: out={ok_proc.stdout}")
        if "age5_bogae_alias_family_contract_checks_text=shape_alias_contract,alias_family,alias_viewer_family" not in ok_proc.stdout:
            return fail(f"pass case output missing bogae alias family checks_text token: out={ok_proc.stdout}")
        if "age5_bogae_alias_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index" not in ok_proc.stdout:
            return fail(f"pass case output missing bogae alias family transport checks_text token: out={ok_proc.stdout}")

        broken = compact.replace(
            " age5_w107_contract_checks_text=golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index",
            "",
        )
        write_text(summary_line, broken)
        bad_proc = run_cmd(
            [
                sys.executable,
                "tests/run_ci_gate_summary_line_check.py",
                "--summary-line",
                str(summary_line),
                "--ci-gate-result-parse",
                str(result_parse),
            ]
        )
        if bad_proc.returncode == 0:
            return fail("broken checks_text summary line case must fail")

    print("[ci-gate-summary-line-check-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
