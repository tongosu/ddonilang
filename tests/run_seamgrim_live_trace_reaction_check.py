#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_live_trace_reaction_v1"


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
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "live_trace_reaction_runtime.js",
        ROOT / "tests" / "seamgrim_live_trace_reaction_vectors.mjs",
        ROOT / "tests" / "seamgrim_live_trace_reaction_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_LIVE_TRACE_REACTION_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("pack") != "seamgrim_live_trace_reaction_v1":
        return fail("E_SEAMGRIM_LIVE_TRACE_REACTION_PACK", repr(contract))
    if contract.get("product_behavior_claim") is not True:
        return fail("E_SEAMGRIM_LIVE_TRACE_REACTION_BEHAVIOR", repr(contract.get("product_behavior_claim")))
    for key in ("parser_change", "runtime_semantics_change", "state_hash_formula_change", "docs_ssot_change"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_LIVE_TRACE_REACTION_BOUNDARY", f"{key}={contract.get(key)!r}")
    return 0


def check_sources() -> int:
    live = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "live_trace_reaction_runtime.js")
    run_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js")
    for token in [
        "evaluateLiveTraceReactionCommit",
        "E_VIEW_LIVE_DUPLICATE_TICK_MISMATCH",
        "E_VIEW_LIVE_OUT_OF_ORDER",
        "W_VIEW_LIVE_TICK_GAP",
        "live_committed_observation",
        "state_hash_participation: false",
    ]:
        if token not in live:
            return fail("E_SEAMGRIM_LIVE_TRACE_REACTION_RUNTIME_JS", token)
    for token in [
        "evaluateLiveTraceReactionCommit",
        "__SEAMGRIM_LIVE_TRACE_REACTION_RUNTIME__",
        "liveTraceReactionRuntime",
        "liveTraceReactionLineageId",
        "liveTraceReactionOncePerTick",
    ]:
        if token not in run_js:
            return fail("E_SEAMGRIM_LIVE_TRACE_REACTION_RUN_JS", token)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_LIVE_TRACE_REACTION_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_LIVE_TRACE_REACTION_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_contract, check_sources):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "solutions/seamgrim_ui_mvp/ui/live_trace_reaction_runtime.js"], "E_SEAMGRIM_LIVE_TRACE_REACTION_RUNTIME_NODE_CHECK", 60),
        (["node", "--check", "tests/seamgrim_live_trace_reaction_vectors.mjs"], "E_SEAMGRIM_LIVE_TRACE_REACTION_VECTOR_NODE_CHECK", 60),
        (["node", "tests/seamgrim_live_trace_reaction_vectors.mjs"], "E_SEAMGRIM_LIVE_TRACE_REACTION_VECTOR", 120),
        (["node", "--check", "tests/seamgrim_live_trace_reaction_runner.mjs"], "E_SEAMGRIM_LIVE_TRACE_REACTION_BROWSER_NODE_CHECK", 60),
        (["node", "tests/seamgrim_live_trace_reaction_runner.mjs"], "E_SEAMGRIM_LIVE_TRACE_REACTION_BROWSER", 300),
        (["python", "tests/run_pack_golden.py", "seamgrim_live_trace_reaction_v1"], "E_SEAMGRIM_LIVE_TRACE_REACTION_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-live-trace-reaction-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
