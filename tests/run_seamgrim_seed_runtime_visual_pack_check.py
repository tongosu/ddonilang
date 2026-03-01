#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def _safe_print(message: str) -> None:
    text = str(message or "").replace("\ufffd", "?")
    try:
        print(text)
    except UnicodeEncodeError:
        enc = getattr(sys.stdout, "encoding", None) or "utf-8"
        sys.stdout.write(text.encode(enc, errors="replace").decode(enc, errors="replace") + "\n")


def fail(detail: str) -> int:
    _safe_print(f"check=seed_runtime_visual_pack detail={detail}")
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    runner = root / "tests" / "seamgrim_seed_runtime_visual_pack_runner.mjs"
    if not runner.exists():
        return fail(f"runner_missing:{runner.as_posix()}")

    proc = subprocess.run(
        ["node", str(runner)],
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or f"returncode={proc.returncode}"
        return fail(f"runner_failed:{detail}")

    _safe_print((proc.stdout or "").strip() or "seamgrim seed runtime visual pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
