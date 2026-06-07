#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_DIRTY_BASELINE_RECHECK_V1.md"
PREV = ROOT / "ROADMAP_V2_STUDIO_PRODUCTIZATION_REBASE_V1.md"
REPORT = ROOT / "docs" / "studio" / "DIRTY_BASELINE_RECHECK_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_dirty_baseline_recheck_v1"
RECHECK = PACK / "recheck.detjson"
STUDIO_ROADMAP = ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
NEXT = "STUDIO_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(path: Path) -> dict:
    return json.loads(read(path))


def run(cmd: list[str], *, timeout: int = 120) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def require_files() -> int:
    required = [
        DOC,
        PREV,
        REPORT,
        INDEX,
        STUDIO_ROADMAP,
        PACK / "README.md",
        PACK / "contract.detjson",
        RECHECK,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "run_roadmap_v2_studio_productization_rebase_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_DIRTY_BASELINE_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_docs() -> int:
    checks = [
        (
            DOC,
            [
                "STUDIO_DIRTY_BASELINE_RECHECK_V1",
                "dirty baseline verification/separation",
                "tracked dirty entries: at least `116`",
                "untracked entries: at least `612`",
                "Logical Separation",
                "1시대 5/5 = 100%, 전체 5/18 = 28%",
                NEXT,
                "docs/ssot/**",
            ],
            "E_STUDIO_DIRTY_BASELINE_DOC",
        ),
        (
            REPORT,
            [
                "Studio Dirty Baseline Recheck V1",
                "tracked dirty entries observed: at least `116`",
                "untracked entries observed: at least `612`",
                "1시대 5/5 = 100%, 전체 5/18 = 28%",
                NEXT,
            ],
            "E_STUDIO_DIRTY_BASELINE_REPORT",
        ),
        (
            INDEX,
            [
                "STUDIO_DIRTY_BASELINE_RECHECK_V1",
                "docs/studio/DIRTY_BASELINE_RECHECK_V1.md",
                "pack/studio_dirty_baseline_recheck_v1",
                "tests/run_studio_dirty_baseline_recheck.py",
            ],
            "E_STUDIO_DIRTY_BASELINE_INDEX",
        ),
        (
            STUDIO_ROADMAP,
            [
                "STUDIO_DIRTY_BASELINE_RECHECK_V1",
                "1시대 5/5 = 100%",
                "전체 5/18 = 28%",
                NEXT,
            ],
            "E_STUDIO_DIRTY_BASELINE_ROADMAP",
        ),
        (
            PREV,
            [
                "STUDIO_DIRTY_BASELINE_RECHECK_V1",
                "Era 1 is fully closed",
                "5/18 = 28%",
            ],
            "E_STUDIO_DIRTY_BASELINE_PREV",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_recheck() -> int:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_dirty_baseline_recheck_v1",
        "kind": "studio_dirty_baseline_recheck",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_DIRTY_BASELINE_RECHECK_V1",
        "based_on": "ROADMAP_V2_STUDIO_PRODUCTIZATION_REBASE_V1",
        "observed_tracked_dirty_min": 116,
        "observed_untracked_min": 612,
        "docs_ssot_clean": True,
        "logical_separation": True,
        "era1_closed_items": 5,
        "era1_total_items": 5,
        "super_long_closed_items": 5,
        "super_long_total_items": 18,
        "selected_next_item": NEXT,
        "runtime_surface_claim": False,
        "lesson_schema_change": False,
        "active_allowlist_change": False,
        "nurigym_a2_claim": False,
        "public_release_claim": False,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_DIRTY_BASELINE_CONTRACT", f"{key}={contract.get(key)!r}")

    recheck = load_json(RECHECK)
    if recheck.get("schema") != "ddn.studio.dirty_baseline_recheck.v1":
        return fail("E_STUDIO_DIRTY_BASELINE_SCHEMA", repr(recheck.get("schema")))
    progress = recheck.get("progress", {})
    for key, value in {
        "era1_done": 5,
        "era1_total": 5,
        "super_long_done": 5,
        "super_long_total": 18,
        "super_long_percent": 28,
        "ma3_done": 1,
        "ma3_required": 4,
        "ma3_percent": 25,
        "roadmap_v2_closed_cells": 21,
        "roadmap_v2_total_cells": 90,
        "roadmap_v2_percent": 23,
    }.items():
        if progress.get(key) != value:
            return fail("E_STUDIO_DIRTY_BASELINE_PROGRESS", f"{key}={progress.get(key)!r}")
    if recheck.get("selected_next_item") != NEXT:
        return fail("E_STUDIO_DIRTY_BASELINE_NEXT", repr(recheck.get("selected_next_item")))
    for key, value in recheck.get("false_claims", {}).items():
        if value is not False:
            return fail("E_STUDIO_DIRTY_BASELINE_FALSE_CLAIM", f"{key}={value!r}")
    return 0


def check_current_dirty_shape() -> int:
    proc = run(["git", "status", "--short"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_DIRTY_BASELINE_STATUS", proc.stdout.strip())
    lines = [line for line in proc.stdout.splitlines() if line.strip()]
    tracked = [line for line in lines if not line.startswith("??")]
    untracked = [line for line in lines if line.startswith("??")]
    if len(tracked) < 116:
        return fail("E_STUDIO_DIRTY_BASELINE_TRACKED_COUNT", str(len(tracked)))
    if len(untracked) < 612:
        return fail("E_STUDIO_DIRTY_BASELINE_UNTRACKED_COUNT", str(len(untracked)))
    status = "\n".join(lines)
    for token in [
        "lang/src/parser.rs",
        "tool/src/ddn_runtime.rs",
        "tools/teul-cli/src/runtime/eval.rs",
        "core/src/nurigym",
    ]:
        if token not in status:
            return fail("E_STUDIO_DIRTY_BASELINE_DIRTY_TOKEN", token)
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_DIRTY_BASELINE_RECHECK_V1",
        "dirty baseline logically separated",
        f"next: {NEXT}",
        "progress: 5/18 = 28%",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_DIRTY_BASELINE_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_dirty_baseline_recheck_v1"],
        ["python", "tests/run_roadmap_v2_studio_productization_rebase_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=240)
        if proc.returncode != 0:
            return fail("E_STUDIO_DIRTY_BASELINE_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    return require_tokens(
        DEV_SUMMARY,
        [
            "STUDIO_DIRTY_BASELINE_RECHECK_V1",
            "studio_dirty_baseline_recheck_v1",
            NEXT,
            "초장기 계획: 1시대 5/5 = 100%, 전체 5/18 = 28%",
            "ROADMAP_V2 전체: queue-expanded 21/90 = 23%",
            "docs/ssot/** 변경 없음",
        ],
        "E_STUDIO_DIRTY_BASELINE_DEV_SUMMARY",
    )


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_DIRTY_BASELINE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_DIRTY_BASELINE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_recheck,
        check_current_dirty_shape,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-dirty-baseline-recheck-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
