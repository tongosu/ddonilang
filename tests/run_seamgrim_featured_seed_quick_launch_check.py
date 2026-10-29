#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    cmd = ["node", "tests/seamgrim_featured_seed_quick_launch_runner.mjs"]
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    stdout = (proc.stdout or "").strip()
    stderr = (proc.stderr or "").strip()
    if stdout:
        print(stdout)
    if stderr:
        print(stderr)

    payload = {
        "schema": "seamgrim.featured_seed_quick_launch_check.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": proc.returncode == 0,
        "returncode": int(proc.returncode),
        "cmd": cmd,
        "stdout": stdout,
        "stderr": stderr,
    }
    report = root / "build" / "reports" / "seamgrim_featured_seed_quick_launch_report.detjson"
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"[featured-seed-quick-launch] report={report}")

    if proc.returncode != 0:
        print("seamgrim featured seed quick launch check failed")
        return proc.returncode
    print("seamgrim featured seed quick launch check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
