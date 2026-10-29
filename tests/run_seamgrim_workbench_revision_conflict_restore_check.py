#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_workbench_revision_conflict_restore_v1"
RUNNER = ROOT / "tests" / "seamgrim_workbench_revision_conflict_restore_runner.mjs"


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
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_WORKBENCH_G3_MISSING", json.dumps(missing, ensure_ascii=False))

    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "pack": "seamgrim_workbench_revision_conflict_restore_v1",
        "gate": "G3",
        "product_behavior_change": True,
        "behavior_closed_claim": True,
        "append_only_revision_store": True,
        "restore_creates_new_revision": True,
        "server_head_overwrite_on_conflict": False,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_SEAMGRIM_WORKBENCH_G3_CONTRACT", f"{key}={contract.get(key)!r}")

    html = (ROOT / "solutions/seamgrim_ui_mvp/ui/workbench/index.html").read_text(encoding="utf-8")
    js = (ROOT / "solutions/seamgrim_ui_mvp/ui/workbench/workbench.js").read_text(encoding="utf-8")
    for token in [
        "project-conflict-panel",
        "project-conflict-local-package",
        "project-conflict-copy-local",
    ]:
        if token not in html:
            return fail("E_SEAMGRIM_WORKBENCH_G3_HTML_TOKEN", token)
    for token in [
        "restoreServerRevision",
        "recordServerConflict",
        "restoreProjectRevision",
        "E_PROJECT_REVISION_CONFLICT",
    ]:
        if token not in js:
            return fail("E_SEAMGRIM_WORKBENCH_G3_JS_TOKEN", token)

    pack = run(["python", "tests/run_pack_golden.py", "seamgrim_workbench_revision_conflict_restore_v1"], timeout=120)
    if pack.returncode:
        return fail("E_SEAMGRIM_WORKBENCH_G3_PACK", pack.stdout.strip())

    browser = run(["node", str(RUNNER.relative_to(ROOT))], timeout=260)
    if browser.returncode:
        return fail("E_SEAMGRIM_WORKBENCH_G3_BROWSER", browser.stdout.strip())
    if "workbench_revision_conflict_restore.runner_result.v1" not in browser.stdout:
        return fail("E_SEAMGRIM_WORKBENCH_G3_BROWSER_OK", browser.stdout.strip())

    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_WORKBENCH_G3_SSOT", ssot.stdout.strip())

    print("[seamgrim-workbench-revision-conflict-restore-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
