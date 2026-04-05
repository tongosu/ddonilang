from __future__ import annotations

from _ci_aggregate_diag_specs_common import (
    build_default_snapshot_from_typed_specs,
    build_identity_field_specs,
)
from _ci_profile_matrix_selftest_lib import (
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
    PROFILE_MATRIX_SUMMARY_VALUE_KEYS,
)


PROFILE_MATRIX_SNAPSHOT_FIELD_SPECS = (
    ("status", "text", "status", "missing_report"),
    ("ok", "bool_text", "ok", "0"),
    ("total_elapsed_ms", "int_text", "total_elapsed_ms", "-"),
    ("selected_real_profiles", "names_text", "selected_real_profiles", "-"),
    ("skipped_real_profiles", "names_text", "skipped_real_profiles", "-"),
    ("step_timeout_defaults_text", "text", "step_timeout_defaults_text", PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT),
    ("core_lang_elapsed_ms", "elapsed_text", "core_lang", "-"),
    ("full_elapsed_ms", "elapsed_text", "full", "-"),
    ("seamgrim_elapsed_ms", "elapsed_text", "seamgrim", "-"),
)
PROFILE_MATRIX_AGGREGATE_SUMMARY_VALUE_KEYS = PROFILE_MATRIX_SUMMARY_VALUE_KEYS
PROFILE_MATRIX_DEFAULT_SNAPSHOT = build_default_snapshot_from_typed_specs(PROFILE_MATRIX_SNAPSHOT_FIELD_SPECS)
PROFILE_MATRIX_SUMMARY_LINE_SPECS = (
    ("ci_profile_matrix_gate_selftest_status", "status"),
    ("ci_profile_matrix_gate_selftest_ok", "ok"),
    ("ci_profile_matrix_gate_selftest_total_elapsed_ms", "total_elapsed_ms"),
    ("ci_profile_matrix_gate_selftest_selected_real_profiles", "selected_real_profiles"),
    ("ci_profile_matrix_gate_selftest_skipped_real_profiles", "skipped_real_profiles"),
    ("ci_profile_matrix_gate_selftest_step_timeout_defaults", "step_timeout_defaults_text"),
    ("ci_profile_matrix_gate_selftest_core_lang_elapsed_ms", "core_lang_elapsed_ms"),
    ("ci_profile_matrix_gate_selftest_full_elapsed_ms", "full_elapsed_ms"),
    ("ci_profile_matrix_gate_selftest_seamgrim_elapsed_ms", "seamgrim_elapsed_ms"),
    ("ci_profile_matrix_gate_selftest_aggregate_summary_sanity_ok", "aggregate_summary_sanity_ok"),
    ("ci_profile_matrix_gate_selftest_aggregate_summary_checked_profiles", "aggregate_summary_sanity_checked_profiles"),
    ("ci_profile_matrix_gate_selftest_aggregate_summary_failed_profiles", "aggregate_summary_sanity_failed_profiles"),
    ("ci_profile_matrix_gate_selftest_aggregate_summary_skipped_profiles", "aggregate_summary_sanity_skipped_profiles"),
    ("ci_profile_matrix_gate_selftest_core_lang_aggregate_summary_status", "core_lang_aggregate_summary_status"),
    ("ci_profile_matrix_gate_selftest_core_lang_aggregate_summary_ok", "core_lang_aggregate_summary_ok"),
    ("ci_profile_matrix_gate_selftest_core_lang_aggregate_summary_values", "core_lang_aggregate_summary_values"),
    ("ci_profile_matrix_gate_selftest_full_aggregate_summary_status", "full_aggregate_summary_status"),
    ("ci_profile_matrix_gate_selftest_full_aggregate_summary_ok", "full_aggregate_summary_ok"),
    ("ci_profile_matrix_gate_selftest_full_aggregate_summary_values", "full_aggregate_summary_values"),
    ("ci_profile_matrix_gate_selftest_seamgrim_aggregate_summary_status", "seamgrim_aggregate_summary_status"),
    ("ci_profile_matrix_gate_selftest_seamgrim_aggregate_summary_ok", "seamgrim_aggregate_summary_ok"),
    ("ci_profile_matrix_gate_selftest_seamgrim_aggregate_summary_values", "seamgrim_aggregate_summary_values"),
)
RUNTIME5_CHECKLIST_SUMMARY_LINE_SPECS = (
    ("seamgrim_5min_checklist_ok", "ok"),
    ("seamgrim_runtime_5min_rewrite_motion_projectile", "rewrite_ok"),
    ("seamgrim_runtime_5min_rewrite_elapsed_ms", "rewrite_elapsed_ms"),
    ("seamgrim_runtime_5min_rewrite_status", "rewrite_status"),
    ("seamgrim_runtime_5min_moyang_view_boundary", "moyang_ok"),
    ("seamgrim_runtime_5min_moyang_elapsed_ms", "moyang_elapsed_ms"),
    ("seamgrim_runtime_5min_moyang_status", "moyang_status"),
    ("seamgrim_runtime_5min_pendulum_tetris_showcase", "showcase_ok"),
    ("seamgrim_runtime_5min_pendulum_tetris_showcase_elapsed_ms", "showcase_elapsed_ms"),
    ("seamgrim_runtime_5min_pendulum_tetris_showcase_status", "showcase_status"),
)
RUNTIME5_CHECKLIST_SNAPSHOT_FIELD_SPECS = build_identity_field_specs(
    "ok",
    "rewrite_ok",
    "rewrite_elapsed_ms",
    "rewrite_status",
    "moyang_ok",
    "moyang_elapsed_ms",
    "moyang_status",
    "showcase_ok",
    "showcase_elapsed_ms",
    "showcase_status",
)
RUNTIME5_CHECKLIST_ROW_SPECS = (
    ("rewrite", "rewrite_motion_projectile_fallback"),
    ("moyang", "moyang_view_boundary_pack_check"),
    ("showcase", "pendulum_tetris_showcase_check"),
)
RUNTIME5_CHECKLIST_DEFAULT_SNAPSHOT = {
    "ok": "0",
    "rewrite_ok": "na",
    "rewrite_elapsed_ms": "-",
    "rewrite_status": "missing_report",
    "moyang_ok": "na",
    "moyang_elapsed_ms": "-",
    "moyang_status": "missing_report",
    "showcase_ok": "na",
    "showcase_elapsed_ms": "-",
    "showcase_status": "missing_report",
}
RUNTIME5_CHECKLIST_ITEMS_MISSING_SNAPSHOT = {
    "rewrite_ok": "na",
    "rewrite_elapsed_ms": "-",
    "rewrite_status": "items_missing",
    "moyang_ok": "na",
    "moyang_elapsed_ms": "-",
    "moyang_status": "items_missing",
    "showcase_ok": "na",
    "showcase_elapsed_ms": "-",
    "showcase_status": "items_missing",
}
