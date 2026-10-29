#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_workbench_run_archive_lineage_v1"
RUNNER = ROOT / "tests" / "seamgrim_workbench_run_archive_lineage_runner.mjs"


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
        ROOT / "solutions/seamgrim_ui_mvp/ui/workbench/persistence/run_archive_adapter.js",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_WORKBENCH_G5_MISSING", json.dumps(missing, ensure_ascii=False))

    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    for key, expected in {
        "pack": "seamgrim_workbench_run_archive_lineage_v1",
        "gate": "G5",
        "product_behavior_change": True,
        "behavior_closed_claim": True,
        "actual_wasm_run_required": True,
        "server_run_archive_required": True,
        "source_revision_mutated_by_run": False,
        "renderer_pixels_stored": False,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
    }.items():
        if contract.get(key) != expected:
            return fail("E_SEAMGRIM_WORKBENCH_G5_CONTRACT", f"{key}={contract.get(key)!r}")

    js = (ROOT / "solutions/seamgrim_ui_mvp/ui/workbench/workbench.js").read_text(encoding="utf-8")
    adapter = (ROOT / "solutions/seamgrim_ui_mvp/ui/workbench/persistence/run_archive_adapter.js").read_text(encoding="utf-8")
    for token in [
        "createServerRunArchive",
        "persistRunCheckpoint",
        "finalizeActiveRun",
        "parent_server_run_id",
        "run-archive-status",
    ]:
        if token not in js:
            return fail("E_SEAMGRIM_WORKBENCH_G5_JS_TOKEN", token)
    for token in ["createRunArchive", "appendRunCheckpoint", "finalizeRunArchive", "getRunArchive", "renderer_pixels_stored"]:
        if token not in adapter:
            return fail("E_SEAMGRIM_WORKBENCH_G5_ADAPTER_TOKEN", token)

    for cmd in [
        ["python", "tests/run_pack_golden.py", "seamgrim_workbench_run_archive_lineage_v1"],
        ["node", str(RUNNER.relative_to(ROOT))],
    ]:
        proc = run(cmd, timeout=300)
        if proc.returncode:
            return fail("E_SEAMGRIM_WORKBENCH_G5_CMD", f"{cmd!r}: {proc.stdout.strip()}")
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_WORKBENCH_G5_SSOT", ssot.stdout.strip())
    print("[seamgrim-workbench-run-archive-lineage-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
