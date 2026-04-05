#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
PACKS = (
    "block_editor_roundtrip_v1",
    "block_editor_raw_fallback_v1",
)


def fail(message: str) -> int:
    print(f"[block-editor-roundtrip] fail: {message}", file=sys.stderr)
    return 1


def run_pack(pack_name: str, *, update: bool) -> None:
    cmd = ["node", "--no-warnings", "tests/block_editor_roundtrip_runner.mjs", f"pack/{pack_name}"]
    if update:
        cmd.append("--update")
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or "runner failed"
        raise ValueError(f"{pack_name}: {detail}")


def validate_expected(pack_name: str) -> None:
    expected_path = ROOT / "pack" / pack_name / "expected" / "block_editor_roundtrip.detjson"
    payload = json.loads(expected_path.read_text(encoding="utf-8"))
    if not payload.get("canon_equal"):
        raise ValueError(f"{pack_name}: canon_equal must be true")
    if str(payload.get("block_plan_schema", "")).strip() != "ddn.block_editor_plan.v1":
        raise ValueError(f"{pack_name}: block_plan_schema mismatch")
    if payload.get("decode_errors"):
        raise ValueError(f"{pack_name}: decode_errors must be empty")
    if pack_name == "block_editor_roundtrip_v1":
        if int(payload.get("raw_block_count", -1)) != 0:
            raise ValueError(f"{pack_name}: raw_block_count must be 0")
    if pack_name == "block_editor_raw_fallback_v1":
        if int(payload.get("raw_block_count", 0)) < 1:
            raise ValueError(f"{pack_name}: raw_block_count must be >= 1")


def main() -> int:
    parser = argparse.ArgumentParser(description="block editor roundtrip/raw fallback 검증")
    parser.add_argument("--update", action="store_true")
    args = parser.parse_args()
    try:
        for pack_name in PACKS:
            run_pack(pack_name, update=args.update)
        for pack_name in PACKS:
            validate_expected(pack_name)
    except ValueError as exc:
        return fail(str(exc))

    print("[block-editor-roundtrip] ok packs=2")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
