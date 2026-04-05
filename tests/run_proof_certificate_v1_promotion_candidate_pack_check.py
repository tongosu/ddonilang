from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "proof_certificate_v1_promotion_candidate_v1"


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
    promotion_candidate = run_selftest("tests/run_proof_certificate_v1_promotion_candidate_selftest.py")
    if promotion_candidate.returncode != 0:
        print(promotion_candidate.stdout, end="")
        print(promotion_candidate.stderr, end="")
        raise SystemExit(promotion_candidate.returncode)
    assert read_text(PACK / "expected" / "promotion_candidate.stdout.txt") == promotion_candidate.stdout.replace(
        "\r\n", "\n"
    ).rstrip("\n")

    print("proof_certificate_v1_promotion_candidate_pack: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
