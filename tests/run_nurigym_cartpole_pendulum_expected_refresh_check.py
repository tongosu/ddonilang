#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"

EXPECTED_TEXT = {
    "pack/nuri_gym_cartpole_v1/golden.txt": "dataset_hash=sha256:5c58bea8c2a4fdf96ba5d332976634f6b5a6f4260aed798513696825226d81cb",
    "pack/nuri_gym_cartpole_v1/golden_alt.txt": "dataset_hash=sha256:3cb723c71af2933fc015e47665b990a5d395be0af1c5aef8803aff8ae1d6d966",
    "pack/nuri_gym_pendulum_v1/golden.txt": "dataset_hash=sha256:81e20f9a44aea091ab2ee783f1fb0f932de650f6fefb99148e9a4d98f6e2912b",
    "pack/nuri_gym_pendulum_v1/golden_alt.txt": "dataset_hash=sha256:918ec12927f9484619a8620cc988841d625542f7c5cb348dd3989b0df8e5b038",
    "pack/nuri_gym_pendulum_v1/golden_alt2.txt": "dataset_hash=sha256:4635383c34620239afdb379529470bae22f27c503731f70d60a51224f1204166",
    "pack/nuri_gym_pendulum_v1/golden_alt3.txt": "dataset_hash=sha256:2e6cc77fb0443c6f04658d79e64c6a1e4dee90f6bcb975771ce9bd6acd9345e5",
}


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        DOC,
        QUEUE,
        ROOT / "ROADMAP_V2_A2_NURIGYM_BANDIT_OR_FINAL_RECHECK_V1.md",
        ROOT / "tests" / "run_roadmap_v2_a2_bandit_or_final_recheck.py",
        ROOT / "pack" / "nuri_gym_cartpole_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_pendulum_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_gridmaze_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_gridworld_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_canon_contract_v1" / "golden.jsonl",
    ]
    required.extend(ROOT / rel for rel in EXPECTED_TEXT)
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_NURIGYM_CARTPOLE_PENDULUM_REFRESH_MISSING", str(missing))
    return 0


def check_expected_files() -> int:
    for rel, expected in EXPECTED_TEXT.items():
        got = read(ROOT / rel).strip()
        if got != expected:
            return fail("E_NURIGYM_CARTPOLE_PENDULUM_REFRESH_VALUE", f"{rel}: {got!r}")
    for rel in [
        "pack/nuri_gym_cartpole_v1/RUN_LOG.txt",
        "pack/nuri_gym_cartpole_v1/SHA256SUMS.txt",
        "pack/nuri_gym_pendulum_v1/RUN_LOG.txt",
        "pack/nuri_gym_pendulum_v1/SHA256SUMS.txt",
    ]:
        if (ROOT / rel).exists():
            return fail("E_NURIGYM_CARTPOLE_PENDULUM_REFRESH_SIDE_FILE", rel)
    for rel in [
        "pack/nuri_gym_cartpole_v1/golden.jsonl",
        "pack/nuri_gym_pendulum_v1/golden.jsonl",
    ]:
        if "exit_code" in read(ROOT / rel):
            return fail("E_NURIGYM_CARTPOLE_PENDULUM_REFRESH_GOLDEN_SHAPE", rel)
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
            "NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1",
            "ROADMAP_V2_A2_NURIGYM_BANDIT_OR_FINAL_RECHECK_V1",
            "nuri_gym_cartpole_v1",
            "nuri_gym_pendulum_v1",
            "sha256:5c58bea8c2a4fdf96ba5d332976634f6b5a6f4260aed798513696825226d81cb",
            "sha256:2e6cc77fb0443c6f04658d79e64c6a1e4dee90f6bcb975771ce9bd6acd9345e5",
            "golden.jsonl` shape is preserved",
            "NURIGYM_BANDIT_MINIMUM_PACK_V1",
            "does not claim",
            "Python/Web parity",
            "dataset registry",
            "training workflow",
        ],
        "E_NURIGYM_CARTPOLE_PENDULUM_REFRESH_DOC",
    )


def run_support_checks() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "nuri_gym_cartpole_v1", "nuri_gym_pendulum_v1"],
        [
            "python",
            "tests/run_pack_golden.py",
            "nuri_gym_cartpole_v1",
            "nuri_gym_gridmaze_v1",
            "nuri_gym_pendulum_v1",
            "nuri_gym_gridworld_v1",
            "nuri_gym_canon_contract_v1",
        ],
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
            return fail("E_NURIGYM_CARTPOLE_PENDULUM_REFRESH_SUPPORT", result.stdout.strip())
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1",
        "closed by `NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1.md`",
        "nuri_gym_cartpole_v1",
        "nuri_gym_pendulum_v1",
        "NURIGYM_BANDIT_MINIMUM_PACK_V1",
        "nuri_gym_bandit_v1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "approval-gated",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_NURIGYM_CARTPOLE_PENDULUM_REFRESH_QUEUE", str(missing))
    if "1. `NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1`" in text:
        return fail(
            "E_NURIGYM_CARTPOLE_PENDULUM_REFRESH_STILL_OPEN",
            "refresh item is still next",
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
        return fail("E_NURIGYM_CARTPOLE_PENDULUM_REFRESH_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_NURIGYM_CARTPOLE_PENDULUM_REFRESH_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_expected_files,
        check_doc,
        run_support_checks,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[nurigym-cartpole-pendulum-expected-refresh-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
