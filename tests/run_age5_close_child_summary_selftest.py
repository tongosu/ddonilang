#!/usr/bin/env python
from __future__ import annotations

import importlib.util
from pathlib import Path


def load_age5_close_module(root: Path):
    path = root / "tests" / "run_age5_close.py"
    spec = importlib.util.spec_from_file_location("age5_close_mod", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"module load failed: {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def fail(detail: str) -> int:
    print(f"[age5-close-child-summary-selftest] fail: {detail}")
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    mod = load_age5_close_module(root)
    expected_default_text = (
        "age5_combined_heavy_full_real_status=skipped|"
        "age5_combined_heavy_runtime_helper_negative_status=skipped|"
        "age5_combined_heavy_group_id_summary_negative_status=skipped"
    )
    expected_digest_default_field = {"age5_close_digest_selftest_ok": "0"}

    base_criteria = [
        {"name": "age5_ci_profile_matrix_full_real_smoke_optin_pass", "ok": True},
        {"name": "age5_ci_profile_core_lang_runtime_helper_negative_optin_pass", "ok": False},
        {"name": "age5_ci_profile_core_lang_group_id_summary_negative_optin_pass", "ok": True},
    ]

    report = mod.build_age5_close_report(
        strict=False,
        with_profile_matrix_full_real_smoke_check=True,
        with_runtime_helper_mismatch_negative_check=True,
        with_group_id_summary_mismatch_negative_check=False,
        with_combined_heavy_runtime_helper_check=False,
        combined_heavy_env_enabled=False,
        criteria=base_criteria,
        failure_digest=[],
        pending_items=[],
        repair={},
    )
    if str(report.get("age5_combined_heavy_full_real_status", "")).strip() != "pass":
        return fail("full_real_status_pass_mismatch")
    if str(report.get("age5_combined_heavy_runtime_helper_negative_status", "")).strip() != "fail":
        return fail("runtime_helper_negative_status_fail_mismatch")
    if str(report.get("age5_combined_heavy_group_id_summary_negative_status", "")).strip() != "skipped":
        return fail("group_id_summary_negative_status_skipped_mismatch")
    if str(report.get("ci_sanity_age5_combined_heavy_child_summary_default_fields", "")).strip() != expected_default_text:
        return fail("ci_sanity_child_summary_default_fields_mismatch")
    if (
        str(report.get("ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields", "")).strip()
        != expected_default_text
    ):
        return fail("ci_sync_child_summary_default_fields_mismatch")
    if str(report.get("combined_digest_selftest_default_field_text", "")).strip() != "age5_close_digest_selftest_ok=0":
        return fail("combined_digest_selftest_default_field_text_mismatch")
    if dict(report.get("combined_digest_selftest_default_field", {})) != expected_digest_default_field:
        return fail("combined_digest_selftest_default_field_mismatch")

    skipped_report = mod.build_age5_close_report(
        strict=False,
        with_profile_matrix_full_real_smoke_check=False,
        with_runtime_helper_mismatch_negative_check=False,
        with_group_id_summary_mismatch_negative_check=False,
        with_combined_heavy_runtime_helper_check=False,
        combined_heavy_env_enabled=False,
        criteria=[],
        failure_digest=[],
        pending_items=[],
        repair={},
    )
    if str(skipped_report.get("age5_combined_heavy_full_real_status", "")).strip() != "skipped":
        return fail("all_skipped_full_real_mismatch")
    if str(skipped_report.get("age5_combined_heavy_runtime_helper_negative_status", "")).strip() != "skipped":
        return fail("all_skipped_runtime_helper_mismatch")
    if str(skipped_report.get("age5_combined_heavy_group_id_summary_negative_status", "")).strip() != "skipped":
        return fail("all_skipped_group_id_mismatch")
    if str(skipped_report.get("ci_sanity_age5_combined_heavy_child_summary_default_fields", "")).strip() != expected_default_text:
        return fail("all_skipped_ci_sanity_child_summary_default_fields_mismatch")
    if (
        str(skipped_report.get("ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields", "")).strip()
        != expected_default_text
    ):
        return fail("all_skipped_ci_sync_child_summary_default_fields_mismatch")
    if (
        str(skipped_report.get("combined_digest_selftest_default_field_text", "")).strip()
        != "age5_close_digest_selftest_ok=0"
    ):
        return fail("all_skipped_combined_digest_selftest_default_field_text_mismatch")
    if dict(skipped_report.get("combined_digest_selftest_default_field", {})) != expected_digest_default_field:
        return fail("all_skipped_combined_digest_selftest_default_field_mismatch")

    print("[age5-close-child-summary-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
