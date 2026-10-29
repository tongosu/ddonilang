#!/usr/bin/env python3
"""Validate A3 NuriGym Python/Web parity by product evidence, not stale matrix tokens."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "nurigym_python_web_parity_v1"
CONTRACT = PACK / "contract.detjson"
PARITY = PACK / "parity.detjson"
CASES = PACK / "cases.detjson"
WEB_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "nurigym_python_web_parity.js"
WEB_RUNNER = ROOT / "tests" / "nurigym_python_web_parity_runner.mjs"


def fail(message: str) -> None:
    print(f"[roadmap-v2-a3-nurigym-python-web-parity] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8-sig")


def read_json(path: Path) -> dict:
    try:
        payload = json.loads(read(path))
    except FileNotFoundError:
        fail(f"missing file: {path.relative_to(ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be JSON object")
    return payload


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def require_tokens(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing {missing}")


def run(args: list[str], *, timeout: float | None = None) -> subprocess.CompletedProcess[str]:
    normalized = [sys.executable if item == "python" else item for item in args]
    proc = subprocess.run(
        normalized,
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


def check_required_files() -> None:
    for path in [
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        PARITY,
        CASES,
        WEB_MODULE,
        WEB_RUNNER,
        ROOT / "pack" / "nuri_gym_gridmaze_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_bandit_v1" / "golden.jsonl",
    ]:
        require_file(path)
    require_tokens(WEB_MODULE, ["simulateGridmaze", "simulateBandit", "compareNuriGymDatasetParity"])


def check_contract_and_payload() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "nurigym_python_web_parity_v1",
        "kind": "roadmap_v2_a3_nurigym_python_web_parity",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": False,
        "roadmap_coordinate": "아-3",
        "cases_manifest": "pack/nurigym_python_web_parity_v1/cases.detjson",
        "web_module": "solutions/seamgrim_ui_mvp/ui/nurigym_python_web_parity.js",
        "browser_compatible_runner": "tests/nurigym_python_web_parity_runner.mjs",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    if contract.get("source_packs") != ["nuri_gym_gridmaze_v1", "nuri_gym_bandit_v1"]:
        fail(f"source_packs mismatch: {contract.get('source_packs')!r}")
    for key, value in contract.get("false_claims", {}).items():
        if value is not False:
            fail(f"contract false claim {key}={value!r}")

    parity = read_json(PARITY)
    if parity.get("schema") != "ddn.nurigym.python_web_parity.v1":
        fail("parity schema mismatch")
    if parity.get("status") != "bounded_python_web_parity_ready":
        fail("parity status mismatch")
    if parity.get("coordinate") != "아-3":
        fail("parity coordinate mismatch")
    case_ids = [row.get("id") for row in parity.get("cases", [])]
    if case_ids != ["gridmaze_layout_episode", "bandit_preferred_action_episode"]:
        fail(f"parity cases mismatch: {case_ids!r}")
    for key, value in parity.get("false_claims", {}).items():
        if value is not False:
            fail(f"parity false claim {key}={value!r}")


def check_cases_manifest() -> None:
    cases = read_json(CASES)
    if cases.get("schema") != "ddn.nurigym.python_web_parity.cases.v1":
        fail("cases schema mismatch")
    rows = cases.get("cases")
    if not isinstance(rows, list) or len(rows) != 2:
        fail("cases must contain exactly two rows")
    for row in rows:
        require_file(ROOT / row["input"])
        require_file(ROOT / row["dataset"])


def check_forbidden_claims() -> None:
    forbidden = [
        '"cartpole_pendulum_physics_parity_claim": true',
        '"full_browser_runtime_claim": true',
        '"public_registry_publish_claim": true',
        '"training_workflow_claim": true',
        '"nurigym_runtime_change": true',
        '"runtime_claim": true',
        '"product_ui_change": true',
    ]
    for path in [PACK / "README.md", CONTRACT, PARITY]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_web_runner_report() -> None:
    proc = run(["node", "tests/nurigym_python_web_parity_runner.mjs"], timeout=120)
    try:
        report = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        fail(f"web runner emitted invalid JSON: {exc}")
    if report.get("schema") != "ddn.nurigym.python_web_parity.report.v1":
        fail("web runner report schema mismatch")
    if report.get("ok") is not True:
        fail(f"web runner ok false: {report!r}")
    rows = report.get("cases")
    if not isinstance(rows, list) or len(rows) != 2:
        fail("web runner case count mismatch")
    expected_counts = {
        "gridmaze_layout_episode": (6, 6),
        "bandit_preferred_action_episode": (4, 4),
    }
    for row in rows:
        expected = expected_counts.get(row.get("id"))
        if expected is None:
            fail(f"unexpected web runner case: {row.get('id')!r}")
        if (row.get("dataset_count"), row.get("web_count")) != expected:
            fail(f"{row.get('id')} count mismatch: {row!r}")
        if row.get("failures") != []:
            fail(f"{row.get('id')} failures: {row.get('failures')!r}")


def check_gates() -> None:
    run(["python", "tests/run_pack_golden.py", "nurigym_python_web_parity_v1", "nuri_gym_gridmaze_v1", "nuri_gym_bandit_v1"], timeout=240)
    check_web_runner_report()
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_contract_and_payload()
    check_forbidden_claims()
    check_gates()
    check_cases_manifest()
    print("[roadmap-v2-a3-nurigym-python-web-parity] OK")


if __name__ == "__main__":
    main()
