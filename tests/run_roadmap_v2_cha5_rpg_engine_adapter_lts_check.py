#!/usr/bin/env python3
"""Validate CHA5_RPG_ENGINE_ADAPTER_LTS_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "CHA5_RPG_ENGINE_ADAPTER_LTS_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "차-5_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "malhim_rpg_5_v1"
CONTRACT = PACK / "contract.detjson"
ADAPTER_LTS = PACK / "adapter_lts.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "rpg_engine_adapter_lts.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
UI_RUNNER = ROOT / "tests" / "rpg_engine_adapter_lts_runner.mjs"
PREREQ_CHECK = ROOT / "tests" / "run_roadmap_v2_cha4_rpg_story_package_check.py"


def fail(message: str) -> None:
    print(f"[roadmap-v2-cha5-rpg-engine-adapter-lts] FAIL: {message}", file=sys.stderr)
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
        ADAPTER_LTS,
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
        "CHA5_RPG_ENGINE_ADAPTER_LTS_V1",
        "CHA5 RPG engine adapter LTS closure 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 17/90 = 19%",
        "ROADMAP_V2 pack evidence 참고값: 37/90 = 41%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "KA2_PUBLICATION_READ_API_CLOSURE_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 4마루 나눔마루 | story package | story artifact family | package pack | 닫힘-동작 |",
            "| 5마루 단단마루 | engine adapter | Godot/native adapter | adapter LTS | 닫힘-동작 |",
        ],
    )
    require_tokens(GUIDE, ["#### 차-5", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `malhim_rpg_5_v1` |"])
    require_tokens(TRACKER, ["| 32 | `차-5` | 말힘누리/RPG engine adapter LTS | 닫힘-동작 |", "| `차-5` | RPG engine adapter LTS | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `차-5` | `malhim_rpg_5_v1`; UI `rpg_engine_adapter_lts.js`; runner `rpg_engine_adapter_lts_runner.mjs` |"])
    require_tokens(DEV_SURFACES, ["rpg_engine_adapter_lts.js", "__SEAMGRIM_RPG_ENGINE_ADAPTER_LTS__"])
    require_tokens(INDEX_HTML, ["id=\"rpg-engine-adapter-lts\"", "data-rpg-engine-adapter-lts"])
    require_tokens(DEV_SURFACES_CSS, [".rpg-engine-adapter-lts", ".rpg-engine-adapter-files", ".rpg-engine-adapter-preview"])


def check_cha5_status_closed() -> None:
    matrix_line = ""
    for line in read(MATRIX).splitlines():
        if "| 5마루 단단마루 | engine adapter |" in line:
            matrix_line = line
            break
    if not matrix_line:
        fail("missing 차-5 matrix line")
    status_cell = matrix_line.rstrip().split("|")[-2].strip()
    if status_cell != "닫힘-동작":
        fail(f"차-5 status must be 닫힘-동작: {matrix_line}")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 17,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 19,
        "roadmap_v2_pack_evidence_reference_closed": 37,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 41,
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
        "pack": "malhim_rpg_5_v1",
        "kind": "roadmap_v2_cha5_rpg_engine_adapter_lts_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "CHA5_RPG_ENGINE_ADAPTER_LTS_V1",
        "roadmap_coordinate": "차-5",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "cha5_matrix_status": "닫힘-동작",
        "requires_cha4_closed": True,
        "requires_browser_runner_evidence": True,
        "adapter_lts_claim": True,
        "godot_manifest_claim": True,
        "native_bridge_contract_claim": True,
        "asset_map_claim": True,
        "lts_gate_claim": True,
        "current_stage": "CHA5 RPG engine adapter LTS closure",
        "next_item": "KA2_PUBLICATION_READ_API_CLOSURE_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    if contract.get("adapter_artifacts") != ["godot_manifest", "native_bridge", "asset_map", "lts_gate"]:
        fail(f"contract adapter_artifacts={contract.get('adapter_artifacts')!r}")
    check_payload(CONTRACT)

    adapter_lts = read_json(ADAPTER_LTS)
    if adapter_lts.get("status") != "rpg_engine_adapter_lts_ready":
        fail(f"adapter_lts status={adapter_lts.get('status')!r}")
    if adapter_lts.get("matrix_closure_tier") != "닫힘-동작":
        fail("adapter_lts must be 닫힘-동작")
    for key in ["adapter_lts_claim", "godot_manifest_claim", "native_bridge_contract_claim", "asset_map_claim", "lts_gate_claim"]:
        if adapter_lts.get(key) is not True:
            fail(f"adapter_lts {key} must be true")
    adapter_ids = [row.get("id") for row in adapter_lts.get("adapters", [])]
    if adapter_ids != ["godot_manifest", "native_bridge", "asset_map", "lts_gate"]:
        fail(f"adapter rows mismatch: {adapter_ids!r}")
    adapter_text = str(adapter_lts.get("adapter_text", ""))
    for token in ["coordinate:차-5", "runtime_execution:false", "native.bridge.contract.detjson"]:
        if token not in adapter_text:
            fail(f"adapter text missing {token}")
    check_payload(ADAPTER_LTS)
    for payload in [contract, adapter_lts]:
        false_claims = payload.get("false_claims", {})
        for key, value in false_claims.items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, ADAPTER_LTS, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 18",
            "Studio-local 초장기 계획: 10/18",
            "KA2_PUBLICATION_READ_API_CLOSURE_V1 PASS 없이 닫힘",
            "native_runtime_execution_claim\": true",
            "godot_project_build_claim\": true",
            "native_binary_claim\": true",
            "registry_publish_claim\": true",
            "public_upload_claim\": true",
            "cloud_sync_claim\": true",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "malhim_rpg_5_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_cha4_rpg_story_package_check.py"], timeout=900)
    run(["node", "tests/rpg_engine_adapter_lts_runner.mjs"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_cha5_status_closed()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-cha5-rpg-engine-adapter-lts] OK")


if __name__ == "__main__":
    main()
