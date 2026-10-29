#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_offline_live_reaction_parity_v1"
RUNTIME = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "trace_reaction_runtime.js"
LIVE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "live_trace_reaction_runtime.js"
RUNNER = ROOT / "tests" / "seamgrim_offline_live_reaction_parity_runner.mjs"


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
        LIVE,
        RUNNER,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_OFFLINE_LIVE_PARITY_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("pack") != "seamgrim_offline_live_reaction_parity_v1":
        return fail("E_SEAMGRIM_OFFLINE_LIVE_PARITY_PACK", repr(contract))
    if contract.get("kind") != "seamgrim_offline_live_reaction_parity":
        return fail("E_SEAMGRIM_OFFLINE_LIVE_PARITY_KIND", repr(contract.get("kind")))
    if contract.get("product_behavior_claim") is not True:
        return fail("E_SEAMGRIM_OFFLINE_LIVE_PARITY_BEHAVIOR", repr(contract.get("product_behavior_claim")))
    expected = [
        "normal_sequential_input",
        "same_tick_same_payload_duplicate",
        "same_tick_different_payload_duplicate",
        "reverse_tick",
        "tick_gap",
        "lineage_transition",
        "same_property_conflict",
        "explicit_priority_resolution",
    ]
    if contract.get("fixtures") != expected:
        return fail("E_SEAMGRIM_OFFLINE_LIVE_PARITY_FIXTURES", repr(contract.get("fixtures")))
    for key in (
        "state_hash_participation",
        "state_hash_formula_change",
        "parser_change",
        "runtime_semantics_change",
        "docs_ssot_change",
        "stage_plan_created",
        "world_write",
    ):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_OFFLINE_LIVE_PARITY_BOUNDARY", f"{key}={contract.get(key)!r}")
    return 0


def check_sources() -> int:
    runtime = read(RUNTIME)
    for token in [
        "resolveActionWrites",
        "suppressed_action_rows",
        "same_priority_conflict",
        "lower_priority",
        "E_TRACE_REACTION_CONFLICT",
        "E_VIEW_REACTION_CONFLICT",
        "what:",
        "where:",
        "fix:",
        "implicit_lww_allowed",
    ]:
        if token not in runtime:
            return fail("E_SEAMGRIM_OFFLINE_LIVE_PARITY_RUNTIME_TOKEN", token)
    live = read(LIVE)
    for token in [
        "evaluateLiveTraceReactionCommit",
        "E_VIEW_LIVE_DUPLICATE_TICK_MISMATCH",
        "E_VIEW_LIVE_OUT_OF_ORDER",
        "W_VIEW_LIVE_TICK_GAP",
        "state_hash_participation: false",
    ]:
        if token not in live:
            return fail("E_SEAMGRIM_OFFLINE_LIVE_PARITY_LIVE_TOKEN", token)
    runner = read(RUNNER)
    for token in [
        "runNormalParity",
        "runDuplicateFixtures",
        "runOrderGapLineageFixtures",
        "runConflictFixture",
        "runPriorityFixture",
        "stableCanonicalJson(offline) === stableCanonicalJson(live)",
        "assertThreeLayerDiagnostics",
        "replay/scrub canonical result changed",
    ]:
        if token not in runner:
            return fail("E_SEAMGRIM_OFFLINE_LIVE_PARITY_RUNNER_TOKEN", token)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_OFFLINE_LIVE_PARITY_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_OFFLINE_LIVE_PARITY_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_contract, check_sources):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "tests/seamgrim_offline_live_reaction_parity_runner.mjs"], "E_SEAMGRIM_OFFLINE_LIVE_PARITY_NODE_CHECK", 60),
        (["node", "tests/seamgrim_offline_live_reaction_parity_runner.mjs"], "E_SEAMGRIM_OFFLINE_LIVE_PARITY_RUNNER", 120),
        (["python", "tests/run_pack_golden.py", "seamgrim_offline_live_reaction_parity_v1"], "E_SEAMGRIM_OFFLINE_LIVE_PARITY_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-offline-live-reaction-parity-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
