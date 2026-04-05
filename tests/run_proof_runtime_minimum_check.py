from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def run(cmd: list[str]) -> None:
    completed = subprocess.run(cmd, cwd=ROOT)
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)


def load_json(path: str) -> dict:
    return json.loads((ROOT / path).read_text(encoding="utf-8"))


def main() -> int:
    run(
        [
            sys.executable,
            "tests/run_pack_golden.py",
            "proof_runtime_smoke_v1",
            "proof_guard_tick_v1",
            "proof_guard_rollback_v1",
            "proof_alert_continue_v1",
        ]
    )

    proof_base = load_json("pack/proof_runtime_smoke_v1/expected/proof_base.detjson")
    proof_check = load_json("pack/proof_runtime_smoke_v1/expected/proof_check.detjson")
    assert proof_base["state_hash"] == proof_check["state_hash"]
    assert proof_base["proof_runtime_count"] == 0
    assert proof_check["proof_runtime_count"] == 1
    assert proof_check["proof_runtime"]["proof_check_count"] == 1

    guard_off = load_json("pack/proof_guard_tick_v1/expected/proof_off.detjson")
    guard_on = load_json("pack/proof_guard_tick_v1/expected/proof_on.detjson")
    assert guard_off["state_hash"] != guard_on["state_hash"]
    assert guard_off["proof_runtime_count"] == 0
    assert guard_on["proof_runtime_count"] == 2
    assert guard_on["proof_runtime"]["proof_check_count"] == 2

    rollback = load_json("pack/proof_guard_rollback_v1/expected/proof.detjson")
    alert = load_json("pack/proof_alert_continue_v1/expected/proof.detjson")
    assert rollback["contract_diag_count"] == 1
    assert alert["contract_diag_count"] == 1
    assert rollback["contract_diags"][0]["mode"] == "abort"
    assert alert["contract_diags"][0]["mode"] == "alert"
    assert rollback["state_hash"] != alert["state_hash"]
    assert rollback["proof_runtime"]["proof_check_count"] == 1
    assert alert["proof_runtime"]["proof_check_count"] == 1

    print("proof_runtime_minimum: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
