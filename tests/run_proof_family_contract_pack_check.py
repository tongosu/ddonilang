from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from datetime import datetime
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "proof_family_contract_v1"


def run_selftest(script: str, *, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, script],
        cwd=ROOT,
        env=env,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8").replace("\r\n", "\n").rstrip("\n")


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def validate_progress(path: Path) -> None:
    actual = read_json(path)
    expected = read_json(PACK / "expected" / "proof_family_contract.progress.detjson")
    generated = str(actual.get("generated_at_utc", "")).strip()
    assert generated
    parsed = datetime.fromisoformat(generated)
    assert parsed.tzinfo is not None
    actual["generated_at_utc"] = expected["generated_at_utc"]
    assert actual == expected


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="proof_family_contract_pack_") as td:
        progress = Path(td) / "proof_family_contract.progress.detjson"
        env = dict(os.environ)
        env["DDN_PROOF_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON"] = str(progress)

        contract = run_selftest("tests/run_proof_family_contract_selftest.py", env=env)
        if contract.returncode != 0:
            print(contract.stdout, end="")
            print(contract.stderr, end="")
            raise SystemExit(contract.returncode)
        assert read_text(PACK / "expected" / "proof_family_contract.stdout.txt") == contract.stdout.replace(
            "\r\n", "\n"
        ).rstrip("\n")
        validate_progress(progress)

        summary = run_selftest("tests/run_proof_family_contract_summary_selftest.py")
        if summary.returncode != 0:
            print(summary.stdout, end="")
            print(summary.stderr, end="")
            raise SystemExit(summary.returncode)
        assert read_text(PACK / "expected" / "proof_family_contract_summary.stdout.txt") == summary.stdout.replace(
            "\r\n", "\n"
        ).rstrip("\n")

    print("proof_family_contract_pack: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
