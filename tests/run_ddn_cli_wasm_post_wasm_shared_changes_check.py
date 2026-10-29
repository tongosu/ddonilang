#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "tests" / "ddn_cli_wasm_post_wasm_shared_changes_runner.mjs"
CONTRACT = ROOT / "pack" / "ddn_cli_wasm_sync_gate_v1" / "post_wasm_shared_changes.detjson"


def fail(message: str) -> int:
    print(f"[ddn-cli-wasm-post-wasm-shared] fail: {message}")
    return 1


def run_cmd(cmd: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )


def validate_report(report: dict, contract: dict) -> list[str]:
    failures: list[str] = []
    if report.get("schema") != "ddn.cli_wasm.post_wasm_shared_changes.report.v1":
        failures.append(f"bad report schema: {report.get('schema')}")
    if report.get("ok") is not True:
        failures.append("runner ok=false")
    checks = report.get("checks")
    if not isinstance(checks, dict):
        failures.append("checks missing")
        return failures
    api = checks.get("public_wasm_api", {})
    if api.get("has_get_input_state_json") is not True:
        failures.append("get_input_state_json export not proven")
    if api.get("has_wrapper_inputStateParsed") is not True:
        failures.append("wrapper inputStateParsed not proven")
    if checks.get("configured_madi") != 3:
        failures.append(f"configured_madi expected 3 got {checks.get('configured_madi')}")
    for key in ("initial_input", "reset_input", "restore_input"):
        item = checks.get(key)
        if not isinstance(item, dict):
            failures.append(f"{key} missing")
            continue
        if item.get("frame_source") != "사람" or item.get("frame_source_code") != 0:
            failures.append(f"{key}: person frame source not preserved")
    step = checks.get("step_input", {})
    for key, expected in {
        "keys_pressed": 8,
        "last_key_name": "d",
        "pointer_x_i32": 12,
        "pointer_y_i32": -34,
        "dt": "0.5",
        "frame_source": "사람",
        "frame_source_code": 0,
    }.items():
        if step.get(key) != expected:
            failures.append(f"step_input.{key} expected {expected!r} got {step.get(key)!r}")
    ai = checks.get("seulgi_injection", {})
    if ai.get("source") != "슬기" or ai.get("source_code") != 1:
        failures.append("seulgi injection source not serialized")
    if ai.get("intent_kind") != "say":
        failures.append("seulgi intent kind not serialized")
    after_ai = checks.get("after_ai_input_state", {})
    if after_ai.get("pending_ai_injection_count") != 0:
        failures.append("pending AI injections did not drain after step")
    restored = checks.get("restore_input", {})
    for key, expected in {
        "keys_pressed": 2,
        "last_key_name": "a",
        "pointer_x_i32": 7,
        "pointer_y_i32": 9,
        "dt": "0.25",
    }.items():
        if restored.get(key) != expected:
            failures.append(f"restore_input.{key} expected {expected!r} got {restored.get(key)!r}")
    state_hash = checks.get("state_hash_exposed", {})
    if not str(state_hash.get("state_hash", "")).startswith("blake3:"):
        failures.append("state hash exposure missing")
    if state_hash.get("formula_change") is not False:
        failures.append("state hash formula_change must be false")
    limits = report.get("limits", {})
    if limits.get("state_hash_formula_change") is not False:
        failures.append("report must deny state_hash formula change")
    for field in contract.get("required_report_checks", []):
        if field not in checks:
            failures.append(f"contract required check missing: {field}")
    return failures


def run_selftest(contract: dict) -> list[str]:
    base = {
        "schema": "ddn.cli_wasm.post_wasm_shared_changes.report.v1",
        "ok": True,
        "checks": {
            "public_wasm_api": {
                "has_get_input_state_json": True,
                "has_wrapper_inputStateParsed": True,
            },
            "configured_madi": 3,
            "initial_input": {"frame_source": "사람", "frame_source_code": 0},
            "step_input": {
                "keys_pressed": 8,
                "last_key_name": "d",
                "pointer_x_i32": 12,
                "pointer_y_i32": -34,
                "dt": "0.5",
                "frame_source": "사람",
                "frame_source_code": 0,
            },
            "seulgi_injection": {"source": "슬기", "source_code": 1, "intent_kind": "say"},
            "after_ai_input_state": {"frame_source": "사람", "pending_ai_injection_count": 0},
            "reset_input": {
                "frame_source": "사람",
                "frame_source_code": 0,
                "keys_pressed": 0,
                "last_key_name": "",
            },
            "restore_input": {
                "frame_source": "사람",
                "frame_source_code": 0,
                "keys_pressed": 2,
                "last_key_name": "a",
                "pointer_x_i32": 7,
                "pointer_y_i32": 9,
                "dt": "0.25",
            },
            "state_hash_exposed": {"state_hash": "blake3:" + "0" * 64, "formula_change": False},
        },
        "limits": {"state_hash_formula_change": False},
    }

    def mutate(mutator):
        doc = json.loads(json.dumps(base, ensure_ascii=False))
        mutator(doc)
        return doc

    cases = {
        "missing_api": lambda doc: doc["checks"]["public_wasm_api"].__setitem__("has_get_input_state_json", False),
        "wrong_frame_source": lambda doc: doc["checks"]["step_input"].__setitem__("frame_source", "슬기"),
        "missing_ai_source": lambda doc: doc["checks"]["seulgi_injection"].__setitem__("source", ""),
        "restore_not_preserved": lambda doc: doc["checks"]["restore_input"].__setitem__("keys_pressed", 0),
        "hash_formula_change": lambda doc: doc["checks"]["state_hash_exposed"].__setitem__("formula_change", True),
    }
    failures: list[str] = []
    for name, mutator in cases.items():
        if not validate_report(mutate(mutator), contract):
            failures.append(f"selftest {name} unexpectedly passed")
    return failures


def main() -> int:
    if not RUNNER.exists():
        return fail(f"missing runner: {RUNNER}")
    if not CONTRACT.exists():
        return fail(f"missing contract: {CONTRACT}")
    contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.cli_wasm.post_wasm_shared_changes.contract.v1":
        return fail(f"bad contract schema: {contract.get('schema')}")
    if contract.get("state_hash_formula_change") is not False:
        return fail("contract must deny state_hash formula change")

    proc = run_cmd(["node", "--no-warnings", str(RUNNER)])
    if proc.returncode != 0:
        return fail(proc.stderr.strip() or proc.stdout.strip() or f"runner rc={proc.returncode}")
    try:
        report = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        return fail(f"runner emitted invalid json: {exc}")
    failures = validate_report(report, contract)
    failures.extend(run_selftest(contract))

    for test_name in contract.get("required_cargo_tests", []):
        cargo = run_cmd(["cargo", "test", "-p", "ddonirang-tool", str(test_name), "--", "--nocapture"], timeout=240)
        if cargo.returncode != 0:
            failures.append(f"cargo test {test_name} failed: {(cargo.stderr or cargo.stdout).strip()[:400]}")

    if failures:
        return fail("; ".join(failures[:8]))
    print("ddn cli/wasm post-wasm shared changes ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
