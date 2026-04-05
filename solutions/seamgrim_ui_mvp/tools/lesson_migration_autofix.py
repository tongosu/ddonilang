#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
DEFAULT_SCAN_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
PROMOTE_BACKUP_SUFFIX = ".before_age3_promote.bak"

RANGE_COMMENT_HINT_RE = re.compile(r"//\s*범위\s*\(")
RANGE_DECL_RE = re.compile(
    r"^(?P<prefix>\s*[A-Za-z_가-힣][A-Za-z0-9_가-힣.]*\s*(?::\s*[A-Za-z_가-힣][A-Za-z0-9_가-힣.]*)?\s*<-\s*)"
    r"(?P<init>.+?)\.\s*//\s*범위\s*\(\s*(?P<min>[^,\n\)]+)\s*,\s*(?P<max>[^,\n\)]+)\s*,\s*(?P<step>[^\)\n]+)\)\s*(?P<tail>//.*)?$"
)
SETUP_COLON_HEADER_RE = re.compile(r"^(?P<indent>\s*)채비\s*:\s*\{\s*(?P<tail>//.*)?$")
HOOK_START_COLON_RE = re.compile(r"^(?P<indent>\s*)\(\s*(?P<name>시작|처음)\s*\)\s*할때\s*:\s*\{\s*(?P<tail>//.*)?$")
HOOK_TICK_COLON_RE = re.compile(
    r"^(?P<indent>\s*)\(\s*(?P<name>(?:매마디|매틱)|(?:[1-9][0-9]*\s*마디))\s*\)\s*마다\s*:\s*\{\s*(?P<tail>//.*)?$"
)
HOOK_START_ALIAS_RE = re.compile(r"\(\s*처음\s*\)\s*할때")
HOOK_TICK_ALIAS_RE = re.compile(r"\(\s*매틱\s*\)\s*마다")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Auto-fix migration-priority lesson DDN patterns (//범위(...) and optional 채비:)."
    )
    parser.add_argument(
        "--scan-root",
        default=str(DEFAULT_SCAN_ROOT),
        help="root directory to scan recursively for *.ddn",
    )
    parser.add_argument(
        "--include-inputs",
        action="store_true",
        help="include inputs/*.ddn files",
    )
    parser.add_argument(
        "--include-preview",
        action="store_true",
        help="include *.age3.preview.ddn files",
    )
    parser.add_argument(
        "--rewrite-setup-colon",
        action="store_true",
        help="rewrite `채비: {` to `채비 {`",
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="write converted content back to files",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="max changed rows to print (default: 20)",
    )
    parser.add_argument(
        "--json-out",
        default="",
        help="optional json output path",
    )
    return parser.parse_args()


def should_include(path: Path, *, include_preview: bool, include_inputs: bool) -> bool:
    if path.suffix != ".ddn":
        return False
    if path.name.endswith(".bak.ddn"):
        return False
    if path.stem.endswith(PROMOTE_BACKUP_SUFFIX):
        return False
    if (not include_preview) and path.stem.endswith(".age3.preview"):
        return False
    parts = path.parts
    if "seed_lessons_v1" in parts:
        return False
    if "samples" in parts:
        return False
    if (not include_inputs) and "inputs" in parts:
        return False
    return True


def normalize_init_expr(init_expr: str) -> str:
    token = init_expr.strip()
    if token.startswith("(") and token.endswith(")"):
        return token
    return f"({token})"


def convert_line(
    line: str,
    *,
    rewrite_setup_colon: bool,
) -> tuple[str, dict[str, int]]:
    stats = {
        "range_rewrites": 0,
        "range_skipped": 0,
        "setup_colon_rewrites": 0,
        "hook_colon_rewrites": 0,
        "hook_alias_rewrites": 0,
    }

    range_match = RANGE_DECL_RE.match(line)
    if range_match:
        prefix = range_match.group("prefix")
        init_expr = normalize_init_expr(range_match.group("init"))
        min_expr = range_match.group("min").strip()
        max_expr = range_match.group("max").strip()
        step_expr = range_match.group("step").strip()
        tail = (range_match.group("tail") or "").strip()
        rewritten = f"{prefix}{init_expr} 매김 {{ 범위: {min_expr}..{max_expr}. 간격: {step_expr}. }}."
        if tail:
            rewritten = f"{rewritten} {tail}"
        stats["range_rewrites"] = 1
        return rewritten, stats

    if RANGE_COMMENT_HINT_RE.search(line):
        stats["range_skipped"] = 1
        return line, stats

    if rewrite_setup_colon:
        setup_match = SETUP_COLON_HEADER_RE.match(line)
        if setup_match:
            indent = setup_match.group("indent")
            tail = (setup_match.group("tail") or "").strip()
            rewritten = f"{indent}채비 {{"
            if tail:
                rewritten = f"{rewritten} {tail}"
            stats["setup_colon_rewrites"] = 1
            return rewritten, stats

    start_colon_match = HOOK_START_COLON_RE.match(line)
    if start_colon_match:
        indent = start_colon_match.group("indent")
        name = start_colon_match.group("name")
        tail = (start_colon_match.group("tail") or "").strip()
        canonical_name = "시작"
        rewritten = f"{indent}({canonical_name})할때 {{"
        if tail:
            rewritten = f"{rewritten} {tail}"
        stats["hook_colon_rewrites"] = 1
        if name != canonical_name:
            stats["hook_alias_rewrites"] = 1
        return rewritten, stats

    tick_colon_match = HOOK_TICK_COLON_RE.match(line)
    if tick_colon_match:
        indent = tick_colon_match.group("indent")
        name = str(tick_colon_match.group("name") or "").strip()
        tail = (tick_colon_match.group("tail") or "").strip()
        canonical_name = "매마디" if name in {"매마디", "매틱"} else re.sub(r"\s+", "", name)
        rewritten = f"{indent}({canonical_name})마다 {{"
        if tail:
            rewritten = f"{rewritten} {tail}"
        stats["hook_colon_rewrites"] = 1
        if name in {"매마디", "매틱"} and name != canonical_name:
            stats["hook_alias_rewrites"] = 1
        return rewritten, stats

    rewritten = line
    rewritten, start_alias_count = HOOK_START_ALIAS_RE.subn("(시작)할때", rewritten)
    rewritten, tick_alias_count = HOOK_TICK_ALIAS_RE.subn("(매마디)마다", rewritten)
    stats["hook_alias_rewrites"] = int(start_alias_count) + int(tick_alias_count)
    return rewritten, stats


def convert_text(text: str, *, rewrite_setup_colon: bool) -> tuple[str, dict[str, int]]:
    lines = text.splitlines()
    out: list[str] = []
    totals = {
        "range_rewrites": 0,
        "range_skipped": 0,
        "setup_colon_rewrites": 0,
        "hook_colon_rewrites": 0,
        "hook_alias_rewrites": 0,
    }
    for line in lines:
        converted, stats = convert_line(
            line,
            rewrite_setup_colon=rewrite_setup_colon,
        )
        out.append(converted)
        for key, value in stats.items():
            totals[key] += int(value)
    rendered = "\n".join(out)
    if text.endswith("\n"):
        rendered += "\n"
    return rendered, totals


def main() -> int:
    args = parse_args()
    scan_root = Path(args.scan_root).resolve()
    if not scan_root.exists():
        raise SystemExit(f"scan root not found: {scan_root}")

    targets = sorted(
        path
        for path in scan_root.rglob("*.ddn")
        if should_include(
            path,
            include_preview=bool(args.include_preview),
            include_inputs=bool(args.include_inputs),
        )
    )

    rows: list[dict[str, object]] = []
    changed = 0
    total_range_rewrites = 0
    total_range_skipped = 0
    total_setup_colon_rewrites = 0
    total_hook_colon_rewrites = 0
    total_hook_alias_rewrites = 0
    for path in targets:
        src = path.read_text(encoding="utf-8")
        converted, stats = convert_text(src, rewrite_setup_colon=bool(args.rewrite_setup_colon))
        is_changed = converted != src
        if is_changed and args.apply:
            path.write_text(converted, encoding="utf-8")
        if is_changed:
            changed += 1
        total_range_rewrites += int(stats["range_rewrites"])
        total_range_skipped += int(stats["range_skipped"])
        total_setup_colon_rewrites += int(stats["setup_colon_rewrites"])
        total_hook_colon_rewrites += int(stats["hook_colon_rewrites"])
        total_hook_alias_rewrites += int(stats["hook_alias_rewrites"])

        try:
            rel_path = path.relative_to(ROOT)
            path_text = str(rel_path)
        except ValueError:
            path_text = str(path)
        rows.append(
            {
                "path": path_text,
                "changed": is_changed,
                **stats,
            }
        )

    print(
        f"targets={len(targets)} changed={changed} apply={int(bool(args.apply))} "
        f"include_inputs={int(bool(args.include_inputs))} include_preview={int(bool(args.include_preview))} "
        f"rewrite_setup_colon={int(bool(args.rewrite_setup_colon))} "
        f"range_rewrites={total_range_rewrites} range_skipped={total_range_skipped} "
        f"setup_colon_rewrites={total_setup_colon_rewrites} "
        f"hook_colon_rewrites={total_hook_colon_rewrites} "
        f"hook_alias_rewrites={total_hook_alias_rewrites}"
    )

    shown = 0
    limit = max(0, int(args.limit))
    for row in rows:
        if not bool(row.get("changed", False)):
            continue
        if shown >= limit:
            break
        shown += 1
        print(
            f"[changed] {row['path']} "
            f"range={row['range_rewrites']} skipped={row['range_skipped']} setup={row['setup_colon_rewrites']} "
            f"hook_colon={row['hook_colon_rewrites']} hook_alias={row['hook_alias_rewrites']}"
        )

    if args.json_out:
        out_path = ROOT / str(args.json_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "schema": "seamgrim.lesson.migration_autofix.v1",
            "scan_root": str(scan_root),
            "apply": bool(args.apply),
            "include_inputs": bool(args.include_inputs),
            "include_preview": bool(args.include_preview),
            "rewrite_setup_colon": bool(args.rewrite_setup_colon),
            "targets": len(targets),
            "changed": changed,
            "totals": {
                "range_rewrites": total_range_rewrites,
                "range_skipped": total_range_skipped,
                "setup_colon_rewrites": total_setup_colon_rewrites,
                "hook_colon_rewrites": total_hook_colon_rewrites,
                "hook_alias_rewrites": total_hook_alias_rewrites,
            },
            "rows": rows,
        }
        out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"json_out={out_path}")

    print(
        "check=lesson_migration_autofix detail="
        f"ok:changed={changed}:range_rewrites={total_range_rewrites}:range_skipped={total_range_skipped}:targets={len(targets)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
