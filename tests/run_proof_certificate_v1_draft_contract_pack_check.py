from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "proof_certificate_v1_draft_contract_v1"


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
    draft_contract = run_selftest("tests/run_proof_certificate_v1_draft_contract_selftest.py")
    if draft_contract.returncode != 0:
        print(draft_contract.stdout, end="")
        print(draft_contract.stderr, end="")
        raise SystemExit(draft_contract.returncode)
    assert read_text(PACK / "expected" / "draft_contract.stdout.txt") == draft_contract.stdout.replace(
        "\r\n", "\n"
    ).rstrip("\n")

    print("proof_certificate_v1_draft_contract_pack: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
