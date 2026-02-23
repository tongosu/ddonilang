#!/usr/bin/env python
from __future__ import annotations

import importlib.util
from pathlib import Path


def load_module(root: Path):
    path = root / "tests" / "run_seamgrim_ci_gate.py"
    spec = importlib.util.spec_from_file_location("seamgrim_ci_gate", path)
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

    space2d_diag = mod.extract_diagnostics(
        "space2d_source_ui_gate",
        "check=playground_space2d_source_persistence missing=html:id=\"space2d-source-mode\"",
        "",
        False,
    )
    if not space2d_diag or space2d_diag[0].get("kind") != "space2d_feature_missing":
        print("diagnostics check failed: space2d_source_ui_gate space2d_feature_missing")
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

    print("seamgrim ci gate diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
