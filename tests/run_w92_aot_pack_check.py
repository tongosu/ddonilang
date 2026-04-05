#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def fail(code: str, msg: str) -> int:
    print(f"[w92-pack-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(path: Path) -> dict:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise ValueError(f"missing file: {path}")
    except Exception as exc:
        raise ValueError(f"invalid json: {path} ({exc})")
    if not isinstance(data, dict):
        raise ValueError(f"json root must be object: {path}")
    return data


def main() -> int:
    parser = argparse.ArgumentParser(description="W92 AOT pack contract checker")
    parser.add_argument(
        "--pack",
        default="pack/gogae9_w92_aot_compiler_v2",
        help="pack directory path",
    )
    args = parser.parse_args()

    pack = Path(args.pack)
    required_files = [
        pack / "README.md",
        pack / "intent.md",
        pack / "bench_cases.json",
        pack / "golden.detjson",
        pack / "golden.jsonl",
    ]
    missing = [str(path).replace("\\", "/") for path in required_files if not path.exists()]
    if missing:
        return fail("E_W92_PACK_FILE_MISSING", ",".join(missing))

    try:
        bench = load_json(pack / "bench_cases.json")
    except ValueError as exc:
        return fail("E_W92_BENCH_JSON_INVALID", str(exc))

    if str(bench.get("schema", "")).strip() != "ddn.gogae9.w92.bench_cases.v1":
        return fail("E_W92_BENCH_SCHEMA", f"schema={bench.get('schema')}")
    bench_cases = bench.get("cases")
    if not isinstance(bench_cases, list) or not bench_cases:
        return fail("E_W92_BENCH_CASES_EMPTY", "cases must be non-empty list")

    bench_case_ids: list[str] = []
    for idx, row in enumerate(bench_cases, 1):
        if not isinstance(row, dict):
            return fail("E_W92_BENCH_CASE_ROW_INVALID", f"index={idx} type={type(row).__name__}")
        case_id = str(row.get("id", "")).strip()
        if not case_id:
            return fail("E_W92_BENCH_CASE_ID_MISSING", f"index={idx}")
        input_path = str(row.get("input", "")).strip()
        if not input_path:
            return fail("E_W92_BENCH_INPUT_MISSING", f"id={case_id}")
        speedup = row.get("min_speedup")
        if not isinstance(speedup, (int, float)) or float(speedup) < 20.0:
            return fail("E_W92_BENCH_SPEEDUP_FLOOR", f"id={case_id} min_speedup={speedup}")
        bench_case_ids.append(case_id)

    try:
        golden = load_json(pack / "golden.detjson")
    except ValueError as exc:
        return fail("E_W92_GOLDEN_JSON_INVALID", str(exc))

    if str(golden.get("schema", "")).strip() != "ddn.gogae9.w92.aot_parity_report.v1":
        return fail("E_W92_GOLDEN_SCHEMA", f"schema={golden.get('schema')}")
    if not bool(golden.get("overall_pass", False)):
        return fail("E_W92_GOLDEN_NOT_PASS", "overall_pass must be true")

    min_speedup_floor = golden.get("min_speedup_floor")
    if not isinstance(min_speedup_floor, (int, float)) or float(min_speedup_floor) < 20.0:
        return fail("E_W92_GOLDEN_SPEEDUP_FLOOR", f"min_speedup_floor={min_speedup_floor}")

    golden_cases = golden.get("cases")
    if not isinstance(golden_cases, list) or not golden_cases:
        return fail("E_W92_GOLDEN_CASES_EMPTY", "cases must be non-empty list")

    golden_case_ids: list[str] = []
    for idx, row in enumerate(golden_cases, 1):
        if not isinstance(row, dict):
            return fail("E_W92_GOLDEN_CASE_ROW_INVALID", f"index={idx} type={type(row).__name__}")
        case_id = str(row.get("id", "")).strip()
        if not case_id:
            return fail("E_W92_GOLDEN_CASE_ID_MISSING", f"index={idx}")
        if not bool(row.get("parity_ok", False)):
            return fail("E_W92_GOLDEN_PARITY_FAIL", f"id={case_id}")
        if not bool(row.get("meets_speedup", False)):
            return fail("E_W92_GOLDEN_SPEEDUP_FAIL", f"id={case_id}")
        golden_case_ids.append(case_id)

    if sorted(bench_case_ids) != sorted(golden_case_ids):
        return fail(
            "E_W92_CASE_SET_MISMATCH",
            f"bench={','.join(sorted(bench_case_ids))} golden={','.join(sorted(golden_case_ids))}",
        )

    print("[w92-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"bench_cases={len(bench_case_ids)}")
    print(f"golden_cases={len(golden_case_ids)}")
    print(f"min_speedup_floor={float(min_speedup_floor):.1f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
