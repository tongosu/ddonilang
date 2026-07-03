#!/usr/bin/env python3
"""Validate CHA4_RPG_STORY_PACKAGE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "CHA4_RPG_STORY_PACKAGE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "차-4_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "malhim_rpg_4_v1"
CONTRACT = PACK / "contract.detjson"
STORY_PACKAGE = PACK / "story_package.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "rpg_story_package.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
UI_RUNNER = ROOT / "tests" / "rpg_story_package_runner.mjs"
PREREQ_CHECK = ROOT / "tests" / "run_roadmap_v2_cha3_rpg_box_authoring_ui_check.py"


def fail(message: str) -> None:
    print(f"[roadmap-v2-cha4-rpg-story-package] FAIL: {message}", file=sys.stderr)
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
        STORY_PACKAGE,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        UI_MODULE,
        APP,
        INDEX_HTML,
        STYLES,
        UI_RUNNER,
        PREREQ_CHECK,
    ]:
        require_file(path)

    shared_tokens = [
        "CHA4_RPG_STORY_PACKAGE_V1",
        "CHA4 RPG story package closure 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 16/90 = 18%",
        "ROADMAP_V2 pack evidence 참고값: 36/90 = 40%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "CHA5_RPG_ENGINE_ADAPTER_LTS_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 3마루 작업실마루 | RPG Box/누리메이커 | map editor + script | authoring UI pack | 닫힘-동작 |",
            "| 4마루 나눔마루 | story package | story artifact family | package pack | 닫힘-동작 |",
        ],
    )
    require_tokens(GUIDE, ["#### 차-4", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `malhim_rpg_4_v1` |"])
    require_tokens(TRACKER, ["| 31 | `차-4` | 말힘누리/RPG story package | 닫힘-동작 |", "| `차-4` | RPG story package | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `차-4` | `malhim_rpg_4_v1`; UI `rpg_story_package.js`; runner `rpg_story_package_runner.mjs` |"])
    require_tokens(DEV_SURFACES, ["rpg_story_package.js", "__SEAMGRIM_RPG_STORY_PACKAGE__"])
    require_tokens(INDEX_HTML, ["id=\"rpg-story-package\"", "data-rpg-story-package"])
    require_tokens(DEV_SURFACES_CSS, [".rpg-story-package", ".rpg-story-package-files", ".rpg-story-package-preview"])


def check_cha4_status_closed() -> None:
    matrix_line = ""
    for line in read(MATRIX).splitlines():
        if "| 4마루 나눔마루 | story package |" in line:
            matrix_line = line
            break
    if not matrix_line:
        fail("missing 차-4 matrix line")
    status_cell = matrix_line.rstrip().split("|")[-2].strip()
    if status_cell != "닫힘-동작":
        fail(f"차-4 status must be 닫힘-동작: {matrix_line}")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 16,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 18,
        "roadmap_v2_pack_evidence_reference_closed": 36,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 40,
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
        "pack": "malhim_rpg_4_v1",
        "kind": "roadmap_v2_cha4_rpg_story_package_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "CHA4_RPG_STORY_PACKAGE_V1",
        "roadmap_coordinate": "차-4",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "cha4_matrix_status": "닫힘-동작",
        "requires_cha3_closed": True,
        "requires_browser_runner_evidence": True,
        "story_package_claim": True,
        "manifest_claim": True,
        "map_snapshot_claim": True,
        "script_bundle_claim": True,
        "playtest_transcript_claim": True,
        "current_stage": "CHA4 RPG story package closure",
        "next_item": "CHA5_RPG_ENGINE_ADAPTER_LTS_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    if contract.get("package_artifacts") != ["manifest", "map_snapshot", "script_bundle", "playtest_transcript"]:
        fail(f"contract package_artifacts={contract.get('package_artifacts')!r}")
    if contract.get("package_files") != [
        "story.manifest.detjson",
        "map.snapshot.detjson",
        "script.bundle.ddn",
        "playtest.transcript.txt",
    ]:
        fail(f"contract package_files={contract.get('package_files')!r}")
    check_payload(CONTRACT)

    story_package = read_json(STORY_PACKAGE)
    if story_package.get("status") != "rpg_story_package_ready":
        fail(f"story_package status={story_package.get('status')!r}")
    if story_package.get("matrix_closure_tier") != "닫힘-동작":
        fail("story_package must be 닫힘-동작")
    for key in ["story_package_claim", "manifest_claim", "map_snapshot_claim", "script_bundle_claim", "playtest_transcript_claim"]:
        if story_package.get(key) is not True:
            fail(f"story_package {key} must be true")
    artifact_ids = [row.get("id") for row in story_package.get("artifacts", [])]
    if artifact_ids != ["manifest", "map_snapshot", "script_bundle", "playtest_transcript"]:
        fail(f"artifact rows mismatch: {artifact_ids!r}")
    file_names = [row.get("name") for row in story_package.get("package_files", [])]
    if file_names != [
        "story.manifest.detjson",
        "map.snapshot.detjson",
        "script.bundle.ddn",
        "playtest.transcript.txt",
    ]:
        fail(f"package files mismatch: {file_names!r}")
    package_text = str(story_package.get("package_text", ""))
    for token in ["coordinate:차-4", "story.manifest.detjson", "playtest.transcript.txt"]:
        if token not in package_text:
            fail(f"package text missing {token}")
    check_payload(STORY_PACKAGE)
    for payload in [contract, story_package]:
        false_claims = payload.get("false_claims", {})
        for key, value in false_claims.items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, STORY_PACKAGE, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 17",
            "Studio-local 초장기 계획: 10/18",
            "CHA5_RPG_ENGINE_ADAPTER_LTS_V1 PASS 없이 닫힘",
            "engine_adapter_claim\": true",
            "registry_publish_claim\": true",
            "public_upload_claim\": true",
            "cloud_sync_claim\": true",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "malhim_rpg_4_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_cha3_rpg_box_authoring_ui_check.py"], timeout=900)
    run(["node", "tests/rpg_story_package_runner.mjs"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_cha4_status_closed()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-cha4-rpg-story-package] OK")


if __name__ == "__main__":
    main()
