#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_workbench_durable_persistence_adapters_v1"
JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js"
RUNNER = ROOT / "tests" / "seamgrim_workbench_durable_persistence_adapters_runner.mjs"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"E_SEAMGRIM_WORKBENCH_DURABLE_PERSISTENCE_ADAPTERS: {message}")


def run(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=ROOT, text=True, encoding="utf-8", errors="replace", capture_output=True)


def main() -> int:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    require(contract.get("schema") == "seamgrim.workbench_durable_persistence_adapters.contract.v1", "contract schema mismatch")
    for rel in contract.get("required_modules", []):
        require((ROOT / rel).is_file(), f"missing required module: {rel}")
    require("server project save/open" in contract.get("non_claims", []), "G2 non-claim missing")
    require("run archive server API" in contract.get("non_claims", []), "G4 non-claim missing")

    for rel in contract.get("required_modules", []):
        result = run(["node", "--check", rel])
        require(result.returncode == 0, f"node syntax failed for {rel}: {result.stderr or result.stdout}")

    js = JS.read_text(encoding="utf-8")
    runner = RUNNER.read_text(encoding="utf-8")
    for token in ["workbench_api_ready", "completed_generation", "disabled_reason", "beginPersistenceOperation"]:
        require(token in js, f"workbench readiness token missing: {token}")
    for token in ["waitForWorkbenchReady", "completed_generation", "owned_residual"]:
        require(token in runner, f"runner readiness token missing: {token}")

    result = run(["node", str(RUNNER.relative_to(ROOT))])
    require(result.returncode == 0, result.stderr or result.stdout)
    require("seamgrim_workbench_durable_persistence_adapters: ok" in result.stdout, "browser runner ok marker missing")
    payload = next((json.loads(line) for line in result.stdout.splitlines() if line.startswith("{")), {})
    require(payload.get("schema") == "seamgrim.workbench_durable_persistence_adapters.runner_result.v2", "runner result schema mismatch")
    require(payload.get("readiness", {}).get("workbench_api_ready") is True, "workbench API readiness missing")
    persistence = payload.get("persistence", {})
    require(persistence.get("phase") == "saved" and persistence.get("mode") == "local_fallback", "save generation did not complete")
    require(persistence.get("completed_generation") == persistence.get("generation"), "save generation mismatch")
    require(payload.get("owned_residual") == {"processes": 0, "sockets": 0, "temp_dirs": 0}, "owned residual not zero")

    print("[seamgrim-workbench-durable-persistence-adapters-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
