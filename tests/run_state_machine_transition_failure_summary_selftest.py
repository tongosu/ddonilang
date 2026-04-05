#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    text = f"[state-transition-failure-summary-selftest] fail: {msg}"
    try:
        print(text)
    except UnicodeEncodeError:
        sys.stdout.buffer.write(text.encode("utf-8", errors="replace") + b"\n")
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    script = root / "tests" / "run_state_machine_transition_failure_summary_check.py"
    with tempfile.TemporaryDirectory(prefix="state-transition-failure-summary-") as tmp:
        out = Path(tmp) / "summary.detjson"
        proc = subprocess.run(
            [sys.executable, str(script), "--json-out", str(out)],
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if proc.returncode != 0:
            return fail(f"script_failed stdout={proc.stdout} stderr={proc.stderr}")
        if not out.exists():
            return fail("summary json missing")
        doc = json.loads(out.read_text(encoding="utf-8"))
        if doc.get("schema") != "ddn.state_transition_failure_summary.v1":
            return fail(f"schema mismatch: {doc.get('schema')}")
        if doc.get("category_counts") != {"check": 2, "guard": 2, "action": 3}:
            return fail(f"category counts mismatch: {doc.get('category_counts')}")
        expected_kinds = {
            "check_failed",
            "check_unresolved",
            "guard_rejected",
            "guard_unresolved",
            "action_unresolved",
            "action_aborted",
            "action_arg_unresolved",
        }
        kinds = set((doc.get("kind_counts") or {}).keys())
        if kinds != expected_kinds:
            return fail(f"kind coverage mismatch: {sorted(kinds)}")
        packs = doc.get("packs")
        if not isinstance(packs, list) or len(packs) != 7:
            return fail(f"pack summary length mismatch: {packs}")

    print("[state-transition-failure-summary-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
