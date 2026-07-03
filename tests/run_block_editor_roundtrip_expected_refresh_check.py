#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"
PACK_SUPPORTED = ROOT / "pack" / "block_editor_roundtrip_v1" / "expected" / "block_editor_roundtrip.detjson"
PACK_RAW = ROOT / "pack" / "block_editor_raw_fallback_v1" / "expected" / "block_editor_roundtrip.detjson"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        DOC,
        ROOT / "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1.md",
        ROOT / "tests" / "block_editor_roundtrip_runner.mjs",
        ROOT / "tests" / "run_block_editor_roundtrip_check.py",
        ROOT / "pack" / "block_editor_roundtrip_v1" / "fixtures" / "source.ddn",
        PACK_SUPPORTED,
        ROOT / "pack" / "block_editor_raw_fallback_v1" / "fixtures" / "source.ddn",
        PACK_RAW,
        QUEUE,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_BLOCK_EDITOR_ROUNDTRIP_REFRESH_MISSING", str(missing))
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
            "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1",
            "block_editor_roundtrip_v1",
            "block_editor_raw_fallback_v1",
            "tests/block_editor_roundtrip_runner.mjs",
            "tests/run_block_editor_roundtrip_check.py",
            "canon_equal = true",
            "raw_block_count = 0",
            "raw_block_count >= 1",
            "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1",
            "docs/ssot/**",
        ],
        "E_BLOCK_EDITOR_ROUNDTRIP_REFRESH_DOC",
    )


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def check_expected_supported() -> int:
    payload = load_json(PACK_SUPPORTED)
    checks = {
        "schema": payload.get("schema") == "ddn.block_editor_roundtrip_smoke.v1",
        "pack": payload.get("pack") == "block_editor_roundtrip_v1",
        "block_plan_schema": payload.get("block_plan_schema") == "ddn.block_editor_plan.v1",
        "canon_equal": payload.get("canon_equal") is True,
        "decode_errors": payload.get("decode_errors") == [],
        "raw_block_count": payload.get("raw_block_count") == 0,
        "raw_blocks": payload.get("raw_blocks") == [],
        "canon_current_if": "만약 t < 3 이라면" in str(payload.get("canon_before", "")),
    }
    bad = [name for name, ok in checks.items() if not ok]
    if bad:
        return fail("E_BLOCK_EDITOR_ROUNDTRIP_REFRESH_SUPPORTED", str(bad))
    return 0


def check_expected_raw() -> int:
    payload = load_json(PACK_RAW)
    raw_blocks = payload.get("raw_blocks")
    checks = {
        "schema": payload.get("schema") == "ddn.block_editor_roundtrip_smoke.v1",
        "pack": payload.get("pack") == "block_editor_raw_fallback_v1",
        "block_plan_schema": payload.get("block_plan_schema") == "ddn.block_editor_plan.v1",
        "canon_equal": payload.get("canon_equal") is True,
        "decode_errors": payload.get("decode_errors") == [],
        "raw_block_count": int(payload.get("raw_block_count", 0)) >= 1,
        "raw_blocks": isinstance(raw_blocks, list) and len(raw_blocks) >= 1,
        "raw_preserves_description": "설명" in str(raw_blocks),
    }
    bad = [name for name, ok in checks.items() if not ok]
    if bad:
        return fail("E_BLOCK_EDITOR_ROUNDTRIP_REFRESH_RAW", str(bad))
    return 0


def run_roundtrip_checker() -> int:
    result = subprocess.run(
        ["python", "tests/run_block_editor_roundtrip_check.py"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0:
        return fail("E_BLOCK_EDITOR_ROUNDTRIP_REFRESH_CHECKER", result.stdout.strip())
    if "[block-editor-roundtrip] ok packs=2" not in result.stdout:
        return fail("E_BLOCK_EDITOR_ROUNDTRIP_REFRESH_CHECKER_STDOUT", result.stdout.strip())
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
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
        return fail("E_BLOCK_EDITOR_ROUNDTRIP_REFRESH_QUEUE", str(missing))
    if "1. `BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1`" in text:
        return fail(
            "E_BLOCK_EDITOR_ROUNDTRIP_REFRESH_STILL_OPEN",
            "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1 is still listed as next open item",
        )
    if "1. `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1`" in text:
        return fail(
            "E_BLOCK_EDITOR_ROUNDTRIP_REFRESH_NEXT",
            "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1 is still listed as next open item",
        )
    if "1. `ROADMAP_V2_LA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_BLOCK_EDITOR_ROUNDTRIP_REFRESH_FINAL_NEXT",
            "ROADMAP_V2_LA2_FINAL_CLOSURE_V1 is still listed as next open item",
        )
    if "1. `ROOT_LOW_RISK_RETIRE_DELETE_V1`" not in text:
        return fail(
            "E_BLOCK_EDITOR_ROUNDTRIP_REFRESH_A1_NEXT",
            "ROOT_LOW_RISK_RETIRE_DELETE_V1 is not next open item",
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
        return fail("E_BLOCK_EDITOR_ROUNDTRIP_REFRESH_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_BLOCK_EDITOR_ROUNDTRIP_REFRESH_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc,
        check_expected_supported,
        check_expected_raw,
        run_roundtrip_checker,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[block-editor-roundtrip-expected-refresh-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
