#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "lang_core_0_v1"


def _sort_json(value):
    if isinstance(value, list):
        return [_sort_json(item) for item in value]
    if isinstance(value, dict):
        return {key: _sort_json(value[key]) for key in sorted(value)}
    return value


def _normalize_stdout(raw: str) -> list[str]:
    return [
        line.strip()
        for line in raw.splitlines()
        if line.strip() and not line.startswith("state_hash=") and not line.startswith("trace_hash=")
    ]


def main() -> int:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    with tempfile.TemporaryDirectory(prefix="ddn_lang_core_0_") as tmp:
        input_path = Path(tmp) / "minimal.ddn"
        input_path.write_text(str(contract["minimal_ddn"]), encoding="utf-8")
        cli = subprocess.run(
            [
                "cargo",
                "run",
                "-q",
                "--manifest-path",
                "tools/teul-cli/Cargo.toml",
                "--",
                "run",
                str(input_path),
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=180,
        )
    if cli.returncode != 0:
        print(f"[lang-core-0] fail: cli minimal failed: {(cli.stderr or cli.stdout).strip()}", file=sys.stderr)
        return 1
    cli_stdout = _normalize_stdout(cli.stdout)
    if cli_stdout != contract["expected_cli_stdout"]:
        print(f"[lang-core-0] fail: cli stdout mismatch: {cli_stdout}", file=sys.stderr)
        return 1

    parity = subprocess.run(
        ["node", "--no-warnings", "tests/seamgrim_wasm_cli_runtime_parity_runner.mjs", contract["wasm_parity_pack"]],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=240,
    )
    if parity.returncode != 0:
        print(f"[lang-core-0] fail: wasm parity failed: {(parity.stderr or parity.stdout).strip()}", file=sys.stderr)
        return 1

    report = {
        "schema": "ddn.roadmap_v2.lang_core_0.report.v1",
        "ok": True,
        "surfaces": contract["surfaces"],
        "cli_minimal_stdout": cli_stdout,
        "wasm_parity_pack": contract["wasm_parity_pack"],
        "wasm_parity_pass": True,
    }
    expected = json.loads((PACK / "expected" / "lang_core_0.detjson").read_text(encoding="utf-8"))
    if _sort_json(report) != _sort_json(expected):
        print("[lang-core-0] fail: expected mismatch", file=sys.stderr)
        print(json.dumps(report, ensure_ascii=False, indent=2), file=sys.stderr)
        return 1
    print("[lang-core-0] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
