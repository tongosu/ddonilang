#!/usr/bin/env python
import argparse
from collections import OrderedDict
import hashlib
import ipaddress
import json
import logging
import os
import re
import shutil
import socket
import sqlite3
import subprocess
import sys
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, unquote, urlparse


class SeamgrimHTTPServer(ThreadingHTTPServer):
    # SO_REUSEADDR on Windows allows two live servers to share a port. A new
    # launcher can then appear ready while the browser still uses the old one.
    allow_reuse_address = sys.platform != "win32"

    def server_bind(self):
        if sys.platform == "win32":
            self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_EXCLUSIVEADDRUSE, 1)
        super().server_bind()

# Reuse export_graph helpers
TOOLS_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS_DIR))

from export_graph import (
    run_ddonilang,
    parse_points,
    build_graph,
    hash_text,
    extract_meta,
    extract_series_labels,
    normalize_ddn_for_hash,
    _parse_numbers_from_line,
    normalize_series_label,
)
from project_store import (
    ROLE_EDIT,
    ROLE_VIEW,
    SCHEMA_VERSION,
    ProjectConflictError,
    ProjectDeletedError,
    ProjectNotFoundError,
    ProjectPermissionError,
    ProjectStoreError,
    ProjectValidationError,
    SeamgrimProjectStore,
)

ROOT = Path(__file__).resolve().parents[3]
UI_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui"
LESSON_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
ALPHA1_LESSON_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons_v25_alpha1"
SEED_LESSON_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "seed_lessons_v1"
REWRITE_LESSON_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons_rewrite_v1"
ACTIVE_ALLOWLIST_PATH = LESSON_DIR / "active_allowlist.detjson"
SCHEMA_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "schema"
SAMPLES_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "samples"
PROJECT_HTTP_PREFIX = "/solutions/seamgrim_ui_mvp"
PROJECT_API_PREFIX = "/api/v1/projects"
WORKBENCH_DEFAULT_ROUTE = f"{PROJECT_HTTP_PREFIX}/ui/workbench/index.html"
DEFAULT_SHELL_PATHS = {"/", "/index.html", "/ui", "/ui/"}
LEGACY_SHELL_PATHS = {"/legacy", "/legacy.html", "/ui/legacy", "/ui/legacy.html"}
LEGACY_DEEP_LINK_KEYS = {"legacy", "tab", "screen", "route", "main_tab", "main-tab"}
CAPABILITY_QUERY_KEYS = {"cap", "capability", "sg_cap", "token"}
WORKBENCH_ROUTE_PROJECT_IDS = {
    "rep_phys_projectile_xy_v1",
    "rep_phys_pendulum_wave1_v1",
    "rep_grid_game_state_drop_v1",
    "local_linear_slope_v1",
}
WORKBENCH_PROJECT_QUERY_KEYS = {"project", "project_id", "lesson", "lesson_id", "source_lesson_id"}
LOGGER = logging.getLogger("seamgrim.ddn_exec_server")
CATEGORY_REPS_ONLY = "reps_only"
CATEGORY_FULL = "full"
APP_JSON_BODY_LIMIT_BYTES = 2_000_000
DEPLOY_PROFILE_DEV = "dev"
DEPLOY_PROFILE_TEST = "test"
DEPLOY_PROFILE_PRIVATE_PILOT = "private-pilot"
VALID_DEPLOY_PROFILES = {DEPLOY_PROFILE_DEV, DEPLOY_PROFILE_TEST, DEPLOY_PROFILE_PRIVATE_PILOT}
PRIVATE_PILOT_SQLITE_MIN_VERSION = (3, 51, 3)
PRIVATE_PILOT_SQLITE_MIN_VERSION_TEXT = "3.51.3"
_ACTIVE_RUNTIME_CONFIG: dict | None = None
SPACE2D_MARKERS = {"space2d", "2d", "공간", "공간2d"}
SPACE2D_SHAPE_MARKERS = {"space2d.shape", "space2d_shape", "shape2d"}
SPACE2D_SHAPE_KEYS = {
    "x1",
    "y1",
    "x2",
    "y2",
    "cx",
    "cy",
    "r",
    "x",
    "y",
    "size",
    "stroke",
    "fill",
    "color",
    "width",
    "token",
    "id",
    "name",
    "label",
    "토큰",
    "group_id",
    "group",
    "groupId",
    "그룹",
    "묶음",
}
TEXT_MARKERS = {"text", "문서", "해설", "설명", "caption", "자막"}
TEXT_END_MARKERS = {"text.end", "endtext", "문서끝", "끝"}
TEXT_PREFIXES = ("text:", "문서:", "해설:", "설명:", "caption:", "자막:")
TABLE_MARKERS = {"table", "표", "테이블"}
TABLE_END_MARKERS = {"table.end", "endtable", "표끝", "테이블끝"}
TABLE_ROW_MARKERS = {"table.row", "표행"}
STRUCTURE_MARKERS = {"structure", "구조", "그래프구조"}
STRUCTURE_END_MARKERS = {"structure.end", "endstructure", "구조끝"}
MAEGIM_CONTROL_FILENAME = "maegim_control.json"
MAEGIM_CONTROL_SCHEMA = "ddn.maegim_control_plan.v1"
MAEGIM_CONTROL_LEGACY_RANGE_WARNING_CODE = "W_LEGACY_RANGE_COMMENT_DEPRECATED"
_MAEGIM_CONTROL_CACHE: OrderedDict[str, tuple[int, int, str, str]] = OrderedDict()
_MAEGIM_CONTROL_CACHE_LOCK = threading.Lock()
_MAEGIM_CONTROL_CACHE_MAX_ENTRIES = 512
NUMERIC_LITERAL_RE = r"[+-]?(?:\d+(?:\.\d+)?|\.\d+)(?:e[+-]?\d+)?"


def _env_flag(name: str, default: bool = False) -> bool:
    raw = str(os.environ.get(name, "")).strip().lower()
    if not raw:
        return bool(default)
    return raw in {"1", "true", "yes", "on"}


class PrivatePilotConfigError(RuntimeError):
    def __init__(self, codes: list[str]) -> None:
        self.codes = [str(code) for code in codes if str(code)]
        super().__init__(";".join(self.codes) or "E_PRIVATE_PILOT_CONFIG_INVALID")


def _split_csv(raw: object) -> list[str]:
    out: list[str] = []
    seen: set[str] = set()
    for part in str(raw or "").split(","):
        item = part.strip()
        if item and item not in seen:
            seen.add(item)
            out.append(item)
    return out


def _parse_version_tuple(raw: str) -> tuple[int, int, int]:
    pieces = re.findall(r"\d+", str(raw or ""))
    nums = [int(piece) for piece in pieces[:3]]
    while len(nums) < 3:
        nums.append(0)
    return tuple(nums[:3])  # type: ignore[return-value]


def _path_is_relative_to(child: Path, parent: Path) -> bool:
    try:
        child.resolve().relative_to(parent.resolve())
        return True
    except Exception:
        return False


def _redacted_path_token(path: Path) -> str:
    try:
        resolved = path.resolve()
        return f"{resolved.anchor}<redacted>/{resolved.name}"
    except Exception:
        return "<redacted>"


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _client_is_loopback(handler: BaseHTTPRequestHandler) -> bool:
    raw = ""
    try:
        raw = str(handler.client_address[0])
        return ipaddress.ip_address(raw).is_loopback
    except Exception:
        return raw in {"localhost", "127.0.0.1", "::1"}


def management_request_allowed(handler: BaseHTTPRequestHandler) -> bool:
    config = active_runtime_config()
    if not config.get("private_pilot"):
        return True
    return _client_is_loopback(handler)


def reject_management_boundary(handler: BaseHTTPRequestHandler, code: str = "E_PRIVATE_PILOT_MANAGEMENT_FORBIDDEN") -> None:
    json_response(
        handler,
        403,
        build_api_error_payload(
            status=403,
            code=code,
            user_message="이 운영 점검 경로는 private management 경계에서만 사용할 수 있습니다.",
            technical_message=code,
        ),
    )


def build_runtime_config(environ: dict | None = None) -> dict:
    env = environ if environ is not None else os.environ
    profile = str(env.get("SEAMGRIM_DEPLOY_PROFILE", DEPLOY_PROFILE_DEV)).strip().lower() or DEPLOY_PROFILE_DEV
    if profile not in VALID_DEPLOY_PROFILES:
        raise PrivatePilotConfigError(["E_PRIVATE_PILOT_PROFILE_INVALID"])

    is_private = profile == DEPLOY_PROFILE_PRIVATE_PILOT
    release_root = ROOT.resolve()
    raw_data_dir = str(env.get("SEAMGRIM_PROJECT_DATA_DIR", "")).strip()
    if raw_data_dir:
        data_dir = Path(raw_data_dir).expanduser()
    else:
        data_dir = resolve_build_dir() / "seamgrim_project_data"
    data_dir_resolved = data_dir.resolve()

    sqlite_sync = str(env.get("SEAMGRIM_PROJECT_STORE_SQLITE_SYNCHRONOUS", "")).strip().upper()
    if not sqlite_sync:
        sqlite_sync = "FULL" if is_private else "NORMAL"

    allowed_hosts = _split_csv(env.get("SEAMGRIM_ALLOWED_HOSTS", ""))
    allowed_origins = _split_csv(env.get("SEAMGRIM_ALLOWED_ORIGINS", ""))
    trusted_proxy_policy = str(env.get("SEAMGRIM_TRUSTED_PROXY_POLICY", "none")).strip().lower() or "none"
    filesystem_policy = str(env.get("SEAMGRIM_DATA_FILESYSTEM", "")).strip().lower()
    sqlite_attestation = str(env.get("SEAMGRIM_SQLITE_WAL_SAFETY_ATTESTATION", "")).strip()
    sqlite_version = sqlite3.sqlite_version
    sqlite_tuple = _parse_version_tuple(sqlite_version)
    sqlite_meets_floor = sqlite_tuple >= PRIVATE_PILOT_SQLITE_MIN_VERSION

    errors: list[str] = []
    warnings: list[str] = []
    if sqlite_sync not in {"OFF", "NORMAL", "FULL", "EXTRA"}:
        errors.append("E_PRIVATE_PILOT_SQLITE_SYNC_INVALID")
    if is_private:
        if not raw_data_dir:
            errors.append("E_PRIVATE_PILOT_DATA_DIR_REQUIRED")
        if not data_dir.is_absolute():
            errors.append("E_PRIVATE_PILOT_DATA_DIR_MUST_BE_ABSOLUTE")
        if _path_is_relative_to(data_dir_resolved, release_root):
            errors.append("E_PRIVATE_PILOT_DATA_DIR_INSIDE_RELEASE_TREE")
        if sqlite_sync != "FULL":
            errors.append("E_PRIVATE_PILOT_SQLITE_FULL_REQUIRED")
        if not sqlite_meets_floor and not sqlite_attestation:
            errors.append("E_PRIVATE_PILOT_SQLITE_VERSION_UNVERIFIED")
        if not allowed_hosts or "*" in allowed_hosts:
            errors.append("E_PRIVATE_PILOT_ALLOWED_HOSTS_REQUIRED")
        if not allowed_origins or "*" in allowed_origins:
            errors.append("E_PRIVATE_PILOT_ALLOWED_ORIGINS_REQUIRED")
        if filesystem_policy != "local":
            errors.append("E_PRIVATE_PILOT_LOCAL_FILESYSTEM_CONFIRMATION_REQUIRED")
        if trusted_proxy_policy not in {"none", "loopback", "private-edge"}:
            errors.append("E_PRIVATE_PILOT_TRUSTED_PROXY_POLICY_INVALID")
        if str(env.get("SEAMGRIM_PUBLIC_BASE_URL", "")).strip().lower().startswith("http://"):
            errors.append("E_PRIVATE_PILOT_PUBLIC_BASE_URL_MUST_BE_HTTPS")
    else:
        if not raw_data_dir:
            warnings.append("W_DEV_DATA_DIR_DEFAULTED")
        if sqlite_sync != "NORMAL":
            warnings.append("W_DEV_SQLITE_SYNC_NON_DEFAULT")

    return {
        "schema": "seamgrim.private_pilot.runtime_config.v1",
        "profile": profile,
        "private_pilot": is_private,
        "release_root_redacted": _redacted_path_token(release_root),
        "project_data_dir": str(data_dir_resolved),
        "project_data_dir_redacted": _redacted_path_token(data_dir_resolved),
        "project_data_dir_inside_release_tree": _path_is_relative_to(data_dir_resolved, release_root),
        "sqlite_version": sqlite_version,
        "sqlite_min_version": PRIVATE_PILOT_SQLITE_MIN_VERSION_TEXT,
        "sqlite_meets_private_pilot_floor": sqlite_meets_floor,
        "sqlite_safety_attested": bool(sqlite_attestation),
        "sqlite_synchronous": sqlite_sync,
        "allowed_hosts": allowed_hosts,
        "allowed_origins": allowed_origins,
        "trusted_proxy_policy": trusted_proxy_policy,
        "data_filesystem": filesystem_policy or ("unverified" if is_private else "dev-default"),
        "app_json_body_limit_bytes": APP_JSON_BODY_LIMIT_BYTES,
        "errors": errors,
        "warnings": warnings,
        "ok": not errors,
    }


def active_runtime_config() -> dict:
    global _ACTIVE_RUNTIME_CONFIG
    if _ACTIVE_RUNTIME_CONFIG is None:
        _ACTIVE_RUNTIME_CONFIG = build_runtime_config()
    return _ACTIVE_RUNTIME_CONFIG


def reset_runtime_config_cache() -> None:
    global _ACTIVE_RUNTIME_CONFIG
    _ACTIVE_RUNTIME_CONFIG = None


def assert_runtime_config_startable() -> dict:
    config = active_runtime_config()
    errors = [str(code) for code in config.get("errors", [])]
    if errors:
        raise PrivatePilotConfigError(errors)
    return config


def _origin_allowed_for_response(handler: BaseHTTPRequestHandler) -> str:
    config = active_runtime_config()
    if not config.get("private_pilot"):
        return "*"
    origin = str(handler.headers.get("Origin", "")).strip()
    if origin and origin in set(config.get("allowed_origins", [])):
        return origin
    return ""


def _host_without_port(raw_host: str) -> str:
    host = str(raw_host or "").strip().lower()
    if not host:
        return ""
    if host.startswith("[") and "]" in host:
        return host[1:host.index("]")]
    return host.split(":", 1)[0]


def request_boundary_error(handler: BaseHTTPRequestHandler) -> tuple[int, str] | None:
    config = active_runtime_config()
    if not config.get("private_pilot"):
        return None
    allowed_hosts = {str(item).lower() for item in config.get("allowed_hosts", [])}
    host = _host_without_port(handler.headers.get("Host", ""))
    if not host or host not in allowed_hosts:
        return (421, "E_PRIVATE_PILOT_HOST_REJECTED")
    origin = str(handler.headers.get("Origin", "")).strip()
    if origin and origin not in set(config.get("allowed_origins", [])):
        return (403, "E_PRIVATE_PILOT_ORIGIN_REJECTED")
    return None


def reject_request_boundary(handler: BaseHTTPRequestHandler, status: int, code: str) -> None:
    json_response(
        handler,
        status,
        build_api_error_payload(
            status=status,
            code=code,
            user_message="이 private pilot 경로에서는 허용된 host/origin만 사용할 수 있습니다.",
            technical_message=code,
        ),
    )


def _normalize_catalog_mode(raw: object) -> str:
    mode = str(raw or "").strip().lower()
    if mode == CATEGORY_FULL:
        return CATEGORY_FULL
    return CATEGORY_REPS_ONLY


RUNTIME_CATALOG_MODE = _normalize_catalog_mode(os.environ.get("SEAMGRIM_LESSON_CATALOG_MODE", CATEGORY_REPS_ONLY))
RUNTIME_ALLOW_LEGACY_LESSONS = _env_flag("SEAMGRIM_ALLOW_LEGACY_LESSONS", False)


def resolve_build_dir() -> Path:
    override = str(os.environ.get("SEAMGRIM_BUILD_DIR", "")).strip()
    if override:
        target = Path(override).expanduser()
        target.mkdir(parents=True, exist_ok=True)
        return target.resolve()
    package_root = Path(__file__).resolve().parents[3]
    target = package_root / ".ddonirang-build"
    target.mkdir(parents=True, exist_ok=True)
    return target


def _strip_utf8_bom_prefix(text: str) -> str:
    data = str(text or "")
    if data.startswith("\ufeff"):
        return data.lstrip("\ufeff")
    return data


def _maegim_control_http_path_from_ddn_path(ddn_path: str) -> str:
    normalized = str(ddn_path or "").strip().replace("\\", "/")
    if not normalized:
        return ""
    if normalized.endswith("/lesson.ddn"):
        return f"{normalized[: -len('/lesson.ddn')]}/{MAEGIM_CONTROL_FILENAME}"
    return ""


def _infer_maegim_control_candidates(ddn_paths: list[str], explicit_candidates: list[str] | None = None) -> list[str]:
    out: list[str] = []
    seen: set[str] = set()
    for raw in [*(explicit_candidates or []), *ddn_paths]:
        candidate = str(raw or "").strip()
        if not candidate:
            continue
        if candidate.endswith("/lesson.ddn"):
            candidate = _maegim_control_http_path_from_ddn_path(candidate)
        if not candidate or candidate in seen:
            continue
        seen.add(candidate)
        out.append(candidate)
    return out


def _to_maegim_control_payload(lesson_path: Path) -> str:
    payload, _ = _build_maegim_control_payload_from_path(lesson_path)
    return json.dumps(payload, ensure_ascii=False, sort_keys=True)


def _build_maegim_control_payload_from_path(lesson_path: Path) -> tuple[dict, str]:
    lesson_path = lesson_path.resolve()
    cache_key = str(lesson_path)
    lesson_stat = lesson_path.stat()
    lesson_mtime_ns = lesson_stat.st_mtime_ns
    lesson_size = int(lesson_stat.st_size)
    with _MAEGIM_CONTROL_CACHE_LOCK:
        cached = _MAEGIM_CONTROL_CACHE.get(cache_key)
        if cached and cached[0] == lesson_mtime_ns and cached[1] == lesson_size:
            _MAEGIM_CONTROL_CACHE.move_to_end(cache_key, last=True)
            return json.loads(cached[2]), str(cached[3] or "canon")

    payload, source = _build_maegim_control_payload_from_source_text(
        lesson_path.read_text(encoding="utf-8-sig"),
        source_label=str(lesson_path),
        allow_legacy_fallback=_allow_legacy_maegim_fallback(),
    )
    normalized_text = json.dumps(payload, ensure_ascii=False, sort_keys=True)
    with _MAEGIM_CONTROL_CACHE_LOCK:
        _MAEGIM_CONTROL_CACHE[cache_key] = (lesson_mtime_ns, lesson_size, normalized_text, source)
        _MAEGIM_CONTROL_CACHE.move_to_end(cache_key, last=True)
        while len(_MAEGIM_CONTROL_CACHE) > int(_MAEGIM_CONTROL_CACHE_MAX_ENTRIES):
            _MAEGIM_CONTROL_CACHE.popitem(last=False)
    return payload, source


def _build_maegim_control_payload_from_source_text(
    source_text: str,
    source_label: str = "input.ddn",
    allow_legacy_fallback: bool = True,
) -> tuple[dict, str]:
    with tempfile.NamedTemporaryFile(
        mode="w",
        encoding="utf-8",
        suffix=".ddn",
        prefix="seamgrim_maegim_control_",
        dir=resolve_build_dir(),
        delete=False,
    ) as temp_file:
        temp_file.write(source_text)
        temp_path = Path(temp_file.name)
    try:
        teul_cli = _resolve_teul_cli_bin(ROOT)
        cmd = [str(teul_cli), "canon", str(temp_path), "--emit", "maegim-control-json"]
        result = subprocess.run(
            cmd,
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if result.returncode == 0:
            payload = json.loads(str(result.stdout or "").strip())
            if isinstance(payload, dict) and str(payload.get("schema") or "").strip() == MAEGIM_CONTROL_SCHEMA:
                payload.setdefault("warnings", [])
                controls = payload.get("controls")
                warnings = payload.get("warnings")
                if not isinstance(controls, list):
                    controls = []
                    payload["controls"] = controls
                # canon 경로가 비어 있거나(controls 없음), legacy 주석 경고를 놓친 경우에는
                # 실행 서버 계약을 위해 legacy 계획으로 강등한다.
                legacy_preview = _build_legacy_maegim_control_plan(source_text, source_label)
                legacy_warnings = legacy_preview.get("warnings")
                if allow_legacy_fallback and not controls:
                    return legacy_preview, "legacy"
                if (
                    allow_legacy_fallback
                    and isinstance(controls, list)
                    and controls
                    and isinstance(legacy_warnings, list)
                    and legacy_warnings
                    and (not isinstance(warnings, list) or not warnings)
                ):
                    return legacy_preview, "legacy"
                return payload, "canon"
        if not allow_legacy_fallback:
            raise RuntimeError("E_MAEGIM_LEGACY_FALLBACK_DISABLED:canon_failed")
        payload = _build_legacy_maegim_control_plan(source_text, source_label)
        if not isinstance(payload, dict) or str(payload.get("schema") or "").strip() != MAEGIM_CONTROL_SCHEMA:
            raise RuntimeError("invalid maegim control payload schema")
        return payload, "legacy"
    finally:
        try:
            temp_path.unlink(missing_ok=True)
        except Exception:
            pass


def _to_canon_number_text(value: float) -> str:
    if float(value).is_integer():
        return str(int(value))
    text = f"{value:.12g}"
    return text


def _build_control_item(
    name: str,
    type_name: str,
    decl_kind: str,
    init_expr_canon: str,
    min_expr_canon: str,
    max_expr_canon: str,
    step_expr_canon: str | None,
    split_count_expr_canon: str | None = None,
) -> dict:
    fields: list[dict[str, str]] = [
        {"name": "범위", "value_canon": f"{min_expr_canon} .. {max_expr_canon}"}
    ]
    if step_expr_canon:
        fields.append({"name": "간격", "value_canon": step_expr_canon})
    if split_count_expr_canon:
        fields.append({"name": "분할수", "value_canon": split_count_expr_canon})
    return {
        "name": name,
        "decl_kind": decl_kind,
        "type_name": type_name or "수",
        "init_expr_canon": init_expr_canon,
        "fields": fields,
        "range": {
            "min_expr_canon": min_expr_canon,
            "max_expr_canon": max_expr_canon,
            "inclusive_end": True,
        },
        "step_expr_canon": step_expr_canon,
        "split_count_expr_canon": split_count_expr_canon,
    }


def _build_legacy_range_warning(name: str) -> dict:
    return {
        "code": MAEGIM_CONTROL_LEGACY_RANGE_WARNING_CODE,
        "message": "`// 범위(...)`는 더 이상 허용되지 않습니다. `매김 { 범위: ... 간격: ... }`로 옮기세요.",
        "name": name,
        "source": "prep_comment",
    }


def _build_maegim_step_split_conflict_warning(name: str) -> dict:
    return {
        "code": "E_MAEGIM_STEP_SPLIT_CONFLICT",
        "message": "`매김`에서 `간격`과 `분할수`는 동시에 사용할 수 없습니다.",
        "name": name,
        "source": "legacy_maegim",
    }


def _derive_step_from_split_count(
    min_expr_canon: str,
    max_expr_canon: str,
    split_count_expr_canon: str | None,
) -> str | None:
    if not split_count_expr_canon:
        return None
    try:
        lo = float(min_expr_canon)
        hi = float(max_expr_canon)
        split_count = float(split_count_expr_canon)
    except Exception:
        return None
    if split_count <= 0:
        return None
    span = abs(hi - lo)
    if span <= 0:
        return None
    return _to_canon_number_text(span / split_count)


def _extract_legacy_range_comment(tail: str) -> tuple[str, str, str] | None:
    match = re.search(
        rf"범위\s*\(\s*({NUMERIC_LITERAL_RE})\s*,\s*({NUMERIC_LITERAL_RE})(?:\s*,\s*({NUMERIC_LITERAL_RE}))?\s*\)",
        str(tail or ""),
        re.IGNORECASE,
    )
    if not match:
        return None
    min_value = float(match.group(1))
    max_value = float(match.group(2))
    step_group = match.group(3)
    step_value = float(step_group) if step_group is not None else 0.1
    lo = min(min_value, max_value)
    hi = max(min_value, max_value)
    return (
        _to_canon_number_text(lo),
        _to_canon_number_text(hi),
        _to_canon_number_text(step_value),
    )


def _parse_maegim_block_fields(block_text: str) -> dict | None:
    range_match = re.search(
        rf"범위\s*:\s*({NUMERIC_LITERAL_RE})\s*\.\.(?:=)?\s*({NUMERIC_LITERAL_RE})",
        str(block_text or ""),
        re.IGNORECASE,
    )
    if not range_match:
        return None
    step_match = re.search(rf"간격\s*:\s*({NUMERIC_LITERAL_RE})", str(block_text or ""), re.IGNORECASE)
    split_count_match = re.search(
        rf"분할수\s*:\s*({NUMERIC_LITERAL_RE})",
        str(block_text or ""),
        re.IGNORECASE,
    )
    min_value = float(range_match.group(1))
    max_value = float(range_match.group(2))
    lo = min(min_value, max_value)
    hi = max(min_value, max_value)
    min_expr_canon = _to_canon_number_text(lo)
    max_expr_canon = _to_canon_number_text(hi)
    step_expr_canon = _to_canon_number_text(float(step_match.group(1))) if step_match else None
    split_count_expr_canon = (
        _to_canon_number_text(float(split_count_match.group(1))) if split_count_match else None
    )
    has_conflict = bool(step_expr_canon and split_count_expr_canon)
    if not step_expr_canon:
        step_expr_canon = _derive_step_from_split_count(
            min_expr_canon,
            max_expr_canon,
            split_count_expr_canon,
        )
    return {
        "min_expr_canon": min_expr_canon,
        "max_expr_canon": max_expr_canon,
        "step_expr_canon": step_expr_canon,
        "split_count_expr_canon": split_count_expr_canon,
        "has_conflict": has_conflict,
    }


def _build_legacy_maegim_control_plan(text: str, source_label: str = "input.ddn") -> dict:
    controls: list[dict] = []
    warnings: list[dict] = []
    seen: set[str] = set()
    lines = text.splitlines()
    line_index = 0

    while line_index < len(lines):
        line = str(lines[line_index] or "")
        maegim_match = re.match(
            rf"^\s*([A-Za-z0-9_가-힣]+)\s*(?::\s*([A-Za-z0-9_가-힣]+))?\s*(<-|=)\s*\(\s*({NUMERIC_LITERAL_RE})\s*\)\s*(?:조건|매김)\s*\{{(.*)$",
            line,
            re.IGNORECASE,
        )
        if maegim_match:
            name = str(maegim_match.group(1) or "").strip()
            if name and name not in seen:
                type_name = str(maegim_match.group(2) or "수").strip() or "수"
                decl_kind = "gureut" if str(maegim_match.group(3) or "").strip() == "<-" else "butbak"
                init_expr_canon = _to_canon_number_text(float(maegim_match.group(4)))
                depth = 1
                body_parts = [str(maegim_match.group(5) or "")]
                scan_index = line_index
                while scan_index < len(lines) and depth > 0:
                    segment = body_parts[-1] if scan_index == line_index else lines[scan_index]
                    if scan_index != line_index:
                        body_parts.append(segment)
                    depth += segment.count("{")
                    depth -= segment.count("}")
                    scan_index += 1
                parsed_fields = _parse_maegim_block_fields("\n".join(body_parts))
                if parsed_fields is not None:
                    if parsed_fields.get("has_conflict"):
                        warnings.append(_build_maegim_step_split_conflict_warning(name))
                        line_index = scan_index
                        continue
                    min_expr_canon = str(parsed_fields.get("min_expr_canon") or "")
                    max_expr_canon = str(parsed_fields.get("max_expr_canon") or "")
                    step_expr_canon = parsed_fields.get("step_expr_canon")
                    split_count_expr_canon = parsed_fields.get("split_count_expr_canon")
                    controls.append(
                        _build_control_item(
                            name,
                            type_name,
                            decl_kind,
                            init_expr_canon,
                            min_expr_canon,
                            max_expr_canon,
                            step_expr_canon,
                            split_count_expr_canon,
                        )
                    )
                    seen.add(name)
                line_index = scan_index
                continue

        comment_match = re.match(
            rf"^\s*([A-Za-z0-9_가-힣]+)\s*(?::\s*([A-Za-z0-9_가-힣]+))?\s*(<-|=)\s*({NUMERIC_LITERAL_RE})\s*\.\s*(?://(.*))?$",
            line,
            re.IGNORECASE,
        )
        if comment_match:
            name = str(comment_match.group(1) or "").strip()
            if name and name not in seen:
                parsed_range = _extract_legacy_range_comment(comment_match.group(5) or "")
                if parsed_range is not None:
                    type_name = str(comment_match.group(2) or "수").strip() or "수"
                    decl_kind = "gureut" if str(comment_match.group(3) or "").strip() == "<-" else "butbak"
                    init_expr_canon = _to_canon_number_text(float(comment_match.group(4)))
                    min_expr_canon, max_expr_canon, step_expr_canon = parsed_range
                    controls.append(
                        _build_control_item(
                            name,
                            type_name,
                            decl_kind,
                            init_expr_canon,
                            min_expr_canon,
                            max_expr_canon,
                            step_expr_canon,
                        )
                    )
                    warnings.append(_build_legacy_range_warning(name))
                    seen.add(name)
        line_index += 1

    return {
        "schema": MAEGIM_CONTROL_SCHEMA,
        "source": source_label,
        "controls": controls,
        "warnings": warnings,
    }


def _read_json_file(path: Path) -> dict | None:
    try:
        if not path.exists():
            return None
        payload = json.loads(path.read_text(encoding="utf-8-sig"))
        return payload if isinstance(payload, dict) else None
    except Exception:
        return None


def _read_active_lesson_allowlist() -> set[str]:
    payload = _read_json_file(ACTIVE_ALLOWLIST_PATH) or {}
    lesson_ids = payload.get("lesson_ids")
    out: set[str] = set()
    if isinstance(lesson_ids, list):
        for raw in lesson_ids:
            lesson_id = str(raw or "").strip()
            if lesson_id:
                out.add(lesson_id)
    return out


def _is_release_reps_only_mode() -> bool:
    return _normalize_catalog_mode(RUNTIME_CATALOG_MODE) == CATEGORY_REPS_ONLY


def _is_retired_line_request_blocked() -> bool:
    return _is_release_reps_only_mode() and not bool(RUNTIME_ALLOW_LEGACY_LESSONS)


def _allow_legacy_maegim_fallback() -> bool:
    if _normalize_catalog_mode(RUNTIME_CATALOG_MODE) == CATEGORY_FULL:
        return True
    return bool(RUNTIME_ALLOW_LEGACY_LESSONS)


def _normalize_subject(raw: object) -> str:
    subject = str(raw or "").strip().lower()
    if subject == "economy":
        return "econ"
    return subject


def _lesson_paths(prefix: str, lesson_id: str) -> tuple[str, str, str]:
    base = f"solutions/seamgrim_ui_mvp/{prefix}/{lesson_id}"
    return (
        f"{base}/lesson.ddn",
        f"{base}/text.md",
        f"{base}/meta.toml",
    )


def _to_string_list(value: object) -> list[str]:
    if isinstance(value, list):
        return [str(item).strip() for item in value if str(item).strip()]
    single = str(value or "").strip()
    return [single] if single else []


def _project_path_to_fs(path: str) -> Path | None:
    normalized = str(path or "").strip().replace("\\", "/")
    if not normalized:
        return None
    if normalized.startswith("http://") or normalized.startswith("https://"):
        return None
    normalized = normalized.lstrip("/")
    if normalized.startswith("solutions/seamgrim_ui_mvp/"):
        return (ROOT / normalized).resolve()
    if normalized.startswith("lessons/"):
        return (LESSON_DIR / normalized[len("lessons/"):]).resolve()
    if normalized.startswith("lessons_v25_alpha1/"):
        return (ALPHA1_LESSON_DIR / normalized[len("lessons_v25_alpha1/"):]).resolve()
    if normalized.startswith("seed_lessons_v1/"):
        return (SEED_LESSON_DIR / normalized[len("seed_lessons_v1/"):]).resolve()
    if normalized.startswith("lessons_rewrite_v1/"):
        return (REWRITE_LESSON_DIR / normalized[len("lessons_rewrite_v1/"):]).resolve()
    return (ROOT / normalized).resolve()


def _existing_meta_candidates(candidates: list[str]) -> list[str]:
    out: list[str] = []
    seen: set[str] = set()
    for raw in candidates:
        candidate = str(raw or "").strip()
        if not candidate or candidate in seen:
            continue
        if candidate.startswith("http://") or candidate.startswith("https://"):
            seen.add(candidate)
            out.append(candidate)
            continue
        fs_path = _project_path_to_fs(candidate)
        if fs_path is None or not fs_path.exists():
            continue
        seen.add(candidate)
        out.append(candidate)
    return out


def _read_text_file(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8-sig")
    except Exception:
        return ""


def _read_inventory_maegim_warning_summary(ddn_paths: list[str]) -> dict:
    warning_count = 0
    warning_names: list[str] = []
    warning_examples: list[str] = []
    seen_names: set[str] = set()
    for raw in ddn_paths:
        fs_path = _project_path_to_fs(raw)
        if fs_path is None or not fs_path.exists() or fs_path.suffix.lower() != ".ddn":
            continue
        text = _read_text_file(fs_path)
        if not text:
            continue
        lines = text.replace("\r\n", "\n").replace("\r", "\n").split("\n")
        for line in lines:
            match = re.match(
                rf"^\s*([A-Za-z0-9_가-힣]+)\s*(?::\s*([A-Za-z0-9_가-힣]+))?\s*(<-|=)\s*({NUMERIC_LITERAL_RE})\s*\.\s*(?://(.*))?$",
                line,
                re.IGNORECASE,
            )
            if not match:
                continue
            parsed_range = _extract_legacy_range_comment(match.group(5) or "")
            if parsed_range is None:
                continue
            warning_count += 1
            name = str(match.group(1) or "").strip()
            type_name = str(match.group(2) or "").strip()
            decl_op = str(match.group(3) or "").strip() or "<-"
            init_expr = _to_canon_number_text(float(match.group(4)))
            min_expr_canon, max_expr_canon, step_expr_canon = parsed_range
            typed_name = f"{name}:{type_name}" if name and type_name else name
            if typed_name:
                warning_examples.append(
                    f"{typed_name} {decl_op} ({init_expr}) 매김 {{ 범위: {min_expr_canon}..{max_expr_canon}. 간격: {step_expr_canon}. }}."
                )
            if name and name not in seen_names:
                seen_names.add(name)
                warning_names.append(name)
    warning_codes = [MAEGIM_CONTROL_LEGACY_RANGE_WARNING_CODE] if warning_count > 0 else []
    return {
        "maegim_control_warning_count": warning_count,
        "maegim_control_warning_codes": warning_codes,
        "maegim_control_warning_names": warning_names,
        "maegim_control_warning_examples": warning_examples,
        "maegim_control_warning_source": "legacy_scan" if warning_count > 0 else "",
    }


def _extract_meta_from_ddn_paths(paths: list[str]) -> dict[str, str]:
    for raw in paths:
        fs_path = _project_path_to_fs(raw)
        if fs_path is None or not fs_path.exists():
            continue
        text = _read_text_file(fs_path)
        if not text:
            continue
        meta = extract_meta(text)
        name = str(meta.get("name") or "").strip()
        desc = str(meta.get("desc") or "").strip()
        default_obs = str(meta.get("default_observation") or "").strip()
        default_obs_x = str(meta.get("default_observation_x") or "").strip()
        if name or desc or default_obs or default_obs_x:
            return {
                "name": name,
                "desc": desc,
                "default_observation": default_obs,
                "default_observation_x": default_obs_x,
            }
    return {
        "name": "",
        "desc": "",
        "default_observation": "",
        "default_observation_x": "",
    }


def _apply_inventory_meta_fallback(row: dict) -> dict:
    ddn_paths = _to_string_list(row.get("ddn_path"))
    if not ddn_paths:
        return row

    warning_summary = _read_inventory_maegim_warning_summary(ddn_paths)

    parsed = _extract_meta_from_ddn_paths(ddn_paths)
    fallback_title = str(parsed.get("name") or "").strip()
    fallback_desc = str(parsed.get("desc") or "").strip()
    fallback_default_obs = str(parsed.get("default_observation") or "").strip()
    fallback_default_obs_x = str(parsed.get("default_observation_x") or "").strip()
    if not fallback_title and not fallback_desc and not fallback_default_obs and not fallback_default_obs_x:
        if not warning_summary.get("maegim_control_warning_count"):
            return row
        return {**row, **warning_summary}

    lesson_id = str(row.get("id") or "").strip()
    title = str(row.get("title") or "").strip()
    desc = str(row.get("description") or "").strip()
    default_obs = str(row.get("default_observation") or "").strip()
    default_obs_x = str(row.get("default_observation_x") or "").strip()
    placeholder_desc = {"seed lesson", "rewrite v1"}
    next_title = title
    next_desc = desc
    next_default_obs = default_obs
    next_default_obs_x = default_obs_x

    if fallback_title and (not title or (lesson_id and title == lesson_id)):
        next_title = fallback_title
    if fallback_desc and (not desc or desc.lower() in placeholder_desc):
        next_desc = fallback_desc
    if fallback_default_obs and not default_obs:
        next_default_obs = fallback_default_obs
    if fallback_default_obs_x and not default_obs_x:
        next_default_obs_x = fallback_default_obs_x

    if (
        next_title == title
        and next_desc == desc
        and next_default_obs == default_obs
        and next_default_obs_x == default_obs_x
        and not warning_summary.get("maegim_control_warning_count")
    ):
        return row
    return {
        **row,
        "title": next_title,
        "description": next_desc,
        "default_observation": next_default_obs,
        "default_observation_x": next_default_obs_x,
        **warning_summary,
    }


def _read_row_meta_candidates(row: dict) -> list[str]:
    out: list[str] = []
    out.extend(_to_string_list(row.get("meta_path")))
    out.extend(_to_string_list(row.get("metaCandidates")))
    out.extend(_to_string_list(row.get("meta_toml")))
    return out


def _merge_inventory_row(rows_by_id: dict[str, dict], next_row: dict):
    lesson_id = str(next_row.get("id", "")).strip()
    if not lesson_id:
        return
    current = rows_by_id.get(lesson_id)
    if current is None:
        rows_by_id[lesson_id] = next_row
        return
    merged = {
        **current,
        **next_row,
        "ddn_path": sorted(set([*(current.get("ddn_path") or []), *(next_row.get("ddn_path") or [])])),
        "maegim_control_path": sorted(
            set([*(current.get("maegim_control_path") or []), *(next_row.get("maegim_control_path") or [])])
        ),
        "text_path": sorted(set([*(current.get("text_path") or []), *(next_row.get("text_path") or [])])),
        "meta_path": sorted(set([*(current.get("meta_path") or []), *(next_row.get("meta_path") or [])])),
    }
    rows_by_id[lesson_id] = merged


def build_lesson_inventory_payload(catalog_mode: str | None = None) -> dict:
    mode = _normalize_catalog_mode(catalog_mode or RUNTIME_CATALOG_MODE)
    reps_only = mode == CATEGORY_REPS_ONLY
    active_allowlist = _read_active_lesson_allowlist()
    rows_by_id: dict[str, dict] = {}

    index_payload = _read_json_file(LESSON_DIR / "index.json") or {}
    index_rows = index_payload.get("lessons")
    if isinstance(index_rows, list):
        for row in index_rows:
            if not isinstance(row, dict):
                continue
            lesson_id = str(row.get("id", "")).strip()
            if not lesson_id:
                continue
            # A release projection must fail closed if its explicit representative
            # allowlist is absent or incomplete.  Falling back to every row in the
            # source catalog would expose deferred lesson lines as public release
            # inventory.
            if reps_only and lesson_id not in active_allowlist:
                continue
            ddn_path, text_path, meta_path = _lesson_paths("lessons", lesson_id)
            maegim_candidates = _infer_maegim_control_candidates([ddn_path], _to_string_list(row.get("maegim_control_path")))
            meta_candidates = _existing_meta_candidates([*_read_row_meta_candidates(row), meta_path])
            _merge_inventory_row(
                rows_by_id,
                {
                    "id": lesson_id,
                    "title": str(row.get("title") or lesson_id),
                    "description": str(row.get("description") or ""),
                    "grade": str(row.get("grade") or ""),
                    "subject": _normalize_subject(row.get("subject")),
                    "quality": "experimental",
                    "source": "official",
                    "ddn_path": [ddn_path],
                    "maegim_control_path": maegim_candidates,
                    "text_path": [text_path],
                    "meta_path": meta_candidates,
                },
            )

    if mode == CATEGORY_FULL:
        seed_payload = _read_json_file(SEED_LESSON_DIR / "seed_manifest.detjson") or {}
        seed_rows = seed_payload.get("seeds")
        if isinstance(seed_rows, list):
            for row in seed_rows:
                if not isinstance(row, dict):
                    continue
                lesson_id = str(row.get("seed_id", "")).strip()
                if not lesson_id:
                    continue
                fallback_ddn, fallback_text, fallback_meta = _lesson_paths("seed_lessons_v1", lesson_id)
                ddn_path = str(row.get("lesson_ddn") or "").strip() or fallback_ddn
                text_path = str(row.get("text_md") or "").strip() or fallback_text
                maegim_candidates = _infer_maegim_control_candidates(
                    [ddn_path, fallback_ddn],
                    _to_string_list(row.get("maegim_control_path")),
                )
                meta_candidates = _existing_meta_candidates([*_read_row_meta_candidates(row), fallback_meta])
                _merge_inventory_row(
                    rows_by_id,
                    {
                        "id": lesson_id,
                        "title": lesson_id,
                        "description": "Seed lesson",
                        "grade": "all",
                        "subject": _normalize_subject(row.get("subject")),
                        "quality": "recommended",
                        "source": "seed",
                        "ddn_path": [ddn_path, fallback_ddn],
                        "maegim_control_path": maegim_candidates,
                        "text_path": [text_path, fallback_text],
                        "meta_path": meta_candidates,
                    },
                )

        rewrite_payload = _read_json_file(REWRITE_LESSON_DIR / "rewrite_manifest.detjson") or {}
        rewrite_rows = rewrite_payload.get("generated")
        if isinstance(rewrite_rows, list):
            for row in rewrite_rows:
                if not isinstance(row, dict):
                    continue
                lesson_id = str(row.get("lesson_id", "")).strip()
                if not lesson_id:
                    continue
                fallback_ddn, fallback_text, fallback_meta = _lesson_paths("lessons_rewrite_v1", lesson_id)
                ddn_path = str(row.get("generated_lesson_ddn") or "").strip() or fallback_ddn
                text_path = str(row.get("generated_text_md") or "").strip() or fallback_text
                maegim_candidates = _infer_maegim_control_candidates(
                    [ddn_path, fallback_ddn],
                    _to_string_list(row.get("maegim_control_path")),
                )
                meta_candidates = _existing_meta_candidates([*_read_row_meta_candidates(row), fallback_meta])
                _merge_inventory_row(
                    rows_by_id,
                    {
                        "id": lesson_id,
                        "title": lesson_id,
                        "description": "Rewrite v1",
                        "grade": "all",
                        "subject": _normalize_subject(row.get("subject")),
                        "quality": "reviewed",
                        "source": "rewrite",
                        "ddn_path": [ddn_path, fallback_ddn],
                        "maegim_control_path": maegim_candidates,
                        "text_path": [text_path, fallback_text],
                        "meta_path": meta_candidates,
                    },
                )

    dispositions = _read_json_file(ALPHA1_LESSON_DIR / "dispositions.detjson") or {}
    for row in dispositions.get("lessons", []):
        lesson_id = str(row.get("id", ""))
        if row.get("disposition") != "알파1 새 구현":
            if reps_only:
                rows_by_id.pop(lesson_id, None)
            continue
        if lesson_id not in active_allowlist:
            continue
        source = str(row.get("source", ""))
        source_path = _project_path_to_fs(source)
        if not source_path or not source_path.is_file():
            continue
        previous = rows_by_id.get(lesson_id, {})
        # Replace, do not merge old and current execution sources under one ID.
        rows_by_id[lesson_id] = {
            **previous, "id": lesson_id, "title": previous.get("title", lesson_id),
            "source": "official", "quality": "reviewed",
            "source_profile": "v1-core-v25", "disposition": row["disposition"],
            "ddn_path": [source], "maegim_control_path": [],
            "meta_path": _existing_meta_candidates([source.replace("lesson.ddn", "meta.toml")]),
            "text_path": [],
        }
    lessons = sorted(
        [_apply_inventory_meta_fallback(row) for row in rows_by_id.values()],
        key=lambda item: (str(item.get("title", "")).lower(), str(item.get("id", "")).lower()),
    )
    return {
        "ok": True,
        "schema": "seamgrim.lesson_inventory.v2",
        "catalog_mode": mode,
        "active_allowlist_count": len(active_allowlist),
        "count": len(lessons),
        "lessons": lessons,
        "deferred": [] if reps_only else [row for row in dispositions.get("lessons", []) if row.get("disposition") == "이월"],
    }


def build_public_lesson_index_payload(catalog_mode: str | None = None) -> dict:
    """Return the catalog index through the same release projection as inventory.

    ``lessons/index.json`` remains the internal source catalog.  The HTTP release
    surface must not bypass the representative allowlist used by the inventory API.
    """
    mode = _normalize_catalog_mode(catalog_mode or RUNTIME_CATALOG_MODE)
    payload = _read_json_file(LESSON_DIR / "index.json") or {}
    rows = payload.get("lessons")
    if not isinstance(rows, list):
        return {**payload, "lessons": []}
    if mode != CATEGORY_REPS_ONLY:
        return {**payload, "lessons": rows}
    inventory = build_lesson_inventory_payload(mode)
    return {
        **payload,
        "lessons": inventory["lessons"],
        "deferred": inventory["deferred"],
    }


def json_response(handler: BaseHTTPRequestHandler, status: int, payload: dict):
    body = json.dumps(payload, ensure_ascii=False, sort_keys=True).encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json; charset=utf-8")
    handler.send_header("Cache-Control", "no-store, max-age=0")
    cors_origin = _origin_allowed_for_response(handler)
    if cors_origin:
        handler.send_header("Access-Control-Allow-Origin", cors_origin)
    handler.send_header("Access-Control-Allow-Headers", "Content-Type, Authorization")
    handler.send_header("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


def send_file(handler: BaseHTTPRequestHandler, path: Path):
    if not path.exists():
        handler.send_response(404)
        handler.end_headers()
        return
    content = path.read_bytes()
    suffix = path.suffix.lower()
    content_type = "text/plain; charset=utf-8"
    if suffix == ".html":
        content_type = "text/html; charset=utf-8"
    elif suffix == ".css":
        content_type = "text/css; charset=utf-8"
    elif suffix == ".js":
        content_type = "application/javascript; charset=utf-8"
    elif suffix == ".json":
        content_type = "application/json; charset=utf-8"
    elif suffix == ".detjson":
        content_type = "application/json; charset=utf-8"
    elif suffix == ".wasm":
        content_type = "application/wasm"
    elif suffix == ".pdf":
        content_type = "application/pdf"
    elif suffix == ".csv":
        content_type = "text/csv; charset=utf-8"
    elif suffix == ".toml":
        content_type = "text/plain; charset=utf-8"
    handler.send_response(200)
    handler.send_header("Cache-Control", "no-store, max-age=0")
    handler.send_header("Content-Type", content_type)
    cors_origin = _origin_allowed_for_response(handler)
    if cors_origin:
        handler.send_header("Access-Control-Allow-Origin", cors_origin)
    handler.send_header("Content-Length", str(len(content)))
    handler.end_headers()
    handler.wfile.write(content)


def send_redirect(handler: BaseHTTPRequestHandler, location: str, status: int = 302):
    handler.send_response(status)
    handler.send_header("Cache-Control", "no-store, max-age=0")
    handler.send_header("Location", location)
    handler.send_header("Content-Length", "0")
    handler.end_headers()


def _route_query(parsed) -> dict[str, list[str]]:
    try:
        return parse_qs(parsed.query or "", keep_blank_values=True)
    except Exception:
        return {}


def _wants_legacy_shell(parsed) -> bool:
    query = _route_query(parsed)
    for key in WORKBENCH_PROJECT_QUERY_KEYS:
        for value in query.get(key, []):
            if value and value not in WORKBENCH_ROUTE_PROJECT_IDS:
                return True
    return any(key in query for key in LEGACY_DEEP_LINK_KEYS)


def _safe_default_route_location(parsed) -> str:
    query = _route_query(parsed)
    if any(key.lower() in CAPABILITY_QUERY_KEYS for key in query):
        return WORKBENCH_DEFAULT_ROUTE
    return f"{WORKBENCH_DEFAULT_ROUTE}?{parsed.query}" if parsed.query else WORKBENCH_DEFAULT_ROUTE


def send_json_text(handler: BaseHTTPRequestHandler, text: str):
    body = str(text or "").encode("utf-8")
    handler.send_response(200)
    handler.send_header("Cache-Control", "no-store, max-age=0")
    handler.send_header("Content-Type", "application/json; charset=utf-8")
    cors_origin = _origin_allowed_for_response(handler)
    if cors_origin:
        handler.send_header("Access-Control-Allow-Origin", cors_origin)
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


def build_api_error_payload(
    *,
    status: int,
    code: str,
    user_message: str,
    technical_message: str,
    path: str = "",
    extra: dict | None = None,
) -> dict:
    normalized_code = str(code or "E_API_UNKNOWN").strip() or "E_API_UNKNOWN"
    user_text = str(user_message or "요청 처리에 실패했습니다.").strip() or "요청 처리에 실패했습니다."
    technical_text = str(technical_message or "request failed").strip() or "request failed"
    payload = {
        "ok": False,
        "status": int(status),
        "code": normalized_code,
        "technical_code": normalized_code,
        "user_message": user_text,
        "technical_message": technical_text,
        "error": technical_text,
        "diagnostics": [
            {
                "severity": "error",
                "code": normalized_code,
                "technical_code": normalized_code,
                "message": technical_text,
                "technical_message": technical_text,
                "user_message": user_text,
            }
        ],
    }
    normalized_path = str(path or "").strip()
    if normalized_path:
        payload["path"] = normalized_path
    if isinstance(extra, dict):
        for key, value in extra.items():
            if key in payload:
                continue
            payload[key] = value
    return payload


def project_data_dir() -> Path:
    config = active_runtime_config()
    target = Path(str(config.get("project_data_dir") or ""))
    target.mkdir(parents=True, exist_ok=True)
    return target.resolve()


def project_store() -> SeamgrimProjectStore:
    config = active_runtime_config()
    return SeamgrimProjectStore(
        project_data_dir(),
        sqlite_synchronous=str(config.get("sqlite_synchronous") or "NORMAL"),
    )


def read_json_body(handler: BaseHTTPRequestHandler, *, max_bytes: int = APP_JSON_BODY_LIMIT_BYTES) -> dict:
    try:
        length = int(handler.headers.get("Content-Length", "0"))
    except Exception:
        raise ProjectValidationError("invalid content length")
    if length < 0 or length > max_bytes:
        raise ProjectValidationError("request body too large")
    body = handler.rfile.read(length).decode("utf-8-sig")
    if not body.strip():
        return {}
    try:
        payload = json.loads(body)
    except Exception as exc:
        raise ProjectValidationError("invalid JSON body") from exc
    if not isinstance(payload, dict):
        raise ProjectValidationError("JSON object required")
    return payload


def bearer_capability(handler: BaseHTTPRequestHandler) -> str:
    header = str(handler.headers.get("Authorization", "")).strip()
    if not header:
        return ""
    parts = header.split(None, 1)
    if len(parts) != 2 or parts[0].lower() != "bearer":
        return ""
    return parts[1].strip()


def project_error_payload(status: int, code: str, message: str, path: str = "") -> dict:
    user_message_by_code = {
        "E_PROJECT_INVALID": "프로젝트 저장 요청 형식이 올바르지 않습니다.",
        "E_PROJECT_AUTH_REQUIRED": "프로젝트 링크 권한이 필요합니다.",
        "E_PROJECT_PERMISSION_DENIED": "이 링크로는 요청한 작업을 할 수 없습니다.",
        "E_PROJECT_NOT_FOUND": "프로젝트를 찾을 수 없습니다.",
        "E_PROJECT_DELETED": "삭제된 프로젝트입니다.",
        "E_PROJECT_REVISION_CONFLICT": "다른 창에서 먼저 저장되었습니다. 버전을 확인한 뒤 다시 저장하세요.",
        "E_PROJECT_TOO_LARGE": "프로젝트가 너무 큽니다. 내용을 줄인 뒤 다시 저장하세요.",
        "E_PROJECT_LIMIT": "이 프로젝트의 저장 기록 한도에 도달했습니다.",
        "E_PROJECT_SERVICE_UNAVAILABLE": "프로젝트 저장소를 사용할 수 없습니다. 잠시 후 다시 시도하세요.",
    }
    return build_api_error_payload(
        status=status,
        code=code,
        user_message=user_message_by_code.get(code, "프로젝트 요청 처리에 실패했습니다."),
        technical_message=message,
        path=path,
    )


def send_project_exception(handler: BaseHTTPRequestHandler, exc: Exception, path: str) -> None:
    if isinstance(exc, ProjectPermissionError):
        raw = bearer_capability(handler)
        code = "E_PROJECT_PERMISSION_DENIED" if raw else "E_PROJECT_AUTH_REQUIRED"
        status = 403 if raw else 401
        json_response(handler, status, project_error_payload(status, code, str(exc), path))
        return
    if isinstance(exc, ProjectConflictError):
        json_response(handler, 409, project_error_payload(409, exc.code, str(exc), path))
        return
    if isinstance(exc, ProjectDeletedError):
        json_response(handler, 404, project_error_payload(404, exc.code, str(exc), path))
        return
    if isinstance(exc, ProjectNotFoundError):
        json_response(handler, 404, project_error_payload(404, exc.code, str(exc), path))
        return
    if isinstance(exc, ProjectValidationError):
        status = 413 if "too large" in str(exc).lower() else 400
        code = "E_PROJECT_TOO_LARGE" if status == 413 else exc.code
        json_response(handler, status, project_error_payload(status, code, str(exc), path))
        return
    if isinstance(exc, ProjectStoreError):
        json_response(handler, exc.http_status, project_error_payload(exc.http_status, exc.code, str(exc), path))
        return
    LOGGER.exception("project service failed for %s", path, exc_info=exc)
    json_response(
        handler,
        500,
        project_error_payload(500, "E_PROJECT_SERVICE_UNAVAILABLE", "project service internal failure", path),
    )


def project_response_from_record(record: dict, capability_role: str = "") -> dict:
    return {
        "project_id": record.get("project_id"),
        "head_revision_id": record.get("head_revision_id"),
        "content_sha256": record.get("content_sha256"),
        "package_hash": record.get("package_hash"),
        "project_snapshot": record.get("project_snapshot"),
        "capability_role": capability_role,
        "origin_project_id": record.get("origin_project_id"),
        "origin_revision_id": record.get("origin_revision_id"),
    }


def build_public_wedge_readiness() -> dict:
    wasm_files = [
        UI_DIR / "wasm" / "ddonirang_tool.js",
        UI_DIR / "wasm" / "ddonirang_tool_bg.wasm",
        UI_DIR / "wasm" / "ddonirang_tool.manifest.detjson",
    ]
    artifacts = []
    for file in wasm_files:
      artifacts.append({
          "name": file.name,
          "exists": file.is_file(),
          "bytes": file.stat().st_size if file.is_file() else 0,
      })
    try:
        project_store()
        store_ready = True
        store_status = "ready"
    except Exception:
        store_ready = False
        store_status = "unavailable"
    wasm_ready = all(row["exists"] and row["bytes"] > 0 for row in artifacts)
    return {
        "ok": wasm_ready and store_ready,
        "schema": "seamgrim.public_wedge.readiness.v1",
        "service": "seamgrim_public_physics_wedge",
        "bind_policy": "loopback_default",
        "external_idc_deployment_claim": False,
        "wasm_artifact": {
            "ready": wasm_ready,
            "files": artifacts,
        },
        "project_store": {
            "ready": store_ready,
            "status": store_status,
        },
    }


def build_private_pilot_readiness() -> dict:
    config = active_runtime_config()
    wasm_files = [
        UI_DIR / "wasm" / "ddonirang_tool.js",
        UI_DIR / "wasm" / "ddonirang_tool_bg.wasm",
        UI_DIR / "wasm" / "ddonirang_tool.manifest.detjson",
    ]
    wasm_ready = all(path.is_file() and path.stat().st_size > 0 for path in wasm_files)
    try:
        store = project_store()
        pragmas = store.runtime_pragmas()
        store_ready = True
        store_status = "ready"
    except Exception:
        pragmas = {}
        store_ready = False
        store_status = "unavailable"
    return {
        "ok": bool(wasm_ready and store_ready and config.get("ok")),
        "schema": "seamgrim.private_pilot.readiness.v1",
        "service": "seamgrim_private_pilot",
        "profile": config.get("profile"),
        "public": False,
        "external_idc_deployment_claim": False,
        "release": {
            "commit": str(os.environ.get("SEAMGRIM_RELEASE_COMMIT", "")).strip() or "operator-input-missing",
            "manifest_sha256": str(os.environ.get("SEAMGRIM_RELEASE_MANIFEST_SHA256", "")).strip() or "operator-input-missing",
        },
        "wasm": {
            "ready": wasm_ready,
            "artifact_count": len(wasm_files),
        },
        "project_store": {
            "ready": store_ready,
            "status": store_status,
            "journal_mode": pragmas.get("journal_mode"),
            "synchronous": pragmas.get("synchronous"),
            "schema": pragmas.get("schema"),
        },
        "limits": {
            "app_json_body_limit_bytes": APP_JSON_BODY_LIMIT_BYTES,
        },
    }


def _backup_staging_summary() -> dict:
    raw = str(os.environ.get("SEAMGRIM_BACKUP_STAGING_DIR", "")).strip()
    if not raw:
        return {"configured": False, "status": "missing_operator_input"}
    path = Path(raw).expanduser().resolve()
    newest_mtime: float | None = None
    if path.exists():
        for candidate in path.rglob("*.detjson"):
            try:
                mtime = candidate.stat().st_mtime
            except OSError:
                continue
            newest_mtime = mtime if newest_mtime is None else max(newest_mtime, mtime)
    age_seconds = None
    if newest_mtime is not None:
        age_seconds = max(0, int(time.time() - newest_mtime))
    return {
        "configured": True,
        "staging_dir_redacted": _redacted_path_token(path),
        "newest_manifest_age_seconds": age_seconds,
        "status": "ok" if age_seconds is not None else "no_backup_manifest_found",
    }


def _disk_summary(path: Path) -> dict:
    try:
        target = path if path.exists() else path.parent
        usage = shutil.disk_usage(target)
        return {
            "path_redacted": _redacted_path_token(path),
            "free_bytes": int(usage.free),
            "total_bytes": int(usage.total),
            "status": "ok",
        }
    except Exception:
        return {"path_redacted": _redacted_path_token(path), "status": "unavailable"}


def build_operator_check() -> dict:
    config = active_runtime_config()
    data_dir = Path(str(config.get("project_data_dir") or ".")).expanduser().resolve()
    try:
        store = project_store()
        audit = store.audit_integrity()
        pragmas = store.runtime_pragmas()
        integrity_ok = bool(audit.get("ok"))
    except Exception as exc:
        audit = {"ok": False, "diagnostics": [{"code": "E_OPERATOR_STORE_UNAVAILABLE", "message": str(exc)}]}
        pragmas = {}
        integrity_ok = False

    wasm_manifest = UI_DIR / "wasm" / "ddonirang_tool.manifest.detjson"
    wasm_hash = _sha256_file(wasm_manifest) if wasm_manifest.is_file() else ""
    return {
        "ok": bool(config.get("ok") and integrity_ok),
        "schema": "seamgrim.private_pilot.operator_check.v1",
        "public": False,
        "release": {
            "commit": str(os.environ.get("SEAMGRIM_RELEASE_COMMIT", "")).strip() or "operator-input-missing",
            "manifest_sha256": str(os.environ.get("SEAMGRIM_RELEASE_MANIFEST_SHA256", "")).strip() or "operator-input-missing",
            "external_idc_deployment_claim": False,
        },
        "versions": {
            "python": sys.version.split()[0],
            "sqlite": sqlite3.sqlite_version,
            "project_store_schema": SCHEMA_VERSION,
        },
        "wasm": {
            "manifest_sha256": wasm_hash,
            "freshness_status": "present" if wasm_hash else "missing",
        },
        "store": {
            "integrity_ok": integrity_ok,
            "audit": audit,
            "runtime_pragmas": pragmas,
        },
        "disk": _disk_summary(data_dir),
        "backup": _backup_staging_summary(),
        "log_policy": {
            "fields": ["timestamp", "severity", "request_id", "route_class", "status", "elapsed_ms"],
            "forbidden_fields": ["capability", "Authorization", "DDN source", "project snapshot", "query string"],
            "rotation_retention": "operator-profile-required",
        },
        "alert_inputs": [
            "restart_loop",
            "http_5xx",
            "disk_free_bytes",
            "backup_age_seconds",
            "integrity_failure",
        ],
        "capacity_claim": False,
    }


def handle_project_get(handler: BaseHTTPRequestHandler, path: str) -> bool:
    if path == f"{PROJECT_API_PREFIX}/health":
        try:
            project_store()
            json_response(handler, 200, {
                "ok": True,
                "schema": "seamgrim.project_api.health.v0",
                "service": "seamgrim_project_service",
                "storage": "ready",
            })
        except Exception as exc:
            send_project_exception(handler, exc, path)
        return True
    if not path.startswith(f"{PROJECT_API_PREFIX}/"):
        return False
    parts = [part for part in path[len(PROJECT_API_PREFIX):].split("/") if part]
    if len(parts) not in (1, 2, 3):
        return False
    project_id = parts[0]
    try:
        store = project_store()
        if len(parts) == 1:
            role = store.authorize_capability(project_id, bearer_capability(handler), required_role=ROLE_VIEW)
            json_response(handler, 200, project_response_from_record(store.get_project(project_id), role))
            return True
        if parts[1] == "revisions":
            role = store.authorize_capability(project_id, bearer_capability(handler), required_role=ROLE_VIEW)
            json_response(handler, 200, {
                "project_id": project_id,
                "capability_role": role,
                "revisions": store.list_revisions(project_id),
            })
            return True
        if parts[1] == "runs" and len(parts) == 2:
            role = store.authorize_capability(project_id, bearer_capability(handler), required_role=ROLE_VIEW)
            json_response(handler, 200, {
                "project_id": project_id,
                "capability_role": role,
                "runs": store.list_runs(project_id),
            })
            return True
        if parts[1] == "runs" and len(parts) == 3:
            role = store.authorize_capability(project_id, bearer_capability(handler), required_role=ROLE_VIEW)
            json_response(handler, 200, {
                "project_id": project_id,
                "capability_role": role,
                "run": store.get_run(project_id, parts[2]),
            })
            return True
    except Exception as exc:
        send_project_exception(handler, exc, path)
        return True
    return False


def handle_project_post(handler: BaseHTTPRequestHandler, path: str) -> bool:
    if path == PROJECT_API_PREFIX:
        try:
            payload = read_json_body(handler)
            artifact = payload.get("project_snapshot_artifact")
            store = project_store()
            record = store.create_project(artifact, title=str(payload.get("title") or ""))
            caps = store.issue_capabilities(record["project_id"])
            json_response(handler, 201, {
                "project_id": record["project_id"],
                "revision_id": record["head_revision_id"],
                "content_sha256": record["content_sha256"],
                "package_hash": record["package_hash"],
                "edit_capability": caps["edit"],
                "view_capability": caps["view"],
                "created_at": record["created_at"],
            })
        except Exception as exc:
            send_project_exception(handler, exc, path)
        return True
    if not path.startswith(f"{PROJECT_API_PREFIX}/"):
        return False
    parts = [part for part in path[len(PROJECT_API_PREFIX):].split("/") if part]
    if len(parts) not in (2, 4):
        return False
    project_id, action = parts[0], parts[1]
    try:
        payload = read_json_body(handler)
        store = project_store()
        if action == "revisions":
            role = store.authorize_capability(project_id, bearer_capability(handler), required_role=ROLE_EDIT)
            record = store.add_revision(
                project_id,
                payload.get("project_snapshot_artifact"),
                expected_parent_revision_id=str(payload.get("expected_parent_revision_id") or ""),
            )
            json_response(handler, 200, {**project_response_from_record(record, role), "revision_id": record["head_revision_id"]})
            return True
        if action == "restore":
            role = store.authorize_capability(project_id, bearer_capability(handler), required_role=ROLE_EDIT)
            record = store.restore_revision(
                project_id,
                str(payload.get("source_revision_id") or ""),
                expected_parent_revision_id=str(payload.get("expected_parent_revision_id") or ""),
            )
            json_response(handler, 200, {**project_response_from_record(record, role), "revision_id": record["head_revision_id"]})
            return True
        if action == "clone":
            role = store.authorize_capability(project_id, bearer_capability(handler), required_role=ROLE_VIEW)
            record = store.clone_project(project_id, title=str(payload.get("title") or ""))
            caps = store.issue_capabilities(record["project_id"])
            lineage = record.get("project_snapshot", {}).get("lineage", {}) if isinstance(record.get("project_snapshot"), dict) else {}
            json_response(handler, 201, {
                **project_response_from_record(record, role),
                "revision_id": record["head_revision_id"],
                "edit_capability": caps["edit"],
                "view_capability": caps["view"],
                "origin_project_id": project_id,
                "origin_revision_id": record.get("origin_revision_id"),
                "origin_content_sha256": lineage.get("origin_content_sha256"),
            })
            return True
        if action == "runs" and len(parts) == 2:
            role = store.authorize_capability(project_id, bearer_capability(handler), required_role=ROLE_EDIT)
            record = store.create_run(
                project_id,
                source_revision_id=str(payload.get("source_revision_id") or ""),
                expected_head_revision_id=str(payload.get("expected_head_revision_id") or ""),
                recipe_hash=str(payload.get("recipe_hash") or ""),
                seed=str(payload.get("seed") or ""),
                parent_server_run_id=str(payload.get("parent_server_run_id") or ""),
                branch_madi=payload.get("branch_madi"),
                branch_state_hash=str(payload.get("branch_state_hash") or ""),
            )
            json_response(handler, 201, {
                "project_id": project_id,
                "capability_role": role,
                "run": record,
            })
            return True
        if action == "runs" and len(parts) == 4 and parts[3] == "checkpoints":
            role = store.authorize_capability(project_id, bearer_capability(handler), required_role=ROLE_EDIT)
            record = store.add_run_checkpoint(project_id, parts[2], payload.get("checkpoint") if isinstance(payload.get("checkpoint"), dict) else payload)
            json_response(handler, 200, {
                "project_id": project_id,
                "capability_role": role,
                "run": record,
            })
            return True
        if action == "runs" and len(parts) == 4 and parts[3] == "finalize":
            role = store.authorize_capability(project_id, bearer_capability(handler), required_role=ROLE_EDIT)
            record = store.finalize_run(
                project_id,
                parts[2],
                final_status=str(payload.get("final_status") or "stopped"),
            )
            json_response(handler, 200, {
                "project_id": project_id,
                "capability_role": role,
                "run": record,
            })
            return True
    except Exception as exc:
        send_project_exception(handler, exc, path)
        return True
    return False


def handle_project_delete(handler: BaseHTTPRequestHandler, path: str) -> bool:
    if not path.startswith(f"{PROJECT_API_PREFIX}/"):
        return False
    parts = [part for part in path[len(PROJECT_API_PREFIX):].split("/") if part]
    if len(parts) != 1:
        return False
    project_id = parts[0]
    try:
        store = project_store()
        role = store.authorize_capability(project_id, bearer_capability(handler), required_role=ROLE_EDIT)
        record = store.soft_delete_project(project_id)
        json_response(handler, 200, {
            "project_id": project_id,
            "deleted_at": record.get("deleted_at"),
            "capability_role": role,
        })
    except Exception as exc:
        send_project_exception(handler, exc, path)
    return True


def try_send_generated_maegim_control(handler: BaseHTTPRequestHandler, root_dir: Path, raw_sub_path: str) -> bool:
    requested_path = (root_dir / raw_sub_path).resolve()
    if root_dir not in requested_path.parents or requested_path.name != MAEGIM_CONTROL_FILENAME:
        return False
    if requested_path.exists():
        send_file(handler, requested_path)
        return True
    lesson_path = requested_path.with_name("lesson.ddn")
    if not lesson_path.exists():
        return False
    try:
        send_json_text(handler, _to_maegim_control_payload(lesson_path))
        return True
    except Exception as exc:
        json_response(
            handler,
            500,
            build_api_error_payload(
                status=500,
                code="E_API_MAEGIM_CONTROL_GENERATE_FAILED",
                user_message="매김 제어 정보를 준비하지 못했습니다.",
                technical_message=str(exc),
                path=str(raw_sub_path or ""),
                extra={"schema": MAEGIM_CONTROL_SCHEMA},
            ),
        )
        return True


def parse_space2d_points(lines: list[str]) -> list[dict]:
    points: list[dict] = []
    idx = 0
    while idx < len(lines):
        marker = lines[idx].strip()
        if marker in SPACE2D_MARKERS:
            nums = []
            j = idx + 1
            while j < len(lines) and len(nums) < 2:
                values, mode = _parse_numbers_from_line(lines[j])
                if mode == "single":
                    nums.extend(values)
                elif mode in ("pair", "triple") and not nums:
                    nums.append(values[0])
                    nums.append(values[1])
                j += 1
            if len(nums) == 2:
                points.append({"x": float(nums[0]), "y": float(nums[1])})
            idx = j
            continue
        idx += 1
    return points


def _parse_space2d_shape(lines: list[str], idx: int) -> tuple[dict | None, int]:
    while idx < len(lines) and not lines[idx].strip():
        idx += 1
    if idx >= len(lines):
        return None, idx
    kind = lines[idx].strip().lower()
    idx += 1
    data: dict[str, object] = {"kind": kind}
    while idx < len(lines):
        key = lines[idx].strip()
        if not key:
            idx += 1
            continue
        if key in SPACE2D_MARKERS or key in SPACE2D_SHAPE_MARKERS:
            break
        if key.startswith("series:"):
            break
        if key not in SPACE2D_SHAPE_KEYS:
            break
        if idx + 1 >= len(lines):
            idx += 1
            break
        value_line = lines[idx + 1].strip()
        if key in (
            "stroke",
            "fill",
            "color",
            "token",
            "id",
            "name",
            "label",
            "토큰",
            "group_id",
            "group",
            "groupId",
            "그룹",
            "묶음",
        ):
            data[key] = value_line
        else:
            try:
                data[key] = float(value_line)
            except Exception:
                pass
        idx += 2

    if kind in ("line", "선", "segment"):
        if not all(k in data for k in ("x1", "y1", "x2", "y2")):
            return None, idx
        result = {
            "kind": "line",
            "x1": float(data["x1"]),
            "y1": float(data["y1"]),
            "x2": float(data["x2"]),
            "y2": float(data["y2"]),
            "stroke": data.get("stroke"),
            "width": data.get("width"),
        }
        if data.get("token") or data.get("토큰"):
            result["token"] = data.get("token") or data.get("토큰")
        if data.get("id") or data.get("name") or data.get("label"):
            result["id"] = data.get("id") or data.get("name") or data.get("label")
        if (
            data.get("group_id")
            or data.get("group")
            or data.get("groupId")
            or data.get("그룹")
            or data.get("묶음")
        ):
            result["group_id"] = (
                data.get("group_id")
                or data.get("group")
                or data.get("groupId")
                or data.get("그룹")
                or data.get("묶음")
            )
        return result, idx
    if kind in ("circle", "원"):
        cx = data.get("cx", data.get("x"))
        cy = data.get("cy", data.get("y"))
        r = data.get("r")
        if cx is None or cy is None or r is None:
            return None, idx
        result = {
            "kind": "circle",
            "x": float(cx),
            "y": float(cy),
            "r": float(r),
            "stroke": data.get("stroke"),
            "fill": data.get("fill"),
            "width": data.get("width"),
        }
        if data.get("token") or data.get("토큰"):
            result["token"] = data.get("token") or data.get("토큰")
        if data.get("id") or data.get("name") or data.get("label"):
            result["id"] = data.get("id") or data.get("name") or data.get("label")
        if (
            data.get("group_id")
            or data.get("group")
            or data.get("groupId")
            or data.get("그룹")
            or data.get("묶음")
        ):
            result["group_id"] = (
                data.get("group_id")
                or data.get("group")
                or data.get("groupId")
                or data.get("그룹")
                or data.get("묶음")
            )
        return result, idx
    if kind in ("point", "점"):
        if "x" not in data or "y" not in data:
            return None, idx
        result = {
            "kind": "point",
            "x": float(data["x"]),
            "y": float(data["y"]),
            "size": data.get("size"),
            "color": data.get("color"),
            "stroke": data.get("stroke"),
        }
        if data.get("token") or data.get("토큰"):
            result["token"] = data.get("token") or data.get("토큰")
        if data.get("id") or data.get("name") or data.get("label"):
            result["id"] = data.get("id") or data.get("name") or data.get("label")
        if (
            data.get("group_id")
            or data.get("group")
            or data.get("groupId")
            or data.get("그룹")
            or data.get("묶음")
        ):
            result["group_id"] = (
                data.get("group_id")
                or data.get("group")
                or data.get("groupId")
                or data.get("그룹")
                or data.get("묶음")
            )
        return result, idx
    return None, idx


def parse_space2d_shapes(lines: list[str]) -> list[dict]:
    current: list[dict] = []
    latest: list[dict] = []
    idx = 0
    while idx < len(lines):
        marker = lines[idx].strip()
        if marker in SPACE2D_MARKERS:
            if current:
                latest = current
                current = []
            idx += 1
            continue
        if marker in SPACE2D_SHAPE_MARKERS:
            shape, next_idx = _parse_space2d_shape(lines, idx + 1)
            if shape:
                current.append(shape)
            idx = next_idx
            continue
        idx += 1
    if current:
        latest = current
    return latest


def parse_text_blocks(lines: list[str]) -> str:
    blocks: list[str] = []
    idx = 0
    while idx < len(lines):
        raw = lines[idx].rstrip("\n")
        trimmed = raw.strip()
        lower = trimmed.lower()
        if lower in TEXT_MARKERS:
            idx += 1
            buf: list[str] = []
            while idx < len(lines):
                line = lines[idx].rstrip("\n")
                t = line.strip()
                lower_t = t.lower()
                if not t:
                    if buf:
                        break
                    idx += 1
                    continue
                if (
                    lower_t in TEXT_END_MARKERS
                    or lower_t in SPACE2D_MARKERS
                    or lower_t in SPACE2D_SHAPE_MARKERS
                    or lower_t.startswith("series:")
                ):
                    break
                buf.append(line)
                idx += 1
            if buf:
                blocks.append("\n".join(buf).strip())
            continue
        prefix = next((p for p in TEXT_PREFIXES if lower.startswith(p)), None)
        if prefix:
            blocks.append(raw.split(":", 1)[1].strip())
        idx += 1
    return "\n\n".join(blocks).strip()


def parse_table_blocks(lines: list[str]) -> dict | None:
    """DDN stdout에서 테이블 마커를 인식하여 seamgrim.table.v0 구조를 반환한다."""
    explicit = parse_explicit_table_blocks(lines)
    if explicit:
        return explicit
    return parse_table_row_blocks(lines)


def parse_explicit_table_blocks(lines: list[str]) -> dict | None:
    idx = 0
    while idx < len(lines):
        trimmed = lines[idx].strip().lower()
        if trimmed not in TABLE_MARKERS:
            idx += 1
            continue
        idx += 1
        while idx < len(lines) and not lines[idx].strip():
            idx += 1
        if idx >= len(lines):
            return None
        header_line = lines[idx].strip()
        sep = "\t" if "\t" in header_line else ","
        headers = [h.strip() for h in header_line.split(sep)]
        columns = []
        for h in headers:
            columns.append({"key": h, "label": h})
        idx += 1
        rows: list[dict] = []
        while idx < len(lines):
            line = lines[idx].strip()
            lower_line = line.lower()
            if not line or lower_line in TABLE_END_MARKERS:
                idx += 1
                break
            if lower_line in TEXT_MARKERS or lower_line in SPACE2D_MARKERS or lower_line.startswith("series:"):
                break
            cells = [c.strip() for c in line.split(sep)]
            row: dict = {}
            for i, col in enumerate(columns):
                val = cells[i] if i < len(cells) else ""
                row[col["key"]] = _coerce_table_cell(val)
            rows.append(row)
            idx += 1
        if columns and rows:
            return finalize_table(columns, rows)
    return None


def parse_table_row_blocks(lines: list[str]) -> dict | None:
    idx = 0
    rows: list[dict] = []
    column_order: list[str] = []
    while idx < len(lines):
        marker = lines[idx].strip().lower()
        if marker not in TABLE_ROW_MARKERS:
            idx += 1
            continue
        idx += 1
        row: dict = {}
        while idx < len(lines):
            key = lines[idx].strip()
            lower_key = key.lower()
            if not key:
                idx += 1
                continue
            if (
                lower_key in TABLE_ROW_MARKERS
                or lower_key in TABLE_MARKERS
                or lower_key in TABLE_END_MARKERS
                or lower_key in TEXT_MARKERS
                or lower_key in SPACE2D_MARKERS
                or lower_key in SPACE2D_SHAPE_MARKERS
                or lower_key in STRUCTURE_MARKERS
                or lower_key in STRUCTURE_END_MARKERS
                or lower_key.startswith("series:")
            ):
                break
            if idx + 1 >= len(lines):
                break
            value_line = lines[idx + 1].strip()
            lower_value = value_line.lower()
            if (
                lower_value in TABLE_ROW_MARKERS
                or lower_value in TABLE_MARKERS
                or lower_value in TABLE_END_MARKERS
                or lower_value in TEXT_MARKERS
                or lower_value in SPACE2D_MARKERS
                or lower_value in SPACE2D_SHAPE_MARKERS
                or lower_value in STRUCTURE_MARKERS
                or lower_value in STRUCTURE_END_MARKERS
                or lower_value.startswith("series:")
            ):
                break
            if key not in column_order:
                column_order.append(key)
            row[key] = _coerce_table_cell(value_line)
            idx += 2
        if row:
            rows.append(row)
            continue
        idx += 1
    if not rows or not column_order:
        return None
    columns = [{"key": key, "label": key} for key in column_order]
    return finalize_table(columns, rows)


def parse_console_output_log(lines: list[str], tick: int = 0) -> list[dict]:
    structured_only_markers = {
        "table.row",
        "space2d",
        "space2d.shape",
        "space2d_shape",
        "shape2d",
        "text.overlay",
        "overlay.text",
        "subtitle",
        "자막",
    }
    out: list[dict] = []
    for raw in lines:
        text = str(raw or "").strip()
        if not text:
            continue
        if text.lower() in structured_only_markers:
            continue
        out.append(
            {
                "tick": max(0, int(tick or 0)),
                "line_no": len(out) + 1,
                "text": text,
                "kind": "output",
            }
        )
    return out


def _coerce_table_cell(value: str):
    try:
        return float(value) if "." in value else int(value)
    except ValueError:
        return value


def finalize_table(columns: list[dict], rows: list[dict]) -> dict:
    for col in columns:
        key = col["key"]
        if all(isinstance(r.get(key), (int, float)) for r in rows):
            col["type"] = "number"
        else:
            col["type"] = "string"
    return {"columns": columns, "rows": rows}


def parse_structure_blocks(lines: list[str]) -> dict | None:
    """DDN stdout에서 구조 마커를 인식하여 seamgrim.structure.v0 구조를 반환한다.

    형식:
        structure
        node <id> [label]
        edge <from> <to> [label]
        structure.end
    """
    idx = 0
    while idx < len(lines):
        trimmed = lines[idx].strip().lower()
        if trimmed not in STRUCTURE_MARKERS:
            idx += 1
            continue
        idx += 1
        nodes: list[dict] = []
        edges: list[dict] = []
        node_ids: set[str] = set()
        while idx < len(lines):
            line = lines[idx].strip()
            lower_line = line.lower()
            if not line or lower_line in STRUCTURE_END_MARKERS:
                idx += 1
                break
            if lower_line in TEXT_MARKERS or lower_line in SPACE2D_MARKERS or lower_line in TABLE_MARKERS:
                break
            parts = line.split()
            cmd = parts[0].lower()
            if cmd in ("node", "노드") and len(parts) >= 2:
                nid = parts[1]
                label = " ".join(parts[2:]) if len(parts) > 2 else nid
                if nid not in node_ids:
                    nodes.append({"id": nid, "label": label})
                    node_ids.add(nid)
            elif cmd in ("edge", "간선", "연결") and len(parts) >= 3:
                edge: dict = {"from": parts[1], "to": parts[2], "directed": True}
                if len(parts) > 3:
                    edge["label"] = " ".join(parts[3:])
                edges.append(edge)
            idx += 1
        if nodes:
            return {"nodes": nodes, "edges": edges}
    return None


class DdnExecHandler(BaseHTTPRequestHandler):
    def log_message(self, format: str, *args) -> None:
        parsed = urlparse(self.path)
        route_class = parsed.path or "/"
        LOGGER.info(
            "http_request method=%s route=%s client=%s",
            getattr(self, "command", ""),
            route_class,
            self.client_address[0] if self.client_address else "",
        )

    def do_OPTIONS(self):
        boundary = request_boundary_error(self)
        if boundary:
            reject_request_boundary(self, boundary[0], boundary[1])
            return
        self.send_response(204)
        self.send_header("Cache-Control", "no-store, max-age=0")
        cors_origin = _origin_allowed_for_response(self)
        if cors_origin:
            self.send_header("Access-Control-Allow-Origin", cors_origin)
        self.send_header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS")
        self.end_headers()

    def do_GET(self):
        boundary = request_boundary_error(self)
        if boundary:
            reject_request_boundary(self, boundary[0], boundary[1])
            return
        parsed = urlparse(self.path)
        raw_path = parsed.path
        if handle_project_get(self, raw_path):
            return
        if raw_path == "/health" or raw_path == "/api/health":
            json_response(self, 200, {
                "ok": True,
                "schema": "seamgrim.public_health.v1",
                "service": "ddn_exec_server",
            })
            return
        if raw_path == "/api/private/readiness":
            if not management_request_allowed(self):
                reject_management_boundary(self)
                return
            payload = build_private_pilot_readiness()
            json_response(self, 200 if payload.get("ok") else 503, payload)
            return
        if raw_path == "/api/operator/check":
            if not management_request_allowed(self):
                reject_management_boundary(self, "E_PRIVATE_PILOT_OPERATOR_CHECK_FORBIDDEN")
                return
            payload = build_operator_check()
            json_response(self, 200 if payload.get("ok") else 503, payload)
            return
        if raw_path in ("/ready", "/readiness", "/api/readiness", "/api/public-wedge/readiness"):
            if active_runtime_config().get("private_pilot"):
                json_response(
                    self,
                    403,
                    build_api_error_payload(
                        status=403,
                        code="E_PRIVATE_PILOT_READINESS_NOT_PUBLIC",
                        user_message="private pilot readiness는 공개 health가 아니라 management 경로에서만 제공됩니다.",
                        technical_message="use /api/private/readiness on the management boundary",
                        path=raw_path,
                    ),
                )
                return
            payload = build_public_wedge_readiness()
            json_response(self, 200 if payload.get("ok") else 503, payload)
            return
        if raw_path in ("/api/lessons/inventory", "/api/lesson-inventory"):
            try:
                json_response(self, 200, build_lesson_inventory_payload(RUNTIME_CATALOG_MODE))
            except Exception as exc:
                json_response(
                    self,
                    500,
                    build_api_error_payload(
                        status=500,
                        code="E_API_INVENTORY_BUILD_FAILED",
                        user_message="교과 목록을 불러오지 못했습니다.",
                        technical_message=str(exc),
                        path=raw_path,
                    ),
                )
            return
        if raw_path == "/favicon.ico":
            self.send_response(204)
            self.send_header("Cache-Control", "no-store, max-age=0")
            self.end_headers()
            return

        # Browsers percent-encode Korean course names. Decode only static-file
        # paths; API routing and all resolved-path containment checks stay intact.
        try:
            path = unquote(raw_path, encoding="utf-8", errors="strict")
        except UnicodeDecodeError:
            json_response(self, 400, {"ok": False, "code": "E_STATIC_PATH_UTF8",
                                     "message": "파일 주소는 올바른 UTF-8이어야 합니다."})
            return
        if path.startswith(PROJECT_HTTP_PREFIX):
            stripped = path[len(PROJECT_HTTP_PREFIX):]
            path = stripped if stripped else "/"

        if path in LEGACY_SHELL_PATHS:
            send_file(self, UI_DIR / "index.html")
            return
        if path in DEFAULT_SHELL_PATHS:
            if _wants_legacy_shell(parsed):
                send_file(self, UI_DIR / "index.html")
            else:
                send_redirect(self, _safe_default_route_location(parsed))
            return
        if path == "/lessons/index.json":
            json_response(self, 200, build_public_lesson_index_payload(RUNTIME_CATALOG_MODE))
            return
        for course in ("first_steps", "first_steps_alpha2", "first_steps_alpha3"):
            prefix = f"/{course}/"
            if path.startswith(prefix):
                course_root = (UI_DIR.parent / course).resolve()
                course_path = (course_root / path[len(prefix):]).resolve()
                if course_root in course_path.parents:
                    send_file(self, course_path)
                    return
                self.send_error(404)
                return
        if path.startswith("/lessons_v25_alpha1/"):
            lesson_path = (ALPHA1_LESSON_DIR / path[len("/lessons_v25_alpha1/"):]).resolve()
            if ALPHA1_LESSON_DIR in lesson_path.parents:
                send_file(self, lesson_path)
                return
        if path.startswith("/lessons/"):
            parts = path[len("/lessons/"):].split("/")
            dispositions = _read_json_file(ALPHA1_LESSON_DIR / "dispositions.detjson") or {}
            declaration = next((row for row in dispositions.get("lessons", []) if row.get("id") == parts[0]), None)
            if RUNTIME_CATALOG_MODE == CATEGORY_REPS_ONLY and declaration:
                if declaration.get("disposition") == "이월":
                    json_response(self, 410, {"ok": False, "code": "E_ALPHA1_LESSON_DEFERRED", "message": declaration["reason"], "stage": declaration["stage"]})
                    return
                if len(parts) == 2 and parts[1] == "lesson.ddn":
                    send_file(self, _project_path_to_fs(declaration["source"]))
                    return
            if try_send_generated_maegim_control(self, LESSON_DIR, path[len("/lessons/"):]):
                return
            lesson_path = (LESSON_DIR / path[len("/lessons/"):]).resolve()
            if LESSON_DIR in lesson_path.parents or lesson_path == LESSON_DIR:
                send_file(self, lesson_path)
                return
        if path.startswith("/seed_lessons_v1/"):
            if _is_retired_line_request_blocked():
                json_response(
                    self,
                    410,
                    build_api_error_payload(
                        status=410,
                        code="E_SEAMGRIM_RETIRED_LINE_BLOCKED",
                        user_message="이 교과 라인은 현재 릴리즈 실행선에서 비활성화되었습니다.",
                        technical_message="retired lesson line is disabled in reps_only release mode",
                        path=path,
                    ),
                )
                return
            if try_send_generated_maegim_control(self, SEED_LESSON_DIR, path[len("/seed_lessons_v1/"):]):
                return
            seed_path = (SEED_LESSON_DIR / path[len("/seed_lessons_v1/"):]).resolve()
            if SEED_LESSON_DIR in seed_path.parents or seed_path == SEED_LESSON_DIR:
                send_file(self, seed_path)
                return
        if path.startswith("/lessons_rewrite_v1/"):
            if _is_retired_line_request_blocked():
                json_response(
                    self,
                    410,
                    build_api_error_payload(
                        status=410,
                        code="E_SEAMGRIM_RETIRED_LINE_BLOCKED",
                        user_message="이 교과 라인은 현재 릴리즈 실행선에서 비활성화되었습니다.",
                        technical_message="retired lesson line is disabled in reps_only release mode",
                        path=path,
                    ),
                )
                return
            if try_send_generated_maegim_control(self, REWRITE_LESSON_DIR, path[len("/lessons_rewrite_v1/"):]):
                return
            rewrite_path = (REWRITE_LESSON_DIR / path[len("/lessons_rewrite_v1/"):]).resolve()
            if REWRITE_LESSON_DIR in rewrite_path.parents or rewrite_path == REWRITE_LESSON_DIR:
                send_file(self, rewrite_path)
                return
        if path.startswith("/schema/"):
            schema_path = (SCHEMA_DIR / path[len("/schema/"):]).resolve()
            if SCHEMA_DIR in schema_path.parents or schema_path == SCHEMA_DIR:
                send_file(self, schema_path)
                return
        if path.startswith("/samples/"):
            if path in ("/samples/alpha3_logical_reaction.ddn", "/samples/alpha3_basic_tensor.ddn"):
                # These released course assets live in the repository samples,
                # not the UI samples. Do not expose the whole repository tree.
                send_file(self, ROOT / path.lstrip("/"))
                return
            sample_path = (SAMPLES_DIR / path[len("/samples/"):]).resolve()
            if SAMPLES_DIR in sample_path.parents or sample_path == SAMPLES_DIR:
                send_file(self, sample_path)
                return
        if path.startswith("/build/reports/"):
            report_root = (resolve_build_dir() / "reports").resolve()
            report_path = (report_root / path[len("/build/reports/"):]).resolve()
            if report_root in report_path.parents or report_path == report_root:
                send_file(self, report_path)
                return
        ui_relative = path
        if ui_relative.startswith("/ui/"):
            ui_relative = ui_relative[len("/ui/"):]
        else:
            ui_relative = ui_relative.lstrip("/")
        file_path = (UI_DIR / ui_relative).resolve()
        if UI_DIR in file_path.parents:
            send_file(self, file_path)
            return
        self.send_response(404)
        self.end_headers()

    def do_POST(self):
        boundary = request_boundary_error(self)
        if boundary:
            reject_request_boundary(self, boundary[0], boundary[1])
            return
        parsed = urlparse(self.path)
        if handle_project_post(self, parsed.path):
            return
        if parsed.path != "/api/run":
            json_response(
                self,
                404,
                build_api_error_payload(
                    status=404,
                    code="E_API_NOT_FOUND",
                    user_message="요청 경로를 찾을 수 없습니다.",
                    technical_message="not found",
                    path=parsed.path,
                ),
            )
            return
        temp_path: Path | None = None
        try:
            try:
                length = int(self.headers.get("Content-Length", "0"))
            except Exception:
                json_response(
                    self,
                    400,
                    build_api_error_payload(
                        status=400,
                        code="E_API_RUN_INVALID_CONTENT_LENGTH",
                        user_message="실행 요청 크기 정보를 확인할 수 없습니다.",
                        technical_message="invalid content length",
                        path=parsed.path,
                    ),
                )
                return
            if length < 0 or length > APP_JSON_BODY_LIMIT_BYTES:
                json_response(
                    self,
                    413,
                    build_api_error_payload(
                        status=413,
                        code="E_API_RUN_BODY_TOO_LARGE",
                        user_message="실행 요청 본문이 너무 큽니다. 내용을 줄인 뒤 다시 시도해 주세요.",
                        technical_message="request body too large",
                        path=parsed.path,
                    ),
                )
                return
            body = self.rfile.read(length).decode("utf-8-sig")
            try:
                payload = json.loads(body)
            except Exception:
                json_response(
                    self,
                    400,
                    build_api_error_payload(
                        status=400,
                        code="E_API_RUN_INVALID_JSON",
                        user_message="실행 요청 형식이 올바르지 않습니다. JSON 본문을 확인해 주세요.",
                        technical_message="invalid JSON body",
                        path=parsed.path,
                    ),
                )
                return
            ddn_text = payload.get("ddn_text")
            label = payload.get("label")
            madi_raw = payload.get("madi")
            madi = None
            if madi_raw is not None:
                try:
                    parsed_madi = int(madi_raw)
                    if parsed_madi > 0:
                        madi = parsed_madi
                except Exception:
                    madi = None
            if not isinstance(ddn_text, str):
                json_response(
                    self,
                    400,
                    build_api_error_payload(
                        status=400,
                        code="E_API_RUN_DDN_TEXT_REQUIRED",
                        user_message="실행할 DDN 본문이 비어 있습니다. 입력을 확인해 주세요.",
                        technical_message="ddn_text required",
                        path=parsed.path,
                    ),
                )
                return
            ddn_text = _strip_utf8_bom_prefix(ddn_text)
            if not ddn_text.strip():
                json_response(
                    self,
                    400,
                    build_api_error_payload(
                        status=400,
                        code="E_API_RUN_DDN_TEXT_REQUIRED",
                        user_message="실행할 DDN 본문이 비어 있습니다. 입력을 확인해 주세요.",
                        technical_message="ddn_text required",
                        path=parsed.path,
                    ),
                )
                return

            build_dir = resolve_build_dir()
            with tempfile.NamedTemporaryFile(
                mode="w",
                encoding="utf-8",
                suffix=".ddn",
                prefix="seamgrim_ui_mvp_input_",
                dir=build_dir,
                delete=False,
            ) as temp_file:
                # Raw V25 source is the official CLI input.  This active path
                # deliberately has no legacy lowering or private CLI fallback.
                temp_file.write(ddn_text)
                temp_path = Path(temp_file.name)
            # Presentation metadata remains a read-only local analysis.  It
            # must not select or invoke a second runtime for /api/run.
            maegim_control_payload = _build_legacy_maegim_control_plan(ddn_text, "<api-run>")
            maegim_control_source = "metadata"
            maegim_controls = maegim_control_payload.get("controls", [])
            maegim_warnings = maegim_control_payload.get("warnings", [])
            maegim_control_names = [
                str(item.get("name") or "").strip()
                for item in maegim_controls
                if isinstance(item, dict) and str(item.get("name") or "").strip()
            ]
            maegim_warning_codes = [
                str(item.get("code") or "").strip()
                for item in maegim_warnings
                if isinstance(item, dict) and str(item.get("code") or "").strip()
            ]

            meta = extract_meta(ddn_text)
            series_labels = extract_series_labels(ddn_text)
            source_hash = hash_text(normalize_ddn_for_hash(ddn_text))
            official_summary = run_ddonilang(ROOT, temp_path, madi=madi)
            raw_output_lines = official_summary.get("output_log_texts")
            if not isinstance(raw_output_lines, list):
                raise RuntimeError("E_DDONILANG_RUN_SUMMARY_INVALID:output_log_texts_required")
            lines = [str(line).strip() for line in raw_output_lines if str(line).strip()]
            # The official runtime output log may contain ordinary scalar/text
            # output rather than graph pairs.  Preserve that output below and
            # leave the graph empty instead of reinterpreting it as legacy data.
            try:
                points = parse_points(lines, series_labels)
            except ValueError:
                points = []
            space2d_points = parse_space2d_points(lines)
            space2d_shapes = parse_space2d_shapes(lines)
            text_block = parse_text_blocks(lines)
            table_data = parse_table_blocks(lines)
            structure_data = parse_structure_blocks(lines)
            console_log = parse_console_output_log(lines, madi)

            if not label:
                label = meta.get("name") or meta.get("desc") or "f(x)"

            graph = build_graph(points, label, source_hash)
            graph["meta"]["source_mode"] = "ddonilang-run-summary"
            if series_labels:
                graph["meta"]["series_labels"] = series_labels
                graph["series"][0]["id"] = normalize_series_label(series_labels[0])
            if meta.get("name"):
                graph["meta"]["input_name"] = meta["name"]
            if meta.get("desc"):
                graph["meta"]["input_desc"] = meta["desc"]
            graph["meta"]["maegim_control_count"] = len(maegim_control_names)
            graph["meta"]["maegim_control_names"] = maegim_control_names
            graph["meta"]["maegim_control_source"] = maegim_control_source
            graph["meta"]["maegim_control_warning_count"] = len(maegim_warning_codes)
            graph["meta"]["maegim_control_warning_codes"] = maegim_warning_codes

            payload = {"ok": True, "graph": graph}
            payload["maegim_control"] = maegim_control_payload
            payload["console_log"] = console_log
            payload["runtime"] = {
                "schema": "seamgrim.runtime_summary.v1",
                "source_profile_identity": str(official_summary.get("source_profile_identity") or ""),
                "state_hash": str(official_summary.get("state_hash") or ""),
                "work_cost_receipt": official_summary.get("work_cost_receipt"),
                "maegim_control_count": len(maegim_control_names),
                "maegim_control_names": maegim_control_names,
                "maegim_control_source": maegim_control_source,
                "maegim_control_warning_count": len(maegim_warning_codes),
                "maegim_control_warning_codes": maegim_warning_codes,
                "console_log_count": len(console_log),
            }
            if space2d_points or space2d_shapes:
                space2d_payload = {
                    "schema": "seamgrim.space2d.v0",
                    "points": space2d_points,
                    "shapes": space2d_shapes,
                    "meta": {
                        "title": meta.get("name") or "space2d",
                        "source_input_hash": source_hash,
                    },
                }
                if space2d_points:
                    xs = [point["x"] for point in space2d_points]
                    ys = [point["y"] for point in space2d_points]
                    space2d_payload["camera"] = {
                        "x_min": min(xs),
                        "x_max": max(xs),
                        "y_min": min(ys),
                        "y_max": max(ys),
                    }
                payload["space2d"] = space2d_payload
            if text_block:
                payload["text"] = {
                    "schema": "seamgrim.text.v0",
                    "content": text_block,
                    "format": "markdown",
                    "meta": {
                        "title": meta.get("name") or "text",
                        "source_input_hash": source_hash,
                    },
                }
            if table_data:
                payload["table"] = {
                    "schema": "seamgrim.table.v0",
                    "columns": table_data["columns"],
                    "rows": table_data["rows"],
                    "meta": {
                        "title": meta.get("name") or "table",
                        "source_input_hash": source_hash,
                    },
                }
            if structure_data:
                payload["structure"] = {
                    "schema": "seamgrim.structure.v0",
                    "nodes": structure_data["nodes"],
                    "edges": structure_data["edges"],
                    "layout": {"type": "dag"},
                    "meta": {
                        "title": meta.get("name") or "structure",
                        "source_input_hash": source_hash,
                    },
                }
            view_families: list[str] = []
            view_contract_by_family: dict[str, dict] = {}
            for family in ["graph", "space2d", "table", "text", "structure"]:
                family_view = payload.get(family)
                if not isinstance(family_view, dict):
                    continue
                family_schema = str(family_view.get("schema", "")).strip()
                view_families.append(family)
                view_contract_by_family[family] = {
                    "schema": family_schema,
                    "source": "api_run",
                    "available": True,
                }
            payload["view_contract"] = {
                "schema": "seamgrim.view_contract.v1",
                "source": "api_run",
                "families": view_families,
                "by_family": view_contract_by_family,
            }
            payload["runtime"]["view_families"] = list(view_families)
            payload["runtime"]["view_contract_schema"] = "seamgrim.view_contract.v1"
            json_response(self, 200, payload)
        except Exception as exc:
            technical_message = str(exc)
            diagnostic_code = (
                "E_V25_DEFINITION_MIGRATION_REQUIRED"
                if "E_V25_DEFINITION_MIGRATION_REQUIRED" in technical_message
                else "E_API_RUN_EXEC_FAILED"
            )
            json_response(
                self,
                500,
                build_api_error_payload(
                    status=500,
                    code=diagnostic_code,
                    user_message="실행 중 오류가 발생했습니다. 입력/설정을 확인한 뒤 다시 시도해 주세요.",
                    technical_message=technical_message,
                    path=parsed.path,
                ),
            )
        finally:
            if temp_path is not None:
                try:
                    temp_path.unlink(missing_ok=True)
                except Exception:
                    pass

    def do_DELETE(self):
        boundary = request_boundary_error(self)
        if boundary:
            reject_request_boundary(self, boundary[0], boundary[1])
            return
        parsed = urlparse(self.path)
        if handle_project_delete(self, parsed.path):
            return
        json_response(
            self,
            404,
            build_api_error_payload(
                status=404,
                code="E_API_NOT_FOUND",
                user_message="요청 경로를 찾을 수 없습니다.",
                technical_message="not found",
                path=parsed.path,
            ),
        )


def parse_args(argv: list[str] | None = None):
    parser = argparse.ArgumentParser(description="Seamgrim DDN exec server")
    parser.add_argument(
        "--host",
        default=os.environ.get("DDN_EXEC_SERVER_HOST", "127.0.0.1"),
        help="bind host (default: env DDN_EXEC_SERVER_HOST or 127.0.0.1)",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=int(os.environ.get("DDN_EXEC_SERVER_PORT", "8787")),
        help="bind port (default: env DDN_EXEC_SERVER_PORT or 8787)",
    )
    parser.add_argument(
        "--catalog-mode",
        choices=[CATEGORY_REPS_ONLY, CATEGORY_FULL],
        default=RUNTIME_CATALOG_MODE,
        help="lesson inventory exposure mode (default: reps_only)",
    )
    parser.add_argument(
        "--allow-legacy-lessons",
        action="store_true",
        default=RUNTIME_ALLOW_LEGACY_LESSONS,
        help="allow /seed_lessons_v1 and /lessons_rewrite_v1 paths in reps_only mode",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None):
    global RUNTIME_CATALOG_MODE, RUNTIME_ALLOW_LEGACY_LESSONS
    args = parse_args(argv)
    try:
        config = assert_runtime_config_startable()
    except PrivatePilotConfigError as exc:
        codes = ",".join(exc.codes)
        raise SystemExit(f"E_PRIVATE_PILOT_CONFIG_INVALID:{codes}") from None
    host = str(args.host).strip() or "127.0.0.1"
    port = int(args.port)
    RUNTIME_CATALOG_MODE = _normalize_catalog_mode(args.catalog_mode)
    RUNTIME_ALLOW_LEGACY_LESSONS = bool(args.allow_legacy_lessons)
    if port < 0 or port > 65535:
        raise SystemExit("port must be in 0..65535")
    try:
        server = SeamgrimHTTPServer((host, port), DdnExecHandler)
    except OSError as exc:
        raise SystemExit(
            f"E_SEAMGRIM_SERVER_BIND: {host}:{port} 서버를 시작하지 못했습니다. "
            "같은 주소의 이전 서버가 실행 중인지 확인하고 그 서버 창에서 종료하세요. "
            f"({exc})"
        ) from None
    bound_host, bound_port = server.server_address[:2]
    ready_record = {
        "schema": "seamgrim.ddn_exec_server.ready.v1",
        "host": str(bound_host),
        "port": int(bound_port),
        "catalog_mode": RUNTIME_CATALOG_MODE,
        "allow_legacy_lessons": bool(RUNTIME_ALLOW_LEGACY_LESSONS),
        "deploy_profile": config.get("profile"),
        "sqlite_sync": config.get("sqlite_synchronous"),
    }
    serve_thread = threading.Thread(target=server.serve_forever, name="seamgrim-ddn-exec-server", daemon=False)
    serve_thread.start()
    try:
        print(f"SEAMGRIM_SERVER_READY {json.dumps(ready_record, ensure_ascii=False, sort_keys=True)}", flush=True)
        print(
            f"Seamgrim ddn exec server running on http://{bound_host}:{bound_port} "
            f"(catalog_mode={RUNTIME_CATALOG_MODE}, allow_legacy_lessons={str(RUNTIME_ALLOW_LEGACY_LESSONS).lower()}, "
            f"deploy_profile={config.get('profile')}, sqlite_sync={config.get('sqlite_synchronous')})",
            flush=True,
        )
        # Windows cannot reliably dispatch Ctrl+C while the main thread waits
        # indefinitely in Thread.join. Return to Python regularly so the
        # KeyboardInterrupt handler below can stop the listening server.
        while serve_thread.is_alive():
            serve_thread.join(timeout=0.25)
    except KeyboardInterrupt:
        server.shutdown()
        serve_thread.join(timeout=5)
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
