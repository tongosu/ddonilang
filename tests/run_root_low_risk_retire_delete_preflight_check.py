#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROOT_LOW_RISK_RETIRE_DELETE_PREFLIGHT_V1.md"
APPROVAL = ROOT / "ROOT_LOW_RISK_RETIRE_APPROVAL_v1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        DOC,
        APPROVAL,
        QUEUE,
        ROOT / "CLEANUP_MANIFEST_v1.md",
        ROOT / "ROOT_LEGACY_BACKUP_MANIFEST_v1.md",
        ROOT / "PROMPTS_LEGACY_RETIRE_MANIFEST_v1.md",
        ROOT / "ROOT_GOLDEN_RETIRE_MANIFEST_v1.md",
        ROOT / "ROOT_LEGACY_BATCH2_MANIFEST_v1.md",
        ROOT / "golden" / "const_reassign_error.test.json",
        ROOT / "golden" / "decl_block_ok.test.json",
        ROOT / "scripts" / "cargo-local.ps1",
        ROOT / "scripts" / "seamgrim_seed_candidates.py",
        ROOT / "scripts" / "ssot_edu_diff_report.py",
        ROOT / "guides" / "math" / "MATH_SUPPORT_CATALOG_V0.md",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROOT_RETIRE_PREFLIGHT_MISSING", str(missing))
    if not (ROOT / "prompts" / "codex_tasks").is_dir():
        return fail("E_ROOT_RETIRE_PREFLIGHT_MISSING", "prompts/codex_tasks")
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
            "non-destructive preflight",
            "does not delete, move, rename, or path-rewrite any file",
            "ROOT_LOW_RISK_RETIRE_DELETE_V1",
            "approval-gated",
            "not that approval",
            "docs/ssot/**",
        ],
        "E_ROOT_RETIRE_PREFLIGHT_DOC",
    )
    if rc:
        return rc
    return require_tokens(
        APPROVAL,
        [
            "approval_requested",
            "no_delete",
            "no_move",
            "This approval package is not deletion approval by itself",
            "must receive explicit user approval",
        ],
        "E_ROOT_RETIRE_PREFLIGHT_APPROVAL",
    )


def run_no_match_rg(pattern: str, paths: list[str], code: str) -> int:
    command = [
        "rg",
        "-n",
        "-g",
        "!tests/run_root_low_risk_retire_delete_preflight_check.py",
        "-g",
        "!tests/run_root_low_risk_retire_delete_dry_run_plan_check.py",
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
    common_paths = [
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
    checks = [
        (
            r"prompts/codex_tasks|docs/context/codex_tasks/legacy_prompts",
            common_paths,
            "E_ROOT_RETIRE_PREFLIGHT_PROMPTS_REF",
        ),
        (
            r"const_reassign_error\.test\.json|decl_block_ok\.test\.json",
            [
                "AGENTS.md",
                "CLAUDE.md",
                "README.md",
                "scripts",
                "tests",
                "tools",
                "tool",
                "solutions",
                ".github",
                ".gitlab-ci.yml",
                "azure-pipelines.yml",
            ],
            "E_ROOT_RETIRE_PREFLIGHT_GOLDEN_REF",
        ),
        (
            r"cargo-local\.ps1|seamgrim_seed_candidates\.py|ssot_edu_diff_report\.py",
            common_paths,
            "E_ROOT_RETIRE_PREFLIGHT_SCRIPT_REF",
        ),
        (
            r"guides/math/MATH_SUPPORT_CATALOG_V0\.md|docs/guides/math/MATH_SUPPORT_CATALOG_V0\.md|MATH_SUPPORT_CATALOG_V0\.md",
            common_paths,
            "E_ROOT_RETIRE_PREFLIGHT_GUIDE_REF",
        ),
    ]
    for pattern, paths, code in checks:
        rc = run_no_match_rg(pattern, paths, code)
        if rc:
            return rc
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "ROOT_LOW_RISK_RETIRE_DELETE_PREFLIGHT_V1",
        "closed by `ROOT_LOW_RISK_RETIRE_DELETE_PREFLIGHT_V1.md`",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "Approval-gated",
        "Do not delete, move, or retire root legacy files until the user explicitly approves",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ROOT_RETIRE_PREFLIGHT_QUEUE", str(missing))
    if "1. `ROOT_LOW_RISK_RETIRE_DELETE_V1`" not in text:
        return fail("E_ROOT_RETIRE_PREFLIGHT_NEXT", "root retire delete is not next")
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
        return fail("E_ROOT_RETIRE_PREFLIGHT_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROOT_RETIRE_PREFLIGHT_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_active_references,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[root-low-risk-retire-delete-preflight-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
