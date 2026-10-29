#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "ddn.public_specific_conformance_aggregate.v1_candidate"

# Owner-approved D7 Option B. These are the exact eleven leaf roots left after
# removing the two internal umbrella gates. Parity and durability remain
# explicit members; they are not inferred from an umbrella PASS.
LEAF_ROOTS = (
    "tests/run_seamgrim_public_default_route_check.py",
    "tests/run_seamgrim_public_physics_first_run_check.py",
    "tests/run_seamgrim_one_file_ddn_create_run_check.py",
    "tests/run_seamgrim_public_physics_change_storyboard_check.py",
    "tests/run_seamgrim_public_share_clone_reopen_check.py",
    "tests/run_seamgrim_legacy_deeplink_compat_check.py",
    "tests/run_seamgrim_public_single_node_serving_check.py",
    "tests/run_seamgrim_public_four_viewport_quality_check.py",
    "tests/run_ddn_cli_wasm_freshness_check.py",
    "tests/run_ddn_cli_wasm_true_parity_successor_check.py",
    "tests/run_seamgrim_view_only_state_hash_invariant_check.py",
)

EXCLUDED_INTERNAL_UMBRELLAS = (
    "tests/run_seamgrim_workbench_durable_quality_gate_check.py",
    "tests/run_ddn_cli_wasm_sync_gate_check.py",
)

PARITY_MEMBERS = (
    "tests/run_ddn_cli_wasm_freshness_check.py",
    "tests/run_ddn_cli_wasm_true_parity_successor_check.py",
    "tests/run_seamgrim_view_only_state_hash_invariant_check.py",
)

DURABILITY_MEMBERS = (
    "tests/run_seamgrim_one_file_ddn_create_run_check.py",
    "tests/run_seamgrim_public_share_clone_reopen_check.py",
    "tests/run_seamgrim_public_four_viewport_quality_check.py",
)


def manifest() -> dict[str, object]:
    return {
        "schema": SCHEMA,
        "status": "CANDIDATE_NOT_PUBLICATION_ACTIVATED",
        "owner_policy": "OPTION_B_PUBLIC_SPECIFIC_AGGREGATE",
        "leaf_roots": list(LEAF_ROOTS),
        "parity_members": list(PARITY_MEMBERS),
        "durability_members": list(DURABILITY_MEMBERS),
        "excluded_internal_umbrellas": list(EXCLUDED_INTERNAL_UMBRELLAS),
        "pc01_public_membership": "PENDING_FRESH_TARGETED_PLAIN_PASS_AND_RIGHTS",
        "projectile_public_copy": "PENDING_RIGHTS_CONFIRMATION",
    }


def check_membership() -> list[str]:
    errors: list[str] = []
    if len(LEAF_ROOTS) != 11 or len(set(LEAF_ROOTS)) != 11:
        errors.append("leaf root count/uniqueness is not exact 11")
    for path in (*LEAF_ROOTS, *EXCLUDED_INTERNAL_UMBRELLAS):
        if not (ROOT / path).is_file():
            errors.append(f"missing root: {path}")
    for path in (*PARITY_MEMBERS, *DURABILITY_MEMBERS):
        if path not in LEAF_ROOTS:
            errors.append(f"curated member is not an exact leaf: {path}")
    for path in EXCLUDED_INTERNAL_UMBRELLAS:
        if path in LEAF_ROOTS:
            errors.append(f"internal umbrella leaked into public aggregate: {path}")
    return errors


def run_leaf(path: str) -> tuple[int, str]:
    proc = subprocess.run(
        [sys.executable, path],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=600,
        check=False,
    )
    return proc.returncode, proc.stdout


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest-json", action="store_true")
    parser.add_argument("--check-files-only", action="store_true")
    args = parser.parse_args()

    errors = check_membership()
    if errors:
        print(json.dumps({"status": "FAIL", "errors": errors}, ensure_ascii=False))
        return 1
    if args.manifest_json:
        print(json.dumps(manifest(), ensure_ascii=False, sort_keys=True))
        return 0
    if args.check_files_only:
        print(json.dumps({"status": "PASS", **manifest()}, ensure_ascii=False, sort_keys=True))
        return 0

    results: list[dict[str, object]] = []
    for path in LEAF_ROOTS:
        try:
            returncode, output = run_leaf(path)
        except subprocess.TimeoutExpired as exc:
            print(json.dumps({"status": "FAIL", "root": path, "error": str(exc)}, ensure_ascii=False))
            return 1
        results.append({"root": path, "returncode": returncode})
        if returncode != 0:
            print(output, end="" if output.endswith("\n") else "\n")
            print(json.dumps({"status": "FAIL", "failed_root": path, "results": results}, ensure_ascii=False))
            return 1

    print(json.dumps({"status": "PASS", "results": results, **manifest()}, ensure_ascii=False, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
