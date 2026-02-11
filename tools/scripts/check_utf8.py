#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path


TEXT_EXTS = {
    ".md",
    ".ddn",
    ".json",
    ".toml",
    ".rs",
    ".ps1",
    ".txt",
    ".detjson",
    ".jsonl",
    ".dtest.json",
    ".test.json",
    ".schema.json",
    ".yml",
    ".yaml",
    ".ini",
    ".cfg",
    ".sh",
    ".bat",
    ".cmd",
    ".html",
    ".js",
    ".css",
    ".xml",
}


def is_text_candidate(path: Path) -> bool:
    suffix = path.suffix.lower()
    if suffix in TEXT_EXTS:
        return True
    name = path.name.lower()
    return name.endswith(".dtest.json") or name.endswith(".test.json") or name.endswith(".schema.json")


def scan(root: Path) -> tuple[list[Path], list[Path], list[Path]]:
    utf8_bom = b"\xef\xbb\xbf"
    bom_files: list[Path] = []
    non_utf8: list[Path] = []
    has_replacement: list[Path] = []

    for path in root.rglob("*"):
        if path.is_dir():
            continue
        if not is_text_candidate(path):
            continue
        try:
            data = path.read_bytes()
        except Exception:
            continue
        if b"\x00" in data:
            continue
        if data.startswith(utf8_bom):
            bom_files.append(path)
        try:
            text = data.decode("utf-8")
        except UnicodeDecodeError:
            non_utf8.append(path)
            continue
        if "\ufffd" in text:
            has_replacement.append(path)
    return bom_files, non_utf8, has_replacement


def main() -> int:
    parser = argparse.ArgumentParser(description="Check UTF-8 (no BOM) text files.")
    parser.add_argument("--root", default=".", help="Root directory to scan.")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    bom_files, non_utf8, has_replacement = scan(root)

    print(f"root={root}")
    print(f"bom_count={len(bom_files)}")
    for path in bom_files:
        print(f"  bom: {path.as_posix()}")
    print(f"non_utf8_count={len(non_utf8)}")
    for path in non_utf8:
        print(f"  non_utf8: {path.as_posix()}")
    print(f"replacement_char_count={len(has_replacement)}")
    for path in has_replacement:
        print(f"  replacement: {path.as_posix()}")

    return 0 if not bom_files and not non_utf8 and not has_replacement else 1


if __name__ == "__main__":
    raise SystemExit(main())
