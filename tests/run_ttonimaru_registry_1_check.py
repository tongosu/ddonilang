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
PACK = ROOT / "pack" / "ttonimaru_registry_1_v1"
SCHEMA = "ddn.ttonimaru.server_mvp.v1"
CHARTER_ID = "ttonimaru_platform_charter_v1"
ID_RE = re.compile(r"^[a-z0-9]+(?:[_-][a-z0-9]+)*$")
REQUIRED_INTERNAL = {
    "POST /internal/v0/projects",
    "GET /internal/v0/projects/{project_id}",
    "POST /internal/v0/projects/{project_id}/save",
    "GET /internal/v0/projects/{project_id}/revisions",
    "GET /internal/v0/revisions/{revision_id}",
    "POST /internal/v0/revisions/{revision_id}/publish",
}
REQUIRED_PUBLIC = {
    "GET /api/v1/publications/{publication_id}",
    "GET /api/v1/publications/{publication_id}/manifest",
    "GET /api/v1/registry/packages/{scope}/{name}",
    "GET /api/v1/registry/packages/{scope}/{name}/{version}",
}


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
        raise ValidationError("E_TTONIMARU_SERVER_JSON_INVALID", str(path), str(exc)) from exc
    if not isinstance(data, dict):
        raise ValidationError("E_TTONIMARU_SERVER_ROOT_NOT_OBJECT", str(path), "root must be an object")
    return data


def require_value(obj: dict[str, Any], field: str, expected: Any, code: str) -> None:
    if obj.get(field) != expected:
        raise ValidationError(code, field, f"expected {expected!r}")


def require_false(obj: dict[str, Any], field: str, code: str) -> None:
    if obj.get(field) is not False:
        raise ValidationError(code, field, "must be false")


def require_true(obj: dict[str, Any], field: str, code: str) -> None:
    if obj.get(field) is not True:
        raise ValidationError(code, field, "must be true")


def require_obj(card: dict[str, Any], field: str) -> dict[str, Any]:
    value = card.get(field)
    if not isinstance(value, dict):
        raise ValidationError(f"E_TTONIMARU_SERVER_{field.upper()}_INVALID", field, "must be an object")
    return value


def validate_schema(card: dict[str, Any]) -> None:
    require_value(card, "schema", SCHEMA, "E_TTONIMARU_SERVER_SCHEMA_UNKNOWN")
    card_id = card.get("id")
    if not isinstance(card_id, str) or not ID_RE.match(card_id):
        raise ValidationError("E_TTONIMARU_SERVER_ID_INVALID", "id", "must be stable lower snake/kebab id")


def validate_stack(card: dict[str, Any]) -> None:
    stack = require_obj(card, "server_stack")
    require_value(stack, "api", "fastapi", "E_TTONIMARU_SERVER_STACK_API_INVALID")
    require_value(stack, "persistence", "sqlite", "E_TTONIMARU_SERVER_STACK_PERSISTENCE_INVALID")
    require_value(stack, "tests", "pytest", "E_TTONIMARU_SERVER_STACK_TESTS_INVALID")


def validate_auth(card: dict[str, Any]) -> None:
    auth = require_obj(card, "auth")
    if auth.get("mode") != "dev_bearer_tokens" or auth.get("internal_v0_required") is not True:
        raise ValidationError("E_TTONIMARU_SERVER_AUTH_REQUIRED_INVALID", "auth", "internal_v0 must require dev bearer tokens")
    require_false(auth, "team_internal_membership_enforced", "E_TTONIMARU_SERVER_TEAM_INTERNAL_ENFORCEMENT_DEFERRED")
    tokens = auth.get("tokens")
    if not isinstance(tokens, list):
        raise ValidationError("E_TTONIMARU_SERVER_AUTH_TOKENS_INVALID", "auth.tokens", "must be a list")
    token_roles: dict[str, set[str]] = {}
    for index, item in enumerate(tokens):
        if not isinstance(item, dict):
            raise ValidationError("E_TTONIMARU_SERVER_AUTH_TOKEN_INVALID", f"auth.tokens[{index}]", "must be an object")
        token = item.get("token")
        roles = item.get("roles")
        if not isinstance(token, str) or not token.strip():
            raise ValidationError("E_TTONIMARU_SERVER_AUTH_TOKEN_INVALID", f"auth.tokens[{index}].token", "must be non-empty")
        if not isinstance(roles, list):
            raise ValidationError("E_TTONIMARU_SERVER_AUTH_ROLES_INVALID", f"auth.tokens[{index}].roles", "must be a list")
        token_roles[token] = {str(role) for role in roles}
    if "dev-owner-token" not in token_roles or not {"owner", "publisher"}.issubset(token_roles["dev-owner-token"]):
        raise ValidationError("E_TTONIMARU_SERVER_AUTH_OWNER_TOKEN_MISSING", "auth.tokens", "dev-owner-token owner/publisher required")
    if "dev-viewer-token" not in token_roles or "viewer" not in token_roles["dev-viewer-token"]:
        raise ValidationError("E_TTONIMARU_SERVER_AUTH_VIEWER_TOKEN_MISSING", "auth.tokens", "dev-viewer-token viewer required")


def validate_mapping(card: dict[str, Any]) -> None:
    mapping = require_obj(card, "object_mapping")
    require_value(mapping, "lesson", "catalog_source_not_mutated", "E_TTONIMARU_SERVER_LESSON_MAPPING_INVALID")
    require_value(mapping, "project", "authoring_object", "E_TTONIMARU_SERVER_PROJECT_MAPPING_INVALID")
    require_value(mapping, "revision", "append_only_snapshot", "E_TTONIMARU_SERVER_REVISION_MAPPING_INVALID")
    require_value(mapping, "artifact", "revision_pinned_publication", "E_TTONIMARU_SERVER_ARTIFACT_MAPPING_INVALID")
    require_value(mapping, "workspace", "ui_session_route_slot", "E_TTONIMARU_SERVER_WORKSPACE_MAPPING_INVALID")


def validate_endpoints(card: dict[str, Any]) -> None:
    endpoints = require_obj(card, "endpoints")
    internal = endpoints.get("internal_v0")
    public = endpoints.get("api_v1")
    if not isinstance(internal, list) or set(internal) != REQUIRED_INTERNAL:
        raise ValidationError("E_TTONIMARU_SERVER_INTERNAL_ENDPOINTS_INVALID", "endpoints.internal_v0", "endpoint set mismatch")
    if not isinstance(public, list) or set(public) != REQUIRED_PUBLIC:
        raise ValidationError("E_TTONIMARU_SERVER_PUBLIC_ENDPOINTS_INVALID", "endpoints.api_v1", "endpoint set mismatch")


def validate_alias(card: dict[str, Any]) -> None:
    alias = require_obj(card, "alias_url")
    require_value(alias, "path_pattern", "/u/{owner}/{slug}", "E_TTONIMARU_SERVER_ALIAS_PATTERN_INVALID")
    require_value(alias, "redirect_only", True, "E_TTONIMARU_SERVER_ALIAS_REDIRECT_ONLY_REQUIRED")
    require_value(alias, "status_code", 302, "E_TTONIMARU_SERVER_ALIAS_STATUS_INVALID")
    require_value(alias, "target", "/api/v1/publications/{publication_id}", "E_TTONIMARU_SERVER_ALIAS_TARGET_INVALID")


def validate_data_policy(card: dict[str, Any]) -> None:
    policy = require_obj(card, "data_policy")
    require_true(policy, "revision_append_only", "E_TTONIMARU_SERVER_REVISION_APPEND_ONLY_REQUIRED")
    require_true(policy, "publication_immutable", "E_TTONIMARU_SERVER_PUBLICATION_IMMUTABLE_REQUIRED")
    require_true(policy, "publication_revision_pin_required", "E_TTONIMARU_SERVER_PUBLICATION_PIN_REQUIRED")
    require_false(policy, "state_hash_generated_by_server", "E_TTONIMARU_SERVER_STATE_HASH_OWNER_FORBIDDEN")
    require_false(policy, "runtime_truth_owner", "E_TTONIMARU_SERVER_RUNTIME_TRUTH_OWNER_FORBIDDEN")
    require_false(policy, "state_hash_owner", "E_TTONIMARU_SERVER_STATE_HASH_OWNER_FORBIDDEN")
    require_false(policy, "replay_owner", "E_TTONIMARU_SERVER_REPLAY_OWNER_FORBIDDEN")


def validate_registry_stub(card: dict[str, Any]) -> None:
    stub = require_obj(card, "registry_stub")
    require_value(stub, "schema", "ddn.ttonimaru.package_metadata_stub.v1", "E_TTONIMARU_SERVER_REGISTRY_SCHEMA_INVALID")
    require_value(stub, "status_code", 200, "E_TTONIMARU_SERVER_REGISTRY_STATUS_CODE_INVALID")
    require_value(stub, "status", "stub", "E_TTONIMARU_SERVER_REGISTRY_STATUS_INVALID")
    require_false(stub, "install_supported", "E_TTONIMARU_SERVER_REGISTRY_INSTALL_FORBIDDEN")
    require_false(stub, "update_supported", "E_TTONIMARU_SERVER_REGISTRY_UPDATE_FORBIDDEN")
    require_false(stub, "remove_supported", "E_TTONIMARU_SERVER_REGISTRY_REMOVE_FORBIDDEN")
    require_false(stub, "public_registry_final", "E_TTONIMARU_SERVER_PUBLIC_REGISTRY_FINAL_FORBIDDEN")


def validate_card(card: dict[str, Any]) -> None:
    validate_schema(card)
    require_value(card, "consumes_charter", CHARTER_ID, "E_TTONIMARU_SERVER_CHARTER_GATE_INVALID")
    validate_stack(card)
    validate_auth(card)
    validate_mapping(card)
    validate_endpoints(card)
    validate_alias(card)
    validate_data_policy(card)
    validate_registry_stub(card)
    excluded = card.get("excluded_scope")
    if not isinstance(excluded, list) or "public_registry_final" not in excluded:
        raise ValidationError("E_TTONIMARU_SERVER_EXCLUDED_SCOPE_INVALID", "excluded_scope", "must include public_registry_final")


def iter_detjson_files(path: Path) -> list[Path]:
    if path.is_file():
        return [path]
    return sorted(p for p in path.glob("*.detjson") if p.is_file())


def validate_valid_file(path: Path) -> None:
    card = load_json(path)
    if "expected_error" in card:
        raise ValidationError("E_TTONIMARU_SERVER_EXPECTED_ERROR_FORBIDDEN", "expected_error", "--file/--dir expects valid cards")
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
    parser = argparse.ArgumentParser(description="Validate TTONIMARU_SERVER_MVP_V1 fixtures.")
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--file", type=Path, help="Validate one valid server MVP card.")
    group.add_argument("--dir", type=Path, help="Validate all *.detjson valid server MVP cards in a directory.")
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

