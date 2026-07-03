#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REBASE = ROOT / "ROADMAP_V2_NA2_UNIT_RANDOM_EVENT_REBASE_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def require_files() -> int:
    required = [
        ROOT / "ROADMAP_V2_SA1_REBASE_V1.md",
        REBASE,
        QUEUE,
        ROOT / "docs" / "status" / "roadmap_v2" / "나-1_REPORT_20260506.md",
        ROOT / "docs" / "status" / "roadmap_v2" / "나-1-보정_REPORT_20260507.md",
        ROOT / "docs" / "status" / "roadmap_v2" / "나-2_PRE_REPORT_20260507.md",
        ROOT / "STD_CORE_GRID_UNIT_CLOSURE_V1.md",
        ROOT / "STD_RANDOM_BAG_MINIMUM_V1.md",
        ROOT / "STD_INPUT_MAP_CLOSURE_V1.md",
        ROOT / "pack" / "std_core_grid_unit_closure_v1" / "golden.jsonl",
        ROOT / "pack" / "std_random_bag_minimum_v1" / "golden.jsonl",
        ROOT / "pack" / "std_input_map_closure_v1" / "golden.jsonl",
        ROOT / "pack" / "lang_unit_temp_smoke_v1" / "golden.jsonl",
        ROOT / "tests" / "run_std_core_grid_unit_closure_check.py",
        ROOT / "tests" / "run_std_random_bag_pack_check.py",
        ROOT / "tests" / "run_std_input_map_closure_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROADMAP_V2_NA2_REBASE_MISSING", str(missing))
    return 0


def check_rebase_doc() -> int:
    text = REBASE.read_text(encoding="utf-8")
    required_tokens = [
        "unit and random axes",
        "No product code changes",
        "STD_CORE_GRID_UNIT_CLOSURE_V1",
        "STD_RANDOM_BAG_MINIMUM_V1",
        "STD_INPUT_MAP_CLOSURE_V1",
        "Do not claim `event` as closed",
        "ROADMAP_V2_NA2_EVENT_MINIMUM_CLOSURE_REBASE_V1",
        "ROADMAP_V2_NA2_UNIT_RANDOM_EVENT_IMPLEMENTATION_V1",
        "too stale and too wide",
        "full actor-event semantics",
        "docs/ssot/**",
    ]
    missing = [token for token in required_tokens if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_NA2_REBASE_DOC_TOKENS", str(missing))
    return 0


def check_existing_evidence_docs() -> int:
    core_unit = (ROOT / "STD_CORE_GRID_UNIT_CLOSURE_V1.md").read_text(encoding="utf-8")
    random = (ROOT / "STD_RANDOM_BAG_MINIMUM_V1.md").read_text(encoding="utf-8")
    input_map = (ROOT / "STD_INPUT_MAP_CLOSURE_V1.md").read_text(encoding="utf-8")
    if "lang_unit_temp_smoke_v1" not in core_unit:
        return fail("E_ROADMAP_V2_NA2_REBASE_UNIT_EVIDENCE", "lang_unit_temp_smoke_v1")
    if "무작위가방.만들기" not in random or "std_random_bag_minimum_v1" not in random:
        return fail("E_ROADMAP_V2_NA2_REBASE_RANDOM_EVIDENCE", "std_random_bag minimum")
    if "std_input_map" not in input_map or "New input devices or event semantics" not in input_map:
        return fail("E_ROADMAP_V2_NA2_REBASE_INPUT_BOUNDARY", "input-map event boundary")
    return 0


def check_queue() -> int:
    text = QUEUE.read_text(encoding="utf-8")
    required_tokens = [
        "ROADMAP_V2_NA2_UNIT_RANDOM_EVENT_REBASE_V1.md",
        "unit and random axes are already closed",
        "ROADMAP_V2_NA2_EVENT_MINIMUM_CLOSURE_REBASE_V1",
        "STD_EVENT_MINIMUM_CLOSURE_V1",
        "full actor-event semantics remain deferred",
        "AGE5_CI_FRONTIER_RECHECK_V1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
    ]
    missing = [token for token in required_tokens if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_NA2_REBASE_QUEUE_TOKENS", str(missing))
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
        return fail("E_ROADMAP_V2_NA2_REBASE_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROADMAP_V2_NA2_REBASE_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_rebase_doc,
        check_existing_evidence_docs,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[roadmap-v2-na2-unit-random-event-rebase-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
