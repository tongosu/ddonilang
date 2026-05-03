#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from _teul_cli_freshness import ensure_teul_cli_bin as shared_ensure_teul_cli_bin


def fail(msg: str) -> int:
    print(f"[contract-tier-age3-min-enforcement-check] fail: {msg}")
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


def write_case(
    tmp_dir: Path,
    *,
    age_target: str,
    det_tier: str,
    detmath_seal_hash: str | None = None,
    nuri_lock_hash: str | None = None,
) -> Path:
    input_path = tmp_dir / "input.ddn"
    project_path = tmp_dir / "ddn.project.json"
    input_path.write_text("x <- 1.\n", encoding="utf-8")
    doc = {
        "name": "contract_tier_age3_min_enforcement_check",
        "age_target": age_target,
        "det_tier": det_tier,
        "trace_tier": "T-OFF",
        "openness": "closed",
        "deps": [],
    }
    if detmath_seal_hash is not None:
        doc["detmath_seal_hash"] = detmath_seal_hash
    if nuri_lock_hash is not None:
        doc["nuri_lock_hash"] = nuri_lock_hash
    project_path.write_text(
        json.dumps(doc, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    return input_path


def merged_output(proc: subprocess.CompletedProcess[str]) -> str:
    return ((proc.stdout or "") + "\n" + (proc.stderr or "")).strip()


def expect_fail_with_code(
    root: Path,
    teul_cli_bin: Path,
    *,
    age_target: str,
    det_tier: str,
    expected_code: str,
    detmath_seal_hash: str | None = None,
    nuri_lock_hash: str | None = None,
) -> int:
    with tempfile.TemporaryDirectory(prefix="contract_tier_age3_min_fail_") as td:
        input_path = write_case(
            Path(td),
            age_target=age_target,
            det_tier=det_tier,
            detmath_seal_hash=detmath_seal_hash,
            nuri_lock_hash=nuri_lock_hash,
        )
        proc = run_teul_cli(root, teul_cli_bin, input_path)
    output = merged_output(proc)
    if proc.returncode == 0:
        return fail(f"{age_target}/{det_tier} must fail but passed")
    if expected_code not in output:
        return fail(
            f"{age_target}/{det_tier} output missing {expected_code}: {output}"
        )
    return 0


def expect_pass(
    root: Path,
    teul_cli_bin: Path,
    *,
    age_target: str,
    det_tier: str,
    detmath_seal_hash: str | None = None,
    nuri_lock_hash: str | None = None,
) -> int:
    with tempfile.TemporaryDirectory(prefix="contract_tier_age3_min_pass_") as td:
        input_path = write_case(
            Path(td),
            age_target=age_target,
            det_tier=det_tier,
            detmath_seal_hash=detmath_seal_hash,
            nuri_lock_hash=nuri_lock_hash,
        )
        proc = run_teul_cli(root, teul_cli_bin, input_path)
    output = merged_output(proc)
    if proc.returncode != 0:
        return fail(f"{age_target}/{det_tier} must pass: {output}")
    return 0


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    teul_cli_bin = ensure_teul_cli_bin(root)
    if teul_cli_bin is None:
        return fail("teul-cli build failed")

    rc = expect_fail_with_code(
        root,
        teul_cli_bin,
        age_target="AGE2",
        det_tier="D-SEALED",
        expected_code="E_CONTRACT_TIER_UNSUPPORTED",
    )
    if rc != 0:
        return rc

    rc = expect_fail_with_code(
        root,
        teul_cli_bin,
        age_target="AGE3",
        det_tier="D-SEALED",
        expected_code="E_RUNTIME_CONTRACT_MISMATCH",
    )
    if rc != 0:
        return rc

    rc = expect_pass(
        root,
        teul_cli_bin,
        age_target="AGE3",
        det_tier="D-SEALED",
        detmath_seal_hash="sha256:seal-a",
        nuri_lock_hash="sha256:lock-a",
    )
    if rc != 0:
        return rc

    rc = expect_pass(
        root,
        teul_cli_bin,
        age_target="AGE3",
        det_tier="D-APPROX",
    )
    if rc != 0:
        return rc

    rc = expect_fail_with_code(
        root,
        teul_cli_bin,
        age_target="AGE2",
        det_tier="D-APPROX",
        expected_code="E_CONTRACT_TIER_UNSUPPORTED",
    )
    if rc != 0:
        return rc

    print("contract tier age3 min enforcement check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
