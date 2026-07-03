#!/usr/bin/env python3
"""Validate BA5_FREE_LAB_RESEARCH_WORKFLOW_CLOSURE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "free_lab_5_v1"
CONTRACT = PACK / "contract.detjson"
RESEARCH_WORKFLOW = PACK / "research_workflow.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "free_lab_research_workflow.js"
UI_RUNNER = ROOT / "tests" / "free_lab_research_workflow_runner.mjs"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ba5-free-lab-research] FAIL: {message}", file=sys.stderr)
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
        RESEARCH_WORKFLOW,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "run_roadmap_v2_ba4_free_lab_share_pack_check.py",
    ]:
        require_file(path)
    require_tokens(UI_MODULE, ["BA5_FREE_LAB_RESEARCH_WORKFLOW_CLOSURE_V1", "buildFreeLabResearchWorkflow", "free_lab.research_workflow.v1"])
    require_tokens(UI_RUNNER, ["free_lab_research_workflow: ok", "data-free-lab-research-workflow", "run_id,coefficient,start_value,frame_3_value"])


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 12,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 13,
        "roadmap_v2_pack_evidence_reference_closed": 31,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 34,
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
        "pack": "free_lab_5_v1",
        "kind": "roadmap_v2_ba5_free_lab_research_workflow_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "BA5_FREE_LAB_RESEARCH_WORKFLOW_CLOSURE_V1",
        "roadmap_coordinate": "바-5",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "ba5_matrix_status": "닫힘-동작",
        "requires_ba4_closed": True,
        "requires_browser_runner_evidence": True,
        "research_mode_claim": True,
        "batch_queue_claim": True,
        "csv_export_claim": True,
        "notebook_handoff_claim": True,
        "current_stage": "BA5 free lab research workflow closure",
        "next_item": "CHA0_RPG_SEED_REBASE_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    if contract.get("workflow_rows") != ["batch", "csv", "notebook"]:
        fail(f"contract workflow_rows={contract.get('workflow_rows')!r}")
    check_payload(CONTRACT)

    workflow = read_json(RESEARCH_WORKFLOW)
    if workflow.get("status") != "free_lab_research_workflow_ready":
        fail(f"research_workflow status={workflow.get('status')!r}")
    if workflow.get("matrix_closure_tier") != "닫힘-동작":
        fail("research_workflow must be 닫힘-동작")
    for key in ["research_mode_claim", "batch_queue_claim", "csv_export_claim", "notebook_handoff_claim"]:
        if workflow.get(key) is not True:
            fail(f"research_workflow {key} must be true")
    workflow_ids = [row.get("id") for row in workflow.get("workflows", [])]
    if workflow_ids != ["batch", "csv", "notebook"]:
        fail(f"workflow rows mismatch: {workflow_ids!r}")
    run_ids = [row.get("id") for row in workflow.get("batch_runs", [])]
    if run_ids != ["baseline", "low_lever", "high_lever"]:
        fail(f"batch runs mismatch: {run_ids!r}")
    if "run_id,coefficient,start_value,frame_3_value" not in str(workflow.get("csv_text", "")):
        fail("research_workflow csv_text missing header")
    for row in workflow.get("workflows", []):
        if row.get("local_only") is not True:
            fail(f"workflow {row.get('id')} must be local_only")
    check_payload(RESEARCH_WORKFLOW)

    for payload in [contract, workflow]:
        false_claims = payload.get("false_claims", {})
        for key, value in false_claims.items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [CONTRACT, RESEARCH_WORKFLOW, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 13",
            "Studio-local 초장기 계획: 10/18",
            "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1 PASS 필요",
            "external_notebook_server_claim\": true",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "free_lab_5_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_ba4_free_lab_share_pack_check.py"], timeout=900)
    run(["node", "tests/free_lab_research_workflow_runner.mjs"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ba5-free-lab-research] OK")


if __name__ == "__main__":
    main()
