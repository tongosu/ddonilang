#!/usr/bin/env python3
"""Validate BA0_FREE_LAB_SEED_REBASE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "BA0_FREE_LAB_SEED_REBASE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "바-0_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "roadmap_v2_ba0_free_lab_seed_rebase_v1"
CONTRACT = PACK / "contract.detjson"
SEED = PACK / "seed.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ba0-free-lab-seed] FAIL: {message}", file=sys.stderr)
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
        DOC,
        REPORT,
        MATRIX,
        GUIDE,
        TRACKER,
        MANIFEST,
        DEV_SUMMARY,
        PACK / "README.md",
        CONTRACT,
        SEED,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "run_roadmap_v2_ma5_seamgrim_curriculum_5_lts_pack_closure_check.py",
    ]:
        require_file(path)

    shared_tokens = [
        "BA0_FREE_LAB_SEED_REBASE_V1",
        "BA0 free lab seed rebase 3/3 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 7/90 = 8%",
        "ROADMAP_V2 pack evidence 참고값: 26/90 = 29%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "BA1_FREE_LAB_FIRST_RUN_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 0마루 씨앗마루 | 빈 작업실 설계 | 새 실험, 매김, 기록 | free lab proposal |",
            "| 5마루 단단마루 | 교과 LTS | 학년/단원/버전 관리 | curriculum LTS | 닫힘-동작 |",
        ],
    )
    require_tokens(GUIDE, ["#### 바-0", "| pack 후보 | `free_lab_0_v1`"])
    require_tokens(TRACKER, ["| 21 | `바-0` | 자유 실험실", "| `바-0` | Free lab"])
    require_tokens(MANIFEST, ["| `바-0` | `roadmap_v2_ba0_free_lab_seed_rebase_v1`"])


def check_ba0_status_seed_only() -> None:
    matrix_line = ""
    for line in read(MATRIX).splitlines():
        if "| 0마루 씨앗마루 | 빈 작업실 설계 |" in line:
            matrix_line = line
            break
    if not matrix_line:
        fail("missing 바-0 matrix line")
    status_cell = matrix_line.rstrip().split("|")[-2].strip()
    if status_cell not in {"씨앗", "닫힘-동작"}:
        fail(f"바-0 status must be 씨앗 or 닫힘-동작: {matrix_line}")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 3,
        "current_stage_total": 3,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 7,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 8,
        "roadmap_v2_pack_evidence_reference_closed": 26,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 29,
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
        "pack": "roadmap_v2_ba0_free_lab_seed_rebase_v1",
        "kind": "roadmap_v2_ba0_free_lab_seed_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "BA0_FREE_LAB_SEED_REBASE_V1",
        "roadmap_coordinate": "바-0",
        "matrix_closure_claim": False,
        "matrix_closure_tier": "닫힘-문서",
        "ba0_matrix_status": "씨앗",
        "current_stage": "BA0 free lab seed rebase",
        "next_item": "BA1_FREE_LAB_FIRST_RUN_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    if contract.get("seed_lanes") != ["new_experiment", "parameter_setup", "recording_boundary"]:
        fail(f"contract seed_lanes={contract.get('seed_lanes')!r}")
    check_payload(CONTRACT)

    seed = read_json(SEED)
    if seed.get("status") != "docs_closed_seed":
        fail(f"seed status={seed.get('status')!r}")
    if seed.get("ba0_matrix_status") != "씨앗":
        fail("seed must record 바-0 as 씨앗")
    if seed.get("matrix_closure_claim") is not False:
        fail("seed must not claim matrix closure")
    lanes = seed.get("seed_boundary", {}).get("lanes", [])
    if [lane.get("id") for lane in lanes] != ["new_experiment", "parameter_setup", "recording_boundary"]:
        fail(f"seed lanes mismatch: {lanes!r}")
    if not all(lane.get("required_for_ba1") is True for lane in lanes):
        fail("all seed lanes must be required for BA1")
    check_payload(SEED)
    false_claims = seed.get("false_claims", {})
    for key, value in false_claims.items():
        if value is not False:
            fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, SEED]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 8",
            "Studio-local 초장기 계획: 10/18",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ba0_free_lab_seed_rebase_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_ma5_seamgrim_curriculum_5_lts_pack_closure_check.py"], timeout=900)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_ba0_status_seed_only()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ba0-free-lab-seed] OK")


if __name__ == "__main__":
    main()
