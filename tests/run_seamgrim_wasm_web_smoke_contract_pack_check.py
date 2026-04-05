from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wasm_web_smoke_contract_v1"
REAL_SMOKE_CMD = [
    sys.executable,
    "-S",
    "tests/run_seamgrim_wasm_smoke.py",
    "seamgrim_wasm_v0_smoke",
    "seamgrim_interactive_event_smoke_v1",
    "seamgrim_temp_lesson_smoke_v1",
    "seamgrim_moyang_render_smoke_v1",
    "--skip-ui-common",
    "--skip-ui-pendulum",
    "--skip-wrapper",
    "--skip-vm-runtime",
    "--skip-space2d-source-gate",
]


def run_selftest(script: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, "-S", script],
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
    proc = run_selftest("tests/run_seamgrim_wasm_web_smoke_contract_selftest.py")
    if proc.returncode != 0:
        print(proc.stdout, end="")
        print(proc.stderr, end="")
        raise SystemExit(proc.returncode)
    assert read_text(PACK / "expected" / "seamgrim_wasm_web_smoke_contract.stdout.txt") == proc.stdout.replace(
        "\r\n", "\n"
    ).rstrip("\n")

    smoke_proc = subprocess.run(
        REAL_SMOKE_CMD,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if smoke_proc.returncode != 0:
        print(smoke_proc.stdout, end="")
        print(smoke_proc.stderr, end="")
        raise SystemExit(smoke_proc.returncode)
    assert read_text(PACK / "expected" / "seamgrim_wasm_web_real_smoke.stdout.txt") == smoke_proc.stdout.replace(
        "\r\n", "\n"
    ).rstrip("\n")
    print("seamgrim_wasm_web_smoke_contract_pack: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
