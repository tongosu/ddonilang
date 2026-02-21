#!/usr/bin/env python
from __future__ import annotations

import json
import platform
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCHEMA = "ddn.fixed64.darwin_probe_artifact.v1"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def run_case(summary: Path, report_dir: Path, require_darwin: bool) -> int:
    cmd = [
        sys.executable,
        "tests/run_fixed64_darwin_probe_artifact.py",
        "--report-dir",
        str(report_dir),
        "--json-out",
        str(summary),
    ]
    if require_darwin:
        cmd.append("--require-darwin")
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return int(proc.returncode)


def main() -> int:
    host = platform.system().strip().lower()
    with tempfile.TemporaryDirectory(prefix="fixed64_darwin_probe_artifact_selftest_") as tmp:
        root = Path(tmp)
        report_dir = root / "reports"
        summary = report_dir / "darwin_probe_summary.detjson"

        rc_normal = run_case(summary, report_dir, require_darwin=False)
        if rc_normal != 0:
            print("[fixed64-darwin-probe-selftest] normal case failed", file=sys.stderr)
            return 1
        normal_doc = load_json(summary)
        if not isinstance(normal_doc, dict) or str(normal_doc.get("schema", "")) != SCHEMA:
            print("[fixed64-darwin-probe-selftest] normal summary invalid", file=sys.stderr)
            return 1
        normal_status = str(normal_doc.get("status", ""))
        if host == "darwin":
            if normal_status != "staged":
                print("[fixed64-darwin-probe-selftest] expected staged on darwin", file=sys.stderr)
                return 1
        else:
            if normal_status != "skip_non_darwin":
                print("[fixed64-darwin-probe-selftest] expected skip_non_darwin on non-darwin", file=sys.stderr)
                return 1

        rc_require = run_case(summary, report_dir, require_darwin=True)
        require_doc = load_json(summary)
        if not isinstance(require_doc, dict):
            print("[fixed64-darwin-probe-selftest] require summary invalid", file=sys.stderr)
            return 1
        require_status = str(require_doc.get("status", ""))
        if host == "darwin":
            if rc_require != 0 or require_status != "staged":
                print("[fixed64-darwin-probe-selftest] require case mismatch on darwin", file=sys.stderr)
                return 1
        else:
            if rc_require == 0 or require_status != "fail_non_darwin":
                print("[fixed64-darwin-probe-selftest] require case mismatch on non-darwin", file=sys.stderr)
                return 1

    print("[fixed64-darwin-probe-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
