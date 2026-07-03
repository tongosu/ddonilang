#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "NURIGYM_DATASET_HASH_EXPECTED_REFRESH_V1.md"
REBASE_DOC = ROOT / "ROADMAP_V2_A1_NURIGYM_REBASE_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"

EXPECTED_HASHES = {
    "pack/nuri_gym_gridmaze_v1/golden.txt": "dataset_hash=sha256:fd8ec4cfce54870da530e9d1619ec51ecac2c3f066e54740f92b149c0e93ddc9",
    "pack/nuri_gym_gridmaze_v1/golden_alt.txt": "dataset_hash=sha256:c72982e6da345723c315010a2ddafab47145e8bca0beb48fdc66a07e7de8382d",
    "pack/nuri_gym_gridmaze_v1/golden_alt2.txt": "dataset_hash=sha256:0cdf2d9ecf68cb221f01e15c5d8233fa2e5aa4d4c29486d672f93dd00670ce2b",
    "pack/nuri_gym_cartpole_shared_sync_v1/golden.txt": "dataset_hash=sha256:c1ff18f7e3ab574e491ee43127e91d65d53f6f22eae32e74ca4f5c92edc9787e",
    "pack/nuri_gym_cartpole_shared_sync_v1/golden_alt.txt": "dataset_hash=sha256:0044e87d98f8637603edd9be4dafb990496b6da88fccad28b047e838e3ca5d85",
    "pack/nuri_gym_cartpole_shared_sync_v1/golden_alt2.txt": "dataset_hash=sha256:e1aa187abc85dd503f14ab5aed0549a7d2dbe81ad65f77a359928f032808b4cd",
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
        ROOT / "pack" / "gogae5_w47_nurigym_observation_spec" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_gridmaze_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_cartpole_shared_sync_v1" / "golden.jsonl",
    ]
    required.extend(ROOT / rel for rel in EXPECTED_HASHES)
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_NURIGYM_HASH_REFRESH_MISSING", str(missing))
    return 0


def check_hash_files() -> int:
    for rel, expected in EXPECTED_HASHES.items():
        got = read(ROOT / rel).strip()
        if got != expected:
            return fail("E_NURIGYM_HASH_REFRESH_VALUE", f"{rel}: {got!r}")
    return 0


def check_docs() -> int:
    doc = read(DOC)
    rebase = read(REBASE_DOC)
    queue = read(QUEUE)
    required_doc_tokens = [
        "NURIGYM_DATASET_HASH_EXPECTED_REFRESH_V1",
        "ROADMAP_V2_A1_NURIGYM_REBASE_V1",
        "nuri_gym_gridmaze_v1",
        "nuri_gym_cartpole_shared_sync_v1",
        "expected evidence only",
        "ROADMAP_V2_A1_NURIGYM_FINAL_CLOSURE_V1",
    ]
    for token in required_doc_tokens:
        if token not in doc:
            return fail("E_NURIGYM_HASH_REFRESH_DOC_TOKEN", token)
    for token in [
        "ROADMAP_V2 `아-1`",
        "reset/step 첫실행",
        "dataset hash expected files were stale",
        "NURIGYM_DATASET_HASH_EXPECTED_REFRESH_V1",
        "ROADMAP_V2_A1_NURIGYM_FINAL_CLOSURE_V1",
    ]:
        if token not in rebase:
            return fail("E_NURIGYM_HASH_REFRESH_REBASE_TOKEN", token)
    for token in [
        "ROADMAP_V2_A1_NURIGYM_REBASE_V1",
        "NURIGYM_DATASET_HASH_EXPECTED_REFRESH_V1",
        "ROADMAP_V2_A1_NURIGYM_FINAL_CLOSURE_V1",
    ]:
        if token not in queue:
            return fail("E_NURIGYM_HASH_REFRESH_QUEUE_TOKEN", token)
    refresh_marker = (
        "`NURIGYM_DATASET_HASH_EXPECTED_REFRESH_V1` is closed by "
        "`NURIGYM_DATASET_HASH_EXPECTED_REFRESH_V1.md`."
    )
    final_marker = (
        "`ROADMAP_V2_A1_NURIGYM_FINAL_CLOSURE_V1` is closed by "
        "`ROADMAP_V2_A1_NURIGYM_FINAL_CLOSURE_V1.md`."
    )
    if refresh_marker not in queue:
        return fail("E_NURIGYM_HASH_REFRESH_QUEUE_CLOSED", "refresh closure marker missing")
    if final_marker not in queue:
        return fail("E_NURIGYM_HASH_REFRESH_QUEUE_FINAL", "A1 final closure marker missing")
    if "1. `ROADMAP_V2_A1_NURIGYM_REBASE_V1`" in queue:
        return fail("E_NURIGYM_HASH_REFRESH_REBASE_OPEN", "rebase is still next open item")
    return 0


def run_pack_golden() -> int:
    command = [
        "python",
        "tests/run_pack_golden.py",
        "nuri_gym_gridmaze_v1",
        "nuri_gym_cartpole_shared_sync_v1",
        "gogae5_w47_nurigym_observation_spec",
    ]
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
        return fail("E_NURIGYM_HASH_REFRESH_PACK_GOLDEN", result.stdout.strip())
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
        return fail("E_NURIGYM_HASH_REFRESH_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_NURIGYM_HASH_REFRESH_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_hash_files,
        check_docs,
        run_pack_golden,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[nurigym-dataset-hash-expected-refresh-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
