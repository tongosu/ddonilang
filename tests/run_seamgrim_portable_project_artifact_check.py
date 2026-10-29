#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_portable_project_artifact_v1"
RUNNER = ROOT / "tests" / "seamgrim_portable_project_artifact_runner.mjs"
HELPER = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "persistence" / "portable_project_artifact.js"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def run(cmd: list[str], *, timeout: int = 120) -> subprocess.CompletedProcess[str]:
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


def require_files() -> int:
    required = [
        HELPER,
        RUNNER,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "project_snapshot.js",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_PORTABLE_ARTIFACT_MISSING", repr(missing))
    return 0


def check_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_portable_project_artifact_v1",
        "gate": "G1",
        "portable_schema": "seamgrim.project_snapshot_artifact.v0",
        "new_stable_schema": False,
        "max_bytes": 2_000_000,
        "capability_export_allowed": False,
        "authorization_export_allowed": False,
        "auto_run_on_import": False,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_PORTABLE_ARTIFACT_CONTRACT", f"{key}={payload.get(key)!r}")
    negatives = payload.get("negative_controls")
    if not isinstance(negatives, list) or len(negatives) < 8:
        return fail("E_PORTABLE_ARTIFACT_NEGATIVES", repr(negatives))
    return 0


def run_node_runner() -> int:
    proc = run(["node", str(RUNNER.relative_to(ROOT))])
    if proc.returncode != 0:
        return fail("E_PORTABLE_ARTIFACT_RUNNER", proc.stdout.strip())
    if "seamgrim_portable_project_artifact: ok" not in proc.stdout:
        return fail("E_PORTABLE_ARTIFACT_RUNNER_OK", proc.stdout.strip())
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=30)
    if proc.returncode != 0:
        return fail("E_PORTABLE_ARTIFACT_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_PORTABLE_ARTIFACT_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (require_files, check_contract, run_node_runner, check_docs_ssot_clean):
        rc = check()
        if rc:
            return rc
    print("[seamgrim-portable-project-artifact-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

