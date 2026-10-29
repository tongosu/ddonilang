#!/usr/bin/env python
from __future__ import annotations

from _ci_profile_matrix_selftest_lib import (
    PROFILE_MATRIX_BRIEF_KEYS,
    PROFILE_MATRIX_SELFTEST_PROFILES,
    PROFILE_MATRIX_SELFTEST_SCHEMA,
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
    build_profile_matrix_brief_payload_from_snapshot,
    build_profile_matrix_snapshot_from_doc,
    build_profile_matrix_triage_payload_from_snapshot,
    expected_profile_matrix_aggregate_summary_contract,
    join_profile_matrix_names,
)


def expect(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def build_valid_doc() -> dict[str, object]:
    aggregate_rows: dict[str, dict[str, object]] = {}
    for profile_name in PROFILE_MATRIX_SELFTEST_PROFILES:
        contract = expected_profile_matrix_aggregate_summary_contract(profile_name)
        aggregate_rows[profile_name] = {
            "expected_present": bool(contract["expected_present"]),
            "present": True,
            "status": str(contract["status"]),
            "reason": "ok",
            "expected_profile": str(contract["expected_profile"]),
            "expected_sync_profile": str(contract["expected_sync_profile"]),
            "profile": str(contract["expected_profile"]),
            "sync_profile": str(contract["expected_sync_profile"]),
            "expected_values": dict(contract["values"]),
            "values": dict(contract["values"]),
            "missing_keys": [],
            "mismatched_keys": [],
            "profile_ok": True,
            "sync_profile_ok": True,
            "values_ok": True,
            "gate_marker_expected": bool(contract["gate_marker_expected"]),
            "gate_marker_present": bool(contract["gate_marker_expected"]),
            "gate_marker_ok": True,
            "ok": bool(contract["ok"]),
        }
    return {
        "schema": PROFILE_MATRIX_SELFTEST_SCHEMA,
        "status": "pass",
        "ok": True,
        "total_elapsed_ms": 777,
        "selected_real_profiles": ["core_lang", "full", "seamgrim"],
        "skipped_real_profiles": [],
        "step_timeout_defaults_text": PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
        "aggregate_summary_sanity_ok": True,
        "aggregate_summary_sanity_checked_profiles": ["core_lang", "full", "seamgrim"],
        "aggregate_summary_sanity_failed_profiles": [],
        "aggregate_summary_sanity_skipped_profiles": [],
        "aggregate_summary_sanity_by_profile": aggregate_rows,
        "real_profiles": {
            "core_lang": {"selected": True, "skipped": False, "status": "pass", "ok": True, "total_elapsed_ms": 111},
            "full": {"selected": True, "skipped": False, "status": "pass", "ok": True, "total_elapsed_ms": 222},
            "seamgrim": {"selected": True, "skipped": False, "status": "pass", "ok": True, "total_elapsed_ms": 333},
        },
    }


def main() -> int:
    expect(join_profile_matrix_names(["core_lang", "full"]) == "core_lang,full", "join names basic mismatch")
    expect(join_profile_matrix_names([]) == "-", "join names empty mismatch")
    expect(join_profile_matrix_names("bad") == "-", "join names non-list mismatch")

    expect(build_profile_matrix_snapshot_from_doc({}) is None, "snapshot invalid schema must fail")
    expect(build_profile_matrix_snapshot_from_doc("bad") is None, "snapshot non-dict must fail")

    doc = build_valid_doc()
    snapshot = build_profile_matrix_snapshot_from_doc(doc, report_path="fake://profile-matrix")
    expect(isinstance(snapshot, dict), "snapshot type mismatch")
    assert snapshot is not None
    expect(snapshot["path"] == "fake://profile-matrix", "snapshot path mismatch")
    expect(snapshot["status"] == "pass", "snapshot status mismatch")
    expect(snapshot["ok"] == "1", "snapshot ok mismatch")
    expect(snapshot["total_elapsed_ms"] == "777", "snapshot total elapsed mismatch")
    expect(snapshot["selected_real_profiles"] == "core_lang,full,seamgrim", "snapshot selected mismatch")
    expect(snapshot["skipped_real_profiles"] == "-", "snapshot skipped mismatch")
    expect(
        snapshot["step_timeout_defaults_text"] == PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
        "snapshot timeout defaults mismatch",
    )
    expect(snapshot["aggregate_summary_sanity_ok"] == "1", "snapshot aggregate ok mismatch")
    expect(
        snapshot["aggregate_summary_sanity_checked_profiles"] == "core_lang,full,seamgrim",
        "snapshot checked profiles mismatch",
    )
    for profile_name, elapsed in (("core_lang", "111"), ("full", "222"), ("seamgrim", "333")):
        contract = expected_profile_matrix_aggregate_summary_contract(profile_name)
        expect(snapshot[f"{profile_name}_aggregate_summary_status"] == str(contract["status"]), f"{profile_name} status mismatch")
        expect(snapshot[f"{profile_name}_aggregate_summary_ok"] == "1", f"{profile_name} ok mismatch")
        expect(
            snapshot[f"{profile_name}_aggregate_summary_values"] == str(contract["values_text"]),
            f"{profile_name} values mismatch",
        )
        expect(snapshot[f"{profile_name}_elapsed_ms"] == elapsed, f"{profile_name} elapsed mismatch")

    brief_payload = build_profile_matrix_brief_payload_from_snapshot(snapshot)
    expect(tuple(brief_payload.keys()) == PROFILE_MATRIX_BRIEF_KEYS, "brief payload key order mismatch")
    expect(brief_payload["profile_matrix_total_elapsed_ms"] == "777", "brief total mismatch")
    expect(
        brief_payload["profile_matrix_selected_real_profiles"] == "core_lang,full,seamgrim",
        "brief selected mismatch",
    )
    expect(brief_payload["profile_matrix_core_lang_elapsed_ms"] == "111", "brief core_lang mismatch")
    expect(brief_payload["profile_matrix_full_elapsed_ms"] == "222", "brief full mismatch")
    expect(brief_payload["profile_matrix_seamgrim_elapsed_ms"] == "333", "brief seamgrim mismatch")

    triage_payload = build_profile_matrix_triage_payload_from_snapshot(snapshot)
    expect(triage_payload["report_path"] == "fake://profile-matrix", "triage path mismatch")
    expect(triage_payload["status"] == "pass", "triage status mismatch")
    expect(triage_payload["ok"] is True, "triage ok mismatch")
    expect(triage_payload["total_elapsed_ms"] == 777, "triage total mismatch")
    expect(triage_payload["selected_real_profiles"] == ["core_lang", "full", "seamgrim"], "triage selected mismatch")
    expect(triage_payload["skipped_real_profiles"] == [], "triage skipped mismatch")
    expect(
        str(triage_payload["step_timeout_defaults_text"]) == PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
        "triage timeout defaults mismatch",
    )
    expect(triage_payload["core_lang_elapsed_ms"] == 111, "triage core_lang mismatch")
    expect(triage_payload["full_elapsed_ms"] == 222, "triage full mismatch")
    expect(triage_payload["seamgrim_elapsed_ms"] == 333, "triage seamgrim mismatch")
    expect(triage_payload["aggregate_summary_sanity_ok"] is True, "triage aggregate ok mismatch")
    expect(
        triage_payload["aggregate_summary_sanity_checked_profiles"] == ["core_lang", "full", "seamgrim"],
        "triage checked mismatch",
    )
    for profile_name in PROFILE_MATRIX_SELFTEST_PROFILES:
        contract = expected_profile_matrix_aggregate_summary_contract(profile_name)
        expect(
            triage_payload[f"{profile_name}_aggregate_summary_status"] == str(contract["status"]),
            f"{profile_name} triage status mismatch",
        )
        expect(
            triage_payload[f"{profile_name}_aggregate_summary_ok"] is True,
            f"{profile_name} triage ok mismatch",
        )
        expect(
            triage_payload[f"{profile_name}_aggregate_summary_values"] == str(contract["values_text"]),
            f"{profile_name} triage values mismatch",
        )

    sparse_doc = {
        "schema": PROFILE_MATRIX_SELFTEST_SCHEMA,
        "status": "",
        "ok": False,
        "selected_real_profiles": [],
        "skipped_real_profiles": ["core_lang", "full", "seamgrim"],
        "aggregate_summary_sanity_ok": False,
        "aggregate_summary_sanity_checked_profiles": [],
        "aggregate_summary_sanity_failed_profiles": [],
        "aggregate_summary_sanity_skipped_profiles": ["core_lang", "full", "seamgrim"],
        "aggregate_summary_sanity_by_profile": {},
        "real_profiles": {},
    }
    sparse_snapshot = build_profile_matrix_snapshot_from_doc(sparse_doc, report_path="")
    expect(isinstance(sparse_snapshot, dict), "sparse snapshot type mismatch")
    assert sparse_snapshot is not None
    expect(sparse_snapshot["status"] == "unknown", "sparse status default mismatch")
    expect(sparse_snapshot["total_elapsed_ms"] == "-", "sparse total default mismatch")
    expect(sparse_snapshot["selected_real_profiles"] == "-", "sparse selected default mismatch")
    expect(
        sparse_snapshot["step_timeout_defaults_text"] == PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
        "sparse timeout defaults mismatch",
    )
    expect(sparse_snapshot["core_lang_aggregate_summary_status"] == "-", "sparse status row mismatch")
    expect(sparse_snapshot["core_lang_aggregate_summary_ok"] == "0", "sparse ok row mismatch")
    expect(sparse_snapshot["core_lang_aggregate_summary_values"] == "-", "sparse values row mismatch")

    print("ci profile matrix snapshot helper selftest ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
