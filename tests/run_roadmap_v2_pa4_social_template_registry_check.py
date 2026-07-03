#!/usr/bin/env python3
"""Validate PA4_SOCIAL_TEMPLATE_REGISTRY_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "PA4_SOCIAL_TEMPLATE_REGISTRY_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "파-4_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "social_world_econ_4_v1"
CONTRACT = PACK / "contract.detjson"
TEMPLATE = PACK / "template_registry.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "social_world_template_registry.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
UI_RUNNER = ROOT / "tests" / "social_world_template_registry_runner.mjs"
PREREQ_PA3 = ROOT / "tests" / "run_roadmap_v2_pa3_policy_ghost_ui_check.py"


def fail(message: str) -> None:
    print(f"[roadmap-v2-pa4-social-template-registry] FAIL: {message}", file=sys.stderr)
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
        TEMPLATE,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        UI_MODULE,
        APP,
        INDEX_HTML,
        STYLES,
        UI_RUNNER,
        PREREQ_PA3,
    ]:
        require_file(path)
    shared_tokens = [
        "PA4_SOCIAL_TEMPLATE_REGISTRY_V1",
        "PA4 social template registry closure 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 27/90 = 30%",
        "ROADMAP_V2 pack evidence 참고값: 47/90 = 52%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "PA5_SOCIAL_WORLD_LTS_V1",
        "public template publish",
        "network_registry_sync:false",
        "account_permission_change:false",
        "state_hash_participation:false",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(MATRIX, ["| 4마루 나눔마루 | 템플릿 공유 | 사회세계 template registry | social template pack | 닫힘-동작 |"])
    require_tokens(GUIDE, ["#### 파-4", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `social_world_econ_4_v1` |"])
    require_tokens(TRACKER, ["| 42 | `파-4` | 템플릿 공유 | 닫힘-동작 |", "| `파-4` | social template registry | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `파-4` | `social_world_econ_4_v1`; UI `social_world_template_registry.js`; runner `social_world_template_registry_runner.mjs` |"])
    require_tokens(DEV_SURFACES, ["social_world_template_registry.js", "__SOCIAL_WORLD_TEMPLATE_REGISTRY__"])
    require_tokens(INDEX_HTML, ["id=\"social-world-template-registry\"", "data-social-world-template-registry"])
    require_tokens(DEV_SURFACES_CSS, [".social-world-template-registry", ".social-template-artifacts", ".social-template-preview"])


def check_status_closed() -> None:
    for line in read(MATRIX).splitlines():
        if "| 4마루 나눔마루 | 템플릿 공유 |" in line:
            if line.rstrip().split("|")[-2].strip() != "닫힘-동작":
                fail(f"파-4 status must be 닫힘-동작: {line}")
            return
    fail("missing 파-4 matrix line")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 27,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 30,
        "roadmap_v2_pack_evidence_reference_closed": 47,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 52,
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
        "pack": "social_world_econ_4_v1",
        "kind": "roadmap_v2_pa4_social_template_registry_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "PA4_SOCIAL_TEMPLATE_REGISTRY_V1",
        "roadmap_coordinate": "파-4",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "pa4_matrix_status": "닫힘-동작",
        "requires_pa3_closed": True,
        "social_template_registry_claim": True,
        "template_catalog_claim": True,
        "share_snapshot_claim": True,
        "remix_contract_claim": True,
        "classroom_registry_claim": True,
        "public_template_publish_claim": False,
        "network_registry_sync_claim": False,
        "account_permission_change_claim": False,
        "policy_advice_claim": False,
        "state_hash_participation_claim": False,
        "parser_frontdoor_change": False,
        "grammar_claim": False,
        "current_stage": "PA4 social template registry closure",
        "next_item": "PA5_SOCIAL_WORLD_LTS_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    check_payload(CONTRACT)
    template = read_json(TEMPLATE)
    if template.get("status") != "social_world_template_registry_ready":
        fail(f"template status={template.get('status')!r}")
    ids = [row.get("id") for row in template.get("rows", [])]
    if ids != ["template_catalog", "share_snapshot", "remix_contract", "classroom_registry"]:
        fail(f"template rows mismatch: {ids!r}")
    for token in ["coordinate:파-4", "public_template_publish:false", "network_registry_sync:false", "account_permission_change:false", "state_hash_participation:false"]:
        if token not in str(template.get("template_text", "")):
            fail(f"template text missing {token}")
    check_payload(TEMPLATE)
    for payload in [contract, template]:
        for key, value in payload.get("false_claims", {}).items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, TEMPLATE, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 28",
            "Studio-local 초장기 계획: 10/18",
            "public_template_publish_claim\": true",
            "network_registry_sync_claim\": true",
            "account_permission_change_claim\": true",
            "policy_advice_claim\": true",
            "state_hash_participation_claim\": true",
            "parser_frontdoor_change\": true",
            "grammar_claim\": true",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "social_world_econ_4_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_pa3_policy_ghost_ui_check.py"], timeout=600)
    run(["node", "tests/social_world_template_registry_runner.mjs"], timeout=240)
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
    print("[roadmap-v2-pa4-social-template-registry] OK")


if __name__ == "__main__":
    main()
