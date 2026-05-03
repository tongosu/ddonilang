#!/usr/bin/env python
from __future__ import annotations

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_console_control_markup_v2"


COMMAND_RE = re.compile(r"\\([가-힣A-Za-z_][가-힣A-Za-z0-9_]*)(?:\{([^}]*)\})?")


def fail(message: str) -> int:
    print(f"[seamgrim-console-control-markup-v2] fail: {message}", file=sys.stderr)
    return 1


def _prefix_conflicts(names: list[str]) -> list[tuple[str, str]]:
    unique = sorted(set(names), key=lambda value: (len(value), value))
    conflicts: list[tuple[str, str]] = []
    for idx, left in enumerate(unique):
        for right in unique[idx + 1 :]:
            if right.startswith(left):
                conflicts.append((left, right))
    return conflicts


def _parse_args(raw: str | None) -> dict[str, str]:
    if not raw:
        return {}
    result: dict[str, str] = {}
    for part in raw.split(","):
        part = part.strip()
        if not part:
            continue
        if "=" in part:
            key, value = part.split("=", 1)
            result[key.strip()] = value.strip()
        else:
            result["_"] = part
    return result


def _ops_and_plain(source: str, commands: dict[str, dict]) -> tuple[list[str], str]:
    ops: list[str] = []
    plain_parts: list[str] = []
    cursor = 0
    for match in COMMAND_RE.finditer(source):
        if match.start() > cursor:
            text = source[cursor : match.start()]
            plain_parts.append(text)
            if text:
                ops.append("text")
        name = match.group(1)
        args = _parse_args(match.group(2))
        if name not in commands:
            raise ValueError(f"unknown command: {name}")
        kind = commands[name]["kind"]
        if name == "색":
            ops.append("style.foreground")
        elif name == "배경":
            ops.append("style.background")
        elif name == "스타일":
            ops.append("style.effect")
        elif name == "되돌림":
            ops.append("style.reset")
        elif name == "커서":
            if "행" not in args or "칸" not in args:
                raise ValueError("cursor absolute requires 행 and 칸")
            ops.append("cursor.absolute")
        elif name == "이동":
            if not set(args).intersection({"위", "아래", "왼", "오른"}):
                raise ValueError("relative move requires direction")
            ops.append("cursor.relative")
        elif name == "위치저장":
            ops.append("cursor.save")
        elif name == "위치복원":
            ops.append("cursor.restore")
        elif name == "화면":
            if args.get("_") != "지우기":
                raise ValueError("screen action must be 지우기")
            ops.append("screen.clear")
        elif name == "행제어":
            if args.get("_") == "지우기":
                ops.append("screen.clear_line")
            elif args.get("_") == "끝까지지우기":
                ops.append("screen.clear_line_to_end")
            else:
                raise ValueError("line action mismatch")
        elif name == "움직임":
            if args.get("_") == "끄기":
                ops.append("motion.stop")
            else:
                ops.append("motion.start")
        else:
            ops.append(kind)
        cursor = match.end()
    if cursor < len(source):
        text = source[cursor:]
        plain_parts.append(text)
        if text:
            ops.append("text")
    dedup_ops: list[str] = []
    for op in ops:
        if op == "text" and dedup_ops and dedup_ops[-1] == "text":
            continue
        dedup_ops.append(op)
    return dedup_ops, "".join(plain_parts)


def main() -> int:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.seamgrim_console_control_markup.contract.v2":
        return fail("schema mismatch")
    if contract["policy"].get("raw_ansi_escape_output") is not False:
        return fail("raw ANSI output must be false")
    if contract["policy"].get("runtime_truth_owner") is not False:
        return fail("console control must not own runtime truth")
    if contract["policy"].get("render_session_scope") != "console_render_session_artifact":
        return fail("console control must be scoped to render session artifact")
    if contract["policy"].get("raw_string_preserved") is not True:
        return fail("console control must preserve raw string")
    if contract["policy"].get("output_entry_style_reset") is not True:
        return fail("console control must reset style per output entry")
    if contract["policy"].get("cursor_screen_line_scope") != "console_view_session_artifact_only":
        return fail("cursor/screen/line control scope mismatch")
    if contract["policy"].get("mirror_result_split") != "raw_text_and_render_summary":
        return fail("mirror/result split policy missing")
    if contract["policy"].get("motion_effect_optional") is not True:
        return fail("motion effect must be optional")
    if contract["policy"].get("accessibility_can_disable_motion") is not True:
        return fail("accessibility must be allowed to disable motion")
    commands = {entry["command"]: entry for entry in contract["commands"]}
    names = list(commands)
    conflicts = _prefix_conflicts(names + ["줄"])
    if conflicts:
        return fail(f"prefix conflicts: {conflicts}")
    if "줄" in names or "줄제어" in names or contract["policy"].get("line_control_command") != "행제어":
        return fail("line control command must be 행제어, not 줄/줄제어")
    excluded = {entry["surface"] for entry in contract["excluded_surfaces"]}
    if "\\줄{지우기}" not in excluded:
        return fail("excluded \\줄{지우기} missing")
    if "\\줄제어{지우기}" not in excluded:
        return fail("excluded \\줄제어{지우기} missing")

    for example in contract["examples"]:
        ops, plain = _ops_and_plain(example["source"], commands)
        if ops != example["expected_ops"]:
            return fail(f"{example['id']} ops mismatch: {ops}")
        if plain != example["plain_text"]:
            return fail(f"{example['id']} plain text mismatch: {plain!r}")

    report = {
        "schema": "ddn.seamgrim_console_control_markup.report.v2",
        "ok": True,
        "pack_id": contract["pack_id"],
        "command_count": len(contract["commands"]),
        "raw_ansi_escape_output": False,
        "runtime_truth_owner": False,
        "line_control_command": "행제어",
        "render_session_scope": "console_render_session_artifact",
        "output_entry_style_reset": True,
        "accessibility_can_disable_motion": True,
        "examples_checked": len(contract["examples"]),
    }
    expected = json.loads((PACK / "expected" / "console_control_markup.detjson").read_text(encoding="utf-8"))
    if report != expected:
        return fail(json.dumps(report, ensure_ascii=False, indent=2))
    print("[seamgrim-console-control-markup-v2] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
