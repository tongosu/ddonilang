#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROADMAP_V2_A2_NURIGYM_FINAL_CLOSURE_RECHECK_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "아-2_REPORT_20260604.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )


def require_files() -> int:
    required = [
        DOC,
        REPORT,
        QUEUE,
        ROOT / "ROADMAP_V2_A2_NURIGYM_REPRESENTATIVE_ENV_REBASE_V1.md",
        ROOT / "NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1.md",
        ROOT / "NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1.md",
        ROOT / "NURIGYM_BANDIT_MINIMUM_PACK_V1.md",
        ROOT / "tests" / "run_nuri_gym_contract_check.py",
        ROOT / "tests" / "run_nurigym_cartpole_pendulum_expected_refresh_check.py",
        ROOT / "tests" / "run_nurigym_bandit_minimum_pack_check.py",
        ROOT / "pack" / "gogae5_w47_nurigym_observation_spec" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_cartpole_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_cartpole_shared_sync_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_gridmaze_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_gridworld_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_pendulum_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_canon_contract_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_bandit_v1" / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROADMAP_V2_A2_FINAL_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_doc() -> int:
    return require_tokens(
        DOC,
        [
            "ROADMAP_V2 `아-2`",
            "closed",
            "representative NuriGym environment golden evidence",
            "CartPole",
            "GridMaze",
            "Bandit",
            "nuri_gym_cartpole_v1",
            "nuri_gym_cartpole_shared_sync_v1",
            "nuri_gym_gridmaze_v1",
            "nuri_gym_gridworld_v1",
            "nuri_gym_pendulum_v1",
            "nuri_gym_bandit_v1",
            "nuri_gym_canon_contract_v1",
            "Python/Web parity",
            "dataset registry",
            "training workflow",
            "ROOT_LOW_RISK_RETIRE_DELETE_V1",
            "approval-gated",
        ],
        "E_ROADMAP_V2_A2_FINAL_DOC",
    )


def check_report() -> int:
    return require_tokens(
        REPORT,
        [
            "ROADMAP_V2 `아-2`",
            "closed",
            "nuri_gym_cartpole_v1",
            "nuri_gym_gridmaze_v1",
            "nuri_gym_bandit_v1",
            "tests/run_nuri_gym_contract_check.py",
            "tests/run_nurigym_bandit_minimum_pack_check.py",
            "docs/ssot",
        ],
        "E_ROADMAP_V2_A2_FINAL_REPORT",
    )


def run_support_checks() -> int:
    commands = [
        [
            "python",
            "tests/run_pack_golden.py",
            "nuri_gym_cartpole_v1",
            "nuri_gym_gridmaze_v1",
            "nuri_gym_pendulum_v1",
            "nuri_gym_gridworld_v1",
            "nuri_gym_canon_contract_v1",
            "nuri_gym_bandit_v1",
        ],
        [
            "python",
            "tests/run_pack_golden.py",
            "nuri_gym_gridmaze_v1",
            "nuri_gym_cartpole_shared_sync_v1",
            "gogae5_w47_nurigym_observation_spec",
        ],
        ["python", "tests/run_nuri_gym_contract_check.py"],
        ["python", "tests/run_nurigym_cartpole_pendulum_expected_refresh_check.py"],
        ["python", "tests/run_nurigym_bandit_minimum_pack_check.py"],
    ]
    for command in commands:
        result = run(command)
        if result.returncode != 0:
            return fail("E_ROADMAP_V2_A2_FINAL_SUPPORT", result.stdout.strip())
    return 0


def check_product_dispatch() -> int:
    text = read(ROOT / "tools" / "teul-cli" / "src" / "cli" / "nurigym.rs")
    for token in [
        "nurigym.bandit1d",
        "nurigym.cartpole1d",
        "nurigym.pendulum1d",
        "nurigym.gridmaze2d",
    ]:
        if token not in text:
            return fail("E_ROADMAP_V2_A2_FINAL_ENV_SUPPORT", token)
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "ROADMAP_V2_A2_NURIGYM_FINAL_CLOSURE_RECHECK_V1",
        "closed by `ROADMAP_V2_A2_NURIGYM_FINAL_CLOSURE_RECHECK_V1.md`",
        "ROADMAP_V2 `아-2` is closed",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "approval-gated",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_A2_FINAL_QUEUE", str(missing))
    if "1. `ROADMAP_V2_A2_NURIGYM_FINAL_CLOSURE_RECHECK_V1`" in text:
        return fail(
            "E_ROADMAP_V2_A2_FINAL_STILL_OPEN",
            "final closure is still listed as next",
        )
    return 0


def check_docs_ssot_clean() -> int:
    result = run(["git", "status", "--short", "--", "docs/ssot"])
    if result.returncode != 0:
        return fail("E_ROADMAP_V2_A2_FINAL_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROADMAP_V2_A2_FINAL_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc,
        check_report,
        check_product_dispatch,
        run_support_checks,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[roadmap-v2-a2-final-closure-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
