#!/usr/bin/env python
from __future__ import annotations

from _ci_profile_matrix_selftest_lib import (
    PROFILE_MATRIX_SELFTEST_PROFILES,
    build_lightweight_profile_gate_contract,
    build_lightweight_profile_gate_lines,
    expected_profile_matrix_summary_values,
)


def expect(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> int:
    expect(
        PROFILE_MATRIX_SELFTEST_PROFILES == ("core_lang", "full", "seamgrim"),
        "profile list mismatch",
    )

    for profile_name in PROFILE_MATRIX_SELFTEST_PROFILES:
        quick_contract = build_lightweight_profile_gate_contract(profile_name, quick=True)
        full_contract = build_lightweight_profile_gate_contract(profile_name, quick=False)

        for contract, quick in ((quick_contract, True), (full_contract, False)):
            expect(isinstance(contract, dict), f"{profile_name} contract type mismatch quick={quick}")
            expect(
                str(contract.get("profile_status_marker", "")).startswith("ci_profile_"),
                f"{profile_name} profile_status_marker mismatch quick={quick}",
            )
            base_markers = contract.get("base_markers", [])
            expect(isinstance(base_markers, list) and len(base_markers) >= 3, f"{profile_name} base_markers mismatch quick={quick}")
            expect(
                str(contract.get("profile_status_marker", "")) in [str(item) for item in base_markers],
                f"{profile_name} profile_status_marker missing from base_markers quick={quick}",
            )
            expect(
                isinstance(contract.get("quick_skip_marker", ""), str) and "aggregate gate skipped by --quick" in str(contract.get("quick_skip_marker", "")),
                f"{profile_name} quick_skip_marker mismatch quick={quick}",
            )

        quick_lines = build_lightweight_profile_gate_lines(profile_name, quick=True)
        full_lines = build_lightweight_profile_gate_lines(profile_name, quick=False)
        expect(isinstance(quick_lines, list) and len(quick_lines) >= 2, f"{profile_name} quick lines mismatch")
        expect(isinstance(full_lines, list) and len(full_lines) > len(quick_lines), f"{profile_name} full lines mismatch")
        expect(
            str(quick_contract.get("quick_skip_marker", "")) in quick_lines,
            f"{profile_name} quick skip marker line missing",
        )
        expect(
            "[ci-gate-summary]" not in "\n".join(quick_lines),
            f"{profile_name} quick lines must not include summary markers",
        )
        expect(
            str(full_contract.get("quick_skip_marker", "")) not in full_lines,
            f"{profile_name} full lines must not include quick skip marker",
        )

        summary_pairs = full_contract.get("summary_pairs", {})
        expect(isinstance(summary_pairs, dict), f"{profile_name} summary_pairs mismatch")
        expected_values = expected_profile_matrix_summary_values(profile_name)
        expect(
            dict(summary_pairs).get("ci_sanity_gate_profile") == profile_name,
            f"{profile_name} ci_sanity_gate_profile mismatch",
        )
        expect(
            dict(summary_pairs).get("ci_sync_readiness_sanity_profile") == profile_name,
            f"{profile_name} ci_sync_readiness_sanity_profile mismatch",
        )
        for key, expected in expected_values.items():
            expect(
                str(dict(summary_pairs).get(key, "")) == str(expected),
                f"{profile_name} summary_pairs value mismatch: {key}",
            )
            expect(
                f"[ci-gate-summary] {key}={expected}" in full_lines,
                f"{profile_name} full lines summary marker missing: {key}",
            )

        post_markers = full_contract.get("post_markers", [])
        expect(isinstance(post_markers, list) and len(post_markers) >= 1, f"{profile_name} post_markers mismatch")
        for marker in post_markers:
            expect(str(marker) in full_lines, f"{profile_name} post marker missing from full lines: {marker}")

        if profile_name == "core_lang":
            expect(
                "[ci-profile-core-lang] aggregate summary sanity markers ok" not in full_lines,
                "core_lang must not require aggregate summary sanity marker",
            )
        else:
            expect(
                any("aggregate summary sanity markers ok" in str(line) for line in full_lines),
                f"{profile_name} aggregate summary sanity marker missing",
            )

    print("ci profile matrix lightweight contract selftest ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
