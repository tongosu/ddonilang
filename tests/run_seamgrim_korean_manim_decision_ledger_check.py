#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LEDGER = ROOT / "docs" / "context" / "reports" / "SEAMGRIM_KOREAN_MANIM_DECISION_LEDGER_V1_20260711.detjson"

VALID = {
    "source_status": {"current_line", "candidate", "superseded"},
    "reflection_status": {"reflected", "partial", "not_reflected"},
    "implementation_status": {"behavior_landed", "artifact_landed", "partial", "not_started"},
    "owner_decision_status": {"accepted", "direction_accepted", "pending", "rejected"},
}


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def git_commit_exists(ref: str) -> bool:
    proc = subprocess.run(
        ["git", "cat-file", "-e", f"{ref}^{{commit}}"],
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return proc.returncode == 0


def git_path_is_tracked(path: str) -> bool:
    proc = subprocess.run(
        ["git", "ls-files", "--error-unmatch", "--", path],
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return proc.returncode == 0


def git_path_is_staged(path: str) -> bool:
    proc = subprocess.run(
        ["git", "diff", "--cached", "--quiet", "--", path],
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return proc.returncode == 1


def main() -> int:
    if not LEDGER.exists():
        return fail("E_KM_LEDGER_MISSING", str(LEDGER.relative_to(ROOT)))

    try:
        payload = json.loads(LEDGER.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return fail("E_KM_LEDGER_JSON", str(exc))

    if payload.get("schema") != "seamgrim.korean_manim.decision_ledger.v1":
        return fail("E_KM_LEDGER_SCHEMA", repr(payload.get("schema")))

    items = payload.get("items")
    if not isinstance(items, list) or not items:
        return fail("E_KM_LEDGER_ITEMS", "items must be a non-empty list")

    seen: set[str] = set()
    for item in items:
        if not isinstance(item, dict):
            return fail("E_KM_LEDGER_ITEM_TYPE", repr(item))
        item_id = str(item.get("id", "")).strip()
        if not item_id or item_id in seen:
            return fail("E_KM_LEDGER_ITEM_ID", repr(item_id))
        seen.add(item_id)

        for axis, allowed in VALID.items():
            value = item.get(axis)
            if value not in allowed:
                return fail("E_KM_LEDGER_AXIS", f"{item_id} {axis}={value!r}")

        if not str(item.get("statement", "")).strip():
            return fail("E_KM_LEDGER_STATEMENT", item_id)
        if not str(item.get("claim_boundary", "")).strip():
            return fail("E_KM_LEDGER_CLAIM_BOUNDARY", item_id)
        if not str(item.get("next_action", "")).strip():
            return fail("E_KM_LEDGER_NEXT_ACTION", item_id)

        source_refs = item.get("source_refs", [])
        if not source_refs:
            return fail("E_KM_LEDGER_SOURCE_REF", item_id)
        for raw_path in source_refs:
            path = ROOT / str(raw_path)
            if not path.exists():
                return fail("E_KM_LEDGER_SOURCE_MISSING", f"{item_id} {raw_path}")
            if (
                not str(raw_path).startswith("docs/ssot/")
                and not git_path_is_tracked(str(raw_path))
                and not git_path_is_staged(str(raw_path))
            ):
                return fail("E_KM_LEDGER_SOURCE_UNTRACKED", f"{item_id} {raw_path}")

        anchors = item.get("reflection_anchors", [])
        if item["reflection_status"] == "reflected" and not anchors:
            return fail("E_KM_LEDGER_REFLECTION_EVIDENCE", item_id)
        for anchor in anchors:
            if not isinstance(anchor, dict):
                return fail("E_KM_LEDGER_ANCHOR_TYPE", f"{item_id} {anchor!r}")
            path = ROOT / str(anchor.get("path", ""))
            needle = str(anchor.get("contains", ""))
            if not path.exists() or not needle:
                return fail("E_KM_LEDGER_ANCHOR", f"{item_id} {anchor!r}")
            if needle not in path.read_text(encoding="utf-8"):
                return fail("E_KM_LEDGER_ANCHOR_DRIFT", f"{item_id} {path.relative_to(ROOT)}: {needle}")

        commits = [str(ref) for ref in item.get("commit_refs", [])]
        packs = [str(ref) for ref in item.get("pack_refs", [])]
        if item["implementation_status"] == "behavior_landed" and (not commits or not packs):
            return fail("E_KM_LEDGER_IMPLEMENTATION_EVIDENCE", item_id)
        for ref in commits:
            if not git_commit_exists(ref):
                return fail("E_KM_LEDGER_COMMIT_MISSING", f"{item_id} {ref}")
        for raw_path in packs:
            path = ROOT / raw_path
            if not path.is_dir() or not (path / "contract.detjson").exists():
                return fail("E_KM_LEDGER_PACK_MISSING", f"{item_id} {raw_path}")

        decisions = item.get("decision_refs", [])
        if item["owner_decision_status"] in {"accepted", "direction_accepted"} and not decisions:
            return fail("E_KM_LEDGER_DECISION_EVIDENCE", item_id)
        if item["owner_decision_status"] == "pending" and decisions:
            return fail("E_KM_LEDGER_PENDING_WITH_DECISION", item_id)

    print(f"[seamgrim-korean-manim-decision-ledger-v1] OK items={len(items)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
