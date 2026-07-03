#!/usr/bin/env python3
"""Validate KA4_PUBLIC_REGISTRY_SEED_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "KA4_PUBLIC_REGISTRY_SEED_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "카-4_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "ttonimaru_registry_4_v1"
CONTRACT = PACK / "contract.detjson"
REGISTRY_SEED = PACK / "public_registry_seed.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "ttonimaru_public_registry_seed.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
UI_RUNNER = ROOT / "tests" / "ttonimaru_public_registry_seed_runner.mjs"
PREREQ_CHECK = ROOT / "tests" / "run_roadmap_v2_ka3_project_share_ui_check.py"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ka4-public-registry-seed] FAIL: {message}", file=sys.stderr)
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
        REGISTRY_SEED,
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
        "KA4_PUBLIC_REGISTRY_SEED_V1",
        "KA4 public registry seed closure 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 20/90 = 22%",
        "ROADMAP_V2 pack evidence 참고값: 40/90 = 44%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "KA5_PLATFORM_HARDENING_V1",
        "registry publish",
        "trust signing",
        "platform hardening",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(MATRIX, ["| 4마루 나눔마루 | public registry seed | trust/lineage/badge | registry seed pack | 닫힘-동작 |"])
    require_tokens(GUIDE, ["#### 카-4", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `ttonimaru_registry_4_v1` |"])
    require_tokens(TRACKER, ["| 35 | `카-4` | 또니마루 public registry seed | 닫힘-동작 |", "| `카-4` | public registry seed | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `카-4` | `ttonimaru_registry_4_v1`; UI `ttonimaru_public_registry_seed.js`; runner `ttonimaru_public_registry_seed_runner.mjs` |"])
    require_tokens(DEV_SURFACES, ["ttonimaru_public_registry_seed.js", "__TTONIMARU_PUBLIC_REGISTRY_SEED__"])
    require_tokens(INDEX_HTML, ["id=\"ttonimaru-public-registry-seed\"", "data-ttonimaru-public-registry-seed"])
    require_tokens(DEV_SURFACES_CSS, [".ttonimaru-public-registry-seed", ".ttonimaru-registry-artifacts", ".ttonimaru-registry-preview"])


def check_ka4_status_closed() -> None:
    for line in read(MATRIX).splitlines():
        if "| 4마루 나눔마루 | public registry seed |" in line:
            if line.rstrip().split("|")[-2].strip() != "닫힘-동작":
                fail(f"카-4 status must be 닫힘-동작: {line}")
            return
    fail("missing 카-4 matrix line")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 20,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 22,
        "roadmap_v2_pack_evidence_reference_closed": 40,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 44,
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
        "pack": "ttonimaru_registry_4_v1",
        "kind": "roadmap_v2_ka4_public_registry_seed_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "KA4_PUBLIC_REGISTRY_SEED_V1",
        "roadmap_coordinate": "카-4",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "ka4_matrix_status": "닫힘-동작",
        "requires_ka3_closed": True,
        "requires_browser_runner_evidence": True,
        "public_registry_seed_claim": True,
        "seed_catalog_claim": True,
        "lineage_record_claim": True,
        "trust_badge_claim": True,
        "registry_preview_claim": True,
        "current_stage": "KA4 public registry seed closure",
        "next_item": "KA5_PLATFORM_HARDENING_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    if contract.get("registry_artifacts") != ["seed_catalog", "lineage_record", "trust_badge", "seed_preview"]:
        fail(f"contract registry_artifacts={contract.get('registry_artifacts')!r}")
    check_payload(CONTRACT)

    seed = read_json(REGISTRY_SEED)
    if seed.get("status") != "ttonimaru_public_registry_seed_ready":
        fail(f"registry seed status={seed.get('status')!r}")
    if seed.get("matrix_closure_tier") != "닫힘-동작":
        fail("registry seed must be 닫힘-동작")
    for key in ["public_registry_seed_claim", "seed_catalog_claim", "lineage_record_claim", "trust_badge_claim", "registry_preview_claim"]:
        if seed.get(key) is not True:
            fail(f"registry seed {key} must be true")
    row_ids = [row.get("id") for row in seed.get("registry_rows", [])]
    if row_ids != ["seed_catalog", "lineage_record", "trust_badge", "seed_preview"]:
        fail(f"registry rows mismatch: {row_ids!r}")
    registry_text = str(seed.get("registry_text", ""))
    for token in ["coordinate:카-4", "registry_publish:false", "trust_signing:false", "cloud_sync:false"]:
        if token not in registry_text:
            fail(f"registry text missing {token}")
    check_payload(REGISTRY_SEED)
    for payload in [contract, seed]:
        for key, value in payload.get("false_claims", {}).items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, REGISTRY_SEED, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 21",
            "Studio-local 초장기 계획: 10/18",
            "KA5_PLATFORM_HARDENING_V1 PASS 없이 닫힘",
            "public_registry_final_claim\": true",
            "registry_publish_claim\": true",
            "install_update_remove_claim\": true",
            "trust_signing_claim\": true",
            "moderation_claim\": true",
            "team_membership_claim\": true",
            "account_permission_claim\": true",
            "cloud_sync_claim\": true",
            "platform_hardening_claim\": true",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "ttonimaru_registry_4_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_ka3_project_share_ui_check.py"], timeout=600)
    run(["node", "tests/ttonimaru_public_registry_seed_runner.mjs"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_ka4_status_closed()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ka4-public-registry-seed] OK")


if __name__ == "__main__":
    main()
