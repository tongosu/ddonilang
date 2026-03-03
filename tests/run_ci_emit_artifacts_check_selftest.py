#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from ci_check_error_codes import EMIT_ARTIFACTS_CODES as CODES

def fail(msg: str) -> int:
    print(f"[ci-emit-artifacts-check-selftest] fail: {msg}")
    return 1


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.rstrip() + "\n", encoding="utf-8")


def run_check(report_dir: Path, *extra: str) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_ci_emit_artifacts_check.py",
        "--report-dir",
        str(report_dir),
        *extra,
    ]
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def build_case(
    report_dir: Path,
    prefix: str,
    status: str,
    with_brief: bool,
    with_triage: bool,
    with_sanity: bool = True,
    with_sync_readiness: bool = True,
    broken_norm: bool = False,
    broken_brief: bool = False,
    broken_triage_final: bool = False,
    broken_artifact_ref: bool = False,
    broken_summary: bool = False,
    broken_verify_issue: bool = False,
    broken_sanity_schema: bool = False,
    broken_sanity_status: bool = False,
    broken_sanity_required_step_missing: bool = False,
    broken_sanity_required_step_failed: bool = False,
    broken_sanity_wired_step_missing: bool = False,
    broken_sanity_wired_step_failed: bool = False,
    broken_sanity_compare_step_missing: bool = False,
    broken_sanity_compare_step_failed: bool = False,
    broken_summary_selftest_missing: bool = False,
    broken_summary_selftest_value: bool = False,
    broken_summary_selftest_step_mismatch: bool = False,
    broken_sync_readiness_schema: bool = False,
    broken_sync_readiness_status_unsupported: bool = False,
    broken_sync_readiness_status_mismatch: bool = False,
    broken_sync_readiness_pass_fields: bool = False,
) -> None:
    index_path = report_dir / f"{prefix}.ci_gate_report_index.detjson"
    result_path = report_dir / f"{prefix}.ci_gate_result.detjson"
    summary_path = report_dir / f"{prefix}.ci_gate_summary.txt"
    summary_line_path = report_dir / f"{prefix}.ci_gate_summary_line.txt"
    brief_path = report_dir / f"{prefix}.ci_fail_brief.txt"
    triage_path = report_dir / f"{prefix}.ci_fail_triage.detjson"
    sanity_path = report_dir / f"{prefix}.ci_sanity_gate.detjson"
    sync_readiness_path = report_dir / f"{prefix}.ci_sync_readiness.detjson"

    failed_steps_count = 0 if status == "pass" else 1
    sample_step_id = "sample_fail"
    sample_stdout_path = report_dir / f"{prefix}.ci_gate_step_{sample_step_id}.stdout.txt"
    sample_stderr_path = report_dir / f"{prefix}.ci_gate_step_{sample_step_id}.stderr.txt"
    if failed_steps_count > 0:
        write_text(sample_stdout_path, "[sample-fail] stdout")
        write_text(sample_stderr_path, "[sample-fail] stderr")
    if status == "pass" or broken_summary:
        summary_lines = [
            "[ci-gate-summary] PASS",
            "[ci-gate-summary] failed_steps=(none)",
        ]
        if not broken_summary_selftest_missing:
            compare_ok = "0" if broken_summary_selftest_value else "1"
            session_ok = "1"
            summary_lines.append(f"[ci-gate-summary] ci_pack_golden_overlay_compare_selftest_ok={compare_ok}")
            summary_lines.append(f"[ci-gate-summary] ci_pack_golden_overlay_session_selftest_ok={session_ok}")
        write_text(summary_path, "\n".join(summary_lines))
    else:
        write_text(
            summary_path,
            "\n".join(
                [
                    "[ci-gate-summary] FAIL",
                    f"[ci-gate-summary] failed_steps={sample_step_id}",
                    f"[ci-gate-summary] failed_step_detail={sample_step_id} rc=1 cmd=python tests/run_sample_fail.py",
                    f"[ci-gate-summary] failed_step_logs={sample_step_id} stdout={sample_stdout_path} stderr={sample_stderr_path}",
                ]
            ),
        )
    summary_line = (
        f"ci_gate_result_status={status} ok={1 if status == 'pass' else 0} "
        f"overall_ok={1 if status == 'pass' else 0} failed_steps={failed_steps_count} "
        f"aggregate_status={status} reason=-"
    )
    write_text(summary_line_path, summary_line)
    write_json(
        result_path,
        {
            "schema": "ddn.ci.gate_result.v1",
            "status": status,
            "ok": status == "pass",
            "reason": "-",
            "failed_steps": failed_steps_count,
        },
    )
    if with_sanity:
        sanity_status = "pass" if status == "pass" else "fail"
        if broken_sanity_status:
            sanity_status = "fail" if sanity_status == "pass" else "pass"
        if sanity_status == "pass":
            sanity_code = "OK"
            sanity_step = "all"
            sanity_steps = [
                {"step": "backup_hygiene_selftest", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {"step": "pipeline_emit_flags_check", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {"step": "pipeline_emit_flags_selftest", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {"step": "seamgrim_ci_gate_seed_meta_step_check", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {
                    "step": "seamgrim_ci_gate_runtime5_passthrough_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_overlay_session_wired_consistency_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_overlay_session_diag_parity_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_overlay_compare_diag_parity_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "age5_close_pack_contract_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "ci_pack_golden_age5_surface_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "ci_pack_golden_guideblock_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "ci_pack_golden_exec_policy_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "ci_pack_golden_jjaim_flatten_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "ci_pack_golden_event_model_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "w92_aot_pack_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "w93_universe_pack_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "w94_social_pack_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "w95_cert_pack_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "w96_somssi_pack_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "w97_self_heal_pack_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_wasm_cli_diag_parity_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
            ]
            if broken_sanity_required_step_missing:
                sanity_steps = [row for row in sanity_steps if row.get("step") != "age5_close_pack_contract_selftest"]
            if broken_sanity_required_step_failed:
                for row in sanity_steps:
                    if row.get("step") == "seamgrim_overlay_session_diag_parity_check":
                        row["ok"] = True
                        row["returncode"] = 1
                        break
            if broken_sanity_wired_step_missing:
                sanity_steps = [row for row in sanity_steps if row.get("step") != "seamgrim_overlay_session_wired_consistency_check"]
            if broken_sanity_wired_step_failed:
                for row in sanity_steps:
                    if row.get("step") == "seamgrim_overlay_session_wired_consistency_check":
                        row["ok"] = True
                        row["returncode"] = 1
                        break
            if broken_sanity_compare_step_missing:
                sanity_steps = [row for row in sanity_steps if row.get("step") != "seamgrim_overlay_compare_diag_parity_check"]
            if broken_sanity_compare_step_failed:
                for row in sanity_steps:
                    if row.get("step") == "seamgrim_overlay_compare_diag_parity_check":
                        row["ok"] = True
                        row["returncode"] = 1
                        break
        else:
            sanity_code = "E_CI_SANITY_SAMPLE_FAIL"
            sanity_step = "pipeline_emit_flags_check"
            sanity_steps = [
                {"step": "backup_hygiene_selftest", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {"step": "pipeline_emit_flags_check", "ok": False, "returncode": 1, "cmd": ["python", "x.py"]},
            ]
        write_json(
            sanity_path,
            {
                "schema": "ddn.ci.sanity_gate.v1" if not broken_sanity_schema else "broken.schema",
                "generated_at_utc": "2026-03-02T00:00:00+00:00",
                "status": sanity_status,
                "code": sanity_code,
                "step": sanity_step,
                "msg": "-",
                "steps": sanity_steps,
            },
        )
    if with_brief:
        brief_status = "fail" if broken_brief else status
        brief_reason = "bad_reason" if broken_brief else "-"
        brief_failed_steps = 99 if broken_brief else failed_steps_count
        brief_final_line = "-" if broken_brief else summary_line
        write_text(
            brief_path,
            f'status={brief_status} reason="{brief_reason}" failed_steps_count={brief_failed_steps} failed_steps="-" top_step=- top_message="-" final_line="{brief_final_line}"',
        )
    if with_triage:
        triage_final_line = "-" if broken_triage_final else summary_line
        failed_steps_rows: list[dict[str, object]] = []
        if failed_steps_count > 0:
            failed_steps_rows.append(
                {
                    "step_id": sample_step_id,
                    "message": "sample failure",
                    "stdout_log_path": str(sample_stdout_path),
                    "stdout_log_path_norm": str(sample_stdout_path).replace("\\", "/"),
                    "stderr_log_path": str(sample_stderr_path),
                    "stderr_log_path_norm": str(sample_stderr_path).replace("\\", "/"),
                }
            )
        triage_summary_line_path = summary_line_path
        triage_summary_line_norm = str(summary_line_path).replace("\\", "/")
        if broken_artifact_ref:
            alt_summary_line_path = report_dir / f"{prefix}.alt_summary_line.txt"
            write_text(alt_summary_line_path, "ci_gate_result_status=pass ok=1 overall_ok=1 failed_steps=0")
            triage_summary_line_path = alt_summary_line_path
            triage_summary_line_norm = str(alt_summary_line_path).replace("\\", "/")
        write_json(
            triage_path,
            {
                "schema": "ddn.ci.fail_triage.v1",
                "generated_at_utc": "2026-02-21T00:00:00+00:00",
                "status": status,
                "reason": "-",
                "report_prefix": prefix,
                "final_line": triage_final_line,
                "summary_verify_ok": False if broken_verify_issue else True,
                "summary_verify_issues": ["bad_issue_token"] if broken_verify_issue else [],
                "summary_verify_issues_count": 1 if broken_verify_issue else 0,
                "summary_verify_top_issue": "bad_issue_token" if broken_verify_issue else "-",
                "failed_steps": failed_steps_rows,
                "failed_steps_count": failed_steps_count,
                "aggregate_digest": [],
                "aggregate_digest_count": 0,
                "summary_report_path_hint": str(summary_path),
                "summary_report_path_hint_norm": (
                    "BROKEN/PATH" if broken_norm else str(summary_path).replace("\\", "/")
                ),
                "artifacts": {
                    "summary": {
                        "path": str(summary_path),
                        "path_norm": "BROKEN/PATH" if broken_norm else str(summary_path).replace("\\", "/"),
                        "exists": True,
                    },
                    "summary_line": {
                        "path": str(triage_summary_line_path),
                        "path_norm": triage_summary_line_norm,
                        "exists": True,
                    },
                    "ci_gate_result_json": {
                        "path": str(result_path),
                        "path_norm": str(result_path).replace("\\", "/"),
                        "exists": True,
                    },
                    "ci_fail_brief_txt": {
                        "path": str(brief_path),
                        "path_norm": str(brief_path).replace("\\", "/"),
                        "exists": bool(with_brief),
                    },
                    "ci_fail_triage_json": {
                        "path": str(triage_path),
                        "path_norm": str(triage_path).replace("\\", "/"),
                        "exists": True,
                    },
                },
            },
        )

    if with_sync_readiness:
        sync_status = "pass" if status == "pass" else "fail"
        if broken_sync_readiness_status_unsupported:
            sync_status = "unknown"
        elif broken_sync_readiness_status_mismatch:
            sync_status = "fail" if sync_status == "pass" else "pass"
        sync_code = "OK" if sync_status == "pass" else "E_SYNC_READINESS_STEP_FAIL"
        sync_step = "all" if sync_status == "pass" else "aggregate_gate"
        if broken_sync_readiness_pass_fields and sync_status == "pass":
            sync_code = "BROKEN"
        write_json(
            sync_readiness_path,
            {
                "schema": "ddn.ci.sync_readiness.v1" if not broken_sync_readiness_schema else "broken.schema",
                "generated_at_utc": "2026-03-02T00:00:00+00:00",
                "status": sync_status,
                "ok": sync_status == "pass",
                "code": sync_code,
                "step": sync_step,
                "msg": "-",
                "steps": [],
                "steps_count": 0,
            },
        )

    write_json(
        index_path,
        {
            "schema": "ddn.ci.aggregate_gate.index.v1",
            "report_prefix": prefix,
            "reports": {
                "summary": str(summary_path),
                "summary_line": str(summary_line_path),
                "ci_gate_result_json": str(result_path),
                "ci_fail_brief_txt": str(brief_path),
                "ci_fail_triage_json": str(triage_path),
                **({"ci_sanity_gate": str(sanity_path)} if with_sanity else {}),
                **({"ci_sync_readiness": str(sync_readiness_path)} if with_sync_readiness else {}),
            },
            "steps": [
                {
                    "name": "ci_pack_golden_overlay_compare_selftest",
                    "returncode": 1 if broken_summary_selftest_step_mismatch else 0,
                    "ok": False if broken_summary_selftest_step_mismatch else True,
                },
                {
                    "name": "ci_pack_golden_overlay_session_selftest",
                    "returncode": 0,
                    "ok": True,
                },
            ],
        },
    )


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="ci_emit_artifacts_selftest_") as tmp:
        report_dir = Path(tmp)

        build_case(report_dir, "okcase", status="pass", with_brief=True, with_triage=True)
        proc_ok = run_check(report_dir, "--prefix", "okcase", "--require-brief", "--require-triage")
        if proc_ok.returncode != 0:
            return fail(f"okcase failed rc={proc_ok.returncode} out={proc_ok.stdout} err={proc_ok.stderr}")

        build_case(report_dir, "failcase", status="fail", with_brief=True, with_triage=True)
        proc_fail = run_check(report_dir, "--prefix", "failcase", "--require-brief", "--require-triage")
        if proc_fail.returncode != 0:
            return fail(f"failcase failed rc={proc_fail.returncode} out={proc_fail.stdout} err={proc_fail.stderr}")

        build_case(
            report_dir,
            "missselftestkey",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_summary_selftest_missing=True,
        )
        proc_miss_selftestkey = run_check(
            report_dir,
            "--prefix",
            "missselftestkey",
            "--require-brief",
            "--require-triage",
        )
        if proc_miss_selftestkey.returncode == 0:
            return fail("missselftestkey must fail")
        if f"fail code={CODES['SUMMARY_SELFTEST_KEY_MISSING']}" not in proc_miss_selftestkey.stderr:
            return fail(f"missselftestkey error code missing: err={proc_miss_selftestkey.stderr}")

        build_case(
            report_dir,
            "badselftestvalue",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_summary_selftest_value=True,
        )
        proc_bad_selftestvalue = run_check(
            report_dir,
            "--prefix",
            "badselftestvalue",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_selftestvalue.returncode == 0:
            return fail("badselftestvalue must fail")
        if f"fail code={CODES['SUMMARY_SELFTEST_EXPECT_PASS']}" not in proc_bad_selftestvalue.stderr:
            return fail(f"badselftestvalue error code missing: err={proc_bad_selftestvalue.stderr}")

        build_case(
            report_dir,
            "badselfteststep",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_summary_selftest_step_mismatch=True,
        )
        proc_bad_selfteststep = run_check(
            report_dir,
            "--prefix",
            "badselfteststep",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_selfteststep.returncode == 0:
            return fail("badselfteststep must fail")
        if f"fail code={CODES['SUMMARY_SELFTEST_STEP_MISMATCH']}" not in proc_bad_selfteststep.stderr:
            return fail(f"badselfteststep error code missing: err={proc_bad_selfteststep.stderr}")

        build_case(report_dir, "missbrief", status="pass", with_brief=False, with_triage=True)
        proc_miss_brief = run_check(report_dir, "--prefix", "missbrief", "--require-brief", "--require-triage")
        if proc_miss_brief.returncode == 0:
            return fail("missbrief must fail")
        if f"fail code={CODES['BRIEF_REQUIRED_MISSING']}" not in proc_miss_brief.stderr:
            return fail(f"missbrief error code missing: err={proc_miss_brief.stderr}")

        build_case(report_dir, "misstriage", status="pass", with_brief=True, with_triage=False)
        proc_miss_triage = run_check(report_dir, "--prefix", "misstriage", "--require-brief", "--require-triage")
        if proc_miss_triage.returncode == 0:
            return fail("misstriage must fail")
        if f"fail code={CODES['TRIAGE_REQUIRED_MISSING']}" not in proc_miss_triage.stderr:
            return fail(f"misstriage error code missing: err={proc_miss_triage.stderr}")

        build_case(report_dir, "misssanity", status="pass", with_brief=True, with_triage=True, with_sanity=False)
        proc_miss_sanity = run_check(report_dir, "--prefix", "misssanity", "--require-brief", "--require-triage")
        if proc_miss_sanity.returncode == 0:
            return fail("misssanity must fail")
        if f"fail code={CODES['SANITY_PATH_MISSING']}" not in proc_miss_sanity.stderr:
            return fail(f"misssanity error code missing: err={proc_miss_sanity.stderr}")

        build_case(
            report_dir,
            "misssync",
            status="pass",
            with_brief=True,
            with_triage=True,
            with_sync_readiness=False,
        )
        proc_miss_sync = run_check(report_dir, "--prefix", "misssync", "--require-brief", "--require-triage")
        if proc_miss_sync.returncode == 0:
            return fail("misssync must fail")
        if f"fail code={CODES['SYNC_READINESS_PATH_MISSING']}" not in proc_miss_sync.stderr:
            return fail(f"misssync error code missing: err={proc_miss_sync.stderr}")

        build_case(
            report_dir,
            "badsyncschema",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_sync_readiness_schema=True,
        )
        proc_bad_sync_schema = run_check(report_dir, "--prefix", "badsyncschema", "--require-brief", "--require-triage")
        if proc_bad_sync_schema.returncode == 0:
            return fail("badsyncschema must fail")
        if f"fail code={CODES['SYNC_READINESS_SCHEMA_MISMATCH']}" not in proc_bad_sync_schema.stderr:
            return fail(f"badsyncschema error code missing: err={proc_bad_sync_schema.stderr}")

        build_case(
            report_dir,
            "badsyncstatus",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_sync_readiness_status_unsupported=True,
        )
        proc_bad_sync_status = run_check(report_dir, "--prefix", "badsyncstatus", "--require-brief", "--require-triage")
        if proc_bad_sync_status.returncode == 0:
            return fail("badsyncstatus must fail")
        if f"fail code={CODES['SYNC_READINESS_STATUS_UNSUPPORTED']}" not in proc_bad_sync_status.stderr:
            return fail(f"badsyncstatus error code missing: err={proc_bad_sync_status.stderr}")

        build_case(
            report_dir,
            "badsyncmismatch",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_sync_readiness_status_mismatch=True,
        )
        proc_bad_sync_mismatch = run_check(
            report_dir,
            "--prefix",
            "badsyncmismatch",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_sync_mismatch.returncode == 0:
            return fail("badsyncmismatch must fail")
        if f"fail code={CODES['SYNC_READINESS_STATUS_MISMATCH']}" not in proc_bad_sync_mismatch.stderr:
            return fail(f"badsyncmismatch error code missing: err={proc_bad_sync_mismatch.stderr}")

        build_case(
            report_dir,
            "badsyncpassfields",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_sync_readiness_pass_fields=True,
        )
        proc_bad_sync_pass_fields = run_check(
            report_dir,
            "--prefix",
            "badsyncpassfields",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_sync_pass_fields.returncode == 0:
            return fail("badsyncpassfields must fail")
        if f"fail code={CODES['SYNC_READINESS_PASS_STATUS_FIELDS']}" not in proc_bad_sync_pass_fields.stderr:
            return fail(f"badsyncpassfields error code missing: err={proc_bad_sync_pass_fields.stderr}")

        build_case(
            report_dir,
            "badsanityschema",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_sanity_schema=True,
        )
        proc_bad_sanity_schema = run_check(
            report_dir,
            "--prefix",
            "badsanityschema",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_sanity_schema.returncode == 0:
            return fail("badsanityschema must fail")
        if f"fail code={CODES['SANITY_SCHEMA_MISMATCH']}" not in proc_bad_sanity_schema.stderr:
            return fail(f"badsanityschema error code missing: err={proc_bad_sanity_schema.stderr}")

        build_case(
            report_dir,
            "badsanitystatus",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_sanity_status=True,
        )
        proc_bad_sanity_status = run_check(
            report_dir,
            "--prefix",
            "badsanitystatus",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_sanity_status.returncode == 0:
            return fail("badsanitystatus must fail")
        if f"fail code={CODES['SANITY_STATUS_MISMATCH']}" not in proc_bad_sanity_status.stderr:
            return fail(f"badsanitystatus error code missing: err={proc_bad_sanity_status.stderr}")

        build_case(
            report_dir,
            "badsanitystepmissing",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_sanity_required_step_missing=True,
        )
        proc_bad_sanity_step_missing = run_check(
            report_dir,
            "--prefix",
            "badsanitystepmissing",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_sanity_step_missing.returncode == 0:
            return fail("badsanitystepmissing must fail")
        if f"fail code={CODES['SANITY_REQUIRED_STEP_MISSING']}" not in proc_bad_sanity_step_missing.stderr:
            return fail(f"badsanitystepmissing error code missing: err={proc_bad_sanity_step_missing.stderr}")

        build_case(
            report_dir,
            "badsanitystepfailed",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_sanity_required_step_failed=True,
        )
        proc_bad_sanity_step_failed = run_check(
            report_dir,
            "--prefix",
            "badsanitystepfailed",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_sanity_step_failed.returncode == 0:
            return fail("badsanitystepfailed must fail")
        if f"fail code={CODES['SANITY_REQUIRED_STEP_FAILED']}" not in proc_bad_sanity_step_failed.stderr:
            return fail(f"badsanitystepfailed error code missing: err={proc_bad_sanity_step_failed.stderr}")

        build_case(
            report_dir,
            "badsanitywiredmissing",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_sanity_wired_step_missing=True,
        )
        proc_bad_sanity_wired_missing = run_check(
            report_dir,
            "--prefix",
            "badsanitywiredmissing",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_sanity_wired_missing.returncode == 0:
            return fail("badsanitywiredmissing must fail")
        if f"fail code={CODES['SANITY_REQUIRED_STEP_MISSING']}" not in proc_bad_sanity_wired_missing.stderr:
            return fail(f"badsanitywiredmissing error code missing: err={proc_bad_sanity_wired_missing.stderr}")

        build_case(
            report_dir,
            "badsanitywiredfailed",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_sanity_wired_step_failed=True,
        )
        proc_bad_sanity_wired_failed = run_check(
            report_dir,
            "--prefix",
            "badsanitywiredfailed",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_sanity_wired_failed.returncode == 0:
            return fail("badsanitywiredfailed must fail")
        if f"fail code={CODES['SANITY_REQUIRED_STEP_FAILED']}" not in proc_bad_sanity_wired_failed.stderr:
            return fail(f"badsanitywiredfailed error code missing: err={proc_bad_sanity_wired_failed.stderr}")

        build_case(
            report_dir,
            "badsanitycomparemissing",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_sanity_compare_step_missing=True,
        )
        proc_bad_sanity_compare_missing = run_check(
            report_dir,
            "--prefix",
            "badsanitycomparemissing",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_sanity_compare_missing.returncode == 0:
            return fail("badsanitycomparemissing must fail")
        if f"fail code={CODES['SANITY_REQUIRED_STEP_MISSING']}" not in proc_bad_sanity_compare_missing.stderr:
            return fail(f"badsanitycomparemissing error code missing: err={proc_bad_sanity_compare_missing.stderr}")

        build_case(
            report_dir,
            "badsanitycomparefailed",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_sanity_compare_step_failed=True,
        )
        proc_bad_sanity_compare_failed = run_check(
            report_dir,
            "--prefix",
            "badsanitycomparefailed",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_sanity_compare_failed.returncode == 0:
            return fail("badsanitycomparefailed must fail")
        if f"fail code={CODES['SANITY_REQUIRED_STEP_FAILED']}" not in proc_bad_sanity_compare_failed.stderr:
            return fail(f"badsanitycomparefailed error code missing: err={proc_bad_sanity_compare_failed.stderr}")

        build_case(report_dir, "badnorm", status="pass", with_brief=True, with_triage=True, broken_norm=True)
        proc_bad_norm = run_check(report_dir, "--prefix", "badnorm", "--require-brief", "--require-triage")
        if proc_bad_norm.returncode == 0:
            return fail("badnorm must fail")

        build_case(report_dir, "badbrief", status="pass", with_brief=True, with_triage=True, broken_brief=True)
        proc_bad_brief = run_check(report_dir, "--prefix", "badbrief", "--require-brief", "--require-triage")
        if proc_bad_brief.returncode == 0:
            return fail("badbrief must fail")

        build_case(
            report_dir,
            "badtriagefinal",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_triage_final=True,
        )
        proc_bad_triage_final = run_check(
            report_dir,
            "--prefix",
            "badtriagefinal",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_triage_final.returncode == 0:
            return fail("badtriagefinal must fail")

        build_case(
            report_dir,
            "badartifactref",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_artifact_ref=True,
        )
        proc_bad_artifact_ref = run_check(
            report_dir,
            "--prefix",
            "badartifactref",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_artifact_ref.returncode == 0:
            return fail("badartifactref must fail")

        build_case(
            report_dir,
            "badsummary",
            status="fail",
            with_brief=True,
            with_triage=True,
            broken_summary=True,
        )
        proc_bad_summary = run_check(
            report_dir,
            "--prefix",
            "badsummary",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_summary.returncode == 0:
            return fail("badsummary must fail")

        build_case(
            report_dir,
            "badverifyissue",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_verify_issue=True,
        )
        proc_bad_verify_issue = run_check(
            report_dir,
            "--prefix",
            "badverifyissue",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_verify_issue.returncode == 0:
            return fail("badverifyissue must fail")

    print("[ci-emit-artifacts-check-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
