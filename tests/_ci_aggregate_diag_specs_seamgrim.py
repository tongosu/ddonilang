from __future__ import annotations

from _ci_aggregate_diag_specs_common import build_identity_field_specs


SEAMGRIM_STEP_DEFAULT_SNAPSHOT = {
    "status": "missing_report",
    "ok": "0",
    "diag_count": "0",
    "detail": "seamgrim report missing or invalid",
}
SEAMGRIM_STEP_SNAPSHOT_FIELD_SPECS = build_identity_field_specs("status", "ok", "diag_count", "detail")
SEAMGRIM_STEP_SUMMARY_LINE_SPECS = build_identity_field_specs("status", "ok", "diag_count", "detail")
SEAMGRIM_FOCUS_STEP_SPECS = (
    ("seamgrim_seed_meta_files", "seed_meta_files"),
    ("seamgrim_seed_overlay_quality", "seed_overlay_quality"),
    ("seamgrim_rewrite_overlay_quality", "rewrite_overlay_quality"),
    ("seamgrim_guideblock_keys_pack", "guideblock_keys_pack"),
    ("seamgrim_moyang_view_boundary_pack", "moyang_view_boundary_pack"),
    ("seamgrim_lesson_warning_tokens", "lesson_warning_tokens"),
    ("seamgrim_stateful_sim_preview_upgrade", "stateful_sim_preview_upgrade"),
    ("seamgrim_pendulum_surface_contract", "pendulum_surface_contract"),
    ("seamgrim_seed_pendulum_export", "seed_pendulum_export"),
    ("seamgrim_pendulum_runtime_visual", "pendulum_runtime_visual"),
    ("seamgrim_seed_runtime_visual_pack", "seed_runtime_visual_pack"),
    ("seamgrim_group_id_summary", "group_id_summary"),
    ("seamgrim_runtime_fallback_metrics", "runtime_fallback_metrics"),
    ("seamgrim_runtime_fallback_policy", "runtime_fallback_policy"),
    ("seamgrim_pendulum_bogae_shape", "pendulum_bogae_shape"),
)
CONTROL_EXPOSURE_POLICY_DEFAULT_SNAPSHOT = {
    "status": "missing_report",
    "step_ok": "0",
    "violation_count": "0",
    "top": "-",
}
CONTROL_EXPOSURE_POLICY_SNAPSHOT_FIELD_SPECS = build_identity_field_specs(
    "status",
    "step_ok",
    "violation_count",
    "top",
)
CONTROL_EXPOSURE_POLICY_SUMMARY_LINE_SPECS = (
    ("seamgrim_control_exposure_policy_status", "status"),
    ("seamgrim_control_exposure_policy_ok", "step_ok"),
    ("seamgrim_control_exposure_policy_violations", "violation_count"),
    ("seamgrim_control_exposure_policy_top", "top"),
)
REWRITE_OVERLAY_REPORT_DEFAULT_SNAPSHOT = {
    "status": "missing_report",
    "ok": "0",
    "violation_count": "0",
    "top": "-",
}
REWRITE_OVERLAY_REPORT_SNAPSHOT_FIELD_SPECS = build_identity_field_specs(
    "status",
    "ok",
    "violation_count",
    "top",
)
REWRITE_OVERLAY_REPORT_SUMMARY_LINE_SPECS = (
    ("seamgrim_rewrite_overlay_quality_violations", "violation_count"),
    ("seamgrim_rewrite_overlay_quality_top", "top"),
)
