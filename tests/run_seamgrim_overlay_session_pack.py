#!/usr/bin/env python
from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path


def ascii_safe(text: str) -> str:
    return text.encode("ascii", errors="backslashreplace").decode("ascii")


def default_report_path(file_name: str) -> str:
    preferred = Path("I:/home/urihanl/ddn/codex/build/reports")
    if os.name == "nt":
        try:
            preferred.mkdir(parents=True, exist_ok=True)
        except OSError:
            pass
        return str(preferred / file_name)
    return f"build/reports/{file_name}"


def main() -> int:
    parser = argparse.ArgumentParser(description="Run Seamgrim overlay session roundtrip pack checks")
    parser.add_argument(
        "--pack-root",
        default="pack/seamgrim_overlay_session_roundtrip_v0",
        help="overlay session roundtrip pack root",
    )
    parser.add_argument(
        "--json-out",
        default=default_report_path("seamgrim_overlay_session_pack_report.detjson"),
        help="optional report output path",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    pack_root = Path(args.pack_root)
    if not pack_root.is_absolute():
        pack_root = (root / pack_root).resolve()
    pack_base = (root / "pack").resolve()
    use_pack_golden = False
    pack_name = ""
    try:
        pack_name = pack_root.relative_to(pack_base).as_posix()
        use_pack_golden = bool(pack_name)
    except Exception:
        use_pack_golden = False

    if use_pack_golden:
        cmd = [
            sys.executable,
            "tests/run_pack_golden.py",
            pack_name,
            "--report-out",
            str(args.json_out),
        ]
    else:
        node_runner = root / "tests" / "seamgrim_overlay_session_pack_runner.mjs"
        cmd = [
            "node",
            str(node_runner),
            "--pack-root",
            str(pack_root),
            "--json-out",
            str(args.json_out),
        ]
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.stdout:
        print(ascii_safe(proc.stdout.strip()))
    if proc.stderr:
        print(ascii_safe(proc.stderr.strip()))
    if proc.returncode != 0:
        print("overlay session pack check failed")
        return proc.returncode
    print("overlay session pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
