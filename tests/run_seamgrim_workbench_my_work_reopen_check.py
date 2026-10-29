#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_workbench_my_work_reopen_v1"
RUNNER = ROOT / "tests" / "seamgrim_workbench_my_work_reopen_runner.mjs"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def run(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
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


def main() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "golden.jsonl",
        PACK / "input.ddn",
        RUNNER,
        ROOT / "solutions/seamgrim_ui_mvp/ui/workbench/workbench.js",
        ROOT / "solutions/seamgrim_ui_mvp/ui/workbench/index.html",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_WORKBENCH_G7_MISSING", json.dumps(missing, ensure_ascii=False))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    for key, expected in {
        "pack": "seamgrim_workbench_my_work_reopen_v1",
        "gate": "G7",
        "product_behavior_change": True,
        "behavior_closed_claim": True,
        "local_capability_index": True,
        "server_enumeration_endpoint": False,
        "restart_reopen_required": True,
        "raw_capability_in_dom": False,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
    }.items():
        if contract.get(key) != expected:
            return fail("E_SEAMGRIM_WORKBENCH_G7_CONTRACT", f"{key}={contract.get(key)!r}")
    js = (ROOT / "solutions/seamgrim_ui_mvp/ui/workbench/workbench.js").read_text(encoding="utf-8")
    for token in [
        "MY_WORK_INDEX_KEY",
        "rememberMyWorkProject",
        "refreshMyWorkIndex",
        "my-work-item",
        "server 확인됨",
    ]:
        if token not in js:
            return fail("E_SEAMGRIM_WORKBENCH_G7_JS_TOKEN", token)
    for cmd in [
        ["python", "tests/run_pack_golden.py", "seamgrim_workbench_my_work_reopen_v1"],
        ["node", str(RUNNER.relative_to(ROOT))],
    ]:
        proc = run(cmd, timeout=320)
        if proc.returncode:
            return fail("E_SEAMGRIM_WORKBENCH_G7_CMD", f"{cmd!r}: {proc.stdout.strip()}")
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_WORKBENCH_G7_SSOT", ssot.stdout.strip())
    print("[seamgrim-workbench-my-work-reopen-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
