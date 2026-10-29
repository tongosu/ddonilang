#!/usr/bin/env python3
"""Validate BA4_FREE_LAB_SHARE_PACK_CLOSURE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "free_lab_4_v1"
CONTRACT = PACK / "contract.detjson"
SHARE_PACK = PACK / "share_pack.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "free_lab_share_pack.js"
UI_RUNNER = ROOT / "tests" / "free_lab_share_pack_runner.mjs"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ba4-free-lab-share] FAIL: {message}", file=sys.stderr)
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
        SHARE_PACK,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "run_roadmap_v2_ba3_free_lab_ui_pack_check.py",
    ]:
        require_file(path)
    require_tokens(UI_MODULE, ["BA4_FREE_LAB_SHARE_PACK_CLOSURE_V1", "buildFreeLabSharePack", "free_lab.share_pack.v1"])
    require_tokens(UI_RUNNER, ["free_lab_share_pack: ok", "data-free-lab-share-pack", "seamgrim://free-lab/local/"])


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 11,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 12,
        "roadmap_v2_pack_evidence_reference_closed": 30,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 33,
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
        "pack": "free_lab_4_v1",
        "kind": "roadmap_v2_ba4_free_lab_share_pack_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "BA4_FREE_LAB_SHARE_PACK_CLOSURE_V1",
        "roadmap_coordinate": "바-4",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "ba4_matrix_status": "닫힘-동작",
        "requires_ba3_closed": True,
        "requires_browser_runner_evidence": True,
        "local_share_claim": True,
        "remix_link_claim": True,
        "current_stage": "BA4 free lab share pack closure",
        "next_item": "BA5_FREE_LAB_RESEARCH_WORKFLOW_CLOSURE_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    if contract.get("share_rows") != ["snapshot", "remix", "handoff"]:
        fail(f"contract share_rows={contract.get('share_rows')!r}")
    check_payload(CONTRACT)

    share_pack = read_json(SHARE_PACK)
    if share_pack.get("status") != "free_lab_share_ready":
        fail(f"share_pack status={share_pack.get('status')!r}")
    if share_pack.get("matrix_closure_tier") != "닫힘-동작":
        fail("share_pack must be 닫힘-동작")
    if share_pack.get("local_share_claim") is not True:
        fail("share_pack must claim local share")
    if share_pack.get("remix_link_claim") is not True:
        fail("share_pack must claim remix link")
    share_ids = [share.get("id") for share in share_pack.get("shares", [])]
    if share_ids != ["snapshot", "remix", "handoff"]:
        fail(f"share_pack rows mismatch: {share_ids!r}")
    for share in share_pack.get("shares", []):
        if share.get("local_only") is not True:
            fail(f"share {share.get('id')} must be local_only")
        if not str(share.get("share_link", "")).startswith("seamgrim://free-lab/local/"):
            fail(f"share link must be local: {share!r}")
    check_payload(SHARE_PACK)

    for payload in [contract, share_pack]:
        false_claims = payload.get("false_claims", {})
        for key, value in false_claims.items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [CONTRACT, SHARE_PACK, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 12",
            "Studio-local 초장기 계획: 10/18",
            "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1 PASS 필요",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "free_lab_4_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_ba3_free_lab_ui_pack_check.py"], timeout=900)
    run(["node", "tests/free_lab_share_pack_runner.mjs"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ba4-free-lab-share] OK")


if __name__ == "__main__":
    main()
