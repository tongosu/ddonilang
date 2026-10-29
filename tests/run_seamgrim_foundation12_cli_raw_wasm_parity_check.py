#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_foundation12_cli_raw_wasm_parity_v1"
RUNNER = ROOT / "tests" / "seamgrim_foundation12_cli_raw_wasm_parity_runner.mjs"


EXPECTED_LESSONS = [
    "elem_math_number_line",
    "elem_math_fraction_line",
    "elem_math_rectangle_area",
    "elem_science_daily_temperature",
    "elem_econ_allowance_budget",
    "elem_econ_store_stock",
    "rep_math_function_line_v1",
    "rep_phys_pendulum_wave1_v1",
    "rep_phys_projectile_xy_v1",
    "rep_econ_supply_demand_tax_v1",
    "rep_phys_damped_oscillation_v1",
    "high_math_derivative",
]


def fail(message: str) -> int:
    print(f"[seamgrim-foundation12-cli-raw-wasm-parity] fail: {message}")
    return 1


def run(cmd: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )


def validate_report(report: dict, contract: dict) -> list[str]:
    failures: list[str] = []
    if report.get("schema") != "seamgrim.foundation12.cli_raw_wasm_parity.report.v1":
        failures.append(f"bad report schema: {report.get('schema')}")
    if report.get("ok") is not True:
        failures.append("runner ok=false")
    if report.get("lesson_count") != 12:
        failures.append(f"lesson_count expected 12 got {report.get('lesson_count')}")
    if report.get("state_hash_policy") != "strict_equality":
        failures.append(f"bad state_hash_policy: {report.get('state_hash_policy')}")
    if report.get("state_hash_formula_change") is not False:
        failures.append("state_hash_formula_change must be false")
    if report.get("full_single_runtime_closure_claim") is not False:
        failures.append("full_single_runtime_closure_claim must be false")
    lessons = report.get("lessons")
    if not isinstance(lessons, list):
        failures.append("lessons must be list")
        return failures
    ids = [str(row.get("lesson_id", "")) for row in lessons]
    if ids != EXPECTED_LESSONS:
        failures.append(f"lesson order mismatch: {ids}")
    if ids != contract.get("lesson_ids"):
        failures.append("contract lesson_ids mismatch")
    for row in lessons:
        lesson_id = row.get("lesson_id")
        if row.get("ok") is not True:
            failures.append(f"{lesson_id}: {row.get('failures')}")
        cli = row.get("cli") if isinstance(row.get("cli"), dict) else {}
        wasm = row.get("wasm") if isinstance(row.get("wasm"), dict) else {}
        if cli.get("effective_ticks") != 2 or wasm.get("executed_ticks") != 2:
            failures.append(f"{lesson_id}: tick count mismatch")
        if not str(cli.get("state_hash", "")).startswith("blake3:"):
            failures.append(f"{lesson_id}: cli state hash missing")
        if not str(wasm.get("state_hash", "")).startswith("blake3:"):
            failures.append(f"{lesson_id}: wasm state hash missing")
        if cli.get("state_hash") != wasm.get("state_hash"):
            failures.append(f"{lesson_id}: cli/wasm state hash mismatch")
        if wasm.get("input_frame_source") != "사람":
            failures.append(f"{lesson_id}: wasm input source missing")
        strict = row.get("strict_evidence") if isinstance(row.get("strict_evidence"), dict) else {}
        for field in contract.get("required_strict_evidence", []):
            if field not in strict:
                failures.append(f"{lesson_id}: strict evidence missing {field}")
        for field in (
            "warning_codes_equal",
            "configured_ticks_equal",
            "final_row_cli_subset_equal",
        ):
            if strict.get(field) is not True:
                failures.append(f"{lesson_id}: {field} is not true")
        if strict.get("final_row_comparison") != "equal":
            failures.append(f"{lesson_id}: final row comparison must be equal")
        if strict.get("canonical_ddn_comparison") != "equal":
            failures.append(f"{lesson_id}: canonical comparison must be equal")
        if strict.get("output_rows_comparison") != "equal":
            failures.append(f"{lesson_id}: output rows comparison must be equal")
        if strict.get("output_log_comparison") != "equal":
            failures.append(f"{lesson_id}: output log comparison must be equal")
        if strict.get("execution_envelope_comparison") != "equal":
            failures.append(f"{lesson_id}: execution envelope comparison must be equal")
        if strict.get("state_hash_comparison") != "equal":
            failures.append(f"{lesson_id}: state hash comparison must be equal")
        if strict.get("state_hash_policy") != "strict_equality":
            failures.append(f"{lesson_id}: state hash policy missing")
        if strict.get("state_hash_formula_change") is not False:
            failures.append(f"{lesson_id}: state hash formula change claimed")
    return failures


def selftest(contract: dict) -> list[str]:
    base_row = {
        "lesson_id": EXPECTED_LESSONS[0],
        "ok": True,
        "failures": [],
        "cli": {
            "configured_ticks": 2,
            "effective_ticks": 2,
            "state_hash": "blake3:" + "0" * 64,
            "trace_hash": "blake3:" + "1" * 64,
            "final_row_key_count": 2,
            "output_row_count": 0,
        },
        "wasm": {
            "configured_ticks": 2,
            "tick_id": 1,
            "executed_ticks": 2,
            "state_hash": "blake3:" + "0" * 64,
            "view_hash": "blake3:" + "3" * 64,
            "final_row_key_count": 3,
            "output_row_count": 0,
            "input_frame_source": "사람",
        },
        "strict_evidence": {
            "canonical_ddn_comparison": "equal",
            "warning_codes_equal": True,
            "configured_ticks_equal": True,
            "final_row_cli_subset_equal": True,
            "final_row_comparison": "equal",
            "output_rows_comparison": "equal",
            "output_log_comparison": "equal",
            "execution_envelope_comparison": "equal",
            "state_hash_comparison": "equal",
            "state_hash_policy": "strict_equality",
            "state_hash_formula_change": False,
        },
    }
    base = {
        "schema": "seamgrim.foundation12.cli_raw_wasm_parity.report.v1",
        "ok": True,
        "lesson_count": 12,
        "state_hash_policy": "strict_equality",
        "state_hash_formula_change": False,
        "full_single_runtime_closure_claim": False,
        "lessons": [
            {**base_row, "lesson_id": lesson_id}
            for lesson_id in EXPECTED_LESSONS
        ],
    }

    def mutate(mutator):
        doc = json.loads(json.dumps(base, ensure_ascii=False))
        mutator(doc)
        return doc

    cases = {
        "canonical_comparison_missing": lambda doc: doc["lessons"][0]["strict_evidence"].__setitem__("canonical_ddn_comparison", ""),
        "canonical_domain_mismatch": lambda doc: doc["lessons"][0]["strict_evidence"].__setitem__(
            "canonical_ddn_comparison", "domain_mismatch_reported"
        ),
        "row_subset_missing": lambda doc: doc["lessons"][0]["strict_evidence"].__setitem__("final_row_cli_subset_equal", False),
        "final_row_domain_mismatch": lambda doc: doc["lessons"][0]["strict_evidence"].__setitem__(
            "final_row_comparison", "domain_mismatch_reported"
        ),
        "output_row_domain_mismatch": lambda doc: doc["lessons"][0]["strict_evidence"].__setitem__(
            "output_rows_comparison", "domain_mismatch_reported"
        ),
        "execution_envelope_mismatch": lambda doc: doc["lessons"][0]["strict_evidence"].__setitem__(
            "execution_envelope_comparison", "domain_mismatch_reported"
        ),
        "state_hash_mismatch": lambda doc: (
            doc["lessons"][0]["wasm"].__setitem__("state_hash", "blake3:" + "2" * 64),
            doc["lessons"][0]["strict_evidence"].__setitem__("state_hash_comparison", "domain_mismatch_reported"),
        ),
        "hash_formula_change": lambda doc: doc["lessons"][0]["strict_evidence"].__setitem__("state_hash_formula_change", True),
        "single_runtime_claim": lambda doc: doc.__setitem__("full_single_runtime_closure_claim", True),
        "lesson_missing": lambda doc: doc["lessons"].pop(),
    }
    failures: list[str] = []
    for name, mutator in cases.items():
        if not validate_report(mutate(mutator), contract):
            failures.append(f"selftest {name} unexpectedly passed")
    return failures


def main() -> int:
    required = [PACK / name for name in ("README.md", "contract.detjson", "input.ddn", "golden.jsonl")]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail(f"missing pack files: {missing}")
    if not RUNNER.exists():
        return fail(f"missing runner: {RUNNER}")
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "seamgrim.foundation12.cli_raw_wasm_parity.contract.v1":
        return fail(f"bad contract schema: {contract.get('schema')}")
    if contract.get("state_hash_formula_change") is not False:
        return fail("contract state_hash_formula_change must be false")
    if contract.get("full_single_runtime_closure_claim") is not False:
        return fail("contract must not claim full single runtime closure")

    commands = [
        (["node", "--check", str(RUNNER)], 60),
        (["node", "--no-warnings", str(RUNNER)], 420),
        (["python", "tests/run_pack_golden.py", "seamgrim_foundation12_cli_raw_wasm_parity_v1"], 120),
    ]
    runner_report: dict | None = None
    failures: list[str] = []
    for cmd, timeout in commands:
        proc = run(cmd, timeout=timeout)
        if proc.returncode != 0:
            return fail(proc.stderr.strip() or proc.stdout.strip() or f"{cmd} rc={proc.returncode}")
        if cmd[0] == "node" and "--no-warnings" in cmd:
            try:
                runner_report = json.loads(proc.stdout)
            except json.JSONDecodeError as exc:
                return fail(f"runner emitted invalid json: {exc}")
    assert runner_report is not None
    failures.extend(validate_report(runner_report, contract))
    failures.extend(selftest(contract))
    if failures:
        return fail("; ".join(failures[:8]))
    print("[seamgrim-foundation12-cli-raw-wasm-parity-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
