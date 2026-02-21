#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
LESSONS_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
PREVIEW_SUFFIX = ".age3.preview"
PROMOTE_BACKUP_SUFFIX = ".before_age3_promote.bak"

BOIM_OPEN_RE = re.compile(r"^(\s*)보임\s*\{\s*$")
BOIM_CLOSE_RE = re.compile(r"^\s*}\s*\.?\s*$")
BOIM_ENTRY_RE = re.compile(r"^\s*([^:#]+)\s*:\s*(.+?)\s*\.\s*$")
ASSIGN_EQ_RE = re.compile(
    r"^(\s*)(?!#)([A-Za-z_가-힣][A-Za-z0-9_가-힣.]*)\s*=\s*(.+?)\s*\.\s*$"
)
INLINE_FORMULA_INJECT_RE = re.compile(
    r"^(\s*)([A-Za-z_가-힣][A-Za-z0-9_가-힣.]*)\s*<-\s*\((.*?)\)\s*(\(#\w+\))\s*수식\{(.*)\}\s*\.\s*$"
)
INLINE_COND_BLOCK_RE = re.compile(
    r"^(\s*)(\{.+\}\s*인것\s*(?:일때|동안))\s*\{\s*(.+?)\s*\}\s*\.\s*$"
)
CALL_STYLE_RE = re.compile(r"\b(sin|cos|tan|sqrt|abs|log|exp)\s*\(\s*([^)]+?)\s*\)")


@dataclass
class TransformStats:
    boim_blocks_replaced: int = 0
    boim_blocks_skipped: int = 0
    show_lines_added: int = 0
    eq_assign_replaced: int = 0
    inline_formula_rewritten: int = 0
    inline_block_terminated: int = 0
    call_style_rewritten: int = 0

    def changed(self) -> bool:
        return (
            self.boim_blocks_replaced > 0
            or self.show_lines_added > 0
            or self.eq_assign_replaced > 0
            or self.inline_formula_rewritten > 0
            or self.inline_block_terminated > 0
            or self.call_style_rewritten > 0
        )


def is_preview_or_backup(path: Path) -> bool:
    stem = path.stem
    return stem.endswith(PREVIEW_SUFFIX) or stem.endswith(PROMOTE_BACKUP_SUFFIX)


def collect_targets(path_args: list[str], include_inputs: bool) -> list[Path]:
    targets: set[Path] = set()
    if not path_args:
        targets.update(path for path in LESSONS_ROOT.rglob("lesson.ddn"))
        if include_inputs:
            targets.update(path for path in LESSONS_ROOT.rglob("inputs/*.ddn"))
        return sorted(path for path in targets if not is_preview_or_backup(path))

    for raw in path_args:
        rel = Path(raw)
        candidate = LESSONS_ROOT / rel
        if candidate.is_file():
            if not is_preview_or_backup(candidate):
                targets.add(candidate)
            continue
        if candidate.is_dir():
            targets.update(path for path in candidate.glob("lesson.ddn"))
            if include_inputs:
                targets.update(path for path in candidate.rglob("inputs/*.ddn"))
            continue
        for item in LESSONS_ROOT.glob(raw):
            if item.is_file():
                targets.add(item)
            elif item.is_dir():
                targets.update(path for path in item.glob("lesson.ddn"))
                if include_inputs:
                    targets.update(path for path in item.rglob("inputs/*.ddn"))
    return sorted(path for path in targets if not is_preview_or_backup(path))


def convert_boim_blocks(lines: list[str], stats: TransformStats) -> list[str]:
    out: list[str] = []
    i = 0
    while i < len(lines):
        open_match = BOIM_OPEN_RE.match(lines[i])
        if not open_match:
            out.append(lines[i])
            i += 1
            continue

        indent = open_match.group(1)
        block_lines = [lines[i]]
        i += 1
        body_lines: list[str] = []
        found_close = False
        while i < len(lines):
            block_lines.append(lines[i])
            if BOIM_CLOSE_RE.match(lines[i]):
                found_close = True
                i += 1
                break
            body_lines.append(lines[i])
            i += 1

        if not found_close:
            out.extend(block_lines)
            stats.boim_blocks_skipped += 1
            continue

        exprs: list[str] = []
        parse_failed = False
        for raw in body_lines:
            stripped = raw.strip()
            if not stripped:
                continue
            if stripped.startswith("//"):
                continue
            entry = BOIM_ENTRY_RE.match(raw)
            if not entry:
                parse_failed = True
                break
            exprs.append(entry.group(2).strip())

        if parse_failed:
            out.extend(block_lines)
            stats.boim_blocks_skipped += 1
            continue

        for expr in exprs:
            out.append(f"{indent}{expr} 보여주기.")
        stats.boim_blocks_replaced += 1
        stats.show_lines_added += len(exprs)
    return out


def convert_plain_eq_assign(lines: list[str], stats: TransformStats) -> list[str]:
    out: list[str] = []
    for line in lines:
        match = ASSIGN_EQ_RE.match(line)
        if not match:
            out.append(line)
            continue
        indent, lhs, rhs = match.groups()
        out.append(f"{indent}{lhs} <- {rhs}.")
        stats.eq_assign_replaced += 1
    return out


def rewrite_inline_formula_inject(lines: list[str], stats: TransformStats) -> list[str]:
    out: list[str] = []
    temp_index = 0
    for line in lines:
        match = INLINE_FORMULA_INJECT_RE.match(line)
        if not match:
            out.append(line)
            continue
        indent, lhs, args, tag, expr = match.groups()
        temp_index += 1
        temp_name = f"__seamgrim_tmp_formula_{temp_index}"
        out.append(f"{indent}{temp_name} <- {tag} 수식{{{expr}}}.")
        out.append(f"{indent}{lhs} <- ({args})인 {temp_name} 풀기.")
        stats.inline_formula_rewritten += 1
    return out


def normalize_inline_block_terminator(lines: list[str], stats: TransformStats) -> list[str]:
    out: list[str] = []
    for line in lines:
        match = INLINE_COND_BLOCK_RE.match(line)
        if not match:
            out.append(line)
            continue
        indent, prefix, body = match.groups()
        body_trim = body.strip()
        if body_trim.endswith("."):
            out.append(line)
            continue
        out.append(f"{indent}{prefix} {{ {body_trim}. }}.")
        stats.inline_block_terminated += 1
    return out


def rewrite_call_style_functions(lines: list[str], stats: TransformStats) -> list[str]:
    out: list[str] = []
    for line in lines:
        if "수식{" in line:
            out.append(line)
            continue
        updated = line
        while True:
            match = CALL_STYLE_RE.search(updated)
            if not match:
                break
            fn, arg = match.groups()
            repl = f"({arg.strip()}) {fn}"
            updated = updated[: match.start()] + repl + updated[match.end() :]
            stats.call_style_rewritten += 1
        out.append(updated)
    return out


def transform_text(text: str) -> tuple[str, TransformStats]:
    stats = TransformStats()
    lines = text.splitlines()
    lines = convert_boim_blocks(lines, stats)
    lines = convert_plain_eq_assign(lines, stats)
    lines = rewrite_inline_formula_inject(lines, stats)
    lines = normalize_inline_block_terminator(lines, stats)
    lines = rewrite_call_style_functions(lines, stats)
    converted = "\n".join(lines)
    if text.endswith("\n"):
        converted += "\n"
    return converted, stats


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Seamgrim 교과 DDN을 현재 parser 호환 스키마로 정렬(보임 블록/평문 '=' 대입)합니다."
    )
    parser.add_argument(
        "--paths",
        nargs="*",
        default=[],
        help="lesson 폴더명/파일/글롭. 비우면 전체 lessons/lesson.ddn 대상.",
    )
    parser.add_argument(
        "--include-inputs",
        action="store_true",
        help="inputs/*.ddn도 대상으로 포함합니다.",
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="실제 파일을 수정합니다. 기본은 dry-run입니다.",
    )
    parser.add_argument(
        "--json-out",
        help="결과 JSON 경로",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=30,
        help="콘솔에 표시할 changed 파일 수(기본 30)",
    )
    args = parser.parse_args()

    if not LESSONS_ROOT.exists():
        raise SystemExit(f"lessons root not found: {LESSONS_ROOT}")

    targets = collect_targets(args.paths, include_inputs=args.include_inputs)
    if not targets:
        print("대상 파일이 없습니다.")
        return 0

    rows: list[dict[str, object]] = []
    total = TransformStats()
    changed = 0
    for path in targets:
        src = path.read_text(encoding="utf-8")
        converted, stats = transform_text(src)
        is_changed = converted != src
        if is_changed and args.apply:
            path.write_text(converted, encoding="utf-8")
        if is_changed:
            changed += 1
        total.boim_blocks_replaced += stats.boim_blocks_replaced
        total.boim_blocks_skipped += stats.boim_blocks_skipped
        total.show_lines_added += stats.show_lines_added
        total.eq_assign_replaced += stats.eq_assign_replaced
        total.inline_formula_rewritten += stats.inline_formula_rewritten
        total.inline_block_terminated += stats.inline_block_terminated
        total.call_style_rewritten += stats.call_style_rewritten
        rows.append(
            {
                "path": str(path.relative_to(ROOT)),
                "changed": is_changed,
                "boim_blocks_replaced": stats.boim_blocks_replaced,
                "boim_blocks_skipped": stats.boim_blocks_skipped,
                "show_lines_added": stats.show_lines_added,
                "eq_assign_replaced": stats.eq_assign_replaced,
                "inline_formula_rewritten": stats.inline_formula_rewritten,
                "inline_block_terminated": stats.inline_block_terminated,
                "call_style_rewritten": stats.call_style_rewritten,
            }
        )

    print(
        f"targets={len(targets)} changed={changed} apply={int(args.apply)} "
        f"boim_replaced={total.boim_blocks_replaced} boim_skipped={total.boim_blocks_skipped} "
        f"show_added={total.show_lines_added} eq_replaced={total.eq_assign_replaced} "
        f"inline_formula_rewritten={total.inline_formula_rewritten} "
        f"inline_block_terminated={total.inline_block_terminated} "
        f"call_style_rewritten={total.call_style_rewritten}"
    )

    printed = 0
    for row in rows:
        if not row["changed"]:
            continue
        if printed >= max(0, int(args.limit)):
            break
        printed += 1
        print(
            f"[changed] {row['path']} boim={row['boim_blocks_replaced']} "
            f"show+={row['show_lines_added']} eq={row['eq_assign_replaced']} "
            f"inline={row['inline_formula_rewritten']} "
            f"inline_dot={row['inline_block_terminated']} "
            f"call={row['call_style_rewritten']}"
        )

    if args.json_out:
        out_path = Path(args.json_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "schema": "seamgrim.lesson.schema_realign.v1",
            "targets": len(targets),
            "changed": changed,
            "apply": bool(args.apply),
            "totals": {
                "boim_blocks_replaced": total.boim_blocks_replaced,
                "boim_blocks_skipped": total.boim_blocks_skipped,
                "show_lines_added": total.show_lines_added,
                "eq_assign_replaced": total.eq_assign_replaced,
                "inline_formula_rewritten": total.inline_formula_rewritten,
                "inline_block_terminated": total.inline_block_terminated,
                "call_style_rewritten": total.call_style_rewritten,
            },
            "rows": rows,
        }
        out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"json_out={out_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
