#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BUILD = ROOT / "build" / "declarative_closure_g4_connect"


def fail(message: str) -> None:
    print(f"declarative_closure_g4_connect_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def run(cmd: list[str], *, timeout: int = 240) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
        check=False,
    )


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8-sig"))


def run_solveplan(input_path: Path, out_path: Path) -> subprocess.CompletedProcess[str]:
    out_path.parent.mkdir(parents=True, exist_ok=True)
    return run(
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            "run",
            str(input_path),
            "--solveplan-out",
            str(out_path),
        ],
        timeout=300,
    )


def check_pack_golden() -> None:
    proc = run(
        [
            sys.executable,
            "tests/run_pack_golden.py",
            "lang_connect_lowering_v1",
            "sim_circuit_series_v1",
            "sim_econ_market_v1",
            "lang_connect_diag_v1",
        ],
        timeout=300,
    )
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_connect_artifact() -> None:
    expected = read_json(ROOT / "pack" / "lang_connect_lowering_v1" / "expected_solveplan_connect.detjson")
    actual_path = BUILD / "lang_connect_lowering_v1.detjson"
    proc = run_solveplan(ROOT / "pack" / "lang_connect_lowering_v1" / "input.ddn", actual_path)
    if proc.returncode != 0:
        fail(f"connect solveplan run failed:\n{proc.stdout}")
    actual = read_json(actual_path)
    if actual != expected:
        fail(f"connect artifact mismatch:\nexpected={expected!r}\nactual={actual!r}")
    connect = actual.get("lowering", {}).get("connect_desugar", {})
    if connect.get("sign_convention") != "incoming_positive_v1":
        fail(f"sign convention mismatch: {connect!r}")
    if connect.get("pure_desugar") is not True:
        fail(f"connect must remain pure desugar: {connect!r}")


def check_econ_artifact() -> None:
    actual_path = BUILD / "sim_econ_market_v1.detjson"
    proc = run_solveplan(ROOT / "pack" / "sim_econ_market_v1" / "input.ddn", actual_path)
    if proc.returncode != 0:
        fail(f"econ solveplan run failed:\n{proc.stdout}")
    actual = read_json(actual_path)
    rows = actual.get("lowering", {}).get("connect_desugar", {}).get("rows")
    if rows != [
        "가계1.구매끝.체결값 =:= 장터.소매끝.체결값.",
        "가계1.구매끝.재화 + 장터.소매끝.재화 =:= 0.",
    ]:
        fail(f"econ rows mismatch: {rows!r}")


def check_diag_artifacts() -> None:
    cases = [
        ("c01_channel_mismatch.ddn", "E_CONNECT_CHANNEL_MISMATCH", 0),
        ("c02_unsupported_rule.ddn", "E_CONNECT_UNSUPPORTED_RULE", 0),
        ("c03_no_port.ddn", "E_CONNECT_NO_PORT", 1),
    ]
    for filename, code, exit_code in cases:
        actual_path = BUILD / f"{filename}.detjson"
        proc = run_solveplan(ROOT / "pack" / "lang_connect_diag_v1" / filename, actual_path)
        if proc.returncode != exit_code:
            fail(f"{filename} exit mismatch expected={exit_code} got={proc.returncode}\n{proc.stdout}")
        actual = read_json(actual_path)
        diagnostics = actual.get("diagnostics", [])
        got = [diag.get("code") for diag in diagnostics]
        if got != [code]:
            fail(f"{filename} diagnostics mismatch: {got!r}")
        if actual.get("method") != "connect_desugar_unsupported_v1":
            fail(f"{filename} method mismatch: {actual.get('method')!r}")


def check_no_docs_ssot_change() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed:\n{proc.stdout}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_pack_golden()
    check_connect_artifact()
    check_econ_artifact()
    check_diag_artifacts()
    check_no_docs_ssot_change()
    print("declarative_closure_g4_connect_check: ok")


if __name__ == "__main__":
    main()
