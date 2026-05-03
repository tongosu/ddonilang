#!/usr/bin/env python
from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "text_surface_registry_table_v1"


def fail(message: str) -> int:
    print(f"[text-surface-registry-table] fail: {message}", file=sys.stderr)
    return 1


def _load_contract() -> dict:
    return json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))


def _prefix_conflicts(names: list[str]) -> list[tuple[str, str]]:
    conflicts: list[tuple[str, str]] = []
    for left in sorted(set(names), key=lambda value: (len(value), value)):
        for right in sorted(set(names)):
            if left == right:
                continue
            if right.startswith(left):
                conflicts.append((left, right))
    return conflicts


def _check_required_docs(contract: dict) -> tuple[list[str], list[str]]:
    missing_paths: list[str] = []
    missing_tokens: list[str] = []
    doc_paths = [
        contract["design_doc"],
        contract["proposal_doc"],
        contract["status_report"],
    ]
    for rel in doc_paths:
        path = ROOT / rel
        if not path.exists():
            missing_paths.append(rel)
            continue
        text = path.read_text(encoding="utf-8")
        for token in contract["required_doc_tokens"]:
            if token not in text:
                missing_tokens.append(f"{rel}:{token}")
    for anchor in contract["ssot_anchors"]:
        path = ROOT / anchor["path"]
        if not path.exists():
            missing_paths.append(anchor["path"])
            continue
        text = path.read_text(encoding="utf-8")
        for token in anchor["tokens"]:
            if token not in text:
                missing_tokens.append(f"{anchor['path']}:{token}")
    return missing_paths, missing_tokens


def _check_forbidden_truth_phrases(contract: dict) -> list[str]:
    forbidden = [
        "runtime truth를 소유한다",
        "state_hash를 소유한다",
        "replay truth를 소유한다",
        "InputSnapshot truth를 소유한다",
    ]
    hits: list[str] = []
    for rel in [contract["design_doc"], contract["proposal_doc"], contract["status_report"]]:
        path = ROOT / rel
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8")
        for phrase in forbidden:
            if phrase in text:
                hits.append(f"{rel}:{phrase}")
    return hits


def main() -> int:
    contract = _load_contract()
    if contract.get("schema") != "ddn.text_surface_registry_table.contract.v1":
        return fail("schema mismatch")
    if not contract["policy"].get("prefix_free_global"):
        return fail("prefix_free_global must be true")
    if not contract["policy"].get("no_compat_shim"):
        return fail("no_compat_shim must be true")
    if contract["policy"].get("rich_markup_runtime_truth_owner") is not False:
        return fail("rich markup must not own runtime truth")
    if contract["policy"].get("implementation_change_allowed_in_this_pack") is not False:
        return fail("this pack must remain table/policy only")

    names: list[str] = []
    surfaces: list[str] = []
    for section in ["text_escape_entries", "universal_character_commands", "named_character_entries", "console_markup_commands"]:
        for entry in contract[section]:
            names.append(entry["command"])
            surfaces.append(entry["surface"])

    duplicate_names = sorted({name for name in names if names.count(name) > 1})
    duplicate_surfaces = sorted({surface for surface in surfaces if surfaces.count(surface) > 1})
    conflicts = _prefix_conflicts(names)
    if duplicate_names:
        return fail(f"duplicate command names: {duplicate_names}")
    if duplicate_surfaces:
        return fail(f"duplicate surfaces: {duplicate_surfaces}")
    if conflicts:
        return fail(f"prefix conflicts: {conflicts[:8]}")

    excluded_surfaces = {entry["surface"] for entry in contract["excluded_surfaces"]}
    active_excluded = sorted(excluded_surfaces.intersection(surfaces))
    if active_excluded:
        return fail(f"excluded surfaces are active: {active_excluded}")
    for banned in ["\\{", "\\}"]:
        if banned in surfaces:
            return fail(f"template brace escape must not be active: {banned}")

    missing_groups = sorted(
        set(["greek", "greek_upper", "math", "logic_set", "arrow", "punctuation", "symbol", "currency", "super_sub", "box"])
        - {entry["group"] for entry in contract["named_character_entries"]}
    )
    if missing_groups:
        return fail(f"named character groups missing: {missing_groups}")
    universal = contract.get("universal_character_commands", [])
    if len(universal) != 1:
        return fail("exactly one universal character command is required")
    if universal[0].get("coverage") != "all_unicode_scalar_values":
        return fail("universal character command must cover all Unicode scalar values")
    examples = universal[0].get("examples", [])
    if "\\문자{U+AC00}" not in examples or "\\문자{U+1F600}" not in examples:
        return fail("universal character examples must include BMP and non-BMP code points")

    missing_paths, missing_tokens = _check_required_docs(contract)
    if missing_paths:
        return fail(f"missing paths: {missing_paths}")
    if missing_tokens:
        return fail(f"missing tokens: {missing_tokens[:8]}")

    forbidden_hits = _check_forbidden_truth_phrases(contract)
    if forbidden_hits:
        return fail(f"forbidden ownership phrases: {forbidden_hits}")

    report = {
        "schema": "ddn.text_surface_registry_table.report.v1",
        "ok": True,
        "pack_id": contract["pack_id"],
        "counts": {
            "text_escape": len(contract["text_escape_entries"]),
            "universal_character_commands": len(contract["universal_character_commands"]),
            "named_character": len(contract["named_character_entries"]),
            "console_markup_commands": len(contract["console_markup_commands"]),
            "excluded_surfaces": len(contract["excluded_surfaces"]),
        },
        "policy": {
            "prefix_free_global": True,
            "no_compat_shim": True,
            "template_literal_brace": contract["policy"]["template_literal_brace"],
            "unknown_backslash_command": contract["policy"]["unknown_backslash_command"],
            "rich_markup_runtime_truth_owner": False,
            "implementation_change_allowed_in_this_pack": False,
        },
        "ssot_contrast_checked": True,
    }
    expected = json.loads((PACK / "expected" / "text_surface_registry_table.detjson").read_text(encoding="utf-8"))
    if report != expected:
        return fail(json.dumps(report, ensure_ascii=False, indent=2))
    print("[text-surface-registry-table] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
