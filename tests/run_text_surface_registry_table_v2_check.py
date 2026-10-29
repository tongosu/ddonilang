#!/usr/bin/env python
from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "text_surface_registry_table_v2"


def fail(message: str) -> int:
    print(f"[text-surface-registry-table-v2] fail: {message}", file=sys.stderr)
    return 1


def _hex_range_count(item: dict) -> int:
    return int(item["end"], 16) - int(item["start"], 16) + 1


def _hex_range_names(item: dict) -> list[str]:
    start = int(item["start"], 16)
    end = int(item["end"], 16)
    prefix = item.get("prefix") or item.get("alias_prefix")
    return [f"{prefix}{code:04X}" for code in range(start, end + 1)]


def _prefix_conflicts(names: list[str]) -> list[tuple[str, str]]:
    unique = sorted(set(names), key=lambda value: (len(value), value))
    conflicts: list[tuple[str, str]] = []
    for idx, left in enumerate(unique):
        for right in unique[idx + 1 :]:
            if right.startswith(left):
                conflicts.append((left, right))
                if len(conflicts) >= 20:
                    return conflicts
    return conflicts


def _read_text(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def main() -> int:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.text_surface_registry_table_v2.contract.v1":
        return fail("schema mismatch")
    policy = contract["policy"]
    if not policy.get("unicode_fallback_last_resort"):
        return fail("unicode fallback must be marked as last resort")
    if policy.get("raw_ansi_escape_output") is not False:
        return fail("raw ANSI output must be disabled")
    if policy.get("removed_surface_migration") != "diagnostic_fixit_only":
        return fail("removed surfaces must migrate through diagnostic + fix-it")
    if policy.get("runtime_compat_shim_for_removed_surface") is not False:
        return fail("removed surfaces must not keep runtime compat shim")
    if policy.get("template_scanner_ignores_registered_braced_command_payload") is not True:
        return fail("template scanner must ignore registered braced command payload")
    if policy.get("implementation_change_allowed_in_this_pack") is not False:
        return fail("table v2 pack must not change implementation behavior")
    roles = contract.get("document_roles", {})
    required_roles = {
        "active_proposal_table",
        "active_machine_contract",
        "generated_review_artifact",
        "generated_machine_artifact",
        "archive_rationale",
        "ssot_patch_proposal",
    }
    if required_roles - set(roles):
        return fail(f"document roles missing: {sorted(required_roles - set(roles))}")
    fixits = {entry["from"]: entry["to"] for entry in contract.get("removed_surface_fixits", [])}
    required_fixits = {
        "\\따옴": "\\\"",
        "\\역빗금": "\\\\",
        "\\굵게": "\\스타일{굵게}",
        "\\굵게끝": "\\스타일{보통}",
        "\\반전": "\\스타일{반전}",
    }
    for old, new in required_fixits.items():
        if fixits.get(old) != new:
            return fail(f"fix-it missing: {old} -> {new}")
    if "\\반전끝" not in fixits or "\\되돌림" not in fixits["\\반전끝"]:
        return fail("fix-it missing: \\반전끝 -> ... \\되돌림")
    template_policy = contract.get("template_braced_command_policy", {})
    if template_policy.get("placeholder_scanner_ignores_registered_braced_command_payload") is not True:
        return fail("template braced command policy missing")
    templates = {entry["template"]: entry["placeholders"] for entry in template_policy.get("examples", [])}
    if templates.get("글무늬{\"\\문자{알파} = {값}\"}") != ["값"]:
        return fail("template example for \\문자 payload is missing")
    if templates.get("글무늬{\"\\색{빨강}경고 {이름}\\되돌림\"}") != ["이름"]:
        return fail("template example for \\색 payload is missing")
    if templates.get("글무늬{\"\\커서{행=1,칸=1}{문장}\"}") != ["문장"]:
        return fail("template example for \\커서 payload is missing")

    direct_names = [entry["command"] for entry in contract["direct_shortcut_exact"]]
    for item in contract["direct_shortcut_ranges"]:
        direct_names.extend(_hex_range_names(item))
    if len(direct_names) != len(set(direct_names)):
        return fail("duplicate direct shortcut commands")
    conflicts = _prefix_conflicts(direct_names + [contract["universal_character_command"]["command"]])
    if conflicts:
        return fail(f"prefix conflicts: {conflicts[:5]}")
    if len(direct_names) < int(policy["min_direct_shortcuts"]):
        return fail(f"direct shortcuts too small: {len(direct_names)}")

    alias_names = [entry["alias"] for entry in contract["braced_alias_exact"]]
    for item in contract["braced_alias_ranges"]:
        alias_names.extend(_hex_range_names(item))
    if len(alias_names) != len(set(alias_names)):
        return fail("duplicate braced aliases")
    if len(alias_names) < int(policy["min_braced_aliases"]):
        return fail(f"braced aliases too small: {len(alias_names)}")

    universal = contract["universal_character_command"]
    if universal.get("coverage") != "all_unicode_scalar_values":
        return fail("universal command must cover all Unicode scalar values")
    examples = set(universal.get("examples", []))
    if {"\\문자{U+AC00}", "\\문자{U+1F600}", "\\문자{알파}"} - examples:
        return fail("universal command examples missing")

    required_groups = {
        "greek",
        "greek_upper",
        "math",
        "geometry",
        "logic_set",
        "arrow",
        "punctuation",
        "symbol",
        "currency",
        "block_named",
    }
    exact_groups = {entry["group"] for entry in contract["direct_shortcut_exact"]}
    range_groups = {entry["group"] for entry in contract["direct_shortcut_ranges"]}
    missing_groups = sorted(required_groups - exact_groups - range_groups)
    if missing_groups:
        return fail(f"missing direct shortcut groups: {missing_groups}")

    excluded = {entry["surface"] for entry in contract["excluded_surfaces"]}
    if "\\줄{...}" not in excluded:
        return fail("\\줄{...} exclusion required because \\줄 is text escape")
    required_block_names = {"아래반블록", "위반블록", "왼반블록", "오른반블록", "전체블록", "연한그늘", "중간그늘", "진한그늘"}
    if required_block_names - set(direct_names):
        return fail(f"human-friendly block shortcuts missing: {sorted(required_block_names - set(direct_names))}")

    console_pack = ROOT / contract["console_control_pack"]
    if not console_pack.exists():
        return fail(f"console control pack missing: {contract['console_control_pack']}")
    console = json.loads(console_pack.read_text(encoding="utf-8"))
    console_names = [entry["command"] for entry in console["commands"]]
    if _prefix_conflicts(console_names + ["줄"]):
        return fail("console command names conflict with text escape/direct names")
    if console["policy"].get("raw_ansi_escape_output") is not False:
        return fail("console control pack must ban raw ANSI output")
    if console["policy"].get("render_session_scope") != "console_render_session_artifact":
        return fail("console control scope must be console render session artifact")
    if console["policy"].get("output_entry_style_reset") is not True:
        return fail("console style must reset per output entry")
    if console["policy"].get("accessibility_can_disable_motion") is not True:
        return fail("motion effects must allow accessibility disable")
    if "줄제어" in console_names:
        return fail("console command names must not include 줄제어")
    console_excluded = {entry["surface"] for entry in console.get("excluded_surfaces", [])}
    if "\\줄제어{지우기}" not in console_excluded:
        return fail("console control pack must exclude \\줄제어{지우기}")

    missing_doc_tokens: list[str] = []
    for rel in [contract["design_doc"], contract["proposal_doc"], contract["status_report"]]:
        text = _read_text(rel)
        for token in contract["required_doc_tokens"]:
            if token not in text:
                missing_doc_tokens.append(f"{rel}:{token}")
    if missing_doc_tokens:
        return fail(f"missing doc tokens: {missing_doc_tokens[:8]}")

    report = {
        "schema": "ddn.text_surface_registry_table_v2.report.v1",
        "ok": True,
        "pack_id": contract["pack_id"],
        "direct_shortcut_count": len(direct_names),
        "braced_alias_count": len(alias_names),
        "console_control_commands": len(console["commands"]),
        "unicode_fallback": contract["policy"]["unicode_fallback"],
        "unicode_fallback_last_resort": True,
        "raw_ansi_escape_output": False,
        "removed_surface_migration": "diagnostic_fixit_only",
        "implementation_change_allowed_in_this_pack": False,
    }
    expected = json.loads((PACK / "expected" / "text_surface_registry_table_v2.detjson").read_text(encoding="utf-8"))
    if report != expected:
        return fail(json.dumps(report, ensure_ascii=False, indent=2))
    print("[text-surface-registry-table-v2] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
