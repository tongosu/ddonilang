#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SOURCE_ROOT = ROOT / "docs" / "guides" / "study"
PREFERRED_BUILD_DIR = Path("I:/home/urihanl/ddn/codex/build")
FALLBACK_BUILD_DIR = Path("C:/ddn/codex/build")

LEGACY_PREFIX_RE = re.compile(r"(?:바탕|살림)\.([0-9A-Za-z_가-힣]+)")
BLOCK_HEADER_COLON_RE = re.compile(r"(에\s+대해):\s*\{")


@dataclass
class FileResult:
    path: Path
    changed: bool
    ddn_block_count: int
    prefix_replacements: int
    block_header_replacements: int
    block_close_replacements: int
    prose_legacy_mentions: int


def resolve_build_dir() -> Path:
    target = PREFERRED_BUILD_DIR if PREFERRED_BUILD_DIR.exists() else FALLBACK_BUILD_DIR
    target.mkdir(parents=True, exist_ok=True)
    return target


def normalize_text(text: str) -> str:
    value = str(text or "").replace("\r\n", "\n").replace("\r", "\n")
    if value.startswith("\ufeff"):
        value = value.lstrip("\ufeff")
    return value


def iter_markdown_files(source_root: Path) -> list[Path]:
    files: list[Path] = []
    for path in sorted(source_root.glob("*.md")):
        if not path.is_file():
            continue
        if path.name.startswith("SEAMGRIM_"):
            continue
        files.append(path)
    return files


def rewrite_ddn_block(lines: list[str]) -> tuple[list[str], dict[str, int]]:
    counts = {
        "prefix_replacements": 0,
        "block_header_replacements": 0,
        "block_close_replacements": 0,
    }
    out: list[str] = []
    for raw in lines:
        line = raw
        line, prefix_count = LEGACY_PREFIX_RE.subn(r"\1", line)
        counts["prefix_replacements"] += prefix_count
        line, header_count = BLOCK_HEADER_COLON_RE.subn(r"\1 {", line)
        counts["block_header_replacements"] += header_count
        if line.strip() == "}":
            indent = line[: len(line) - len(line.lstrip(" \t"))]
            line = indent + "}."
            counts["block_close_replacements"] += 1
        out.append(line)
    return out, counts


def process_file(path: Path, apply_safe: bool) -> FileResult:
    original = normalize_text(path.read_text(encoding="utf-8"))
    lines = original.split("\n")
    inside_ddn = False
    ddn_block_count = 0
    prose_legacy_mentions = 0
    prefix_replacements = 0
    block_header_replacements = 0
    block_close_replacements = 0
    changed = False
    out_lines: list[str] = []
    block_buffer: list[str] = []

    def flush_block() -> None:
        nonlocal changed, prefix_replacements, block_header_replacements, block_close_replacements
        rewritten, counts = rewrite_ddn_block(block_buffer)
        prefix_replacements += counts["prefix_replacements"]
        block_header_replacements += counts["block_header_replacements"]
        block_close_replacements += counts["block_close_replacements"]
        if rewritten != block_buffer:
            changed = True
        out_lines.extend(rewritten)

    for line in lines:
        stripped = line.strip()
        if stripped == "```ddn":
            inside_ddn = True
            ddn_block_count += 1
            out_lines.append(line)
            block_buffer = []
            continue
        if inside_ddn and stripped == "```":
            flush_block()
            block_buffer = []
            inside_ddn = False
            out_lines.append(line)
            continue
        if inside_ddn:
            block_buffer.append(line)
            continue
        prose_legacy_mentions += len(LEGACY_PREFIX_RE.findall(line))
        out_lines.append(line)

    if inside_ddn:
        flush_block()

    rewritten = "\n".join(out_lines)
    if original.endswith("\n") and not rewritten.endswith("\n"):
        rewritten += "\n"
    if apply_safe and changed:
        path.write_text(rewritten, encoding="utf-8")

    return FileResult(
        path=path,
        changed=changed,
        ddn_block_count=ddn_block_count,
        prefix_replacements=prefix_replacements,
        block_header_replacements=block_header_replacements,
        block_close_replacements=block_close_replacements,
        prose_legacy_mentions=prose_legacy_mentions,
    )


def display_path(path: Path) -> str:
    try:
        return path.relative_to(ROOT).as_posix()
    except Exception:
        return path.as_posix()


def main() -> int:
    parser = argparse.ArgumentParser(description="docs/guides/study의 fenced ddn block을 current-line 안전 규칙으로 보정한다.")
    parser.add_argument("--source-root", default=str(SOURCE_ROOT))
    parser.add_argument("--apply-safe", action="store_true")
    parser.add_argument("--json-out")
    args = parser.parse_args()

    source_root = Path(args.source_root)
    if not source_root.is_absolute():
        source_root = (ROOT / source_root).resolve()
    if not source_root.is_dir():
        raise SystemExit(f"source root not found: {source_root}")

    file_results = [process_file(path, args.apply_safe) for path in iter_markdown_files(source_root)]
    changed_files = [fr for fr in file_results if fr.changed]
    prose_problem_files = [fr for fr in file_results if fr.prose_legacy_mentions > 0]

    report = {
        "schema": "ddn.seamgrim.study_markdown_repair_report.v1",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "apply_safe": bool(args.apply_safe),
        "source_root": source_root.as_posix(),
        "counts": {
            "markdown_files": len(file_results),
            "changed_files": len(changed_files),
            "ddn_blocks": sum(fr.ddn_block_count for fr in file_results),
            "prefix_replacements": sum(fr.prefix_replacements for fr in file_results),
            "block_header_replacements": sum(fr.block_header_replacements for fr in file_results),
            "block_close_replacements": sum(fr.block_close_replacements for fr in file_results),
            "prose_problem_files": len(prose_problem_files),
            "prose_legacy_mentions": sum(fr.prose_legacy_mentions for fr in file_results),
        },
        "files": [
            {
                "path": display_path(fr.path),
                "changed": fr.changed,
                "ddn_block_count": fr.ddn_block_count,
                "prefix_replacements": fr.prefix_replacements,
                "block_header_replacements": fr.block_header_replacements,
                "block_close_replacements": fr.block_close_replacements,
                "prose_legacy_mentions": fr.prose_legacy_mentions,
            }
            for fr in file_results
        ],
        "prose_problem_paths": [display_path(fr.path) for fr in prose_problem_files],
    }

    output_path = Path(args.json_out).resolve() if args.json_out else resolve_build_dir() / "study_practice" / "study_markdown_repair_report.detjson"
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")

    print(f"source_root={source_root}")
    print(f"markdown_files={report['counts']['markdown_files']}")
    print(f"changed_files={report['counts']['changed_files']}")
    print(f"ddn_blocks={report['counts']['ddn_blocks']}")
    print(f"prefix_replacements={report['counts']['prefix_replacements']}")
    print(f"block_header_replacements={report['counts']['block_header_replacements']}")
    print(f"block_close_replacements={report['counts']['block_close_replacements']}")
    print(f"prose_problem_files={report['counts']['prose_problem_files']}")
    print(f"json_out={output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
