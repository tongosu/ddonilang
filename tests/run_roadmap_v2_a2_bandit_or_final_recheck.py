#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROADMAP_V2_A2_NURIGYM_BANDIT_OR_FINAL_RECHECK_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "아-2_BANDIT_OR_FINAL_RECHECK_20260604.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        DOC,
        REPORT,
        QUEUE,
        ROOT / "NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1.md",
        ROOT / "NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1.md",
        ROOT / "NURIGYM_BANDIT_MINIMUM_PACK_V1.md",
        ROOT / "tests" / "run_nurigym_contract_gridworld_expected_refresh_check.py",
        ROOT / "tests" / "run_nurigym_cartpole_pendulum_expected_refresh_check.py",
        ROOT / "tests" / "run_nurigym_bandit_minimum_pack_check.py",
        ROOT / "tests" / "run_nuri_gym_contract_check.py",
        ROOT / "pack" / "nuri_gym_cartpole_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_gridmaze_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_gridworld_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_pendulum_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_canon_contract_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_bandit_v1" / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROADMAP_V2_A2_BANDIT_RECHECK_MISSING", str(missing))
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
            "CartPole",
            "GridMaze",
            "Bandit",
            "nuri_gym_cartpole_v1",
            "nuri_gym_gridmaze_v1",
            "nuri_gym_gridworld_v1",
            "nuri_gym_pendulum_v1",
            "nuri_gym_canon_contract_v1",
            "nuri_gym_cartpole_v1` and `nuri_gym_pendulum_v1` expected dataset hashes are stale",
            "sha256:5c58bea8c2a4fdf96ba5d332976634f6b5a6f4260aed798513696825226d81cb",
            "sha256:2e6cc77fb0443c6f04658d79e64c6a1e4dee90f6bcb975771ce9bd6acd9345e5",
            "No `nuri_gym_bandit_*` pack exists",
            "nurigym.cartpole1d",
            "nurigym.pendulum1d",
            "nurigym.gridmaze2d",
            "does not support `nurigym.bandit1d`",
            "ROADMAP_V2 `아-2` remains open",
            "NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1",
            "NURIGYM_BANDIT_MINIMUM_PACK_V1",
            "ROADMAP_V2_A2_NURIGYM_FINAL_CLOSURE_RECHECK_V1",
            "does not claim",
            "Python/Web parity",
            "dataset registry",
            "training workflow",
        ],
        "E_ROADMAP_V2_A2_BANDIT_RECHECK_DOC",
    )


def check_report() -> int:
    return require_tokens(
        REPORT,
        [
            "ROADMAP_V2 `아-2`",
            "nuri_gym_cartpole_v1",
            "nuri_gym_pendulum_v1",
            "expected dataset hashes are stale",
            "No `nuri_gym_bandit_*` pack exists",
            "does not yet support a Bandit env id",
            "NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1",
            "NURIGYM_BANDIT_MINIMUM_PACK_V1",
            "ROADMAP_V2_A2_NURIGYM_FINAL_CLOSURE_RECHECK_V1",
            "keep open",
            "docs/ssot",
        ],
        "E_ROADMAP_V2_A2_BANDIT_RECHECK_REPORT",
    )


def check_bandit_pack_exists() -> int:
    bandit = sorted((ROOT / "pack").glob("*bandit*"))
    if not bandit:
        return fail(
            "E_ROADMAP_V2_A2_BANDIT_RECHECK_PACK_MISSING",
            "nuri_gym_bandit_v1 is missing",
        )
    return 0


def check_product_env_dispatch() -> int:
    text = read(ROOT / "tools" / "teul-cli" / "src" / "cli" / "nurigym.rs")
    for token in [
        "nurigym.bandit1d",
        "nurigym.cartpole1d",
        "nurigym.pendulum1d",
        "nurigym.gridmaze2d",
    ]:
        if token not in text:
            return fail("E_ROADMAP_V2_A2_BANDIT_RECHECK_ENV_SUPPORT", token)
    return 0


def run_representative_support_checks() -> int:
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
        ["python", "tests/run_nuri_gym_contract_check.py"],
        ["python", "tests/run_nurigym_cartpole_pendulum_expected_refresh_check.py"],
        ["python", "tests/run_nurigym_bandit_minimum_pack_check.py"],
    ]
    for command in commands:
        result = subprocess.run(
            command,
            cwd=ROOT,
            text=True,
            encoding="utf-8",
            errors="replace",
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        if result.returncode != 0:
            return fail("E_ROADMAP_V2_A2_BANDIT_RECHECK_SUPPORT", result.stdout.strip())
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "ROADMAP_V2_A2_NURIGYM_BANDIT_OR_FINAL_RECHECK_V1",
        "closed by `ROADMAP_V2_A2_NURIGYM_BANDIT_OR_FINAL_RECHECK_V1.md`",
        "NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1",
        "closed by `NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1.md`",
        "nuri_gym_cartpole_v1",
        "nuri_gym_pendulum_v1",
        "NURIGYM_BANDIT_MINIMUM_PACK_V1",
        "closed by `NURIGYM_BANDIT_MINIMUM_PACK_V1.md`",
        "nuri_gym_bandit_v1",
        "ROADMAP_V2_A2_NURIGYM_FINAL_CLOSURE_RECHECK_V1",
        "ROADMAP_V2 `아-2` remains open",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "Approval-gated",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_A2_BANDIT_RECHECK_QUEUE", str(missing))
    if "1. `ROADMAP_V2_A2_NURIGYM_BANDIT_OR_FINAL_RECHECK_V1`" in text:
        return fail(
            "E_ROADMAP_V2_A2_BANDIT_RECHECK_STILL_OPEN",
            "bandit/final recheck is still next",
        )
    if "1. `NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1`" in text:
        return fail(
            "E_ROADMAP_V2_A2_CARTPOLE_PENDULUM_STILL_OPEN",
            "cartpole/pendulum expected refresh is still next",
        )
    if "1. `ROOT_LOW_RISK_RETIRE_DELETE_V1`" not in text:
        return fail(
            "E_ROADMAP_V2_A2_BANDIT_RECHECK_NEXT",
            "ROOT_LOW_RISK_RETIRE_DELETE_V1 is not next",
        )
    return 0


def check_docs_ssot_clean() -> int:
    result = subprocess.run(
        ["git", "status", "--short", "--", "docs/ssot"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0:
        return fail("E_ROADMAP_V2_A2_BANDIT_RECHECK_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROADMAP_V2_A2_BANDIT_RECHECK_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc,
        check_report,
        check_bandit_pack_exists,
        check_product_env_dispatch,
        run_representative_support_checks,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[roadmap-v2-a2-bandit-or-final-recheck-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
