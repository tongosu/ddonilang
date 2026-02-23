#!/usr/bin/env python
from __future__ import annotations

import argparse
import re
from pathlib import Path


FORMULA_RE = re.compile(r"\(#ascii1?\)\s*수식\{([^}]*)\}")
FUNC_CALL_RE = re.compile(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(")
DECIMAL_EXP_RE = re.compile(r"\^\s*[-+]?(?:\d+\.\d+|\.\d+)")


def fail(message: str) -> int:
    print(message)
    return 1


def line_of(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def collect_targets(root: Path, scope: str) -> list[tuple[str, list[Path]]]:
    targets: list[tuple[str, list[Path]]] = []
    if scope in ("seed", "all"):
        seed_root = root / "solutions" / "seamgrim_ui_mvp" / "seed_lessons_v1"
        if not seed_root.exists():
            raise FileNotFoundError(f"missing target root: label=seed path={seed_root}")
        seed_lessons = sorted(seed_root.glob("*/lesson.ddn"))
        if not seed_lessons:
            raise FileNotFoundError(f"no lesson files found under target: label=seed path={seed_root}")
        targets.append(("seed", seed_lessons))

    if scope in ("lessons", "all"):
        lessons_root = root / "solutions" / "seamgrim_ui_mvp" / "lessons"
        if not lessons_root.exists():
            raise FileNotFoundError(f"missing target root: label=lessons path={lessons_root}")
        lesson_main = sorted(lessons_root.glob("*/lesson.ddn"))
        lesson_inputs = sorted(lessons_root.glob("*/inputs/*.ddn"))
        lesson_files = lesson_main + lesson_inputs
        if not lesson_files:
            raise FileNotFoundError(f"no lesson files found under target: label=lessons path={lessons_root}")
        targets.append(("lessons", lesson_files))

    if scope in ("samples", "all"):
        samples_root = root / "solutions" / "seamgrim_ui_mvp" / "samples"
        if not samples_root.exists():
            raise FileNotFoundError(f"missing target root: label=samples path={samples_root}")
        sample_files = sorted(samples_root.rglob("*.ddn"))
        if not sample_files:
            raise FileNotFoundError(f"no lesson files found under target: label=samples path={samples_root}")
        targets.append(("samples", sample_files))

    return targets


def main() -> int:
    parser = argparse.ArgumentParser(description="Gate: seamgrim formula compatibility check")
    parser.add_argument(
        "--scope",
        choices=("seed", "lessons", "samples", "all"),
        default="all",
        help="검사 범위 (기본: all)",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    try:
        grouped_targets = collect_targets(root, args.scope)
    except FileNotFoundError as exc:
        return fail(str(exc))

    issues: list[tuple[str, int, str, str]] = []
    for scope_label, lesson_files in grouped_targets:
        for lesson_path in lesson_files:
            text = lesson_path.read_text(encoding="utf-8")
            rel = lesson_path.relative_to(root).as_posix()
            for match in FORMULA_RE.finditer(text):
                body = match.group(1).strip()
                line = line_of(text, match.start())

                call_match = FUNC_CALL_RE.search(body)
                if call_match:
                    issues.append((f"{scope_label}:{rel}", line, "formula_function_call", body))
                    continue

                if DECIMAL_EXP_RE.search(body):
                    issues.append((f"{scope_label}:{rel}", line, "formula_decimal_exponent", body))

    if issues:
        for rel, line, kind, body in issues:
            print(f"check={kind} file={rel}:{line} expr={body}")
        return fail(f"seamgrim formula compat check failed: {len(issues)} issue(s)")

    print("seamgrim formula compat check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
