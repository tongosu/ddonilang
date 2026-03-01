from __future__ import annotations

import re


def extract_diagnostics(name: str, stdout: str, stderr: str, ok: bool) -> list[dict[str, str]]:
    lines = [line.strip() for line in (stdout + "\n" + stderr).splitlines() if line.strip()]
    out: list[dict[str, str]] = []

    if name == "full_check":
        graph_export = re.compile(r"^graph export failed for (.+?):\s*(.+)$")
        graph_mismatch = re.compile(r"^graph json mismatch:\s*(.+)$")
        for line in lines:
            m1 = graph_export.match(line)
            if m1:
                out.append(
                    {
                        "kind": "graph_export_failed",
                        "target": m1.group(1).strip(),
                        "detail": m1.group(2).strip(),
                    }
                )
                continue
            m2 = graph_mismatch.match(line)
            if m2:
                out.append(
                    {
                        "kind": "graph_json_mismatch",
                        "target": m2.group(1).strip(),
                        "detail": line,
                    }
                )
                continue
    elif name == "schema_gate":
        for line in lines:
            if line.startswith("missing status file:"):
                out.append({"kind": "missing_status_file", "target": "schema_status", "detail": line})
            elif "schema_status.json drift detected" in line:
                out.append({"kind": "schema_status_drift", "target": "schema_status", "detail": line})
            elif "promote report has pending source updates" in line:
                out.append({"kind": "promote_pending", "target": "lessons", "detail": line})
            elif "promote report has missing preview" in line:
                out.append({"kind": "missing_preview", "target": "lessons", "detail": line})
            elif line.startswith("non-age3 profiles found"):
                out.append({"kind": "non_age3_profile", "target": "lessons", "detail": line})
            elif line.startswith("committed schema has non-age3 profiles"):
                out.append({"kind": "non_age3_profile", "target": "schema_status", "detail": line})
            elif line.startswith("committed schema has lesson without preview"):
                out.append({"kind": "missing_preview", "target": "schema_status", "detail": line})
    elif name == "ui_age3_gate":
        for line in lines:
            if line.startswith("missing ui html:"):
                out.append({"kind": "ui_html_missing", "target": "ui/index.html", "detail": line})
            elif line.startswith("missing ui js:"):
                out.append({"kind": "ui_js_missing", "target": "ui/app.js", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "age3_feature_missing", "target": "age3_ui", "detail": line})
            elif line.startswith("age3 ui gate failed:"):
                out.append({"kind": "age3_gate_failed", "target": "age3_ui", "detail": line})
    elif name == "sim_core_contract_gate":
        for line in lines:
            if line.startswith("missing file:"):
                out.append({"kind": "sim_core_file_missing", "target": "sim_core_contract", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "sim_core_contract_missing", "target": "sim_core_contract", "detail": line})
            elif line.startswith("seamgrim sim core contract gate failed"):
                out.append({"kind": "sim_core_contract_failed", "target": "sim_core_contract", "detail": line})
    elif name == "shape_fallback_mode":
        for line in lines:
            if line.startswith("check=shape_fallback_mode"):
                out.append({"kind": "shape_fallback_mode_failed", "target": "shape_fallback_mode", "detail": line})
            elif line.startswith("seamgrim shape fallback mode check failed"):
                out.append({"kind": "shape_fallback_mode_failed", "target": "shape_fallback_mode", "detail": line})
    elif name == "space2d_primitive_source":
        for line in lines:
            if line.startswith("check=space2d_primitive_source"):
                out.append(
                    {
                        "kind": "space2d_primitive_source_failed",
                        "target": "space2d_primitive_source",
                        "detail": line,
                    }
                )
            elif line.startswith("seamgrim space2d primitive source check failed"):
                out.append(
                    {
                        "kind": "space2d_primitive_source_failed",
                        "target": "space2d_primitive_source",
                        "detail": line,
                    }
                )
    elif name == "space2d_source_ui_gate":
        for line in lines:
            if line.startswith("missing ui file:"):
                out.append({"kind": "space2d_ui_file_missing", "target": "playground_or_smoke_ui", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "space2d_feature_missing", "target": "space2d_source_ui", "detail": line})
            elif line.startswith("space2d source ui gate failed:"):
                out.append({"kind": "space2d_gate_failed", "target": "space2d_source_ui", "detail": line})
    elif name == "phase3_cleanup_gate":
        for line in lines:
            if line.startswith("missing ui file:"):
                out.append({"kind": "phase3_ui_file_missing", "target": "phase3_cleanup", "detail": line})
            elif line.startswith("missing ui root:"):
                out.append({"kind": "phase3_ui_root_missing", "target": "phase3_cleanup", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "phase3_cleanup_missing", "target": "phase3_cleanup", "detail": line})
            elif line.startswith("phase3 cleanup gate failed:"):
                out.append({"kind": "phase3_cleanup_failed", "target": "phase3_cleanup", "detail": line})
    elif name == "lesson_path_fallback":
        for line in lines:
            if line.startswith("missing ui js:"):
                out.append({"kind": "ui_js_missing", "target": "ui/app.js", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "lesson_path_fallback_missing", "target": "lesson_path_fallback", "detail": line})
            elif line.startswith("seamgrim lesson path fallback check failed:"):
                out.append({"kind": "lesson_path_fallback_failed", "target": "lesson_path_fallback", "detail": line})
    elif name == "new_grammar_no_legacy_control_meta":
        for line in lines:
            if line.startswith("check=legacy_control_meta_found"):
                out.append(
                    {
                        "kind": "legacy_control_meta_found",
                        "target": "new_grammar_no_legacy_control_meta",
                        "detail": line,
                    }
                )
    elif name == "seed_meta_files":
        for line in lines:
            if line.startswith("check=seed_meta_files"):
                out.append(
                    {
                        "kind": "seed_meta_files_failed",
                        "target": "seed_meta_files",
                        "detail": line,
                    }
                )
    elif name == "visual_contract":
        for line in lines:
            if line.startswith("check=visual_contract"):
                detail_prefix = "check=visual_contract detail="
                detail_value = line[len(detail_prefix) :].strip() if line.startswith(detail_prefix) else ""
                parsed = False
                if detail_value:
                    for item in [part.strip() for part in detail_value.split(";") if part.strip()]:
                        if item.startswith("rewrite:"):
                            out.append(
                                {
                                    "kind": "visual_contract_rewrite_failed",
                                    "target": "visual_contract_rewrite",
                                    "detail": item,
                                }
                            )
                            parsed = True
                        elif item.startswith("seed:"):
                            out.append(
                                {
                                    "kind": "visual_contract_seed_failed",
                                    "target": "visual_contract_seed",
                                    "detail": item,
                                }
                            )
                            parsed = True
                if parsed:
                    continue
                out.append(
                    {
                        "kind": "visual_contract_failed",
                        "target": "visual_contract",
                        "detail": line,
                    }
                )
            elif line.startswith("seamgrim visual contract check failed"):
                out.append(
                    {
                        "kind": "visual_contract_failed",
                        "target": "visual_contract",
                        "detail": line,
                    }
                )
    elif name == "seed_overlay_quality":
        for line in lines:
            if line.startswith("check=seed_overlay_quality"):
                out.append(
                    {
                        "kind": "seed_overlay_quality_failed",
                        "target": "seed_overlay_quality",
                        "detail": line,
                    }
                )
    elif name == "rewrite_overlay_quality":
        for line in lines:
            if line.startswith("check=rewrite_overlay_quality"):
                out.append(
                    {
                        "kind": "rewrite_overlay_quality_failed",
                        "target": "rewrite_overlay_quality",
                        "detail": line,
                    }
                )
    elif name == "pendulum_surface_contract":
        for line in lines:
            if line.startswith("check=pendulum_surface_contract"):
                out.append(
                    {
                        "kind": "pendulum_surface_contract_failed",
                        "target": "pendulum_surface_contract",
                        "detail": line,
                    }
                )
            elif line.startswith("seamgrim pendulum surface contract check failed"):
                out.append(
                    {
                        "kind": "pendulum_surface_contract_failed",
                        "target": "pendulum_surface_contract",
                        "detail": line,
                    }
                )
    elif name == "control_exposure_policy":
        for line in lines:
            if line.startswith("check=control_exposure_policy"):
                out.append(
                    {
                        "kind": "control_exposure_policy_failed",
                        "target": "control_exposure_policy",
                        "detail": line,
                    }
                )
            elif line.startswith("seamgrim control exposure policy check failed"):
                out.append(
                    {
                        "kind": "control_exposure_policy_failed",
                        "target": "control_exposure_policy",
                        "detail": line,
                    }
                )
            elif line.startswith("seamgrim new grammar check failed:"):
                out.append(
                    {
                        "kind": "new_grammar_check_failed",
                        "target": "new_grammar_no_legacy_control_meta",
                        "detail": line,
                    }
                )
    elif name == "browse_selection_flow":
        for line in lines:
            if line.startswith("check="):
                out.append({"kind": "browse_selection_flow_failed", "target": "browse_selection_flow", "detail": line})
            elif line.startswith("seamgrim browse selection flow check failed"):
                out.append({"kind": "browse_selection_flow_failed", "target": "browse_selection_flow", "detail": line})
    elif name == "browse_selection_report":
        for line in lines:
            if line.startswith("check="):
                out.append({"kind": "browse_selection_report_invalid", "target": "browse_selection_report", "detail": line})
            elif line.startswith("seamgrim browse selection report check failed"):
                out.append({"kind": "browse_selection_report_invalid", "target": "browse_selection_report", "detail": line})
    elif name == "overlay_compare_pack":
        for line in lines:
            if line.startswith("missing pack root:"):
                out.append({"kind": "overlay_pack_root_missing", "target": "overlay_compare_pack", "detail": line})
            elif line.startswith("missing pack case:"):
                out.append({"kind": "overlay_pack_case_missing", "target": "overlay_compare_pack", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "overlay_compare_case_failed", "target": "overlay_compare_pack", "detail": line})
            elif line.startswith("[FAIL] pack="):
                out.append({"kind": "overlay_compare_case_failed", "target": "overlay_compare_pack", "detail": line})
            elif line.startswith("overlay compare pack failed:"):
                out.append({"kind": "overlay_compare_pack_failed", "target": "overlay_compare_pack", "detail": line})
    elif name == "overlay_session_pack":
        for line in lines:
            if line.startswith("missing pack root:"):
                out.append({"kind": "overlay_session_pack_root_missing", "target": "overlay_session_pack", "detail": line})
            elif line.startswith("missing pack case:"):
                out.append({"kind": "overlay_session_pack_case_missing", "target": "overlay_session_pack", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "overlay_session_case_failed", "target": "overlay_session_pack", "detail": line})
            elif line.startswith("[FAIL] pack="):
                out.append({"kind": "overlay_session_case_failed", "target": "overlay_session_pack", "detail": line})
            elif line.startswith("overlay session pack failed:"):
                out.append({"kind": "overlay_session_pack_failed", "target": "overlay_session_pack", "detail": line})
    elif name == "overlay_session_contract":
        for line in lines:
            if line.startswith("overlay session contract failed"):
                out.append({"kind": "overlay_session_contract_failed", "target": "overlay_session_contract", "detail": line})
            elif line.startswith("[overlay-session-contract]"):
                out.append({"kind": "overlay_session_contract_log", "target": "overlay_session_contract", "detail": line})
    elif name == "age5_close":
        for line in lines:
            if line.startswith("[age5-close] overall_ok=0"):
                out.append({"kind": "age5_close_failed", "target": "age5_close", "detail": line})
            elif line.startswith(" - ") and "ok=0" in line:
                out.append({"kind": "age5_criteria_failed", "target": "age5_close", "detail": line})
    elif name == "export_graph_preprocess":
        for line in lines:
            if line.startswith("seamgrim export_graph preprocess check failed:"):
                out.append({"kind": "preprocess_check_failed", "target": "export_graph", "detail": line})
    elif name == "deploy_artifacts":
        for line in lines:
            if line.startswith("missing deploy file:"):
                out.append({"kind": "deploy_file_missing", "target": "deploy_artifacts", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "deploy_artifact_mismatch", "target": "deploy_artifacts", "detail": line})
            elif line.startswith("seamgrim deploy artifacts check failed:"):
                out.append({"kind": "deploy_check_failed", "target": "deploy_artifacts", "detail": line})
    elif name == "ddn_exec_server_check":
        for line in lines:
            if line.startswith("check="):
                out.append({"kind": "ddn_exec_server_check_failed", "target": "ddn_exec_server_check", "detail": line})
            elif line.startswith("ddn exec server failed to start"):
                out.append({"kind": "ddn_exec_server_start_failed", "target": "ddn_exec_server_check", "detail": line})
    elif name == "seed_pendulum_export":
        for line in lines:
            if line.startswith("check=seed_pendulum_export"):
                out.append({"kind": "seed_pendulum_export_failed", "target": "seed_pendulum_export", "detail": line})
            elif line.startswith("seed pendulum runtime check failed"):
                out.append({"kind": "seed_pendulum_export_failed", "target": "seed_pendulum_export", "detail": line})
    elif name == "pendulum_runtime_visual":
        for line in lines:
            if line.startswith("check=pendulum_runtime_visual"):
                out.append(
                    {"kind": "pendulum_runtime_visual_failed", "target": "pendulum_runtime_visual", "detail": line}
                )
            elif line.startswith("seamgrim pendulum runtime visual check failed"):
                out.append(
                    {"kind": "pendulum_runtime_visual_failed", "target": "pendulum_runtime_visual", "detail": line}
                )
    elif name == "seed_runtime_visual_pack":
        for line in lines:
            if line.startswith("check=seed_runtime_visual_pack"):
                out.append(
                    {"kind": "seed_runtime_visual_pack_failed", "target": "seed_runtime_visual_pack", "detail": line}
                )
            elif line.startswith("seamgrim seed runtime visual pack check failed"):
                out.append(
                    {"kind": "seed_runtime_visual_pack_failed", "target": "seed_runtime_visual_pack", "detail": line}
                )
    elif name == "runtime_fallback_metrics":
        for line in lines:
            if line.startswith("check=runtime_fallback_metrics"):
                out.append(
                    {"kind": "runtime_fallback_metrics_failed", "target": "runtime_fallback_metrics", "detail": line}
                )
            elif line.startswith("[runtime-fallback]"):
                out.append(
                    {"kind": "runtime_fallback_metrics_info", "target": "runtime_fallback_metrics", "detail": line}
                )
            elif line.startswith("seamgrim runtime fallback metrics check failed"):
                out.append(
                    {"kind": "runtime_fallback_metrics_failed", "target": "runtime_fallback_metrics", "detail": line}
                )
    elif name == "runtime_fallback_policy":
        for line in lines:
            if line.startswith("check=runtime_fallback_policy"):
                out.append(
                    {"kind": "runtime_fallback_policy_failed", "target": "runtime_fallback_policy", "detail": line}
                )
            elif line.startswith("[runtime-fallback-policy]"):
                out.append(
                    {"kind": "runtime_fallback_policy_info", "target": "runtime_fallback_policy", "detail": line}
                )
            elif line.startswith("seamgrim runtime fallback policy check failed"):
                out.append(
                    {"kind": "runtime_fallback_policy_failed", "target": "runtime_fallback_policy", "detail": line}
                )
    elif name == "pendulum_bogae_shape":
        for line in lines:
            if line.startswith("check=pendulum_bogae_shape"):
                out.append({"kind": "pendulum_bogae_shape_failed", "target": "pendulum_bogae_shape", "detail": line})
            elif line.startswith("seamgrim pendulum bogae fallback runner ok"):
                continue
            elif line.startswith("seamgrim pendulum bogae shape check failed"):
                out.append({"kind": "pendulum_bogae_shape_failed", "target": "pendulum_bogae_shape", "detail": line})
    elif name == "runtime_5min":
        for line in lines:
            if line.startswith("runtime 5min check failed:"):
                out.append({"kind": "runtime_5min_failed", "target": "runtime_5min", "detail": line})
            elif line.startswith("[runtime-5min] ok=0"):
                out.append({"kind": "runtime_5min_failed", "target": "runtime_5min", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "runtime_5min_subcheck_failed", "target": "runtime_5min", "detail": line})
    elif name == "runtime_5min_checklist":
        for line in lines:
            if line.startswith("check=seamgrim_5min_checklist"):
                out.append({"kind": "runtime_5min_checklist_invalid", "target": "runtime_5min_checklist", "detail": line})
            elif line.startswith("seamgrim 5min checklist failed"):
                out.append({"kind": "runtime_5min_checklist_failed", "target": "runtime_5min_checklist", "detail": line})
    elif name == "workflow_contract":
        for line in lines:
            if line.startswith("missing workflow file:"):
                out.append({"kind": "workflow_file_missing", "target": "workflow_contract", "detail": line})
            elif line.startswith("missing branch protection file:"):
                out.append({"kind": "branch_protection_file_missing", "target": "workflow_contract", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "workflow_contract_mismatch", "target": "workflow_contract", "detail": line})
            elif line.startswith("seamgrim workflow contract check failed:"):
                out.append({"kind": "workflow_contract_failed", "target": "workflow_contract", "detail": line})
    elif name == "formula_compat":
        for line in lines:
            if line.startswith("missing target root:"):
                out.append({"kind": "formula_scope_root_missing", "target": "seamgrim_formula", "detail": line})
            elif line.startswith("no lesson files found under target:"):
                out.append({"kind": "formula_scope_files_missing", "target": "seamgrim_formula", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "formula_incompat", "target": "seamgrim_formula", "detail": line})
            elif line.startswith("seamgrim formula compat check failed:"):
                out.append({"kind": "formula_compat_failed", "target": "seamgrim_formula", "detail": line})
    elif name == "schema_realign_formula_compat":
        for line in lines:
            if line.startswith("schema realign compat check failed:"):
                out.append({"kind": "schema_realign_formula_compat_failed", "target": "lesson_schema_realign", "detail": line})
    elif name == "schema_upgrade_formula_compat":
        for line in lines:
            if line.startswith("schema upgrade formula compat check failed:"):
                out.append({"kind": "schema_upgrade_formula_compat_failed", "target": "lesson_schema_upgrade", "detail": line})

    if not out and not ok:
        for line in lines[:5]:
            out.append({"kind": "generic_error", "target": name, "detail": line})
    return out


def build_failure_digest(steps: list[dict[str, object]], limit: int = 8) -> list[str]:
    priority_map = {
        "visual_contract_rewrite_failed": 0,
        "visual_contract_seed_failed": 1,
        "visual_contract_failed": 2,
    }
    rows: list[tuple[int, int, str]] = []
    failed_idx = 0
    for step in steps:
        if bool(step.get("ok", False)):
            continue
        name = str(step.get("name", "-"))
        rank = 99
        diagnostics = step.get("diagnostics")
        if isinstance(diagnostics, list) and diagnostics:
            first = diagnostics[0] if isinstance(diagnostics[0], dict) else {}
            kind = str(first.get("kind", "generic_error"))
            rank = int(priority_map.get(kind, 99))
            target = str(first.get("target", "-"))
            detail = str(first.get("detail", "")).strip()
            detail = " ".join(detail.split())
            if len(detail) > 120:
                detail = detail[:120] + "..."
            row = f"step={name} kind={kind} target={target}"
            if detail:
                row += f" detail={detail}"
            rows.append((rank, failed_idx, row))
        else:
            stderr = str(step.get("stderr") or "").strip()
            stdout = str(step.get("stdout") or "").strip()
            detail = stderr or stdout
            detail = " ".join(detail.split())
            if len(detail) > 120:
                detail = detail[:120] + "..."
            row = f"step={name}"
            if detail:
                row += f" detail={detail}"
            rows.append((rank, failed_idx, row))
        failed_idx += 1

    rows.sort(key=lambda item: (item[0], item[1]))
    out = [item[2] for item in rows[:limit]]
    return out
