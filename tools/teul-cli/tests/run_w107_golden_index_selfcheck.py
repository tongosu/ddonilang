#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import re
import sys
from datetime import datetime, timezone
from pathlib import Path


SCRIPT_TAG = "w107-golden-index-selfcheck"
PROGRESS_ENV_KEY = "DDN_W107_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON"
CASE_DIR_RE = re.compile(r"^W107_([CG]\d{2})_")
CASE_CODE_RE = re.compile(r"^[CG]\d{2}$")
ALLOWED_COMMANDS = {"canon", "run"}
ALLOWED_CLASSIFICATIONS = {"canon_success", "run_success"}
EXPECTED_CODES = {"C01"} | {f"G{i:02d}" for i in range(1, 54)}
EXPECTED_INACTIVE_DIRS = {"W107_G28_contract_execute_age2_observed_typed_stale_blocked"}


def fail(detail: str) -> int:
    print(f"[{SCRIPT_TAG}] fail: {detail}")
    return 1


def write_progress_snapshot(
    path_text: str,
    *,
    status: str,
    current_probe: str,
    last_completed_probe: str,
    active_cases: int,
    inactive_cases: int,
    index_codes: int,
) -> None:
    if not str(path_text).strip():
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.ci.w107_golden_index_selfcheck.progress.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": status,
        "current_case": "-",
        "last_completed_case": "-",
        "current_probe": current_probe,
        "last_completed_probe": last_completed_probe,
        "active_cases": int(active_cases),
        "inactive_cases": int(inactive_cases),
        "index_codes": int(index_codes),
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def read_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8-sig"))


def parse_case_dirs(case_root: Path) -> tuple[dict[str, str], set[str]]:
    active: dict[str, str] = {}
    inactive: set[str] = set()
    for entry in sorted(case_root.iterdir()):
        if not entry.is_dir():
            continue
        match = CASE_DIR_RE.match(entry.name)
        if not match:
            continue
        code = match.group(1)
        has_spec = (entry / "test.dtest.json").exists()
        if has_spec:
            if code in active:
                raise ValueError(f"duplicate active code: {code}")
            active[code] = entry.name
        else:
            inactive.add(entry.name)
    return active, inactive


def validate_success_expectation(*, case_dir: str, command: str, classification: str, spec: object) -> None:
    if not isinstance(spec, dict):
        raise ValueError(f"{case_dir}: spec root must be an object")
    expect = spec.get("expect")
    if not isinstance(expect, dict):
        raise ValueError(f"{case_dir}: expect must be an object")
    if expect.get("exit") != 0:
        raise ValueError(f"{case_dir}: success classification must have expect.exit=0")

    if classification == "canon_success":
        if command != "canon":
            raise ValueError(f"{case_dir}: canon_success requires command=canon")
        return

    if classification == "run_success":
        if command != "run":
            raise ValueError(f"{case_dir}: run_success requires command=run")
        if "state_hash" not in expect or str(expect.get("state_hash")).strip() == "ABSENT":
            raise ValueError(f"{case_dir}: run_success requires non-ABSENT expect.state_hash")
        if "trace_hash" not in expect or str(expect.get("trace_hash")).strip() == "ABSENT":
            raise ValueError(f"{case_dir}: run_success requires non-ABSENT expect.trace_hash")
        return

    raise ValueError(f"{case_dir}: unsupported classification={classification}")


def main() -> int:
    progress_path = os.environ.get(PROGRESS_ENV_KEY, "")
    root = Path(__file__).resolve().parents[3]
    case_root = root / "tools" / "teul-cli" / "tests" / "golden" / "W107"
    index_path = case_root / "index.json"
    indexed_case_dirs: set[str] = set()
    indexed_codes: set[str] = set()
    active_count = 0
    inactive_count = 0

    write_progress_snapshot(
        progress_path,
        status="running",
        current_probe="scan_case_root",
        last_completed_probe="-",
        active_cases=0,
        inactive_cases=0,
        index_codes=0,
    )

    def fail_probe(detail: str, *, probe: str, last_probe: str) -> int:
        write_progress_snapshot(
            progress_path,
            status="failed",
            current_probe=probe,
            last_completed_probe=last_probe,
            active_cases=active_count,
            inactive_cases=inactive_count,
            index_codes=len(indexed_codes),
        )
        return fail(detail)

    if not case_root.exists():
        return fail_probe(f"missing case root: {case_root}", probe="scan_case_root", last_probe="-")
    if not index_path.exists():
        return fail_probe(f"missing index: {index_path}", probe="scan_case_root", last_probe="-")

    try:
        active_by_dir, inactive_dirs = parse_case_dirs(case_root)
    except ValueError as exc:
        return fail_probe(str(exc), probe="scan_case_root", last_probe="-")

    active_count = len(active_by_dir)
    inactive_count = len(inactive_dirs)

    write_progress_snapshot(
        progress_path,
        status="running",
        current_probe="read_index_json",
        last_completed_probe="scan_case_root",
        active_cases=active_count,
        inactive_cases=inactive_count,
        index_codes=0,
    )

    try:
        index = read_json(index_path)
    except Exception as exc:
        return fail_probe(f"cannot read index json: {exc}", probe="read_index_json", last_probe="scan_case_root")

    if not isinstance(index, dict):
        return fail_probe("index root must be an object", probe="validate_index_meta", last_probe="read_index_json")
    if index.get("schema") != "ddn.teul_cli.golden_index.v1":
        return fail_probe("index schema mismatch", probe="validate_index_meta", last_probe="read_index_json")
    if index.get("walk") != 107:
        return fail_probe("index walk must be 107", probe="validate_index_meta", last_probe="read_index_json")

    inactive_decl = index.get("inactive_case_dirs", [])
    if not isinstance(inactive_decl, list) or any(not isinstance(item, str) for item in inactive_decl):
        return fail_probe(
            "inactive_case_dirs must be a list of strings",
            probe="validate_index_meta",
            last_probe="read_index_json",
        )
    if set(inactive_decl) != EXPECTED_INACTIVE_DIRS:
        return fail_probe(
            "inactive_case_dirs mismatch "
            f"expected={','.join(sorted(EXPECTED_INACTIVE_DIRS)) or '-'} "
            f"actual={','.join(sorted(set(inactive_decl))) or '-'}",
            probe="validate_index_meta",
            last_probe="read_index_json",
        )
    if inactive_dirs != set(inactive_decl):
        return fail_probe(
            "inactive directory scan mismatch "
            f"expected={','.join(sorted(set(inactive_decl))) or '-'} "
            f"actual={','.join(sorted(inactive_dirs)) or '-'}",
            probe="validate_index_meta",
            last_probe="read_index_json",
        )

    cases = index.get("cases")
    if not isinstance(cases, list) or not cases:
        return fail_probe(
            "index cases must be a non-empty list",
            probe="validate_index_meta",
            last_probe="read_index_json",
        )

    write_progress_snapshot(
        progress_path,
        status="running",
        current_probe="validate_case_specs",
        last_completed_probe="validate_index_meta",
        active_cases=active_count,
        inactive_cases=inactive_count,
        index_codes=0,
    )

    for idx, item in enumerate(cases):
        if not isinstance(item, dict):
            return fail_probe(
                f"cases[{idx}] must be an object",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )
        code = item.get("code")
        case_dir = item.get("case_dir")
        command = item.get("command")
        classification = item.get("classification")

        if not isinstance(code, str) or not CASE_CODE_RE.match(code):
            return fail_probe(
                f"cases[{idx}] invalid code: {code!r}",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )
        if not isinstance(case_dir, str):
            return fail_probe(
                f"cases[{idx}] case_dir must be string",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )
        if not case_dir.startswith(f"W107_{code}_"):
            return fail_probe(
                f"cases[{idx}] case_dir/code mismatch: code={code} case_dir={case_dir}",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )
        if not isinstance(command, str) or command not in ALLOWED_COMMANDS:
            return fail_probe(
                f"cases[{idx}] invalid command: {command!r}",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )
        if not isinstance(classification, str) or classification not in ALLOWED_CLASSIFICATIONS:
            return fail_probe(
                f"cases[{idx}] invalid classification: {classification!r}",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )
        if code in indexed_codes:
            return fail_probe(
                f"duplicate code in index: {code}",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )
        if case_dir in indexed_case_dirs:
            return fail_probe(
                f"duplicate case_dir in index: {case_dir}",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )

        case_dir_path = case_root / case_dir
        spec_path = case_dir_path / "test.dtest.json"
        if not case_dir_path.exists():
            return fail_probe(
                f"missing case dir: {case_dir}",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )
        if not spec_path.exists():
            return fail_probe(
                f"missing test.dtest.json for indexed case: {case_dir}",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )
        try:
            spec = read_json(spec_path)
        except Exception as exc:
            return fail_probe(
                f"cannot read test spec for {case_dir}: {exc}",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )

        if not isinstance(spec, dict):
            return fail_probe(
                f"{case_dir}: test spec root must be an object",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )
        if spec.get("command") != command:
            return fail_probe(
                f"{case_dir}: index command mismatch with test.dtest.json",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )
        try:
            validate_success_expectation(
                case_dir=case_dir,
                command=command,
                classification=classification,
                spec=spec,
            )
        except ValueError as exc:
            return fail_probe(
                str(exc),
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )

        indexed_codes.add(code)
        indexed_case_dirs.add(case_dir)

    write_progress_snapshot(
        progress_path,
        status="running",
        current_probe="validate_index_coverage",
        last_completed_probe="validate_case_specs",
        active_cases=active_count,
        inactive_cases=inactive_count,
        index_codes=len(indexed_codes),
    )

    active_dirs = set(active_by_dir.values())
    active_codes = set(active_by_dir.keys())
    if active_codes != EXPECTED_CODES:
        missing = ",".join(sorted(EXPECTED_CODES - active_codes)) or "-"
        extra = ",".join(sorted(active_codes - EXPECTED_CODES)) or "-"
        return fail_probe(
            f"active case code mismatch missing={missing} extra={extra}",
            probe="validate_index_coverage",
            last_probe="validate_case_specs",
        )
    if indexed_codes != EXPECTED_CODES:
        missing = ",".join(sorted(EXPECTED_CODES - indexed_codes)) or "-"
        extra = ",".join(sorted(indexed_codes - EXPECTED_CODES)) or "-"
        return fail_probe(
            f"index code mismatch missing={missing} extra={extra}",
            probe="validate_index_coverage",
            last_probe="validate_case_specs",
        )
    if indexed_case_dirs != active_dirs:
        missing = ",".join(sorted(active_dirs - indexed_case_dirs)) or "-"
        extra = ",".join(sorted(indexed_case_dirs - active_dirs)) or "-"
        return fail_probe(
            f"index coverage mismatch missing={missing} extra={extra}",
            probe="validate_index_coverage",
            last_probe="validate_case_specs",
        )

    write_progress_snapshot(
        progress_path,
        status="completed",
        current_probe="-",
        last_completed_probe="validate_index_coverage",
        active_cases=active_count,
        inactive_cases=inactive_count,
        index_codes=len(indexed_codes),
    )
    print(
        f"[{SCRIPT_TAG}] ok active_cases={active_count} "
        f"inactive_cases={inactive_count} "
        f"index_codes={len(indexed_codes)}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
