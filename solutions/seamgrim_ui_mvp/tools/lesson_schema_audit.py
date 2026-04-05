#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
LESSONS_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
DEFAULT_PROMOTE_BACKUP_SUFFIX = ".before_age3_promote.bak"

GUIDE_META_ALIASES: dict[str, tuple[str, ...]] = {
    "name": (
        "이름",
        "name",
        "title",
        "제목",
        "표제",
        "헤드라인",
        "headline",
        "caption",
        "label",
        "guide_name",
        "guide-name",
        "가이드이름",
        "가이드_이름",
        "ガイド名",
        "タイトル",
    ),
    "desc": (
        "설명",
        "풀이",
        "해설",
        "설명글",
        "안내",
        "요지",
        "description",
        "desc",
        "summary",
        "요약",
        "guide_desc",
        "guide-desc",
        "guide_summary",
        "guide-summary",
        "subtitle",
        "sub_title",
        "sub-title",
        "caption_text",
        "guide_text",
        "guide-text",
        "説明",
        "解説",
    ),
    "default_observation": (
        "기본관찰",
        "기본관측",
        "기본관찰y",
        "기본관측y",
        "기본축y",
        "기본y축",
        "기본시리즈",
        "기본계열",
        "default_obs",
        "default-observation",
        "default_observation",
        "default_y",
        "default-y",
        "default_series",
        "default-series",
        "default_signal",
        "default-signal",
        "obs",
        "observation",
        "series",
        "y_axis",
        "y-axis",
        "yaxis",
        "既定観測",
        "既定系列",
    ),
    "default_observation_x": (
        "기본관찰x",
        "기본관측x",
        "기본x축",
        "기본축x",
        "기본관찰X",
        "기본관측X",
        "기본X축",
        "기본축X",
        "기본가로축",
        "default_obs_x",
        "default-observation-x",
        "default_observation_x",
        "default_x",
        "default-x",
        "default_x_axis",
        "default-x-axis",
        "default_xaxis",
        "x_axis",
        "x-axis",
        "xaxis",
        "既定観測X",
        "既定横軸",
    ),
}


def build_meta_header_pattern(aliases: tuple[str, ...]) -> re.Pattern[str]:
    escaped = [re.escape(alias) for alias in aliases]
    union = "|".join(escaped)
    return re.compile(rf"^\s*#(?:{union})\s*:", re.MULTILINE | re.IGNORECASE)


PATTERNS = {
    "meta_name": build_meta_header_pattern(GUIDE_META_ALIASES["name"]),
    "meta_desc": build_meta_header_pattern(GUIDE_META_ALIASES["desc"]),
    "meta_default_observation": build_meta_header_pattern(GUIDE_META_ALIASES["default_observation"]),
    "meta_default_observation_x": build_meta_header_pattern(GUIDE_META_ALIASES["default_observation_x"]),
    "legacy_show": re.compile(r"보여주기\s*\.", re.MULTILINE),
    "legacy_solver_bind_eq": re.compile(r"\([^)]*=[^)]*\)\s*인\s*[^\n.]*풀기\s*\.", re.MULTILINE),
    "legacy_storage_block": re.compile(r"^\s*그릇채비\s*:", re.MULTILINE),
    "modern_view_block": re.compile(r"\b보임\s*\{", re.MULTILINE),
    "modern_tick_block": re.compile(r"\b매틱\s*:", re.MULTILINE),
    "modern_tick_hook": re.compile(r"\(\s*매마디\s*\)\s*마다\s*:?", re.MULTILINE),
}


def scan_file(path: Path) -> dict[str, int]:
    text = path.read_text(encoding="utf-8")
    out: dict[str, int] = {}
    for key, pattern in PATTERNS.items():
        out[key] = len(pattern.findall(text))
    return out


def merge_counts(total: dict[str, int], row: dict[str, int]) -> None:
    for key, value in row.items():
        total[key] = total.get(key, 0) + int(value)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Scan seamgrim lesson.ddn/input presets for legacy vs modern schema patterns."
    )
    parser.add_argument("--json-out", help="optional report path")
    parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="max file rows to print in console (default: 20)",
    )
    parser.add_argument(
        "--include-preview",
        action="store_true",
        help="*.age3.preview.ddn 파일도 스캔에 포함합니다. (기본: 제외)",
    )
    args = parser.parse_args()

    if not LESSONS_ROOT.exists():
        raise SystemExit(f"lessons root not found: {LESSONS_ROOT}")

    files = sorted(LESSONS_ROOT.rglob("*.ddn"))
    if not args.include_preview:
        files = [file_path for file_path in files if not file_path.stem.endswith(".age3.preview")]
    files = [file_path for file_path in files if not file_path.stem.endswith(DEFAULT_PROMOTE_BACKUP_SUFFIX)]
    rows: list[dict[str, object]] = []
    total: dict[str, int] = {key: 0 for key in PATTERNS}
    for file_path in files:
        counts = scan_file(file_path)
        merge_counts(total, counts)
        legacy_score = counts["legacy_show"] + counts["legacy_solver_bind_eq"] + counts["legacy_storage_block"]
        modern_score = counts["modern_view_block"] + counts["modern_tick_block"] + counts["modern_tick_hook"]
        rows.append(
            {
                "path": str(file_path.relative_to(ROOT)),
                **counts,
                "legacy_score": legacy_score,
                "modern_score": modern_score,
            }
        )

    rows_sorted = sorted(rows, key=lambda x: int(x["legacy_score"]), reverse=True)
    limit = max(0, int(args.limit))
    print(f"files={len(rows_sorted)}")
    print(
        "summary:",
        " ".join([f"{k}={v}" for k, v in total.items()]),
    )
    for row in rows_sorted[:limit]:
        if int(row["legacy_score"]) <= 0:
            continue
        print(
            f"[legacy] {row['path']} score={row['legacy_score']} "
            f"show={row['legacy_show']} solver_eq={row['legacy_solver_bind_eq']} block={row['legacy_storage_block']}"
        )

    if args.json_out:
        out_path = Path(args.json_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        report = {
            "schema": "seamgrim.lesson.schema_audit.v1",
            "root": str(LESSONS_ROOT),
            "files": len(rows_sorted),
            "totals": total,
            "rows": rows_sorted,
        }
        out_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
