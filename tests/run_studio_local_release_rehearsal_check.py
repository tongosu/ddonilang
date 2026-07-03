from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def fail(message: str) -> None:
    print(f"studio_local_release_rehearsal_check: FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def run(cmd: list[str], timeout: int = 420) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def main() -> None:
    proc = run(["python", "tests/run_roadmap_v2_ma5_lts_candidate_progress_boundary_check.py"])
    if proc.returncode != 0:
        fail(f"MA5 LTS candidate progress boundary failed:\n{proc.stdout}")
    print("studio_local_release_rehearsal_check: ok")


if __name__ == "__main__":
    main()
