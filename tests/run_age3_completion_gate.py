#!/usr/bin/env python
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
import json
import os
import re
import subprocess
import sys
import time
from contextlib import contextmanager
from datetime import datetime, timezone
from pathlib import Path

PACK_TARGETS: list[tuple[str, list[str]]] = [
    ("w67_bogae_adapter_v1", ["bogae_adapter_v1_smoke"]),
    ("w68_bogae_asset_manifest_v1", ["bogae_asset_manifest_v1"]),
    ("w69_bogae_mapping_v1", ["bogae_mapping_v1"]),
    ("w70_bogae_web_viewer_v1", ["bogae_web_viewer_v1"]),
    ("w71_bogae_editor_v0", ["bogae_editor_v0", "bogae_editor_smoke_v0"]),
    ("w72_bogae_hash_determinism", ["bogae_hash_determinism_v1"]),
    ("w73_bogae_bundle_v1", ["bogae_bundle_v1"]),
    ("w74_runner_pack_integration", ["bogae_runner_bogae_hash"]),
    ("w76_bogae_cache_min", ["bogae_cache_min"]),
    ("w77_gogae7_demo_one", ["gogae7_demo_one"]),
]

DOC_PATHS = [
    "docs/ssot/age/AGE3/INDEX.md",
    "docs/ssot/age/AGE3/AGE3_DEVELOPMENT_GUIDE.md",
    "docs/ssot/walks/gogae7/w67_bogae_adapter_v1/README.md",
    "docs/ssot/walks/gogae7/w68_bogae_asset_manifest_v1/README.md",
    "docs/ssot/walks/gogae7/w69_bogae_mapping_v1/README.md",
    "docs/ssot/walks/gogae7/w70_bogae_web_viewer_v1/README.md",
    "docs/ssot/walks/gogae7/w71_bogae_editor_v0/README.md",
    "docs/ssot/walks/gogae7/w72_bogae_hash_determinism/README.md",
    "docs/ssot/walks/gogae7/w73_bogae_bundle_v1/README.md",
    "docs/ssot/walks/gogae7/w74_runner_pack_integration/README.md",
    "docs/ssot/walks/gogae7/w75_docs_ssot_walks_update/README.md",
    "docs/ssot/walks/gogae7/w76_bogae_cache_min/README.md",
    "docs/ssot/walks/gogae7/w77_gogae7_demo_one/README.md",
]

DEFAULT_PACK_GOLDEN_LOCK_TIMEOUT_SEC = 180.0

AGE3_INDEX_PATH = "docs/ssot/age/AGE3/INDEX.md"
AGE3_PACK_SHARD_COUNT_ENV_KEY = "DDN_AGE3_PACK_SHARD_COUNT"
AGE3_DYNAMIC_SHARD_MODE_ENV_KEY = "DDN_AGE3_DYNAMIC_SHARD_MODE"
AGE3_DYNAMIC_SHARD_SOURCE_REPORT_ENV_KEY = "DDN_AGE3_DYNAMIC_SHARD_SOURCE_REPORT"
AGE3_DYNAMIC_HINT_LOOKBACK_ENV_KEY = "DDN_AGE3_DYNAMIC_HINT_LOOKBACK"
AGE3_DYNAMIC_SHARD_LOOKBACK_ENV_KEY = "DDN_AGE3_DYNAMIC_SHARD_LOOKBACK"
AGE3_DYNAMIC_SHARD_MODE_OFF = "off"
AGE3_DYNAMIC_SHARD_MODE_PREVIOUS = "previous"
AGE3_PACK_SHARD_COUNT = 5
AGE3_PACK_ELAPSED_HINT_MS = {
    "bogae_hash_determinism_v1": 432,
    "bogae_adapter_v1_smoke": 269,
    "bogae_cache_min": 267,
    "bogae_web_viewer_v1": 263,
    "bogae_mapping_v1": 255,
    "gogae7_demo_one": 254,
    "bogae_runner_bogae_hash": 251,
    "bogae_editor_v0": 248,
    "bogae_asset_manifest_v1": 99,
    "bogae_bundle_v1": 98,
}
ERROR_CODE_PATTERN = re.compile(r"\b([EW]_[A-Z0-9_]{2,})\b")


def build_pack_name_index(pack_root: Path) -> tuple[list[str], set[str]]:
    if not pack_root.exists():
        return [], set()
    names = sorted(path.name for path in pack_root.iterdir() if path.is_dir())
    return names, set(names)


def clip(text: str, limit: int = 140) -> str:
    normalized = " ".join(text.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def emit_text_safely(text: str, *, stream=None) -> None:
    if not text:
        return
    target = stream if stream is not None else sys.stdout
    try:
        print(text, end="", file=target)
        return
    except UnicodeEncodeError:
        encoding = getattr(target, "encoding", None) or "utf-8"
        safe = text.encode(encoding, errors="replace").decode(encoding, errors="replace")
        print(safe, end="", file=target)


def collect_error_codes(text: str, sink: list[str] | None, limit: int = 16) -> None:
    if sink is None:
        return
    for match in ERROR_CODE_PATTERN.findall(text or ""):
        code = str(match).strip()
        if not code or code in sink:
            continue
        sink.append(code)
        if len(sink) >= limit:
            return


def default_report_path(file_name: str) -> str:
    candidates = [
        Path("I:/home/urihanl/ddn/codex/build/reports"),
        Path("C:/ddn/codex/build/reports"),
        Path("build/reports"),
    ]
    for base in candidates:
        try:
            base.mkdir(parents=True, exist_ok=True)
            return str(base / file_name)
        except OSError:
            continue
    return str(Path("build/reports") / file_name)


def derive_bogae_geoul_smoke_report_path(gate_report_out: Path) -> Path:
    stem = gate_report_out.stem if gate_report_out.suffix else gate_report_out.name
    return gate_report_out.parent / f"{stem}.bogae_geoul_visibility_smoke.detjson"


def derive_wasm_web_step_check_report_path(gate_report_out: Path) -> Path:
    stem = gate_report_out.stem if gate_report_out.suffix else gate_report_out.name
    return gate_report_out.parent / f"{stem}.seamgrim_wasm_web_step_check.detjson"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def load_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError:
        return ""


def resolve_pack_golden_lock_timeout_sec() -> float:
    raw = str(os.environ.get("DDN_PACK_GOLDEN_LOCK_TIMEOUT_SEC", "")).strip()
    if not raw:
        return DEFAULT_PACK_GOLDEN_LOCK_TIMEOUT_SEC
    try:
        value = float(raw)
    except ValueError:
        return DEFAULT_PACK_GOLDEN_LOCK_TIMEOUT_SEC
    if value <= 0.0:
        return DEFAULT_PACK_GOLDEN_LOCK_TIMEOUT_SEC
    return value


def resolve_age3_pack_shard_count() -> int:
    raw = str(os.environ.get(AGE3_PACK_SHARD_COUNT_ENV_KEY, "")).strip()
    if not raw:
        return AGE3_PACK_SHARD_COUNT
    try:
        parsed = int(raw)
    except ValueError:
        return AGE3_PACK_SHARD_COUNT
    if parsed <= 0:
        return AGE3_PACK_SHARD_COUNT
    return parsed


def resolve_age3_dynamic_shard_mode() -> str:
    raw = str(os.environ.get(AGE3_DYNAMIC_SHARD_MODE_ENV_KEY, "")).strip().lower()
    if raw in ("", "0", "off", "false", "no"):
        return AGE3_DYNAMIC_SHARD_MODE_OFF
    if raw in ("1", "on", "true", "yes", "previous"):
        return AGE3_DYNAMIC_SHARD_MODE_PREVIOUS
    return AGE3_DYNAMIC_SHARD_MODE_OFF


def resolve_age3_dynamic_shard_source_report(report_out: Path) -> Path:
    raw = str(os.environ.get(AGE3_DYNAMIC_SHARD_SOURCE_REPORT_ENV_KEY, "")).strip()
    if raw:
        return Path(raw)
    return report_out


def resolve_age3_dynamic_hint_lookback() -> int:
    raw = str(os.environ.get(AGE3_DYNAMIC_HINT_LOOKBACK_ENV_KEY, "")).strip()
    if not raw:
        return 3
    try:
        parsed = int(raw)
    except ValueError:
        return 3
    return min(max(1, parsed), 20)


def resolve_age3_dynamic_shard_lookback() -> int:
    raw = str(os.environ.get(AGE3_DYNAMIC_SHARD_LOOKBACK_ENV_KEY, "")).strip()
    if not raw:
        return 3
    try:
        parsed = int(raw)
    except ValueError:
        return 3
    return min(max(1, parsed), 20)


def clamp_shard_count(value: int, pack_count: int) -> int:
    if pack_count <= 0:
        return 1
    return min(max(1, int(value)), int(pack_count))


def pack_hint_elapsed_ms(pack: str, hint_override: dict[str, int] | None = None) -> int:
    if isinstance(hint_override, dict):
        try:
            value = int(hint_override.get(pack, 0))
        except Exception:
            value = 0
        if value > 0:
            return value
    return int(AGE3_PACK_ELAPSED_HINT_MS.get(pack, 200))


def sum_hint_elapsed_ms(packs: list[str], hint_override: dict[str, int] | None = None) -> int:
    total = 0
    for pack in packs:
        total += pack_hint_elapsed_ms(pack, hint_override)
    return total


def observed_elapsed_ms_from_doc(doc: dict | None) -> int:
    if not isinstance(doc, dict):
        return 0
    try:
        return max(0, int(doc.get("elapsed_ms", 0)))
    except Exception:
        return 0


def extract_gogae7_pack_shards(report_doc: dict | None) -> list[dict]:
    if not isinstance(report_doc, dict):
        return []
    rows = report_doc.get("gogae7_pack_shards")
    if not isinstance(rows, list):
        return []
    out: list[dict] = []
    for row in rows:
        if isinstance(row, dict):
            out.append(row)
    return out


def extract_pack_elapsed_hints(report_doc: dict | None) -> dict[str, int]:
    out: dict[str, int] = {}
    if not isinstance(report_doc, dict):
        return out
    rows = report_doc.get("packs")
    if not isinstance(rows, list):
        return out
    for row in rows:
        if not isinstance(row, dict):
            continue
        pack = row.get("pack")
        if not isinstance(pack, str) or not pack:
            continue
        try:
            elapsed_ms = int(row.get("elapsed_ms", 0))
        except Exception:
            continue
        if elapsed_ms > 0:
            out[pack] = elapsed_ms
    return out


def median_int(values: list[int]) -> int:
    if not values:
        return 0
    ordered = sorted(values)
    n = len(ordered)
    mid = n // 2
    if n % 2 == 1:
        return int(ordered[mid])
    return int(round((ordered[mid - 1] + ordered[mid]) / 2.0))


def build_recent_report_glob_pattern(reference_report_path: Path) -> str:
    name = reference_report_path.name
    if ".core_lang." in name:
        prefix = name.split(".core_lang.", 1)[0]
        return f"{prefix}.core_lang.*.detjson"
    return name


def load_recent_pack_reports(reference_report_path: Path, lookback: int) -> list[dict]:
    out: list[dict] = []
    seen: set[Path] = set()
    parent = reference_report_path.parent
    pattern = build_recent_report_glob_pattern(reference_report_path)
    candidates: list[Path] = []
    try:
        candidates = sorted(
            parent.glob(pattern),
            key=lambda p: p.stat().st_mtime,
            reverse=True,
        )
    except OSError:
        candidates = []
    for candidate in candidates:
        if len(out) >= lookback:
            break
        resolved = candidate.resolve()
        if resolved in seen:
            continue
        doc = load_json(candidate)
        if not isinstance(doc, dict):
            continue
        out.append(doc)
        seen.add(resolved)
    if len(out) < lookback and reference_report_path.exists():
        resolved_ref = reference_report_path.resolve()
        if resolved_ref not in seen:
            doc = load_json(reference_report_path)
            if isinstance(doc, dict):
                out.append(doc)
    return out


def build_recent_source_report_glob_pattern(reference_report_path: Path) -> str:
    name = reference_report_path.name
    if name.endswith(".dynamic.last.detjson"):
        prefix = name[: -len(".dynamic.last.detjson")]
        return f"{prefix}.*.detjson"
    if ".core_lang." in name:
        prefix = name.split(".core_lang.", 1)[0]
        return f"{prefix}.core_lang.*.detjson"
    if name.endswith(".detjson"):
        stem = name[: -len(".detjson")]
        parts = stem.split(".")
        if len(parts) >= 4 and parts[-1].isdigit() and parts[-2].isdigit():
            prefix = ".".join(parts[:-2])
            return f"{prefix}.*.detjson"
    return name


def load_recent_source_reports(reference_report_path: Path, lookback: int) -> list[dict]:
    def is_timestamped_gate_report(path: Path) -> bool:
        name = path.name
        if not name.endswith(".detjson"):
            return False
        stem = name[: -len(".detjson")]
        parts = stem.split(".")
        if len(parts) < 4:
            return False
        return parts[-1].isdigit() and parts[-2].isdigit()

    out: list[dict] = []
    seen: set[Path] = set()
    parent = reference_report_path.parent
    pattern = build_recent_source_report_glob_pattern(reference_report_path)
    candidates: list[Path] = []
    try:
        candidates = sorted(
            parent.glob(pattern),
            key=lambda p: p.stat().st_mtime,
            reverse=True,
        )
    except OSError:
        candidates = []
    for candidate in candidates:
        if len(out) >= lookback:
            break
        if candidate.name.endswith(".dynamic.last.detjson"):
            continue
        if not is_timestamped_gate_report(candidate):
            continue
        resolved = candidate.resolve()
        if resolved in seen:
            continue
        doc = load_json(candidate)
        if not isinstance(doc, dict):
            continue
        out.append(doc)
        seen.add(resolved)
    if len(out) < lookback and reference_report_path.exists():
        resolved_ref = reference_report_path.resolve()
        if resolved_ref not in seen:
            doc = load_json(reference_report_path)
            if isinstance(doc, dict):
                out.append(doc)
    return out


def build_smoothed_pack_elapsed_hints(report_docs: list[dict]) -> dict[str, int]:
    samples: dict[str, list[int]] = {}
    for doc in report_docs:
        hints = extract_pack_elapsed_hints(doc)
        for pack, elapsed in hints.items():
            if elapsed <= 0:
                continue
            samples.setdefault(pack, []).append(int(elapsed))
    out: dict[str, int] = {}
    for pack, rows in samples.items():
        value = median_int(rows)
        if value > 0:
            out[pack] = value
    return out


def resolve_age3_dynamic_hint_override(dynamic_source_doc: dict | None) -> tuple[dict[str, int], dict]:
    hint_override: dict[str, int] = {}
    detail = {
        "mode": "static",
        "pack_report_path": "-",
        "overrides": 0,
    }
    if not isinstance(dynamic_source_doc, dict):
        return hint_override, detail
    report_path_text = str(dynamic_source_doc.get("gogae7_pack_report_path", "")).strip()
    lookback = resolve_age3_dynamic_hint_lookback()
    report_docs = load_recent_pack_reports(Path(report_path_text), lookback) if report_path_text else []
    if report_docs:
        hint_override = build_smoothed_pack_elapsed_hints(report_docs)
    detail = {
        "mode": "previous_pack_report" if len(hint_override) > 0 else "static",
        "pack_report_path": report_path_text if report_path_text else "-",
        "overrides": len(hint_override),
        "lookback": int(lookback),
        "reports_used": len(report_docs),
        "aggregation": "median",
    }
    return hint_override, detail


def median_float(values: list[float]) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    n = len(ordered)
    mid = n // 2
    if n % 2 == 1:
        return float(ordered[mid])
    return float((ordered[mid - 1] + ordered[mid]) / 2.0)


def peak_ratio_from_shards(previous_shards: list[dict]) -> float:
    observed: list[int] = []
    for row in previous_shards:
        try:
            value = int(row.get("observed_elapsed_ms", 0))
        except Exception:
            continue
        if value > 0:
            observed.append(value)
    if not observed:
        return 0.0
    total = sum(observed)
    if total <= 0:
        return 0.0
    return max(observed) / float(total)


def suggest_shard_count_from_previous(
    previous_source_docs: list[dict],
    current: int,
    pack_count: int,
) -> tuple[int, str, int]:
    safe_current = clamp_shard_count(current, pack_count)
    ratios: list[float] = []
    for doc in previous_source_docs:
        ratio = peak_ratio_from_shards(extract_gogae7_pack_shards(doc))
        if ratio > 0.0:
            ratios.append(ratio)
    if not ratios:
        return safe_current, "no-observed-shards", 0
    peak_ratio = median_float(ratios)
    if peak_ratio >= 0.50 and safe_current < pack_count:
        return safe_current + 1, f"increase-hotspot ratio={peak_ratio:.3f}", len(ratios)
    # core_lang 실측에서 5 shard 기본값이 안정적이어서 축소 임계를 보수적으로 둔다.
    if peak_ratio <= 0.20 and safe_current > 1:
        return safe_current - 1, f"decrease-underload ratio={peak_ratio:.3f}", len(ratios)
    return safe_current, f"keep ratio={peak_ratio:.3f}", len(ratios)


def resolve_pack_golden_lock_path() -> Path:
    candidates = [
        Path("I:/home/urihanl/ddn/codex/build/tmp"),
        Path("C:/ddn/codex/build/tmp"),
        Path("build/tmp"),
    ]
    for base in candidates:
        try:
            base.mkdir(parents=True, exist_ok=True)
            return base / "run_pack_golden.lock"
        except OSError:
            continue
    fallback = Path("build/tmp")
    fallback.mkdir(parents=True, exist_ok=True)
    return fallback / "run_pack_golden.lock"


def split_packs_balanced(
    packs: list[str],
    shard_count: int,
    hint_override: dict[str, int] | None = None,
) -> list[list[str]]:
    if not packs:
        return []
    group_count = min(max(1, shard_count), len(packs))
    groups: list[list[str]] = [[] for _ in range(group_count)]
    loads = [0 for _ in range(group_count)]
    ordered = sorted(
        packs,
        key=lambda name: (-pack_hint_elapsed_ms(name, hint_override), name),
    )
    for pack in ordered:
        idx = min(range(group_count), key=lambda i: loads[i])
        groups[idx].append(pack)
        loads[idx] += pack_hint_elapsed_ms(pack, hint_override)
    # Greedy 배치 후 이동/교환 기반 로컬 리밸런싱으로 최대 shard 부하를 완화한다.
    def current_max_load() -> int:
        return max(loads) if loads else 0

    while True:
        best_kind = ""
        best_src = -1
        best_dst = -1
        best_i = -1
        best_j = -1
        best_value = current_max_load()

        for src in range(group_count):
            if len(groups[src]) <= 1:
                continue
            for i, pack in enumerate(groups[src]):
                w = pack_hint_elapsed_ms(pack, hint_override)
                for dst in range(group_count):
                    if src == dst:
                        continue
                    next_src = loads[src] - w
                    next_dst = loads[dst] + w
                    next_max = max(
                        next_src if k == src else next_dst if k == dst else loads[k]
                        for k in range(group_count)
                    )
                    if next_max < best_value:
                        best_kind = "move"
                        best_src = src
                        best_dst = dst
                        best_i = i
                        best_j = -1
                        best_value = next_max

        for left in range(group_count):
            if not groups[left]:
                continue
            for right in range(left + 1, group_count):
                if not groups[right]:
                    continue
                for i, a in enumerate(groups[left]):
                    wa = pack_hint_elapsed_ms(a, hint_override)
                    for j, b in enumerate(groups[right]):
                        wb = pack_hint_elapsed_ms(b, hint_override)
                        next_left = loads[left] - wa + wb
                        next_right = loads[right] - wb + wa
                        next_max = max(
                            next_left if k == left else next_right if k == right else loads[k]
                            for k in range(group_count)
                        )
                        if next_max < best_value:
                            best_kind = "swap"
                            best_src = left
                            best_dst = right
                            best_i = i
                            best_j = j
                            best_value = next_max

        if not best_kind:
            break

        if best_kind == "move":
            pack = groups[best_src].pop(best_i)
            w = pack_hint_elapsed_ms(pack, hint_override)
            groups[best_dst].append(pack)
            loads[best_src] -= w
            loads[best_dst] += w
            continue

        left_pack = groups[best_src][best_i]
        right_pack = groups[best_dst][best_j]
        wl = pack_hint_elapsed_ms(left_pack, hint_override)
        wr = pack_hint_elapsed_ms(right_pack, hint_override)
        groups[best_src][best_i] = right_pack
        groups[best_dst][best_j] = left_pack
        loads[best_src] = loads[best_src] - wl + wr
        loads[best_dst] = loads[best_dst] - wr + wl

    for idx in range(group_count):
        groups[idx] = sorted(groups[idx], key=lambda name: packs.index(name))
    return [group for group in groups if group]


def shard_report_path(report_out: Path, shard_index: int) -> Path:
    suffix = report_out.suffix if report_out.suffix else ".detjson"
    stem = report_out.stem if report_out.suffix else report_out.name
    return report_out.parent / f"{stem}.gogae7.s{shard_index}{suffix}"


def combine_pack_reports(source_docs: list[dict | None]) -> dict:
    now_text = datetime.now(timezone.utc).isoformat()
    mode = "check"
    updated_pack_files = 0
    merged_rows: list[dict] = []
    seen_packs: set[str] = set()
    for doc in source_docs:
        if not isinstance(doc, dict):
            continue
        source_mode = str(doc.get("mode", "")).strip()
        if source_mode:
            mode = source_mode
        try:
            updated_pack_files += int(doc.get("updated_pack_files", 0))
        except Exception:
            pass
        rows = doc.get("packs")
        if not isinstance(rows, list):
            continue
        for row in rows:
            if not isinstance(row, dict):
                continue
            pack = row.get("pack")
            if not isinstance(pack, str) or not pack.strip():
                continue
            if pack in seen_packs:
                continue
            seen_packs.add(pack)
            merged_rows.append(row)
    failure_count = sum(1 for row in merged_rows if not bool(row.get("ok", False)))
    elapsed_ms = 0
    for row in merged_rows:
        try:
            elapsed_ms += int(row.get("elapsed_ms", 0))
        except Exception:
            continue
    overall_ok = len(merged_rows) > 0 and failure_count == 0
    return {
        "schema": "ddn.pack.golden.report.v1",
        "generated_at_utc": now_text,
        "mode": mode,
        "overall_ok": overall_ok,
        "updated_pack_files": updated_pack_files,
        "failure_count": failure_count,
        "elapsed_ms": elapsed_ms,
        "packs": merged_rows,
        "missing_packs": [],
    }


def build_subset_pack_report(source_doc: dict | None, target_packs: list[str]) -> dict:
    now_text = datetime.now(timezone.utc).isoformat()
    source_rows = report_rows(source_doc)
    subset_rows: list[dict] = []
    missing_packs: list[str] = []
    for pack in target_packs:
        row = source_rows.get(pack)
        if row is None:
            missing_packs.append(pack)
            continue
        subset_rows.append(row)

    failed_count = len(missing_packs)
    for row in subset_rows:
        if not bool(row.get("ok", False)):
            failed_count += 1
    elapsed_ms = 0
    for row in subset_rows:
        try:
            elapsed_ms += int(row.get("elapsed_ms", 0))
        except Exception:
            continue
    mode = "check"
    updated_pack_files = 0
    if isinstance(source_doc, dict):
        source_mode = str(source_doc.get("mode", "")).strip()
        if source_mode:
            mode = source_mode
        try:
            updated_pack_files = int(source_doc.get("updated_pack_files", 0))
        except Exception:
            updated_pack_files = 0
    return {
        "schema": "ddn.pack.golden.report.v1",
        "generated_at_utc": now_text,
        "mode": mode,
        "overall_ok": failed_count == 0 and len(subset_rows) == len(target_packs),
        "updated_pack_files": updated_pack_files,
        "failure_count": failed_count,
        "elapsed_ms": elapsed_ms,
        "packs": subset_rows,
        "missing_packs": missing_packs,
    }


@contextmanager
def pack_golden_file_lock(timeout_sec: float = 180.0):
    lock_path = resolve_pack_golden_lock_path()
    lock_path.parent.mkdir(parents=True, exist_ok=True)
    handle = lock_path.open("a+b")
    locked = False
    deadline = time.monotonic() + timeout_sec
    try:
        while True:
            try:
                if os.name == "nt":
                    import msvcrt

                    handle.seek(0)
                    msvcrt.locking(handle.fileno(), msvcrt.LK_NBLCK, 1)
                else:
                    import fcntl

                    fcntl.flock(handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                locked = True
                break
            except OSError:
                if time.monotonic() >= deadline:
                    raise
                time.sleep(0.05)
        yield
    finally:
        if locked:
            try:
                if os.name == "nt":
                    import msvcrt

                    handle.seek(0)
                    msvcrt.locking(handle.fileno(), msvcrt.LK_UNLCK, 1)
                else:
                    import fcntl

                    fcntl.flock(handle.fileno(), fcntl.LOCK_UN)
            except OSError:
                pass
        handle.close()


def run_cmd(root: Path, cmd: list[str], label: str) -> tuple[int, str, str]:
    print(f"[age3-completion] run[{label}]: {' '.join(cmd)}")
    try:
        proc = subprocess.Popen(
            cmd,
            cwd=root,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except OSError as exc:
        return 1, "", f"[age3-completion] run[{label}] spawn failed: {exc}\n"
    first_rc = int(proc.wait() or 0)
    if first_rc == 0:
        return 0, "", ""
    rerun = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    rerun_rc = int(rerun.returncode or 0)
    return rerun_rc, str(rerun.stdout or ""), str(rerun.stderr or "")


def py_cmd(script_path: str, *extra_args: str) -> list[str]:
    return [sys.executable, "-S", script_path, *extra_args]


def run_pack_golden_job(
    root: Path,
    resolved_packs: list[str],
    pack_report_out: Path,
    shard_count: int,
    hint_override: dict[str, int] | None = None,
) -> tuple[int, str, str, list[dict]]:
    groups = split_packs_balanced(resolved_packs, shard_count, hint_override)
    jobs: list[dict] = []
    for idx, packs in enumerate(groups, 1):
        jobs.append(
            {
                "shard_index": idx,
                "packs": packs,
                "hint_elapsed_ms": sum_hint_elapsed_ms(packs, hint_override),
                "report_out": shard_report_path(pack_report_out, idx),
            }
        )
    stdout_parts: list[str] = []
    stderr_parts: list[str] = []
    shard_docs: list[dict | None] = []
    shard_doc_by_path: dict[str, dict | None] = {}
    rc = 0
    for idx, job in enumerate(jobs, 1):
        cmd_preview = py_cmd(
            "tests/run_pack_golden.py",
            *list(job["packs"]),
            "--report-out",
            str(job["report_out"]),
            "--report-summary-only",
        )
        print(f"[age3-completion] run[gogae7_packs#{idx}]: {' '.join(cmd_preview)}")
    try:
        with pack_golden_file_lock(resolve_pack_golden_lock_timeout_sec()):
            for job in jobs:
                cmd = py_cmd(
                    "tests/run_pack_golden.py",
                    *list(job["packs"]),
                    "--report-out",
                    str(job["report_out"]),
                    "--report-summary-only",
                )
                job["cmd"] = cmd
                job["started_at"] = time.perf_counter()
                job["proc"] = subprocess.Popen(
                    cmd,
                    cwd=root,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                )
            for job in jobs:
                proc = job["proc"]
                proc.wait()
                finished_at = time.perf_counter()
                step_rc = int(proc.returncode or 0)
                job["returncode"] = step_rc
                started_at = float(job.get("started_at", finished_at))
                job["observed_elapsed_ms"] = max(0, int(round((finished_at - started_at) * 1000.0)))
                job["observed_elapsed_source"] = "wallclock"
                rc = max(rc, step_rc)
                if step_rc != 0:
                    rerun = subprocess.run(
                        list(job.get("cmd", [])),
                        cwd=root,
                        capture_output=True,
                        text=True,
                        encoding="utf-8",
                        errors="replace",
                    )
                    rerun_rc = int(rerun.returncode or 0)
                    job["stdout"] = str(rerun.stdout or "")
                    job["stderr"] = str(rerun.stderr or "")
                    step_rc = rerun_rc
                    job["returncode"] = step_rc
                else:
                    job["stdout"] = ""
                    job["stderr"] = ""
            for job in jobs:
                step_rc = int(job.get("returncode", 0))
                rc = max(rc, step_rc)
                if step_rc != 0:
                    stdout_parts.append(str(job.get("stdout", "")))
                    stderr_parts.append(str(job.get("stderr", "")))
                shard_doc = load_json(Path(str(job["report_out"])))
                shard_docs.append(shard_doc)
                shard_doc_by_path[str(job["report_out"])] = shard_doc
    except OSError as exc:
        return 1, "", f"[age3-completion] pack-golden lock acquire failed: {exc}\n", []
    source_doc = combine_pack_reports(shard_docs)
    merged_doc = build_subset_pack_report(source_doc, resolved_packs)
    pack_report_out.parent.mkdir(parents=True, exist_ok=True)
    pack_report_out.write_text(json.dumps(merged_doc, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    shard_rows: list[dict] = []
    for job in jobs:
        shard_doc = shard_doc_by_path.get(str(job.get("report_out", "")))
        observed_elapsed_ms = int(job.get("observed_elapsed_ms", 0))
        observed_elapsed_source = str(job.get("observed_elapsed_source", "")).strip()
        if observed_elapsed_ms <= 0:
            observed_elapsed_ms = observed_elapsed_ms_from_doc(shard_doc)
            observed_elapsed_source = "report_elapsed_fallback"
        elif not observed_elapsed_source:
            observed_elapsed_source = "wallclock"
        shard_rows.append(
            {
                "shard_index": int(job.get("shard_index", 0)),
                "packs": list(job.get("packs", [])),
                "pack_count": len(list(job.get("packs", []))),
                "hint_elapsed_ms": int(job.get("hint_elapsed_ms", 0)),
                "observed_elapsed_ms": max(0, observed_elapsed_ms),
                "observed_elapsed_source": observed_elapsed_source,
                "returncode": int(job.get("returncode", 0)),
                "report_loaded": bool(isinstance(shard_doc, dict)),
                "report_path": str(job.get("report_out", "")),
            }
        )
    shard_rows.sort(key=lambda row: int(row.get("shard_index", 0)))
    return rc, "".join(stdout_parts), "".join(stderr_parts), shard_rows


def resolve_pack_name(root: Path, candidates: list[str]) -> str | None:
    pack_root = root / "pack"
    for name in candidates:
        path = pack_root / name / "golden.jsonl"
        if path.exists():
            return name
    return None


def report_rows(doc: dict | None) -> dict[str, dict]:
    out: dict[str, dict] = {}
    if not isinstance(doc, dict):
        return out
    rows = doc.get("packs")
    if not isinstance(rows, list):
        return out
    for row in rows:
        if not isinstance(row, dict):
            continue
        pack = row.get("pack")
        if isinstance(pack, str):
            out[pack] = row
    return out


def evaluate_pack_set(target_packs: list[str], rows: dict[str, dict]) -> tuple[bool, list[str], list[dict]]:
    missing: list[str] = []
    failed_rows: list[dict] = []
    for pack in target_packs:
        row = rows.get(pack)
        if row is None:
            missing.append(pack)
            continue
        if not bool(row.get("ok", False)):
            failed_rows.append(row)
    ok = len(missing) == 0 and len(failed_rows) == 0
    return ok, missing, failed_rows


def pick_case_detail(case: dict) -> str:
    for key in ("issues", "stderr", "expected", "got"):
        value = case.get(key)
        if not isinstance(value, list):
            continue
        for item in value:
            text = str(item).strip()
            if text:
                return text
    return ""


def build_pack_failure_digest(
    missing: list[str],
    failed_rows: list[dict],
    limit: int = 8,
    code_sink: list[str] | None = None,
) -> list[str]:
    digest: list[str] = []
    for pack in missing:
        digest.append(f"pack: missing row={pack}")
        if len(digest) >= limit:
            return digest
    for row in failed_rows:
        pack = str(row.get("pack", "-"))
        failed_count = int(row.get("failed_case_count", 0))
        summary = f"pack: pack={pack} failed_case_count={failed_count}"
        cases = row.get("cases")
        if isinstance(cases, list):
            for case in cases:
                if not isinstance(case, dict):
                    continue
                if bool(case.get("ok", True)):
                    continue
                index = case.get("index")
                if isinstance(index, int) and index > 0:
                    summary += f" first_case={index}"
                detail = pick_case_detail(case)
                if detail:
                    collect_error_codes(detail, code_sink)
                    summary += f" detail={clip(detail)}"
                break
        digest.append(summary)
        if len(digest) >= limit:
            return digest
    return digest


def append_unique(target: list[str], value: str) -> None:
    if value and value not in target:
        target.append(value)


def parse_age3_index_walk_ids(index_text: str) -> list[str]:
    walk_ids: list[str] = []
    for line in index_text.splitlines():
        match = re.search(r"`\.\./(w\d+_[A-Za-z0-9_]+)/README\.md`", line)
        if match is None:
            continue
        append_unique(walk_ids, match.group(1))
    return walk_ids


def parse_readme_pack_patterns(readme_text: str) -> list[str]:
    patterns: list[str] = []
    for token in re.findall(r"pack/([A-Za-z0-9_.-]+(?:\*)?)", readme_text):
        append_unique(patterns, token)
    return patterns


def expand_pack_pattern(
    pack_names: list[str],
    pack_name_set: set[str],
    pattern: str,
) -> list[str]:
    if "*" not in pattern:
        return [pattern] if pattern in pack_name_set else []
    prefix = pattern.split("*", 1)[0]
    return [name for name in pack_names if name.startswith(prefix)]


def resolve_age3_ssot_pack_contract(
    root: Path,
    gate_candidates_by_walk: dict[str, list[str]],
) -> tuple[list[str], dict[str, list[str]], list[str], list[str], list[str]]:
    missing_docs: list[str] = []
    unresolved_patterns: list[str] = []
    unresolved_walks: list[str] = []
    expected_packs_by_walk: dict[str, list[str]] = {}
    expected_walk_ids: list[str] = []

    index_path = root / AGE3_INDEX_PATH
    if not index_path.exists():
        missing_docs.append(AGE3_INDEX_PATH)
        return expected_walk_ids, expected_packs_by_walk, missing_docs, unresolved_patterns, unresolved_walks

    expected_walk_ids = parse_age3_index_walk_ids(load_text(index_path))
    pack_root = root / "pack"
    pack_names, pack_name_set = build_pack_name_index(pack_root)
    walk_root = root / "docs" / "ssot" / "walks" / "gogae7"
    for walk_id in expected_walk_ids:
        readme_path = walk_root / walk_id / "README.md"
        if not readme_path.exists():
            missing_docs.append(str(readme_path.relative_to(root)))
            continue
        patterns = parse_readme_pack_patterns(load_text(readme_path))
        if not patterns:
            continue
        expected_packs: list[str] = []
        for pattern in patterns:
            expanded = expand_pack_pattern(pack_names, pack_name_set, pattern)
            if not expanded:
                fallback_candidates = gate_candidates_by_walk.get(walk_id, [])
                fallback_hits: list[str] = []
                for fallback in fallback_candidates:
                    if (pack_root / fallback).exists():
                        append_unique(fallback_hits, fallback)
                if fallback_hits:
                    for pack_name in fallback_hits:
                        append_unique(expected_packs, pack_name)
                    continue
                unresolved_patterns.append(f"{walk_id}:{pattern}")
                continue
            for pack_name in expanded:
                append_unique(expected_packs, pack_name)
        if not expected_packs:
            unresolved_walks.append(walk_id)
            continue
        expected_packs_by_walk[walk_id] = expected_packs

    return expected_walk_ids, expected_packs_by_walk, missing_docs, unresolved_patterns, unresolved_walks


def main() -> int:
    parser = argparse.ArgumentParser(description="AGE3 completion gate (gogae7 + web/wasm smoke)")
    parser.add_argument(
        "--report-out",
        default=default_report_path("age3_completion_gate.detjson"),
        help="output JSON report path",
    )
    parser.add_argument(
        "--pack-report-out",
        default=default_report_path("age3_completion_pack_report.detjson"),
        help="ddn.pack.golden.report.v1 output path for gogae7 packs",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    report_out = Path(args.report_out)
    pack_report_out = Path(args.pack_report_out)
    bogae_geoul_smoke_report_out = derive_bogae_geoul_smoke_report_path(report_out)
    seamgrim_wasm_web_step_check_report_out = derive_wasm_web_step_check_report_path(report_out)

    gate_walk_keys = [key for key, _ in PACK_TARGETS]
    gate_candidates_by_walk = {key: list(candidates) for key, candidates in PACK_TARGETS}
    gate_candidate_packs: list[str] = []
    for _, candidates in PACK_TARGETS:
        for pack_name in candidates:
            append_unique(gate_candidate_packs, pack_name)

    ssot_walk_ids, ssot_expected_packs_by_walk, ssot_missing_docs, ssot_unresolved_patterns, ssot_unresolved_walks = (
        resolve_age3_ssot_pack_contract(root, gate_candidates_by_walk)
    )
    ssot_required_walks = list(ssot_expected_packs_by_walk.keys())
    ssot_required_packs: list[str] = []
    for packs in ssot_expected_packs_by_walk.values():
        for pack_name in packs:
            append_unique(ssot_required_packs, pack_name)
    ssot_missing_walks_from_gate = [walk for walk in ssot_required_walks if walk not in gate_walk_keys]
    ssot_missing_packs_from_gate_candidates = [
        pack for pack in ssot_required_packs if pack not in gate_candidate_packs
    ]
    ssot_sync_ok = (
        len(ssot_walk_ids) > 0
        and len(ssot_missing_docs) == 0
        and len(ssot_unresolved_patterns) == 0
        and len(ssot_unresolved_walks) == 0
        and len(ssot_missing_walks_from_gate) == 0
        and len(ssot_missing_packs_from_gate_candidates) == 0
    )

    resolved_pack_map: dict[str, str] = {}
    unresolved_targets: list[str] = []
    for key, candidates in PACK_TARGETS:
        resolved = resolve_pack_name(root, candidates)
        if resolved is None:
            unresolved_targets.append(f"{key}: {','.join(candidates)}")
            continue
        resolved_pack_map[key] = resolved

    resolved_packs = [resolved_pack_map[key] for key, _ in PACK_TARGETS if key in resolved_pack_map]
    requested_pack_shard_count = resolve_age3_pack_shard_count()
    dynamic_shard_mode = resolve_age3_dynamic_shard_mode()
    dynamic_source_report_path = resolve_age3_dynamic_shard_source_report(report_out)
    dynamic_shard_lookback = resolve_age3_dynamic_shard_lookback()
    dynamic_source_docs: list[dict] = []
    dynamic_source_doc: dict | None = None
    effective_pack_shard_count = clamp_shard_count(requested_pack_shard_count, len(resolved_packs))
    dynamic_shard_decisions: list[str] = []
    dynamic_source_reports_used = 0
    dynamic_hint_override: dict[str, int] = {}
    dynamic_hint_policy = {
        "mode": "static",
        "pack_report_path": "-",
        "overrides": 0,
    }
    if len(resolved_packs) > 0 and dynamic_shard_mode == AGE3_DYNAMIC_SHARD_MODE_PREVIOUS:
        dynamic_source_docs = load_recent_source_reports(dynamic_source_report_path, dynamic_shard_lookback)
        if dynamic_source_docs:
            dynamic_source_doc = dynamic_source_docs[0]
        suggested_count, reason, used_reports = suggest_shard_count_from_previous(
            dynamic_source_docs,
            effective_pack_shard_count,
            len(resolved_packs),
        )
        effective_pack_shard_count = clamp_shard_count(suggested_count, len(resolved_packs))
        dynamic_shard_decisions.append(reason)
        dynamic_source_reports_used = int(used_reports)
        dynamic_hint_override, dynamic_hint_policy = resolve_age3_dynamic_hint_override(dynamic_source_doc)
    else:
        dynamic_source_doc = load_json(dynamic_source_report_path)
    pack_rc = 1
    pack_stdout = ""
    pack_stderr = ""
    pack_doc: dict | None = None
    pack_rows: dict[str, dict] = {}
    pack_ok = False
    pack_missing: list[str] = []
    pack_failed_rows: list[dict] = []
    gogae7_pack_shards: list[dict] = []
    parallel_jobs: list[tuple[str, list[str]]] = [
        (
            "bogae_backend_profile_smoke",
            py_cmd("tests/run_bogae_backend_profile_smoke_check.py"),
        ),
        (
            "lang_teulcli_parser_parity_selftest",
            py_cmd("tests/run_lang_teulcli_parser_parity_selftest.py"),
        ),
        (
            "diag_fixit_selftest",
            py_cmd("tests/run_diag_fixit_selftest.py"),
        ),
        (
            "lang_maegim_smoke_pack",
            py_cmd("tests/run_pack_golden.py", "lang_maegim_smoke_v1"),
        ),
        (
            "lang_unit_temp_smoke_pack",
            py_cmd("tests/run_pack_golden.py", "lang_unit_temp_smoke_v1"),
        ),
        (
            "gate0_contract_abort_state_check",
            py_cmd("tests/run_gate0_contract_abort_state_check.py"),
        ),
        (
            "block_editor_roundtrip_check",
            py_cmd("tests/run_block_editor_roundtrip_check.py"),
        ),
        (
            "seamgrim_wasm_canon_contract_check",
            py_cmd(
                "tests/run_seamgrim_wasm_smoke.py",
                "seamgrim_wasm_canon_contract_v1",
                "--skip-ui-common",
                "--skip-ui-pendulum",
                "--skip-wrapper",
                "--skip-vm-runtime",
                "--skip-space2d-source-gate",
            ),
        ),
        (
            "proof_runtime_minimum_check",
            py_cmd("tests/run_proof_runtime_minimum_check.py"),
        ),
        (
            "seamgrim_wasm_web_smoke_contract",
            py_cmd("tests/run_seamgrim_wasm_web_smoke_contract_pack_check.py"),
        ),
        (
            "seamgrim_wasm_web_step_check",
            py_cmd(
                "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py",
                "--report-out",
                str(seamgrim_wasm_web_step_check_report_out),
            ),
        ),
        (
            "bogae_geoul_visibility_smoke",
            py_cmd(
                "tests/run_bogae_geoul_visibility_smoke_check.py",
                "--report-out",
                str(bogae_geoul_smoke_report_out),
            ),
        ),
        (
            "external_intent_boundary_pack",
            py_cmd("tests/run_external_intent_boundary_pack_check.py"),
        ),
        (
            "seulgi_v1_pack",
            py_cmd("tests/run_seulgi_v1_pack_check.py"),
        ),
        (
            "sam_inputsnapshot_contract_pack",
            py_cmd("tests/run_sam_inputsnapshot_contract_pack_check.py"),
        ),
        (
            "sam_ai_ordering_pack",
            py_cmd("tests/run_sam_ai_ordering_pack_check.py"),
        ),
        (
            "seulgi_gatekeeper_pack",
            py_cmd("tests/run_seulgi_gatekeeper_pack_check.py"),
        ),
        (
            "external_intent_seulgi_walk_alignment",
            py_cmd("tests/run_external_intent_seulgi_walk_alignment_check.py"),
        ),
    ]
    parallel_results: dict[str, object] = {}
    with ThreadPoolExecutor(max_workers=(4 if len(unresolved_targets) == 0 else 3)) as executor:
        futures: dict[str, object] = {}
        if len(unresolved_targets) == 0:
            futures["gogae7_packs"] = executor.submit(
                run_pack_golden_job,
                root,
                resolved_packs,
                pack_report_out,
                effective_pack_shard_count,
                dynamic_hint_override,
            )
        for label, cmd in parallel_jobs:
            futures[label] = executor.submit(run_cmd, root, cmd, label)
        for label in futures:
            parallel_results[label] = futures[label].result()

    if len(unresolved_targets) == 0:
        pack_rc, pack_stdout, pack_stderr, gogae7_pack_shards = parallel_results["gogae7_packs"]
        if pack_stdout:
            emit_text_safely(pack_stdout)
        if pack_stderr:
            emit_text_safely(pack_stderr, stream=sys.stderr)
        pack_doc = load_json(pack_report_out)
        pack_rows = report_rows(pack_doc)
        pack_ok, pack_missing, pack_failed_rows = evaluate_pack_set(resolved_packs, pack_rows)
        if not gogae7_pack_shards:
            gogae7_pack_shards = [
                {
                    "shard_index": 1,
                    "packs": list(resolved_packs),
                    "pack_count": len(resolved_packs),
                    "hint_elapsed_ms": sum_hint_elapsed_ms(resolved_packs, dynamic_hint_override),
                    "observed_elapsed_ms": observed_elapsed_ms_from_doc(pack_doc),
                    "observed_elapsed_source": "report_elapsed_fallback",
                    "returncode": int(pack_rc),
                    "report_loaded": bool(isinstance(pack_doc, dict)),
                    "report_path": str(pack_report_out),
                }
            ]

    backend_smoke_rc, backend_smoke_stdout, backend_smoke_stderr = parallel_results[
        "bogae_backend_profile_smoke"
    ]
    if backend_smoke_stdout:
        emit_text_safely(backend_smoke_stdout)
    if backend_smoke_stderr:
        emit_text_safely(backend_smoke_stderr, stream=sys.stderr)

    parser_parity_rc, parser_parity_stdout, parser_parity_stderr = parallel_results[
        "lang_teulcli_parser_parity_selftest"
    ]
    if parser_parity_stdout:
        emit_text_safely(parser_parity_stdout)
    if parser_parity_stderr:
        emit_text_safely(parser_parity_stderr, stream=sys.stderr)

    diag_fixit_rc, diag_fixit_stdout, diag_fixit_stderr = parallel_results[
        "diag_fixit_selftest"
    ]
    if diag_fixit_stdout:
        emit_text_safely(diag_fixit_stdout)
    if diag_fixit_stderr:
        emit_text_safely(diag_fixit_stderr, stream=sys.stderr)

    maegim_smoke_rc, maegim_smoke_stdout, maegim_smoke_stderr = parallel_results[
        "lang_maegim_smoke_pack"
    ]
    if maegim_smoke_stdout:
        emit_text_safely(maegim_smoke_stdout)
    if maegim_smoke_stderr:
        emit_text_safely(maegim_smoke_stderr, stream=sys.stderr)

    unit_temp_smoke_rc, unit_temp_smoke_stdout, unit_temp_smoke_stderr = parallel_results[
        "lang_unit_temp_smoke_pack"
    ]
    if unit_temp_smoke_stdout:
        emit_text_safely(unit_temp_smoke_stdout)
    if unit_temp_smoke_stderr:
        emit_text_safely(unit_temp_smoke_stderr, stream=sys.stderr)

    contract_abort_state_check_rc, contract_abort_state_check_stdout, contract_abort_state_check_stderr = parallel_results[
        "gate0_contract_abort_state_check"
    ]
    if contract_abort_state_check_stdout:
        emit_text_safely(contract_abort_state_check_stdout)
    if contract_abort_state_check_stderr:
        emit_text_safely(contract_abort_state_check_stderr, stream=sys.stderr)

    block_editor_roundtrip_rc, block_editor_roundtrip_stdout, block_editor_roundtrip_stderr = parallel_results[
        "block_editor_roundtrip_check"
    ]
    if block_editor_roundtrip_stdout:
        emit_text_safely(block_editor_roundtrip_stdout)
    if block_editor_roundtrip_stderr:
        emit_text_safely(block_editor_roundtrip_stderr, stream=sys.stderr)

    wasm_canon_contract_rc, wasm_canon_contract_stdout, wasm_canon_contract_stderr = parallel_results[
        "seamgrim_wasm_canon_contract_check"
    ]
    if wasm_canon_contract_stdout:
        emit_text_safely(wasm_canon_contract_stdout)
    if wasm_canon_contract_stderr:
        emit_text_safely(wasm_canon_contract_stderr, stream=sys.stderr)

    proof_runtime_minimum_rc, proof_runtime_minimum_stdout, proof_runtime_minimum_stderr = parallel_results[
        "proof_runtime_minimum_check"
    ]
    if proof_runtime_minimum_stdout:
        emit_text_safely(proof_runtime_minimum_stdout)
    if proof_runtime_minimum_stderr:
        emit_text_safely(proof_runtime_minimum_stderr, stream=sys.stderr)

    wasm_web_rc, wasm_web_stdout, wasm_web_stderr = parallel_results[
        "seamgrim_wasm_web_smoke_contract"
    ]
    if wasm_web_stdout:
        emit_text_safely(wasm_web_stdout)
    if wasm_web_stderr:
        emit_text_safely(wasm_web_stderr, stream=sys.stderr)

    wasm_web_step_check_rc, wasm_web_step_check_stdout, wasm_web_step_check_stderr = parallel_results[
        "seamgrim_wasm_web_step_check"
    ]
    if wasm_web_step_check_stdout:
        emit_text_safely(wasm_web_step_check_stdout)
    if wasm_web_step_check_stderr:
        emit_text_safely(wasm_web_step_check_stderr, stream=sys.stderr)

    bogae_geoul_smoke_rc, bogae_geoul_smoke_stdout, bogae_geoul_smoke_stderr = parallel_results[
        "bogae_geoul_visibility_smoke"
    ]
    if bogae_geoul_smoke_stdout:
        emit_text_safely(bogae_geoul_smoke_stdout)
    if bogae_geoul_smoke_stderr:
        emit_text_safely(bogae_geoul_smoke_stderr, stream=sys.stderr)

    external_intent_boundary_pack_rc, external_intent_boundary_pack_stdout, external_intent_boundary_pack_stderr = (
        parallel_results["external_intent_boundary_pack"]
    )
    if external_intent_boundary_pack_stdout:
        emit_text_safely(external_intent_boundary_pack_stdout)
    if external_intent_boundary_pack_stderr:
        emit_text_safely(external_intent_boundary_pack_stderr, stream=sys.stderr)

    seulgi_v1_pack_rc, seulgi_v1_pack_stdout, seulgi_v1_pack_stderr = parallel_results["seulgi_v1_pack"]
    if seulgi_v1_pack_stdout:
        emit_text_safely(seulgi_v1_pack_stdout)
    if seulgi_v1_pack_stderr:
        emit_text_safely(seulgi_v1_pack_stderr, stream=sys.stderr)

    sam_inputsnapshot_contract_pack_rc, sam_inputsnapshot_contract_pack_stdout, sam_inputsnapshot_contract_pack_stderr = parallel_results[
        "sam_inputsnapshot_contract_pack"
    ]
    if sam_inputsnapshot_contract_pack_stdout:
        emit_text_safely(sam_inputsnapshot_contract_pack_stdout)
    if sam_inputsnapshot_contract_pack_stderr:
        emit_text_safely(sam_inputsnapshot_contract_pack_stderr, stream=sys.stderr)

    sam_ai_ordering_pack_rc, sam_ai_ordering_pack_stdout, sam_ai_ordering_pack_stderr = parallel_results[
        "sam_ai_ordering_pack"
    ]
    if sam_ai_ordering_pack_stdout:
        emit_text_safely(sam_ai_ordering_pack_stdout)
    if sam_ai_ordering_pack_stderr:
        emit_text_safely(sam_ai_ordering_pack_stderr, stream=sys.stderr)

    seulgi_gatekeeper_pack_rc, seulgi_gatekeeper_pack_stdout, seulgi_gatekeeper_pack_stderr = parallel_results[
        "seulgi_gatekeeper_pack"
    ]
    if seulgi_gatekeeper_pack_stdout:
        emit_text_safely(seulgi_gatekeeper_pack_stdout)
    if seulgi_gatekeeper_pack_stderr:
        emit_text_safely(seulgi_gatekeeper_pack_stderr, stream=sys.stderr)

    external_intent_seulgi_walk_alignment_rc, external_intent_seulgi_walk_alignment_stdout, external_intent_seulgi_walk_alignment_stderr = parallel_results[
        "external_intent_seulgi_walk_alignment"
    ]
    if external_intent_seulgi_walk_alignment_stdout:
        emit_text_safely(external_intent_seulgi_walk_alignment_stdout)
    if external_intent_seulgi_walk_alignment_stderr:
        emit_text_safely(external_intent_seulgi_walk_alignment_stderr, stream=sys.stderr)

    missing_docs: list[str] = []
    for path_text in DOC_PATHS:
        if not (root / path_text).exists():
            missing_docs.append(path_text)

    criteria = [
        {
            "name": "age3_ssot_walk_pack_contract_sync",
            "ok": ssot_sync_ok,
            "detail": (
                f"ssot_walks={len(ssot_walk_ids)} required_walks={len(ssot_required_walks)} required_packs={len(ssot_required_packs)} "
                f"missing_docs={len(ssot_missing_docs)} unresolved_patterns={len(ssot_unresolved_patterns)} unresolved_walks={len(ssot_unresolved_walks)} "
                f"missing_walks_from_gate={len(ssot_missing_walks_from_gate)} "
                f"missing_packs_from_gate_candidates={len(ssot_missing_packs_from_gate_candidates)}"
            ),
        },
        {
            "name": "gogae7_pack_set_pass",
            "ok": len(unresolved_targets) == 0 and pack_rc == 0 and pack_ok,
            "detail": (
                f"packs={len(resolved_packs)} unresolved={len(unresolved_targets)} "
                f"missing={len(pack_missing)} failed={len(pack_failed_rows)} exit={pack_rc}"
            ),
        },
        {
            "name": "bogae_backend_profile_smoke_pass",
            "ok": backend_smoke_rc == 0,
            "detail": f"exit={backend_smoke_rc}",
        },
        {
            "name": "lang_teulcli_parser_parity_selftest_pass",
            "ok": parser_parity_rc == 0,
            "detail": f"exit={parser_parity_rc}",
        },
        {
            "name": "diag_fixit_selftest_pass",
            "ok": diag_fixit_rc == 0,
            "detail": f"exit={diag_fixit_rc}",
        },
        {
            "name": "lang_maegim_smoke_pack_pass",
            "ok": maegim_smoke_rc == 0,
            "detail": f"exit={maegim_smoke_rc}",
        },
        {
            "name": "lang_unit_temp_smoke_pack_pass",
            "ok": unit_temp_smoke_rc == 0,
            "detail": f"exit={unit_temp_smoke_rc}",
        },
        {
            "name": "gate0_contract_abort_state_check_pass",
            "ok": contract_abort_state_check_rc == 0,
            "detail": f"exit={contract_abort_state_check_rc}",
        },
        {
            "name": "block_editor_roundtrip_check_pass",
            "ok": block_editor_roundtrip_rc == 0,
            "detail": f"exit={block_editor_roundtrip_rc}",
        },
        {
            "name": "seamgrim_wasm_canon_contract_check_pass",
            "ok": wasm_canon_contract_rc == 0,
            "detail": f"exit={wasm_canon_contract_rc}",
        },
        {
            "name": "proof_runtime_minimum_check_pass",
            "ok": proof_runtime_minimum_rc == 0,
            "detail": f"exit={proof_runtime_minimum_rc}",
        },
        {
            "name": "seamgrim_wasm_web_smoke_contract_pass",
            "ok": wasm_web_rc == 0,
            "detail": f"exit={wasm_web_rc}",
        },
        {
            "name": "seamgrim_wasm_web_step_check_pass",
            "ok": wasm_web_step_check_rc == 0,
            "detail": f"exit={wasm_web_step_check_rc}",
        },
        {
            "name": "bogae_geoul_visibility_smoke_pass",
            "ok": bogae_geoul_smoke_rc == 0,
            "detail": f"exit={bogae_geoul_smoke_rc}",
        },
        {
            "name": "external_intent_boundary_pack_pass",
            "ok": external_intent_boundary_pack_rc == 0,
            "detail": f"exit={external_intent_boundary_pack_rc}",
        },
        {
            "name": "seulgi_v1_pack_pass",
            "ok": seulgi_v1_pack_rc == 0,
            "detail": f"exit={seulgi_v1_pack_rc}",
        },
        {
            "name": "sam_inputsnapshot_contract_pack_pass",
            "ok": sam_inputsnapshot_contract_pack_rc == 0,
            "detail": f"exit={sam_inputsnapshot_contract_pack_rc}",
        },
        {
            "name": "sam_ai_ordering_pack_pass",
            "ok": sam_ai_ordering_pack_rc == 0,
            "detail": f"exit={sam_ai_ordering_pack_rc}",
        },
        {
            "name": "seulgi_gatekeeper_pack_pass",
            "ok": seulgi_gatekeeper_pack_rc == 0,
            "detail": f"exit={seulgi_gatekeeper_pack_rc}",
        },
        {
            "name": "external_intent_seulgi_walk_alignment_pass",
            "ok": external_intent_seulgi_walk_alignment_rc == 0,
            "detail": f"exit={external_intent_seulgi_walk_alignment_rc}",
        },
        {
            "name": "age3_doc_paths_exist",
            "ok": len(missing_docs) == 0,
            "detail": f"required={len(DOC_PATHS)} missing={len(missing_docs)}",
        },
    ]

    overall_ok = all(bool(row.get("ok", False)) for row in criteria)
    failure_digest: list[str] = []
    failure_codes: list[str] = []
    if not bool(criteria[0]["ok"]):
        if ssot_missing_docs:
            failure_digest.extend([f"ssot-doc-missing: {item}" for item in ssot_missing_docs[:8]])
        if ssot_unresolved_patterns:
            failure_digest.extend([f"ssot-pattern-unresolved: {clip(item)}" for item in ssot_unresolved_patterns[:8]])
        if ssot_unresolved_walks:
            failure_digest.extend([f"ssot-walk-unresolved: {item}" for item in ssot_unresolved_walks[:8]])
        if ssot_missing_walks_from_gate:
            failure_digest.extend([f"ssot-walk-missing-from-gate: {item}" for item in ssot_missing_walks_from_gate[:8]])
        if ssot_missing_packs_from_gate_candidates:
            failure_digest.extend(
                [f"ssot-pack-missing-from-gate-candidates: {item}" for item in ssot_missing_packs_from_gate_candidates[:8]]
            )
    if len(unresolved_targets) > 0:
        failure_digest.extend([f"pack-resolve: {clip(item)}" for item in unresolved_targets[:8]])
    if not bool(criteria[1]["ok"]):
        failure_digest.extend(build_pack_failure_digest(pack_missing, pack_failed_rows, code_sink=failure_codes))
        if pack_rc != 0:
            failure_digest.append(f"pack: run_pack_golden exit={pack_rc}")
    if backend_smoke_rc != 0:
        text = backend_smoke_stderr.strip() or backend_smoke_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"backend_smoke: {clip(text)}")
    if parser_parity_rc != 0:
        text = parser_parity_stderr.strip() or parser_parity_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"lang_teulcli_parser_parity_selftest: {clip(text)}")
    if diag_fixit_rc != 0:
        text = diag_fixit_stderr.strip() or diag_fixit_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"diag_fixit_selftest: {clip(text)}")
    if maegim_smoke_rc != 0:
        text = maegim_smoke_stderr.strip() or maegim_smoke_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"lang_maegim_smoke_pack: {clip(text)}")
    if unit_temp_smoke_rc != 0:
        text = unit_temp_smoke_stderr.strip() or unit_temp_smoke_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"lang_unit_temp_smoke_pack: {clip(text)}")
    if contract_abort_state_check_rc != 0:
        text = contract_abort_state_check_stderr.strip() or contract_abort_state_check_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"gate0_contract_abort_state_check: {clip(text)}")
    if block_editor_roundtrip_rc != 0:
        text = block_editor_roundtrip_stderr.strip() or block_editor_roundtrip_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"block_editor_roundtrip_check: {clip(text)}")
    if wasm_canon_contract_rc != 0:
        text = wasm_canon_contract_stderr.strip() or wasm_canon_contract_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"seamgrim_wasm_canon_contract_check: {clip(text)}")
    if proof_runtime_minimum_rc != 0:
        text = proof_runtime_minimum_stderr.strip() or proof_runtime_minimum_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"proof_runtime_minimum_check: {clip(text)}")
    if wasm_web_rc != 0:
        text = wasm_web_stderr.strip() or wasm_web_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"wasm_web_smoke_contract: {clip(text)}")
    if wasm_web_step_check_rc != 0:
        text = wasm_web_step_check_stderr.strip() or wasm_web_step_check_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"wasm_web_step_check: {clip(text)}")
    if bogae_geoul_smoke_rc != 0:
        text = bogae_geoul_smoke_stderr.strip() or bogae_geoul_smoke_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"bogae_geoul_visibility_smoke: {clip(text)}")
    if external_intent_boundary_pack_rc != 0:
        text = external_intent_boundary_pack_stderr.strip() or external_intent_boundary_pack_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"external_intent_boundary_pack: {clip(text)}")
    if seulgi_v1_pack_rc != 0:
        text = seulgi_v1_pack_stderr.strip() or seulgi_v1_pack_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"seulgi_v1_pack: {clip(text)}")
    if sam_inputsnapshot_contract_pack_rc != 0:
        text = sam_inputsnapshot_contract_pack_stderr.strip() or sam_inputsnapshot_contract_pack_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"sam_inputsnapshot_contract_pack: {clip(text)}")
    if sam_ai_ordering_pack_rc != 0:
        text = sam_ai_ordering_pack_stderr.strip() or sam_ai_ordering_pack_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"sam_ai_ordering_pack: {clip(text)}")
    if seulgi_gatekeeper_pack_rc != 0:
        text = seulgi_gatekeeper_pack_stderr.strip() or seulgi_gatekeeper_pack_stdout.strip() or "-"
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"seulgi_gatekeeper_pack: {clip(text)}")
    if external_intent_seulgi_walk_alignment_rc != 0:
        text = (
            external_intent_seulgi_walk_alignment_stderr.strip()
            or external_intent_seulgi_walk_alignment_stdout.strip()
            or "-"
        )
        collect_error_codes(text, failure_codes)
        failure_digest.append(f"external_intent_seulgi_walk_alignment: {clip(text)}")
    if missing_docs:
        failure_digest.extend([f"doc-missing: {path}" for path in missing_docs[:8]])

    report = {
        "schema": "ddn.age3.completion_gate.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "overall_ok": overall_ok,
        "shard_policy": {
            "mode": dynamic_shard_mode,
            "source_report_path": str(dynamic_source_report_path),
            "source_report_loaded": bool(len(dynamic_source_docs) > 0) or bool(isinstance(dynamic_source_doc, dict)),
            "lookback": int(dynamic_shard_lookback),
            "source_reports_used": int(dynamic_source_reports_used),
            "requested": {
                "gogae7_packs": int(requested_pack_shard_count),
            },
            "effective": {
                "gogae7_packs": int(effective_pack_shard_count),
            },
            "decisions": list(dynamic_shard_decisions),
            "hint_policy": dynamic_hint_policy,
        },
        "criteria": criteria,
        "ssot_walk_pack_contract_sync": {
            "index_path": AGE3_INDEX_PATH,
            "ssot_walk_ids": ssot_walk_ids,
            "required_walks": ssot_required_walks,
            "required_packs": ssot_required_packs,
            "missing_docs": ssot_missing_docs,
            "unresolved_patterns": ssot_unresolved_patterns,
            "unresolved_walks": ssot_unresolved_walks,
            "missing_walks_from_gate": ssot_missing_walks_from_gate,
            "missing_packs_from_gate_candidates": ssot_missing_packs_from_gate_candidates,
            "gate_walk_keys": gate_walk_keys,
            "gate_pack_candidates": gate_candidate_packs,
        },
        "resolved_pack_map": resolved_pack_map,
        "unresolved_pack_targets": unresolved_targets,
        "gogae7_pack_shards": gogae7_pack_shards if len(unresolved_targets) == 0 else [],
        "gogae7_pack_report_path": str(pack_report_out),
        "bogae_geoul_visibility_smoke_report_path": str(bogae_geoul_smoke_report_out),
        "seamgrim_wasm_web_step_check_report_path": str(seamgrim_wasm_web_step_check_report_out),
        "failure_digest": failure_digest[:24],
        "failure_codes": failure_codes[:16],
    }
    report_out.parent.mkdir(parents=True, exist_ok=True)
    report_out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    failed_count = sum(1 for row in criteria if not bool(row.get("ok", False)))
    print(
        f"[age3-completion] overall_ok={int(overall_ok)} criteria={len(criteria)} "
        f"failed={failed_count} report={report_out}"
    )
    print(
        f"[age3-completion] shard_policy mode={dynamic_shard_mode} "
        f"gogae7_packs={effective_pack_shard_count}"
    )
    for row in criteria:
        print(f" - {row['name']}: ok={int(bool(row.get('ok', False)))}")
    if not overall_ok:
        for line in failure_digest[:10]:
            print(f"   {line}")
        if failure_codes:
            print(f"   failure_codes={','.join(failure_codes[:8])}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
