#!/usr/bin/env python3
"""Build minimal core smoke pack from rewrite manifest for CI quick gate."""

from __future__ import annotations

import argparse
import json
import re
from collections import defaultdict
from pathlib import Path


def sanitize_name(value: str) -> str:
    text = value.strip().lower()
    text = re.sub(r"[^a-z0-9_]+", "_", text)
    text = re.sub(r"_+", "_", text).strip("_")
    return text or "lesson"


def checkpoint_ticks(subject: str) -> tuple[int, int]:
    s = subject.strip().lower()
    if s in {"economy", "econ"}:
        return (0, 10)
    return (0, 6)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--manifest",
        default="solutions/seamgrim_ui_mvp/lessons_rewrite_v1/rewrite_manifest.detjson",
        help="Rewrite manifest path",
    )
    parser.add_argument(
        "--pack-dir",
        default="pack/seamgrim_curriculum_rewrite_core_smoke_v1",
        help="Target core smoke pack directory",
    )
    parser.add_argument(
        "--max-per-subject",
        type=int,
        default=2,
        help="Max lessons per subject for core pack",
    )
    args = parser.parse_args()

    manifest = json.loads(Path(args.manifest).read_text(encoding="utf-8"))
    rows = list(manifest.get("generated", []))
    pack_dir = Path(args.pack_dir)
    pack_dir.mkdir(parents=True, exist_ok=True)

    # pick deterministic lessons: diverse templates first, then fill by lesson_id order
    by_subject = defaultdict(list)
    for row in rows:
        subject = str(row.get("subject", "")).strip().lower()
        if not subject:
            continue
        by_subject[subject].append(row)
    for subject in by_subject:
        by_subject[subject].sort(key=lambda x: str(x.get("lesson_id", "")))

    selected = []
    for subject, items in sorted(by_subject.items()):
        chosen = []
        seen_templates = set()
        for row in items:
            template_id = str(row.get("template_id", "")).strip()
            if template_id and template_id not in seen_templates:
                chosen.append(row)
                seen_templates.add(template_id)
            if len(chosen) >= max(1, args.max_per_subject):
                break
        if len(chosen) < max(1, args.max_per_subject):
            used_ids = {str(x.get("lesson_id", "")) for x in chosen}
            for row in items:
                lid = str(row.get("lesson_id", ""))
                if lid in used_ids:
                    continue
                chosen.append(row)
                if len(chosen) >= max(1, args.max_per_subject):
                    break
        selected.extend(chosen)

    golden_lines = []
    sample_rows = []
    for row in selected:
        lesson_id = str(row.get("lesson_id", "")).strip()
        ddn_path = str(row.get("generated_lesson_ddn", "")).strip()
        subject = str(row.get("subject", "")).strip().lower()
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
        sample_rows.append(
            {
                "lesson_id": lesson_id,
                "subject": subject,
                "template_id": row.get("template_id", ""),
                "smoke_file": smoke_name,
            }
        )

    (pack_dir / "golden.jsonl").write_text("\n".join(golden_lines) + "\n", encoding="utf-8")
    report = {
        "schema": "seamgrim.rewrite.core_manifest.v1",
        "max_per_subject": args.max_per_subject,
        "count": len(sample_rows),
        "samples": sample_rows,
    }
    (pack_dir / "core_manifest.detjson").write_text(
        json.dumps(report, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    print(f"[ok] wrote {pack_dir / 'golden.jsonl'} ({len(sample_rows)} cases)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
