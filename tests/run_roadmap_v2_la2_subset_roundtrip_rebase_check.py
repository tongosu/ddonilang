#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"
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
        ROOT / "ROADMAP_V2_GA2_FINAL_CLOSURE_V1.md",
        ROOT / "LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1.md",
        ROOT / "pack" / "lang_core_2_v1" / "expected" / "lang_core_2.detjson",
        ROOT / "pack" / "seamgrim_malblock_codegen_v1" / "expected" / "malblock_codegen.detjson",
        ROOT / "pack" / "block_editor_roundtrip_v1" / "expected" / "block_editor_roundtrip.detjson",
        ROOT / "pack" / "block_editor_raw_fallback_v1" / "expected" / "block_editor_roundtrip.detjson",
        ROOT / "tests" / "run_lang_core_2_representative_grammar_pack_check.py",
        ROOT / "tests" / "run_seamgrim_malblock_codegen_check.py",
        ROOT / "tests" / "run_block_editor_roundtrip_check.py",
        ROADMAP,
        QUEUE,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROADMAP_V2_LA2_REBASE_MISSING", str(missing))
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
            "documentation/checker-only",
            "ROADMAP_V2 `라-2`",
            "DDN -> 말블록 subset roundtrip",
            "ROADMAP_V2_GA2_FINAL_CLOSURE_V1",
            "pack/lang_core_2_v1",
            "seamgrim_malblock_codegen_v1",
            "block_editor_roundtrip_v1",
            "block_editor_raw_fallback_v1",
            "`라-2` remains open",
            "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1",
            "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1",
            "pack/seamgrim_malblock_roundtrip_subset_v1",
            "canonical DDN bytes",
            "raw/opaque blocks",
            "test-only lowering",
            "docs/ssot/**",
        ],
        "E_ROADMAP_V2_LA2_REBASE_DOC",
    )


def check_roadmap_source() -> int:
    return require_tokens(
        ROADMAP,
        [
            "라-2 말블록 닫힘",
            "DDN → 말블록 subset roundtrip",
            "raw/opaque block 보존",
            "말블록 JSON/XML은 편집 보조 상태다",
            "말블록은 DDN을 대체하지 않는다",
        ],
        "E_ROADMAP_V2_LA2_REBASE_ROADMAP",
    )


def check_existing_evidence() -> int:
    lang_core_2 = json.loads(
        (ROOT / "pack" / "lang_core_2_v1" / "expected" / "lang_core_2.detjson").read_text(
            encoding="utf-8"
        )
    )
    surfaces = {
        surface
        for case in lang_core_2.get("cases", [])
        for surface in case.get("surfaces", [])
    }
    required_surfaces = {"채비", "훅", "조건", "임자", "계약"}
    missing_surfaces = sorted(required_surfaces - surfaces)
    if missing_surfaces:
        return fail("E_ROADMAP_V2_LA2_REBASE_LANG_CORE_2", str(missing_surfaces))

    malblock = json.loads(
        (
            ROOT / "pack" / "seamgrim_malblock_codegen_v1" / "expected" / "malblock_codegen.detjson"
        ).read_text(encoding="utf-8")
    )
    case_ids = {str(row.get("id")) for row in malblock.get("cases", [])}
    if not {"charim_variable_show", "if_else_show", "choose_exhaustive_show"}.issubset(case_ids):
        return fail("E_ROADMAP_V2_LA2_REBASE_MALBLOCK_CODEGEN", str(sorted(case_ids)))

    roundtrip = json.loads(
        (
            ROOT
            / "pack"
            / "block_editor_roundtrip_v1"
            / "expected"
            / "block_editor_roundtrip.detjson"
        ).read_text(encoding="utf-8")
    )
    if not roundtrip.get("canon_equal"):
        return fail("E_ROADMAP_V2_LA2_REBASE_BLOCK_ROUNDTRIP", "canon_equal must be true")
    if roundtrip.get("raw_block_count") != 0:
        return fail("E_ROADMAP_V2_LA2_REBASE_BLOCK_ROUNDTRIP_RAW", "raw count must be 0")

    raw = json.loads(
        (
            ROOT
            / "pack"
            / "block_editor_raw_fallback_v1"
            / "expected"
            / "block_editor_roundtrip.detjson"
        ).read_text(encoding="utf-8")
    )
    if not raw.get("canon_equal"):
        return fail("E_ROADMAP_V2_LA2_REBASE_RAW_FALLBACK", "canon_equal must be true")
    if int(raw.get("raw_block_count", 0)) < 1:
        return fail("E_ROADMAP_V2_LA2_REBASE_RAW_FALLBACK_COUNT", "raw_block_count < 1")
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1",
        "closed by `ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1.md`",
        "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1",
        "closed by `BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1.md`",
        "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1",
        "closed by `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1.md`",
        "pack/seamgrim_malblock_roundtrip_subset_v1",
        "ROADMAP_V2_LA2_FINAL_CLOSURE_V1",
        "closed by `ROADMAP_V2_LA2_FINAL_CLOSURE_V1.md`",
        "ROADMAP_V2_A1_NURIGYM_REBASE_V1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "Approval-gated",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_LA2_REBASE_QUEUE", str(missing))
    if "1. `ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_LA2_REBASE_STILL_OPEN",
            "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1 is still listed as next open item",
        )
    if "1. `BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1`" in text:
        return fail(
            "E_ROADMAP_V2_LA2_REBASE_REFRESH_OPEN",
            "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1 is still listed as next open item",
        )
    if "1. `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1`" in text:
        return fail(
            "E_ROADMAP_V2_LA2_REBASE_NEXT",
            "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1 is still listed as next open item",
        )
    if "1. `ROADMAP_V2_LA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_LA2_REBASE_FINAL_NEXT",
            "ROADMAP_V2_LA2_FINAL_CLOSURE_V1 is still listed as next open item",
        )
    if "1. `ROOT_LOW_RISK_RETIRE_DELETE_V1`" not in text:
        return fail(
            "E_ROADMAP_V2_LA2_REBASE_A1_NEXT",
            "ROOT_LOW_RISK_RETIRE_DELETE_V1 is not next open item",
        )
    return 0


def run_support_checks() -> int:
    for command in [
        ["python", "tests/run_seamgrim_malblock_codegen_check.py"],
        ["python", "tests/run_lang_core_2_representative_grammar_pack_check.py"],
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
            return fail("E_ROADMAP_V2_LA2_REBASE_SUPPORT_CHECK", result.stdout.strip())
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
        return fail("E_ROADMAP_V2_LA2_REBASE_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROADMAP_V2_LA2_REBASE_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc,
        check_roadmap_source,
        check_existing_evidence,
        check_queue,
        run_support_checks,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[roadmap-v2-la2-subset-roundtrip-rebase-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
