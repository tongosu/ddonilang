#!/usr/bin/env python
from __future__ import annotations

import json
import os
from datetime import datetime, timezone
from pathlib import Path


EXEC_CACHE_ENV_KEY = "DDN_SELFTEST_EXEC_CACHE_JSON"
EXEC_CACHE_SCHEMA = "ddn.ci.selftest_exec_cache.v1"
_CACHE_PATH_KEY = ""
_CACHE_MTIME_NS: int | None = None
_CACHE_OK_SCRIPTS: set[str] = set()


def _normalize_script_key(script_text: str) -> str:
    text = str(script_text).strip()
    if not text:
        return ""
    return Path(text).as_posix().lower()


def _resolve_cache_path() -> Path | None:
    path_text = os.environ.get(EXEC_CACHE_ENV_KEY, "").strip()
    if not path_text:
        return None
    return Path(path_text)


def _read_ok_scripts(path: Path) -> set[str]:
    if not path.exists():
        return set()
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return set()
    scripts = payload.get("ok_scripts")
    if not isinstance(scripts, list):
        return set()
    normalized: set[str] = set()
    for item in scripts:
        key = _normalize_script_key(str(item))
        if key:
            normalized.add(key)
    return normalized


def _write_ok_scripts(path: Path, ok_scripts: set[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": EXEC_CACHE_SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok_scripts": sorted(ok_scripts),
    }
    path.write_text(
        json.dumps(payload, ensure_ascii=False, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


def _cache_key(path: Path) -> str:
    return str(path).replace("\\", "/").lower()


def _read_mtime_ns(path: Path) -> int | None:
    try:
        return path.stat().st_mtime_ns
    except OSError:
        return None


def _refresh_cache(path: Path, *, force: bool = False) -> set[str]:
    global _CACHE_PATH_KEY, _CACHE_MTIME_NS, _CACHE_OK_SCRIPTS
    cache_key = _cache_key(path)
    mtime_ns = _read_mtime_ns(path)
    if (not force) and _CACHE_PATH_KEY == cache_key and _CACHE_MTIME_NS == mtime_ns:
        return _CACHE_OK_SCRIPTS
    _CACHE_OK_SCRIPTS = _read_ok_scripts(path)
    _CACHE_PATH_KEY = cache_key
    _CACHE_MTIME_NS = mtime_ns
    return _CACHE_OK_SCRIPTS


def _set_cache(path: Path, ok_scripts: set[str]) -> None:
    global _CACHE_PATH_KEY, _CACHE_MTIME_NS, _CACHE_OK_SCRIPTS
    _CACHE_PATH_KEY = _cache_key(path)
    _CACHE_OK_SCRIPTS = set(ok_scripts)
    _CACHE_MTIME_NS = _read_mtime_ns(path)


def reset_exec_cache() -> None:
    path = _resolve_cache_path()
    if path is None:
        return
    _write_ok_scripts(path, set())
    _set_cache(path, set())


def is_script_cached(script_text: str) -> bool:
    key = _normalize_script_key(script_text)
    if not key:
        return False
    path = _resolve_cache_path()
    if path is None:
        return False
    return key in _refresh_cache(path)


def mark_script_ok(script_text: str) -> None:
    key = _normalize_script_key(script_text)
    if not key:
        return
    path = _resolve_cache_path()
    if path is None:
        return
    ok_scripts = set(_refresh_cache(path))
    if key in ok_scripts:
        return
    ok_scripts.add(key)
    _write_ok_scripts(path, ok_scripts)
    _set_cache(path, ok_scripts)
