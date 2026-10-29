#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACKET = ROOT / "docs/context/proposals/PROPOSAL_SEAMGRIM_KOREAN_MANIM_WAVE1_TO_WAVE2B_SSOT_PROMOTION_PACKET_V1_20260711.md"
LEDGER = ROOT / "docs/context/reports/SEAMGRIM_KOREAN_MANIM_PROMOTION_LEDGER_V2_20260711.detjson"

VALID = {
    "reflection_status": {"reflected", "partial", "not_reflected"},
    "implementation_status": {"behavior_landed", "artifact_landed", "partial", "in_progress", "not_started"},
    "owner_decision_status": {"accepted", "direction_accepted", "pending", "rejected"},
}


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def commit_exists(ref: str) -> bool:
    return subprocess.run(
        ["git", "cat-file", "-e", f"{ref}^{{commit}}"],
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    ).returncode == 0


def main() -> int:
    if not PACKET.is_file() or not LEDGER.is_file():
        return fail("E_KMP_FILE_MISSING", f"packet={PACKET.is_file()} ledger={LEDGER.is_file()}")

    packet_text = PACKET.read_text(encoding="utf-8")
    for token in (
        "Wave 1~2B SSOT 통합 승격 패킷",
        "SSOT_UPDATE_NOTES_v24.12.9.md",
        "SSOT_DECISIONS_v24.12.9.md",
        "SSOT_OPEN_ISSUES_v24.12.9.md",
        "SSOT_PLANS_v24.12.9.md",
        "SSOT_TOOLCHAIN_v24.12.9.md",
        "SSOT_TERMS_v24.12.9.md",
        "한국어 Manim 전체 완료",
    ):
        if token not in packet_text:
            return fail("E_KMP_PACKET_TOKEN", token)

    try:
        payload = json.loads(LEDGER.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return fail("E_KMP_LEDGER_JSON", str(exc))

    if payload.get("schema") != "seamgrim.korean_manim.promotion_ledger.v2":
        return fail("E_KMP_LEDGER_SCHEMA", repr(payload.get("schema")))
    if payload.get("packet_status") not in {"live_draft_waiting_wave2b_g8", "ready_for_owner_decision"}:
        return fail("E_KMP_PACKET_STATUS", repr(payload.get("packet_status")))
    if payload.get("wave2b_status") not in {"in_progress", "complete"}:
        return fail("E_KMP_WAVE2B_STATUS", repr(payload.get("wave2b_status")))
    if payload.get("packet_status") == "ready_for_owner_decision" and payload.get("wave2b_status") != "complete":
        return fail("E_KMP_PREMATURE_READY", "Wave 2B must be complete")

    items = payload.get("items")
    if not isinstance(items, list) or not items:
        return fail("E_KMP_ITEMS", "items must be non-empty")

    seen: set[str] = set()
    for item in items:
        item_id = str(item.get("id", ""))
        if not item_id or item_id in seen:
            return fail("E_KMP_ITEM_ID", repr(item_id))
        seen.add(item_id)
        for axis, allowed in VALID.items():
            if item.get(axis) not in allowed:
                return fail("E_KMP_AXIS", f"{item_id} {axis}={item.get(axis)!r}")
        if not str(item.get("claim_boundary", "")).strip():
            return fail("E_KMP_CLAIM_BOUNDARY", item_id)

        for raw_path in item.get("source_refs", []):
            if not (ROOT / raw_path).exists():
                return fail("E_KMP_SOURCE_MISSING", f"{item_id} {raw_path}")
        for anchor in item.get("reflection_anchors", []):
            path = ROOT / str(anchor.get("path", ""))
            needle = str(anchor.get("contains", ""))
            if not path.is_file() or not needle or needle not in path.read_text(encoding="utf-8"):
                return fail("E_KMP_ANCHOR_DRIFT", f"{item_id} {anchor!r}")
        if item["reflection_status"] == "reflected" and not item.get("reflection_anchors"):
            return fail("E_KMP_REFLECTION_EVIDENCE", item_id)

        commits = item.get("commit_refs", [])
        packs = item.get("pack_refs", [])
        product_paths = item.get("product_path_refs", [])
        if item["implementation_status"] == "behavior_landed" and not (commits and packs and product_paths):
            return fail("E_KMP_BEHAVIOR_EVIDENCE", item_id)
        for ref in commits:
            if not commit_exists(str(ref)):
                return fail("E_KMP_COMMIT_MISSING", f"{item_id} {ref}")
        for raw_path in packs:
            path = ROOT / raw_path
            if not path.is_dir() or not (path / "contract.detjson").is_file():
                return fail("E_KMP_PACK_MISSING", f"{item_id} {raw_path}")
        for raw_path in product_paths:
            if not (ROOT / raw_path).exists():
                return fail("E_KMP_PRODUCT_PATH_MISSING", f"{item_id} {raw_path}")

        if item["implementation_status"] in {"in_progress", "not_started"} and item.get("promotion_status", "").startswith("record_"):
            return fail("E_KMP_PREMATURE_PROMOTION", item_id)

    print(
        "[seamgrim-korean-manim-promotion-packet-v1] OK "
        f"items={len(items)} packet_status={payload['packet_status']} wave2b_status={payload['wave2b_status']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
