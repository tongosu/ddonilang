#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


REQUIRED_TOKEN_MAP = {
    "tests/ci_check_error_codes.py": [
        "GATE_REPORT_INDEX_CODES",
        '"INDEX_MISSING": "E_GATE_INDEX_MISSING"',
        '"INDEX_JSON_INVALID": "E_GATE_INDEX_JSON_INVALID"',
        '"INDEX_SCHEMA": "E_GATE_INDEX_SCHEMA"',
        '"INDEX_REPORTS_MISSING": "E_GATE_INDEX_REPORTS_MISSING"',
        '"REPORT_KEY_MISSING": "E_GATE_INDEX_REPORT_KEY_MISSING"',
        '"REPORT_PATH_MISSING": "E_GATE_INDEX_REPORT_PATH_MISSING"',
        '"ARTIFACT_JSON_INVALID": "E_GATE_INDEX_ARTIFACT_JSON_INVALID"',
        '"ARTIFACT_SCHEMA_MISMATCH": "E_GATE_INDEX_ARTIFACT_SCHEMA_MISMATCH"',
    ],
    "tests/run_ci_gate_report_index_check.py": [
        "from ci_check_error_codes import GATE_REPORT_INDEX_CODES as CODES",
        "INDEX_SCHEMA = \"ddn.ci.aggregate_gate.index.v1\"",
        "REQUIRED_REPORT_PATH_KEYS",
        "\"seamgrim_wasm_cli_diag_parity\"",
        "ARTIFACT_SCHEMA_MAP",
        "\"ddn.ci.gate_result.v1\"",
        "\"ddn.ci.sanity_gate.v1\"",
        "\"ddn.ci.sync_readiness.v1\"",
        "\"ddn.seamgrim.wasm_cli_diag_parity.v1\"",
        "missing index reports key/path",
        "missing report path for",
        "artifact schema mismatch",
    ],
    "tests/run_ci_gate_report_index_check_selftest.py": [
        "run_ci_gate_report_index_check.py",
        "missing key case must fail",
        "missing path case must fail",
        "bad schema case must fail",
        "REPORT_KEY_MISSING",
        "REPORT_PATH_MISSING",
        "ARTIFACT_SCHEMA_MISMATCH",
    ],
    "tests/run_ci_aggregate_gate.py": [
        "check_ci_gate_report_index",
        "check_ci_gate_report_index_selftest",
        "check_ci_gate_report_index_diagnostics",
        "ci_gate_report_index_check",
        "ci_gate_report_index_selftest",
        "ci_gate_report_index_diagnostics_check",
        "tests/run_ci_gate_report_index_check.py",
        "tests/run_ci_gate_report_index_check_selftest.py",
        "tests/run_ci_gate_report_index_diagnostics_check.py",
    ],
}


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    missing: list[str] = []
    for rel_path, tokens in REQUIRED_TOKEN_MAP.items():
        target = root / rel_path
        if not target.exists():
            print(f"missing target: {target}")
            return 1
        text = target.read_text(encoding="utf-8")
        for token in tokens:
            if token not in text:
                missing.append(f"{rel_path}::{token}")

    if missing:
        print("ci gate report-index diagnostics check failed:")
        for token in missing[:16]:
            print(f" - missing token: {token}")
        return 1

    print("ci gate report-index diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
