#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_editor_run_handoff_v1"


def _sort_json(value):
    if isinstance(value, list):
        return [_sort_json(item) for item in value]
    if isinstance(value, dict):
        return {key: _sort_json(value[key]) for key in sorted(value)}
    return value


def main() -> int:
    proc = subprocess.run(
        ["node", "--no-warnings", "tests/seamgrim_editor_run_handoff_runner.mjs"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=120,
    )
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip()
        print(f"[seamgrim-editor-run-handoff] fail: {detail}", file=sys.stderr)
        return 1
    actual = _sort_json(json.loads(proc.stdout))
    expected = _sort_json(json.loads((PACK / "expected" / "editor_run_handoff.detjson").read_text(encoding="utf-8")))
    if actual != expected:
        print("[seamgrim-editor-run-handoff] fail: expected mismatch", file=sys.stderr)
        print(json.dumps(actual, ensure_ascii=False, indent=2), file=sys.stderr)
        return 1
    rail = subprocess.run(
        [sys.executable, "tests/run_seamgrim_intro_exec_rail_check.py"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=240,
    )
    if rail.returncode != 0:
        detail = (rail.stderr or rail.stdout or "").strip()
        print(f"[seamgrim-editor-run-handoff] fail: intro rail regression failed: {detail}", file=sys.stderr)
        return 1
    print("[seamgrim-editor-run-handoff] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
