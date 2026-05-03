#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "relation_solve_ddn_bridge_v2"
FORMULA_QUADRATIC_PACK = ROOT / "pack" / "formula_relation_solve_quadratic_v1" / "golden.jsonl"
SYSTEM_PACK = ROOT / "pack" / "relation_solve_system_2x2_v1" / "golden.jsonl"
WASM_CLI_PARITY_CONTRACT = ROOT / "pack" / "relation_solve_wasm_cli_parity_v2" / "contract.detjson"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def main() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "input_quadratic_success.ddn",
        PACK / "input_system_2x2_success.ddn",
        PACK / "input_unsupported_higher_degree.ddn",
        PACK / "input_unsupported_quadratic.ddn",
        PACK / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_RELATION_BRIDGE_V2_MISSING", str(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.relation_solve_ddn_bridge.v2.pack.contract.v1":
        return fail("E_RELATION_BRIDGE_V2_SCHEMA", str(contract.get("schema")))
    if contract.get("parity_rails") != ["direct_formula_bridge", "cli_tool_wasm"]:
        return fail("E_RELATION_BRIDGE_V2_PARITY_RAILS", str(contract.get("parity_rails")))
    row_contract = contract.get("representative_rows")
    if not isinstance(row_contract, list) or len(row_contract) != 4:
        return fail("E_RELATION_BRIDGE_V2_ROW_CONTRACT", str(row_contract))
    rows = [
        json.loads(line)
        for line in (PACK / "golden.jsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    row_map = {str(row.get("id", "")).strip(): row for row in rows}
    required_rows = {
        "c01_quadratic_success",
        "c02_system_2x2_success",
        "c03_unsupported_higher_degree",
        "c04_unsupported_quadratic",
    }
    if set(row_map) != required_rows:
        return fail("E_RELATION_BRIDGE_V2_GOLDEN", str(sorted(row_map)))
    quadratic = row_map["c01_quadratic_success"].get("stdout")
    system = row_map["c02_system_2x2_success"].get("stdout")
    unsupported_degree = row_map["c03_unsupported_higher_degree"].get("stdout")
    unsupported_quadratic = row_map["c04_unsupported_quadratic"].get("stdout")
    if quadratic != ['#성공(미지수="x", 값=2)']:
        return fail("E_RELATION_BRIDGE_V2_QUADRATIC", str(quadratic))
    if system != ['#성공(해=(x=3, y=2))']:
        return fail("E_RELATION_BRIDGE_V2_SYSTEM", str(system))
    if unsupported_degree != ['#실패(사유="unsupported")']:
        return fail("E_RELATION_BRIDGE_V2_UNSUPPORTED_HIGHER_DEGREE", str(unsupported_degree))
    if unsupported_quadratic != ['#실패(사유="unsupported")']:
        return fail("E_RELATION_BRIDGE_V2_UNSUPPORTED_QUADRATIC", str(unsupported_quadratic))
    row_contract_map = {str(row.get("id", "")).strip(): row for row in row_contract}
    if set(row_contract_map) != required_rows:
        return fail("E_RELATION_BRIDGE_V2_ROW_IDS", str(sorted(row_contract_map)))
    expected_contract = {
        "c01_quadratic_success": {
            "input": "input_quadratic_success.ddn",
            "expected_surface": '#성공(미지수="x", 값=2)',
            "unsupported_reason": None,
            "parity_source": ["direct_formula_bridge", "cli_tool_wasm"],
        },
        "c02_system_2x2_success": {
            "input": "input_system_2x2_success.ddn",
            "expected_surface": '#성공(해=(x=3, y=2))',
            "unsupported_reason": None,
            "parity_source": ["direct_formula_bridge", "cli_tool_wasm"],
        },
        "c03_unsupported_higher_degree": {
            "input": "input_unsupported_higher_degree.ddn",
            "expected_surface": '#실패(사유="unsupported")',
            "unsupported_reason": "higher_degree",
            "parity_source": ["cli_tool_wasm"],
        },
        "c04_unsupported_quadratic": {
            "input": "input_unsupported_quadratic.ddn",
            "expected_surface": '#실패(사유="unsupported")',
            "unsupported_reason": "unsupported_quadratic",
            "parity_source": ["direct_formula_bridge", "cli_tool_wasm"],
        },
    }
    for row_id, expected in expected_contract.items():
        actual = row_contract_map[row_id]
        for field, expected_value in expected.items():
            if actual.get(field) != expected_value:
                return fail(f"E_RELATION_BRIDGE_V2_ROW_FIELD:{row_id}:{field}", str(actual.get(field)))
    if not FORMULA_QUADRATIC_PACK.exists() or not SYSTEM_PACK.exists():
        return fail("E_RELATION_BRIDGE_V2_PARITY_GOLDEN_MISSING", "formula/system pack golden missing")
    formula_rows = [
        json.loads(line)
        for line in FORMULA_QUADRATIC_PACK.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    system_rows = [
        json.loads(line)
        for line in SYSTEM_PACK.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    formula_map = {str(row.get("id", "")).strip(): row for row in formula_rows}
    system_map = {str(row.get("id", "")).strip(): row for row in system_rows}
    if formula_map.get("c01_quadratic_single_root", {}).get("stdout") != quadratic:
        return fail("E_RELATION_BRIDGE_V2_FORMULA_PARITY", str(formula_map.get("c01_quadratic_single_root")))
    if formula_map.get("c03_quadratic_unsupported", {}).get("stdout") != unsupported_quadratic:
        return fail("E_RELATION_BRIDGE_V2_FORMULA_UNSUPPORTED_PARITY", str(formula_map.get("c03_quadratic_unsupported")))
    if system_map.get("c01_system_success", {}).get("stdout") != system:
        return fail("E_RELATION_BRIDGE_V2_SYSTEM_PARITY", str(system_map.get("c01_system_success")))
    if not WASM_CLI_PARITY_CONTRACT.exists():
        return fail("E_RELATION_BRIDGE_V2_WASM_PARITY_CONTRACT_MISSING", str(WASM_CLI_PARITY_CONTRACT))
    wasm_contract = json.loads(WASM_CLI_PARITY_CONTRACT.read_text(encoding="utf-8"))
    case_ids = {str(case.get("id", "")).strip() for case in wasm_contract.get("cases", [])}
    expected_case_ids = required_rows
    if case_ids != expected_case_ids:
        return fail("E_RELATION_BRIDGE_V2_WASM_PARITY_CASES", str(sorted(case_ids)))
    readme_text = (PACK / "README.md").read_text(encoding="utf-8")
    required_readme_tokens = [
        "| `c01_quadratic_success` | `input_quadratic_success.ddn` |",
        "| `c02_system_2x2_success` | `input_system_2x2_success.ddn` |",
        "| `c03_unsupported_higher_degree` | `input_unsupported_higher_degree.ddn` |",
        "| `c04_unsupported_quadratic` | `input_unsupported_quadratic.ddn` |",
        "| direct formula/system | `formula_relation_solve_quadratic_v1`, `relation_solve_system_2x2_v1` |",
        "| DDN bridge | `relation_solve_ddn_bridge_v2` |",
        "| CLI/tool/WASM parity | `relation_solve_wasm_cli_parity_v2` |",
        "direct_formula_bridge",
        "cli_tool_wasm",
    ]
    for token in required_readme_tokens:
        if token not in readme_text:
            return fail("E_RELATION_BRIDGE_V2_README", token)
    print("relation solve ddn bridge v2 pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
