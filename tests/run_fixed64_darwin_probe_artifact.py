#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import platform
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCHEMA = "ddn.fixed64.darwin_probe_artifact.v1"
PROBE_SCHEMA = "ddn.fixed64.cross_platform_probe.v1"
REPORT_NAME = "fixed64_cross_platform_probe_darwin.detjson"


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def main() -> int:
    parser = argparse.ArgumentParser(description="macOS에서 fixed64 darwin probe 아티팩트를 생성한다.")
    parser.add_argument("--python", default=sys.executable, help="python executable")
    parser.add_argument("--report-dir", default="build/reports", help="report root directory")
    parser.add_argument(
        "--out-dir",
        default="",
        help="darwin report output directory (default: <report-dir>/darwin_probe)",
    )
    parser.add_argument(
        "--json-out",
        default="",
        help="summary detjson path (default: <report-dir>/fixed64_darwin_probe_artifact.detjson)",
    )
    parser.add_argument("--require-darwin", action="store_true", help="darwin host가 아니면 실패")
    args = parser.parse_args()

    report_dir = Path(args.report_dir).resolve()
    report_dir.mkdir(parents=True, exist_ok=True)
    out_dir = Path(args.out_dir).resolve() if args.out_dir.strip() else report_dir / "darwin_probe"
    out_dir.mkdir(parents=True, exist_ok=True)
    summary_path = (
        Path(args.json_out).resolve()
        if args.json_out.strip()
        else report_dir / "fixed64_darwin_probe_artifact.detjson"
    )
    probe_path = out_dir / REPORT_NAME

    host = platform.system().strip().lower()
    payload: dict[str, object] = {
        "schema": SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": False,
        "status": "fail",
        "reason": "-",
        "host_system": host,
        "probe_report": str(probe_path),
        "summary_report": str(summary_path),
        "cmd": [],
        "returncode": -1,
    }

    if host != "darwin":
        if args.require_darwin:
            payload["reason"] = "host is not darwin"
            payload["status"] = "fail_non_darwin"
            write_json(summary_path, payload)
            print("[fixed64-darwin-probe] non-darwin host", file=sys.stderr)
            return 1
        payload["ok"] = True
        payload["status"] = "skip_non_darwin"
        payload["reason"] = "host is not darwin"
        write_json(summary_path, payload)
        print(f"[fixed64-darwin-probe] skip host={host} summary={summary_path}")
        return 0

    cmd = [
        args.python,
        "tests/run_fixed64_cross_platform_probe.py",
        "--report-out",
        str(probe_path),
    ]
    payload["cmd"] = cmd
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    payload["returncode"] = int(proc.returncode)
    payload["stdout"] = (proc.stdout or "").strip().splitlines()
    payload["stderr"] = (proc.stderr or "").strip().splitlines()
    if proc.returncode != 0:
        payload["reason"] = "probe command failed"
        write_json(summary_path, payload)
        print("[fixed64-darwin-probe] probe command failed", file=sys.stderr)
        return int(proc.returncode)

    probe_doc = load_json(probe_path)
    if not isinstance(probe_doc, dict):
        payload["reason"] = "probe report invalid"
        write_json(summary_path, payload)
        print("[fixed64-darwin-probe] invalid probe report", file=sys.stderr)
        return 1
    if str(probe_doc.get("schema", "")) != PROBE_SCHEMA:
        payload["reason"] = "probe schema mismatch"
        write_json(summary_path, payload)
        print("[fixed64-darwin-probe] probe schema mismatch", file=sys.stderr)
        return 1
    if not bool(probe_doc.get("ok", False)):
        payload["reason"] = "probe ok=false"
        write_json(summary_path, payload)
        print("[fixed64-darwin-probe] probe ok=false", file=sys.stderr)
        return 1
    platform_doc = probe_doc.get("platform")
    system = str(platform_doc.get("system", "")).strip().lower() if isinstance(platform_doc, dict) else ""
    if system != "darwin":
        payload["reason"] = f"probe platform mismatch: {system or '-'}"
        write_json(summary_path, payload)
        print("[fixed64-darwin-probe] probe platform mismatch", file=sys.stderr)
        return 1

    payload["ok"] = True
    payload["status"] = "staged"
    payload["reason"] = "-"
    write_json(summary_path, payload)
    print(f"[fixed64-darwin-probe] ok report={probe_path} summary={summary_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
