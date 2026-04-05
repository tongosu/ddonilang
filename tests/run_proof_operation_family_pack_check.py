from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "proof_operation_family_v1"


def run_selftest(script: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, script],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8").replace("\r\n", "\n").rstrip("\n")


def main() -> int:
    proc = run_selftest("tests/run_proof_operation_family_selftest.py")
    if proc.returncode != 0:
        print(proc.stdout, end="")
        print(proc.stderr, end="")
        raise SystemExit(proc.returncode)
    assert read_text(PACK / "expected" / "proof_operation_family.stdout.txt") == proc.stdout.replace(
        "\r\n", "\n"
    ).rstrip("\n")
    print("proof_operation_family_pack: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
