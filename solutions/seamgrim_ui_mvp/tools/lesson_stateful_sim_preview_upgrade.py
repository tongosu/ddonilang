#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
LESSONS_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
DEFAULT_PREVIEW_SUFFIX = ".sim.age3.preview"
AGE3_PREVIEW_SUFFIX = ".age3.preview"
PROMOTE_BACKUP_SUFFIX = ".before_age3_promote.bak"

MADI_OPEN_RE = re.compile(r"^(\s*)\(\s*매마디\s*\)\s*마다\s*\{\s*$")
START_OPEN_RE = re.compile(r"^(\s*)\(\s*시작\s*\)\s*할때\s*\{\s*$")
RANGE_RE = re.compile(
    r"^\s*([A-Za-z_가-힣][A-Za-z0-9_가-힣.]*)\s*<-\s*\(\s*([^,]+)\s*,\s*([^,]+)\s*,\s*([^)]+)\)\s*범위\s*\.\s*$"
)
FOREACH_RE = re.compile(
    r"^\s*\(\s*([A-Za-z_가-힣][A-Za-z0-9_가-힣.]*)\s*\)\s*([A-Za-z_가-힣][A-Za-z0-9_가-힣.]*)\s*에\s*대해\s*:\s*\{\s*$"
)
ASSIGN_RE = re.compile(r"^\s*([A-Za-z_가-힣][A-Za-z0-9_가-힣.]*)\s*<-\s*(.+?)\s*\.\s*$")
IDENT_RE = re.compile(r"^[A-Za-z_가-힣][A-Za-z0-9_가-힣.]*$")


def should_skip(path: Path) -> bool:
    stem = path.stem
    return (
        stem.endswith(AGE3_PREVIEW_SUFFIX)
        or stem.endswith(DEFAULT_PREVIEW_SUFFIX)
        or stem.endswith(PROMOTE_BACKUP_SUFFIX)
        or stem.endswith(".bak")
    )


def iter_targets(include_inputs: bool) -> list[Path]:
    targets: list[Path] = []
    targets.extend(sorted(path for path in LESSONS_ROOT.rglob("lesson.ddn") if not should_skip(path)))
    if include_inputs:
        targets.extend(sorted(path for path in LESSONS_ROOT.rglob("inputs/*.ddn") if not should_skip(path)))
    return targets


def count_brace_delta(line: str) -> int:
    return line.count("{") - line.count("}")


def find_block_end(lines: list[str], start_index: int) -> int | None:
    depth = count_brace_delta(lines[start_index])
    idx = start_index + 1
    while idx < len(lines):
        depth += count_brace_delta(lines[idx])
        if depth <= 0:
            return idx
        idx += 1
    return None


def has_self_update(lhs: str, rhs: str) -> bool:
    token_pattern = re.compile(rf"(?<![0-9A-Za-z_가-힣.]){re.escape(lhs)}(?![0-9A-Za-z_가-힣.])")
    return bool(token_pattern.search(rhs))


def remove_prefix_once(line: str, prefix: str) -> str:
    if prefix and line.startswith(prefix):
        return line[len(prefix) :]
    return line.lstrip()


def parse_stateful_loop(loop_body: list[str]) -> set[str]:
    vars_out: set[str] = set()
    for line in loop_body:
        match = ASSIGN_RE.match(line)
        if not match:
            continue
        lhs, rhs = (value.strip() for value in match.groups())
        if has_self_update(lhs, rhs):
            vars_out.add(lhs)
    return vars_out


def convert_file(text: str) -> tuple[str, dict[str, object]]:
    lines = text.splitlines()
    if any(START_OPEN_RE.match(line) for line in lines):
        return text, {"converted": False, "reason": "already_has_start_block"}

    madi_indices = [idx for idx, line in enumerate(lines) if MADI_OPEN_RE.match(line)]
    if len(madi_indices) != 1:
        return text, {"converted": False, "reason": "madi_block_not_single"}

    madi_start = madi_indices[0]
    madi_end = find_block_end(lines, madi_start)
    if madi_end is None:
        return text, {"converted": False, "reason": "madi_block_unclosed"}

    madi_match = MADI_OPEN_RE.match(lines[madi_start])
    if madi_match is None:
        return text, {"converted": False, "reason": "madi_open_parse_failed"}
    outer_indent = madi_match.group(1)
    block_indent = outer_indent + "  "
    inner_indent = block_indent + "  "

    body = lines[madi_start + 1 : madi_end]
    range_idx = -1
    range_match = None
    for idx, line in enumerate(body):
        current = RANGE_RE.match(line)
        if current:
            range_idx = idx
            range_match = current
            break
    if range_idx < 0 or range_match is None:
        return text, {"converted": False, "reason": "range_not_found"}

    list_var, start_expr, end_expr, step_expr = (value.strip() for value in range_match.groups())

    foreach_idx = -1
    foreach_match = None
    for idx in range(range_idx + 1, len(body)):
        current = FOREACH_RE.match(body[idx])
        if current and current.group(2).strip() == list_var:
            foreach_idx = idx
            foreach_match = current
            break
    if foreach_idx < 0 or foreach_match is None:
        return text, {"converted": False, "reason": "foreach_not_found"}

    loop_end_rel = find_block_end(body, foreach_idx)
    if loop_end_rel is None:
        return text, {"converted": False, "reason": "foreach_block_unclosed"}

    trailing = [line for line in body[loop_end_rel + 1 :] if line.strip()]
    if trailing:
        return text, {"converted": False, "reason": "madi_has_trailing_statements"}

    iter_var = foreach_match.group(1).strip()
    if not IDENT_RE.fullmatch(iter_var):
        return text, {"converted": False, "reason": "iter_var_invalid"}

    loop_body = body[foreach_idx + 1 : loop_end_rel]
    self_update_vars = parse_stateful_loop(loop_body)
    if not self_update_vars:
        return text, {"converted": False, "reason": "no_self_update_in_loop"}
    if iter_var in self_update_vars:
        return text, {"converted": False, "reason": "iter_self_update_not_supported"}

    pre_lines = body[:range_idx]
    moved_init_lines: list[str] = []
    top_level_lines: list[str] = []
    for line in pre_lines:
        assign_match = ASSIGN_RE.match(line)
        if assign_match:
            lhs = assign_match.group(1).strip()
            if lhs in self_update_vars:
                moved_init_lines.append(remove_prefix_once(line, block_indent))
                continue
        top_level_lines.append(remove_prefix_once(line, block_indent))

    if not moved_init_lines:
        return text, {"converted": False, "reason": "state_init_not_found"}

    loop_body_dedented = [remove_prefix_once(line, inner_indent) for line in loop_body]

    out: list[str] = []
    out.extend(lines[:madi_start])

    while out and not out[-1].strip():
        out.pop()
    if out:
        out.append("")

    out.append("// stateful-sim preview(auto): range sampler -> step sim")
    for line in top_level_lines:
        out.append(line)
    if top_level_lines:
        out.append("")

    out.append(f"{outer_indent}(시작)할때 {{")
    out.append(f"{block_indent}{iter_var} <- {start_expr}.")
    for line in moved_init_lines:
        out.append(f"{block_indent}{line}")
    out.append(f"{outer_indent}}}.")
    out.append("")

    out.append(f"{outer_indent}(매마디)마다 {{")
    out.append(f"{block_indent}{{ {iter_var} <= {end_expr} }}인것 일때 {{")
    for line in loop_body_dedented:
        out.append(f"{inner_indent}{line}")
    out.append(f"{inner_indent}{iter_var} <- {iter_var} + ({step_expr}).")
    out.append(f"{block_indent}}}.")
    out.append(f"{outer_indent}}}.")

    after = lines[madi_end + 1 :]
    if after:
        out.extend(after)

    converted = "\n".join(out)
    if text.endswith("\n"):
        converted += "\n"

    return converted, {
        "converted": True,
        "reason": "ok",
        "iter_var": iter_var,
        "list_var": list_var,
        "start_expr": start_expr,
        "end_expr": end_expr,
        "step_expr": step_expr,
        "self_update_vars": sorted(self_update_vars),
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="stateful candidate lesson을 step simulation preview로 자동 변환합니다."
    )
    parser.add_argument("--include-inputs", action="store_true", help="inputs/*.ddn 포함")
    parser.add_argument("--write-preview", action="store_true", help="*.sim.age3.preview.ddn 파일 생성")
    parser.add_argument("--apply", action="store_true", help="원본 파일 직접 치환")
    parser.add_argument("--preview-suffix", default=DEFAULT_PREVIEW_SUFFIX, help="preview suffix")
    parser.add_argument("--json-out", default="", help="리포트 출력 경로")
    parser.add_argument("--limit", type=int, default=20, help="콘솔 출력 최대 개수")
    args = parser.parse_args()

    targets = iter_targets(include_inputs=bool(args.include_inputs))
    rows: list[dict[str, object]] = []
    convertible = 0
    written = 0

    for path in targets:
        src = path.read_text(encoding="utf-8")
        converted, meta = convert_file(src)
        is_convertible = bool(meta.get("converted"))
        if is_convertible:
            convertible += 1
            if args.apply:
                path.write_text(converted, encoding="utf-8")
                written += 1
            elif args.write_preview:
                preview_path = path.with_name(f"{path.stem}{args.preview_suffix}{path.suffix}")
                preview_path.write_text(converted, encoding="utf-8")
                written += 1
        rows.append(
            {
                "path": str(path.relative_to(ROOT)),
                **meta,
            }
        )

    reasons: dict[str, int] = {}
    for row in rows:
        reason = str(row.get("reason", "unknown"))
        reasons[reason] = reasons.get(reason, 0) + 1

    print(
        f"targets={len(targets)} convertible={convertible} "
        f"write_preview={int(bool(args.write_preview))} apply={int(bool(args.apply))} written={written}"
    )
    print("reasons:", " ".join(f"{key}={value}" for key, value in sorted(reasons.items())))

    shown = 0
    for row in rows:
        if not row.get("converted"):
            continue
        if shown >= max(0, int(args.limit)):
            break
        shown += 1
        print(
            f"[convertible] {row['path']} "
            f"iter={row.get('iter_var')} end={row.get('end_expr')} step={row.get('step_expr')} "
            f"state={','.join(row.get('self_update_vars') or [])}"
        )

    if args.json_out:
        out_path = ROOT / str(args.json_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "schema": "seamgrim.stateful_sim_preview_upgrade.v1",
            "targets": len(targets),
            "convertible": convertible,
            "written": written,
            "write_preview": bool(args.write_preview),
            "apply": bool(args.apply),
            "reasons": reasons,
            "rows": rows,
        }
        out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"json_out={out_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
