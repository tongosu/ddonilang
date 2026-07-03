#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROOT_LOW_RISK_RETIRE_DELETE_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"

DELETED = [
    "prompts/codex_tasks/TASK_IMPL_NOTES_L01_L03.md",
    "prompts/codex_tasks/TASK_LEDGER_SEED_FROM_SSOT_v20.2.9.md",
    "prompts/codex_tasks/TASK_TEUL_TEST_RESULT_AND_PINNING.md",
    "golden/const_reassign_error.test.json",
    "golden/decl_block_ok.test.json",
    "scripts/cargo-local.ps1",
    "scripts/seamgrim_seed_candidates.py",
    "scripts/ssot_edu_diff_report.py",
    "guides/math/MATH_SUPPORT_CATALOG_V0.md",
]

PRESERVED = [
    "docs/context/codex_tasks/legacy_prompts/TASK_IMPL_NOTES_L01_L03.md",
    "docs/context/codex_tasks/legacy_prompts/TASK_LEDGER_SEED_FROM_SSOT_v20.2.9.md",
    "docs/context/codex_tasks/legacy_prompts/TASK_TEUL_TEST_RESULT_AND_PINNING.md",
    "pack/decl_block_runtime/golden/const_reassign_error.test.json",
    "pack/decl_block_runtime/golden/decl_block_ok.test.json",
    "docs/guides/math/MATH_SUPPORT_CATALOG_V0.md",
]


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        DOC,
        QUEUE,
        ROOT / "ROOT_LOW_RISK_RETIRE_DELETE_DRY_RUN_PLAN_V1.md",
        ROOT / "ROOT_LOW_RISK_RETIRE_DELETE_PREFLIGHT_V1.md",
    ]
    required.extend(ROOT / path for path in PRESERVED)
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROOT_RETIRE_DELETE_MISSING", str(missing))
    return 0


def check_deleted_absent() -> int:
    present = [path for path in DELETED if (ROOT / path).exists()]
    if present:
        return fail("E_ROOT_RETIRE_DELETE_STILL_PRESENT", str(present))
    if (ROOT / "prompts" / "codex_tasks").exists():
        return fail("E_ROOT_RETIRE_DELETE_DIR_STILL_PRESENT", "prompts/codex_tasks")
    return 0


def check_doc() -> int:
    text = read(DOC)
    required = [
        "Deleted Files",
        "Preserved Counterparts",
        "No automatic next development item is selected",
        "docs/ssot/**",
    ]
    required.extend(DELETED)
    required.extend(PRESERVED)
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ROOT_RETIRE_DELETE_DOC", str(missing))
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "closed by `ROOT_LOW_RISK_RETIRE_DELETE_V1.md`",
        "No automatic next development item is selected",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ROOT_RETIRE_DELETE_QUEUE", str(missing))
    if "1. `ROOT_LOW_RISK_RETIRE_DELETE_V1`" in text:
        return fail("E_ROOT_RETIRE_DELETE_QUEUE_STILL_NEXT", "root delete is still listed as next")
    return 0


def run_no_match_rg(pattern: str, code: str) -> int:
    paths = [
        "AGENTS.md",
        "CLAUDE.md",
        "README.md",
        "scripts",
        "tests",
        "tools",
        "tool",
        "solutions",
        "pack",
        ".github",
        ".gitlab-ci.yml",
        "azure-pipelines.yml",
    ]
    command = [
        "rg",
        "-n",
        "-g",
        "!tests/run_root_low_risk_retire_delete*.py",
        pattern,
        *paths,
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
    if result.returncode == 1 and not result.stdout.strip():
        return 0
    if result.returncode == 0:
        return fail(code, result.stdout.strip())
    return fail(code, result.stdout.strip() or f"rg exited {result.returncode}")


def check_active_references() -> int:
    checks = [
        (
            r"prompts/codex_tasks|docs/context/codex_tasks/legacy_prompts",
            "E_ROOT_RETIRE_DELETE_PROMPTS_REF",
        ),
        (
            r"const_reassign_error\.test\.json|decl_block_ok\.test\.json",
            "E_ROOT_RETIRE_DELETE_GOLDEN_REF",
        ),
        (
            r"cargo-local\.ps1|seamgrim_seed_candidates\.py|ssot_edu_diff_report\.py",
            "E_ROOT_RETIRE_DELETE_SCRIPT_REF",
        ),
        (
            r"guides/math/MATH_SUPPORT_CATALOG_V0\.md|docs/guides/math/MATH_SUPPORT_CATALOG_V0\.md|MATH_SUPPORT_CATALOG_V0\.md",
            "E_ROOT_RETIRE_DELETE_GUIDE_REF",
        ),
    ]
    for pattern, code in checks:
        rc = run_no_match_rg(pattern, code)
        if rc:
            return rc
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
        return fail("E_ROOT_RETIRE_DELETE_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROOT_RETIRE_DELETE_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_deleted_absent,
        check_doc,
        check_queue,
        check_active_references,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[root-low-risk-retire-delete-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
