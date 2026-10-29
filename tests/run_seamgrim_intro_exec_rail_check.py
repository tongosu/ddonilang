#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_intro_exec_rail_v1"


def fail(message: str) -> int:
    print(f"[seamgrim-intro-exec-rail] fail: {message}", file=sys.stderr)
    return 1


def sort_json(value):
    if isinstance(value, list):
        return [sort_json(item) for item in value]
    if isinstance(value, dict):
        return {key: sort_json(value[key]) for key in sorted(value)}
    return value


def format_json(value) -> str:
    return json.dumps(sort_json(value), ensure_ascii=False, indent=2) + "\n"


def run_step(name: str, command: list[str]) -> dict:
    proc = subprocess.run(
        command,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=300,
    )
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip()
        raise RuntimeError(f"{name} failed: {detail}")
    return {
        "name": name,
        "command": command,
        "ok": True,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="ROADMAP_V2 라-1 intro exec rail aggregate checker")
    parser.add_argument("--update", action="store_true", help="expected report 갱신")
    args = parser.parse_args()
    try:
        steps = [
            run_step("text_ddn_wasm", [sys.executable, "tests/run_seamgrim_intro_exec_wasm_check.py"]),
            run_step("blocky_codegen_wasm", [sys.executable, "tests/run_seamgrim_intro_exec_blocky_check.py"]),
            run_step(
                "block_editor_screen_intro",
                [
                    "node",
                    "--no-warnings",
                    "tests/seamgrim_block_editor_runner.mjs",
                    "pack/block_editor_screen_intro_exec_v1",
                ],
            ),
        ]
        report = {
            "schema": "ddn.seamgrim_intro_exec_rail_report.v1",
            "status": "closed",
            "steps": steps,
        }
        expected_path = PACK / "expected" / "intro_exec_rail.detjson"
        actual_text = format_json(report)
        if args.update:
            expected_path.parent.mkdir(parents=True, exist_ok=True)
            expected_path.write_text(actual_text, encoding="utf-8")
            print(f"[seamgrim-intro-exec-rail] updated {expected_path.relative_to(ROOT)}")
            return 0
        expected_text = expected_path.read_text(encoding="utf-8")
        if expected_text != actual_text:
            raise RuntimeError(f"expected mismatch: {expected_path.relative_to(ROOT)}")
    except Exception as exc:
        return fail(str(exc))

    print(f"[seamgrim-intro-exec-rail] ok steps={len(steps)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

