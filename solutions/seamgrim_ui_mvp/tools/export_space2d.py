#!/usr/bin/env python
"""DDN 실행 결과에서 seamgrim.space2d.v0 JSON을 추출한다."""
import argparse
import json
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

TOOLS_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS_DIR))

from export_graph import (
    run_teul_cli,
    hash_text,
    extract_meta,
    normalize_ddn_for_hash,
    preprocess_ddn_for_teul,
)
from bridge_server import parse_space2d_points, parse_space2d_shapes


def build_space2d(points: list[dict], shapes: list[dict], source_hash: str, meta: dict) -> dict:
    result: dict = {
        "schema": "seamgrim.space2d.v0",
        "points": points,
        "shapes": shapes,
        "meta": {
            "title": meta.get("name") or "space2d",
            "source_input_hash": source_hash,
        },
    }
    if points:
        xs = [p["x"] for p in points]
        ys = [p["y"] for p in points]
        result["camera"] = {
            "x_min": min(xs),
            "x_max": max(xs),
            "y_min": min(ys),
            "y_max": max(ys),
        }
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description="Export seamgrim.space2d.v0 from DDN output")
    parser.add_argument("input", help="DDN 파일 경로")
    parser.add_argument("output", nargs="?", help="출력 JSON 경로")
    parser.add_argument("--output-dir", dest="output_dir", help="출력 디렉토리")
    parser.add_argument("--auto-name", action="store_true", help="타임스탬프 포함 자동 이름")
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[3]
    input_path = Path(args.input).resolve()
    output_path: Optional[Path] = None
    if args.output:
        output_path = Path(args.output).resolve()
    elif args.output_dir:
        output_dir = Path(args.output_dir).resolve()
        stem = input_path.stem
        if args.auto_name:
            stamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
            name = f"{stem}_{stamp}_space2d.json"
        else:
            name = f"{stem}_space2d.json"
        output_path = output_dir / name
    else:
        raise ValueError("출력 경로가 필요합니다 (OUTPUT 또는 --output-dir)")

    input_text = input_path.read_text(encoding="utf-8")
    meta = extract_meta(input_text)
    source_hash = hash_text(normalize_ddn_for_hash(input_text))
    stripped_text = preprocess_ddn_for_teul(input_text, strip_draw=True)
    teul_input_path = input_path
    temp_path: Optional[Path] = None
    if stripped_text != input_text:
        temp = tempfile.NamedTemporaryFile("w", suffix=".ddn", delete=False, encoding="utf-8", newline="\n")
        temp.write(stripped_text)
        temp.flush()
        temp.close()
        temp_path = Path(temp.name)
        teul_input_path = temp_path
    try:
        lines = run_teul_cli(root, teul_input_path)
    finally:
        if temp_path and temp_path.exists():
            temp_path.unlink()

    points = parse_space2d_points(lines)
    shapes = parse_space2d_shapes(lines)
    if not points and not shapes:
        print("경고: space2d 데이터가 없습니다.", file=sys.stderr)
        return 1

    space2d = build_space2d(points, shapes, source_hash, meta)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        json.dumps(space2d, ensure_ascii=False, indent=2, sort_keys=True),
        encoding="utf-8",
    )
    print(f"ok: {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
