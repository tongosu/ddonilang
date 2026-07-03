#!/usr/bin/env python3
"""Validate BA1_FREE_LAB_FIRST_RUN_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "free_lab_1_v1"
CONTRACT = PACK / "contract.detjson"
FIRST_RUN = PACK / "first_run.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "free_lab_first_run.js"
UI_RUNNER = ROOT / "tests" / "free_lab_first_run_runner.mjs"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ba1-free-lab-first-run] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    try:
        payload = json.loads(read(path))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be a JSON object")
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


def check_files_and_docs() -> None:
    for path in [
        UI_MODULE,
        UI_RUNNER,
        PACK / "README.md",
        CONTRACT,
        FIRST_RUN,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
    ]:
        require_file(path)
    require_tokens(UI_MODULE, ["BA1_FREE_LAB_FIRST_RUN_V1", "buildFreeLabFirstRun", "free_lab.first_run.v1"])
    require_tokens(UI_RUNNER, ["free_lab_first_run: ok", "자유 실험 첫실행", "data-free-lab-first-run"])


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 8,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 9,
        "roadmap_v2_pack_evidence_reference_closed": 27,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 30,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
    }
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "free_lab_1_v1",
        "kind": "roadmap_v2_ba1_free_lab_first_run_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "BA1_FREE_LAB_FIRST_RUN_V1",
        "roadmap_coordinate": "바-1",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "ba1_matrix_status": "닫힘-동작",
        "requires_ba0_seed": True,
        "requires_browser_runner_evidence": True,
        "first_run_claim": True,
        "current_stage": "BA1 free lab first run",
        "next_item": "BA2_FREE_LAB_EXPERIMENT_REPORT_PACK_CLOSURE_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    if contract.get("first_run_lanes") != ["new_experiment", "parameter_setup", "recording_boundary"]:
        fail(f"contract first_run_lanes={contract.get('first_run_lanes')!r}")
    check_payload(CONTRACT)

    first_run = read_json(FIRST_RUN)
    if first_run.get("status") != "first_run_ready":
        fail(f"first_run status={first_run.get('status')!r}")
    if first_run.get("matrix_closure_tier") != "닫힘-동작":
        fail("first_run must be 닫힘-동작")
    if first_run.get("first_run_claim") is not True:
        fail("first_run must claim first-run")
    lane_ids = [lane.get("id") for lane in first_run.get("lanes", [])]
    if lane_ids != ["new_experiment", "parameter_setup", "recording_boundary"]:
        fail(f"first_run lanes mismatch: {lane_ids!r}")
    if not all(lane.get("ready") is True for lane in first_run.get("lanes", [])):
        fail("all first_run lanes must be ready")
    check_payload(FIRST_RUN)

    for payload in [contract, first_run]:
      false_claims = payload.get("false_claims", {})
      for key, value in false_claims.items():
          if value is not False:
              fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [CONTRACT, FIRST_RUN, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 9",
            "Studio-local 초장기 계획: 10/18",
            "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1 PASS 필요",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "free_lab_1_v1"], timeout=240)
    run(["node", "tests/free_lab_first_run_runner.mjs"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ba1-free-lab-first-run] OK")


if __name__ == "__main__":
    main()
