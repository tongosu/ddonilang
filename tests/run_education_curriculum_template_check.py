#!/usr/bin/env python
from __future__ import annotations

import argparse
import re
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "education_curriculum_template_v1"
SCHEMA = "CurriculumMetaV1"
VIEW_KINDS = {"text", "table", "graph", "space2d", "console_grid", "grid2d"}
ID_RE = re.compile(r"^[a-z0-9]+(?:[_-][a-z0-9]+)*$")

ALIASES = {
    "과목": "subject",
    "학년군": "grade_band",
    "단원": "unit",
    "차시": "lesson",
    "난이도": "difficulty",
    "학습목표": "learning_goals",
    "핵심개념": "core_concepts",
    "선수개념": "prerequisites",
    "오개념": "misconceptions",
    "허용조작": "allowed_controls",
    "필수계기판": "required_views",
}


@dataclass(frozen=True)
class ValidationError(Exception):
    code: str
    field: str
    detail: str

    def __str__(self) -> str:
        return f"{self.code}: {self.field}: {self.detail}"


def load_toml(path: Path) -> dict[str, Any]:
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as exc:
        raise ValidationError("E_EDU_CURRICULUM_TOML_INVALID", str(path), str(exc)) from exc
    if not isinstance(data, dict):
        raise ValidationError("E_EDU_CURRICULUM_ROOT_NOT_OBJECT", str(path), "root must be a table")
    return data


def canonicalize_keys(meta: dict[str, Any]) -> dict[str, Any]:
    out = dict(meta)
    for alias, canonical in ALIASES.items():
        if canonical not in out and alias in out:
            out[canonical] = out[alias]
    return out


def nonempty_string(meta: dict[str, Any], field: str, code: str | None = None) -> str:
    value = meta.get(field)
    if not isinstance(value, str) or not value.strip():
        raise ValidationError(code or f"E_EDU_CURRICULUM_{field.upper()}_INVALID", field, "must be a non-empty string")
    return value


def string_list(
    meta: dict[str, Any],
    field: str,
    *,
    allow_empty: bool,
    empty_code: str | None = None,
) -> list[str]:
    value = meta.get(field)
    if value is None and allow_empty:
        return []
    if not isinstance(value, list):
        raise ValidationError(f"E_EDU_CURRICULUM_{field.upper()}_INVALID", field, "must be a list")
    if not allow_empty and not value:
        raise ValidationError(empty_code or f"E_EDU_CURRICULUM_{field.upper()}_EMPTY", field, "must not be empty")
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item.strip():
            raise ValidationError(
                f"E_EDU_CURRICULUM_{field.upper()}_ITEM_INVALID",
                f"{field}[{index}]",
                "must be a non-empty string",
            )
    return value


def validate_schema(meta: dict[str, Any]) -> None:
    if "schema" not in meta:
        raise ValidationError("E_EDU_CURRICULUM_SCHEMA_MISSING", "schema", "field is required")
    if meta["schema"] != SCHEMA:
        raise ValidationError("E_EDU_CURRICULUM_SCHEMA_UNKNOWN", "schema", f"expected {SCHEMA}")


def validate_meta(raw_meta: dict[str, Any]) -> None:
    meta = canonicalize_keys(raw_meta)
    validate_schema(meta)
    lesson_id = nonempty_string(meta, "lesson_id")
    if not ID_RE.match(lesson_id):
        raise ValidationError("E_EDU_CURRICULUM_LESSON_ID_INVALID", "lesson_id", "must be stable snake/kebab lower-case id")
    for field in ("title", "subject", "grade_band", "unit", "lesson", "difficulty"):
        nonempty_string(meta, field)
    string_list(
        meta,
        "learning_goals",
        allow_empty=False,
        empty_code="E_EDU_CURRICULUM_LEARNING_GOALS_EMPTY",
    )
    string_list(meta, "core_concepts", allow_empty=False)
    for field in ("prerequisites", "misconceptions", "allowed_controls"):
        string_list(meta, field, allow_empty=True)
    required_views = string_list(meta, "required_views", allow_empty=False)
    for index, item in enumerate(required_views):
        if item not in VIEW_KINDS:
            raise ValidationError(
                "E_EDU_CURRICULUM_REQUIRED_VIEW_UNKNOWN",
                f"required_views[{index}]",
                f"expected one of {sorted(VIEW_KINDS)}",
            )
    string_list(
        meta,
        "evidence",
        allow_empty=False,
        empty_code="E_EDU_CURRICULUM_EVIDENCE_EMPTY",
    )
    for field in ("teacher_notes_ref", "student_sheet_ref"):
        value = meta.get(field)
        if value is not None and (not isinstance(value, str) or not value.strip()):
            raise ValidationError(f"E_EDU_CURRICULUM_{field.upper()}_INVALID", field, "must be a non-empty string when present")


def iter_toml_files(path: Path) -> list[Path]:
    if path.is_file():
        return [path]
    return sorted(p for p in path.glob("*.toml") if p.is_file())


def validate_valid_file(path: Path) -> None:
    meta = load_toml(path)
    if "expected_error" in meta:
        raise ValidationError("E_EDU_CURRICULUM_EXPECTED_ERROR_FORBIDDEN", "expected_error", "--file/--dir expects valid meta")
    validate_meta(meta)


def run_valid_paths(paths: list[Path]) -> int:
    checked = 0
    for path in paths:
        try:
            validate_valid_file(path)
        except ValidationError as exc:
            print(f"{path}: {exc}", file=sys.stderr)
            return 1
        checked += 1
    print(f"cases={checked} valid={checked} PASS")
    return 0


def run_default_pack() -> int:
    valid_files = iter_toml_files(PACK / "valid")
    invalid_files = iter_toml_files(PACK / "invalid")
    failures: list[str] = []

    for path in valid_files:
        try:
            validate_valid_file(path)
        except ValidationError as exc:
            failures.append(f"{path}: expected PASS, got {exc}")

    for path in invalid_files:
        try:
            fixture = load_toml(path)
            expected_error = fixture.pop("expected_error", None)
            if not isinstance(expected_error, str) or not expected_error.strip():
                failures.append(f"{path}: missing expected_error")
                continue
            try:
                validate_meta(fixture)
            except ValidationError as exc:
                if exc.code != expected_error:
                    failures.append(f"{path}: expected {expected_error}, got {exc}")
                continue
            failures.append(f"{path}: expected {expected_error}, got PASS")
        except ValidationError as exc:
            failures.append(f"{path}: fixture load failed: {exc}")

    if failures:
        for failure in failures:
            print(failure, file=sys.stderr)
        return 1

    cases = len(valid_files) + len(invalid_files)
    print(f"cases={cases} valid={len(valid_files)} invalid={len(invalid_files)} PASS")
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Validate CurriculumMetaV1 fixtures.")
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--file", type=Path, help="Validate one valid curriculum meta TOML.")
    group.add_argument("--dir", type=Path, help="Validate all *.toml valid curriculum meta files in a directory.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.file:
        return run_valid_paths([args.file])
    if args.dir:
        paths = iter_toml_files(args.dir)
        if not paths:
            print(f"{args.dir}: no *.toml files found", file=sys.stderr)
            return 1
        return run_valid_paths(paths)
    return run_default_pack()


if __name__ == "__main__":
    raise SystemExit(main())
