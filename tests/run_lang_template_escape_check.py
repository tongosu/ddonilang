#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "lang_template_escape_v1"


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


def _stdout_lines(raw: str) -> list[str]:
    return [
        line.strip()
        for line in raw.splitlines()
        if line.strip() and not line.startswith("state_hash=") and not line.startswith("trace_hash=")
    ]


def fail(message: str) -> int:
    print(f"[lang-template-escape] fail: {message}", file=sys.stderr)
    return 1


def main() -> int:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    for case in contract["valid_cases"]:
        input_path = PACK / case["input"]
        run = _run([
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            "run",
            str(input_path),
        ])
        if run.returncode != 0:
            return fail(f"{case['id']} run failed: {(run.stderr or run.stdout).strip()}")
        actual_stdout = _stdout_lines(run.stdout)
        if actual_stdout != case["expected_stdout"]:
            return fail(f"{case['id']} stdout mismatch: {actual_stdout}")
        canon = _run([
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            "canon",
            str(input_path),
            "--emit",
            "ddn",
        ])
        if canon.returncode != 0:
            return fail(f"{case['id']} canon failed: {(canon.stderr or canon.stdout).strip()}")
        for token in case["expected_canon_tokens"]:
            if token not in canon.stdout:
                return fail(f"{case['id']} canon token missing: {token}\n{canon.stdout}")

    for case in contract["invalid_cases"]:
        bad = _run([
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            "run",
            str(PACK / case["input"]),
        ])
        if bad.returncode == 0:
            return fail(f"{case['id']} unexpectedly passed")
        if case["expected_error"] not in f"{bad.stderr}\n{bad.stdout}":
            return fail(f"{case['id']} error mismatch: {(bad.stderr or bad.stdout).strip()}")

    report = {
        "schema": "ddn.lang_template_escape.report.v1",
        "ok": True,
        "pack_id": contract["pack_id"],
        "valid_case_count": len(contract["valid_cases"]),
        "invalid_case_count": len(contract["invalid_cases"]),
        "template_escape_policy": {
            "hangul_escape_input": True,
            "literal_brace_escape": "{{ }}",
            "placeholder_required": True,
        },
    }
    expected = json.loads((PACK / "expected" / "lang_template_escape.detjson").read_text(encoding="utf-8"))
    if _sort_json(report) != _sort_json(expected):
        return fail(json.dumps(report, ensure_ascii=False, indent=2))
    print("[lang-template-escape] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
