#!/usr/bin/env python
import argparse
from pathlib import Path
import shutil


def iter_lesson_dirs(ssot_root: Path):
    for path in ssot_root.rglob("lesson.ddn"):
        yield path.parent


def mirror_lesson(lesson_dir: Path, ssot_root: Path, pack_root: Path) -> Path:
    rel = lesson_dir.relative_to(ssot_root)
    parts = rel.parts
    if not parts:
        raise ValueError("empty lesson path")
    dest_root = pack_root / f"edu_{parts[0]}"
    dest = dest_root.joinpath(*parts[1:]) if len(parts) > 1 else dest_root
    dest.mkdir(parents=True, exist_ok=True)
    for src in lesson_dir.rglob("*"):
        if src.is_dir():
            continue
        rel_file = src.relative_to(lesson_dir)
        target = dest / rel_file
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, target)
    return dest


def main() -> int:
    parser = argparse.ArgumentParser(description="Mirror SSOT edu lessons into pack/edu_* folders")
    parser.add_argument("--root", default=None, help="repo root (default: script location/..)")
    args = parser.parse_args()

    root = Path(args.root).resolve() if args.root else Path(__file__).resolve().parent.parent
    ssot_root = root / "docs" / "ssot" / "pack" / "edu"
    pack_root = root / "pack"

    if not ssot_root.exists():
        print(f"missing SSOT edu root: {ssot_root}")
        return 1

    lessons = list(iter_lesson_dirs(ssot_root))
    if not lessons:
        print("no SSOT edu lessons found")
        return 0

    mirrored = []
    for lesson_dir in lessons:
        dest = mirror_lesson(lesson_dir, ssot_root, pack_root)
        mirrored.append((lesson_dir, dest))

    print("mirrored lessons:")
    for src, dst in mirrored:
        print(f"- {src} -> {dst}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
