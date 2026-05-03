#!/usr/bin/env python
from __future__ import annotations

import json
import tempfile
from pathlib import Path

from _ci_aggregate_contract_only_lib import (
    build_contract_only_profile_matrix_triage_snapshot,
    resolve_contract_only_required_steps,
    resolve_contract_only_sanity_steps,
    resolve_contract_only_selected_profiles,
    write_contract_only_ci_gate_outputs,
    write_contract_only_ci_sanity_report,
    write_contract_only_ci_sync_readiness_report,
    write_contract_only_fixed64_reports,
    write_contract_only_profile_matrix_selftest_report,
    write_contract_only_stub_reports,
)
from _ci_seamgrim_step_contract import SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS
from _ci_profile_matrix_selftest_lib import (
    PROFILE_MATRIX_SELFTEST_PROFILES,
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC,
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
    PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS,
    PROFILE_MATRIX_TRIAGE_REQUIRED_KEYS,
    build_profile_matrix_selftest_fixture,
    expected_profile_matrix_summary_values,
    format_profile_matrix_summary_values,
    parse_profile_matrix_selftest_real_profiles,
    profile_matrix_triage_missing_keys,
    validate_profile_matrix_aggregate_summary,
)


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def expect(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> int:
    expect(
        PROFILE_MATRIX_SELFTEST_PROFILES == ("core_lang", "full", "seamgrim"),
        "profile matrix selftest helper profiles mismatch",
    )
    fixture_selected, fixture_invalid = parse_profile_matrix_selftest_real_profiles("core_lang, seamgrim, invalid")
    expect(fixture_selected == ["core_lang", "seamgrim"], "profile matrix helper selected profiles mismatch")
    expect(fixture_invalid == ["invalid"], "profile matrix helper invalid profiles mismatch")
    base_fixture = build_profile_matrix_selftest_fixture(["core_lang", "seamgrim"], quick=True, dry=False)
    expect(base_fixture["selected_real_profiles"] == ["core_lang", "seamgrim"], "base fixture selected profiles mismatch")
    expect(base_fixture["skipped_real_profiles"] == ["full"], "base fixture skipped profiles mismatch")
    expect(
        str(base_fixture.get("step_timeout_defaults_text", "")).strip() == PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
        "base fixture timeout defaults mismatch",
    )
    expect(base_fixture["aggregate_summary_sanity_skipped_profiles"] == ["core_lang", "seamgrim"], "base fixture aggregate skipped profiles mismatch")
    expect(
        base_fixture["aggregate_summary_sanity_by_profile"]["core_lang"]["expected_values"]
        == expected_profile_matrix_summary_values("core_lang"),
        "base fixture core_lang expected values mismatch",
    )
    expect(
        base_fixture["aggregate_summary_sanity_by_profile"]["seamgrim"]["expected_values"]
        == expected_profile_matrix_summary_values("seamgrim"),
        "base fixture seamgrim expected values mismatch",
    )

    expect(
        resolve_contract_only_selected_profiles(" full , seamgrim , unknown ", "core_lang") == ["full", "seamgrim"],
        "selected profiles parser must preserve valid profile order",
    )
    expect(
        resolve_contract_only_selected_profiles("", "core_lang") == ["core_lang"],
        "selected profiles parser must use fallback on empty input",
    )
    core_lang_sanity_steps = resolve_contract_only_sanity_steps("core_lang")
    full_sanity_steps = resolve_contract_only_sanity_steps("full")
    seamgrim_sanity_steps = resolve_contract_only_sanity_steps("seamgrim")
    core_lang_required_steps = resolve_contract_only_required_steps("core_lang")
    full_required_steps = resolve_contract_only_required_steps("full")
    expect(
        "seamgrim_wasm_cli_diag_parity_check" not in core_lang_sanity_steps,
        "core_lang sanity steps must not include seamgrim-only parity step",
    )
    expect(
        "age3_close_selftest" in core_lang_sanity_steps,
        "core_lang sanity steps must include age3_close_selftest",
    )
    expect(
        "seamgrim_wasm_cli_diag_parity_check" in full_sanity_steps,
        "full sanity steps must include seamgrim parity step",
    )
    expect(
        "ci_emit_artifacts_sanity_contract_selftest" in full_sanity_steps,
        "full sanity steps must include ci_emit_artifacts_sanity_contract_selftest",
    )
    expect(
        "seamgrim_ci_gate_sam_seulgi_family_step_check" in full_sanity_steps,
        "full sanity steps must include seamgrim_ci_gate_sam_seulgi_family_step_check",
    )
    expect(
        "seamgrim_ci_gate_pack_evidence_tier_step_check" in full_sanity_steps,
        "full sanity steps must include seamgrim_ci_gate_pack_evidence_tier_step_check",
    )
    expect(
        "seamgrim_ci_gate_pack_evidence_tier_step_check_selftest" in full_sanity_steps,
        "full sanity steps must include seamgrim_ci_gate_pack_evidence_tier_step_check_selftest",
    )
    expect(
        "seamgrim_ci_gate_pack_evidence_tier_runner_check" in full_sanity_steps,
        "full sanity steps must include seamgrim_ci_gate_pack_evidence_tier_runner_check",
    )
    expect(
        "seamgrim_ci_gate_pack_evidence_tier_report_check" in full_sanity_steps,
        "full sanity steps must include seamgrim_ci_gate_pack_evidence_tier_report_check",
    )
    expect(
        "seamgrim_ci_gate_pack_evidence_tier_report_check_selftest" in full_sanity_steps,
        "full sanity steps must include seamgrim_ci_gate_pack_evidence_tier_report_check_selftest",
    )
    expect(
        "seamgrim_wasm_cli_diag_parity_check" in seamgrim_sanity_steps,
        "seamgrim sanity steps must include seamgrim parity step",
    )
    expect(
        "ci_emit_artifacts_sanity_contract_selftest" in seamgrim_sanity_steps,
        "seamgrim sanity steps must include ci_emit_artifacts_sanity_contract_selftest",
    )
    expect(
        "seamgrim_ci_gate_sam_seulgi_family_step_check" in seamgrim_sanity_steps,
        "seamgrim sanity steps must include seamgrim_ci_gate_sam_seulgi_family_step_check",
    )
    expect(
        "seamgrim_ci_gate_pack_evidence_tier_step_check" in seamgrim_sanity_steps,
        "seamgrim sanity steps must include seamgrim_ci_gate_pack_evidence_tier_step_check",
    )
    expect(
        "seamgrim_ci_gate_pack_evidence_tier_step_check_selftest" in seamgrim_sanity_steps,
        "seamgrim sanity steps must include seamgrim_ci_gate_pack_evidence_tier_step_check_selftest",
    )
    expect(
        "seamgrim_ci_gate_pack_evidence_tier_runner_check" in seamgrim_sanity_steps,
        "seamgrim sanity steps must include seamgrim_ci_gate_pack_evidence_tier_runner_check",
    )
    expect(
        "seamgrim_ci_gate_pack_evidence_tier_report_check" in seamgrim_sanity_steps,
        "seamgrim sanity steps must include seamgrim_ci_gate_pack_evidence_tier_report_check",
    )
    expect(
        "seamgrim_ci_gate_pack_evidence_tier_report_check_selftest" in seamgrim_sanity_steps,
        "seamgrim sanity steps must include seamgrim_ci_gate_pack_evidence_tier_report_check_selftest",
    )
    expect(
        "seamgrim_wasm_cli_diag_parity_check" not in core_lang_required_steps,
        "core_lang required aggregate steps must exclude seamgrim parity step",
    )
    expect(
        "ci_gate_report_index_latest_smoke_check" in core_lang_required_steps,
        "core_lang required aggregate steps must include report-index latest smoke step",
    )
    expect(
        "seamgrim_wasm_cli_diag_parity_check" in full_required_steps,
        "full required aggregate steps must include seamgrim parity step",
    )
    expect(
        "ci_gate_report_index_latest_smoke_check" in full_required_steps,
        "full required aggregate steps must include report-index latest smoke step",
    )
    for step_name in SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS:
        expect(
            step_name in full_sanity_steps,
            f"full sanity steps must include seamgrim required step {step_name}",
        )
        expect(
            step_name in seamgrim_sanity_steps,
            f"seamgrim sanity steps must include seamgrim required step {step_name}",
        )
        expect(
            step_name in full_required_steps,
            f"full required aggregate steps must include seamgrim required step {step_name}",
        )
        expect(
            step_name not in core_lang_required_steps,
            f"core_lang required aggregate steps must exclude seamgrim required step {step_name}",
        )

    with tempfile.TemporaryDirectory(prefix="ci_aggregate_contract_only_lib_selftest_") as td:
        root = Path(td)
        profile_matrix_report = root / "ci_profile_matrix_gate_selftest.detjson"
        write_contract_only_profile_matrix_selftest_report(
            profile_matrix_report,
            ["core_lang", "full", "seamgrim"],
        )
        profile_matrix_doc = load_json(profile_matrix_report)
        expect(
            profile_matrix_doc.get("schema") == "ddn.ci.profile_matrix_gate_selftest.v1",
            "profile matrix schema mismatch",
        )
        expect(
            profile_matrix_doc.get("aggregate_summary_sanity_checked_profiles") == ["core_lang", "full", "seamgrim"],
            "profile matrix checked profiles mismatch",
        )
        expect(
            profile_matrix_doc.get("selected_real_profiles") == list(PROFILE_MATRIX_SELFTEST_PROFILES),
            "contract-only profile matrix selected profiles mismatch",
        )
        expect(
            str(profile_matrix_doc.get("step_timeout_defaults_text", "")).strip() == PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
            "contract-only profile matrix timeout defaults mismatch",
        )
        by_profile = profile_matrix_doc.get("aggregate_summary_sanity_by_profile", {})
        expect(
            by_profile.get("seamgrim", {}).get("values", {}).get("ci_sanity_canon_ast_dpack_ok") == "0",
            "seamgrim aggregate summary values mismatch",
        )
        expect(
            by_profile.get("core_lang", {}).get("values", {}) == expected_profile_matrix_summary_values("core_lang"),
            "core_lang aggregate summary values mismatch",
        )
        expect(
            by_profile.get("full", {}).get("values", {}) == expected_profile_matrix_summary_values("full"),
            "full aggregate summary values mismatch",
        )
        expect(
            by_profile.get("core_lang", {}).get("expected_profile") == "core_lang"
            and by_profile.get("full", {}).get("expected_profile") == "full"
            and by_profile.get("seamgrim", {}).get("expected_profile") == "seamgrim",
            "aggregate summary expected_profile mismatch",
        )
        expect(
            validate_profile_matrix_aggregate_summary(
                by_profile.get("core_lang"),
                profile="core_lang",
                expected_values=expected_profile_matrix_summary_values("core_lang"),
                expected_present=True,
                expected_gate_marker=False,
            )
            is None,
            "core_lang aggregate summary validator mismatch",
        )
        expect(
            validate_profile_matrix_aggregate_summary(
                by_profile.get("seamgrim"),
                profile="seamgrim",
                expected_values=expected_profile_matrix_summary_values("seamgrim"),
                expected_present=True,
                expected_gate_marker=True,
            )
            is None,
            "seamgrim aggregate summary validator mismatch",
        )

        single_triage = build_contract_only_profile_matrix_triage_snapshot(profile_matrix_report, ["core_lang"])
        expect(
            single_triage["aggregate_summary_sanity_checked_profiles"] == ["core_lang"],
            "single-profile triage checked profiles mismatch",
        )
        expect(
            single_triage["seamgrim_aggregate_summary_status"] == "skipped",
            "single-profile triage seamgrim status must be skipped",
        )
        expect(
            str(single_triage.get("step_timeout_defaults_text", "")).strip() == PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
            "single-profile triage timeout defaults mismatch",
        )
        all_triage = build_contract_only_profile_matrix_triage_snapshot(
            profile_matrix_report,
            ["core_lang", "full", "seamgrim"],
        )
        expect(
            all_triage["aggregate_summary_sanity_skipped_profiles"] == [],
            "all-profile triage skipped profiles must be empty",
        )
        expect(
            all_triage["seamgrim_aggregate_summary_values"]
            == format_profile_matrix_summary_values(expected_profile_matrix_summary_values("seamgrim")),
            "all-profile triage seamgrim values mismatch",
        )

        ci_sanity_gate_report = root / "ci_sanity_gate.detjson"
        write_contract_only_ci_sanity_report(ci_sanity_gate_report, "full")
        sanity_doc = load_json(ci_sanity_gate_report)
        expect(sanity_doc.get("schema") == "ddn.ci.sanity_gate.v1", "ci_sanity schema mismatch")
        expect(sanity_doc.get("status") == "pass", "ci_sanity status mismatch")
        expect(
            sanity_doc.get("ci_sanity_emit_artifacts_sanity_contract_selftest_ok") == "1",
            "ci_sanity emit artifacts sanity contract selftest key mismatch",
        )
        expect(
            sanity_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_ok") == "na",
            "ci_sanity seamgrim pack evidence ok default mismatch",
        )
        expect(
            sanity_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_report_path") == "-",
            "ci_sanity seamgrim pack evidence report path default mismatch",
        )
        full_numeric_policy_report_path = Path(
            str(sanity_doc.get("ci_sanity_seamgrim_numeric_factor_policy_report_path", ""))
        )
        expect(
            sanity_doc.get("ci_sanity_seamgrim_numeric_factor_policy_ok") == "1",
            "ci_sanity full numeric factor policy ok mismatch",
        )
        expect(
            sanity_doc.get("ci_sanity_seamgrim_numeric_factor_policy_schema")
            == "ddn.numeric_factor_route_diag_contract.v1",
            "ci_sanity full numeric factor policy schema mismatch",
        )
        expect(
            sanity_doc.get("ci_sanity_seamgrim_numeric_factor_policy_bit_limit") == "512",
            "ci_sanity full numeric factor policy bit_limit mismatch",
        )
        expect(
            full_numeric_policy_report_path.exists(),
            "ci_sanity full numeric factor policy report path must exist",
        )
        expect(
            len(sanity_doc.get("steps", [])) == len(resolve_contract_only_sanity_steps("full")),
            "ci_sanity step count mismatch",
        )

        ci_sync_readiness_report = root / "ci_sync_readiness.detjson"
        write_contract_only_ci_sync_readiness_report(ci_sync_readiness_report, "full")
        sync_doc = load_json(ci_sync_readiness_report)
        expect(sync_doc.get("schema") == "ddn.ci.sync_readiness.v1", "ci_sync schema mismatch")
        expect(sync_doc.get("sanity_profile") == "full", "ci_sync profile mismatch")
        expect(
            sync_doc.get("ci_sanity_emit_artifacts_sanity_contract_selftest_ok") == "1",
            "ci_sync emit artifacts sanity contract selftest key mismatch",
        )
        expect(
            sync_doc.get("ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok") == "1",
            "ci_sync prefixed emit artifacts sanity contract selftest key mismatch",
        )
        expect(
            sync_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_ok") == "na",
            "ci_sync seamgrim pack evidence ok default mismatch",
        )
        expect(
            sync_doc.get("ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok") == "na",
            "ci_sync prefixed seamgrim pack evidence ok default mismatch",
        )
        full_sync_numeric_policy_report_path = Path(
            str(
                sync_doc.get(
                    "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_report_path",
                    "",
                )
            )
        )
        expect(
            sync_doc.get("ci_sanity_seamgrim_numeric_factor_policy_ok") == "1",
            "ci_sync full numeric factor policy ok mismatch",
        )
        expect(
            sync_doc.get("ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok") == "1",
            "ci_sync prefixed full numeric factor policy ok mismatch",
        )
        expect(
            sync_doc.get("ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_schema")
            == "ddn.numeric_factor_route_diag_contract.v1",
            "ci_sync prefixed full numeric factor policy schema mismatch",
        )
        expect(
            sync_doc.get("ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_small_prime_max")
            == "101",
            "ci_sync prefixed full numeric factor policy small_prime_max mismatch",
        )
        expect(
            full_sync_numeric_policy_report_path.exists(),
            "ci_sync prefixed full numeric factor policy report path must exist",
        )
        expect(sync_doc.get("steps", [{}])[0].get("step") == "validate_only_sanity_json", "ci_sync step mismatch")

        ci_sanity_core_lang_report = root / "ci_sanity_gate.core_lang.detjson"
        write_contract_only_ci_sanity_report(ci_sanity_core_lang_report, "core_lang")
        sanity_core_lang_doc = load_json(ci_sanity_core_lang_report)
        expect(
            sanity_core_lang_doc.get("ci_sanity_seamgrim_numeric_factor_policy_ok") == "na",
            "core_lang ci_sanity numeric factor policy ok default mismatch",
        )
        expect(
            sanity_core_lang_doc.get("ci_sanity_seamgrim_numeric_factor_policy_report_path") == "-",
            "core_lang ci_sanity numeric factor policy report path default mismatch",
        )
        expect(
            sanity_core_lang_doc.get("ci_sanity_seamgrim_numeric_factor_policy_schema") == "-",
            "core_lang ci_sanity numeric factor policy schema default mismatch",
        )

        ci_sanity_seamgrim_report = root / "ci_sanity_gate.seamgrim.detjson"
        write_contract_only_ci_sanity_report(ci_sanity_seamgrim_report, "seamgrim")
        sanity_seamgrim_doc = load_json(ci_sanity_seamgrim_report)
        seamgrim_pack_report_path = Path(
            str(sanity_seamgrim_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_report_path", ""))
        )
        seamgrim_numeric_policy_report_path = Path(
            str(sanity_seamgrim_doc.get("ci_sanity_seamgrim_numeric_factor_policy_report_path", ""))
        )
        expect(
            sanity_seamgrim_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_ok") == "1",
            "seamgrim ci_sanity pack evidence ok mismatch",
        )
        expect(
            sanity_seamgrim_doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_schema")
            == "ddn.pack_evidence_tier_runner_check.v1",
            "seamgrim ci_sanity pack evidence schema mismatch",
        )
        expect(
            seamgrim_pack_report_path.exists(),
            "seamgrim ci_sanity pack evidence report path must exist",
        )
        expect(
            sanity_seamgrim_doc.get("ci_sanity_seamgrim_numeric_factor_policy_ok") == "1",
            "seamgrim ci_sanity numeric factor policy ok mismatch",
        )
        expect(
            sanity_seamgrim_doc.get("ci_sanity_seamgrim_numeric_factor_policy_schema")
            == "ddn.numeric_factor_route_diag_contract.v1",
            "seamgrim ci_sanity numeric factor policy schema mismatch",
        )
        expect(
            sanity_seamgrim_doc.get("ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds") == "6",
            "seamgrim ci_sanity numeric factor policy pollard_x0_seeds mismatch",
        )
        expect(
            seamgrim_numeric_policy_report_path.exists(),
            "seamgrim ci_sanity numeric factor policy report path must exist",
        )

        fixed64_inputs_report = root / "fixed64_threeway_inputs.detjson"
        fixed64_gate_report = root / "fixed64_cross_platform_threeway_gate.detjson"
        write_contract_only_fixed64_reports(fixed64_inputs_report, fixed64_gate_report)
        expect(
            load_json(fixed64_inputs_report).get("schema") == "ddn.fixed64.threeway_inputs.v1",
            "fixed64 inputs schema mismatch",
        )
        expect(
            load_json(fixed64_gate_report).get("schema") == "ddn.fixed64.cross_platform_threeway_gate.v1",
            "fixed64 gate schema mismatch",
        )

        seamgrim_report = root / "seamgrim.ci_gate_report.detjson"
        seamgrim_ui_age3_report = root / "seamgrim.ui_age3_gate_report.detjson"
        seamgrim_phase3_cleanup_report = root / "seamgrim.phase3_cleanup_gate_report.detjson"
        seamgrim_browse_selection_report = root / "seamgrim.browse_selection_flow_report.detjson"
        age2_close_report = root / "age2_close_report.detjson"
        age3_close_report = root / "age3_close_report.detjson"
        age4_close_report = root / "age4_close_report.detjson"
        age5_close_report = root / "age5_close_report.detjson"
        age4_pack_report = root / "age4_close_pack_report.detjson"
        oi_report = root / "oi405_406_close_report.detjson"
        oi_pack_report = root / "oi405_406_pack_report.detjson"
        write_contract_only_stub_reports(
            seamgrim_report,
            seamgrim_ui_age3_report,
            seamgrim_phase3_cleanup_report,
            seamgrim_browse_selection_report,
            age2_close_report,
            age3_close_report,
            age4_close_report,
            age5_close_report,
            age4_pack_report,
            oi_report,
            oi_pack_report,
        )
        age2_doc = load_json(age2_close_report)
        expect(age2_doc.get("schema") == "ddn.age2_close_report.v1", "age2 close schema mismatch")
        age3_doc = load_json(age3_close_report)
        seamgrim_doc = load_json(seamgrim_report)
        expect(age3_doc.get("schema") == "ddn.seamgrim.age3_close_report.v1", "age3 close schema mismatch")
        expect(
            age3_doc.get("seamgrim_report_path") == str(seamgrim_report),
            "age3 close seamgrim report path mismatch",
        )
        seamgrim_step_names = [str(row.get("name", "")).strip() for row in seamgrim_doc.get("steps", []) if isinstance(row, dict)]
        expect("group_id_summary" in seamgrim_step_names, "contract-only seamgrim report must include group_id_summary step")
        expect(
            "control_exposure_policy" in seamgrim_step_names,
            "contract-only seamgrim report must include control_exposure_policy step",
        )
        age5_doc = load_json(age5_close_report)
        expect(
            age5_doc.get("age5_full_real_w107_golden_index_selftest_active_cases") == "-",
            "contract-only age5 w107 golden index active cases default mismatch",
        )
        expect(
            age5_doc.get("age5_full_real_w107_progress_contract_selftest_progress_present") == "0",
            "contract-only age5 w107 progress present default mismatch",
        )

        age3_close_status_json = root / "age3_close_status.detjson"
        age3_close_status_line = root / "age3_close_status.txt"
        age3_close_badge_json = root / "age3_close_badge.detjson"
        age3_close_summary_md = root / "age3_close_summary.md"
        seamgrim_wasm_cli_diag_parity_report = root / "seamgrim_wasm_cli_diag_parity_report.detjson"
        final_status_line = root / "final_status_line.txt"
        final_status_parse_json = root / "final_status_parse.detjson"
        ci_gate_result_json = root / "ci_gate_result.detjson"
        ci_gate_badge_json = root / "ci_gate_badge.detjson"
        ci_fail_brief_txt = root / "ci_fail_brief.txt"
        ci_fail_triage_json = root / "ci_fail_triage.detjson"
        summary_path = root / "ci_gate_summary.txt"
        summary_line_path = root / "ci_gate_summary_line.txt"
        index_report_path = root / "ci_gate_report_index.detjson"
        write_contract_only_ci_gate_outputs(
            "full",
            ["core_lang", "full", "seamgrim"],
            profile_matrix_report,
            age3_close_status_json,
            age3_close_status_line,
            age3_close_badge_json,
            age3_close_summary_md,
            seamgrim_wasm_cli_diag_parity_report,
            final_status_line,
            final_status_parse_json,
            ci_gate_result_json,
            ci_gate_badge_json,
            ci_fail_brief_txt,
            ci_fail_triage_json,
            summary_path,
            summary_line_path,
            index_report_path,
        )
        expect(
            final_status_line.read_text(encoding="utf-8").strip()
            == "status=pass reason=ok failed_steps=0 aggregate_status=pass overall_ok=1 age4_proof_ok=1 age4_proof_failed_criteria=0 age4_proof_failed_preview=- age4_proof_summary_hash=sha256:contract-only-age4-proof-summary",
            "final status line mismatch",
        )
        expect(load_json(ci_gate_result_json).get("schema") == "ddn.ci.gate_result.v1", "ci gate result schema mismatch")
        expect(load_json(ci_gate_badge_json).get("status") == "pass", "ci gate badge status mismatch")
        expect(
            "profile_matrix_selected_real_profiles=\"core_lang,full,seamgrim\""
            in ci_fail_brief_txt.read_text(encoding="utf-8"),
            "ci fail brief selected profiles marker mismatch",
        )
        triage_doc = load_json(ci_fail_triage_json)
        expect(triage_doc.get("schema") == "ddn.ci.fail_triage.v1", "ci fail triage schema mismatch")
        expect(triage_doc.get("failed_step_detail_rows_count") == 0, "ci fail triage detail rows count mismatch")
        expect(triage_doc.get("failed_step_logs_rows_count") == 0, "ci fail triage logs rows count mismatch")
        expect(triage_doc.get("failed_step_detail_order") == [], "ci fail triage detail order mismatch")
        expect(triage_doc.get("failed_step_logs_order") == [], "ci fail triage logs order mismatch")
        triage_profile_matrix = triage_doc.get("profile_matrix_selftest", {})
        expect(isinstance(triage_profile_matrix, dict), "triage profile matrix missing")
        missing_profile_matrix_keys = profile_matrix_triage_missing_keys(
            triage_profile_matrix if isinstance(triage_profile_matrix, dict) else {}
        )
        expect(
            missing_profile_matrix_keys == [],
            "triage profile matrix required keys missing: "
            + (",".join(missing_profile_matrix_keys) if missing_profile_matrix_keys else "-"),
        )
        expect(
            sorted(triage_profile_matrix.keys()) == sorted(PROFILE_MATRIX_TRIAGE_REQUIRED_KEYS),
            "triage profile matrix key set mismatch",
        )
        expect(
            triage_profile_matrix.get("step_timeout_defaults_text")
            == PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
            "triage profile matrix timeout defaults text mismatch",
        )
        expect(
            triage_profile_matrix.get("step_timeout_defaults_sec")
            == dict(PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC),
            "triage profile matrix timeout defaults sec mismatch",
        )
        expect(
            triage_profile_matrix.get("step_timeout_env_keys")
            == dict(PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS),
            "triage profile matrix timeout env keys mismatch",
        )
        expect(
            triage_profile_matrix.get("aggregate_summary_sanity_checked_profiles") == ["core_lang", "full", "seamgrim"],
            "triage profile matrix checked profiles mismatch",
        )

    print("ci aggregate contract-only lib selftest ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
