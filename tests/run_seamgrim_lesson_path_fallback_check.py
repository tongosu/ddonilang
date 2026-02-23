#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


def has_all_patterns(text: str, patterns: list[str]) -> tuple[bool, str]:
    for pattern in patterns:
        if pattern not in text:
            return False, pattern
    return True, ""


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    app_path = root / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
    if not app_path.exists():
        print(f"missing ui js: {app_path.relative_to(root).as_posix()}")
        return 1

    text = app_path.read_text(encoding="utf-8")
    required_tokens = [
        'const PROJECT_PREFIX = "solutions/seamgrim_ui_mvp/";',
        "function buildPathCandidates(path)",
        "async function fetchFirstOk(urls, parseAs = \"text\")",
        "async function fetchJson(path)",
        "async function fetchText(pathCandidates)",
        "async function loadCatalogLessons()",
        'fetchJson("solutions/seamgrim_ui_mvp/lessons/index.json")',
        'fetchJson("solutions/seamgrim_ui_mvp/seed_lessons_v1/seed_manifest.detjson")',
        'fetchJson("solutions/seamgrim_ui_mvp/lessons_rewrite_v1/rewrite_manifest.detjson")',
        "const ddnText = await fetchText(base.ddnCandidates);",
        "const textMd = (await fetchText(base.textCandidates)) ?? \"\";",
        "const metaRaw = await fetchText(base.metaCandidates);",
    ]
    ok, missing = has_all_patterns(text, required_tokens)
    if not ok:
        print(f"check=lesson_path_fallback_tokens missing={missing}")
        return 1

    print("seamgrim lesson path fallback check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
