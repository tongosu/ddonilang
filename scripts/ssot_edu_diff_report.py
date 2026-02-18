#!/usr/bin/env python
import argparse
from pathlib import Path
from datetime import datetime


def iter_lessons(root: Path):
    for path in root.rglob("lesson.ddn"):
        yield path.parent


def to_pack_path(lesson_dir: Path, ssot_root: Path, pack_root: Path) -> Path:
    rel = lesson_dir.relative_to(ssot_root)
    parts = rel.parts
    dest_root = pack_root / f"edu_{parts[0]}"
    return dest_root.joinpath(*parts[1:]) if len(parts) > 1 else dest_root


def main() -> int:
    parser = argparse.ArgumentParser(description="SSOT edu mirror diff report")
    parser.add_argument("--root", default=None, help="repo root")
    parser.add_argument("--out", default=None, help="report path")
    args = parser.parse_args()

    root = Path(args.root).resolve() if args.root else Path(__file__).resolve().parent.parent
    ssot_root = root / "docs" / "ssot" / "pack" / "edu"
    pack_root = root / "pack"

    lessons = list(iter_lessons(ssot_root))
    ssot_pairs = [(lesson, to_pack_path(lesson, ssot_root, pack_root)) for lesson in lessons]

    pack_lessons = []
    for pack_dir in pack_root.glob("edu_*"):
        if not pack_dir.is_dir():
            continue
        for path in pack_dir.rglob("lesson.ddn"):
            pack_lessons.append(path.parent)

    missing = [dst for _, dst in ssot_pairs if not dst.exists()]
    extra = [pack for pack in pack_lessons if not any(pack == dst for _, dst in ssot_pairs)]

    now = datetime.now()
    stamp = now.strftime("%Y%m%d_%H%M")
    out_path = Path(args.out) if args.out else root / "docs" / "reports" / "impl" / f"REPORT_{stamp}_SSOT_EDU_DIFF.md"

    lines = []
    lines.append(f"# REPORT_{stamp}_SSOT_EDU_DIFF.md")
    lines.append("")
    lines.append("## generated_at")
    lines.append(f"- {now.strftime('%Y-%m-%d')}")
    lines.append("")
    lines.append("## scope")
    lines.append("- SSOT edu(pack) ↔ pack/edu_* 미러 현황 비교")
    lines.append("")
    lines.append("## summary")
    lines.append(f"- ssot_lessons: {len(ssot_pairs)}")
    lines.append(f"- pack_lessons: {len(pack_lessons)}")
    lines.append(f"- missing_mirrors: {len(missing)}")
    lines.append(f"- extra_mirrors: {len(extra)}")
    lines.append("")
    lines.append("## missing mirrors")
    if missing:
        for path in missing:
            lines.append(f"- {path}")
    else:
        lines.append("- (none)")
    lines.append("")
    lines.append("## extra mirrors")
    if extra:
        for path in extra:
            lines.append(f"- {path}")
    else:
        lines.append("- (none)")
    lines.append("")
    lines.append("## mapping")
    for src, dst in ssot_pairs:
        lines.append(f"- {src} -> {dst}")

    out_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
