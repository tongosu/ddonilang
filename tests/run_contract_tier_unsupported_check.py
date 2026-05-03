#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from _teul_cli_freshness import ensure_teul_cli_bin as shared_ensure_teul_cli_bin

EXPECTED_CODE = "E_CONTRACT_TIER_UNSUPPORTED"
SUPPORTED_TIER = "D-STRICT"
UNSUPPORTED_TIERS = ("D-SEALED", "D-APPROX")


def fail(msg: str) -> int:
    print(f"[contract-tier-unsupported-check] fail: {msg}")
    return 1


def teul_cli_candidates(root: Path) -> list[Path]:
    suffix = ".exe" if sys.platform.startswith("win") else ""
    return [
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        root / "target" / "debug" / f"teul-cli{suffix}",
    ]


def ensure_teul_cli_bin(root: Path) -> Path | None:
    try:
        return shared_ensure_teul_cli_bin(
            root,
            candidates=teul_cli_candidates(root),
            include_which=False,
            manifest_path=root / "tools" / "teul-cli" / "Cargo.toml",
        )
    except (SystemExit, FileNotFoundError):
        return None


def run_teul_cli(root: Path, teul_cli_bin: Path, input_path: Path) -> subprocess.CompletedProcess[str]:
    cmd = [str(teul_cli_bin), "run", str(input_path)]
    return subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def write_case(tmp_dir: Path, det_tier: str) -> Path:
    input_path = tmp_dir / "input.ddn"
    project_path = tmp_dir / "ddn.project.json"
    input_path.write_text("x <- 1.\n", encoding="utf-8")
    project_path.write_text(
        json.dumps(
            {
                "name": "contract_tier_unsupported_check",
                "age_target": "AGE2",
                "det_tier": det_tier,
                "trace_tier": "T-OFF",
                "openness": "closed",
                "deps": [],
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    return input_path


def merged_output(proc: subprocess.CompletedProcess[str]) -> str:
    return ((proc.stdout or "") + "\n" + (proc.stderr or "")).strip()


def run_supported_case(root: Path, teul_cli_bin: Path) -> int:
    with tempfile.TemporaryDirectory(prefix="contract_tier_supported_") as td:
        input_path = write_case(Path(td), SUPPORTED_TIER)
        proc = run_teul_cli(root, teul_cli_bin, input_path)
    output = merged_output(proc)
    if proc.returncode != 0:
        return fail(f"{SUPPORTED_TIER} must pass: {output}")
    if EXPECTED_CODE in output:
        return fail(f"{SUPPORTED_TIER} output must not include {EXPECTED_CODE}")
    return 0


def run_unsupported_case(root: Path, teul_cli_bin: Path, det_tier: str) -> int:
    with tempfile.TemporaryDirectory(prefix="contract_tier_unsupported_") as td:
        input_path = write_case(Path(td), det_tier)
        proc = run_teul_cli(root, teul_cli_bin, input_path)
    output = merged_output(proc)
    if proc.returncode == 0:
        return fail(f"{det_tier} must fail but passed")
    if EXPECTED_CODE not in output:
        return fail(f"{det_tier} output missing {EXPECTED_CODE}: {output}")
    if f"det_tier={det_tier}" not in output:
        return fail(f"{det_tier} output missing det_tier marker: {output}")
    return 0


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    teul_cli_bin = ensure_teul_cli_bin(root)
    if teul_cli_bin is None:
        return fail("teul-cli build failed")

    rc = run_supported_case(root, teul_cli_bin)
    if rc != 0:
        return rc

    for det_tier in UNSUPPORTED_TIERS:
        rc = run_unsupported_case(root, teul_cli_bin, det_tier)
        if rc != 0:
            return rc

    print("contract tier unsupported check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
