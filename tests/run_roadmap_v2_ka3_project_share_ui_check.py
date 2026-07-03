#!/usr/bin/env python3
"""Validate KA3_PROJECT_SHARE_UI_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "KA3_PROJECT_SHARE_UI_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "카-3_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "ttonimaru_registry_3_v1"
CONTRACT = PACK / "contract.detjson"
PROJECT_SHARE = PACK / "project_share_ui.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "ttonimaru_project_share_ui.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
UI_RUNNER = ROOT / "tests" / "ttonimaru_project_share_ui_runner.mjs"
PREREQ_CHECK = ROOT / "tests" / "run_roadmap_v2_ka2_publication_read_api_check.py"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ka3-project-share-ui] FAIL: {message}", file=sys.stderr)
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
        PROJECT_SHARE,
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
        "KA3_PROJECT_SHARE_UI_V1",
        "KA3 project/share UI closure 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 19/90 = 21%",
        "ROADMAP_V2 pack evidence 참고값: 39/90 = 43%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "KA4_PUBLIC_REGISTRY_SEED_V1",
        "public registry seed",
        "account permission",
        "cloud sync",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 3마루 작업실마루 | project/share UI | share link, revision pin | share UI pack | 닫힘-동작 |",
        ],
    )
    require_tokens(
        GUIDE,
        [
            "#### 카-3",
            "| 현재 상태 | 닫힘-동작 |",
            "| pack 후보 | `ttonimaru_registry_3_v1` |",
        ],
    )
    require_tokens(
        TRACKER,
        [
            "| 34 | `카-3` | 또니마루 project/share UI | 닫힘-동작 |",
            "| `카-3` | project/share UI | 닫힘-동작 |",
        ],
    )
    require_tokens(
        MANIFEST,
        [
            "| `카-3` | `ttonimaru_registry_3_v1`; UI `ttonimaru_project_share_ui.js`; runner `ttonimaru_project_share_ui_runner.mjs` |",
        ],
    )
    require_tokens(DEV_SURFACES, [
            "ttonimaru_project_share_ui.js",
            "__TTONIMARU_PROJECT_SHARE_UI__",
        ],
    )
    require_tokens(INDEX_HTML, ["id=\"ttonimaru-project-share-ui\"", "data-ttonimaru-project-share-ui"])
    require_tokens(DEV_SURFACES_CSS, [".ttonimaru-project-share-ui", ".ttonimaru-share-artifacts", ".ttonimaru-share-preview"])


def check_ka3_status_closed() -> None:
    matrix_line = ""
    for line in read(MATRIX).splitlines():
        if "| 3마루 작업실마루 | project/share UI |" in line:
            matrix_line = line
            break
    if not matrix_line:
        fail("missing 카-3 matrix line")
    status_cell = matrix_line.rstrip().split("|")[-2].strip()
    if status_cell != "닫힘-동작":
        fail(f"카-3 status must be 닫힘-동작: {matrix_line}")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 19,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 21,
        "roadmap_v2_pack_evidence_reference_closed": 39,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 43,
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
        "pack": "ttonimaru_registry_3_v1",
        "kind": "roadmap_v2_ka3_project_share_ui_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "KA3_PROJECT_SHARE_UI_V1",
        "roadmap_coordinate": "카-3",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "ka3_matrix_status": "닫힘-동작",
        "requires_ka2_closed": True,
        "requires_browser_runner_evidence": True,
        "project_share_ui_claim": True,
        "project_snapshot_claim": True,
        "revision_pin_claim": True,
        "share_link_claim": True,
        "remix_handoff_claim": True,
        "current_stage": "KA3 project/share UI closure",
        "next_item": "KA4_PUBLIC_REGISTRY_SEED_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    if contract.get("share_artifacts") != ["project_snapshot", "revision_pin", "share_link", "remix_handoff"]:
        fail(f"contract share_artifacts={contract.get('share_artifacts')!r}")
    check_payload(CONTRACT)

    project_share = read_json(PROJECT_SHARE)
    if project_share.get("status") != "ttonimaru_project_share_ui_ready":
        fail(f"project_share status={project_share.get('status')!r}")
    if project_share.get("matrix_closure_tier") != "닫힘-동작":
        fail("project_share must be 닫힘-동작")
    for key in [
        "project_share_ui_claim",
        "project_snapshot_claim",
        "revision_pin_claim",
        "share_link_claim",
        "remix_handoff_claim",
    ]:
        if project_share.get(key) is not True:
            fail(f"project_share {key} must be true")
    share_ids = [row.get("id") for row in project_share.get("share_rows", [])]
    if share_ids != ["project_snapshot", "revision_pin", "share_link", "remix_handoff"]:
        fail(f"share rows mismatch: {share_ids!r}")
    share_text = str(project_share.get("share_text", ""))
    for token in ["coordinate:카-3", "public_registry_seed:false", "registry_publish:false", "cloud_sync:false"]:
        if token not in share_text:
            fail(f"share text missing {token}")
    check_payload(PROJECT_SHARE)
    for payload in [contract, project_share]:
        false_claims = payload.get("false_claims", {})
        for key, value in false_claims.items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, PROJECT_SHARE, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 20",
            "Studio-local 초장기 계획: 10/18",
            "KA4_PUBLIC_REGISTRY_SEED_V1 PASS 없이 닫힘",
            "public_registry_seed_claim\": true",
            "public_registry_final_claim\": true",
            "registry_publish_claim\": true",
            "install_update_remove_claim\": true",
            "trust_signing_claim\": true",
            "team_membership_claim\": true",
            "account_permission_claim\": true",
            "cloud_sync_claim\": true",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "ttonimaru_registry_3_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_ka2_publication_read_api_check.py"], timeout=600)
    run(["node", "tests/ttonimaru_project_share_ui_runner.mjs"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_ka3_status_closed()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ka3-project-share-ui] OK")


if __name__ == "__main__":
    main()
