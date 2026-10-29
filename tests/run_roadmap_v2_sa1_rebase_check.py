#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REBASE = ROOT / "ROADMAP_V2_SA1_REBASE_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def require_files() -> int:
    required = [
        REBASE,
        QUEUE,
        TRACKER,
        ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md",
        ROOT / "docs" / "status" / "roadmap_v2" / "사-1_REPORT_20260503.md",
        ROOT / "docs" / "status" / "roadmap_v2" / "사-1-후속_CELL_GRID_REPORT_20260509.md",
        ROOT / "docs" / "status" / "roadmap_v2" / "나-1_REPORT_20260506.md",
        ROOT / "docs" / "status" / "roadmap_v2" / "나-1-보정_REPORT_20260507.md",
        ROOT / "docs" / "status" / "roadmap_v2" / "나-2_PRE_REPORT_20260507.md",
        ROOT / "docs" / "status" / "roadmap_v2" / "카-1_REPORT_20260512.md",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROADMAP_V2_SA1_REBASE_MISSING", str(missing))
    return 0


def row_has_closed(text: str, coord: str) -> bool:
    for line in text.splitlines():
        if f"`{coord}`" in line and "닫힘" in line:
            return True
    return False


def check_tracker() -> int:
    text = TRACKER.read_text(encoding="utf-8")
    for coord in ["사-1", "나-1", "나-1-후속", "카-1"]:
        if not row_has_closed(text, coord):
            return fail("E_ROADMAP_V2_SA1_REBASE_TRACKER_NOT_CLOSED", coord)
    for token in ["public registry final", "나-2", "사-1_REPORT_20260503.md"]:
        if token not in text:
            return fail("E_ROADMAP_V2_SA1_REBASE_TRACKER_TOKEN", token)
    return 0


def check_rebase_doc() -> int:
    text = REBASE.read_text(encoding="utf-8")
    required_tokens = [
        "사-1` is already closed",
        "Do not start `ROADMAP_V2_SA1_IMPLEMENTATION_V1`",
        "superseded",
        "ROADMAP_V2_NA2_UNIT_RANDOM_EVENT_REBASE_V1",
        "ROADMAP_V2_NA2_UNIT_RANDOM_EVENT_IMPLEMENTATION_V1",
        "AGE5_CI_FRONTIER_RECHECK_V1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "approval-gated",
        "No product code",
        "docs/ssot/**",
    ]
    missing = [token for token in required_tokens if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_SA1_REBASE_DOC_TOKENS", str(missing))
    return 0


def check_queue() -> int:
    text = QUEUE.read_text(encoding="utf-8")
    required_tokens = [
        "ROADMAP_V2_SA1_REBASE_V1.md",
        "ROADMAP_V2_SA1_IMPLEMENTATION_V1` is superseded",
        "ROADMAP_V2_NA2_UNIT_RANDOM_EVENT_REBASE_V1",
        "ROADMAP_V2_NA2_EVENT_MINIMUM_CLOSURE_REBASE_V1",
        "STD_EVENT_MINIMUM_CLOSURE_V1",
        "AGE5_CI_FRONTIER_RECHECK_V1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
    ]
    missing = [token for token in required_tokens if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_SA1_REBASE_QUEUE_TOKENS", str(missing))
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
        return fail("E_ROADMAP_V2_SA1_REBASE_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROADMAP_V2_SA1_REBASE_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_tracker,
        check_rebase_doc,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[roadmap-v2-sa1-rebase-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
