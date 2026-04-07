#!/usr/bin/env python
import argparse
from collections import OrderedDict
import json
import os
import re
import subprocess
import sys
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlparse

# Reuse export_graph helpers
TOOLS_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS_DIR))

from export_graph import (
    run_teul_cli,
    parse_points,
    build_graph,
    hash_text,
    extract_meta,
    extract_series_labels,
    normalize_ddn_for_hash,
    preprocess_ddn_for_teul,
    _parse_numbers_from_line,
    normalize_series_label,
    _resolve_teul_cli_bin,
)

ROOT = Path(__file__).resolve().parents[3]
UI_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui"
LESSON_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
SEED_LESSON_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "seed_lessons_v1"
REWRITE_LESSON_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons_rewrite_v1"
SCHEMA_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "schema"
SAMPLES_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "samples"
PROJECT_HTTP_PREFIX = "/solutions/seamgrim_ui_mvp"
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


def resolve_build_dir() -> Path:
    primary = Path("I:/home/urihanl/ddn/codex/build")
    fallback = Path("C:/ddn/codex/build")
    target = primary if primary.exists() else fallback
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
    )
    normalized_text = json.dumps(payload, ensure_ascii=False, sort_keys=True)
    with _MAEGIM_CONTROL_CACHE_LOCK:
        _MAEGIM_CONTROL_CACHE[cache_key] = (lesson_mtime_ns, lesson_size, normalized_text, source)
        _MAEGIM_CONTROL_CACHE.move_to_end(cache_key, last=True)
        while len(_MAEGIM_CONTROL_CACHE) > int(_MAEGIM_CONTROL_CACHE_MAX_ENTRIES):
            _MAEGIM_CONTROL_CACHE.popitem(last=False)
    return payload, source


def _build_maegim_control_payload_from_source_text(source_text: str, source_label: str = "input.ddn") -> tuple[dict, str]:
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
                # canon 경로가 비어 있거나(controls 없음), legacy 주석 경고를 놓친 경우에는
                # 실행 서버 계약을 위해 legacy 계획으로 강등한다.
                legacy_preview = _build_legacy_maegim_control_plan(source_text, source_label)
                legacy_warnings = legacy_preview.get("warnings")
                if not isinstance(controls, list) or not controls:
                    return legacy_preview, "legacy"
                if (
                    isinstance(legacy_warnings, list)
                    and legacy_warnings
                    and (not isinstance(warnings, list) or not warnings)
                ):
                    return legacy_preview, "legacy"
                return payload, "canon"
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
        "message": "`// 범위(...)`는 deprecated입니다. `매김 {}`으로 옮기세요.",
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


def build_lesson_inventory_payload() -> dict:
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

    lessons = sorted(
        [_apply_inventory_meta_fallback(row) for row in rows_by_id.values()],
        key=lambda item: (str(item.get("title", "")).lower(), str(item.get("id", "")).lower()),
    )
    return {
        "ok": True,
        "schema": "seamgrim.lesson_inventory.v2",
        "count": len(lessons),
        "lessons": lessons,
    }


def json_response(handler: BaseHTTPRequestHandler, status: int, payload: dict):
    body = json.dumps(payload, ensure_ascii=False, sort_keys=True).encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json; charset=utf-8")
    handler.send_header("Access-Control-Allow-Origin", "*")
    handler.send_header("Access-Control-Allow-Headers", "Content-Type")
    handler.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
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
    content_type = "text/plain"
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
    elif suffix == ".csv":
        content_type = "text/csv; charset=utf-8"
    elif suffix == ".toml":
        content_type = "text/plain; charset=utf-8"
    handler.send_response(200)
    handler.send_header("Cache-Control", "no-store, max-age=0")
    handler.send_header("Content-Type", content_type)
    handler.send_header("Access-Control-Allow-Origin", "*")
    handler.send_header("Content-Length", str(len(content)))
    handler.end_headers()
    handler.wfile.write(content)


def send_json_text(handler: BaseHTTPRequestHandler, text: str):
    body = str(text or "").encode("utf-8")
    handler.send_response(200)
    handler.send_header("Cache-Control", "no-store, max-age=0")
    handler.send_header("Content-Type", "application/json; charset=utf-8")
    handler.send_header("Access-Control-Allow-Origin", "*")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


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
        json_response(handler, 500, {"ok": False, "error": str(exc), "schema": MAEGIM_CONTROL_SCHEMA})
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
    def do_OPTIONS(self):
        self.send_response(204)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.end_headers()

    def do_GET(self):
        parsed = urlparse(self.path)
        raw_path = parsed.path
        if raw_path == "/health" or raw_path == "/api/health":
            json_response(self, 200, {"ok": True, "service": "ddn_exec_server"})
            return
        if raw_path in ("/api/lessons/inventory", "/api/lesson-inventory"):
            try:
                json_response(self, 200, build_lesson_inventory_payload())
            except Exception as exc:
                json_response(self, 500, {"ok": False, "error": str(exc)})
            return
        if raw_path == "/favicon.ico":
            self.send_response(204)
            self.end_headers()
            return

        path = raw_path
        if path.startswith(PROJECT_HTTP_PREFIX):
            stripped = path[len(PROJECT_HTTP_PREFIX):]
            path = stripped if stripped else "/"

        if path in ("/", "/index.html", "/ui", "/ui/"):
            send_file(self, UI_DIR / "index.html")
            return
        if path.startswith("/lessons/"):
            if try_send_generated_maegim_control(self, LESSON_DIR, path[len("/lessons/"):]):
                return
            lesson_path = (LESSON_DIR / path[len("/lessons/"):]).resolve()
            if LESSON_DIR in lesson_path.parents or lesson_path == LESSON_DIR:
                send_file(self, lesson_path)
                return
        if path.startswith("/seed_lessons_v1/"):
            if try_send_generated_maegim_control(self, SEED_LESSON_DIR, path[len("/seed_lessons_v1/"):]):
                return
            seed_path = (SEED_LESSON_DIR / path[len("/seed_lessons_v1/"):]).resolve()
            if SEED_LESSON_DIR in seed_path.parents or seed_path == SEED_LESSON_DIR:
                send_file(self, seed_path)
                return
        if path.startswith("/lessons_rewrite_v1/"):
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
        parsed = urlparse(self.path)
        if parsed.path != "/api/run":
            json_response(self, 404, {"ok": False, "error": "not found"})
            return
        temp_path: Path | None = None
        try:
            length = int(self.headers.get("Content-Length", "0"))
            body = self.rfile.read(length).decode("utf-8-sig")
            payload = json.loads(body)
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
                json_response(self, 400, {"ok": False, "error": "ddn_text required"})
                return
            ddn_text = _strip_utf8_bom_prefix(ddn_text)
            if not ddn_text.strip():
                json_response(self, 400, {"ok": False, "error": "ddn_text required"})
                return

            build_dir = resolve_build_dir()
            preprocessed = preprocess_ddn_for_teul(ddn_text, strip_draw=True)
            with tempfile.NamedTemporaryFile(
                mode="w",
                encoding="utf-8",
                suffix=".ddn",
                prefix="seamgrim_ui_mvp_input_",
                dir=build_dir,
                delete=False,
            ) as temp_file:
                temp_file.write(preprocessed)
                temp_path = Path(temp_file.name)
            maegim_control_payload, maegim_control_source = _build_maegim_control_payload_from_source_text(
                ddn_text,
                source_label="<api-run>",
            )
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
            lines = run_teul_cli(ROOT, temp_path, madi=madi)
            points = parse_points(lines, series_labels)
            space2d_points = parse_space2d_points(lines)
            space2d_shapes = parse_space2d_shapes(lines)
            text_block = parse_text_blocks(lines)
            table_data = parse_table_blocks(lines)
            structure_data = parse_structure_blocks(lines)

            if not label:
                label = meta.get("name") or meta.get("desc") or "f(x)"

            graph = build_graph(points, label, source_hash)
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
            payload["runtime"] = {
                "schema": "seamgrim.runtime_summary.v1",
                "maegim_control_count": len(maegim_control_names),
                "maegim_control_names": maegim_control_names,
                "maegim_control_source": maegim_control_source,
                "maegim_control_warning_count": len(maegim_warning_codes),
                "maegim_control_warning_codes": maegim_warning_codes,
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
            json_response(self, 200, payload)
        except Exception as exc:
            json_response(self, 500, {"ok": False, "error": str(exc)})
        finally:
            if temp_path is not None:
                try:
                    temp_path.unlink(missing_ok=True)
                except Exception:
                    pass


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
    return parser.parse_args(argv)


def main(argv: list[str] | None = None):
    args = parse_args(argv)
    host = str(args.host).strip() or "127.0.0.1"
    port = int(args.port)
    if port < 1 or port > 65535:
        raise SystemExit("port must be in 1..65535")
    server = ThreadingHTTPServer((host, port), DdnExecHandler)
    print(f"Seamgrim ddn exec server running on http://{host}:{port}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
