#!/usr/bin/env python
"""Block editor 화면 smoke 검증."""
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
RUNNER = ROOT / "tests" / "seamgrim_block_editor_runner.mjs"
PACK_DIRS = [
    ROOT / "pack" / "block_editor_screen_rpg_smoke_v1",
    ROOT / "pack" / "block_editor_screen_seamgrim_smoke_v1",
    ROOT / "pack" / "block_editor_screen_seamgrim_flow_smoke_v1",
    ROOT / "pack" / "block_editor_screen_seamgrim_contract_smoke_v1",
    ROOT / "pack" / "block_editor_screen_seamgrim_prompt_smoke_v1",
    ROOT / "pack" / "block_editor_screen_seamgrim_runtime_smoke_v1",
    ROOT / "pack" / "block_editor_screen_seamgrim_view_smoke_v1",
    ROOT / "pack" / "block_editor_screen_seamgrim_structured_smoke_v1",
    ROOT / "pack" / "block_editor_screen_seamgrim_expr_smoke_v1",
    ROOT / "pack" / "block_editor_screen_seamgrim_expr_edit_smoke_v1",
    ROOT / "pack" / "block_editor_screen_seamgrim_expr_struct_edit_smoke_v1",
]


def main() -> int:
    parser = argparse.ArgumentParser(description="block editor 화면 smoke 검증")
    parser.add_argument("--update", action="store_true", help="golden 파일 갱신")
    args = parser.parse_args()

    for pack_dir in PACK_DIRS:
        cmd = ["node", "--no-warnings", str(RUNNER), str(pack_dir)]
        if args.update:
            cmd.append("--update")
        proc = subprocess.run(
            cmd,
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        stdout = (proc.stdout or "").strip()
        stderr = (proc.stderr or "").strip()
        if proc.returncode != 0:
            print(f"[FAIL] block editor screen smoke: {pack_dir.name}", file=sys.stderr)
            if stderr:
                print(stderr, file=sys.stderr)
            if stdout:
                print(stdout, file=sys.stderr)
            return 1
        print(stdout or f"[ok] block editor screen smoke: {pack_dir.name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
