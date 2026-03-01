#!/usr/bin/env python
from __future__ import annotations

import importlib.util
from pathlib import Path


def load_module(root: Path):
    path = root / "tests" / "_seamgrim_ci_diag_lib.py"
    spec = importlib.util.spec_from_file_location("seamgrim_ci_diag_lib", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"module load failed: {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    mod = load_module(root)

    full_diag = mod.extract_diagnostics(
        "full_check",
        "graph export failed for pack/abc: E_PARSE_UNEXPECTED_TOKEN\nother",
        "",
        False,
    )
    if not full_diag or full_diag[0].get("kind") != "graph_export_failed":
        print("diagnostics check failed: full_check graph_export_failed")
        return 1

    schema_diag = mod.extract_diagnostics(
        "schema_gate",
        "schema_status.json drift detected. regenerate with lesson_schema_upgrade.py --status-out ...",
        "",
        False,
    )
    if not schema_diag or schema_diag[0].get("kind") != "schema_status_drift":
        print("diagnostics check failed: schema_gate schema_status_drift")
        return 1

    ui_diag = mod.extract_diagnostics(
        "ui_age3_gate",
        "check=r3c_media_export missing=html:id=\"media-export-format\"",
        "",
        False,
    )
    if not ui_diag or ui_diag[0].get("kind") != "age3_feature_missing":
        print("diagnostics check failed: ui_age3_gate age3_feature_missing")
        return 1

    sim_core_diag = mod.extract_diagnostics(
        "sim_core_contract_gate",
        "check=sim_core_removed_nonessential_dom missing=html:class=\"statusbar\"",
        "",
        False,
    )
    if not sim_core_diag or sim_core_diag[0].get("kind") != "sim_core_contract_missing":
        print("diagnostics check failed: sim_core_contract_gate sim_core_contract_missing")
        return 1

    shape_fallback_diag = mod.extract_diagnostics(
        "shape_fallback_mode",
        "check=shape_fallback_mode detail=app_token_missing:allowShapeFallback,",
        "",
        False,
    )
    if not shape_fallback_diag or shape_fallback_diag[0].get("kind") != "shape_fallback_mode_failed":
        print("diagnostics check failed: shape_fallback_mode shape_fallback_mode_failed")
        return 1

    primitive_source_diag = mod.extract_diagnostics(
        "space2d_primitive_source",
        "check=space2d_primitive_source detail=node_runner_failed:shapes mode must fallback to drawlist",
        "",
        False,
    )
    if not primitive_source_diag or primitive_source_diag[0].get("kind") != "space2d_primitive_source_failed":
        print("diagnostics check failed: space2d_primitive_source space2d_primitive_source_failed")
        return 1

    space2d_diag = mod.extract_diagnostics(
        "space2d_source_ui_gate",
        "check=playground_space2d_source_persistence missing=html:id=\"space2d-source-mode\"",
        "",
        False,
    )
    if not space2d_diag or space2d_diag[0].get("kind") != "space2d_feature_missing":
        print("diagnostics check failed: space2d_source_ui_gate space2d_feature_missing")
        return 1

    phase3_diag = mod.extract_diagnostics(
        "phase3_cleanup_gate",
        "check=phase3_app_contract_scaffold_removed missing=forbidden:function buildSessionContractScaffold(",
        "",
        False,
    )
    if not phase3_diag or phase3_diag[0].get("kind") != "phase3_cleanup_missing":
        print("diagnostics check failed: phase3_cleanup_gate phase3_cleanup_missing")
        return 1

    lesson_path_diag = mod.extract_diagnostics(
        "lesson_path_fallback",
        "check=lesson_path_fallback_tokens missing=fetchFirstOkJson(buildCatalogCandidateUrls(\"/lessons/index.json\"))",
        "",
        False,
    )
    if not lesson_path_diag or lesson_path_diag[0].get("kind") != "lesson_path_fallback_missing":
        print("diagnostics check failed: lesson_path_fallback lesson_path_fallback_missing")
        return 1

    new_grammar_diag = mod.extract_diagnostics(
        "new_grammar_no_legacy_control_meta",
        "check=legacy_control_meta_found detail=solutions/seamgrim_ui_mvp/seed_lessons_v1/x/lesson.ddn:3",
        "",
        False,
    )
    if not new_grammar_diag or new_grammar_diag[0].get("kind") != "legacy_control_meta_found":
        print("diagnostics check failed: new_grammar_no_legacy_control_meta legacy_control_meta_found")
        return 1

    seed_meta_diag = mod.extract_diagnostics(
        "seed_meta_files",
        "check=seed_meta_files detail=missing_meta:solutions/seamgrim_ui_mvp/seed_lessons_v1/physics_pendulum_seed_v1",
        "",
        False,
    )
    if not seed_meta_diag or seed_meta_diag[0].get("kind") != "seed_meta_files_failed":
        print("diagnostics check failed: seed_meta_files seed_meta_files_failed")
        return 1

    visual_contract_diag = mod.extract_diagnostics(
        "visual_contract",
        "check=visual_contract detail=rewrite:shape_block_missing:ssot_edu_phys_p001_01_uniform_motion_xt",
        "",
        False,
    )
    if not visual_contract_diag or visual_contract_diag[0].get("kind") != "visual_contract_rewrite_failed":
        print("diagnostics check failed: visual_contract visual_contract_rewrite_failed")
        return 1

    visual_contract_meta_diag = mod.extract_diagnostics(
        "visual_contract",
        "check=visual_contract detail=seed:missing_meta:solutions/seamgrim_ui_mvp/seed_lessons_v1/physics_pendulum_seed_v1",
        "",
        False,
    )
    if not visual_contract_meta_diag or visual_contract_meta_diag[0].get("kind") != "visual_contract_seed_failed":
        print("diagnostics check failed: visual_contract missing_meta visual_contract_seed_failed")
        return 1

    seed_overlay_diag = mod.extract_diagnostics(
        "seed_overlay_quality",
        "check=seed_overlay_quality detail=violations:physics_pendulum_seed_v1:section_bogae_madang",
        "",
        False,
    )
    if not seed_overlay_diag or seed_overlay_diag[0].get("kind") != "seed_overlay_quality_failed":
        print("diagnostics check failed: seed_overlay_quality seed_overlay_quality_failed")
        return 1

    rewrite_overlay_diag = mod.extract_diagnostics(
        "rewrite_overlay_quality",
        "check=rewrite_overlay_quality detail=violations:college_physics_harmonic:too_short",
        "",
        False,
    )
    if not rewrite_overlay_diag or rewrite_overlay_diag[0].get("kind") != "rewrite_overlay_quality_failed":
        print("diagnostics check failed: rewrite_overlay_quality rewrite_overlay_quality_failed")
        return 1

    pendulum_surface_diag = mod.extract_diagnostics(
        "pendulum_surface_contract",
        "check=pendulum_surface_contract detail=tick_show_vars_missing:energy",
        "",
        False,
    )
    if not pendulum_surface_diag or pendulum_surface_diag[0].get("kind") != "pendulum_surface_contract_failed":
        print("diagnostics check failed: pendulum_surface_contract pendulum_surface_contract_failed")
        return 1

    control_exposure_diag = mod.extract_diagnostics(
        "control_exposure_policy",
        "check=control_exposure_policy detail=runner_failed:check=control_exposure_policy_violation detail=chabi_variable_not_exposed:file.ddn:theta",
        "",
        False,
    )
    if not control_exposure_diag or control_exposure_diag[0].get("kind") != "control_exposure_policy_failed":
        print("diagnostics check failed: control_exposure_policy control_exposure_policy_failed")
        return 1

    browse_selection_diag = mod.extract_diagnostics(
        "browse_selection_flow",
        "check=browse_selection_flow detail=selection payload object missing",
        "",
        False,
    )
    if not browse_selection_diag or browse_selection_diag[0].get("kind") != "browse_selection_flow_failed":
        print("diagnostics check failed: browse_selection_flow browse_selection_flow_failed")
        return 1

    browse_selection_report_diag = mod.extract_diagnostics(
        "browse_selection_report",
        "check=browse_selection_report_missing detail=path=build/reports/missing.detjson",
        "",
        False,
    )
    if not browse_selection_report_diag or browse_selection_report_diag[0].get("kind") != "browse_selection_report_invalid":
        print("diagnostics check failed: browse_selection_report browse_selection_report_invalid")
        return 1

    overlay_diag = mod.extract_diagnostics(
        "overlay_compare_pack",
        "check=c02_axis_mismatch_blocked expected_ok=0 actual_ok=1 expected_code=mismatch_xUnit actual_code=ok",
        "",
        False,
    )
    if not overlay_diag or overlay_diag[0].get("kind") != "overlay_compare_case_failed":
        print("diagnostics check failed: overlay_compare_pack overlay_compare_case_failed")
        return 1

    overlay_fail_line_diag = mod.extract_diagnostics(
        "overlay_compare_pack",
        "[FAIL] pack=pack/seamgrim_overlay_param_compare_v0 case=2",
        "",
        False,
    )
    if not overlay_fail_line_diag or overlay_fail_line_diag[0].get("kind") != "overlay_compare_case_failed":
        print("diagnostics check failed: overlay_compare_pack fail-line overlay_compare_case_failed")
        return 1

    overlay_session_diag = mod.extract_diagnostics(
        "overlay_session_pack",
        "check=c03_drop_variant_on_axis_mismatch enabled=1 baseline=run-base variant=- dropped=1 drop_code=mismatch_xUnit",
        "",
        False,
    )
    if not overlay_session_diag or overlay_session_diag[0].get("kind") != "overlay_session_case_failed":
        print("diagnostics check failed: overlay_session_pack overlay_session_case_failed")
        return 1

    overlay_session_diag = mod.extract_diagnostics(
        "overlay_session_contract",
        "overlay session contract failed",
        "",
        False,
    )
    if not overlay_session_diag or overlay_session_diag[0].get("kind") != "overlay_session_contract_failed":
        print("diagnostics check failed: overlay_session_contract overlay_session_contract_failed")
        return 1

    age5_diag = mod.extract_diagnostics(
        "age5_close",
        "[age5-close] overall_ok=0 criteria=13 failed=1 report=build/reports/age5_close_report.detjson\n - s5_detailed_dod_checked: ok=0",
        "",
        False,
    )
    if not age5_diag or age5_diag[0].get("kind") != "age5_close_failed":
        print("diagnostics check failed: age5_close age5_close_failed")
        return 1

    deploy_diag = mod.extract_diagnostics(
        "deploy_artifacts",
        "check=dockerfile_required_tokens missing=/api/health",
        "",
        False,
    )
    if not deploy_diag or deploy_diag[0].get("kind") != "deploy_artifact_mismatch":
        print("diagnostics check failed: deploy_artifacts deploy_artifact_mismatch")
        return 1

    ddn_exec_diag = mod.extract_diagnostics(
        "ddn_exec_server_check",
        "check=wasm_mime_invalid detail=content_type=text/plain",
        "",
        False,
    )
    if not ddn_exec_diag or ddn_exec_diag[0].get("kind") != "ddn_exec_server_check_failed":
        print("diagnostics check failed: ddn_exec_server_check ddn_exec_server_check_failed")
        return 1

    seed_pendulum_export_diag = mod.extract_diagnostics(
        "seed_pendulum_export",
        "check=seed_pendulum_export detail=numbers_too_few:804",
        "",
        False,
    )
    if not seed_pendulum_export_diag or seed_pendulum_export_diag[0].get("kind") != "seed_pendulum_export_failed":
        print("diagnostics check failed: seed_pendulum_export seed_pendulum_export_failed")
        return 1

    pendulum_runtime_visual_diag = mod.extract_diagnostics(
        "pendulum_runtime_visual",
        "check=pendulum_runtime_visual detail=runner_failed:theta_points_too_few:10",
        "",
        False,
    )
    if not pendulum_runtime_visual_diag or pendulum_runtime_visual_diag[0].get("kind") != "pendulum_runtime_visual_failed":
        print("diagnostics check failed: pendulum_runtime_visual pendulum_runtime_visual_failed")
        return 1

    seed_runtime_visual_pack_diag = mod.extract_diagnostics(
        "seed_runtime_visual_pack",
        "check=seed_runtime_visual_pack detail=runner_failed:series_points_too_few:econ_supply_demand_seed_v1:12",
        "",
        False,
    )
    if not seed_runtime_visual_pack_diag or seed_runtime_visual_pack_diag[0].get("kind") != "seed_runtime_visual_pack_failed":
        print("diagnostics check failed: seed_runtime_visual_pack seed_runtime_visual_pack_failed")
        return 1

    runtime_fallback_metrics_diag = mod.extract_diagnostics(
        "runtime_fallback_metrics",
        "check=runtime_fallback_metrics detail=pack_detail_parse_failed",
        "",
        False,
    )
    if not runtime_fallback_metrics_diag or runtime_fallback_metrics_diag[0].get("kind") != "runtime_fallback_metrics_failed":
        print("diagnostics check failed: runtime_fallback_metrics runtime_fallback_metrics_failed")
        return 1

    runtime_fallback_metrics_info = mod.extract_diagnostics(
        "runtime_fallback_metrics",
        "[runtime-fallback] total=5 fallback=5 native=0 ratio=1.000 report=build/reports/seamgrim_runtime_fallback_metrics.detjson",
        "",
        True,
    )
    if not runtime_fallback_metrics_info or runtime_fallback_metrics_info[0].get("kind") != "runtime_fallback_metrics_info":
        print("diagnostics check failed: runtime_fallback_metrics runtime_fallback_metrics_info")
        return 1

    runtime_fallback_policy_diag = mod.extract_diagnostics(
        "runtime_fallback_policy",
        "check=runtime_fallback_policy detail=ratio_exceeds:max=0.200:ratio=1.000:fallback=5:native=0:total=5",
        "",
        False,
    )
    if not runtime_fallback_policy_diag or runtime_fallback_policy_diag[0].get("kind") != "runtime_fallback_policy_failed":
        print("diagnostics check failed: runtime_fallback_policy runtime_fallback_policy_failed")
        return 1

    runtime_fallback_policy_info = mod.extract_diagnostics(
        "runtime_fallback_policy",
        "[runtime-fallback-policy] status=pass max_ratio=0.200 ratio=0.200 fallback=1 native=4 total=5",
        "",
        True,
    )
    if not runtime_fallback_policy_info or runtime_fallback_policy_info[0].get("kind") != "runtime_fallback_policy_info":
        print("diagnostics check failed: runtime_fallback_policy runtime_fallback_policy_info")
        return 1

    pendulum_bogae_diag = mod.extract_diagnostics(
        "pendulum_bogae_shape",
        "check=pendulum_bogae_shape detail=node_runner_failed:error",
        "",
        False,
    )
    if not pendulum_bogae_diag or pendulum_bogae_diag[0].get("kind") != "pendulum_bogae_shape_failed":
        print("diagnostics check failed: pendulum_bogae_shape pendulum_bogae_shape_failed")
        return 1

    runtime_5min_diag = mod.extract_diagnostics(
        "runtime_5min",
        "runtime 5min check failed: ddn_exec_server_check, lesson_path_fallback",
        "",
        False,
    )
    if not runtime_5min_diag or runtime_5min_diag[0].get("kind") != "runtime_5min_failed":
        print("diagnostics check failed: runtime_5min runtime_5min_failed")
        return 1

    runtime_5min_checklist_diag = mod.extract_diagnostics(
        "runtime_5min_checklist",
        "seamgrim 5min checklist failed",
        "",
        False,
    )
    if not runtime_5min_checklist_diag or runtime_5min_checklist_diag[0].get("kind") != "runtime_5min_checklist_failed":
        print("diagnostics check failed: runtime_5min_checklist runtime_5min_checklist_failed")
        return 1

    workflow_diag = mod.extract_diagnostics(
        "workflow_contract",
        "check=branch_workflow_context_mismatch expected=seamgrim-gate actual=seamgrim-gate,extra",
        "",
        False,
    )
    if not workflow_diag or workflow_diag[0].get("kind") != "workflow_contract_mismatch":
        print("diagnostics check failed: workflow_contract workflow_contract_mismatch")
        return 1

    formula_diag = mod.extract_diagnostics(
        "formula_compat",
        "check=formula_function_call file=solutions/seamgrim_ui_mvp/seed_lessons_v1/physics_pendulum_seed_v1/lesson.ddn:40 expr=theta0*cos(wn*t)",
        "",
        False,
    )
    if not formula_diag or formula_diag[0].get("kind") != "formula_incompat":
        print("diagnostics check failed: formula_compat formula_incompat")
        return 1

    formula_fail_diag = mod.extract_diagnostics(
        "formula_compat",
        "seamgrim formula compat check failed: 1 issue(s)",
        "",
        False,
    )
    if not formula_fail_diag or formula_fail_diag[0].get("kind") != "formula_compat_failed":
        print("diagnostics check failed: formula_compat formula_compat_failed")
        return 1

    realign_diag = mod.extract_diagnostics(
        "schema_realign_formula_compat",
        "schema realign compat check failed: theta line not rewritten",
        "",
        False,
    )
    if not realign_diag or realign_diag[0].get("kind") != "schema_realign_formula_compat_failed":
        print("diagnostics check failed: schema_realign_formula_compat schema_realign_formula_compat_failed")
        return 1

    upgrade_diag = mod.extract_diagnostics(
        "schema_upgrade_formula_compat",
        "schema upgrade formula compat check failed: theta line not rewritten",
        "",
        False,
    )
    if not upgrade_diag or upgrade_diag[0].get("kind") != "schema_upgrade_formula_compat_failed":
        print("diagnostics check failed: schema_upgrade_formula_compat schema_upgrade_formula_compat_failed")
        return 1

    generic_diag = mod.extract_diagnostics("unknown", "line1\nline2", "", False)
    if not generic_diag or generic_diag[0].get("kind") != "generic_error":
        print("diagnostics check failed: generic_error fallback")
        return 1

    failure_digest = mod.build_failure_digest(
        [
            {
                "name": "formula_compat",
                "ok": False,
                "diagnostics": [
                    {
                        "kind": "formula_incompat",
                        "target": "seamgrim_formula",
                        "detail": "check=formula_function_call",
                    }
                ],
            },
            {
                "name": "visual_contract",
                "ok": False,
                "diagnostics": [
                    {
                        "kind": "visual_contract_seed_failed",
                        "target": "visual_contract_seed",
                        "detail": "seed:missing_meta:solutions/seamgrim_ui_mvp/seed_lessons_v1/physics_pendulum_seed_v1",
                    }
                ],
            },
            {
                "name": "visual_contract",
                "ok": False,
                "diagnostics": [
                    {
                        "kind": "visual_contract_rewrite_failed",
                        "target": "visual_contract_rewrite",
                        "detail": "rewrite:shape_block_missing:ssot_edu_phys_p001_01_uniform_motion_xt",
                    }
                ],
            },
        ],
        limit=3,
    )
    if len(failure_digest) != 3:
        print("diagnostics check failed: build_failure_digest length")
        return 1
    if "kind=visual_contract_rewrite_failed" not in failure_digest[0]:
        print("diagnostics check failed: build_failure_digest rewrite priority")
        return 1
    if "kind=visual_contract_seed_failed" not in failure_digest[1]:
        print("diagnostics check failed: build_failure_digest seed priority")
        return 1

    print("seamgrim ci gate diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
