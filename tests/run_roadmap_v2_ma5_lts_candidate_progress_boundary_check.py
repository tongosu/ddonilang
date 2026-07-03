#!/usr/bin/env python3
"""Validate current progress boundaries for MA5 LTS candidate evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACKS = [
    "studio_education_operations_lts_v1",
    "studio_benchmark_lts_matrix_v1",
    "studio_ma3_regression_gate_matrix_v1",
    "studio_local_release_rehearsal_check_v1",
]
CURRENT_PROGRESS = {
    "super_long_behavior_closed": 8,
    "super_long_total": 18,
    "super_long_percent": 44,
    "current_stage_closed": 4,
    "current_stage_total": 4,
    "current_stage_percent": 100,
    "roadmap_v2_behavior_closed": 6,
    "roadmap_v2_total": 90,
    "roadmap_v2_percent": 7,
    "roadmap_v2_matrix_behavior_closed": 6,
    "roadmap_v2_matrix_behavior_total": 90,
    "roadmap_v2_matrix_behavior_percent": 7,
    "roadmap_v2_pack_evidence_reference_closed": 25,
    "roadmap_v2_pack_evidence_reference_total": 90,
    "roadmap_v2_pack_evidence_reference_percent": 28,
}
FORBIDDEN_TEXT = ["18/18 = 100%", "90/90 = 100%"]
FALSE_CLAIMS = [
    "lts_certification_claim",
    "benchmark_execution_claim",
    "performance_baseline_claim",
    "performance_baseline_generation_claim",
    "performance_baseline_publication_claim",
    "release_approval_claim",
    "release_execution_claim",
    "public_release_claim",
    "github_release_claim",
    "public_upload_claim",
    "registry_publish_claim",
    "cloud_sync_claim",
    "account_setup_claim",
    "permission_system_claim",
]


def fail(message: str) -> None:
    print(f"[ma5-lts-candidate-progress-boundary] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing file: {path.relative_to(ROOT)}")


def load_json(path: Path) -> dict:
    try:
        payload = json.loads(read(path))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be a JSON object")
    return payload


def run(args: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )
    if proc.returncode != 0:
        print(proc.stdout, end="")
        fail(f"command failed: {' '.join(args)}")
    return proc


def require_current_progress(payload: dict, label: str) -> None:
    progress = payload.get("progress", payload)
    for key, expected in CURRENT_PROGRESS.items():
        if progress.get(key) != expected:
            fail(f"{label} {key}={progress.get(key)!r}")


def require_false_claims(payload: dict, label: str) -> None:
    for key in FALSE_CLAIMS:
        if key in payload and payload.get(key) is not False:
            fail(f"{label} {key}={payload.get(key)!r}")


def check_pack(pack: str) -> None:
    pack_dir = ROOT / "pack" / pack
    contract = load_json(pack_dir / "contract.detjson")
    require_current_progress(contract, f"{pack}/contract")
    require_false_claims(contract, f"{pack}/contract")

    for path in pack_dir.iterdir():
        if path.suffix not in {".detjson", ".jsonl", ".md", ".ddn"}:
            continue
        text = read(path)
        present = [token for token in FORBIDDEN_TEXT if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden progress claim: {present}")
        if path.suffix == ".detjson":
            payload = load_json(path)
            require_false_claims(payload, path.relative_to(ROOT).as_posix())
            if path.name != "contract.detjson" and "progress" in payload:
                require_current_progress(payload, path.relative_to(ROOT).as_posix())


def check_product_runners() -> None:
    run(["python", "tests/run_pack_golden.py", *PACKS], timeout=300)
    run(["node", "tests/studio_ma3_regression_gate_matrix_runner.mjs"], timeout=240)
    run(["node", "tests/studio_local_release_rehearsal_check_runner.mjs"], timeout=240)


def check_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    for pack in PACKS:
        check_pack(pack)
    check_product_runners()
    check_docs_ssot_clean()
    print("[ma5-lts-candidate-progress-boundary] OK")


if __name__ == "__main__":
    main()
