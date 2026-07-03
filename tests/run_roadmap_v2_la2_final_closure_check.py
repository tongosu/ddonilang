#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROADMAP_V2_LA2_FINAL_CLOSURE_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "라-2_REPORT_20260604.md"
ROUNDTRIP_EXPECTED = (
    ROOT / "pack" / "seamgrim_malblock_roundtrip_subset_v1" / "expected" / "malblock_roundtrip_subset.detjson"
)
ROADMAP = (
    ROOT
    / "docs"
    / "context"
    / "proposals"
    / "PROPOSAL_ROADMAP_V2_GANADA_15_JULGI_6_MARU_MASTER_20260426_FULL_R2.md"
)


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        DOC,
        ROOT / "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1.md",
        ROOT / "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1.md",
        ROUNDTRIP_EXPECTED,
        ROOT / "pack" / "block_editor_roundtrip_v1" / "expected" / "block_editor_roundtrip.detjson",
        ROOT / "pack" / "block_editor_raw_fallback_v1" / "expected" / "block_editor_roundtrip.detjson",
        ROOT / "tests" / "run_seamgrim_malblock_roundtrip_subset_check.py",
        ROOT / "tests" / "run_block_editor_roundtrip_check.py",
        REPORT,
        QUEUE,
        ROADMAP,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROADMAP_V2_LA2_FINAL_MISSING", str(missing))
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
            "ROADMAP_V2 `라-2`",
            "DDN -> 말블록 subset roundtrip",
            "raw/opaque block 보존",
            "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1",
            "canonBlockEditorPlan",
            "raw_block_count=0",
            "매김",
            "does not claim",
            "full block editor UI integration",
            "arbitrary DDN grammar support",
            "(조건)이 될때",
            "(조건)인 동안",
            "ROADMAP_V2_A1_NURIGYM_REBASE_V1",
        ],
        "E_ROADMAP_V2_LA2_FINAL_DOC",
    )


def check_report() -> int:
    return require_tokens(
        REPORT,
        [
            "ROADMAP_V2 라-2",
            "Matrix 상태 제안",
            "닫힘",
            "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1",
            "raw_block_count=0",
            "canonical equality",
            "(조건)이 될때",
            "(조건)인 동안",
            "ROADMAP_V2_A1_NURIGYM_REBASE_V1",
            "docs/ssot/**",
        ],
        "E_ROADMAP_V2_LA2_FINAL_REPORT",
    )


def check_roadmap_source() -> int:
    return require_tokens(
        ROADMAP,
        [
            "라-2-01 DDN → 말블록 subset roundtrip",
            "라-2-02 raw/opaque block 보존",
            "DDN text/canon이 정본이다",
            "말블록 JSON/XML은 편집 보조 상태다",
            "말블록은 DDN을 대체하지 않는다",
            "미지원 DDN은 삭제하지 않고 raw/opaque block으로 보존한다",
        ],
        "E_ROADMAP_V2_LA2_FINAL_ROADMAP",
    )


def check_roundtrip_expected() -> int:
    payload = json.loads(ROUNDTRIP_EXPECTED.read_text(encoding="utf-8"))
    checks = {
        "schema": payload.get("schema") == "ddn.seamgrim_malblock_roundtrip_subset_report.v1",
        "all_canon_equal": payload.get("all_canon_equal") is True,
        "case_count": payload.get("case_count") == 5,
        "supported_case_count": payload.get("supported_case_count") == 4,
        "raw_fallback_case_count": payload.get("raw_fallback_case_count") == 1,
    }
    bad = [name for name, ok in checks.items() if not ok]
    if bad:
        return fail("E_ROADMAP_V2_LA2_FINAL_EXPECTED", str(bad))
    excluded_text = json.dumps(payload.get("excluded_surfaces", []), ensure_ascii=False)
    for token in ["(조건)이 될때", "(조건)인 동안"]:
        if token not in excluded_text:
            return fail("E_ROADMAP_V2_LA2_FINAL_EXCLUDED", token)
    for row in payload.get("cases", []):
        if row.get("canon_equal") is not True:
            return fail("E_ROADMAP_V2_LA2_FINAL_CASE_CANON", str(row.get("id")))
    return 0


def run_support_checks() -> int:
    for command in [
        ["python", "tests/run_seamgrim_malblock_roundtrip_subset_check.py"],
        ["python", "tests/run_block_editor_roundtrip_check.py"],
    ]:
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
            return fail("E_ROADMAP_V2_LA2_FINAL_SUPPORT_CHECK", result.stdout.strip())
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "ROADMAP_V2_LA2_FINAL_CLOSURE_V1",
        "closed by `ROADMAP_V2_LA2_FINAL_CLOSURE_V1.md`",
        "ROADMAP_V2 `라-2` is closed",
        "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1",
        "closed by `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1.md`",
        "ROADMAP_V2_A1_NURIGYM_REBASE_V1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "No automatic next development item is selected.",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_LA2_FINAL_QUEUE", str(missing))
    if "1. `ROADMAP_V2_LA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_LA2_FINAL_STILL_OPEN",
            "ROADMAP_V2_LA2_FINAL_CLOSURE_V1 is still listed as the next open item",
        )
    if "1. `ROOT_LOW_RISK_RETIRE_DELETE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_LA2_FINAL_NEXT",
            "ROOT_LOW_RISK_RETIRE_DELETE_V1 is still listed as the next open item",
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
        return fail("E_ROADMAP_V2_LA2_FINAL_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROADMAP_V2_LA2_FINAL_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc,
        check_report,
        check_roadmap_source,
        check_roundtrip_expected,
        run_support_checks,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[roadmap-v2-la2-final-closure-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
