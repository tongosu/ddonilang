#!/usr/bin/env python3
"""Validate CHA2_RPG_STORY_PACK_CLOSURE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "CHA2_RPG_STORY_PACK_CLOSURE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "차-2_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "malhim_rpg_2_v1"
CONTRACT = PACK / "contract.detjson"
STORY_PACK = PACK / "story_pack.detjson"
PREREQ_CHECK = ROOT / "tests" / "run_roadmap_v2_cha1_rpg_phrase_action_smoke_check.py"
EXPECTED_STDOUT = [
    "actor:용사@문앞",
    "state:position=문앞",
    "dialogue:안녕 용사 방문=1",
    "map:시작->문앞",
]


def fail(message: str) -> None:
    print(f"[roadmap-v2-cha2-rpg-story-pack] FAIL: {message}", file=sys.stderr)
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
        STORY_PACK,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        PREREQ_CHECK,
    ]:
        require_file(path)

    shared_tokens = [
        "CHA2_RPG_STORY_PACK_CLOSURE_V1",
        "CHA2 RPG story pack closure 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 14/90 = 16%",
        "ROADMAP_V2 pack evidence 참고값: 34/90 = 38%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "CHA3_RPG_BOX_NURIMAKER_AUTHORING_UI_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 1마루 첫실행마루 | phrase/action smoke | phrase→confirmed_event | smoke pack | 닫힘-동작 |",
            "| 2마루 닫힘마루 | story/RPG pack | actor/state/dialogue/map | story pack | 닫힘-동작 |",
        ],
    )
    require_tokens(GUIDE, ["#### 차-2", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `malhim_rpg_2_v1` |"])
    require_tokens(TRACKER, ["| 29 | `차-2` | 말힘누리/RPG story pack | 닫힘-동작 |", "| `차-2` | RPG story pack | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `차-2` | `malhim_rpg_2_v1`; DDN runtime story pack |"])


def check_cha2_status_closed() -> None:
    matrix_line = ""
    for line in read(MATRIX).splitlines():
        if "| 2마루 닫힘마루 | story/RPG pack |" in line:
            matrix_line = line
            break
    if not matrix_line:
        fail("missing 차-2 matrix line")
    status_cell = matrix_line.rstrip().split("|")[-2].strip()
    if status_cell != "닫힘-동작":
        fail(f"차-2 status must be 닫힘-동작: {matrix_line}")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 14,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 16,
        "roadmap_v2_pack_evidence_reference_closed": 34,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 38,
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
        "pack": "malhim_rpg_2_v1",
        "kind": "roadmap_v2_cha2_rpg_story_pack_closure",
        "runtime_claim": True,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "CHA2_RPG_STORY_PACK_CLOSURE_V1",
        "roadmap_coordinate": "차-2",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "cha2_matrix_status": "닫힘-동작",
        "requires_cha1_closed": True,
        "current_stage": "CHA2 RPG story pack closure",
        "next_item": "CHA3_RPG_BOX_NURIMAKER_AUTHORING_UI_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    if contract.get("story_lanes") != ["actor", "state", "dialogue", "map", "runtime_golden"]:
        fail(f"contract story_lanes={contract.get('story_lanes')!r}")
    if contract.get("expected_stdout") != EXPECTED_STDOUT:
        fail(f"contract expected_stdout={contract.get('expected_stdout')!r}")
    check_payload(CONTRACT)

    story_pack = read_json(STORY_PACK)
    if story_pack.get("status") != "story_pack_runtime_pass":
        fail(f"story_pack status={story_pack.get('status')!r}")
    if story_pack.get("matrix_closure_tier") != "닫힘-동작":
        fail("story_pack must be 닫힘-동작")
    lanes = story_pack.get("story_boundary", {}).get("lanes", [])
    lane_ids = [lane.get("id") for lane in lanes]
    if lane_ids != ["actor", "state", "dialogue", "map", "runtime_golden"]:
        fail(f"story lanes mismatch: {lane_ids!r}")
    if not all(lane.get("required_for_cha2") is True for lane in lanes):
        fail("all story lanes must be required for CHA2")
    evidence_text = json.dumps(story_pack, ensure_ascii=False)
    for token in EXPECTED_STDOUT:
        if token not in evidence_text:
            fail(f"story_pack missing expected stdout token: {token}")
    check_payload(STORY_PACK)
    false_claims = story_pack.get("false_claims", {})
    for key, value in false_claims.items():
        if value is not False:
            fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, STORY_PACK]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 15",
            "Studio-local 초장기 계획: 10/18",
            "CHA3_RPG_BOX_NURIMAKER_AUTHORING_UI_V1 PASS 없이 닫힘",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "malhim_rpg_2_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_cha1_rpg_phrase_action_smoke_check.py"], timeout=900)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_cha2_status_closed()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-cha2-rpg-story-pack] OK")


if __name__ == "__main__":
    main()
