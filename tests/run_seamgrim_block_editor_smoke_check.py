#!/usr/bin/env python
"""Block editor 화면 smoke 검증."""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import os
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


def run_pack(pack_dir: Path, update: bool) -> tuple[Path, int, str, str]:
    cmd = ["node", "--no-warnings", str(RUNNER), str(pack_dir)]
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
    return pack_dir, proc.returncode, (proc.stdout or "").strip(), (proc.stderr or "").strip()


def main() -> int:
    parser = argparse.ArgumentParser(description="block editor 화면 smoke 검증")
    parser.add_argument("--update", action="store_true", help="golden 파일 갱신")
    parser.add_argument(
        "--jobs",
        type=int,
        default=0,
        help="병렬 실행 worker 수(기본: min(3, pack 개수), 1이면 직렬)",
    )
    args = parser.parse_args()

    env_jobs_text = str(os.environ.get("DDN_BLOCK_EDITOR_SMOKE_JOBS", "")).strip()
    env_jobs = 0
    if env_jobs_text:
        try:
            env_jobs = int(env_jobs_text)
        except ValueError:
            env_jobs = 0
    requested_jobs = int(args.jobs) if int(args.jobs) > 0 else env_jobs
    default_jobs = 3
    max_workers = max(1, min(requested_jobs if requested_jobs > 0 else default_jobs, len(PACK_DIRS)))

    indexed_results: list[tuple[Path, int, str, str] | None] = [None] * len(PACK_DIRS)
    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        pending = {
            executor.submit(run_pack, pack_dir, bool(args.update)): idx
            for idx, pack_dir in enumerate(PACK_DIRS)
        }
        for future in as_completed(pending):
            idx = pending[future]
            indexed_results[idx] = future.result()

    has_failure = False
    for row in indexed_results:
        if row is None:
            continue
        pack_dir, returncode, stdout, stderr = row
        if returncode != 0:
            has_failure = True
            print(f"[FAIL] block editor screen smoke: {pack_dir.name}", file=sys.stderr)
            if stderr:
                print(stderr, file=sys.stderr)
            if stdout:
                print(stdout, file=sys.stderr)
            continue
        print(stdout or f"[ok] block editor screen smoke: {pack_dir.name}")

    return 1 if has_failure else 0


if __name__ == "__main__":
    raise SystemExit(main())
