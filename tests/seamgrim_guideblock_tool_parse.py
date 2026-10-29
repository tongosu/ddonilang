#!/usr/bin/env python
from __future__ import annotations

import argparse
import importlib.util
import json
from pathlib import Path


def load_export_graph_module(root: Path):
    script_path = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "export_graph.py"
    spec = importlib.util.spec_from_file_location("seamgrim_export_graph", script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"module load failed: {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> int:
    parser = argparse.ArgumentParser(description="Parse guideblock/meta header using export_graph parser")
    parser.add_argument("--input", required=True, help="ddn input path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    input_path = Path(args.input)
    if not input_path.is_absolute():
        input_path = (root / input_path).resolve()
    if not input_path.exists():
        print(json.dumps({"error": f"missing_input:{input_path.as_posix()}"}, ensure_ascii=False))
        return 1

    module = load_export_graph_module(root)
    text = input_path.read_text(encoding="utf-8")
    canon_meta, raw_meta, body = module.parse_guide_meta_header(text)
    payload = {
        "meta": canon_meta,
        "rawMeta": raw_meta,
        "body": body,
    }
    print(json.dumps(payload, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
