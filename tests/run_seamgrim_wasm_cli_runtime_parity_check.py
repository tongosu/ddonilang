#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wasm_cli_runtime_parity_v1"
RUNNER = ROOT / "tests" / "seamgrim_wasm_cli_runtime_parity_runner.mjs"


def fail(message: str) -> int:
    print(f"[seamgrim-wasm-cli-runtime-parity] fail: {message}")
    return 1


def validate_report(report: dict, *, expected_case_count: int | None = 13) -> list[str]:
    failures: list[str] = []
    if report.get("schema") != "ddn.seamgrim.wasm_cli_runtime_parity.report.v1":
        failures.append(f"bad report schema: {report.get('schema')}")
    if report.get("ok") is not True:
        failures.append("runner report ok=false")
    if report.get("strict_runtime_parity") is not True:
        failures.append("strict_runtime_parity flag missing")
    if report.get("state_hash_policy") != "strict_equality":
        failures.append(f"bad state_hash_policy: {report.get('state_hash_policy')}")
    if not str(report.get("hash_domain_note", "")).strip():
        failures.append("hash_domain_note missing")
    cases = report.get("cases", [])
    if not isinstance(cases, list):
        failures.append("cases must be a list")
        return failures
    if expected_case_count is not None and len(cases) != expected_case_count:
        failures.append("runner report case count mismatch")
    for row in cases:
        case_id = row.get("id")
        if not row.get("ok"):
            failures.append(f"{case_id}: {row.get('failures')}")
        if not isinstance(row.get("cli_canonical_ddn", ""), str):
            failures.append(f"{case_id}: missing cli canonical ddn field")
        if not isinstance(row.get("wasm_canonical_ddn", ""), str):
            failures.append(f"{case_id}: missing wasm canonical ddn field")
        strict = row.get("strict_evidence")
        if not isinstance(strict, dict):
            failures.append(f"{case_id}: strict_evidence missing")
            continue
        reject_case = int(row.get("cli_exit_code", 0)) != 0
        if not reject_case and strict.get("canonical_ddn_equal") is not True:
            failures.append(f"{case_id}: canonical_ddn_equal is not true")
        warnings = strict.get("warning_codes")
        if not isinstance(warnings, dict) or warnings.get("equal") is not True:
            failures.append(f"{case_id}: warning code order parity missing")
        envelope = strict.get("execution_envelope")
        if not reject_case and (not isinstance(envelope, dict) or envelope.get("equal") is not True):
            failures.append(f"{case_id}: execution envelope parity missing")
        state_hash = strict.get("state_hash")
        if not isinstance(state_hash, dict):
            failures.append(f"{case_id}: state_hash strict evidence missing")
        elif not reject_case:
            if not str(state_hash.get("cli", "")).startswith("blake3:"):
                failures.append(f"{case_id}: cli state_hash missing")
            if not str(state_hash.get("wasm", "")).startswith("blake3:"):
                failures.append(f"{case_id}: wasm state_hash missing")
            if state_hash.get("comparison") != "equal":
                failures.append(f"{case_id}: state_hash comparison must be equal")
            if state_hash.get("cli") != state_hash.get("wasm"):
                failures.append(f"{case_id}: state_hash values differ")
            if state_hash.get("formula_change") is not False:
                failures.append(f"{case_id}: state_hash formula_change must be false")
        forbidden = strict.get("forbidden_normalization")
        if not isinstance(forbidden, dict):
            failures.append(f"{case_id}: forbidden_normalization evidence missing")
        else:
            for key in ("state_hash_removed", "warning_order_sorted", "output_row_dropped"):
                if forbidden.get(key) is not False:
                    failures.append(f"{case_id}: forbidden normalization flag {key} is not false")
        if not reject_case and strict.get("output_log", {}).get("checked") is not True:
            failures.append(f"{case_id}: output_log must be checked")
        if strict.get("output_log", {}).get("checked") is True and strict.get("output_log", {}).get("equal") is not True:
            failures.append(f"{case_id}: output_log strict parity failed")
        if not reject_case and strict.get("output_rows", {}).get("checked") is not True:
            failures.append(f"{case_id}: output_rows must be checked")
        if strict.get("output_rows", {}).get("checked") is True and strict.get("output_rows", {}).get("equal") is not True:
            failures.append(f"{case_id}: output_rows strict parity failed")
        if strict.get("final_row", {}).get("checked_keys") and strict.get("final_row", {}).get("equal") is not True:
            failures.append(f"{case_id}: final_row strict subset parity failed")
        if not reject_case and strict.get("final_row", {}).get("all_equal") is not True:
            failures.append(f"{case_id}: final_row full parity failed")
        if strict.get("value_json", {}).get("checked_keys") and strict.get("value_json", {}).get("equal") is not True:
            failures.append(f"{case_id}: value_json strict subset parity failed")
        if not reject_case and strict.get("value_json", {}).get("all_equal") is not True:
            failures.append(f"{case_id}: value_json full parity failed")
    return failures


def run_selftest() -> list[str]:
    base_case = {
        "id": "selftest",
        "ok": True,
        "cli_exit_code": 0,
        "cli_canonical_ddn": "x <- 1.",
        "wasm_canonical_ddn": "x <- 1.",
        "strict_evidence": {
            "canonical_ddn_equal": True,
            "warning_codes": {"cli": [], "wasm": [], "equal": True},
            "execution_envelope": {"cli": {"tick": 1}, "wasm": {"tick": 1}, "equal": True},
            "output_log": {"checked": True, "cli": ["1"], "wasm": ["1"], "equal": True},
            "output_rows": {"checked": True, "cli": [], "wasm": [], "equal": True},
            "final_row": {
                "checked_keys": ["x"],
                "cli_subset": {"x": 1},
                "wasm_subset": {"x": 1},
                "equal": True,
                "all_cli": {"x": 1},
                "all_wasm": {"x": 1},
                "all_equal": True,
            },
            "value_json": {
                "checked_keys": [],
                "cli_subset": {},
                "wasm_subset": {},
                "equal": None,
                "all_cli": {"x": 1},
                "all_wasm": {"x": 1},
                "all_equal": True,
            },
            "state_hash": {
                "cli": "blake3:" + "0" * 64,
                "wasm": "blake3:" + "0" * 64,
                "comparison": "equal",
                "formula_change": False,
            },
            "forbidden_normalization": {
                "state_hash_removed": False,
                "warning_order_sorted": False,
                "output_row_dropped": False,
            },
        },
    }
    base_report = {
        "schema": "ddn.seamgrim.wasm_cli_runtime_parity.report.v1",
        "ok": True,
        "strict_runtime_parity": True,
        "state_hash_policy": "strict_equality",
        "hash_domain_note": "selftest",
        "cases": [base_case],
    }

    def mutated_report(mutator):
        doc = json.loads(json.dumps(base_report))
        mutator(doc["cases"][0]["strict_evidence"])
        return doc

    negative_cases = {
        "missing_state_hash": lambda strict: strict.pop("state_hash"),
        "state_hash_mismatch": lambda strict: strict.__setitem__(
            "state_hash",
            {
                "cli": "blake3:" + "0" * 64,
                "wasm": "blake3:" + "1" * 64,
                "comparison": "domain_mismatch_reported",
                "formula_change": False,
            },
        ),
        "execution_envelope_mismatch": lambda strict: strict.__setitem__(
            "execution_envelope", {"cli": {"tick": 1}, "wasm": {"tick": 2}, "equal": False}
        ),
        "warning_order_not_checked": lambda strict: strict.__setitem__(
            "warning_codes", {"cli": ["A", "B"], "wasm": ["B", "A"], "equal": False}
        ),
        "output_row_dropped": lambda strict: strict.__setitem__(
            "output_rows", {"checked": True, "cli": [{"key": "x"}], "wasm": [], "equal": False}
        ),
        "final_row_full_mismatch": lambda strict: strict["final_row"].__setitem__("all_equal", False),
        "value_json_full_mismatch": lambda strict: strict["value_json"].__setitem__("all_equal", False),
        "hash_removed_normalization": lambda strict: strict.__setitem__(
            "forbidden_normalization",
            {"state_hash_removed": True, "warning_order_sorted": False, "output_row_dropped": False},
        ),
    }
    failures: list[str] = []
    for name, mutator in negative_cases.items():
        if not validate_report(mutated_report(mutator), expected_case_count=1):
            failures.append(f"selftest {name} unexpectedly passed")
    return failures


def main() -> int:
    contract_path = PACK / "contract.detjson"
    if not contract_path.exists():
        return fail(f"missing contract: {contract_path}")
    if not RUNNER.exists():
        return fail(f"missing runner: {RUNNER}")

    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    case_ids = [str(case.get("id", "")).strip() for case in contract.get("cases", [])]
    expected = {
        "c01_console_grid_scalar",
        "c02_moyang_space2d",
        "c03_symbolic_simplify",
        "c04_proof_tactic_equivalence",
        "c05_lambda_store",
        "c06_chaebi_derived_show",
        "c07_hook_tick_show",
        "c08_parse_warning_reassign",
        "c09_boim_structured_view",
        "c10_setting_madi_count",
        "c11_show_and_boim_split",
        "c12_setting_madi_bad_value",
        "c13_bogae_draw_madi_view_sugar",
    }
    if set(case_ids) != expected:
        return fail(f"case set mismatch: {case_ids}")
    for case in contract.get("cases", []):
        rel = str(case.get("input", "")).strip()
        if not rel:
            return fail(f"{case.get('id')}: input missing")
        if not (PACK / rel).exists():
            return fail(f"{case.get('id')}: input not found: {rel}")

    proc = subprocess.run(
        ["node", "--no-warnings", str(RUNNER), str(PACK)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=300,
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or f"returncode={proc.returncode}"
        return fail(detail)
    try:
        report = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        return fail(f"runner emitted invalid json: {exc}")
    validation_failures = validate_report(report)
    validation_failures.extend(run_selftest())
    if validation_failures:
        return fail("; ".join(validation_failures[:8]))
    if len(report.get("cases", [])) != len(expected):
        return fail("runner report case count mismatch")

    print("seamgrim wasm/cli runtime parity ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
