#!/usr/bin/env python
from __future__ import annotations

import hashlib
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROOT_LOW_RISK_RETIRE_DELETE_DRY_RUN_PLAN_V1.md"
PREFLIGHT_DOC = ROOT / "ROOT_LOW_RISK_RETIRE_DELETE_PREFLIGHT_V1.md"
PREFLIGHT_CHECK = ROOT / "tests" / "run_root_low_risk_retire_delete_preflight_check.py"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"

EXPECTED = {
    "prompts/codex_tasks/TASK_IMPL_NOTES_L01_L03.md": (420, "47436cd531c873748547d4288929a4842679cf88fcddab488fd9fd455ea6cfaa"),
    "prompts/codex_tasks/TASK_LEDGER_SEED_FROM_SSOT_v20.2.9.md": (1995, "1163a43118b6990da1e9bca3dd788aea78df591ad7be4fa5c6fa4454a1532cd8"),
    "prompts/codex_tasks/TASK_TEUL_TEST_RESULT_AND_PINNING.md": (783, "20354aaed4a607501056fa74406c2a77def680a2d9cb539ebc88c1afd9dbbbeb"),
    "golden/const_reassign_error.test.json": (324, "aab677b2965d356c1feffebd2fd8e0e5d3dc3d4b7603353c60bc59bef770ad12"),
    "golden/decl_block_ok.test.json": (335, "9d85e3dee83f4ad9d56f94376cd707e96f1860197e248a56394a055f37f33c78"),
    "scripts/cargo-local.ps1": (398, "48fa7d9853c2b43a8a5ba5910e8cfa945dc6991f7ba8f0c63e702efa6b41b69d"),
    "scripts/seamgrim_seed_candidates.py": (3844, "6d8264ac368e455bcb27b8b0cbae46f9a53aacfbd15e0a835a9e07564c2fd487"),
    "scripts/ssot_edu_diff_report.py": (2928, "f303198fadce859f5232f183fdf0a2a7dea1c67c34efd3475a1a8945d22bdbdf"),
    "guides/math/MATH_SUPPORT_CATALOG_V0.md": (1974, "b388a3f9abc91c1f2f883e18c504c8f088e12e3289bcb9e57f24d33700f9da77"),
}


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def require_files() -> int:
    required = [DOC, PREFLIGHT_DOC, PREFLIGHT_CHECK, QUEUE]
    required.extend(ROOT / path for path in EXPECTED)
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROOT_RETIRE_DRY_RUN_MISSING", str(missing))
    return 0


def check_doc() -> int:
    text = read(DOC)
    required = [
        "non-destructive",
        "does not delete, move, rename, path-rewrite, or archive any file",
        "File-Level Candidate Inventory",
        "Approval-Gated Command Shape",
        "Dry-run print shape",
        "Approved execution shape",
        "Remove-Item -LiteralPath",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1` remains blocked",
        "not deletion approval",
    ]
    for path, (_, digest) in EXPECTED.items():
        required.extend([path, digest])
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ROOT_RETIRE_DRY_RUN_DOC", str(missing))
    return 0


def check_inventory() -> int:
    mismatches: list[str] = []
    for rel, (size, digest) in EXPECTED.items():
        path = ROOT / rel
        if path.stat().st_size != size:
            mismatches.append(f"{rel}: size {path.stat().st_size} != {size}")
        actual_digest = sha256(path)
        if actual_digest != digest:
            mismatches.append(f"{rel}: sha256 {actual_digest} != {digest}")
    if mismatches:
        return fail("E_ROOT_RETIRE_DRY_RUN_INVENTORY", str(mismatches))
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "ROOT_LOW_RISK_RETIRE_DELETE_DRY_RUN_PLAN_V1",
        "closed by `ROOT_LOW_RISK_RETIRE_DELETE_DRY_RUN_PLAN_V1.md`",
        "file-level deletion candidate inventory",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "Approval-gated",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ROOT_RETIRE_DRY_RUN_QUEUE", str(missing))
    if "1. `ROOT_LOW_RISK_RETIRE_DELETE_V1`" not in text:
        return fail("E_ROOT_RETIRE_DRY_RUN_NEXT", "root retire delete is not next")
    return 0


def check_preflight() -> int:
    result = subprocess.run(
        ["python", "tests/run_root_low_risk_retire_delete_preflight_check.py"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0:
        return fail("E_ROOT_RETIRE_DRY_RUN_PREFLIGHT", result.stdout.strip())
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
        return fail("E_ROOT_RETIRE_DRY_RUN_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROOT_RETIRE_DRY_RUN_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc,
        check_inventory,
        check_queue,
        check_preflight,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[root-low-risk-retire-delete-dry-run-plan-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
