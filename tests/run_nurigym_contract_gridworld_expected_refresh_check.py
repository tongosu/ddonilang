#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1.md"
REBASE_DOC = ROOT / "ROADMAP_V2_A2_NURIGYM_REPRESENTATIVE_ENV_REBASE_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"

EXPECTED_TEXT = {
    "pack/nuri_gym_canon_contract_v1/expected/dataset_hash.txt": "sha256:34289c2282de6f9b8f4869a20fa48dff68ff4f3b49f2d0340178c321b2aad313",
    "pack/nuri_gym_canon_contract_v1/expected/export_stdout.txt": "dataset_hash=sha256:34289c2282de6f9b8f4869a20fa48dff68ff4f3b49f2d0340178c321b2aad313",
    "pack/nuri_gym_gridworld_v1/expected/stdout.txt": "dataset_hash=sha256:d46e2c6d9e1c6befe2979eb26a47b91339f64a9dd49ea64a748dadff7ef9d23d",
}


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        DOC,
        REBASE_DOC,
        QUEUE,
        ROOT / "tests" / "run_nuri_gym_contract_check.py",
        ROOT / "pack" / "nuri_gym_canon_contract_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_gridworld_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_canon_contract_v1" / "RUN_LOG.txt",
        ROOT / "pack" / "nuri_gym_canon_contract_v1" / "SHA256SUMS.txt",
        ROOT / "pack" / "nuri_gym_gridworld_v1" / "RUN_LOG.txt",
        ROOT / "pack" / "nuri_gym_gridworld_v1" / "SHA256SUMS.txt",
        ROOT / "pack" / "nuri_gym_gridworld_v1" / "expected" / "nurigym.dataset.jsonl",
    ]
    required.extend(ROOT / rel for rel in EXPECTED_TEXT)
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_NURIGYM_CONTRACT_GRIDWORLD_REFRESH_MISSING", str(missing))
    return 0


def check_expected_files() -> int:
    for rel, expected in EXPECTED_TEXT.items():
        got = read(ROOT / rel).strip()
        if got != expected:
            return fail("E_NURIGYM_CONTRACT_GRIDWORLD_REFRESH_VALUE", f"{rel}: {got!r}")
    dataset = read(ROOT / "pack" / "nuri_gym_gridworld_v1" / "expected" / "nurigym.dataset.jsonl")
    for token in [
        "sha256:062a37291f73fd6b2d09e50392f7228c56a4d030c0958a7ed050b3ecfd0e993a",
        "\"schema\":\"nurigym.dataset.v1\"",
        "\"schema\":\"nurigym.step.v1\"",
    ]:
        if token not in dataset:
            return fail("E_NURIGYM_CONTRACT_GRIDWORLD_REFRESH_DATASET", token)
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_docs() -> int:
    rc = require_tokens(
        DOC,
        [
            "NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1",
            "ROADMAP_V2_A2_NURIGYM_REPRESENTATIVE_ENV_REBASE_V1",
            "nuri_gym_canon_contract_v1",
            "nuri_gym_gridworld_v1",
            "sha256:34289c2282de6f9b8f4869a20fa48dff68ff4f3b49f2d0340178c321b2aad313",
            "sha256:062a37291f73fd6b2d09e50392f7228c56a4d030c0958a7ed050b3ecfd0e993a",
            "sha256:d46e2c6d9e1c6befe2979eb26a47b91339f64a9dd49ea64a748dadff7ef9d23d",
            "python tests/run_nuri_gym_contract_check.py` PASS",
            "ROADMAP_V2_A2_NURIGYM_BANDIT_OR_FINAL_RECHECK_V1",
        ],
        "E_NURIGYM_CONTRACT_GRIDWORLD_REFRESH_DOC",
    )
    if rc:
        return rc
    return require_tokens(
        REBASE_DOC,
        [
            "ROADMAP_V2 `아-2`",
            "nuri_gym_canon_contract_v1",
            "nuri_gym_gridworld_v1",
            "NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1",
        ],
        "E_NURIGYM_CONTRACT_GRIDWORLD_REFRESH_REBASE_DOC",
    )


def run_support_checks() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "nuri_gym_canon_contract_v1", "nuri_gym_gridworld_v1"],
        ["python", "tests/run_nuri_gym_contract_check.py"],
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
            return fail("E_NURIGYM_CONTRACT_GRIDWORLD_REFRESH_SUPPORT", result.stdout.strip())
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1",
        "closed by `NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1.md`",
        "tests/run_nuri_gym_contract_check.py",
        "ROADMAP_V2_A2_NURIGYM_BANDIT_OR_FINAL_RECHECK_V1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "Approval-gated",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_NURIGYM_CONTRACT_GRIDWORLD_REFRESH_QUEUE", str(missing))
    if "1. `NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1`" in text:
        return fail(
            "E_NURIGYM_CONTRACT_GRIDWORLD_REFRESH_STILL_OPEN",
            "refresh item is still next",
        )
    if "1. `ROOT_LOW_RISK_RETIRE_DELETE_V1`" not in text:
        return fail(
            "E_NURIGYM_CONTRACT_GRIDWORLD_REFRESH_NEXT",
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
        return fail("E_NURIGYM_CONTRACT_GRIDWORLD_REFRESH_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_NURIGYM_CONTRACT_GRIDWORLD_REFRESH_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_expected_files,
        check_docs,
        run_support_checks,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[nurigym-contract-gridworld-expected-refresh-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
