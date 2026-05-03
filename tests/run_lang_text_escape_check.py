#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "lang_text_escape_v1"


def _sort_json(value):
    if isinstance(value, list):
        return [_sort_json(item) for item in value]
    if isinstance(value, dict):
        return {key: _sort_json(value[key]) for key in sorted(value)}
    return value


def _run(args: list[str], timeout: int = 240) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )


def fail(message: str) -> int:
    print(f"[lang-text-escape] fail: {message}", file=sys.stderr)
    return 1


def main() -> int:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    parity = _run(["node", "--no-warnings", "tests/seamgrim_wasm_cli_runtime_parity_runner.mjs", str(PACK)], timeout=300)
    if parity.returncode != 0:
        return fail((parity.stderr or parity.stdout).strip())

    canon_checks = [
        ("cases/c01_hangul_escape_run/input.ddn", ['"첫\\n둘"', '"A\\tB"', '"그는 \\"안녕\\""', '"C\\\\D"']),
        ("cases/c02_compat_escape_run/input.ddn", ['"첫\\n둘"', '"A\\tB"', '"그는 \\"안녕\\""', '"C\\\\D"']),
        ("cases/c03_rich_markup_preserved/input.ddn", ['"\\\\색{빨강}경고\\\\되돌림"']),
        ("cases/c04_carriage_return_canon/input.ddn", ['"왼쪽\\r오른쪽"']),
    ]
    for rel, tokens in canon_checks:
        proc = _run([
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            "canon",
            str(PACK / rel),
            "--emit",
            "ddn",
        ])
        if proc.returncode != 0:
            return fail(f"canon failed for {rel}: {(proc.stderr or proc.stdout).strip()}")
        for token in tokens:
            if token not in proc.stdout:
                return fail(f"canon token missing for {rel}: {token}\n{proc.stdout}")

    invalid = contract["invalid_cases"][0]
    bad = _run([
        "cargo",
        "run",
        "-q",
        "--manifest-path",
        "tools/teul-cli/Cargo.toml",
        "--",
        "run",
        str(PACK / invalid["input"]),
    ])
    if bad.returncode == 0:
        return fail("unknown escape unexpectedly passed")
    if str(invalid["expected_error"]) not in f"{bad.stderr}\n{bad.stdout}":
        return fail(f"unknown escape error mismatch: {(bad.stderr or bad.stdout).strip()}")

    report = {
        "schema": "ddn.lang_text_escape.report.v1",
        "ok": True,
        "pack_id": contract["pack_id"],
        "wasm_parity_pass": True,
        "canon_policy": {
            "hangul_escape_input": True,
            "compat_escape_input": True,
            "canonical_escape_output": "standard",
            "rich_markup_preserved_as_string": True,
        },
        "invalid_escape_error": invalid["expected_error"],
    }
    expected = json.loads((PACK / "expected" / "lang_text_escape.detjson").read_text(encoding="utf-8"))
    if _sort_json(report) != _sort_json(expected):
        return fail(json.dumps(report, ensure_ascii=False, indent=2))
    print("[lang-text-escape] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
