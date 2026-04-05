#!/usr/bin/env python
from __future__ import annotations

import run_ci_sanity_gate


def fail(detail: str) -> int:
    print(f"[ci-sanity-dynamic-source-profile-split-selftest] fail: {detail}")
    return 1


def expect_suffix(path_text: str, suffix: str, label: str) -> int:
    if str(path_text).strip().endswith(suffix):
        return 0
    return fail(f"{label}_suffix_mismatch expected=*{suffix} actual={path_text}")


def main() -> int:
    for profile in ("core_lang", "full", "seamgrim"):
        report_paths = run_ci_sanity_gate.build_completion_gate_report_paths(profile)
        checks = (
            ("age2_dynamic_source", f"age2_completion_gate.{profile}.dynamic.last.detjson"),
            ("age3_dynamic_source", f"age3_completion_gate.{profile}.dynamic.last.detjson"),
            (
                "age4_dynamic_source",
                f"age4_proof_transport_contract_selftest.{profile}.dynamic.last.detjson",
            ),
            ("age5_dynamic_source", f"pack_golden_age5_surface_selftest.{profile}.dynamic.last.detjson"),
        )
        for key, suffix in checks:
            value = str(report_paths.get(key, "")).strip()
            if not value:
                return fail(f"{profile}.{key}_missing")
            if expect_suffix(value, suffix, f"{profile}.{key}") != 0:
                return 1

    print("[ci-sanity-dynamic-source-profile-split-selftest] ok profiles=3")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
