#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import re
import sys
from datetime import datetime, timezone
from pathlib import Path


SCRIPT_TAG = "w49-golden-index-selfcheck"
PROGRESS_ENV_KEY = "DDN_W49_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON"
CASE_DIR_RE = re.compile(r"^W49_(G\d{2})_")
CASE_CODE_RE = re.compile(r"^G\d{2}$")
ALLOWED_CLASSIFICATIONS = {"simulate", "replay", "diag"}


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
        "schema": "ddn.ci.w49_golden_index_selfcheck.progress.v1",
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


def validate_case_expectations(*, case_dir: str, classification: str, spec: object) -> None:
    if not isinstance(spec, dict):
        raise ValueError(f"{case_dir}: spec root must be an object")
    expect = spec.get("expect")
    if not isinstance(expect, dict):
        raise ValueError(f"{case_dir}: expect must be an object")
    if expect.get("exit") != 0:
        raise ValueError(f"{case_dir}: expect.exit must be 0")
    command = spec.get("command")

    if classification == "simulate":
        if command != "latency_simulate":
            raise ValueError(f"{case_dir}: classification=simulate requires command=latency_simulate")
        stdout_text = expect.get("stdout_text")
        if not isinstance(stdout_text, str):
            raise ValueError(f"{case_dir}: classification=simulate requires expect.stdout_text")
        if "latency_hash=sha256:" not in stdout_text:
            raise ValueError(f"{case_dir}: simulate stdout_text must include latency_hash")
        return

    if classification == "replay":
        if command != "multi":
            raise ValueError(f"{case_dir}: classification=replay requires command=multi")
        stdout = expect_list_of_str(expect.get("stdout", []), "expect.stdout", case_dir)
        if "verify_ok=true" not in stdout:
            raise ValueError(f"{case_dir}: replay expect.stdout must include verify_ok=true")
        state_hash = expect.get("state_hash")
        if not isinstance(state_hash, str):
            raise ValueError(f"{case_dir}: replay expect.state_hash must be a string")
        if not (state_hash.startswith("blake3:") or state_hash.startswith("DIFFERS_FROM:")):
            raise ValueError(f"{case_dir}: replay expect.state_hash contract mismatch")
        return

    if classification == "diag":
        if command != "run":
            raise ValueError(f"{case_dir}: classification=diag requires command=run")
        files = expect.get("files")
        if not isinstance(files, dict):
            raise ValueError(f"{case_dir}: diag expect.files must be an object")
        diag_file = files.get("geoul.diag.jsonl")
        if not isinstance(diag_file, dict):
            raise ValueError(f"{case_dir}: diag requires geoul.diag.jsonl file contract")
        jsonl_contains = diag_file.get("jsonl_contains")
        if not isinstance(jsonl_contains, list) or not jsonl_contains:
            raise ValueError(f"{case_dir}: diag jsonl_contains must be a non-empty list")
        if not any(
            isinstance(item, dict) and item.get("event") == "latency_schedule" for item in jsonl_contains
        ):
            raise ValueError(f"{case_dir}: diag jsonl_contains must include latency_schedule")
        return

    raise ValueError(f"{case_dir}: unsupported classification={classification}")


def main() -> int:
    progress_path = os.environ.get(PROGRESS_ENV_KEY, "")
    root = Path(__file__).resolve().parent
    case_root = root / "golden" / "W49"
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
    if index.get("walk") != 49:
        return fail_probe("index walk must be 49", probe="validate_index_meta", last_probe="read_index_json")

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
        if not case_dir.startswith(f"W49_{code}_"):
            return fail_probe(
                f"cases[{idx}] case_dir/code mismatch: code={code} case_dir={case_dir}",
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
        if spec.get("walk") != 49:
            return fail_probe(
                f"{case_dir}: spec walk must be 49",
                probe="validate_case_specs",
                last_probe="validate_index_meta",
            )

        try:
            validate_case_expectations(case_dir=case_dir, classification=classification, spec=spec)
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

    expected_codes = {f"G{i:02d}" for i in range(1, 8)}
    if indexed_codes != expected_codes:
        missing_codes = sorted(expected_codes - indexed_codes)
        extra_codes = sorted(indexed_codes - expected_codes)
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
        f"simulate={sum(1 for c in cases if c.get('classification') == 'simulate')} "
        f"replay={sum(1 for c in cases if c.get('classification') == 'replay')} "
        f"diag={sum(1 for c in cases if c.get('classification') == 'diag')}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
