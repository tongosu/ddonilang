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
    AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA_TEXT,
    AGE5_COMBINED_HEAVY_REQUIRED_REPORTS_TEXT,
)
from run_ci_sync_readiness_check import (
    AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA,
    SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS,
    SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA,
    SANITY_CONTRACT_SUMMARY_FIELDS,
    SANITY_REQUIRED_PASS_STEPS,
    SANITY_SUMMARY_STEP_FIELDS,
)

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
        sanity_json = report_dir / f"{prefix}.ci_sanity_gate.detjson"
        write_valid_sanity_json(sanity_json, profile="full")
        expected_default_report = report_dir / f"{prefix}.ci_sync_readiness.detjson"

        quick_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            prefix,
            "--validate-only-sanity-json",
            str(sanity_json),
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
        if expect(str(doc.get("ci_sanity_pipeline_emit_flags_ok", "")) == "1", "pipeline_flags_ok_should_be_1") != 0:
            return 1
        if (
            expect(
                str(doc.get("ci_sanity_pipeline_emit_flags_selftest_ok", "")) == "1",
                "pipeline_flags_selftest_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if expect(
            str(doc.get("ci_sanity_age2_completion_gate_ok", "")) == "1",
            "age2_completion_gate_ok_should_be_1",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sanity_age2_completion_gate_selftest_ok", "")) == "1",
            "age2_completion_gate_selftest_ok_should_be_1",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sanity_age3_completion_gate_ok", "")) == "1",
            "age3_completion_gate_ok_should_be_1",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sanity_age3_completion_gate_selftest_ok", "")) == "1",
            "age3_completion_gate_selftest_ok_should_be_1",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sanity_age2_completion_gate_failure_codes", "")) == "-",
            "age2_completion_gate_failure_codes_should_be_dash",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sanity_age2_completion_gate_failure_code_count", "")) == "0",
            "age2_completion_gate_failure_code_count_should_be_0",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sanity_age3_completion_gate_failure_codes", "")) == "-",
            "age3_completion_gate_failure_codes_should_be_dash",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sanity_age3_completion_gate_failure_code_count", "")) == "0",
            "age3_completion_gate_failure_code_count_should_be_0",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sync_readiness_ci_sanity_age2_completion_gate_failure_codes", "")) == "-",
            "sync_age2_completion_gate_failure_codes_should_be_dash",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sync_readiness_ci_sanity_age2_completion_gate_failure_code_count", "")) == "0",
            "sync_age2_completion_gate_failure_code_count_should_be_0",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sync_readiness_ci_sanity_age3_completion_gate_failure_codes", "")) == "-",
            "sync_age3_completion_gate_failure_codes_should_be_dash",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sync_readiness_ci_sanity_age3_completion_gate_failure_code_count", "")) == "0",
            "sync_age3_completion_gate_failure_code_count_should_be_0",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_ok", "")) == "1",
            "age3_bogae_geoul_visibility_smoke_ok_should_be_1",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_ok", "")) == "na",
            "pack_evidence_runner_ok_should_be_na_for_full_profile",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok", "")) == "na",
            "sync_pack_evidence_runner_ok_should_be_na_for_full_profile",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sanity_fixed64_darwin_real_report_live_status", "")) == "skip_disabled",
            "fixed64_live_status_should_be_skip_disabled",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sanity_fixed64_darwin_real_report_live_resolved_status", "")) == "-",
            "fixed64_live_resolved_status_should_be_dash",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_status", "")) == "-",
            "sync_fixed64_live_resolved_status_should_be_dash",
        ) != 0:
            return 1
        if (
            expect(
                str(doc.get("ci_sanity_age5_combined_heavy_policy_selftest_ok", "")) == "1",
                "age5_combined_heavy_policy_selftest_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(doc.get("ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok", "")) == "1",
                "profile_matrix_full_real_smoke_policy_selftest_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(doc.get("ci_sanity_dynamic_source_profile_split_selftest_ok", "")) == "1",
                "dynamic_source_profile_split_selftest_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if expect(
            str(doc.get("ci_sanity_age5_combined_heavy_report_schema", "")) == AGE5_COMBINED_HEAVY_REPORT_SCHEMA,
            "age5_combined_heavy_report_schema_should_match",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sanity_age5_combined_heavy_required_reports", "")) == AGE5_COMBINED_HEAVY_REQUIRED_REPORTS_TEXT,
            "age5_combined_heavy_required_reports_should_match",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sanity_age5_combined_heavy_required_criteria", "")) == AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA_TEXT,
            "age5_combined_heavy_required_criteria_should_match",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sanity_age5_combined_heavy_child_summary_default_fields", "")) == AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT,
            "age5_combined_heavy_child_summary_default_fields_should_match",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sanity_age5_combined_heavy_combined_contract_summary_fields", "")) == AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT,
            "age5_combined_heavy_combined_contract_summary_fields_should_match",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sanity_age5_combined_heavy_full_summary_contract_fields", "")) == AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT,
            "age5_combined_heavy_full_summary_contract_fields_should_match",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sync_readiness_ci_sanity_age5_combined_heavy_report_schema", "")) == AGE5_COMBINED_HEAVY_REPORT_SCHEMA,
            "sync_age5_combined_heavy_report_schema_should_match",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields", "")) == AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT,
            "sync_age5_combined_heavy_child_summary_default_fields_should_match",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sync_readiness_ci_sanity_age5_combined_heavy_combined_contract_summary_fields", "")) == AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT,
            "sync_age5_combined_heavy_combined_contract_summary_fields_should_match",
        ) != 0:
            return 1
        if expect(
            str(doc.get("ci_sync_readiness_ci_sanity_age5_combined_heavy_full_summary_contract_fields", "")) == AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT,
            "sync_age5_combined_heavy_full_summary_contract_fields_should_match",
        ) != 0:
            return 1
        if expect(str(doc.get("step", "")) == "all", "step_should_be_all") != 0:
            return 1
        if expect(bool(doc.get("skip_aggregate", False)), "skip_aggregate_should_be_true") != 0:
            return 1
        if expect(int(doc.get("steps_count", 0)) == 1, "steps_count_quick_should_be_1") != 0:
            return 1
        steps = doc.get("steps")
        if expect(isinstance(steps, list), "steps_should_be_list") != 0:
            return 1
        step_names = [str(row.get("name", "")) for row in steps if isinstance(row, dict)]
        if expect(
            step_names
            == [
                "sanity_gate_contract",
            ],
            "unexpected_quick_steps",
        ) != 0:
            return 1
        if expect(bool(steps[-1].get("ok", False)), "sanity_gate_contract_should_be_ok") != 0:
            return 1

        seamgrim_prefix = "sync_selftest_seamgrim"
        seamgrim_sanity_json = report_dir / f"{seamgrim_prefix}.ci_sanity_gate.detjson"
        write_valid_sanity_json(seamgrim_sanity_json, profile="seamgrim")
        seamgrim_report = report_dir / f"{seamgrim_prefix}.ci_sync_readiness.detjson"
        seamgrim_quick_cmd = [
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
        ]
        seamgrim_quick_proc = run(seamgrim_quick_cmd, cwd=root)
        if expect(seamgrim_quick_proc.returncode == 0, "seamgrim_quick_mode_should_pass", seamgrim_quick_proc) != 0:
            return 1
        if expect(seamgrim_report.exists(), "seamgrim_report_missing", seamgrim_quick_proc) != 0:
            return 1
        seamgrim_doc = load_json(seamgrim_report)
        if expect(str(seamgrim_doc.get("status", "")) == "pass", "seamgrim_status_should_be_pass") != 0:
            return 1
        if expect(str(seamgrim_doc.get("sanity_profile", "")) == "seamgrim", "seamgrim_profile_should_be_seamgrim") != 0:
            return 1
        if expect(
            str(seamgrim_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_ok", "")) == "1",
            "seamgrim_pack_evidence_runner_ok_should_be_1",
        ) != 0:
            return 1
        if expect(
            str(seamgrim_doc.get("ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok", "")) == "1",
            "seamgrim_sync_pack_evidence_runner_ok_should_be_1",
        ) != 0:
            return 1
        seamgrim_pack_evidence_report_path = Path(
            str(seamgrim_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_report_path", ""))
        )
        if expect(seamgrim_pack_evidence_report_path.exists(), "seamgrim_pack_evidence_report_path_should_exist") != 0:
            return 1
        if expect(
            str(seamgrim_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists", "")) == "1",
            "seamgrim_pack_evidence_report_exists_should_be_1",
        ) != 0:
            return 1
        if expect(
            str(seamgrim_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_schema", "")) == SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA,
            "seamgrim_pack_evidence_schema_should_match",
        ) != 0:
            return 1
        if expect(
            str(seamgrim_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count", "")) == "0",
            "seamgrim_pack_evidence_docs_issue_count_should_be_0",
        ) != 0:
            return 1
        if expect(
            str(seamgrim_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count", "")) == "0",
            "seamgrim_pack_evidence_repo_issue_count_should_be_0",
        ) != 0:
            return 1
        if expect(
            str(seamgrim_doc.get("ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path", ""))
            == str(seamgrim_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_report_path", "")),
            "seamgrim_sync_pack_evidence_report_path_should_match_sanity",
        ) != 0:
            return 1
        if expect(
            str(seamgrim_doc.get("ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_schema", ""))
            == str(seamgrim_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_schema", "")),
            "seamgrim_sync_pack_evidence_schema_should_match_sanity",
        ) != 0:
            return 1
        if expect(
            str(seamgrim_doc.get("ci_sanity_seamgrim_wasm_web_step_check_ok", "")) == "1",
            "seamgrim_wasm_web_step_check_ok_should_be_1",
        ) != 0:
            return 1

        seamgrim_bad_pack_schema_sanity_json = report_dir / "sync_readiness.seamgrim_bad_pack_schema.ci_sanity_gate.detjson"
        seamgrim_bad_pack_schema_doc = json.loads(json.dumps(load_json(seamgrim_sanity_json), ensure_ascii=False))
        if expect(
            isinstance(seamgrim_bad_pack_schema_doc, dict),
            "seamgrim_bad_pack_schema_doc_should_be_dict",
        ) != 0:
            return 1
        seamgrim_bad_pack_schema_doc["ci_sanity_seamgrim_pack_evidence_tier_runner_schema"] = (
            "ddn.pack_evidence_tier_runner_check.v0"
        )
        write_json(seamgrim_bad_pack_schema_sanity_json, seamgrim_bad_pack_schema_doc)
        seamgrim_bad_pack_schema_json_out = report_dir / "sync_readiness.seamgrim_bad_pack_schema.detjson"
        seamgrim_bad_pack_schema_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            f"{prefix}_seamgrim_bad_pack_schema",
            "--validate-only-sanity-json",
            str(seamgrim_bad_pack_schema_sanity_json),
            "--sanity-profile",
            "seamgrim",
            "--skip-aggregate",
            "--json-out",
            str(seamgrim_bad_pack_schema_json_out),
        ]
        seamgrim_bad_pack_schema_proc = run(seamgrim_bad_pack_schema_cmd, cwd=root)
        if expect(
            seamgrim_bad_pack_schema_proc.returncode != 0,
            "seamgrim_bad_pack_schema_should_fail",
            seamgrim_bad_pack_schema_proc,
        ) != 0:
            return 1
        seamgrim_bad_pack_schema_result_doc = load_json(seamgrim_bad_pack_schema_json_out)
        if expect(
            str(seamgrim_bad_pack_schema_result_doc.get("code", "")) == "E_SYNC_READINESS_SANITY_CONTRACT_FAIL",
            "seamgrim_bad_pack_schema_code_should_be_sanity_contract_fail",
        ) != 0:
            return 1
        if expect(
            "sanity pack_evidence schema mismatch" in str(seamgrim_bad_pack_schema_result_doc.get("msg", "")),
            "seamgrim_bad_pack_schema_msg_should_mention_schema",
        ) != 0:
            return 1

        seamgrim_bad_pack_repo_issue_sanity_json = report_dir / "sync_readiness.seamgrim_bad_pack_repo_issue.ci_sanity_gate.detjson"
        seamgrim_bad_pack_repo_issue_doc = json.loads(json.dumps(load_json(seamgrim_sanity_json), ensure_ascii=False))
        if expect(
            isinstance(seamgrim_bad_pack_repo_issue_doc, dict),
            "seamgrim_bad_pack_repo_issue_doc_should_be_dict",
        ) != 0:
            return 1
        seamgrim_bad_pack_repo_issue_doc["ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count"] = "1"
        write_json(seamgrim_bad_pack_repo_issue_sanity_json, seamgrim_bad_pack_repo_issue_doc)
        seamgrim_bad_pack_repo_issue_json_out = report_dir / "sync_readiness.seamgrim_bad_pack_repo_issue.detjson"
        seamgrim_bad_pack_repo_issue_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            f"{prefix}_seamgrim_bad_pack_repo_issue",
            "--validate-only-sanity-json",
            str(seamgrim_bad_pack_repo_issue_sanity_json),
            "--sanity-profile",
            "seamgrim",
            "--skip-aggregate",
            "--json-out",
            str(seamgrim_bad_pack_repo_issue_json_out),
        ]
        seamgrim_bad_pack_repo_issue_proc = run(seamgrim_bad_pack_repo_issue_cmd, cwd=root)
        if expect(
            seamgrim_bad_pack_repo_issue_proc.returncode != 0,
            "seamgrim_bad_pack_repo_issue_should_fail",
            seamgrim_bad_pack_repo_issue_proc,
        ) != 0:
            return 1
        seamgrim_bad_pack_repo_issue_result_doc = load_json(seamgrim_bad_pack_repo_issue_json_out)
        if expect(
            str(seamgrim_bad_pack_repo_issue_result_doc.get("code", "")) == "E_SYNC_READINESS_SANITY_CONTRACT_FAIL",
            "seamgrim_bad_pack_repo_issue_code_should_be_sanity_contract_fail",
        ) != 0:
            return 1
        if expect(
            "sanity pack_evidence repo_issue_count must be 0" in str(seamgrim_bad_pack_repo_issue_result_doc.get("msg", "")),
            "seamgrim_bad_pack_repo_issue_msg_should_mention_repo_issue_count",
        ) != 0:
            return 1

        seamgrim_bad_wasm_checked_files_sanity_json = report_dir / "sync_readiness.seamgrim_bad_wasm_checked_files.ci_sanity_gate.detjson"
        seamgrim_bad_wasm_checked_files_doc = json.loads(json.dumps(load_json(seamgrim_sanity_json), ensure_ascii=False))
        if expect(
            isinstance(seamgrim_bad_wasm_checked_files_doc, dict),
            "seamgrim_bad_wasm_checked_files_doc_should_be_dict",
        ) != 0:
            return 1
        seamgrim_bad_wasm_checked_files_doc["ci_sanity_seamgrim_wasm_web_step_check_checked_files"] = "10"
        write_json(seamgrim_bad_wasm_checked_files_sanity_json, seamgrim_bad_wasm_checked_files_doc)
        seamgrim_bad_wasm_checked_files_json_out = report_dir / "sync_readiness.seamgrim_bad_wasm_checked_files.detjson"
        seamgrim_bad_wasm_checked_files_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            f"{prefix}_seamgrim_bad_wasm_checked_files",
            "--validate-only-sanity-json",
            str(seamgrim_bad_wasm_checked_files_sanity_json),
            "--sanity-profile",
            "seamgrim",
            "--skip-aggregate",
            "--json-out",
            str(seamgrim_bad_wasm_checked_files_json_out),
        ]
        seamgrim_bad_wasm_checked_files_proc = run(seamgrim_bad_wasm_checked_files_cmd, cwd=root)
        if expect(
            seamgrim_bad_wasm_checked_files_proc.returncode != 0,
            "seamgrim_bad_wasm_checked_files_should_fail",
            seamgrim_bad_wasm_checked_files_proc,
        ) != 0:
            return 1
        seamgrim_bad_wasm_checked_files_result_doc = load_json(seamgrim_bad_wasm_checked_files_json_out)
        if expect(
            str(seamgrim_bad_wasm_checked_files_result_doc.get("code", "")) == "E_SYNC_READINESS_SANITY_CONTRACT_FAIL",
            "seamgrim_bad_wasm_checked_files_code_should_be_sanity_contract_fail",
        ) != 0:
            return 1
        if expect(
            "sanity wasm/web step checked_files too small" in str(seamgrim_bad_wasm_checked_files_result_doc.get("msg", "")),
            "seamgrim_bad_wasm_checked_files_msg_should_mention_checked_files",
        ) != 0:
            return 1

        custom_json = report_dir / "sync_readiness.custom.detjson"
        custom_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            f"{prefix}_custom",
            "--validate-only-sanity-json",
            str(sanity_json),
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
        if expect(str(custom_doc.get("ci_sanity_pipeline_emit_flags_ok", "")) == "1", "custom_pipeline_flags_ok_should_be_1") != 0:
            return 1
        if expect(
            str(custom_doc.get("ci_sanity_age2_completion_gate_ok", "")) == "1",
            "custom_age2_completion_gate_ok_should_be_1",
        ) != 0:
            return 1
        if expect(
            str(custom_doc.get("ci_sanity_age2_completion_gate_selftest_ok", "")) == "1",
            "custom_age2_completion_gate_selftest_ok_should_be_1",
        ) != 0:
            return 1
        if expect(
            str(custom_doc.get("ci_sanity_age3_completion_gate_ok", "")) == "1",
            "custom_age3_completion_gate_ok_should_be_1",
        ) != 0:
            return 1
        if expect(
            str(custom_doc.get("ci_sanity_age3_completion_gate_selftest_ok", "")) == "1",
            "custom_age3_completion_gate_selftest_ok_should_be_1",
        ) != 0:
            return 1
        if expect(
            str(custom_doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_ok", "")) == "1",
            "custom_age3_bogae_geoul_visibility_smoke_ok_should_be_1",
        ) != 0:
            return 1
        if (
            expect(
                str(custom_doc.get("ci_sanity_age5_combined_heavy_policy_selftest_ok", "")) == "1",
                "custom_age5_combined_heavy_policy_selftest_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if expect(
            str(custom_doc.get("ci_sanity_dynamic_source_profile_split_selftest_ok", "")) == "1",
            "custom_dynamic_source_profile_split_selftest_ok_should_be_1",
        ) != 0:
            return 1
        if expect(
            str(custom_doc.get("ci_sanity_age5_combined_heavy_report_schema", "")) == AGE5_COMBINED_HEAVY_REPORT_SCHEMA,
            "custom_age5_combined_heavy_report_schema_should_match",
        ) != 0:
            return 1
        if expect(int(custom_doc.get("steps_count", 0)) == 1, "custom_steps_count_should_be_1") != 0:
            return 1

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
        if (
            expect(
                str(validate_ok_doc.get("ci_sanity_age2_completion_gate_ok", "")) == "1",
                "validate_only_age2_completion_gate_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(validate_ok_doc.get("ci_sanity_age2_completion_gate_selftest_ok", "")) == "1",
                "validate_only_age2_completion_gate_selftest_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(validate_ok_doc.get("ci_sanity_age3_completion_gate_ok", "")) == "1",
                "validate_only_age3_completion_gate_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(validate_ok_doc.get("ci_sanity_age3_completion_gate_selftest_ok", "")) == "1",
                "validate_only_age3_completion_gate_selftest_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(validate_ok_doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_ok", "")) == "1",
                "validate_only_age3_bogae_geoul_visibility_smoke_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(validate_ok_doc.get("ci_sanity_age5_combined_heavy_policy_selftest_ok", "")) == "1",
                "validate_only_age5_combined_heavy_policy_selftest_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(validate_ok_doc.get("ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok", "")) == "1",
                "validate_only_profile_matrix_full_real_smoke_policy_selftest_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(validate_ok_doc.get("ci_sanity_dynamic_source_profile_split_selftest_ok", "")) == "1",
                "validate_only_dynamic_source_profile_split_selftest_ok_should_be_1",
            )
            != 0
        ):
            return 1
        if expect(
            str(validate_ok_doc.get("ci_sanity_age5_combined_heavy_required_reports", "")) == AGE5_COMBINED_HEAVY_REQUIRED_REPORTS_TEXT,
            "validate_only_age5_combined_heavy_required_reports_should_match",
        ) != 0:
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

        bad_summary_json = report_dir / "sync_readiness.bad_summary.detjson"
        bad_summary_doc = load_json(sanity_json)
        if expect(isinstance(bad_summary_doc, dict), "bad_summary_doc_should_be_dict") != 0:
            return 1
        bad_summary_doc.pop("ci_sanity_pipeline_emit_flags_ok", None)
        write_json(bad_summary_json, bad_summary_doc)
        validate_bad_summary_json = report_dir / "sync_readiness.validate_bad_summary.detjson"
        validate_bad_summary_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            f"{prefix}_validate_bad_summary",
            "--validate-only-sanity-json",
            str(bad_summary_json),
            "--json-out",
            str(validate_bad_summary_json),
        ]
        validate_bad_summary_proc = run(validate_bad_summary_cmd, cwd=root)
        if expect(
            validate_bad_summary_proc.returncode != 0,
            "validate_only_missing_summary_should_fail",
            validate_bad_summary_proc,
        ) != 0:
            return 1
        validate_bad_summary_doc = load_json(validate_bad_summary_json)
        if expect(
            str(validate_bad_summary_doc.get("code", "")) == "E_SYNC_READINESS_SANITY_CONTRACT_FAIL",
            "validate_bad_summary_code_should_be_sanity_contract_fail",
        ) != 0:
            return 1
        if expect(
            "sanity summary key missing" in str(validate_bad_summary_doc.get("msg", "")),
            "validate_bad_summary_msg_should_mention_summary_key",
        ) != 0:
            return 1

        bad_missing_parity_json = report_dir / "sync_readiness.bad_missing_parity.detjson"
        base_sanity_doc = load_json(sanity_json)
        if expect(isinstance(base_sanity_doc, dict), "base_sanity_doc_should_be_dict") != 0:
            return 1
        base_steps = base_sanity_doc.get("steps") if isinstance(base_sanity_doc, dict) else None
        if expect(isinstance(base_steps, list), "base_sanity_steps_should_be_list") != 0:
            return 1
        filtered_steps = [
            row
            for row in base_steps
            if isinstance(row, dict)
            and str(row.get("step", row.get("name", ""))).strip() != "seamgrim_wasm_cli_diag_parity_check"
        ]
        if expect(
            len(filtered_steps) < len(base_steps),
            "filtered_steps_should_remove_seamgrim_wasm_cli_diag_parity_check",
        ) != 0:
            return 1
        if len(filtered_steps) + 1 == len(base_steps):
            filtered_steps.append(
                {
                    "step": "dummy_preserve_count",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "dummy.py"],
                }
            )
        bad_missing_parity_doc = json.loads(json.dumps(base_sanity_doc, ensure_ascii=False))
        bad_missing_parity_doc["steps"] = filtered_steps
        write_json(bad_missing_parity_json, bad_missing_parity_doc)

        validate_missing_parity_json = report_dir / "sync_readiness.validate_missing_parity.detjson"
        validate_missing_parity_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            f"{prefix}_validate_missing_parity",
            "--validate-only-sanity-json",
            str(bad_missing_parity_json),
            "--json-out",
            str(validate_missing_parity_json),
        ]
        validate_missing_parity_proc = run(validate_missing_parity_cmd, cwd=root)
        if expect(
            validate_missing_parity_proc.returncode != 0,
            "validate_only_missing_parity_should_fail",
            validate_missing_parity_proc,
        ) != 0:
            return 1
        validate_missing_parity_doc = load_json(validate_missing_parity_json)
        if expect(
            str(validate_missing_parity_doc.get("status", "")) == "fail",
            "validate_missing_parity_status_should_be_fail",
        ) != 0:
            return 1
        if expect(
            str(validate_missing_parity_doc.get("code", "")) == "E_SYNC_READINESS_SANITY_CONTRACT_FAIL",
            "validate_missing_parity_code_should_be_sanity_contract_fail",
        ) != 0:
            return 1
        if expect(
            str(validate_missing_parity_doc.get("step", "")) == "sanity_gate_contract",
            "validate_missing_parity_step_should_be_sanity_gate_contract",
        ) != 0:
            return 1
        if expect(
            "seamgrim_wasm_cli_diag_parity_check" in str(validate_missing_parity_doc.get("msg", "")),
            "validate_missing_parity_msg_should_mention_step",
        ) != 0:
            return 1

        bad_missing_wasm_web_selftest_json = report_dir / "sync_readiness.bad_missing_wasm_web_selftest.detjson"
        base_sanity_doc_for_wasm_selftest = load_json(sanity_json)
        if expect(isinstance(base_sanity_doc_for_wasm_selftest, dict), "base_sanity_doc_for_wasm_selftest_should_be_dict") != 0:
            return 1
        base_steps_for_wasm_selftest = (
            base_sanity_doc_for_wasm_selftest.get("steps")
            if isinstance(base_sanity_doc_for_wasm_selftest, dict)
            else None
        )
        if expect(isinstance(base_steps_for_wasm_selftest, list), "base_sanity_steps_for_wasm_selftest_should_be_list") != 0:
            return 1
        filtered_steps_for_wasm_selftest = [
            row
            for row in base_steps_for_wasm_selftest
            if isinstance(row, dict)
            and str(row.get("step", row.get("name", ""))).strip()
            != "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest"
        ]
        if expect(
            len(filtered_steps_for_wasm_selftest) < len(base_steps_for_wasm_selftest),
            "filtered_steps_should_remove_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
        ) != 0:
            return 1
        if len(filtered_steps_for_wasm_selftest) + 1 == len(base_steps_for_wasm_selftest):
            filtered_steps_for_wasm_selftest.append(
                {
                    "step": "dummy_preserve_count_wasm_web_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "dummy.py"],
                }
            )
        bad_missing_wasm_web_selftest_doc = json.loads(
            json.dumps(base_sanity_doc_for_wasm_selftest, ensure_ascii=False)
        )
        bad_missing_wasm_web_selftest_doc["steps"] = filtered_steps_for_wasm_selftest
        write_json(bad_missing_wasm_web_selftest_json, bad_missing_wasm_web_selftest_doc)

        validate_missing_wasm_web_selftest_json = report_dir / "sync_readiness.validate_missing_wasm_web_selftest.detjson"
        validate_missing_wasm_web_selftest_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            f"{prefix}_validate_missing_wasm_web_selftest",
            "--validate-only-sanity-json",
            str(bad_missing_wasm_web_selftest_json),
            "--json-out",
            str(validate_missing_wasm_web_selftest_json),
        ]
        validate_missing_wasm_web_selftest_proc = run(validate_missing_wasm_web_selftest_cmd, cwd=root)
        if expect(
            validate_missing_wasm_web_selftest_proc.returncode != 0,
            "validate_only_missing_wasm_web_smoke_selftest_should_fail",
            validate_missing_wasm_web_selftest_proc,
        ) != 0:
            return 1
        validate_missing_wasm_web_selftest_doc = load_json(validate_missing_wasm_web_selftest_json)
        if expect(
            str(validate_missing_wasm_web_selftest_doc.get("status", "")) == "fail",
            "validate_missing_wasm_web_smoke_selftest_status_should_be_fail",
        ) != 0:
            return 1
        if expect(
            str(validate_missing_wasm_web_selftest_doc.get("code", "")) == "E_SYNC_READINESS_SANITY_CONTRACT_FAIL",
            "validate_missing_wasm_web_smoke_selftest_code_should_be_sanity_contract_fail",
        ) != 0:
            return 1
        if expect(
            str(validate_missing_wasm_web_selftest_doc.get("step", "")) == "sanity_gate_contract",
            "validate_missing_wasm_web_smoke_selftest_step_should_be_sanity_gate_contract",
        ) != 0:
            return 1
        if expect(
            "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest"
            in str(validate_missing_wasm_web_selftest_doc.get("msg", "")),
            "validate_missing_wasm_web_smoke_selftest_msg_should_mention_step",
        ) != 0:
            return 1

        bad_missing_age2_close_selftest_json = report_dir / "sync_readiness.bad_missing_age2_close_selftest.detjson"
        base_sanity_doc_for_age2_close_selftest = load_json(sanity_json)
        if expect(isinstance(base_sanity_doc_for_age2_close_selftest, dict), "base_sanity_doc_for_age2_close_selftest_should_be_dict") != 0:
            return 1
        base_steps_for_age2_close_selftest = (
            base_sanity_doc_for_age2_close_selftest.get("steps")
            if isinstance(base_sanity_doc_for_age2_close_selftest, dict)
            else None
        )
        if expect(isinstance(base_steps_for_age2_close_selftest, list), "base_sanity_steps_for_age2_close_selftest_should_be_list") != 0:
            return 1
        filtered_steps_for_age2_close_selftest = [
            row
            for row in base_steps_for_age2_close_selftest
            if isinstance(row, dict)
            and str(row.get("step", row.get("name", ""))).strip() != "age2_close_selftest"
        ]
        if expect(
            len(filtered_steps_for_age2_close_selftest) < len(base_steps_for_age2_close_selftest),
            "filtered_steps_should_remove_age2_close_selftest",
        ) != 0:
            return 1
        if len(filtered_steps_for_age2_close_selftest) + 1 == len(base_steps_for_age2_close_selftest):
            filtered_steps_for_age2_close_selftest.append(
                {
                    "step": "dummy_preserve_count_age2_close_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "dummy.py"],
                }
            )
        bad_missing_age2_close_selftest_doc = json.loads(
            json.dumps(base_sanity_doc_for_age2_close_selftest, ensure_ascii=False)
        )
        bad_missing_age2_close_selftest_doc["steps"] = filtered_steps_for_age2_close_selftest
        write_json(bad_missing_age2_close_selftest_json, bad_missing_age2_close_selftest_doc)

        validate_missing_age2_close_selftest_json = report_dir / "sync_readiness.validate_missing_age2_close_selftest.detjson"
        validate_missing_age2_close_selftest_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            f"{prefix}_validate_missing_age2_close_selftest",
            "--validate-only-sanity-json",
            str(bad_missing_age2_close_selftest_json),
            "--json-out",
            str(validate_missing_age2_close_selftest_json),
        ]
        validate_missing_age2_close_selftest_proc = run(validate_missing_age2_close_selftest_cmd, cwd=root)
        if expect(
            validate_missing_age2_close_selftest_proc.returncode != 0,
            "validate_only_missing_age2_close_selftest_should_fail",
            validate_missing_age2_close_selftest_proc,
        ) != 0:
            return 1
        validate_missing_age2_close_selftest_doc = load_json(validate_missing_age2_close_selftest_json)
        if expect(
            str(validate_missing_age2_close_selftest_doc.get("status", "")) == "fail",
            "validate_missing_age2_close_selftest_status_should_be_fail",
        ) != 0:
            return 1
        if expect(
            str(validate_missing_age2_close_selftest_doc.get("code", "")) == "E_SYNC_READINESS_SANITY_CONTRACT_FAIL",
            "validate_missing_age2_close_selftest_code_should_be_sanity_contract_fail",
        ) != 0:
            return 1
        if expect(
            str(validate_missing_age2_close_selftest_doc.get("step", "")) == "sanity_gate_contract",
            "validate_missing_age2_close_selftest_step_should_be_sanity_gate_contract",
        ) != 0:
            return 1
        if expect(
            "age2_close_selftest" in str(validate_missing_age2_close_selftest_doc.get("msg", "")),
            "validate_missing_age2_close_selftest_msg_should_mention_step",
        ) != 0:
            return 1

        bad_missing_age3_close_selftest_json = report_dir / "sync_readiness.bad_missing_age3_close_selftest.detjson"
        base_sanity_doc_for_age3_close_selftest = load_json(sanity_json)
        if expect(isinstance(base_sanity_doc_for_age3_close_selftest, dict), "base_sanity_doc_for_age3_close_selftest_should_be_dict") != 0:
            return 1
        base_steps_for_age3_close_selftest = (
            base_sanity_doc_for_age3_close_selftest.get("steps")
            if isinstance(base_sanity_doc_for_age3_close_selftest, dict)
            else None
        )
        if expect(isinstance(base_steps_for_age3_close_selftest, list), "base_sanity_steps_for_age3_close_selftest_should_be_list") != 0:
            return 1
        filtered_steps_for_age3_close_selftest = [
            row
            for row in base_steps_for_age3_close_selftest
            if isinstance(row, dict)
            and str(row.get("step", row.get("name", ""))).strip() != "age3_close_selftest"
        ]
        if expect(
            len(filtered_steps_for_age3_close_selftest) < len(base_steps_for_age3_close_selftest),
            "filtered_steps_should_remove_age3_close_selftest",
        ) != 0:
            return 1
        if len(filtered_steps_for_age3_close_selftest) + 1 == len(base_steps_for_age3_close_selftest):
            filtered_steps_for_age3_close_selftest.append(
                {
                    "step": "dummy_preserve_count_age3_close_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "dummy.py"],
                }
            )
        bad_missing_age3_close_selftest_doc = json.loads(
            json.dumps(base_sanity_doc_for_age3_close_selftest, ensure_ascii=False)
        )
        bad_missing_age3_close_selftest_doc["steps"] = filtered_steps_for_age3_close_selftest
        write_json(bad_missing_age3_close_selftest_json, bad_missing_age3_close_selftest_doc)

        validate_missing_age3_close_selftest_json = report_dir / "sync_readiness.validate_missing_age3_close_selftest.detjson"
        validate_missing_age3_close_selftest_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            f"{prefix}_validate_missing_age3_close_selftest",
            "--validate-only-sanity-json",
            str(bad_missing_age3_close_selftest_json),
            "--json-out",
            str(validate_missing_age3_close_selftest_json),
        ]
        validate_missing_age3_close_selftest_proc = run(validate_missing_age3_close_selftest_cmd, cwd=root)
        if expect(
            validate_missing_age3_close_selftest_proc.returncode != 0,
            "validate_only_missing_age3_close_selftest_should_fail",
            validate_missing_age3_close_selftest_proc,
        ) != 0:
            return 1
        validate_missing_age3_close_selftest_doc = load_json(validate_missing_age3_close_selftest_json)
        if expect(
            str(validate_missing_age3_close_selftest_doc.get("status", "")) == "fail",
            "validate_missing_age3_close_selftest_status_should_be_fail",
        ) != 0:
            return 1
        if expect(
            str(validate_missing_age3_close_selftest_doc.get("code", "")) == "E_SYNC_READINESS_SANITY_CONTRACT_FAIL",
            "validate_missing_age3_close_selftest_code_should_be_sanity_contract_fail",
        ) != 0:
            return 1
        if expect(
            str(validate_missing_age3_close_selftest_doc.get("step", "")) == "sanity_gate_contract",
            "validate_missing_age3_close_selftest_step_should_be_sanity_gate_contract",
        ) != 0:
            return 1
        if expect(
            "age3_close_selftest" in str(validate_missing_age3_close_selftest_doc.get("msg", "")),
            "validate_missing_age3_close_selftest_msg_should_mention_step",
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
