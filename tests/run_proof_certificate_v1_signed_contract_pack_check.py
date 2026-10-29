from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "proof_certificate_v1_signed_contract_v1"


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
    signed_contract = run_selftest("tests/run_proof_certificate_v1_signed_contract_selftest.py")
    if signed_contract.returncode != 0:
        print(signed_contract.stdout, end="")
        print(signed_contract.stderr, end="")
        raise SystemExit(signed_contract.returncode)
    assert read_text(PACK / "expected" / "signed_contract.stdout.txt") == signed_contract.stdout.replace(
        "\r\n", "\n"
    ).rstrip("\n")

    print("proof_certificate_v1_signed_contract_pack: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
