#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
LESSONS_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
PREVIEW_SUFFIX = ".age3.preview"
PROMOTE_BACKUP_SUFFIX = ".before_age3_promote.bak"

IDENT_RE = re.compile(r"^[A-Za-z_가-힣][A-Za-z0-9_가-힣.]*$")
NUMBER_RE = re.compile(r"^-?\d+(?:\.\d+)?$")
FORMULA_DEF_RE = re.compile(
    r"^\s*([A-Za-z_가-힣][A-Za-z0-9_가-힣.]*)\s*(?:=|<-)\s*\(#\w+\)\s*수식\{\s*[A-Za-z_가-힣][A-Za-z0-9_가-힣.]*\s*=\s*(.+?)\s*\}\s*\.\s*$"
)
SOLVER_BIND_RE = re.compile(
    r"^(\s*)([A-Za-z_가-힣][A-Za-z0-9_가-힣.]*)\s*<-\s*\((.*?)\)\s*인\s*([A-Za-z_가-힣][A-Za-z0-9_가-힣.]*)\s*풀기\s*\.\s*(//.*)?$"
)
STORAGE_HEADER_RE = re.compile(r"^(\s*그릇채비)\s*:\s*\{\s*(//.*)?$")


def should_skip(path: Path, include_preview: bool) -> bool:
    stem = path.stem
    if stem.endswith(PROMOTE_BACKUP_SUFFIX) or stem.endswith(".bak"):
        return True
    if not include_preview and stem.endswith(PREVIEW_SUFFIX):
        return True
    return False


def split_top_level_commas(text: str) -> list[str]:
    parts: list[str] = []
    start = 0
    depth = 0
    for idx, ch in enumerate(text):
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth = max(0, depth - 1)
        elif ch == "," and depth == 0:
            parts.append(text[start:idx].strip())
            start = idx + 1
    tail = text[start:].strip()
    if tail:
        parts.append(tail)
    return parts


def parse_named_args(args_text: str) -> dict[str, str] | None:
    mapping: dict[str, str] = {}
    for token in split_top_level_commas(args_text):
        if "=" not in token:
            return None
        key, value = token.split("=", 1)
        key = key.strip()
        value = value.strip()
        if not key or not value or not IDENT_RE.fullmatch(key):
            return None
        mapping[key] = value
    return mapping


def as_runtime_atom(text: str) -> str:
    token = text.strip()
    if IDENT_RE.fullmatch(token) or NUMBER_RE.fullmatch(token):
        return token
    return f"({token})"


def substitute_named_args(expr: str, mapping: dict[str, str]) -> str:
    updated = expr
    for key in sorted(mapping.keys(), key=len, reverse=True):
        val = as_runtime_atom(mapping[key])
        updated = re.sub(
            rf"(?<![0-9A-Za-z_가-힣.]){re.escape(key)}(?![0-9A-Za-z_가-힣.])",
            val,
            updated,
        )
    return updated


def transform_text(text: str) -> tuple[str, dict[str, int]]:
    lines = text.splitlines()
    formula_map: dict[str, str] = {}
    for line in lines:
        match = FORMULA_DEF_RE.match(line)
        if not match:
            continue
        formula_name, expr = match.groups()
        formula_map[formula_name] = expr.strip()

    out: list[str] = []
    solver_rewrites = 0
    solver_skipped = 0
    storage_rewrites = 0

    for line in lines:
        storage_match = STORAGE_HEADER_RE.match(line)
        if storage_match:
            prefix, comment = storage_match.groups()
            if comment:
                out.append(f"{prefix} {{ {comment}")
            else:
                out.append(f"{prefix} {{")
            storage_rewrites += 1
            continue

        solve_match = SOLVER_BIND_RE.match(line)
        if not solve_match:
            out.append(line)
            continue

        indent, lhs, args_text, formula_name, comment = solve_match.groups()
        formula_expr = formula_map.get(formula_name)
        if not formula_expr:
            out.append(line)
            solver_skipped += 1
            continue

        mapping = parse_named_args(args_text)
        if mapping is None:
            out.append(line)
            solver_skipped += 1
            continue

        runtime_expr = substitute_named_args(formula_expr, mapping)
        rewritten = f"{indent}{lhs} <- {runtime_expr}."
        if comment:
            rewritten = f"{rewritten} {comment.strip()}"
        out.append(rewritten)
        solver_rewrites += 1

    converted = "\n".join(out)
    if text.endswith("\n"):
        converted += "\n"
    return converted, {
        "solver_rewrites": solver_rewrites,
        "solver_skipped": solver_skipped,
        "storage_rewrites": storage_rewrites,
    }


def iter_targets(include_inputs: bool, include_preview: bool) -> list[Path]:
    targets: list[Path] = []
    targets.extend(
        sorted(
            path for path in LESSONS_ROOT.rglob("lesson.ddn") if not should_skip(path, include_preview=include_preview)
        )
    )
    if include_preview:
        targets.extend(
            sorted(
                path
                for path in LESSONS_ROOT.rglob(f"*{PREVIEW_SUFFIX}.ddn")
                if not should_skip(path, include_preview=include_preview)
            )
        )
    if include_inputs:
        targets.extend(
            sorted(
                path
                for path in LESSONS_ROOT.rglob("inputs/*.ddn")
                if not should_skip(path, include_preview=include_preview)
            )
        )
        if include_preview:
            targets.extend(
                sorted(
                    path
                    for path in LESSONS_ROOT.rglob(f"inputs/*{PREVIEW_SUFFIX}.ddn")
                    if not should_skip(path, include_preview=include_preview)
                )
            )
    return targets


def main() -> int:
    parser = argparse.ArgumentParser(
        description="lesson 경고 토큰(legacy solver bind / 그릇채비:)을 자동 치환합니다."
    )
    parser.add_argument("--include-inputs", action="store_true", help="inputs/*.ddn도 포함")
    parser.add_argument("--include-preview", action="store_true", help="*.age3.preview.ddn도 포함")
    parser.add_argument("--apply", action="store_true", help="실제 파일을 수정")
    parser.add_argument("--limit", type=int, default=30, help="콘솔 출력 최대 개수")
    parser.add_argument("--json-out", default="", help="결과 리포트 경로")
    args = parser.parse_args()

    targets = iter_targets(
        include_inputs=bool(args.include_inputs),
        include_preview=bool(args.include_preview),
    )
    if not targets:
        print("targets=0")
        return 0

    changed = 0
    total_solver_rewrites = 0
    total_solver_skipped = 0
    total_storage_rewrites = 0
    rows: list[dict[str, object]] = []
    for path in targets:
        src = path.read_text(encoding="utf-8")
        converted, stats = transform_text(src)
        is_changed = converted != src
        if is_changed and args.apply:
            path.write_text(converted, encoding="utf-8")
        if is_changed:
            changed += 1
        total_solver_rewrites += int(stats["solver_rewrites"])
        total_solver_skipped += int(stats["solver_skipped"])
        total_storage_rewrites += int(stats["storage_rewrites"])
        rows.append(
            {
                "path": str(path.relative_to(ROOT)),
                "changed": is_changed,
                **stats,
            }
        )

    print(
        f"targets={len(targets)} changed={changed} apply={int(bool(args.apply))} "
        f"include_preview={int(bool(args.include_preview))} "
        f"solver_rewrites={total_solver_rewrites} solver_skipped={total_solver_skipped} "
        f"storage_rewrites={total_storage_rewrites}"
    )
    shown = 0
    for row in rows:
        if not row["changed"]:
            continue
        if shown >= max(0, int(args.limit)):
            break
        shown += 1
        print(
            f"[changed] {row['path']} solver={row['solver_rewrites']} "
            f"storage={row['storage_rewrites']} skipped={row['solver_skipped']}"
        )

    if args.json_out:
        out_path = ROOT / str(args.json_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "schema": "seamgrim.lesson_legacy_warning_autofix.v1",
            "targets": len(targets),
            "changed": changed,
            "apply": bool(args.apply),
            "include_preview": bool(args.include_preview),
            "totals": {
                "solver_rewrites": total_solver_rewrites,
                "solver_skipped": total_solver_skipped,
                "storage_rewrites": total_storage_rewrites,
            },
            "rows": rows,
        }
        out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"json_out={out_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
