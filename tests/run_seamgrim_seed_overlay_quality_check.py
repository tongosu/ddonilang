#!/usr/bin/env python
from __future__ import annotations

import json
import re
from pathlib import Path


REQUIRED_SUBSTRINGS = [
    ("section_observe", "## 관찰 안내"),
    ("section_bogae_madang", "## 보개마당 안내"),
    ("axis_line", "기본 관찰축"),
    ("execution_syntax", "실행 문법"),
]

REQUIRED_PATTERNS = [
    ("bogae_intro", re.compile(r"^\s*-\s*도입\s*:\s*.+$", re.MULTILINE)),
    ("bogae_observe", re.compile(r"^\s*-\s*관찰\s*:\s*.+$", re.MULTILINE)),
    ("bogae_summary", re.compile(r"^\s*-\s*정리\s*:\s*.+$", re.MULTILINE)),
]

FORBIDDEN_SUBSTRINGS = [
    ("legacy_compat_header", "호환형"),
    ("legacy_bogae_meta", "#보개마당_"),
    ("legacy_future_note", "정식 블록 전환은 후속 단계"),
]


def fail(detail: str) -> int:
    print(f"check=seed_overlay_quality detail={detail}")
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    manifest_path = root / "solutions" / "seamgrim_ui_mvp" / "seed_lessons_v1" / "seed_manifest.detjson"
    if not manifest_path.exists():
        return fail(f"manifest_missing:{manifest_path.as_posix()}")

    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except Exception as exc:
        return fail(f"manifest_parse_failed:{exc}")

    seeds = manifest.get("seeds") if isinstance(manifest, dict) else None
    if not isinstance(seeds, list) or not seeds:
        return fail("manifest_seeds_missing")

    issues: list[str] = []
    checked = 0
    for row in seeds:
        if not isinstance(row, dict):
            continue
        seed_id = str(row.get("seed_id") or "").strip() or "(unknown)"
        text_rel = str(row.get("text_md") or "").strip()
        if not text_rel:
            issues.append(f"{seed_id}:text_md_missing")
            continue
        text_path = (root / text_rel).resolve()
        if not text_path.exists():
            issues.append(f"{seed_id}:text_missing:{text_rel}")
            continue

        checked += 1
        text = text_path.read_text(encoding="utf-8")
        for code, token in REQUIRED_SUBSTRINGS:
            if token not in text:
                issues.append(f"{seed_id}:{code}")
        for code, pattern in REQUIRED_PATTERNS:
            if not pattern.search(text):
                issues.append(f"{seed_id}:{code}")
        for code, token in FORBIDDEN_SUBSTRINGS:
            if token in text:
                issues.append(f"{seed_id}:{code}")

    if checked <= 0:
        return fail("no_seed_text_checked")
    if issues:
        head = ", ".join(issues[:10])
        tail = f", ... ({len(issues) - 10} more)" if len(issues) > 10 else ""
        return fail(f"violations:{head}{tail}")

    print(f"seamgrim seed overlay quality check ok count={checked}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
