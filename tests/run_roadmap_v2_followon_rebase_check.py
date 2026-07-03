#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REBASE = ROOT / "ROADMAP_V2_FOLLOWON_REBASE_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        REBASE,
        QUEUE,
        ROOT / "AGE5_CI_FRONTIER_RECHECK_V1.md",
        ROOT / "ROADMAP_V2_SA1_REBASE_V1.md",
        ROOT / "ROADMAP_V2_NA2_UNIT_RANDOM_EVENT_REBASE_V1.md",
        ROOT / "STD_EVENT_MINIMUM_CLOSURE_V1.md",
        TRACKER,
        MANIFEST,
        ROOT / "pack" / "relation_solve_ddn_bridge_v2" / "golden.jsonl",
        ROOT / "pack" / "relation_solve_system_2x2_v1" / "golden.jsonl",
        ROOT / "pack" / "formula_relation_solve_v1" / "golden.jsonl",
        ROOT / "pack" / "math_numeric_int_v1" / "golden.jsonl",
        ROOT / "pack" / "math_calculus_v1" / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROADMAP_V2_FOLLOWON_REBASE_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_rebase_doc() -> int:
    return require_tokens(
        REBASE,
        [
            "documentation/checker-only",
            "no product code",
            "거-1+",
            "do not start `거-1+`",
            "public registry final",
            "do not start public registry final",
            "`다-1` exact/vector/function first-run",
            "ROADMAP_V2_DA1_MATH_REBASE_V1",
            "ROOT_LOW_RISK_RETIRE_DELETE_V1",
            "Approval-gated",
            "docs/ssot/**",
        ],
        "E_ROADMAP_V2_FOLLOWON_REBASE_DOC",
    )


def check_roadmap_sources() -> int:
    tracker = read(TRACKER)
    for token in [
        "`거-0` | 질문카드 schema",
        "`카-1` | 또니마루 server/local MVP",
        "public registry final",
        "거-1+",
    ]:
        if token not in tracker:
            return fail("E_ROADMAP_V2_FOLLOWON_REBASE_TRACKER", token)
    matrix = read(
        ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
    )
    for token in ["exact/vector/function", "math smoke pack"]:
        if token not in matrix:
            return fail("E_ROADMAP_V2_FOLLOWON_REBASE_MATRIX", token)
    manifest = read(MANIFEST)
    for token in [
        "`거-0`",
        "`카-1`",
        "public registry final/install/update/remove/team-internal membership enforcement 없음",
    ]:
        if token not in manifest:
            return fail("E_ROADMAP_V2_FOLLOWON_REBASE_MANIFEST", token)
    return 0


def check_math_evidence_exists_but_unrebased() -> int:
    dev_summary = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    for token in [
        "relation_solve_ddn_bridge_v2",
        "relation_solve_system_2x2_v1",
        "formula_relation_solve_v1",
        "math_numeric_int_v1",
        "math_calculus_v1",
    ]:
        if token not in dev_summary and not (ROOT / "pack" / token / "golden.jsonl").exists():
            return fail("E_ROADMAP_V2_FOLLOWON_REBASE_MATH_EVIDENCE", token)
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "ROADMAP_V2_FOLLOWON_REBASE_V1",
        "closed by `ROADMAP_V2_FOLLOWON_REBASE_V1.md`",
        "ROADMAP_V2_DA1_MATH_REBASE_V1",
        "closed by `ROADMAP_V2_DA1_MATH_REBASE_V1.md`",
        "MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1",
        "do not start `거-1+`",
        "public registry final without explicit user selection",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "Approval-gated",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_FOLLOWON_REBASE_QUEUE", str(missing))
    if "1. `ROADMAP_V2_FOLLOWON_REBASE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_FOLLOWON_REBASE_QUEUE_OPEN",
            "follow-on rebase is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_DA1_MATH_REBASE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_FOLLOWON_REBASE_DA1_OPEN",
            "DA1 rebase is still listed as the next open item",
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
        return fail("E_ROADMAP_V2_FOLLOWON_REBASE_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROADMAP_V2_FOLLOWON_REBASE_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_rebase_doc,
        check_roadmap_sources,
        check_math_evidence_exists_but_unrebased,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[roadmap-v2-followon-rebase-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
