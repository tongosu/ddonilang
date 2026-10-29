from __future__ import annotations

from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
SURFACE_ROOTS = (
    ROOT / "pack",
    ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons",
    ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons_rewrite_v1",
    ROOT / "tools" / "teul-cli" / "tests" / "golden",
)
ALLOWED_SUFFIXES = {".ddn", ".json", ".jsonl", ".detjson", ".txt", ".md"}
ALLOWED_NAMES = {"ddn.schema.json", "test.dtest.json", "golden.jsonl"}
FORBIDDEN_PATTERNS = (
    r"살림\.",
    r"바탕\.",
    r"#바탕숨김",
    r"#암묵살림",
    r"//\s*범위\(",
    r"final_state_digest",
)
FORBIDDEN_RE = re.compile("|".join(f"(?:{pattern})" for pattern in FORBIDDEN_PATTERNS))


def should_scan(path: Path) -> bool:
    return path.name in ALLOWED_NAMES or path.suffix.lower() in ALLOWED_SUFFIXES


def main() -> int:
    failures: list[str] = []
    for surface_root in SURFACE_ROOTS:
        for path in sorted(surface_root.rglob("*")):
            if not path.is_file() or not should_scan(path):
                continue
            try:
                text = path.read_text(encoding="utf-8")
            except UnicodeDecodeError:
                continue
            match = FORBIDDEN_RE.search(text)
            if match:
                line_no = text[: match.start()].count("\n") + 1
                rel = path.relative_to(ROOT)
                failures.append(f"{rel}:{line_no}: {match.group(0)}")
    if failures:
        print("legacy surface tokens detected:")
        for item in failures[:200]:
            print(item)
        if len(failures) > 200:
            print(f"... {len(failures) - 200} more")
        return 1
    print("legacy surface scan PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
