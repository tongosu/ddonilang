#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REBASE = ROOT / "ROADMAP_V2_NA2_EVENT_MINIMUM_CLOSURE_REBASE_V1.md"
EVENT_CLOSURE = ROOT / "STD_EVENT_MINIMUM_CLOSURE_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def require_files() -> int:
    required = [
        ROOT / "ROADMAP_V2_NA2_UNIT_RANDOM_EVENT_REBASE_V1.md",
        REBASE,
        EVENT_CLOSURE,
        ROOT / "pack" / "seamgrim_event_surface_canon_v1" / "golden.jsonl",
        ROOT / "pack" / "seamgrim_event_model_ir_v1" / "golden.jsonl",
        ROOT / "pack" / "vol4_event_dispatch_runtime_v1" / "golden.jsonl",
        ROOT / "pack" / "std_event_minimum_closure_v1" / "contract.detjson",
        ROOT / "pack" / "std_event_minimum_closure_v1" / "golden.jsonl",
        ROOT / "tests" / "run_std_event_minimum_closure_check.py",
        ROOT / "tests" / "run_pack_golden_event_model_selftest.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROADMAP_V2_NA2_EVENT_REBASE_MISSING", str(missing))
    return 0


def check_rebase_doc() -> int:
    text = REBASE.read_text(encoding="utf-8")
    required_tokens = [
        "STD_EVENT_MINIMUM_CLOSURE_V1",
        "seamgrim_event_surface_canon_v1",
        "seamgrim_event_model_ir_v1",
        "vol4_event_dispatch_runtime_v1",
        "supporting UI/web evidence",
        "not the core event minimum closure claim",
        "Do not claim full actor-event semantics",
        "docs/ssot/**",
    ]
    missing = [token for token in required_tokens if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_NA2_EVENT_REBASE_DOC_TOKENS", str(missing))
    return 0


def check_closure_doc() -> int:
    text = EVENT_CLOSURE.read_text(encoding="utf-8")
    required_tokens = [
        "no new stdlib public surface",
        '"KIND"라는 알림이 오면',
        "알림씨",
        "임자",
        "받으면",
        "~~>",
        "Full actor-event native runtime semantics",
        "Browser keyboard/input delivery",
        "Block editor UI codec/rendering claim",
    ]
    missing = [token for token in required_tokens if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_NA2_EVENT_CLOSURE_DOC_TOKENS", str(missing))
    return 0


def check_queue() -> int:
    text = QUEUE.read_text(encoding="utf-8")
    required_tokens = [
        "ROADMAP_V2_NA2_EVENT_MINIMUM_CLOSURE_REBASE_V1.md",
        "STD_EVENT_MINIMUM_CLOSURE_V1",
        "full actor-event semantics remain deferred",
        "AGE5_CI_FRONTIER_RECHECK_V1",
        "ROADMAP_V2_FOLLOWON_REBASE_V1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
    ]
    missing = [token for token in required_tokens if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_NA2_EVENT_REBASE_QUEUE_TOKENS", str(missing))
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
        return fail("E_ROADMAP_V2_NA2_EVENT_REBASE_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROADMAP_V2_NA2_EVENT_REBASE_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_rebase_doc,
        check_closure_doc,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[roadmap-v2-na2-event-minimum-closure-rebase-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
