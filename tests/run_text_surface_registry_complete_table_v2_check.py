#!/usr/bin/env python
from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
V1 = ROOT / "pack" / "text_surface_registry_table_v1" / "contract.detjson"
V2 = ROOT / "pack" / "text_surface_registry_table_v2" / "contract.detjson"
CONSOLE = ROOT / "pack" / "seamgrim_console_control_markup_v2" / "contract.detjson"
COMPLETE = ROOT / "pack" / "text_surface_registry_table_v2" / "complete_table.detjson"
COMPLETE_MD = ROOT / "docs" / "context" / "design" / "TEXT_SURFACE_REGISTRY_COMPLETE_TABLE_V2_20260430.md"


def fail(message: str) -> int:
    print(f"[text-surface-registry-complete-table-v2] fail: {message}", file=sys.stderr)
    return 1


def _codepoint(value: str) -> str | None:
    return f"U+{ord(value):04X}" if isinstance(value, str) and len(value) == 1 else None


def _expand_range(item: dict, kind: str) -> list[dict]:
    start = int(item["start"], 16)
    end = int(item["end"], 16)
    prefix = item.get("prefix") or item.get("alias_prefix")
    group = item.get("group")
    rows: list[dict] = []
    for code in range(start, end + 1):
        name = f"{prefix}{code:04X}"
        value = chr(code)
        if kind == "direct":
            rows.append(
                {
                    "command": name,
                    "surface": f"\\{name}",
                    "value": value,
                    "codepoint": f"U+{code:04X}",
                    "group": group,
                    "source": "range",
                }
            )
        else:
            rows.append(
                {
                    "alias": name,
                    "surface": f"\\문자{{{name}}}",
                    "value": value,
                    "codepoint": f"U+{code:04X}",
                    "group": group,
                    "source": "range",
                }
            )
    return rows


def _expected_complete() -> dict:
    v1 = json.loads(V1.read_text(encoding="utf-8"))
    v2 = json.loads(V2.read_text(encoding="utf-8"))
    console = json.loads(CONSOLE.read_text(encoding="utf-8"))

    text_escape = []
    for row in v1["text_escape_entries"]:
        text_escape.append(
            {
                "surface": row["surface"],
                "command": row["command"],
                "meaning": row["meaning"],
                "ssot_status": row["ssot_status"],
            }
        )

    direct = []
    for row in v2["direct_shortcut_exact"]:
        direct.append(
            {
                "command": row["command"],
                "surface": f"\\{row['command']}",
                "value": row["value"],
                "codepoint": _codepoint(row["value"]),
                "group": row["group"],
                "source": "exact",
            }
        )
    for item in v2["direct_shortcut_ranges"]:
        direct.extend(_expand_range(item, "direct"))

    aliases = []
    for row in v2["braced_alias_exact"]:
        aliases.append(
            {
                "alias": row["alias"],
                "surface": f"\\문자{{{row['alias']}}}",
                "value": row["value"],
                "codepoint": _codepoint(row["value"]),
                "group": "exact_alias",
                "source": "exact",
            }
        )
    for item in v2["braced_alias_ranges"]:
        aliases.extend(_expand_range(item, "alias"))

    return {
        "schema": "ddn.text_surface_registry_complete_table.v2",
        "generated_from": {
            "v1_contract": "pack/text_surface_registry_table_v1/contract.detjson",
            "v2_contract": "pack/text_surface_registry_table_v2/contract.detjson",
            "console_control_contract": "pack/seamgrim_console_control_markup_v2/contract.detjson",
        },
        "counts": {
            "text_escape": len(text_escape),
            "direct_shortcuts": len(direct),
            "braced_aliases": len(aliases),
            "console_control_commands": len(console["commands"]),
            "excluded_surfaces": len(v2["excluded_surfaces"]),
        },
        "policy": {
            "direct_shortcut_minimum": v2["policy"]["min_direct_shortcuts"],
            "braced_alias_minimum": v2["policy"]["min_braced_aliases"],
            "unicode_fallback": v2["policy"]["unicode_fallback"],
            "unicode_fallback_last_resort": v2["policy"]["unicode_fallback_last_resort"],
            "raw_ansi_escape_output": False,
            "no_compat_shim": True,
        },
        "text_escape_entries": text_escape,
        "universal_character_command": v2["universal_character_command"],
        "direct_shortcuts": direct,
        "braced_aliases": aliases,
        "console_control_commands": console["commands"],
        "console_control_arguments": console["arguments"],
        "excluded_surfaces": v2["excluded_surfaces"],
    }


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


def main() -> int:
    if not COMPLETE.exists():
        return fail("complete_table.detjson missing")
    if not COMPLETE_MD.exists():
        return fail("complete markdown table missing")

    actual = json.loads(COMPLETE.read_text(encoding="utf-8"))
    expected = _expected_complete()
    if actual != expected:
        return fail("complete table is not synchronized with v2 contracts")

    direct_names = [row["command"] for row in actual["direct_shortcuts"]]
    conflicts = _prefix_conflicts(direct_names + [actual["universal_character_command"]["command"]])
    if conflicts:
        return fail(f"prefix conflicts: {conflicts[:5]}")

    alias_names = [row["alias"] for row in actual["braced_aliases"]]
    if len(alias_names) != len(set(alias_names)):
        return fail("duplicate braced alias in complete table")

    md = COMPLETE_MD.read_text(encoding="utf-8")
    required_tokens = [
        "## Direct Shortcuts",
        "## Braced Aliases",
        "`\\아래반블록`",
        "`\\문자{아래반블록}`",
        "`\\커서{행=1,칸=1}`",
        "`\\행제어{지우기}`",
        "direct_shortcuts: 696",
        "braced_aliases: 1359",
    ]
    missing = [token for token in required_tokens if token not in md]
    if missing:
        return fail(f"complete markdown tokens missing: {missing}")

    print("[text-surface-registry-complete-table-v2] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
