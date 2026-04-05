#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from _ci_age3_completion_gate_contract import AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS
from _ci_age5_combined_heavy_contract import (
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_REPORT_SCHEMA,
)
from run_ci_sync_readiness_check import (
    AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA,
    SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS,
    SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA,
    SANITY_CONTRACT_SUMMARY_FIELDS,
    SANITY_REQUIRED_PASS_STEPS,
    SANITY_SUMMARY_STEP_FIELDS,
)
from ci_check_error_codes import SYNC_READINESS_REPORT_CODES as CODES

SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA = "ddn.pack_evidence_tier_runner_check.v1"
SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS = (
    "bit_limit",
    "pollard_iters",
    "pollard_c_seeds",
    "pollard_x0_seeds",
    "fallback_limit",
    "small_prime_max",
)


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


def write_valid_sanity_json(path: Path, profile: str = "full") -> None:
    steps = [
        {
            "step": step_name,
            "ok": True,
            "returncode": 0,
            "cmd": ["python", "x.py"],
        }
        for step_name in SANITY_REQUIRED_PASS_STEPS
    ]
    summary_fields: dict[str, str] = {}
    for key, _step_name, enabled_profiles in SANITY_SUMMARY_STEP_FIELDS:
        summary_fields[key] = "1" if profile in enabled_profiles else "na"
    summary_fields.update(
        {
            "ci_sanity_pack_golden_graph_export_ok": "1" if profile in {"full", "core_lang"} else "0",
            "ci_sanity_age2_completion_gate_failure_codes": "-",
            "ci_sanity_age2_completion_gate_failure_code_count": "0",
            "ci_sanity_age3_completion_gate_failure_codes": "-",
            "ci_sanity_age3_completion_gate_failure_code_count": "0",
        }
    )
    summary_fields.update({key: "1" for key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS})
    seamgrim_wasm_web_step_check_schema = "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1"
    seamgrim_wasm_web_step_check_enabled = profile == "seamgrim"
    seamgrim_wasm_web_step_check_min_files = 20
    seamgrim_wasm_web_step_check_report = path.with_name(f"{path.stem}.seamgrim_wasm_web_step_check.detjson")
    if seamgrim_wasm_web_step_check_enabled:
        write_json(
            seamgrim_wasm_web_step_check_report,
            {
                "schema": seamgrim_wasm_web_step_check_schema,
                "status": "pass",
                "ok": True,
                "code": "OK",
                "checked_files": seamgrim_wasm_web_step_check_min_files,
                "missing_count": 0,
                "missing": [],
            },
        )
    summary_fields.update(
        {
            "ci_sanity_seamgrim_wasm_web_step_check_ok": "1" if seamgrim_wasm_web_step_check_enabled else "na",
            "ci_sanity_seamgrim_wasm_web_step_check_report_path": (
                str(seamgrim_wasm_web_step_check_report) if seamgrim_wasm_web_step_check_enabled else "-"
            ),
            "ci_sanity_seamgrim_wasm_web_step_check_report_exists": (
                "1" if seamgrim_wasm_web_step_check_enabled else "na"
            ),
            "ci_sanity_seamgrim_wasm_web_step_check_schema": (
                seamgrim_wasm_web_step_check_schema if seamgrim_wasm_web_step_check_enabled else "-"
            ),
            "ci_sanity_seamgrim_wasm_web_step_check_checked_files": (
                str(seamgrim_wasm_web_step_check_min_files) if seamgrim_wasm_web_step_check_enabled else "-"
            ),
            "ci_sanity_seamgrim_wasm_web_step_check_missing_count": "0" if seamgrim_wasm_web_step_check_enabled else "-",
        }
    )
    seamgrim_pack_evidence_tier_runner_enabled = profile == "seamgrim"
    seamgrim_pack_evidence_tier_runner_report = path.with_name(
        f"{path.stem}.seamgrim_pack_evidence_tier_runner_check.detjson"
    )
    if seamgrim_pack_evidence_tier_runner_enabled:
        write_json(
            seamgrim_pack_evidence_tier_runner_report,
            {
                "schema": SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA,
                "status": "pass",
                "ok": True,
                "docs_profile": {"name": "docs_ssot_rep10", "issue_count": 0},
                "repo_profile": {"name": "repo_rep10", "issue_count": 0},
            },
        )
    summary_fields.update(
        {
            "ci_sanity_seamgrim_pack_evidence_tier_runner_ok": (
                "1" if seamgrim_pack_evidence_tier_runner_enabled else "na"
            ),
            "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path": (
                str(seamgrim_pack_evidence_tier_runner_report)
                if seamgrim_pack_evidence_tier_runner_enabled
                else "-"
            ),
            "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists": (
                "1" if seamgrim_pack_evidence_tier_runner_enabled else "na"
            ),
            "ci_sanity_seamgrim_pack_evidence_tier_runner_schema": (
                SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA
                if seamgrim_pack_evidence_tier_runner_enabled
                else "-"
            ),
            "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count": (
                "0" if seamgrim_pack_evidence_tier_runner_enabled else "-"
            ),
            "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count": (
                "0" if seamgrim_pack_evidence_tier_runner_enabled else "-"
            ),
        }
    )
    seamgrim_numeric_factor_policy_enabled = profile in {"full", "seamgrim"}
    seamgrim_numeric_factor_policy_report = path.with_name(
        f"{path.stem}.seamgrim_numeric_factor_policy.detjson"
    )
    seamgrim_numeric_factor_policy_text = ";".join(
        f"{key}={SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS[key]}"
        for key in SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS
    )
    if seamgrim_numeric_factor_policy_enabled:
        write_json(
            seamgrim_numeric_factor_policy_report,
            {
                "schema": SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA,
                "status": "pass",
                "ok": True,
                "code": "OK",
                "numeric_factor_policy_text": seamgrim_numeric_factor_policy_text,
                "numeric_factor_policy": {
                    key: int(SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS[key])
                    for key in SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS
                },
            },
        )
    summary_fields.update(
        {
            "ci_sanity_seamgrim_numeric_factor_policy_ok": (
                "1" if seamgrim_numeric_factor_policy_enabled else "na"
            ),
            "ci_sanity_seamgrim_numeric_factor_policy_report_path": (
                str(seamgrim_numeric_factor_policy_report)
                if seamgrim_numeric_factor_policy_enabled
                else "-"
            ),
            "ci_sanity_seamgrim_numeric_factor_policy_report_exists": (
                "1" if seamgrim_numeric_factor_policy_enabled else "na"
            ),
            "ci_sanity_seamgrim_numeric_factor_policy_schema": (
                SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA
                if seamgrim_numeric_factor_policy_enabled
                else "-"
            ),
            "ci_sanity_seamgrim_numeric_factor_policy_text": (
                seamgrim_numeric_factor_policy_text
                if seamgrim_numeric_factor_policy_enabled
                else "-"
            ),
            "ci_sanity_seamgrim_numeric_factor_policy_bit_limit": (
                str(SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS["bit_limit"])
                if seamgrim_numeric_factor_policy_enabled
                else "-"
            ),
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_iters": (
                str(SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS["pollard_iters"])
                if seamgrim_numeric_factor_policy_enabled
                else "-"
            ),
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds": (
                str(SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS["pollard_c_seeds"])
                if seamgrim_numeric_factor_policy_enabled
                else "-"
            ),
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds": (
                str(SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS["pollard_x0_seeds"])
                if seamgrim_numeric_factor_policy_enabled
                else "-"
            ),
            "ci_sanity_seamgrim_numeric_factor_policy_fallback_limit": (
                str(SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS["fallback_limit"])
                if seamgrim_numeric_factor_policy_enabled
                else "-"
            ),
            "ci_sanity_seamgrim_numeric_factor_policy_small_prime_max": (
                str(SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS["small_prime_max"])
                if seamgrim_numeric_factor_policy_enabled
                else "-"
            ),
        }
    )
    smoke_report = path.with_name(f"{path.stem}.bogae_geoul_visibility_smoke.detjson")
    write_json(
        smoke_report,
        {
            "schema": AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA,
            "overall_ok": True,
            "checks": [{"name": "artifact_presence", "ok": True}],
            "simulation_hash_delta": {
                "state_hash_changes": True,
                "bogae_hash_changes": True,
            },
        },
    )
    summary_fields.update(
        {
            "ci_sanity_age3_bogae_geoul_visibility_smoke_ok": "1",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path": str(smoke_report),
            "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists": "1",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_schema": AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA,
            "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok": "1",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok": "1",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes": "1",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes": "1",
        }
    )
    fixed64_live_report = path.with_name(f"{path.stem}.fixed64_darwin_real_report_live_check.detjson")
    write_json(
        fixed64_live_report,
        {
            "schema": "ddn.fixed64.darwin_real_report_live_check.v1",
            "ok": True,
            "status": "skip_disabled",
            "resolved_status": "-",
            "resolved_source": "",
            "resolve_invalid_hits": [],
        },
    )
    summary_fields.update(
        {
            "ci_sanity_fixed64_darwin_real_report_live_report_path": str(fixed64_live_report),
            "ci_sanity_fixed64_darwin_real_report_live_report_exists": "1",
            "ci_sanity_fixed64_darwin_real_report_live_status": "skip_disabled",
            "ci_sanity_fixed64_darwin_real_report_live_resolved_status": "-",
            "ci_sanity_fixed64_darwin_real_report_live_resolved_source": "-",
            "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count": "0",
            "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip": "0",
        }
    )
    contract_fields = {key: value for key, value in SANITY_CONTRACT_SUMMARY_FIELDS}
    write_json(
        path,
        {
            "schema": "ddn.ci.sanity_gate.v1",
            "status": "pass",
            "code": "OK",
            "step": "all",
            "profile": profile,
            "msg": "-",
            **summary_fields,
            **contract_fields,
            "steps": steps,
        },
    )


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
        sanity_json = report_dir / f"{prefix}.ci_sanity_gate.detjson"
        write_valid_sanity_json(sanity_json, profile="full")
        run_proc = run(
            [
                py,
                "tests/run_ci_sync_readiness_check.py",
                "--report-dir",
                str(report_dir),
                "--report-prefix",
                prefix,
                "--validate-only-sanity-json",
                str(sanity_json),
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
        if (
            expect(
                str(report_doc.get("ci_sanity_pipeline_emit_flags_ok", "")) == "1",
                "pipeline_flags_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sanity_age2_completion_gate_ok", "")) == "1",
                "age2_completion_gate_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sanity_age3_completion_gate_ok", "")) == "1",
                "age3_completion_gate_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sanity_age2_completion_gate_failure_codes", "")) == "-",
                "age2_completion_gate_failure_codes_should_be_dash",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sanity_age2_completion_gate_failure_code_count", "")) == "0",
                "age2_completion_gate_failure_code_count_should_be_0",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sanity_age3_completion_gate_failure_codes", "")) == "-",
                "age3_completion_gate_failure_codes_should_be_dash",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sanity_age3_completion_gate_failure_code_count", "")) == "0",
                "age3_completion_gate_failure_code_count_should_be_0",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sync_readiness_ci_sanity_age2_completion_gate_failure_codes", "")) == "-",
                "sync_age2_completion_gate_failure_codes_should_be_dash",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sync_readiness_ci_sanity_age2_completion_gate_failure_code_count", "")) == "0",
                "sync_age2_completion_gate_failure_code_count_should_be_0",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sync_readiness_ci_sanity_age3_completion_gate_failure_codes", "")) == "-",
                "sync_age3_completion_gate_failure_codes_should_be_dash",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sync_readiness_ci_sanity_age3_completion_gate_failure_code_count", "")) == "0",
                "sync_age3_completion_gate_failure_code_count_should_be_0",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_ok", "")) == "1",
                "age3_bogae_geoul_visibility_smoke_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_ok", "")) == "na",
                "pack_evidence_runner_ok_should_be_na_for_full_profile",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok", "")) == "na",
                "sync_pack_evidence_runner_ok_should_be_na_for_full_profile",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sanity_fixed64_darwin_real_report_live_status", "")) == "skip_disabled",
                "fixed64_live_status_should_be_skip_disabled",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sanity_fixed64_darwin_real_report_live_resolved_status", "")) == "-",
                "fixed64_live_resolved_status_should_be_dash",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_status", "")) == "-",
                "sync_fixed64_live_resolved_status_should_be_dash",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sanity_age5_combined_heavy_policy_selftest_ok", "")) == "1",
                "age5_combined_heavy_policy_selftest_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok", "")) == "1",
                "profile_matrix_policy_selftest_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sanity_age5_combined_heavy_report_schema", "")) == AGE5_COMBINED_HEAVY_REPORT_SCHEMA,
                "age5_combined_heavy_report_schema_should_be_present",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sanity_age5_combined_heavy_combined_contract_summary_fields", "")) == AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT,
                "age5_combined_heavy_combined_contract_summary_fields_should_be_present",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sanity_age5_combined_heavy_full_summary_contract_fields", "")) == AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT,
                "age5_combined_heavy_full_summary_contract_fields_should_be_present",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(report_doc.get("ci_sanity_age5_combined_heavy_child_summary_default_fields", "")) == AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT,
                "age5_combined_heavy_child_summary_default_fields_should_be_present",
            )
            != 0
        ):
            return 1
        proc_ok = run_report_check(py, root, report, require_pass=True)
        if expect(proc_ok.returncode == 0, "report_check_should_pass", proc_ok) != 0:
            return 1

        seamgrim_prefix = "sync_report_selftest_seamgrim"
        seamgrim_sanity_json = report_dir / f"{seamgrim_prefix}.ci_sanity_gate.detjson"
        write_valid_sanity_json(seamgrim_sanity_json, profile="seamgrim")
        seamgrim_run_proc = run(
            [
                py,
                "tests/run_ci_sync_readiness_check.py",
                "--report-dir",
                str(report_dir),
                "--report-prefix",
                seamgrim_prefix,
                "--validate-only-sanity-json",
                str(seamgrim_sanity_json),
                "--sanity-profile",
                "seamgrim",
                "--skip-aggregate",
            ],
            root,
        )
        if expect(seamgrim_run_proc.returncode == 0, "seamgrim_sync_readiness_run_should_pass", seamgrim_run_proc) != 0:
            return 1
        seamgrim_report = report_dir / f"{seamgrim_prefix}.ci_sync_readiness.detjson"
        if expect(seamgrim_report.exists(), "seamgrim_sync_readiness_report_missing", seamgrim_run_proc) != 0:
            return 1
        seamgrim_report_doc = load_json(seamgrim_report)
        if expect(
            str(seamgrim_report_doc.get("sanity_profile", "")) == "seamgrim",
            "seamgrim_report_sanity_profile_should_be_seamgrim",
        ) != 0:
            return 1
        if expect(
            str(seamgrim_report_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_ok", "")) == "1",
            "seamgrim_report_pack_evidence_ok_should_be_1",
        ) != 0:
            return 1
        if expect(
            str(seamgrim_report_doc.get("ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok", "")) == "1",
            "seamgrim_report_sync_pack_evidence_ok_should_be_1",
        ) != 0:
            return 1
        if expect(
            str(seamgrim_report_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_schema", "")) == SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA,
            "seamgrim_report_pack_evidence_schema_should_match",
        ) != 0:
            return 1
        seamgrim_pack_evidence_report_path = Path(
            str(seamgrim_report_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_report_path", ""))
        )
        if expect(seamgrim_pack_evidence_report_path.exists(), "seamgrim_report_pack_evidence_report_path_should_exist") != 0:
            return 1
        proc_seamgrim_ok = run_report_check(py, root, seamgrim_report, require_pass=True, sanity_profile="seamgrim")
        if expect(proc_seamgrim_ok.returncode == 0, "seamgrim_report_check_should_pass", proc_seamgrim_ok) != 0:
            return 1

        seamgrim_bad_pack_schema_report = report_dir / "bad_seamgrim_pack_schema.ci_sync_readiness.detjson"
        seamgrim_bad_pack_schema_doc = load_json(seamgrim_report)
        seamgrim_bad_pack_schema_doc["ci_sanity_seamgrim_pack_evidence_tier_runner_schema"] = (
            "ddn.pack_evidence_tier_runner_check.v0"
        )
        seamgrim_bad_pack_schema_doc["ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_schema"] = (
            "ddn.pack_evidence_tier_runner_check.v0"
        )
        write_json(seamgrim_bad_pack_schema_report, seamgrim_bad_pack_schema_doc)
        proc_seamgrim_bad_pack_schema = run_report_check(
            py,
            root,
            seamgrim_bad_pack_schema_report,
            require_pass=True,
            sanity_profile="seamgrim",
        )
        if expect(
            proc_seamgrim_bad_pack_schema.returncode != 0,
            "seamgrim_bad_pack_schema_should_fail",
            proc_seamgrim_bad_pack_schema,
        ) != 0:
            return 1
        if (
            expect_fail_code(
                proc_seamgrim_bad_pack_schema,
                CODES["SANITY_SUMMARY_VALUE_INVALID"],
                "seamgrim_bad_pack_schema_fail_code_should_match",
            )
            != 0
        ):
            return 1

        seamgrim_bad_pack_repo_issue_count_report = report_dir / "bad_seamgrim_pack_repo_issue_count.ci_sync_readiness.detjson"
        seamgrim_bad_pack_repo_issue_count_doc = load_json(seamgrim_report)
        seamgrim_bad_pack_repo_issue_count_doc["ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count"] = "1"
        seamgrim_bad_pack_repo_issue_count_doc[
            "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count"
        ] = "1"
        write_json(seamgrim_bad_pack_repo_issue_count_report, seamgrim_bad_pack_repo_issue_count_doc)
        proc_seamgrim_bad_pack_repo_issue_count = run_report_check(
            py,
            root,
            seamgrim_bad_pack_repo_issue_count_report,
            require_pass=True,
            sanity_profile="seamgrim",
        )
        if expect(
            proc_seamgrim_bad_pack_repo_issue_count.returncode != 0,
            "seamgrim_bad_pack_repo_issue_count_should_fail",
            proc_seamgrim_bad_pack_repo_issue_count,
        ) != 0:
            return 1
        if (
            expect_fail_code(
                proc_seamgrim_bad_pack_repo_issue_count,
                CODES["SANITY_SUMMARY_VALUE_INVALID"],
                "seamgrim_bad_pack_repo_issue_count_fail_code_should_match",
            )
            != 0
        ):
            return 1

        seamgrim_bad_pack_sync_path_report = report_dir / "bad_seamgrim_pack_sync_path.ci_sync_readiness.detjson"
        seamgrim_bad_pack_sync_path_doc = load_json(seamgrim_report)
        seamgrim_bad_pack_sync_path_doc["ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path"] = (
            str(seamgrim_report_doc.get("ci_sanity_seamgrim_wasm_web_step_check_report_path", ""))
        )
        write_json(seamgrim_bad_pack_sync_path_report, seamgrim_bad_pack_sync_path_doc)
        proc_seamgrim_bad_pack_sync_path = run_report_check(
            py,
            root,
            seamgrim_bad_pack_sync_path_report,
            require_pass=True,
            sanity_profile="seamgrim",
        )
        if expect(
            proc_seamgrim_bad_pack_sync_path.returncode != 0,
            "seamgrim_bad_pack_sync_path_should_fail",
            proc_seamgrim_bad_pack_sync_path,
        ) != 0:
            return 1
        if (
            expect_fail_code(
                proc_seamgrim_bad_pack_sync_path,
                CODES["SANITY_SUMMARY_VALUE_INVALID"],
                "seamgrim_bad_pack_sync_path_fail_code_should_match",
            )
            != 0
        ):
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

        bad_summary_report = report_dir / "bad_summary.ci_sync_readiness.detjson"
        bad_summary_doc = load_json(report)
        bad_summary_doc["ci_sanity_pipeline_emit_flags_ok"] = "BROKEN"
        write_json(bad_summary_report, bad_summary_doc)
        proc_bad_summary = run_report_check(py, root, bad_summary_report, require_pass=True)
        if expect(proc_bad_summary.returncode != 0, "bad_summary_should_fail", proc_bad_summary) != 0:
            return 1
        if (
            expect_fail_code(
                proc_bad_summary,
                CODES["SANITY_SUMMARY_VALUE_INVALID"],
                "bad_summary_fail_code_should_match",
            )
            != 0
        ):
            return 1

        bad_emit_artifacts_sanity_report = report_dir / "bad_emit_artifacts_sanity.ci_sync_readiness.detjson"
        bad_emit_artifacts_sanity_doc = load_json(report)
        bad_emit_artifacts_sanity_doc["ci_sanity_emit_artifacts_sanity_contract_selftest_ok"] = "0"
        write_json(bad_emit_artifacts_sanity_report, bad_emit_artifacts_sanity_doc)
        proc_bad_emit_artifacts_sanity = run_report_check(
            py,
            root,
            bad_emit_artifacts_sanity_report,
            require_pass=True,
        )
        if (
            expect(
                proc_bad_emit_artifacts_sanity.returncode != 0,
                "bad_emit_artifacts_sanity_should_fail",
                proc_bad_emit_artifacts_sanity,
            )
            != 0
        ):
            return 1
        if (
            expect_fail_code(
                proc_bad_emit_artifacts_sanity,
                CODES["SANITY_SUMMARY_VALUE_INVALID"],
                "bad_emit_artifacts_sanity_fail_code_should_match",
            )
            != 0
        ):
            return 1

        bad_emit_artifacts_sync_mirror_report = report_dir / "bad_emit_artifacts_sync_mirror.ci_sync_readiness.detjson"
        bad_emit_artifacts_sync_mirror_doc = load_json(report)
        bad_emit_artifacts_sync_mirror_doc["ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok"] = (
            "0"
        )
        write_json(bad_emit_artifacts_sync_mirror_report, bad_emit_artifacts_sync_mirror_doc)
        proc_bad_emit_artifacts_sync_mirror = run_report_check(
            py,
            root,
            bad_emit_artifacts_sync_mirror_report,
            require_pass=True,
        )
        if (
            expect(
                proc_bad_emit_artifacts_sync_mirror.returncode != 0,
                "bad_emit_artifacts_sync_mirror_should_fail",
                proc_bad_emit_artifacts_sync_mirror,
            )
            != 0
        ):
            return 1
        if (
            expect_fail_code(
                proc_bad_emit_artifacts_sync_mirror,
                CODES["SANITY_SUMMARY_VALUE_INVALID"],
                "bad_emit_artifacts_sync_mirror_fail_code_should_match",
            )
            != 0
        ):
            return 1

    print("ci sync readiness report check selftest ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
