#!/usr/bin/env python3
"""Validate CHA1_RPG_PHRASE_ACTION_SMOKE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "CHA1_RPG_PHRASE_ACTION_SMOKE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "차-1_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "malhim_rpg_1_v1"
CONTRACT = PACK / "contract.detjson"
SMOKE = PACK / "smoke.detjson"
PREREQ_CHECK = ROOT / "tests" / "run_roadmap_v2_cha0_rpg_seed_rebase_check.py"
EXPECTED_STDOUT = ["1", "confirmed_event:open_door:문을 연다:count=1"]


def fail(message: str) -> None:
    print(f"[roadmap-v2-cha1-rpg-phrase-action] FAIL: {message}", file=sys.stderr)
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
        SMOKE,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        PREREQ_CHECK,
    ]:
        require_file(path)

    shared_tokens = [
        "CHA1_RPG_PHRASE_ACTION_SMOKE_V1",
        "CHA1 RPG phrase action smoke 4/4 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 13/90 = 14%",
        "ROADMAP_V2 pack evidence 참고값: 33/90 = 37%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "CHA2_RPG_STORY_PACK_CLOSURE_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 0마루 씨앗마루 | action/story 설계 | action_method, phrase binding | proposal/schema |",
            "| 1마루 첫실행마루 | phrase/action smoke | phrase→confirmed_event | smoke pack | 닫힘-동작 |",
        ],
    )
    require_tokens(GUIDE, ["#### 차-1", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `malhim_rpg_1_v1` |"])
    require_tokens(TRACKER, ["| 28 | `차-1` | 말힘누리/RPG phrase action smoke | 닫힘-동작 |", "| `차-1` | RPG phrase/action smoke | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `차-1` | `malhim_rpg_1_v1`; DDN runtime phrase/action smoke |"])


def check_cha1_status_closed() -> None:
    matrix_line = ""
    for line in read(MATRIX).splitlines():
        if "| 1마루 첫실행마루 | phrase/action smoke |" in line:
            matrix_line = line
            break
    if not matrix_line:
        fail("missing 차-1 matrix line")
    status_cell = matrix_line.rstrip().split("|")[-2].strip()
    if status_cell != "닫힘-동작":
        fail(f"차-1 status must be 닫힘-동작: {matrix_line}")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 4,
        "current_stage_total": 4,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 13,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 14,
        "roadmap_v2_pack_evidence_reference_closed": 33,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 37,
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
        "pack": "malhim_rpg_1_v1",
        "kind": "roadmap_v2_cha1_rpg_phrase_action_smoke",
        "runtime_claim": True,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "CHA1_RPG_PHRASE_ACTION_SMOKE_V1",
        "roadmap_coordinate": "차-1",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "cha1_matrix_status": "닫힘-동작",
        "requires_cha0_seed": True,
        "current_stage": "CHA1 RPG phrase action smoke",
        "next_item": "CHA2_RPG_STORY_PACK_CLOSURE_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    if contract.get("smoke_lanes") != ["phrase_input", "action_method", "confirmed_event", "runtime_golden"]:
        fail(f"contract smoke_lanes={contract.get('smoke_lanes')!r}")
    if contract.get("expected_stdout") != EXPECTED_STDOUT:
        fail(f"contract expected_stdout={contract.get('expected_stdout')!r}")
    check_payload(CONTRACT)

    smoke = read_json(SMOKE)
    if smoke.get("status") != "runtime_smoke_pass":
        fail(f"smoke status={smoke.get('status')!r}")
    if smoke.get("matrix_closure_tier") != "닫힘-동작":
        fail("smoke must be 닫힘-동작")
    lanes = smoke.get("smoke_boundary", {}).get("lanes", [])
    lane_ids = [lane.get("id") for lane in lanes]
    if lane_ids != ["phrase_input", "action_method", "confirmed_event", "runtime_golden"]:
        fail(f"smoke lanes mismatch: {lane_ids!r}")
    if not all(lane.get("required_for_cha1") is True for lane in lanes):
        fail("all smoke lanes must be required for CHA1")
    evidence_text = json.dumps(smoke, ensure_ascii=False)
    for token in EXPECTED_STDOUT:
        if token not in evidence_text:
            fail(f"smoke missing expected stdout token: {token}")
    check_payload(SMOKE)
    false_claims = smoke.get("false_claims", {})
    for key, value in false_claims.items():
        if value is not False:
            fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, SMOKE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 14",
            "Studio-local 초장기 계획: 10/18",
            "CHA2_RPG_STORY_PACK_CLOSURE_V1 PASS 없이 닫힘",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "malhim_rpg_1_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_cha0_rpg_seed_rebase_check.py"], timeout=900)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_cha1_status_closed()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-cha1-rpg-phrase-action] OK")


if __name__ == "__main__":
    main()
