#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import re
from datetime import datetime, timezone
from pathlib import Path


SCRIPT_TAG = "w109-golden-index-selfcheck"
PROGRESS_ENV_KEY = "DDN_W109_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON"
CASE_DIR_RE = re.compile(r"^W109_([BCGKL]\d{2})_")
CASE_CODE_RE = re.compile(r"^[BCGKL]\d{2}$")
ALLOWED_COMMANDS = {"build", "canon", "run", "check", "lint"}
ALLOWED_CLASSIFICATIONS = {
    "build_parse_error",
    "canon_success",
    "canon_error",
    "run_success",
    "run_parse_error",
    "check_parse_error",
    "lint_parse_error",
}
EXPECTED_CODES = {
    "B01",
    "B02",
    "B03",
    "C01",
    "C02",
    "C03",
    "C04",
    "C05",
    "G01",
    "G02",
    "G03",
    "G04",
    "G05",
    "G06",
    "K01",
    "K02",
    "K03",
    "L01",
    "L02",
    "L03",
}


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
    index_codes: int,
) -> None:
    if not str(path_text).strip():
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.ci.w109_golden_index_selfcheck.progress.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": status,
        "current_case": "-",
        "last_completed_case": "-",
        "current_probe": current_probe,
        "last_completed_probe": last_completed_probe,
        "active_cases": int(active_cases),
        "index_codes": int(index_codes),
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def read_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8-sig"))


def expect_list_of_str(value: object, field_name: str, case_dir: str) -> list[str]:
    if not isinstance(value, list):
        raise ValueError(f"{case_dir}: {field_name} must be a list")
    out: list[str] = []
    for item in value:
        if not isinstance(item, str):
            raise ValueError(f"{case_dir}: {field_name} must contain only strings")
        out.append(item)
    return out


def require_parse_error_contract(
    *,
    case_dir: str,
    command: str,
    expect: dict,
    diag_code: str | None,
) -> None:
    if expect.get("exit") != 1:
        raise ValueError(f"{case_dir}: {command} parse_error must have expect.exit=1")
    stderr_contains = expect_list_of_str(expect.get("stderr_contains", []), "expect.stderr_contains", case_dir)
    if not stderr_contains:
        raise ValueError(f"{case_dir}: {command} parse_error requires non-empty expect.stderr_contains")
    if not any("E_PARSE_" in text for text in stderr_contains):
        raise ValueError(f"{case_dir}: {command} parse_error must include E_PARSE_* in expect.stderr_contains")
    if diag_code and diag_code not in stderr_contains:
        raise ValueError(f"{case_dir}: missing diag_code={diag_code} in expect.stderr_contains")
    state_hash = expect.get("state_hash")
    if state_hash != "ABSENT":
        raise ValueError(f"{case_dir}: {command} parse_error must have expect.state_hash=ABSENT")
    if command == "run":
        if expect.get("trace_hash") != "ABSENT":
            raise ValueError(f"{case_dir}: run parse_error must have expect.trace_hash=ABSENT")


def validate_case_expectations(
    *,
    case_dir: str,
    command: str,
    classification: str,
    diag_code: str | None,
    requires_diag_fixit: bool,
    spec: object,
) -> None:
    if not isinstance(spec, dict):
        raise ValueError(f"{case_dir}: spec root must be an object")
    expect = spec.get("expect")
    if not isinstance(expect, dict):
        raise ValueError(f"{case_dir}: expect must be an object")

    if classification == "canon_success":
        if command != "canon":
            raise ValueError(f"{case_dir}: canon_success requires command=canon")
        if expect.get("exit") != 0:
            raise ValueError(f"{case_dir}: canon_success must have expect.exit=0")
        return

    if classification == "canon_error":
        if command != "canon":
            raise ValueError(f"{case_dir}: canon_error requires command=canon")
        if expect.get("exit") != 1:
            raise ValueError(f"{case_dir}: canon_error must have expect.exit=1")
        stderr_contains = expect_list_of_str(expect.get("stderr_contains", []), "expect.stderr_contains", case_dir)
        if not stderr_contains:
            raise ValueError(f"{case_dir}: canon_error requires non-empty expect.stderr_contains")
        if not any("E_CANON_" in text for text in stderr_contains):
            raise ValueError(f"{case_dir}: canon_error must include E_CANON_* in expect.stderr_contains")
        if diag_code and diag_code not in stderr_contains:
            raise ValueError(f"{case_dir}: missing diag_code={diag_code} in expect.stderr_contains")
        if requires_diag_fixit:
            args = spec.get("args")
            if not isinstance(args, dict):
                raise ValueError(f"{case_dir}: requires_diag_fixit expects args object")
            cli = args.get("cli")
            if not isinstance(cli, list):
                raise ValueError(f"{case_dir}: requires_diag_fixit expects args.cli list")
            for required in ("--check", "--fixits-json", "--diag-jsonl"):
                if required not in cli:
                    raise ValueError(f"{case_dir}: requires_diag_fixit missing cli flag {required}")
            files = expect.get("files")
            if not isinstance(files, dict):
                raise ValueError(f"{case_dir}: requires_diag_fixit expects expect.files object")
            if "diag.jsonl" not in files or "fixits.json" not in files:
                raise ValueError(f"{case_dir}: requires_diag_fixit expects diag.jsonl and fixits.json contracts")
        return

    if classification == "run_success":
        if command != "run":
            raise ValueError(f"{case_dir}: run_success requires command=run")
        if expect.get("exit") != 0:
            raise ValueError(f"{case_dir}: run_success must have expect.exit=0")
        return

    if classification == "run_parse_error":
        if command != "run":
            raise ValueError(f"{case_dir}: run_parse_error requires command=run")
        require_parse_error_contract(case_dir=case_dir, command=command, expect=expect, diag_code=diag_code)
        return

    if classification == "check_parse_error":
        if command != "check":
            raise ValueError(f"{case_dir}: check_parse_error requires command=check")
        require_parse_error_contract(case_dir=case_dir, command=command, expect=expect, diag_code=diag_code)
        return

    if classification == "lint_parse_error":
        if command != "lint":
            raise ValueError(f"{case_dir}: lint_parse_error requires command=lint")
        require_parse_error_contract(case_dir=case_dir, command=command, expect=expect, diag_code=diag_code)
        return

    if classification == "build_parse_error":
        if command != "build":
            raise ValueError(f"{case_dir}: build_parse_error requires command=build")
        require_parse_error_contract(case_dir=case_dir, command=command, expect=expect, diag_code=diag_code)
        return

    raise ValueError(f"{case_dir}: unsupported classification={classification}")


def main() -> int:
    progress_path = os.environ.get(PROGRESS_ENV_KEY, "")
    root = Path(__file__).resolve().parent
    case_root = root / "golden" / "W109"
    index_path = case_root / "index.json"
    indexed_case_dirs: set[str] = set()
    indexed_codes: set[str] = set()

    write_progress_snapshot(
        progress_path,
        status="running",
        current_probe="scan_case_root",
        last_completed_probe="-",
        active_cases=0,
        index_codes=0,
    )

    def fail_probe(detail: str, *, probe: str, last_probe: str) -> int:
        write_progress_snapshot(
            progress_path,
            status="failed",
            current_probe=probe,
            last_completed_probe=last_probe,
            active_cases=len(indexed_case_dirs),
            index_codes=len(indexed_codes),
        )
        return fail(detail)

    if not case_root.exists():
        return fail_probe(f"missing case root: {case_root}", probe="scan_case_root", last_probe="-")
    if not index_path.exists():
        return fail_probe(f"missing index: {index_path}", probe="scan_case_root", last_probe="-")

    try:
        index = read_json(index_path)
    except Exception as exc:
        return fail_probe(f"cannot read index json: {exc}", probe="read_index_json", last_probe="scan_case_root")

    if not isinstance(index, dict):
        return fail_probe("index root must be an object", probe="validate_index_meta", last_probe="read_index_json")
    if index.get("schema") != "ddn.teul_cli.golden_index.v1":
        return fail_probe("index schema mismatch", probe="validate_index_meta", last_probe="read_index_json")
    if index.get("walk") != 109:
        return fail_probe("index walk must be 109", probe="validate_index_meta", last_probe="read_index_json")

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
        active_cases=0,
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
        diag_code = item.get("diag_code")
        requires_diag_fixit = bool(item.get("requires_diag_fixit", False))

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
        if not case_dir.startswith(f"W109_{code}_"):
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
        if diag_code is not None and not isinstance(diag_code, str):
            return fail_probe(
                f"cases[{idx}] diag_code must be string when present",
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
        indexed_codes.add(code)
        indexed_case_dirs.add(case_dir)

        case_path = case_root / case_dir
        spec_path = case_path / "test.dtest.json"
        if not case_path.is_dir():
            return fail_probe(
                f"missing case directory: {case_dir}",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )
        if not spec_path.exists():
            return fail_probe(
                f"missing test spec: {spec_path}",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )

        try:
            spec = read_json(spec_path)
        except Exception as exc:
            return fail_probe(
                f"cannot read spec {spec_path}: {exc}",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )

        if not isinstance(spec, dict):
            return fail_probe(
                f"{case_dir}: spec root must be an object",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )
        if spec.get("name") != case_dir:
            return fail_probe(
                f"{case_dir}: spec name must match case_dir",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )
        if spec.get("walk") != 109:
            return fail_probe(
                f"{case_dir}: spec walk must be 109",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )
        if spec.get("command") != command:
            return fail_probe(
                f"{case_dir}: spec command must match index command ({command})",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )

        try:
            validate_case_expectations(
                case_dir=case_dir,
                command=command,
                classification=classification,
                diag_code=diag_code,
                requires_diag_fixit=requires_diag_fixit,
                spec=spec,
            )
        except ValueError as exc:
            return fail_probe(
                str(exc),
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )

    write_progress_snapshot(
        progress_path,
        status="running",
        current_probe="validate_index_coverage",
        last_completed_probe="validate_case_specs",
        active_cases=len(indexed_case_dirs),
        index_codes=len(indexed_codes),
    )

    actual_case_dirs: set[str] = set()
    for entry in sorted(case_root.iterdir()):
        if not entry.is_dir():
            continue
        if CASE_DIR_RE.match(entry.name):
            actual_case_dirs.add(entry.name)

    missing_in_index = sorted(actual_case_dirs - indexed_case_dirs)
    extra_in_index = sorted(indexed_case_dirs - actual_case_dirs)
    if missing_in_index or extra_in_index:
        return fail_probe(
            "index coverage mismatch "
            f"missing_in_index={','.join(missing_in_index) or '-'} "
            f"extra_in_index={','.join(extra_in_index) or '-'}",
            probe="validate_index_coverage",
            last_probe="validate_case_specs",
        )

    if indexed_codes != EXPECTED_CODES:
        missing_codes = sorted(EXPECTED_CODES - indexed_codes)
        extra_codes = sorted(indexed_codes - EXPECTED_CODES)
        return fail_probe(
            "code set mismatch "
            f"missing={','.join(missing_codes) or '-'} "
            f"extra={','.join(extra_codes) or '-'}",
            probe="validate_index_coverage",
            last_probe="validate_case_specs",
        )

    write_progress_snapshot(
        progress_path,
        status="completed",
        current_probe="-",
        last_completed_probe="validate_index_coverage",
        active_cases=len(indexed_case_dirs),
        index_codes=len(indexed_codes),
    )

    print(
        f"[{SCRIPT_TAG}] ok "
        f"cases={len(indexed_case_dirs)} "
        f"canon_errors={sum(1 for c in cases if c.get('classification') == 'canon_error')} "
        f"parse_errors={sum(1 for c in cases if 'parse_error' in str(c.get('classification')))}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
