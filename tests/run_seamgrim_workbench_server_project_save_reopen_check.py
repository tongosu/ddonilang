#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_workbench_server_project_save_reopen_v1"
RUNNER = ROOT / "tests" / "seamgrim_workbench_server_project_save_reopen_runner.mjs"


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
        ROOT / "solutions/seamgrim_ui_mvp/ui/workbench/index.html",
        ROOT / "solutions/seamgrim_ui_mvp/ui/workbench/workbench.js",
        ROOT / "solutions/seamgrim_ui_mvp/ui/workbench/persistence/project_adapter.js",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_WORKBENCH_G2_MISSING", json.dumps(missing, ensure_ascii=False))

    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "pack": "seamgrim_workbench_server_project_save_reopen_v1",
        "gate": "G2",
        "product_behavior_change": True,
        "behavior_closed_claim": True,
        "parallel_project_store_created": False,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_SEAMGRIM_WORKBENCH_G2_CONTRACT", f"{key}={contract.get(key)!r}")

    js = (ROOT / "solutions/seamgrim_ui_mvp/ui/workbench/workbench.js").read_text(encoding="utf-8")
    for token in [
        "createServerProject",
        "appendServerRevision",
        "openServerProject",
        "listServerRevisions",
        "project-revision-item",
        "capability_redacted",
    ]:
        if token not in js:
            return fail("E_SEAMGRIM_WORKBENCH_G2_JS_TOKEN", token)

    pack = run(["python", "tests/run_pack_golden.py", "seamgrim_workbench_server_project_save_reopen_v1"], timeout=120)
    if pack.returncode:
        return fail("E_SEAMGRIM_WORKBENCH_G2_PACK", pack.stdout.strip())

    browser = run(["node", str(RUNNER.relative_to(ROOT))], timeout=240)
    if browser.returncode:
        return fail("E_SEAMGRIM_WORKBENCH_G2_BROWSER", browser.stdout.strip())
    if "workbench_server_project_save_reopen.runner_result.v1" not in browser.stdout:
        return fail("E_SEAMGRIM_WORKBENCH_G2_BROWSER_OK", browser.stdout.strip())

    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_WORKBENCH_G2_SSOT", ssot.stdout.strip())

    print("[seamgrim-workbench-server-project-save-reopen-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
