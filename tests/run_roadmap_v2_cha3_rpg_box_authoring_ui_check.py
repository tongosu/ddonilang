#!/usr/bin/env python3
"""Validate CHA3_RPG_BOX_NURIMAKER_AUTHORING_UI_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "CHA3_RPG_BOX_NURIMAKER_AUTHORING_UI_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "차-3_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "malhim_rpg_3_v1"
CONTRACT = PACK / "contract.detjson"
AUTHORING_UI = PACK / "authoring_ui.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "rpg_box_authoring_ui.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
UI_RUNNER = ROOT / "tests" / "rpg_box_authoring_ui_runner.mjs"
PREREQ_CHECK = ROOT / "tests" / "run_roadmap_v2_cha2_rpg_story_pack_closure_check.py"
RPGBOX_BLOCK_CHECK = ROOT / "tests" / "run_rpgbox_block_smoke_check.py"


def fail(message: str) -> None:
    print(f"[roadmap-v2-cha3-rpg-box-authoring-ui] FAIL: {message}", file=sys.stderr)
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
        AUTHORING_UI,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        UI_MODULE,
        APP,
        INDEX_HTML,
        STYLES,
        UI_RUNNER,
        PREREQ_CHECK,
        RPGBOX_BLOCK_CHECK,
    ]:
        require_file(path)

    shared_tokens = [
        "CHA3_RPG_BOX_NURIMAKER_AUTHORING_UI_V1",
        "CHA3 RPG box authoring UI closure 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 15/90 = 17%",
        "ROADMAP_V2 pack evidence 참고값: 35/90 = 39%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "CHA4_RPG_STORY_PACKAGE_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 2마루 닫힘마루 | story/RPG pack | actor/state/dialogue/map | story pack | 닫힘-동작 |",
            "| 3마루 작업실마루 | RPG Box/누리메이커 | map editor + script | authoring UI pack | 닫힘-동작 |",
        ],
    )
    require_tokens(GUIDE, ["#### 차-3", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `malhim_rpg_3_v1` |"])
    require_tokens(TRACKER, ["| 30 | `차-3` | 말힘누리/RPG Box authoring UI | 닫힘-동작 |", "| `차-3` | RPG Box/누리메이커 authoring UI | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `차-3` | `malhim_rpg_3_v1`; UI `rpg_box_authoring_ui.js`; runner `rpg_box_authoring_ui_runner.mjs` |"])
    require_tokens(DEV_SURFACES, ["rpg_box_authoring_ui.js", "__SEAMGRIM_RPG_BOX_AUTHORING_UI__"])
    require_tokens(INDEX_HTML, ["id=\"rpg-box-authoring-ui\"", "data-rpg-box-authoring-ui"])
    require_tokens(STYLES, [".rpg-box-authoring-ui", ".rpg-box-authoring-grid", ".rpg-box-authoring-script"])


def check_cha3_status_closed() -> None:
    matrix_line = ""
    for line in read(MATRIX).splitlines():
        if "| 3마루 작업실마루 | RPG Box/누리메이커 |" in line:
            matrix_line = line
            break
    if not matrix_line:
        fail("missing 차-3 matrix line")
    status_cell = matrix_line.rstrip().split("|")[-2].strip()
    if status_cell != "닫힘-동작":
        fail(f"차-3 status must be 닫힘-동작: {matrix_line}")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 15,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 17,
        "roadmap_v2_pack_evidence_reference_closed": 35,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 39,
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
        "pack": "malhim_rpg_3_v1",
        "kind": "roadmap_v2_cha3_rpg_box_authoring_ui_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "CHA3_RPG_BOX_NURIMAKER_AUTHORING_UI_V1",
        "roadmap_coordinate": "차-3",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "cha3_matrix_status": "닫힘-동작",
        "requires_cha2_closed": True,
        "requires_browser_runner_evidence": True,
        "authoring_ui_claim": True,
        "map_editor_claim": True,
        "script_block_claim": True,
        "playtest_handoff_claim": True,
        "current_stage": "CHA3 RPG box authoring UI closure",
        "next_item": "CHA4_RPG_STORY_PACKAGE_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    if contract.get("authoring_panels") != ["map_editor", "script_blocks", "playtest"]:
        fail(f"contract authoring_panels={contract.get('authoring_panels')!r}")
    if contract.get("map_cells") != ["start", "door", "npc"]:
        fail(f"contract map_cells={contract.get('map_cells')!r}")
    check_payload(CONTRACT)

    authoring = read_json(AUTHORING_UI)
    if authoring.get("status") != "rpg_box_authoring_ui_ready":
        fail(f"authoring_ui status={authoring.get('status')!r}")
    if authoring.get("matrix_closure_tier") != "닫힘-동작":
        fail("authoring_ui must be 닫힘-동작")
    for key in ["authoring_ui_claim", "map_editor_claim", "script_block_claim", "playtest_handoff_claim"]:
        if authoring.get(key) is not True:
            fail(f"authoring_ui {key} must be true")
    panel_ids = [row.get("id") for row in authoring.get("panels", [])]
    if panel_ids != ["map_editor", "script_blocks", "playtest"]:
        fail(f"authoring panel rows mismatch: {panel_ids!r}")
    cell_ids = [row.get("id") for row in authoring.get("map_cells", [])]
    if cell_ids != ["start", "door", "npc"]:
        fail(f"map cells mismatch: {cell_ids!r}")
    script_text = "\n".join(str(row) for row in authoring.get("script_lines", []))
    for token in ["open_door", "position=문앞", "안녕 용사 방문=1"]:
        if token not in script_text:
            fail(f"authoring script missing {token}")
    if "roadmap matrix behavior: 15/90 = 17%" not in str(authoring.get("playtest_ddn", "")):
        fail("authoring playtest DDN missing progress")
    check_payload(AUTHORING_UI)
    for payload in [contract, authoring]:
        false_claims = payload.get("false_claims", {})
        for key, value in false_claims.items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, AUTHORING_UI, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 16",
            "Studio-local 초장기 계획: 10/18",
            "CHA4_RPG_STORY_PACKAGE_V1 PASS 없이 닫힘",
            "engine_adapter_claim\": true",
            "story_package_claim\": true",
            "registry_publish_claim\": true",
            "cloud_sync_claim\": true",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "malhim_rpg_3_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_cha2_rpg_story_pack_closure_check.py"], timeout=900)
    run([sys.executable, "tests/run_rpgbox_block_smoke_check.py"], timeout=240)
    run(["node", "tests/rpg_box_authoring_ui_runner.mjs"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_cha3_status_closed()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-cha3-rpg-box-authoring-ui] OK")


if __name__ == "__main__":
    main()
