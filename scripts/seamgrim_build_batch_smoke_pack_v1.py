#!/usr/bin/env python3
"""Build seamgrim curriculum batch smoke pack from rewrite manifest."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


def sanitize_name(value: str) -> str:
    text = value.strip().lower()
    text = re.sub(r"[^a-z0-9_]+", "_", text)
    text = re.sub(r"_+", "_", text).strip("_")
    return text or "lesson"


def checkpoint_ticks(subject: str) -> tuple[int, int]:
    s = subject.strip().lower()
    if s in {"economy", "econ"}:
        return (0, 15)
    return (0, 10)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--manifest",
        default="solutions/seamgrim_ui_mvp/lessons_rewrite_v1/rewrite_manifest.detjson",
        help="Rewrite manifest path",
    )
    parser.add_argument(
        "--pack-dir",
        default="pack/seamgrim_curriculum_batch_smoke_v1",
        help="Target smoke pack directory",
    )
    args = parser.parse_args()

    manifest_path = Path(args.manifest)
    pack_dir = Path(args.pack_dir)
    pack_dir.mkdir(parents=True, exist_ok=True)

    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    rows = list(manifest.get("generated", []))

    golden_lines: list[str] = []
    for row in rows:
        lesson_id = str(row.get("lesson_id", "")).strip()
        ddn_path = str(row.get("generated_lesson_ddn", "")).strip()
        subject = str(row.get("subject", "")).strip()
        if not lesson_id or not ddn_path:
            continue
        safe = sanitize_name(lesson_id)
        smoke_name = f"smoke_{safe}.v1.json"
        t0, t1 = checkpoint_ticks(subject)
        smoke_doc = {
            "schema": "ddn.smoke.golden.v1",
            "ddn_file": f"../../{ddn_path}",
            "contract": "D-STRICT",
            "checkpoints": [
                {"tick": t0, "state_hash": "blake3:pending"},
                {"tick": t1, "state_hash": "blake3:pending"},
            ],
            "max_ticks": t1,
            "timeout_ms": 30000,
        }
        (pack_dir / smoke_name).write_text(
            json.dumps(smoke_doc, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )
        golden_lines.append(json.dumps({"smoke_golden": smoke_name, "exit_code": 0}, ensure_ascii=False))

    (pack_dir / "golden.jsonl").write_text("\n".join(golden_lines) + "\n", encoding="utf-8")
    print(f"[ok] wrote {pack_dir / 'golden.jsonl'} ({len(golden_lines)} cases)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
