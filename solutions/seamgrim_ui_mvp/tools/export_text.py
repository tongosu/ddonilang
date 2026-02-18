#!/usr/bin/env python
"""DDN 실행 결과에서 seamgrim.text.v0 JSON을 추출한다."""
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
from bridge_server import parse_text_blocks


def build_text_doc(content: str, source_hash: str, meta: dict) -> dict:
    return {
        "schema": "seamgrim.text.v0",
        "content": content,
        "format": "markdown",
        "meta": {
            "title": meta.get("name") or "text",
            "source_input_hash": source_hash,
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Export seamgrim.text.v0 from DDN output")
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
            name = f"{stem}_{stamp}_text.json"
        else:
            name = f"{stem}_text.json"
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

    text_content = parse_text_blocks(lines)
    if not text_content:
        print("경고: text 데이터가 없습니다.", file=sys.stderr)
        return 1

    text_doc = build_text_doc(text_content, source_hash, meta)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        json.dumps(text_doc, ensure_ascii=False, indent=2, sort_keys=True),
        encoding="utf-8",
    )
    print(f"ok: {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
