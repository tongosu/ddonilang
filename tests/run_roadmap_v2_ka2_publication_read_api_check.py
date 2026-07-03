#!/usr/bin/env python3
"""Validate KA2_PUBLICATION_READ_API_CLOSURE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "KA2_PUBLICATION_READ_API_CLOSURE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "카-2_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "ttonimaru_registry_2_v1"
CONTRACT = PACK / "contract.detjson"
PUBLICATION_API = PACK / "publication_read_api.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "ttonimaru_publication_read_api.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
UI_RUNNER = ROOT / "tests" / "ttonimaru_publication_read_api_runner.mjs"
PREREQ_CHECK = ROOT / "tests" / "run_ttonimaru_registry_1_check.py"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ka2-publication-read-api] FAIL: {message}", file=sys.stderr)
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
        PUBLICATION_API,
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
        "KA2_PUBLICATION_READ_API_CLOSURE_V1",
        "KA2 publication/read API closure 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 18/90 = 20%",
        "ROADMAP_V2 pack evidence 참고값: 38/90 = 42%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "KA3_PROJECT_SHARE_UI_V1",
        "public registry final",
        "trust signing",
        "install/update/remove",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 2마루 닫힘마루 | publication/read API | immutable publication, read-only API | API pack | 닫힘-동작 |",
        ],
    )
    require_tokens(
        GUIDE,
        [
            "#### 카-2",
            "| 현재 상태 | 닫힘-동작 |",
            "| pack 후보 | `ttonimaru_registry_2_v1` |",
        ],
    )
    require_tokens(
        TRACKER,
        [
            "| 33 | `카-2` | 또니마루 publication/read API | 닫힘-동작 |",
            "| `카-2` | publication/read API | 닫힘-동작 |",
        ],
    )
    require_tokens(
        MANIFEST,
        [
            "| `카-2` | `ttonimaru_registry_2_v1`; UI `ttonimaru_publication_read_api.js`; runner `ttonimaru_publication_read_api_runner.mjs` |",
        ],
    )
    require_tokens(DEV_SURFACES, [
            "ttonimaru_publication_read_api.js",
            "__TTONIMARU_PUBLICATION_READ_API__",
        ],
    )
    require_tokens(INDEX_HTML, ["id=\"ttonimaru-publication-read-api\"", "data-ttonimaru-publication-read-api"])
    require_tokens(DEV_SURFACES_CSS, [".ttonimaru-publication-read-api", ".ttonimaru-publication-artifacts", ".ttonimaru-publication-preview"])


def check_ka2_status_closed() -> None:
    matrix_line = ""
    for line in read(MATRIX).splitlines():
        if "| 2마루 닫힘마루 | publication/read API |" in line:
            matrix_line = line
            break
    if not matrix_line:
        fail("missing 카-2 matrix line")
    status_cell = matrix_line.rstrip().split("|")[-2].strip()
    if status_cell != "닫힘-동작":
        fail(f"카-2 status must be 닫힘-동작: {matrix_line}")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 18,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 20,
        "roadmap_v2_pack_evidence_reference_closed": 38,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 42,
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
        "pack": "ttonimaru_registry_2_v1",
        "kind": "roadmap_v2_ka2_publication_read_api_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "KA2_PUBLICATION_READ_API_CLOSURE_V1",
        "roadmap_coordinate": "카-2",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "ka2_matrix_status": "닫힘-동작",
        "requires_ka1_closed": True,
        "requires_browser_runner_evidence": True,
        "publication_read_api_claim": True,
        "immutable_publication_claim": True,
        "read_only_api_claim": True,
        "revision_pin_claim": True,
        "package_metadata_read_claim": True,
        "current_stage": "KA2 publication/read API closure",
        "next_item": "KA3_PROJECT_SHARE_UI_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    if contract.get("api_artifacts") != ["api_contract", "read_fixture", "manifest_v1", "metadata_read"]:
        fail(f"contract api_artifacts={contract.get('api_artifacts')!r}")
    check_payload(CONTRACT)

    publication_api = read_json(PUBLICATION_API)
    if publication_api.get("status") != "ttonimaru_publication_read_api_ready":
        fail(f"publication_read_api status={publication_api.get('status')!r}")
    if publication_api.get("matrix_closure_tier") != "닫힘-동작":
        fail("publication_read_api must be 닫힘-동작")
    for key in [
        "publication_read_api_claim",
        "immutable_publication_claim",
        "read_only_api_claim",
        "revision_pin_claim",
        "package_metadata_read_claim",
    ]:
        if publication_api.get(key) is not True:
            fail(f"publication_api {key} must be true")
    endpoint_ids = [row.get("id") for row in publication_api.get("endpoints", [])]
    if endpoint_ids != ["publication_read", "manifest_read", "package_metadata", "alias_redirect"]:
        fail(f"endpoint rows mismatch: {endpoint_ids!r}")
    api_text = str(publication_api.get("api_text", ""))
    for token in ["coordinate:카-2", "public_registry_final:false", "mutation_api:false", "trust_signing:false"]:
        if token not in api_text:
            fail(f"api text missing {token}")
    check_payload(PUBLICATION_API)
    for payload in [contract, publication_api]:
        false_claims = payload.get("false_claims", {})
        for key, value in false_claims.items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, PUBLICATION_API, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 19",
            "Studio-local 초장기 계획: 10/18",
            "KA3_PROJECT_SHARE_UI_V1 PASS 없이 닫힘",
            "public_registry_final_claim\": true",
            "mutation_api_claim\": true",
            "registry_publish_claim\": true",
            "install_update_remove_claim\": true",
            "trust_signing_claim\": true",
            "team_membership_claim\": true",
            "cloud_sync_claim\": true",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "ttonimaru_registry_2_v1"], timeout=240)
    run([sys.executable, "tests/run_ttonimaru_registry_1_check.py"], timeout=240)
    run(["node", "tests/ttonimaru_publication_read_api_runner.mjs"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_ka2_status_closed()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ka2-publication-read-api] OK")


if __name__ == "__main__":
    main()
