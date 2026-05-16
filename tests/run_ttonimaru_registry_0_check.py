#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "ttonimaru_registry_0_v1"
SCHEMA = "ddn.ttonimaru.platform_charter.v1"
OBJECTS = {"lesson", "project", "package", "artifact"}
SHARE_KINDS = {"link", "clone", "package"}
VISIBILITY = {"private", "team", "internal", "public"}
ROLES = {"owner", "editor", "viewer", "publisher"}
CATALOG_OBJECTS = {
    "lesson_catalog": "lesson",
    "package_catalog": "package",
    "project_catalog": "project",
}
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
        raise ValidationError("E_TTONIMARU_JSON_INVALID", str(path), str(exc)) from exc
    if not isinstance(data, dict):
        raise ValidationError("E_TTONIMARU_ROOT_NOT_OBJECT", str(path), "root must be an object")
    return data


def nonempty_string(card: dict[str, Any], field: str, code: str | None = None) -> str:
    value = card.get(field)
    if not isinstance(value, str) or not value.strip():
        raise ValidationError(code or f"E_TTONIMARU_{field.upper()}_INVALID", field, "must be a non-empty string")
    return value


def exact_string_list(card: dict[str, Any], field: str, expected: set[str]) -> list[str]:
    value = card.get(field)
    if not isinstance(value, list):
        raise ValidationError(f"E_TTONIMARU_{field.upper()}_INVALID", field, "must be a list")
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item.strip():
            raise ValidationError(
                f"E_TTONIMARU_{field.upper()}_ITEM_INVALID",
                f"{field}[{index}]",
                "must be a non-empty string",
            )
        if item not in expected:
            code = f"E_TTONIMARU_{field.upper()}_UNKNOWN"
            if field == "objects":
                code = "E_TTONIMARU_OBJECT_KIND_UNKNOWN"
            raise ValidationError(code, f"{field}[{index}]", f"expected one of {sorted(expected)}")
    if set(value) != expected:
        raise ValidationError(f"E_TTONIMARU_{field.upper()}_MISMATCH", field, f"expected exactly {sorted(expected)}")
    if len(value) != len(set(value)):
        raise ValidationError(f"E_TTONIMARU_{field.upper()}_DUPLICATE", field, "must not contain duplicates")
    return value


def string_list(card: dict[str, Any], field: str, *, allow_empty: bool) -> list[str]:
    value = card.get(field)
    if not isinstance(value, list):
        raise ValidationError(f"E_TTONIMARU_{field.upper()}_INVALID", field, "must be a list")
    if not allow_empty and not value:
        raise ValidationError(f"E_TTONIMARU_{field.upper()}_EMPTY", field, "must not be empty")
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item.strip():
            raise ValidationError(
                f"E_TTONIMARU_{field.upper()}_ITEM_INVALID",
                f"{field}[{index}]",
                "must be a non-empty string",
            )
    return value


def require_false(card: dict[str, Any], field: str, code: str) -> None:
    if field not in card:
        raise ValidationError(f"E_TTONIMARU_{field.upper()}_MISSING", field, "field is required")
    value = card.get(field)
    if not isinstance(value, bool):
        raise ValidationError(f"E_TTONIMARU_{field.upper()}_INVALID", field, "must be boolean")
    if value:
        raise ValidationError(code, field, "must be false")


def require_value(obj: dict[str, Any], field: str, expected: Any, code: str) -> None:
    if field not in obj:
        raise ValidationError(code, field, "field is required")
    if obj.get(field) != expected:
        raise ValidationError(code, field, f"expected {expected!r}")


def validate_schema(card: dict[str, Any]) -> None:
    if "schema" not in card:
        raise ValidationError("E_TTONIMARU_SCHEMA_MISSING", "schema", "field is required")
    if card["schema"] != SCHEMA:
        raise ValidationError("E_TTONIMARU_SCHEMA_UNKNOWN", "schema", f"expected {SCHEMA}")


def validate_id(card: dict[str, Any]) -> None:
    card_id = nonempty_string(card, "id")
    if not ID_RE.match(card_id):
        raise ValidationError("E_TTONIMARU_ID_INVALID", "id", "must be stable snake/kebab lower-case id")


def validate_catalogs(card: dict[str, Any]) -> None:
    catalogs = card.get("catalogs")
    if not isinstance(catalogs, list) or not catalogs:
        raise ValidationError("E_TTONIMARU_CATALOGS_INVALID", "catalogs", "must be a non-empty list")
    seen: dict[str, str] = {}
    for index, item in enumerate(catalogs):
        field = f"catalogs[{index}]"
        if not isinstance(item, dict):
            raise ValidationError("E_TTONIMARU_CATALOG_INVALID", field, "must be an object")
        kind = item.get("kind")
        object_kind = item.get("object_kind")
        if kind not in CATALOG_OBJECTS:
            raise ValidationError("E_TTONIMARU_CATALOG_KIND_UNKNOWN", f"{field}.kind", f"expected one of {sorted(CATALOG_OBJECTS)}")
        if object_kind != CATALOG_OBJECTS[kind]:
            raise ValidationError("E_TTONIMARU_CATALOG_OBJECT_MISMATCH", f"{field}.object_kind", f"{kind} must map to {CATALOG_OBJECTS[kind]}")
        if kind in seen and seen[kind] != object_kind:
            raise ValidationError("E_TTONIMARU_CATALOG_OBJECT_MISMATCH", field, f"{kind} is already mapped to {seen[kind]}")
        seen[kind] = object_kind
    if set(seen) != set(CATALOG_OBJECTS):
        raise ValidationError("E_TTONIMARU_CATALOGS_MISMATCH", "catalogs", f"expected exactly {sorted(CATALOG_OBJECTS)}")


def validate_publication_policy(card: dict[str, Any]) -> None:
    policy = card.get("publication_policy")
    if not isinstance(policy, dict):
        raise ValidationError("E_TTONIMARU_PUBLICATION_POLICY_INVALID", "publication_policy", "must be an object")
    require_value(policy, "revision_pin_required", True, "E_TTONIMARU_REVISION_PIN_REQUIRED")
    require_value(policy, "published_artifact_immutable", True, "E_TTONIMARU_ARTIFACT_IMMUTABLE_REQUIRED")
    require_value(policy, "draft_auto_reflect_public", False, "E_TTONIMARU_DRAFT_AUTO_REFLECT_FORBIDDEN")
    require_value(policy, "republish_mode", "new_artifact", "E_TTONIMARU_REPUBLISH_MODE_INVALID")


def validate_api_ledgers(card: dict[str, Any]) -> None:
    ledgers = card.get("api_ledgers")
    if not isinstance(ledgers, dict):
        raise ValidationError("E_TTONIMARU_API_LEDGERS_INVALID", "api_ledgers", "must be an object")
    internal = ledgers.get("internal_v0")
    public = ledgers.get("api_v1")
    alias = ledgers.get("alias_url")
    if not isinstance(internal, dict):
        raise ValidationError("E_TTONIMARU_INTERNAL_API_LEDGER_INVALID", "api_ledgers.internal_v0", "must be an object")
    if not isinstance(public, dict):
        raise ValidationError("E_TTONIMARU_PUBLIC_API_LEDGER_INVALID", "api_ledgers.api_v1", "must be an object")
    if not isinstance(alias, dict):
        raise ValidationError("E_TTONIMARU_ALIAS_URL_LEDGER_INVALID", "api_ledgers.alias_url", "must be an object")
    require_value(internal, "path_prefix", "/internal/v0", "E_TTONIMARU_INTERNAL_API_PATH_INVALID")
    require_value(internal, "stability", "breakable", "E_TTONIMARU_INTERNAL_API_STABILITY_INVALID")
    require_value(internal, "mutation", True, "E_TTONIMARU_INTERNAL_API_MUTATION_INVALID")
    require_value(internal, "audience", "authoring", "E_TTONIMARU_INTERNAL_API_AUDIENCE_INVALID")
    require_value(public, "path_prefix", "/api/v1", "E_TTONIMARU_PUBLIC_API_PATH_INVALID")
    require_value(public, "stability", "add_only", "E_TTONIMARU_PUBLIC_API_STABILITY_INVALID")
    require_value(public, "mutation", False, "E_TTONIMARU_PUBLIC_API_MUTATION_FORBIDDEN")
    require_value(public, "audience", "public_read", "E_TTONIMARU_PUBLIC_API_AUDIENCE_INVALID")
    require_value(alias, "path_pattern", "/u/{owner}/{slug}", "E_TTONIMARU_ALIAS_URL_PATTERN_INVALID")
    require_value(alias, "redirect_only", True, "E_TTONIMARU_ALIAS_URL_REDIRECT_ONLY_REQUIRED")


def validate_charter(card: dict[str, Any]) -> None:
    validate_schema(card)
    validate_id(card)
    require_value(card, "product_name", "ttonimaru", "E_TTONIMARU_PRODUCT_NAME_INVALID")
    require_value(card, "layer", "platform_shell", "E_TTONIMARU_LAYER_INVALID")
    require_false(card, "runtime_truth_owner", "E_TTONIMARU_RUNTIME_TRUTH_OWNER_FORBIDDEN")
    require_false(card, "state_hash_owner", "E_TTONIMARU_STATE_HASH_OWNER_FORBIDDEN")
    require_false(card, "replay_owner", "E_TTONIMARU_REPLAY_OWNER_FORBIDDEN")
    exact_string_list(card, "objects", OBJECTS)
    exact_string_list(card, "share_kinds", SHARE_KINDS)
    exact_string_list(card, "visibility", VISIBILITY)
    exact_string_list(card, "roles", ROLES)
    validate_catalogs(card)
    validate_publication_policy(card)
    validate_api_ledgers(card)
    string_list(card, "excluded_scope", allow_empty=False)


def iter_detjson_files(path: Path) -> list[Path]:
    if path.is_file():
        return [path]
    return sorted(p for p in path.glob("*.detjson") if p.is_file())


def validate_valid_file(path: Path) -> None:
    card = load_json(path)
    if "expected_error" in card:
        raise ValidationError("E_TTONIMARU_EXPECTED_ERROR_FORBIDDEN", "expected_error", "--file/--dir expects valid charters")
    validate_charter(card)


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
                validate_charter(fixture)
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
    parser = argparse.ArgumentParser(description="Validate TTONIMARU_PLATFORM_CHARTER_V1 fixtures.")
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--file", type=Path, help="Validate one valid platform charter.")
    group.add_argument("--dir", type=Path, help="Validate all *.detjson valid platform charters in a directory.")
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
