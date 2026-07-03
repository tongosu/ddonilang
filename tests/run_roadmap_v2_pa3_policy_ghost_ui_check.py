#!/usr/bin/env python3
"""Validate PA3_POLICY_GHOST_UI_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "PA3_POLICY_GHOST_UI_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "파-3_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "social_world_econ_3_v1"
CONTRACT = PACK / "contract.detjson"
POLICY = PACK / "policy_ghost.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "social_world_policy_ghost_ui.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
UI_RUNNER = ROOT / "tests" / "social_world_policy_ghost_ui_runner.mjs"
PREREQ_PA2 = ROOT / "tests" / "run_roadmap_v2_pa2_social_bridge_pack_check.py"


def fail(message: str) -> None:
    print(f"[roadmap-v2-pa3-policy-ghost-ui] FAIL: {message}", file=sys.stderr)
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
        POLICY,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        UI_MODULE,
        APP,
        INDEX_HTML,
        STYLES,
        UI_RUNNER,
        PREREQ_PA2,
    ]:
        require_file(path)
    shared_tokens = [
        "PA3_POLICY_GHOST_UI_V1",
        "PA3 policy ghost UI closure 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 26/90 = 29%",
        "ROADMAP_V2 pack evidence 참고값: 46/90 = 51%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "PA4_SOCIAL_TEMPLATE_REGISTRY_V1",
        "real-world prediction",
        "policy_advice:false",
        "agent_simulation_execution:false",
        "live_policy_deployment:false",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(MATRIX, ["| 3마루 작업실마루 | 정책 고스트 UI | 비교 실행, 고스트 overlay | policy ghost pack | 닫힘-동작 |"])
    require_tokens(GUIDE, ["#### 파-3", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `social_world_econ_3_v1` |"])
    require_tokens(TRACKER, ["| 41 | `파-3` | 정책 고스트 UI | 닫힘-동작 |", "| `파-3` | policy ghost UI | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `파-3` | `social_world_econ_3_v1`; UI `social_world_policy_ghost_ui.js`; runner `social_world_policy_ghost_ui_runner.mjs` |"])
    require_tokens(DEV_SURFACES, ["social_world_policy_ghost_ui.js", "__SOCIAL_WORLD_POLICY_GHOST_UI__"])
    require_tokens(INDEX_HTML, ["id=\"social-world-policy-ghost-ui\"", "data-social-world-policy-ghost-ui"])
    require_tokens(DEV_SURFACES_CSS, [".social-world-policy-ghost-ui", ".social-policy-artifacts", ".social-policy-preview"])


def check_status_closed() -> None:
    for line in read(MATRIX).splitlines():
        if "| 3마루 작업실마루 | 정책 고스트 UI |" in line:
            if line.rstrip().split("|")[-2].strip() != "닫힘-동작":
                fail(f"파-3 status must be 닫힘-동작: {line}")
            return
    fail("missing 파-3 matrix line")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 26,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 29,
        "roadmap_v2_pack_evidence_reference_closed": 46,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 51,
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
        "pack": "social_world_econ_3_v1",
        "kind": "roadmap_v2_pa3_policy_ghost_ui_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "PA3_POLICY_GHOST_UI_V1",
        "roadmap_coordinate": "파-3",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "pa3_matrix_status": "닫힘-동작",
        "requires_pa2_closed": True,
        "policy_ghost_ui_claim": True,
        "compare_run_claim": True,
        "ghost_overlay_claim": True,
        "classroom_compare_claim": True,
        "real_world_prediction_claim": False,
        "policy_advice_claim": False,
        "agent_simulation_execution_claim": False,
        "live_policy_deployment_claim": False,
        "state_hash_participation_claim": False,
        "parser_frontdoor_change": False,
        "grammar_claim": False,
        "current_stage": "PA3 policy ghost UI closure",
        "next_item": "PA4_SOCIAL_TEMPLATE_REGISTRY_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    check_payload(CONTRACT)
    policy = read_json(POLICY)
    if policy.get("status") != "social_world_policy_ghost_ready":
        fail(f"policy status={policy.get('status')!r}")
    ids = [row.get("id") for row in policy.get("rows", [])]
    if ids != ["baseline_run", "policy_variant", "ghost_overlay", "classroom_compare"]:
        fail(f"policy rows mismatch: {ids!r}")
    for token in ["coordinate:파-3", "real_world_prediction:false", "policy_advice:false", "agent_simulation_execution:false", "live_policy_deployment:false"]:
        if token not in str(policy.get("ghost_text", "")):
            fail(f"policy ghost text missing {token}")
    check_payload(POLICY)
    for payload in [contract, policy]:
        for key, value in payload.get("false_claims", {}).items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, POLICY, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 27",
            "Studio-local 초장기 계획: 10/18",
            "real_world_prediction_claim\": true",
            "policy_advice_claim\": true",
            "agent_simulation_execution_claim\": true",
            "live_policy_deployment_claim\": true",
            "state_hash_participation_claim\": true",
            "parser_frontdoor_change\": true",
            "grammar_claim\": true",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "social_world_econ_3_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_pa2_social_bridge_pack_check.py"], timeout=600)
    run(["node", "tests/social_world_policy_ghost_ui_runner.mjs"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_status_closed()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-pa3-policy-ghost-ui] OK")


if __name__ == "__main__":
    main()
