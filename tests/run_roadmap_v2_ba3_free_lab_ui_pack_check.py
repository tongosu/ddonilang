#!/usr/bin/env python3
"""Validate BA3_FREE_LAB_UI_PACK_CLOSURE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "free_lab_3_v1"
CONTRACT = PACK / "contract.detjson"
UI_PACK = PACK / "ui_pack.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "free_lab_ui_pack.js"
UI_RUNNER = ROOT / "tests" / "free_lab_ui_pack_runner.mjs"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ba3-free-lab-ui] FAIL: {message}", file=sys.stderr)
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
        UI_PACK,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "run_roadmap_v2_ba2_free_lab_experiment_report_check.py",
    ]:
        require_file(path)
    require_tokens(UI_MODULE, ["BA3_FREE_LAB_UI_PACK_CLOSURE_V1", "buildFreeLabUiPack", "free_lab.ui_pack.v1"])
    require_tokens(UI_RUNNER, ["free_lab_ui_pack: ok", "data-free-lab-ui-pack", "ghost_compare_claim"])


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 10,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 11,
        "roadmap_v2_pack_evidence_reference_closed": 29,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 32,
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
        "pack": "free_lab_3_v1",
        "kind": "roadmap_v2_ba3_free_lab_ui_pack_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "BA3_FREE_LAB_UI_PACK_CLOSURE_V1",
        "roadmap_coordinate": "바-3",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "ba3_matrix_status": "닫힘-동작",
        "requires_ba2_closed": True,
        "requires_browser_runner_evidence": True,
        "parameter_sweep_claim": True,
        "ghost_compare_claim": True,
        "current_stage": "BA3 free lab UI pack closure",
        "next_item": "BA4_FREE_LAB_SHARE_PACK_CLOSURE_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    if contract.get("ui_runs") != ["baseline", "low_lever", "high_lever"]:
        fail(f"contract ui_runs={contract.get('ui_runs')!r}")
    check_payload(CONTRACT)

    ui_pack = read_json(UI_PACK)
    if ui_pack.get("status") != "free_lab_ui_ready":
        fail(f"ui_pack status={ui_pack.get('status')!r}")
    if ui_pack.get("matrix_closure_tier") != "닫힘-동작":
        fail("ui_pack must be 닫힘-동작")
    if ui_pack.get("parameter_sweep_claim") is not True:
        fail("ui_pack must claim parameter sweep")
    if ui_pack.get("ghost_compare_claim") is not True:
        fail("ui_pack must claim ghost compare")
    run_ids = [run.get("id") for run in ui_pack.get("runs", [])]
    if run_ids != ["baseline", "low_lever", "high_lever"]:
        fail(f"ui_pack runs mismatch: {run_ids!r}")
    if len([run for run in ui_pack.get("runs", []) if run.get("ghost") is True]) != 2:
        fail("ui_pack must include two ghost runs")
    check_payload(UI_PACK)

    for payload in [contract, ui_pack]:
        false_claims = payload.get("false_claims", {})
        for key, value in false_claims.items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [CONTRACT, UI_PACK, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 11",
            "Studio-local 초장기 계획: 10/18",
            "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1 PASS 필요",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "free_lab_3_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_ba2_free_lab_experiment_report_check.py"], timeout=900)
    run(["node", "tests/free_lab_ui_pack_runner.mjs"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ba3-free-lab-ui] OK")


if __name__ == "__main__":
    main()
