#!/usr/bin/env python3
"""Validate SA4_ASSET_VIEW_SHARE_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "사-4_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_sa4_asset_view_share_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-sa4-asset-view-share-reconciliation] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    payload = json.loads(read(path))
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


def check_docs() -> None:
    for path in [
        MATRIX,
        TRACKER,
        MANIFEST,
        REPORT,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        RECONCILIATION,
        ROOT / "pack" / "roadmap_v2_sa3_game_preview_reconciliation_v1" / "contract.detjson",
        ROOT / "pack" / "bogae_asset_manifest_v1" / "manifest.detjson",
        ROOT / "pack" / "bogae_asset_manifest_v1" / "golden.jsonl",
        ROOT / "pack" / "bogae_bundle_v1" / "bogae.bundle" / "manifest.toml",
        ROOT / "pack" / "sa2_sprite_skin_minimum_v1" / "skin" / "sa2_skin.detjson",
        ROOT / "pack" / "std_grid_game_bogae_web_playback_closure_v1" / "contract.detjson",
    ]:
        require_file(path)
    require_tokens(MATRIX, ["| 4마루 나눔마루 | asset/view 공유 | 모양씨, asset registry | asset registry pack | 닫힘-동작 |"])
    require_tokens(TRACKER, ["| 16.2 | `사-4` | asset/view 공유 | 닫힘-동작 |", "사-4_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `사-4` | `roadmap_v2_sa3_game_preview_reconciliation_v1`; `bogae_asset_manifest_v1`; `bogae_bundle_v1`; `sa2_sprite_skin_minimum_v1`; `std_grid_game_bogae_web_playback_closure_v1`; `roadmap_v2_sa4_asset_view_share_reconciliation_v1` |"])
    shared = [
        "SA4_ASSET_VIEW_SHARE_RECONCILIATION_V1",
        "SA4 asset/view share reconciliation 4/4 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 71/90 = 79%",
        "ROADMAP_V2 docs-closed: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 73/90 = 81%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_SA5_RENDERER_HARDENING_RECONCILIATION_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 SA4 asset/view share reconciliation", "71/90 = 79%", "73/90 = 81%"])


def check_payload() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_sa4_asset_view_share_reconciliation_v1",
        "kind": "roadmap_v2_sa4_asset_view_share_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "SA4_ASSET_VIEW_SHARE_RECONCILIATION_V1",
        "roadmap_coordinate": "사-4",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "current_stage_closed": 4,
        "current_stage_total": 4,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 71,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 79,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 73,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 81,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "next_item": "ROADMAP_V2_SA5_RENDERER_HARDENING_RECONCILIATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("coordinate") != "사-4":
        fail("coordinate mismatch")
    if reconciliation.get("new_status") != "닫힘-동작":
        fail("new_status mismatch")
    if reconciliation.get("status") != "behavior_closed":
        fail("status mismatch")
    for payload in [contract, reconciliation]:
        for key, value in payload.get("false_claims", {}).items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_evidence_content() -> None:
    asset_manifest = read_json(ROOT / "pack" / "bogae_asset_manifest_v1" / "manifest.detjson")
    assets = asset_manifest.get("assets")
    if not isinstance(assets, list) or len(assets) != 1:
        fail("bogae asset manifest asset count mismatch")
    if not str(assets[0].get("id", "")).startswith("blake3:"):
        fail("bogae asset manifest id missing")
    asset_golden = read(ROOT / "pack" / "bogae_asset_manifest_v1" / "golden.jsonl")
    if "asset_manifest_hash=blake3:6f2b22e21e4a6936d889d569482af1d37b0c078d3e9495f4732d63cb1d774a1c" not in asset_golden:
        fail("bogae asset manifest golden hash mismatch")
    bundle_manifest = read(ROOT / "pack" / "bogae_bundle_v1" / "bogae.bundle" / "manifest.toml")
    for token in ["version = \"bogae_bundle_v1\"", "[[asset]]", "source = \"bundle.txt\""]:
        if token not in bundle_manifest:
            fail(f"bundle manifest missing {token}")
    skin = read_json(ROOT / "pack" / "sa2_sprite_skin_minimum_v1" / "skin" / "sa2_skin.detjson")
    symbols = skin.get("symbols")
    if not isinstance(symbols, list) or symbols[0].get("web", {}).get("asset_uri") != "assets/sa2_hero.svg":
        fail("skin web asset uri mismatch")
    playback = read_json(ROOT / "pack" / "std_grid_game_bogae_web_playback_closure_v1" / "contract.detjson")
    required_assets = playback.get("required_assets")
    if not isinstance(required_assets, list) or "viewer/index.html" not in required_assets or "viewer/viewer.js" not in required_assets:
        fail("playback required assets mismatch")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 72/90",
        '"roadmap_v2_matrix_behavior_closed": 72',
        '"public_registry_publish_claim": true',
        '"cloud_sync_claim": true',
        '"remote_asset_hosting_claim": true',
        '"cryptographic_signing_claim": true',
        '"production_editor_share_claim": true',
        '"parser_runtime_change_claim": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
    ]
    for path in [REPORT, PACK / "README.md", CONTRACT, RECONCILIATION]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_sa4_asset_view_share_reconciliation_v1"], timeout=120)
    run([sys.executable, "tests/run_roadmap_v2_sa3_game_preview_reconciliation_check.py"], timeout=900)
    run([sys.executable, "tests/run_pack_golden.py", "bogae_asset_manifest_v1", "bogae_bundle_v1"], timeout=300)
    run([sys.executable, "tests/run_sa2_sprite_skin_minimum_pack_check.py"], timeout=300)
    run([sys.executable, "tests/run_std_grid_game_bogae_web_playback_controls_check.py"], timeout=300)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_payload()
    check_evidence_content()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-sa4-asset-view-share-reconciliation] OK")


if __name__ == "__main__":
    main()
