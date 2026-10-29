#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_sealed_observation_trace_v1"
RUNTIME = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "sealed_trace_evaluator.js"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(cmd: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def check_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNTIME,
        ROOT / "tests" / "seamgrim_sealed_observation_trace_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_SEALED_TRACE_ARTIFACT_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("trace_schema") != "seamgrim.sealed_observation_trace.v1":
        return fail("E_SEAMGRIM_SEALED_TRACE_ARTIFACT_SCHEMA", repr(contract))
    if contract.get("required_values") != ["tick_id", "t", "theta", "omega"]:
        return fail("E_SEAMGRIM_SEALED_TRACE_ARTIFACT_REQUIRED_VALUES", repr(contract.get("required_values")))
    for key in ("console_string_parse", "state_hash_participation", "condition_evaluator_claim", "parser_change", "runtime_semantics_change", "docs_ssot_change"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_SEALED_TRACE_ARTIFACT_BOUNDARY", f"{key}={contract.get(key)!r}")
    source = read(RUNTIME)
    for token in [
        "seamgrim.sealed_observation_trace.v1",
        "seamgrim_product_wasm",
        "console_string_parse: false",
        "state_hash_participation: false",
        "trace_hash_candidate",
    ]:
        if token not in source:
            return fail("E_SEAMGRIM_SEALED_TRACE_ARTIFACT_SOURCE", token)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_SEALED_TRACE_ARTIFACT_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_SEALED_TRACE_ARTIFACT_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    rc = check_contract()
    if rc:
        return rc
    gates = [
        (["node", "--check", "solutions/seamgrim_ui_mvp/ui/sealed_trace_evaluator.js"], "E_SEAMGRIM_SEALED_TRACE_ARTIFACT_NODE_CHECK", 60),
        (["node", "--check", "tests/seamgrim_sealed_observation_trace_runner.mjs"], "E_SEAMGRIM_SEALED_TRACE_ARTIFACT_RUNNER_NODE_CHECK", 60),
        (["node", "tests/seamgrim_sealed_observation_trace_runner.mjs"], "E_SEAMGRIM_SEALED_TRACE_ARTIFACT_BROWSER", 240),
        (["python", "tests/run_pack_golden.py", "seamgrim_sealed_observation_trace_v1"], "E_SEAMGRIM_SEALED_TRACE_ARTIFACT_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-sealed-observation-trace-v1] OK artifact-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
