#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK_SCHEMA = "ddn.lang_flow_hook_interaction.pack.contract.v1"
REQUIRED_CASES = {
    "c01_flow_then_hook_snapshot": ("ok", ""),
    "c02_no_same_tick_refire": ("ok", ""),
    "c03_multiple_flow_source_conflict": ("diag", "E_FLOW_MULTIPLE_SOURCE_CONFLICT"),
    "c04_flow_cycle_fatal": ("diag", "E_FLOW_CIRCULAR_REFERENCE"),
}
RUNTIME_OK_CHECKS = [
    (
        "c01_flow_then_hook_snapshot",
        1,
        "after_tick_1",
        {"입력": 5, "흐름값": 10, "훅실행": 1, "결과": 10},
    ),
    (
        "c02_no_same_tick_refire",
        1,
        "after_tick_1",
        {"값": 4, "카운트": 0},
    ),
    (
        "c02_no_same_tick_refire",
        3,
        "after_tick_2",
        {"값": 16, "카운트": 1},
    ),
]


def fail(code: str, msg: str) -> int:
    print(f"[lang-flow-hook-pack-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(path: Path) -> object:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise ValueError(f"missing file: {path}")
    except Exception as exc:
        raise ValueError(f"invalid json: {path} ({exc})")


def run_teul(pack: Path, input_path: Path, ticks: int, summary_path: Path) -> subprocess.CompletedProcess[str]:
    cmd = [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        str(ROOT / "tools" / "teul-cli" / "Cargo.toml"),
        "--",
        "run",
        str(input_path),
        "--madi",
        str(ticks),
        "--summary-json",
        str(summary_path),
    ]
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=120,
    )


def runtime_summary_value(summary: object, key: str) -> object:
    if not isinstance(summary, dict):
        raise ValueError("summary root must be object")
    resources = summary.get("resources")
    if not isinstance(resources, dict):
        raise ValueError("summary.resources must be object")
    value_json = resources.get("value_json")
    if not isinstance(value_json, dict):
        raise ValueError("summary.resources.value_json must be object")
    if key not in value_json:
        raise ValueError(f"summary missing state key: {key}")
    return value_json[key]


def check_ok_runtime(pack: Path, case_id: str, ticks: int, label: str, expected: dict[str, object]) -> int:
    case_dir = pack / "cases" / case_id
    input_path = case_dir / "input.ddn"
    with tempfile.TemporaryDirectory(prefix=f"flow_hook_{case_id}_{ticks}_") as td:
        summary_path = Path(td) / "summary.detjson"
        result = run_teul(pack, input_path, ticks, summary_path)
        if result.returncode != 0:
            return fail(
                "E_LANG_FLOW_HOOK_RUNTIME_RUN",
                f"id={case_id} ticks={ticks} rc={result.returncode} stderr={result.stderr.strip()}",
            )
        try:
            summary = load_json(summary_path)
        except ValueError as exc:
            return fail("E_LANG_FLOW_HOOK_RUNTIME_SUMMARY", f"id={case_id} {exc}")
        for key, want in expected.items():
            try:
                got = runtime_summary_value(summary, key)
            except ValueError as exc:
                return fail("E_LANG_FLOW_HOOK_RUNTIME_STATE", f"id={case_id} {exc}")
            if got != want:
                return fail(
                    "E_LANG_FLOW_HOOK_RUNTIME_VALUE",
                    f"id={case_id} {label} key={key} expected={want} actual={got}",
                )
    print(f"[lang-flow-hook-pack-check] runtime ok id={case_id} ticks={ticks} label={label}")
    return 0


def check_diag_runtime(pack: Path, case_id: str, expected_error: str) -> int:
    case_dir = pack / "cases" / case_id
    input_path = case_dir / "input.ddn"
    with tempfile.TemporaryDirectory(prefix=f"flow_hook_{case_id}_diag_") as td:
        summary_path = Path(td) / "summary.detjson"
        result = run_teul(pack, input_path, 1, summary_path)
        if result.returncode == 0:
            return fail("E_LANG_FLOW_HOOK_RUNTIME_DIAG_RC", f"id={case_id} expected nonzero")
        combined = "\n".join([result.stderr, result.stdout])
        if expected_error not in combined:
            return fail(
                "E_LANG_FLOW_HOOK_RUNTIME_DIAG_CODE",
                f"id={case_id} expected={expected_error} actual={combined.strip()}",
            )
    print(f"[lang-flow-hook-pack-check] runtime diag id={case_id} error={expected_error}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Lang flow/hook interaction pack checker")
    parser.add_argument("--pack", default="pack/lang_flow_hook_interaction_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required_files = [
        pack / "README.md",
        pack / "intent.md",
        pack / "contract.detjson",
        pack / "tests" / "README.md",
    ]
    missing = [str(path).replace("\\", "/") for path in required_files if not path.exists()]
    if missing:
        return fail("E_LANG_FLOW_HOOK_PACK_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
    except ValueError as exc:
        return fail("E_LANG_FLOW_HOOK_CONTRACT_INVALID", str(exc))

    if not isinstance(contract, dict):
        return fail("E_LANG_FLOW_HOOK_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_LANG_FLOW_HOOK_SCHEMA", f"schema={contract.get('schema')}")
    if str(contract.get("evidence_tier", "")).strip() != "runtime":
        return fail("E_LANG_FLOW_HOOK_EVIDENCE_TIER", f"tier={contract.get('evidence_tier')}")
    if str(contract.get("closure_claim", "")).strip() != "yes":
        return fail("E_LANG_FLOW_HOOK_CLOSURE_CLAIM", f"closure={contract.get('closure_claim')}")
    if contract.get("ordering_contract") != [
        "ordinary_assignment",
        "flow_fixed_point",
        "tail_phase_hook",
    ]:
        return fail("E_LANG_FLOW_HOOK_ORDERING", f"ordering={contract.get('ordering_contract')}")

    cases = contract.get("cases")
    if not isinstance(cases, list):
        return fail("E_LANG_FLOW_HOOK_CASES_TYPE", "cases must be list")
    case_index = {
        str(row.get("id", "")).strip(): row for row in cases if isinstance(row, dict)
    }
    if set(case_index) != set(REQUIRED_CASES):
        return fail("E_LANG_FLOW_HOOK_CASE_SET", f"cases={sorted(case_index)}")

    for case_id, (case_kind, expected_error) in REQUIRED_CASES.items():
        row = case_index[case_id]
        if str(row.get("case_kind", "")).strip() != case_kind:
            return fail("E_LANG_FLOW_HOOK_CASE_KIND", f"id={case_id} kind={row.get('case_kind')}")
        if expected_error and str(row.get("expected_error", "")).strip() != expected_error:
            return fail("E_LANG_FLOW_HOOK_CASE_ERROR", f"id={case_id} error={row.get('expected_error')}")

        case_dir = pack / "cases" / case_id
        input_path = case_dir / "input.ddn"
        expected_path = case_dir / "expected.json"
        if not input_path.exists() or not expected_path.exists():
            return fail("E_LANG_FLOW_HOOK_CASE_FILE_MISSING", case_id)
        try:
            expected = load_json(expected_path)
        except ValueError as exc:
            return fail("E_LANG_FLOW_HOOK_EXPECTED_INVALID", str(exc))
        if not isinstance(expected, dict):
            return fail("E_LANG_FLOW_HOOK_EXPECTED_TYPE", case_id)
        if str(expected.get("case_id", "")).strip() != case_id:
            return fail("E_LANG_FLOW_HOOK_EXPECTED_ID", f"id={case_id}")
        if str(expected.get("case_kind", "")).strip() != case_kind:
            return fail("E_LANG_FLOW_HOOK_EXPECTED_KIND", f"id={case_id}")
        if expected_error and str(expected.get("expected_error", "")).strip() != expected_error:
            return fail("E_LANG_FLOW_HOOK_EXPECTED_ERROR", f"id={case_id}")

    for case_id, ticks, label, expected in RUNTIME_OK_CHECKS:
        rc = check_ok_runtime(pack, case_id, ticks, label, expected)
        if rc != 0:
            return rc
    for case_id, (_, expected_error) in REQUIRED_CASES.items():
        if expected_error:
            rc = check_diag_runtime(pack, case_id, expected_error)
            if rc != 0:
                return rc

    print("[lang-flow-hook-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(case_index)}")
    print("runtime_checks=5")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
