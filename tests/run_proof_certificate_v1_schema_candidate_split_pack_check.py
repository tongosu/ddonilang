from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "proof_certificate_v1_schema_candidate_split_v1"


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
    schema_split = run_selftest("tests/run_proof_certificate_v1_schema_candidate_split_selftest.py")
    if schema_split.returncode != 0:
        print(schema_split.stdout, end="")
        print(schema_split.stderr, end="")
        raise SystemExit(schema_split.returncode)
    assert read_text(PACK / "expected" / "schema_candidate_split.stdout.txt") == schema_split.stdout.replace(
        "\r\n", "\n"
    ).rstrip("\n")

    print("proof_certificate_v1_schema_candidate_split_pack: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
