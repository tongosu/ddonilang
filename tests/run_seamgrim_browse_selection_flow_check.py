#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description="Run Seamgrim browse selection flow check")
    parser.add_argument("--json-out", default="", help="optional json report output path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    cmd = ["node", "tests/seamgrim_browse_selection_runner.mjs"]
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.stdout:
        print(proc.stdout.strip())
    if proc.stderr:
        print(proc.stderr.strip())
    payload = {
        "schema": "seamgrim.browse_selection_flow_check.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": proc.returncode == 0,
        "returncode": int(proc.returncode),
        "cmd": cmd,
        "stdout": (proc.stdout or "").strip(),
        "stderr": (proc.stderr or "").strip(),
    }
    if args.json_out:
        out_path = Path(args.json_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"[browse-selection-flow] report={out_path}")
    if proc.returncode != 0:
        print("seamgrim browse selection flow check failed")
        return proc.returncode
    print("seamgrim browse selection flow check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
