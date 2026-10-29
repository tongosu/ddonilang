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
PACK = ROOT / "pack" / "seulgi_question_card_schema_v1"
SCHEMA = "ddn.seulgi.question_card.v1"
CARD_KINDS = {
    "code_help",
    "error_explain",
    "proposal",
    "pack_skeleton",
    "textbook_draft",
    "ssot_drift_question",
}
ORIGINS = {"human", "codex", "toolchain", "lesson", "report"}
RESPONSE_STATUSES = {"none", "proposed", "rejected", "accepted_for_review"}
REQUEST_KEYS = ("요청", "필요", "현재", "대안", "제약", "출력")
ID_RE = re.compile(r"^[a-z0-9]+(?:[_-][a-z0-9]+)*$")
SHA256_RE = re.compile(r"^sha256:[0-9a-f]{64}$")


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
        raise ValidationError("E_SEULGI_QUESTION_JSON_INVALID", str(path), str(exc)) from exc
    if not isinstance(data, dict):
        raise ValidationError("E_SEULGI_QUESTION_ROOT_NOT_OBJECT", str(path), "root must be an object")
    return data


def nonempty_string(card: dict[str, Any], field: str, code: str | None = None) -> str:
    value = card.get(field)
    if not isinstance(value, str) or not value.strip():
        raise ValidationError(code or f"E_SEULGI_QUESTION_{field.upper()}_INVALID", field, "must be a non-empty string")
    return value


def string_list(value: Any, field: str, *, allow_empty: bool) -> list[str]:
    if not isinstance(value, list):
        raise ValidationError(f"E_SEULGI_QUESTION_{field.upper()}_INVALID", field, "must be a list")
    if not allow_empty and not value:
        raise ValidationError(f"E_SEULGI_QUESTION_{field.upper()}_EMPTY", field, "must not be empty")
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item.strip():
            raise ValidationError(
                f"E_SEULGI_QUESTION_{field.upper()}_ITEM_INVALID",
                f"{field}[{index}]",
                "must be a non-empty string",
            )
    return value


def validate_hash(card: dict[str, Any], field: str, code: str) -> None:
    value = card.get(field)
    if not isinstance(value, str) or not SHA256_RE.match(value):
        raise ValidationError(code, field, "must match sha256:<64 lowercase hex>")


def validate_schema(card: dict[str, Any]) -> None:
    if "schema" not in card:
        raise ValidationError("E_SEULGI_QUESTION_SCHEMA_MISSING", "schema", "field is required")
    if card["schema"] != SCHEMA:
        raise ValidationError("E_SEULGI_QUESTION_SCHEMA_UNKNOWN", "schema", f"expected {SCHEMA}")


def validate_id(card: dict[str, Any]) -> None:
    card_id = nonempty_string(card, "id")
    if not ID_RE.match(card_id):
        raise ValidationError("E_SEULGI_QUESTION_ID_INVALID", "id", "must be stable snake/kebab lower-case id")


def validate_card_kind(card: dict[str, Any]) -> None:
    card_kind = nonempty_string(card, "card_kind")
    if card_kind not in CARD_KINDS:
        raise ValidationError("E_SEULGI_QUESTION_CARD_KIND_UNKNOWN", "card_kind", f"expected one of {sorted(CARD_KINDS)}")


def validate_origin(card: dict[str, Any]) -> None:
    origin = nonempty_string(card, "origin")
    if origin not in ORIGINS:
        raise ValidationError("E_SEULGI_QUESTION_ORIGIN_UNKNOWN", "origin", f"expected one of {sorted(ORIGINS)}")


def validate_request(card: dict[str, Any]) -> None:
    request = card.get("request")
    if not isinstance(request, dict):
        raise ValidationError("E_SEULGI_QUESTION_REQUEST_INVALID", "request", "must be an object")
    for key in REQUEST_KEYS:
        if key not in request:
            raise ValidationError("E_SEULGI_QUESTION_REQUEST_KEY_MISSING", f"request.{key}", "field is required")
    for key in ("요청", "출력"):
        value = request.get(key)
        if not isinstance(value, str) or not value.strip():
            raise ValidationError("E_SEULGI_QUESTION_REQUEST_TEXT_EMPTY", f"request.{key}", "must be a non-empty string")
    for key in ("현재", "대안"):
        if not isinstance(request.get(key), str):
            raise ValidationError("E_SEULGI_QUESTION_REQUEST_TEXT_INVALID", f"request.{key}", "must be a string")
    for key in ("필요", "제약"):
        string_list(request.get(key), f"request.{key}", allow_empty=True)


def validate_response(card: dict[str, Any]) -> None:
    response = card.get("response")
    if response is None:
        return
    if not isinstance(response, dict):
        raise ValidationError("E_SEULGI_QUESTION_RESPONSE_INVALID", "response", "must be null or object")
    status = response.get("status")
    if status not in RESPONSE_STATUSES:
        raise ValidationError("E_SEULGI_QUESTION_RESPONSE_STATUS_UNKNOWN", "response.status", f"expected one of {sorted(RESPONSE_STATUSES)}")
    summary = response.get("summary")
    if not isinstance(summary, str):
        raise ValidationError("E_SEULGI_QUESTION_RESPONSE_SUMMARY_INVALID", "response.summary", "must be a string")
    if "response_hash" not in response:
        raise ValidationError("E_SEULGI_QUESTION_RESPONSE_HASH_MISSING", "response.response_hash", "field is required")
    value = response.get("response_hash")
    if not isinstance(value, str) or not SHA256_RE.match(value):
        raise ValidationError("E_SEULGI_QUESTION_RESPONSE_HASH_INVALID", "response.response_hash", "must match sha256:<64 lowercase hex>")


def validate_policy(card: dict[str, Any]) -> None:
    if "policy" not in card:
        raise ValidationError("E_SEULGI_QUESTION_POLICY_MISSING", "policy", "field is required")
    policy = card.get("policy")
    if not isinstance(policy, dict):
        raise ValidationError("E_SEULGI_QUESTION_POLICY_INVALID", "policy", "must be an object")
    required = ("runtime_ast_persisted", "auto_apply", "state_hash_owner")
    for key in required:
        if key not in policy:
            raise ValidationError("E_SEULGI_QUESTION_POLICY_KEY_MISSING", f"policy.{key}", "field is required")
        if not isinstance(policy.get(key), bool):
            raise ValidationError("E_SEULGI_QUESTION_POLICY_VALUE_INVALID", f"policy.{key}", "must be boolean")
    if policy["runtime_ast_persisted"]:
        raise ValidationError("E_SEULGI_QUESTION_RUNTIME_AST_FORBIDDEN", "policy.runtime_ast_persisted", "must be false")
    if policy["auto_apply"]:
        raise ValidationError("E_SEULGI_QUESTION_AUTO_APPLY_FORBIDDEN", "policy.auto_apply", "must be false")
    if policy["state_hash_owner"]:
        raise ValidationError("E_SEULGI_QUESTION_STATE_HASH_OWNER_FORBIDDEN", "policy.state_hash_owner", "must be false")


def validate_card(card: dict[str, Any]) -> None:
    validate_schema(card)
    validate_id(card)
    validate_card_kind(card)
    validate_origin(card)
    nonempty_string(card, "target")
    validate_request(card)
    validate_hash(card, "prompt_hash", "E_SEULGI_QUESTION_PROMPT_HASH_INVALID")
    validate_response(card)
    validate_hash(card, "question_card_hash", "E_SEULGI_QUESTION_CARD_HASH_INVALID")
    validate_policy(card)
    evidence = card.get("evidence")
    if evidence is None:
        raise ValidationError("E_SEULGI_QUESTION_EVIDENCE_INVALID", "evidence", "must be a list")
    string_list(evidence, "evidence", allow_empty=True)


def iter_detjson_files(path: Path) -> list[Path]:
    if path.is_file():
        return [path]
    return sorted(p for p in path.glob("*.detjson") if p.is_file())


def validate_valid_file(path: Path) -> None:
    card = load_json(path)
    if "expected_error" in card:
        raise ValidationError("E_SEULGI_QUESTION_EXPECTED_ERROR_FORBIDDEN", "expected_error", "--file/--dir expects valid cards")
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
    parser = argparse.ArgumentParser(description="Validate SEULGI_QUESTION_CARD_V1 fixtures.")
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--file", type=Path, help="Validate one valid question card.")
    group.add_argument("--dir", type=Path, help="Validate all *.detjson valid question cards in a directory.")
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
