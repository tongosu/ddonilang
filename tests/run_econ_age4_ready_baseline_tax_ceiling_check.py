from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "econ_age4_ready_baseline_tax_ceiling_v1"
TEUL_MANIFEST = ROOT / "tools" / "teul-cli" / "Cargo.toml"
HASH_RE = re.compile(r"^blake3:[0-9a-f]{64}$")


def load_json(path: Path) -> dict[str, Any]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError(f"{path}: expected object")
    return data


def run_command(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def run_case_card_gate() -> None:
    proc = run_command(
        [
            sys.executable,
            "tests/run_jojo_case_card_schema_check.py",
            "--dir",
            str(PACK / "case_cards"),
        ]
    )
    if proc.returncode != 0:
        raise RuntimeError(
            "case card gate failed\n"
            + (proc.stdout or "")
            + (proc.stderr or "")
        )


def run_ddn(input_path: Path) -> tuple[list[str], str, str]:
    proc = run_command(
        [
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            str(TEUL_MANIFEST),
            "--",
            "run",
            str(input_path),
        ]
    )
    if proc.returncode != 0:
        raise RuntimeError(f"{input_path}: run failed\n{proc.stdout}{proc.stderr}")

    user_lines: list[str] = []
    state_hash = ""
    trace_hash = ""
    for raw_line in proc.stdout.splitlines():
        line = raw_line.strip()
        if not line:
            continue
        if line.startswith("state_hash="):
            state_hash = line.removeprefix("state_hash=")
            continue
        if line.startswith("trace_hash="):
            trace_hash = line.removeprefix("trace_hash=")
            continue
        user_lines.append(line)

    if not HASH_RE.match(state_hash):
        raise AssertionError(f"{input_path}: missing or invalid state_hash: {state_hash!r}")
    if not HASH_RE.match(trace_hash):
        raise AssertionError(f"{input_path}: missing or invalid trace_hash: {trace_hash!r}")
    return user_lines, state_hash, trace_hash


def parse_value_lines(lines: list[str]) -> tuple[str, dict[str, int]]:
    if not lines or not lines[0].startswith("case="):
        raise AssertionError(f"missing case line: {lines}")
    case_id = lines[0].split("=", 1)[1]
    tail = lines[1:]
    if len(tail) % 2 != 0:
        raise AssertionError(f"{case_id}: expected label/value pairs: {tail}")
    values: dict[str, int] = {}
    for index in range(0, len(tail), 2):
        key = tail[index]
        raw_value = tail[index + 1]
        try:
            values[key] = int(raw_value)
        except ValueError as exc:
            raise AssertionError(f"{case_id}: {key} is not an integer: {raw_value}") from exc
    return case_id, values


def validate_expected_doc(path: Path) -> None:
    doc = load_json(path)
    if doc.get("schema") != "ddn.econ_age4_ready.expected.v1":
        raise AssertionError(f"{path}: unknown schema")
    input_path = PACK / str(doc["input_path"])
    expected_stdout = doc.get("expected_stdout")
    expected_values = doc.get("expected_values")
    if not isinstance(expected_stdout, list) or not all(isinstance(row, str) for row in expected_stdout):
        raise AssertionError(f"{path}: expected_stdout must be a string list")
    if not isinstance(expected_values, dict):
        raise AssertionError(f"{path}: expected_values must be an object")

    actual_stdout, _state_hash, _trace_hash = run_ddn(input_path)
    if actual_stdout != expected_stdout:
        raise AssertionError(f"{path}: stdout mismatch\nexpected={expected_stdout}\nactual={actual_stdout}")

    case_id, actual_values = parse_value_lines(actual_stdout)
    if case_id != doc.get("case_id"):
        raise AssertionError(f"{path}: case_id mismatch: expected {doc.get('case_id')}, got {case_id}")
    for key, expected_value in expected_values.items():
        if actual_values.get(key) != expected_value:
            raise AssertionError(f"{path}: {key} expected {expected_value}, got {actual_values.get(key)}")


def validate_scope_text() -> None:
    readme = (PACK / "README.md").read_text(encoding="utf-8")
    required = [
        "does not add a parser",
        "runtime",
        "stdlib API",
        "renderer",
        "not an economic forecast",
    ]
    lowered = readme.lower()
    for token in required:
        if token.lower() not in lowered:
            raise AssertionError(f"README.md missing scope boundary: {token}")


def main() -> int:
    run_case_card_gate()
    expected_files = sorted((PACK / "expected").glob("*.detjson"))
    if len(expected_files) != 3:
        raise AssertionError(f"expected 3 expected docs, got {len(expected_files)}")
    for path in expected_files:
        validate_expected_doc(path)
    validate_scope_text()
    print("econ_age4_ready_baseline_tax_ceiling_v1: PASS cases=3")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
