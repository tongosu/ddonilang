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

IDENT_RE = re.compile(r"[A-Za-z_가-힣][A-Za-z0-9_가-힣.]*")
RANGE_RE = re.compile(
    r"^\s*([A-Za-z_가-힣][A-Za-z0-9_가-힣.]*)\s*<-\s*\(\s*([^,]+)\s*,\s*([^,]+)\s*,\s*([^)]+)\)\s*범위\s*\.\s*$"
)
FOREACH_RE = re.compile(
    r"^\s*\(\s*([A-Za-z_가-힣][A-Za-z0-9_가-힣.]*)\s*\)\s*([A-Za-z_가-힣][A-Za-z0-9_가-힣.]*)\s*에\s*대해\s*:\s*\{"
)
ASSIGN_RE = re.compile(
    r"^\s*([A-Za-z_가-힣][A-Za-z0-9_가-힣.]*)\s*<-\s*(.+?)\s*\.\s*$"
)


def should_skip(path: Path) -> bool:
    stem = path.stem
    return stem.endswith(PREVIEW_SUFFIX) or stem.endswith(PROMOTE_BACKUP_SUFFIX) or stem.endswith(".bak")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Graph-only lesson을 시뮬 전환 후보로 분류하는 리포트 생성기"
    )
    parser.add_argument("--include-inputs", action="store_true", help="inputs/*.ddn까지 포함")
    parser.add_argument(
        "--json-out",
        default="build/reports/seamgrim_sim_conversion_plan.detjson",
        help="리포트 출력 경로",
    )
    parser.add_argument("--limit", type=int, default=20, help="콘솔 출력 최대 개수")
    return parser.parse_args()


def iter_targets(include_inputs: bool) -> list[Path]:
    targets: list[Path] = []
    targets.extend(sorted(p for p in LESSONS_ROOT.rglob("lesson.ddn") if not should_skip(p)))
    if include_inputs:
        targets.extend(sorted(p for p in LESSONS_ROOT.rglob("inputs/*.ddn") if not should_skip(p)))
    return targets


def collect_range_map(lines: list[str]) -> dict[str, dict[str, str]]:
    out: dict[str, dict[str, str]] = {}
    for line in lines:
        match = RANGE_RE.match(line)
        if not match:
            continue
        name, start, end, step = match.groups()
        out[name] = {
            "list_var": name.strip(),
            "start": start.strip(),
            "end": end.strip(),
            "step": step.strip(),
        }
    return out


def count_brace_delta(line: str) -> int:
    return line.count("{") - line.count("}")


def extract_loop_body(lines: list[str], start_index: int) -> tuple[list[str], int]:
    body: list[str] = []
    depth = 1
    index = start_index + 1
    while index < len(lines):
        line = lines[index]
        depth += count_brace_delta(line)
        if depth <= 0:
            return body, index
        body.append(line)
        index += 1
    return body, len(lines) - 1


def has_self_update(lhs: str, rhs: str) -> bool:
    token_pattern = re.compile(rf"(?<![0-9A-Za-z_가-힣.]){re.escape(lhs)}(?![0-9A-Za-z_가-힣.])")
    return bool(token_pattern.search(rhs))


def classify_file(path: Path) -> dict[str, object]:
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    range_map = collect_range_map(lines)

    loop_rows: list[dict[str, object]] = []
    index = 0
    while index < len(lines):
        match = FOREACH_RE.match(lines[index])
        if not match:
            index += 1
            continue

        iter_var, list_var = (v.strip() for v in match.groups())
        body, end_index = extract_loop_body(lines, index)
        assigns: list[dict[str, str]] = []
        self_update_vars: set[str] = set()
        for body_line in body:
            assign_match = ASSIGN_RE.match(body_line)
            if not assign_match:
                continue
            lhs, rhs = (v.strip() for v in assign_match.groups())
            assigns.append({"lhs": lhs, "rhs": rhs})
            if has_self_update(lhs, rhs):
                self_update_vars.add(lhs)

        loop_rows.append(
            {
                "iter_var": iter_var,
                "list_var": list_var,
                "has_range_source": list_var in range_map,
                "range": range_map.get(list_var),
                "assign_count": len(assigns),
                "self_update_vars": sorted(self_update_vars),
                "state_vars": sorted({row["lhs"] for row in assigns if row["lhs"] != iter_var}),
            }
        )
        index = end_index + 1

    if not loop_rows:
        category = "no_sampler_loop"
    else:
        has_range_loop = any(bool(loop["has_range_source"]) for loop in loop_rows)
        has_self = any(bool(loop["self_update_vars"]) for loop in loop_rows)
        if has_range_loop and not has_self:
            category = "graph_sampler_only"
        elif has_self:
            category = "stateful_sim_candidate"
        else:
            category = "manual_review"

    proposal: dict[str, object] | None = None
    if category == "graph_sampler_only":
        primary = next((loop for loop in loop_rows if loop["has_range_source"]), None)
        if primary is not None:
            proposal = {
                "mode": "sampler_to_step_sim_v1",
                "time_var": primary["iter_var"],
                "range": primary["range"],
                "candidate_state_vars": primary["state_vars"],
                "notes": [
                    "현재는 범위 샘플러 기반 그래프 생성 패턴으로 감지됨",
                    "자동 변환은 위험하므로 preview 생성 후 수동 승인 권장",
                    "목표 형태: 시간 변수 누적(증분) + 상태 업데이트 + 관측(보임) 분리",
                ],
            }

    return {
        "path": str(path.relative_to(ROOT)),
        "category": category,
        "loop_count": len(loop_rows),
        "loops": loop_rows,
        "proposal": proposal,
    }


def main() -> int:
    args = parse_args()
    targets = iter_targets(include_inputs=bool(args.include_inputs))
    if not targets:
        print("targets=0")
        return 0

    rows = [classify_file(path) for path in targets]
    counts: dict[str, int] = {}
    for row in rows:
        category = str(row["category"])
        counts[category] = counts.get(category, 0) + 1

    out_path = ROOT / str(args.json_out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "seamgrim.sim_conversion_plan.v1",
        "targets": len(targets),
        "category_counts": counts,
        "rows": rows,
    }
    out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    print(f"targets={len(targets)}")
    print("category_counts:", " ".join(f"{k}={v}" for k, v in sorted(counts.items())))
    printed = 0
    limit = max(0, int(args.limit))
    for row in rows:
        if row["category"] != "graph_sampler_only":
            continue
        if printed >= limit:
            break
        printed += 1
        print(f"[graph-only] {row['path']} loops={row['loop_count']}")
    print(f"json_out={out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
