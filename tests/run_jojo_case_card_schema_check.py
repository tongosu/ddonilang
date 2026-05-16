from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "jojo_case_card_schema_v1"
SCHEMA = "ddn.jojo.case_card.v1"
DOMAINS = {"economics", "social", "history"}
CASE_KINDS = {"reduced_form_model", "large_number_simulation", "historical_scenario"}
VIEW_KINDS = {"text", "table", "graph", "space2d", "console_grid"}
ID_RE = re.compile(r"^[a-z0-9]+(?:[_-][a-z0-9]+)*$")


@dataclass(frozen=True)
class ValidationError(Exception):
    code: str
    field: str
    detail: str

    def __str__(self) -> str:
        return f"{self.code}: {self.field}: {self.detail}"


def load_json(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValidationError("E_JOJO_CARD_JSON_INVALID", str(path), str(exc)) from exc
    if not isinstance(data, dict):
        raise ValidationError("E_JOJO_CARD_ROOT_NOT_OBJECT", str(path), "root must be an object")
    return data


def nonempty_string(card: dict[str, Any], field: str, code: str | None = None) -> str:
    value = card.get(field)
    if not isinstance(value, str) or not value.strip():
        raise ValidationError(code or f"E_JOJO_CARD_{field.upper()}_INVALID", field, "must be a non-empty string")
    return value


def string_list(
    card: dict[str, Any],
    field: str,
    *,
    allow_empty: bool,
    empty_code: str | None = None,
) -> list[str]:
    value = card.get(field)
    if not isinstance(value, list):
        raise ValidationError(f"E_JOJO_CARD_{field.upper()}_INVALID", field, "must be a list")
    if not allow_empty and not value:
        raise ValidationError(empty_code or f"E_JOJO_CARD_{field.upper()}_EMPTY", field, "must not be empty")
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item.strip():
            raise ValidationError(
                f"E_JOJO_CARD_{field.upper()}_ITEM_INVALID",
                f"{field}[{index}]",
                "must be a non-empty string",
            )
    return value


def validate_schema(card: dict[str, Any]) -> None:
    if "schema" not in card:
        raise ValidationError("E_JOJO_CARD_SCHEMA_MISSING", "schema", "field is required")
    if card["schema"] != SCHEMA:
        raise ValidationError("E_JOJO_CARD_SCHEMA_UNKNOWN", "schema", f"expected {SCHEMA}")


def validate_id(card: dict[str, Any]) -> None:
    card_id = nonempty_string(card, "id")
    if not ID_RE.match(card_id):
        raise ValidationError("E_JOJO_CARD_ID_INVALID", "id", "must be stable snake/kebab lower-case id")


def validate_domain(card: dict[str, Any]) -> None:
    domain = nonempty_string(card, "domain")
    if domain not in DOMAINS:
        raise ValidationError("E_JOJO_CARD_DOMAIN_UNKNOWN", "domain", f"expected one of {sorted(DOMAINS)}")


def validate_case_kind(card: dict[str, Any]) -> None:
    case_kind = nonempty_string(card, "case_kind")
    if case_kind not in CASE_KINDS:
        raise ValidationError("E_JOJO_CARD_CASE_KIND_UNKNOWN", "case_kind", f"expected one of {sorted(CASE_KINDS)}")


def validate_view_requirements(card: dict[str, Any]) -> None:
    value = card.get("view_requirements")
    if not isinstance(value, list):
        raise ValidationError("E_JOJO_CARD_VIEW_REQUIREMENTS_INVALID", "view_requirements", "must be a list")
    if not value:
        raise ValidationError("E_JOJO_CARD_VIEW_REQUIREMENTS_EMPTY", "view_requirements", "must not be empty")
    for index, item in enumerate(value):
        field = f"view_requirements[{index}]"
        if not isinstance(item, dict):
            raise ValidationError("E_JOJO_CARD_VIEW_REQUIREMENT_INVALID", field, "must be an object")
        kind = item.get("kind")
        if kind not in VIEW_KINDS:
            raise ValidationError("E_JOJO_CARD_VIEW_KIND_UNKNOWN", f"{field}.kind", f"expected one of {sorted(VIEW_KINDS)}")
        purpose = item.get("purpose")
        if not isinstance(purpose, str) or not purpose.strip():
            raise ValidationError("E_JOJO_CARD_VIEW_PURPOSE_INVALID", f"{field}.purpose", "must be a non-empty string")


def validate_truth_contract(card: dict[str, Any]) -> None:
    value = card.get("truth_contract")
    if not isinstance(value, dict):
        raise ValidationError("E_JOJO_CARD_TRUTH_CONTRACT_INVALID", "truth_contract", "must be an object")
    state_fields = string_list(
        value,
        "state_hash_fields",
        allow_empty=False,
        empty_code="E_JOJO_CARD_STATE_HASH_FIELDS_EMPTY",
    )
    view_fields = string_list(value, "view_only_fields", allow_empty=True)
    overlap = sorted(set(state_fields).intersection(view_fields))
    if overlap:
        raise ValidationError("E_JOJO_CARD_TRUTH_VIEW_OVERLAP", "truth_contract", ", ".join(overlap))


def validate_card(card: dict[str, Any]) -> None:
    validate_schema(card)
    validate_id(card)
    nonempty_string(card, "title")
    validate_domain(card)
    validate_case_kind(card)
    string_list(
        card,
        "theory_basis",
        allow_empty=False,
        empty_code="E_JOJO_CARD_THEORY_BASIS_EMPTY",
    )
    nonempty_string(card, "educational_goal")
    string_list(card, "assumptions", allow_empty=False)
    string_list(card, "inputs", allow_empty=True)
    string_list(card, "outputs", allow_empty=False)
    validate_view_requirements(card)
    validate_truth_contract(card)
    string_list(card, "limits", allow_empty=False)
    string_list(card, "evidence", allow_empty=True)


def iter_detjson_files(path: Path) -> list[Path]:
    if path.is_file():
        return [path]
    return sorted(p for p in path.glob("*.detjson") if p.is_file())


def validate_valid_file(path: Path) -> None:
    card = load_json(path)
    if "expected_error" in card:
        raise ValidationError("E_JOJO_CARD_EXPECTED_ERROR_FORBIDDEN", "expected_error", "--file/--dir expects valid cards")
    validate_card(card)


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
    valid_files = iter_detjson_files(PACK / "valid")
    invalid_files = iter_detjson_files(PACK / "invalid")
    failures: list[str] = []

    for path in valid_files:
        try:
            validate_valid_file(path)
        except ValidationError as exc:
            failures.append(f"{path}: expected PASS, got {exc}")

    for path in invalid_files:
        try:
            fixture = load_json(path)
            expected_error = fixture.pop("expected_error", None)
            if not isinstance(expected_error, str) or not expected_error.strip():
                failures.append(f"{path}: missing expected_error")
                continue
            try:
                validate_card(fixture)
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
    parser = argparse.ArgumentParser(description="Validate JOJO_CASE_CARD_V1 fixtures.")
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--file", type=Path, help="Validate one valid case card.")
    group.add_argument("--dir", type=Path, help="Validate all *.detjson valid case cards in a directory.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.file:
        return run_valid_paths([args.file])
    if args.dir:
        paths = iter_detjson_files(args.dir)
        if not paths:
            print(f"{args.dir}: no *.detjson files found", file=sys.stderr)
            return 1
        return run_valid_paths(paths)
    return run_default_pack()


if __name__ == "__main__":
    raise SystemExit(main())
