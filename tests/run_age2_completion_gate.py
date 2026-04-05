#!/usr/bin/env python
from __future__ import annotations

import argparse
import io
import importlib
import json
import os
import re
import subprocess
import sys
import time
from contextlib import contextmanager
from contextlib import redirect_stderr, redirect_stdout
from datetime import datetime, timezone
from pathlib import Path

MUST_PACKS = [
    "open_clock_record_replay",
    "open_file_read_record_replay",
    "open_rand_record_replay",
    "open_diag_minimal_v1",
    "open_decl_policy",
    "open_policy_allowlist",
    "open_policy_conflict",
    "open_deny_policy",
    "open_replay_missing",
    "open_replay_invalid",
    "open_replay_hash_mismatch",
]

SHOULD_PACKS = [
    "open_net_record_replay",
    "open_ffi_record_replay",
    "open_gpu_record_replay",
    "open_replay_schema_mismatch",
    "open_replay_mismatch_diag",
    "open_bundle_artifact",
    "open_end_to_end",
]
ALL_PACKS = MUST_PACKS + [pack for pack in SHOULD_PACKS if pack not in MUST_PACKS]
AGE2_MUST_SHARD_COUNT_ENV_KEY = "DDN_AGE2_MUST_SHARD_COUNT"
AGE2_SHOULD_SHARD_COUNT_ENV_KEY = "DDN_AGE2_SHOULD_SHARD_COUNT"
AGE2_DYNAMIC_SHARD_MODE_ENV_KEY = "DDN_AGE2_DYNAMIC_SHARD_MODE"
AGE2_DYNAMIC_SHARD_SOURCE_REPORT_ENV_KEY = "DDN_AGE2_DYNAMIC_SHARD_SOURCE_REPORT"
AGE2_DYNAMIC_HINT_LOOKBACK_ENV_KEY = "DDN_AGE2_DYNAMIC_HINT_LOOKBACK"
AGE2_DYNAMIC_SHARD_LOOKBACK_ENV_KEY = "DDN_AGE2_DYNAMIC_SHARD_LOOKBACK"
AGE2_DYNAMIC_SHARD_MODE_OFF = "off"
AGE2_DYNAMIC_SHARD_MODE_PREVIOUS = "previous"
AGE2_MUST_SHARD_COUNT = 4
AGE2_SHOULD_SHARD_COUNT = 4
AGE2_PACK_ELAPSED_HINT_MS = {
    # 2026-03-26 재측정(각 pack 단독 3회 평균) 기준 hint.
    "open_end_to_end": 597,
    "open_replay_mismatch_diag": 473,
    "open_deny_policy": 421,
    "open_diag_minimal_v1": 356,
    "open_replay_invalid": 270,
    "open_bundle_artifact": 263,
    "open_gpu_record_replay": 256,
    "open_rand_record_replay": 254,
    "open_ffi_record_replay": 253,
    "open_replay_schema_mismatch": 252,
    "open_policy_allowlist": 252,
    "open_net_record_replay": 249,
    "open_decl_policy": 249,
    "open_replay_missing": 248,
    "open_file_read_record_replay": 248,
    "open_replay_hash_mismatch": 246,
    "open_clock_record_replay": 241,
    "open_policy_conflict": 96,
}

DEFAULT_PACK_GOLDEN_LOCK_TIMEOUT_SEC = 180.0

AGE2_SSOT_PLAN_PATH = "docs/ssot/age/AGE2/PLAN.md"
AGE2_SSOT_README_PATH = "docs/ssot/age/AGE2/README.md"
_RUN_PACK_GOLDEN_MODULE = None
ERROR_CODE_PATTERN = re.compile(r"\b([EW]_[A-Z0-9_]{2,})\b")


def clip(text: str, limit: int = 140) -> str:
    normalized = " ".join(text.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


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


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


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


def resolve_age2_must_shard_count() -> int:
    raw = str(os.environ.get(AGE2_MUST_SHARD_COUNT_ENV_KEY, "")).strip()
    if not raw:
        return AGE2_MUST_SHARD_COUNT
    try:
        parsed = int(raw)
    except ValueError:
        return AGE2_MUST_SHARD_COUNT
    if parsed <= 0:
        return AGE2_MUST_SHARD_COUNT
    return parsed


def resolve_age2_should_shard_count() -> int:
    raw = str(os.environ.get(AGE2_SHOULD_SHARD_COUNT_ENV_KEY, "")).strip()
    if not raw:
        return AGE2_SHOULD_SHARD_COUNT
    try:
        parsed = int(raw)
    except ValueError:
        return AGE2_SHOULD_SHARD_COUNT
    if parsed <= 0:
        return AGE2_SHOULD_SHARD_COUNT
    return parsed


def resolve_age2_dynamic_shard_mode() -> str:
    raw = str(os.environ.get(AGE2_DYNAMIC_SHARD_MODE_ENV_KEY, "")).strip().lower()
    if raw in ("", "0", "off", "false", "no"):
        return AGE2_DYNAMIC_SHARD_MODE_OFF
    if raw in ("1", "on", "true", "yes", "previous"):
        return AGE2_DYNAMIC_SHARD_MODE_PREVIOUS
    return AGE2_DYNAMIC_SHARD_MODE_OFF


def resolve_age2_dynamic_shard_source_report(report_out: Path) -> Path:
    raw = str(os.environ.get(AGE2_DYNAMIC_SHARD_SOURCE_REPORT_ENV_KEY, "")).strip()
    if raw:
        return Path(raw)
    return report_out


def resolve_age2_dynamic_hint_lookback() -> int:
    raw = str(os.environ.get(AGE2_DYNAMIC_HINT_LOOKBACK_ENV_KEY, "")).strip()
    if not raw:
        return 3
    try:
        parsed = int(raw)
    except ValueError:
        return 3
    return min(max(1, parsed), 20)


def resolve_age2_dynamic_shard_lookback() -> int:
    raw = str(os.environ.get(AGE2_DYNAMIC_SHARD_LOOKBACK_ENV_KEY, "")).strip()
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
    return int(AGE2_PACK_ELAPSED_HINT_MS.get(pack, 200))


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


def extract_kind_shards(report_doc: dict | None, kind: str) -> list[dict]:
    if not isinstance(report_doc, dict):
        return []
    key = f"{kind}_shards"
    rows = report_doc.get(key)
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


def resolve_age2_dynamic_hint_overrides(dynamic_source_doc: dict | None) -> tuple[dict[str, int], dict[str, int], dict]:
    must_overrides: dict[str, int] = {}
    should_overrides: dict[str, int] = {}
    detail = {
        "mode": "static",
        "must_report_path": "-",
        "should_report_path": "-",
        "must_overrides": 0,
        "should_overrides": 0,
    }
    if not isinstance(dynamic_source_doc, dict):
        return must_overrides, should_overrides, detail
    must_report_path_text = str(dynamic_source_doc.get("must_report_path", "")).strip()
    should_report_path_text = str(dynamic_source_doc.get("should_report_path", "")).strip()
    lookback = resolve_age2_dynamic_hint_lookback()
    must_docs = load_recent_pack_reports(Path(must_report_path_text), lookback) if must_report_path_text else []
    should_docs = load_recent_pack_reports(Path(should_report_path_text), lookback) if should_report_path_text else []
    if must_docs:
        must_overrides = build_smoothed_pack_elapsed_hints(must_docs)
    if should_docs:
        should_overrides = build_smoothed_pack_elapsed_hints(should_docs)
    detail = {
        "mode": "previous_pack_report",
        "must_report_path": must_report_path_text if must_report_path_text else "-",
        "should_report_path": should_report_path_text if should_report_path_text else "-",
        "must_overrides": len(must_overrides),
        "should_overrides": len(should_overrides),
        "lookback": int(lookback),
        "must_reports_used": len(must_docs),
        "should_reports_used": len(should_docs),
        "aggregation": "median",
    }
    if len(must_overrides) == 0 and len(should_overrides) == 0:
        detail["mode"] = "static"
    return must_overrides, should_overrides, detail


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
    *,
    kind: str,
    current: int,
    pack_count: int,
) -> tuple[int, str, int]:
    safe_current = clamp_shard_count(current, pack_count)
    ratios: list[float] = []
    for doc in previous_source_docs:
        ratio = peak_ratio_from_shards(extract_kind_shards(doc, kind))
        if ratio > 0.0:
            ratios.append(ratio)
    if not ratios:
        return safe_current, "no-observed-shards", 0
    peak_ratio = median_float(ratios)
    if peak_ratio >= 0.50 and safe_current < pack_count:
        return safe_current + 1, f"increase-hotspot ratio={peak_ratio:.3f}", len(ratios)
    # core_lang 실측에서 4/4 기본 shard 안정성이 높아 과도한 축소를 피한다.
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
    # Greedy 배치 후 이동/교환 기반 로컬 리밸런싱으로 최대 shard 부하를 추가 완화한다.
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


def shard_report_path(report_out: Path, kind: str, shard_index: int) -> Path:
    suffix = report_out.suffix if report_out.suffix else ".detjson"
    stem = report_out.stem if report_out.suffix else report_out.name
    return report_out.parent / f"{stem}.{kind}.s{shard_index}{suffix}"


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


def run_pack_batch(root: Path, packs: list[str], report_out: Path) -> tuple[int, str, str]:
    cmd = [
        sys.executable,
        "tests/run_pack_golden.py",
        *packs,
        "--report-out",
        str(report_out),
        "--report-summary-only",
    ]
    print(f"[age2-completion] run: {' '.join(cmd)}")
    try:
        with pack_golden_file_lock(resolve_pack_golden_lock_timeout_sec()):
            return run_pack_batch_inprocess(root, packs, report_out)
    except OSError as exc:
        stderr = f"[age2-completion] pack-golden lock acquire failed: {exc}\n"
        return 1, "", stderr


def run_pack_batch_inprocess(root: Path, packs: list[str], report_out: Path) -> tuple[int, str, str]:
    global _RUN_PACK_GOLDEN_MODULE
    if _RUN_PACK_GOLDEN_MODULE is None:
        _RUN_PACK_GOLDEN_MODULE = importlib.import_module("run_pack_golden")

    argv = ["tests/run_pack_golden.py", *packs, "--report-out", str(report_out), "--report-summary-only"]
    stdout_buf = io.StringIO()
    stderr_buf = io.StringIO()
    old_argv = sys.argv
    old_cwd = Path.cwd()
    returncode = 0
    try:
        sys.argv = argv
        os.chdir(root)
        with redirect_stdout(stdout_buf), redirect_stderr(stderr_buf):
            try:
                code = _RUN_PACK_GOLDEN_MODULE.main()
                if code is None:
                    returncode = 0
                elif isinstance(code, int):
                    returncode = code
                else:
                    returncode = 1
                    stderr_buf.write(str(code))
            except SystemExit as exc:
                code = exc.code
                if code is None:
                    returncode = 0
                elif isinstance(code, int):
                    returncode = code
                else:
                    returncode = 1
                    stderr_buf.write(str(code))
            except Exception as exc:  # pragma: no cover - defensive fallback
                returncode = 1
                stderr_buf.write(f"{type(exc).__name__}: {exc}")
    finally:
        sys.argv = old_argv
        os.chdir(old_cwd)

    stdout_text = stdout_buf.getvalue()
    stderr_text = stderr_buf.getvalue()
    if returncode == 0:
        return returncode, "", ""
    return returncode, stdout_text, stderr_text


def run_pack_batches_parallel(
    root: Path,
    *,
    must_packs: list[str],
    should_packs: list[str],
    must_shard_count: int,
    should_shard_count: int,
    must_report_out: Path,
    should_report_out: Path,
    must_hint_override: dict[str, int] | None = None,
    should_hint_override: dict[str, int] | None = None,
) -> tuple[tuple[int, str, str], tuple[int, str, str], list[dict], list[dict]]:
    must_groups = split_packs_balanced(must_packs, must_shard_count, must_hint_override)
    should_groups = split_packs_balanced(should_packs, should_shard_count, should_hint_override)
    jobs: list[dict] = []
    for idx, packs in enumerate(must_groups, 1):
        jobs.append(
            {
                "kind": "must",
                "shard_index": idx,
                "packs": packs,
                "hint_elapsed_ms": sum_hint_elapsed_ms(packs, must_hint_override),
                "report_out": shard_report_path(must_report_out, "must", idx),
            }
        )
    for idx, packs in enumerate(should_groups, 1):
        jobs.append(
            {
                "kind": "should",
                "shard_index": idx,
                "packs": packs,
                "hint_elapsed_ms": sum_hint_elapsed_ms(packs, should_hint_override),
                "report_out": shard_report_path(should_report_out, "should", idx),
            }
        )
    for idx, job in enumerate(jobs, 1):
        cmd = [
            sys.executable,
            "-S",
            "tests/run_pack_golden.py",
            *list(job["packs"]),
            "--report-out",
            str(job["report_out"]),
            "--report-summary-only",
        ]
        print(f"[age2-completion] run[{job['kind']}#{idx}]: {' '.join(cmd)}")
    try:
        with pack_golden_file_lock(resolve_pack_golden_lock_timeout_sec()):
            for job in jobs:
                cmd = [
                    sys.executable,
                    "-S",
                    "tests/run_pack_golden.py",
                    *list(job["packs"]),
                    "--report-out",
                    str(job["report_out"]),
                    "--report-summary-only",
                ]
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
                if step_rc == 0:
                    job["stdout"] = ""
                    job["stderr"] = ""
                else:
                    rerun = subprocess.run(
                        list(job.get("cmd", [])),
                        cwd=root,
                        capture_output=True,
                        text=True,
                        encoding="utf-8",
                        errors="replace",
                    )
                    job["stdout"] = str(rerun.stdout or "")
                    job["stderr"] = str(rerun.stderr or "")
    except OSError as exc:
        err = f"[age2-completion] pack-golden lock acquire failed: {exc}\n"
        return (1, "", err), (1, "", err), [], []
    must_docs: list[dict | None] = []
    should_docs: list[dict | None] = []
    must_stdout_parts: list[str] = []
    must_stderr_parts: list[str] = []
    should_stdout_parts: list[str] = []
    should_stderr_parts: list[str] = []
    must_rc = 0
    should_rc = 0
    for job in jobs:
        kind = str(job["kind"])
        doc = load_json(Path(str(job["report_out"])))
        job["report_loaded"] = bool(isinstance(doc, dict))
        observed_elapsed_ms = int(job.get("observed_elapsed_ms", 0))
        observed_elapsed_source = str(job.get("observed_elapsed_source", "")).strip()
        if observed_elapsed_ms <= 0:
            observed_elapsed_ms = observed_elapsed_ms_from_doc(doc)
            observed_elapsed_source = "report_elapsed_fallback"
        elif not observed_elapsed_source:
            observed_elapsed_source = "wallclock"
        job["observed_elapsed_ms"] = max(0, observed_elapsed_ms)
        job["observed_elapsed_source"] = observed_elapsed_source
        if kind == "must":
            must_docs.append(doc)
            must_stdout_parts.append(str(job.get("stdout", "")))
            must_stderr_parts.append(str(job.get("stderr", "")))
            must_rc = max(must_rc, int(job.get("returncode", 0)))
        else:
            should_docs.append(doc)
            should_stdout_parts.append(str(job.get("stdout", "")))
            should_stderr_parts.append(str(job.get("stderr", "")))
            should_rc = max(should_rc, int(job.get("returncode", 0)))

    must_source = combine_pack_reports(must_docs)
    should_source = combine_pack_reports(should_docs)
    write_json(must_report_out, build_subset_pack_report(must_source, must_packs))
    write_json(should_report_out, build_subset_pack_report(should_source, should_packs))
    must_shards: list[dict] = []
    should_shards: list[dict] = []
    for job in jobs:
        shard_row = {
            "shard_index": int(job.get("shard_index", 0)),
            "packs": list(job.get("packs", [])),
            "pack_count": len(list(job.get("packs", []))),
            "hint_elapsed_ms": int(job.get("hint_elapsed_ms", 0)),
            "observed_elapsed_ms": int(job.get("observed_elapsed_ms", 0)),
            "observed_elapsed_source": str(job.get("observed_elapsed_source", "unknown")),
            "returncode": int(job.get("returncode", 0)),
            "report_loaded": bool(job.get("report_loaded", False)),
            "report_path": str(job.get("report_out", "")),
        }
        if str(job.get("kind", "")) == "must":
            must_shards.append(shard_row)
        else:
            should_shards.append(shard_row)
    must_shards.sort(key=lambda row: int(row.get("shard_index", 0)))
    should_shards.sort(key=lambda row: int(row.get("shard_index", 0)))
    return (
        (must_rc, "".join(must_stdout_parts), "".join(must_stderr_parts)),
        (should_rc, "".join(should_stdout_parts), "".join(should_stderr_parts)),
        must_shards,
        should_shards,
    )


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

    failed_count = 0
    for row in subset_rows:
        if not bool(row.get("ok", False)):
            failed_count += 1
    failed_count += len(missing_packs)
    overall_ok = failed_count == 0 and len(subset_rows) == len(target_packs)
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
        "overall_ok": overall_ok,
        "updated_pack_files": updated_pack_files,
        "failure_count": failed_count,
        "elapsed_ms": elapsed_ms,
        "packs": subset_rows,
        "missing_packs": missing_packs,
    }


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


def build_failure_digest(
    prefix: str,
    missing: list[str],
    failed_rows: list[dict],
    limit: int = 8,
    code_sink: list[str] | None = None,
) -> list[str]:
    digest: list[str] = []
    for pack in missing:
        digest.append(f"{prefix}: missing pack row={pack}")
        if len(digest) >= limit:
            return digest
    for row in failed_rows:
        pack = str(row.get("pack", "-"))
        failed_count = int(row.get("failed_case_count", 0))
        summary = f"{prefix}: pack={pack} failed_case_count={failed_count}"
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


def extract_pack_names_from_line(line: str) -> list[str]:
    packs: list[str] = []
    for token in re.findall(r"pack/[A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.-]+)*", line):
        segments = [seg for seg in token.split("/") if seg and seg != "pack"]
        if not segments:
            continue
        append_unique(packs, segments[-1])
    return packs


def parse_age2_ssot_pack_sections(text: str) -> tuple[list[str], list[str]]:
    must: list[str] = []
    should: list[str] = []
    section = ""
    for raw in text.splitlines():
        line = raw.strip()
        if not line:
            continue
        lowered = line.lower()
        if lowered.startswith("must:") or "필수 팩" in line:
            section = "must"
            continue
        if lowered.startswith("should:") or "권장(확장) 팩" in line:
            section = "should"
            continue
        if line.startswith("## "):
            section = ""
            continue
        if "pack/" not in line:
            continue
        packs = extract_pack_names_from_line(line)
        if section == "must":
            for pack in packs:
                append_unique(must, pack)
        elif section == "should":
            for pack in packs:
                append_unique(should, pack)
    return must, should


def resolve_age2_ssot_pack_contract(root: Path) -> tuple[list[str], list[str], list[str], list[str]]:
    missing_docs: list[str] = []
    must_expected: list[str] = []
    should_expected: list[str] = []
    for rel_path in (AGE2_SSOT_PLAN_PATH, AGE2_SSOT_README_PATH):
        doc_path = root / rel_path
        if not doc_path.exists():
            missing_docs.append(rel_path)
            continue
        must_part, should_part = parse_age2_ssot_pack_sections(load_text(doc_path))
        for pack in must_part:
            append_unique(must_expected, pack)
        for pack in should_part:
            append_unique(should_expected, pack)
    return must_expected, should_expected, missing_docs, [AGE2_SSOT_PLAN_PATH, AGE2_SSOT_README_PATH]


def main() -> int:
    parser = argparse.ArgumentParser(description="AGE2 completion gate (MUST/SHOULD pack set)")
    parser.add_argument(
        "--report-out",
        default=default_report_path("age2_completion_gate.detjson"),
        help="output JSON report path",
    )
    parser.add_argument(
        "--must-report-out",
        default=default_report_path("age2_completion_must_pack_report.detjson"),
        help="ddn.pack.golden.report.v1 output for MUST packs",
    )
    parser.add_argument(
        "--should-report-out",
        default=default_report_path("age2_completion_should_pack_report.detjson"),
        help="ddn.pack.golden.report.v1 output for SHOULD packs",
    )
    parser.add_argument(
        "--allow-should-fail",
        action="store_true",
        help="do not fail overall result when SHOULD packs fail",
    )
    parser.add_argument(
        "--must-only",
        action="store_true",
        help="run MUST packs only (implies --allow-should-fail)",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    report_out = Path(args.report_out)
    must_report_out = Path(args.must_report_out)
    should_report_out = Path(args.should_report_out)

    strict_should = (not args.allow_should_fail) and (not args.must_only)
    run_should = not args.must_only
    requested_must_shard_count = resolve_age2_must_shard_count()
    requested_should_shard_count = resolve_age2_should_shard_count()
    dynamic_shard_lookback = resolve_age2_dynamic_shard_lookback()
    dynamic_shard_mode = resolve_age2_dynamic_shard_mode()
    dynamic_source_report_path = resolve_age2_dynamic_shard_source_report(report_out)
    dynamic_source_docs: list[dict] = []
    dynamic_source_doc = None
    effective_must_shard_count = clamp_shard_count(requested_must_shard_count, len(MUST_PACKS))
    effective_should_shard_count = clamp_shard_count(requested_should_shard_count, len(SHOULD_PACKS))
    dynamic_shard_decisions: list[str] = []
    dynamic_source_reports_used_must = 0
    dynamic_source_reports_used_should = 0
    must_hint_override: dict[str, int] = {}
    should_hint_override: dict[str, int] = {}
    dynamic_hint_policy = {
        "mode": "static",
        "must_report_path": "-",
        "should_report_path": "-",
        "must_overrides": 0,
        "should_overrides": 0,
    }
    if run_should and dynamic_shard_mode == AGE2_DYNAMIC_SHARD_MODE_PREVIOUS:
        dynamic_source_docs = load_recent_source_reports(dynamic_source_report_path, dynamic_shard_lookback)
        dynamic_source_doc = dynamic_source_docs[0] if dynamic_source_docs else None
        suggested_must, must_reason, used_must = suggest_shard_count_from_previous(
            dynamic_source_docs,
            kind="must",
            current=effective_must_shard_count,
            pack_count=len(MUST_PACKS),
        )
        suggested_should, should_reason, used_should = suggest_shard_count_from_previous(
            dynamic_source_docs,
            kind="should",
            current=effective_should_shard_count,
            pack_count=len(SHOULD_PACKS),
        )
        dynamic_source_reports_used_must = int(used_must)
        dynamic_source_reports_used_should = int(used_should)
        effective_must_shard_count = clamp_shard_count(suggested_must, len(MUST_PACKS))
        effective_should_shard_count = clamp_shard_count(suggested_should, len(SHOULD_PACKS))
        dynamic_shard_decisions.append(f"must:{must_reason}")
        dynamic_shard_decisions.append(f"should:{should_reason}")
        must_hint_override, should_hint_override, dynamic_hint_policy = resolve_age2_dynamic_hint_overrides(
            dynamic_source_doc
        )
    elif run_should:
        dynamic_source_doc = load_json(dynamic_source_report_path)

    ssot_must_expected, ssot_should_expected, ssot_missing_docs, ssot_source_docs = resolve_age2_ssot_pack_contract(root)
    must_missing_from_gate = [pack for pack in ssot_must_expected if pack not in MUST_PACKS]
    should_missing_from_gate = [pack for pack in ssot_should_expected if pack not in SHOULD_PACKS]
    must_extra_in_gate = [pack for pack in MUST_PACKS if pack not in ssot_must_expected] if ssot_must_expected else []
    should_extra_in_gate = [pack for pack in SHOULD_PACKS if pack not in ssot_should_expected] if ssot_should_expected else []
    ssot_sync_ok = (
        len(ssot_missing_docs) == 0
        and len(ssot_must_expected) > 0
        and len(ssot_should_expected) > 0
        and len(must_missing_from_gate) == 0
        and len(should_missing_from_gate) == 0
        and len(must_extra_in_gate) == 0
        and len(should_extra_in_gate) == 0
    )

    must_rc = 0
    must_doc: dict | None = None
    must_rows: dict[str, dict] = {}
    must_ok = False
    must_missing: list[str] = []
    must_failed_rows: list[dict] = []

    should_rc = 0
    should_doc: dict | None = None
    should_rows: dict[str, dict] = {}
    should_ok = True
    should_missing: list[str] = []
    should_failed_rows: list[dict] = []
    must_shards: list[dict] = []
    should_shards: list[dict] = []

    if run_should:
        (
            (must_rc, must_stdout, must_stderr),
            (should_rc, should_stdout, should_stderr),
            must_shards,
            should_shards,
        ) = run_pack_batches_parallel(
            root,
            must_packs=MUST_PACKS,
            should_packs=SHOULD_PACKS,
            must_shard_count=effective_must_shard_count,
            should_shard_count=effective_should_shard_count,
            must_report_out=must_report_out,
            should_report_out=should_report_out,
            must_hint_override=must_hint_override,
            should_hint_override=should_hint_override,
        )
        if must_stdout:
            print(must_stdout, end="")
        if must_stderr:
            print(must_stderr, end="", file=sys.stderr)
        if should_stdout:
            print(should_stdout, end="")
        if should_stderr:
            print(should_stderr, end="", file=sys.stderr)

        must_doc = load_json(must_report_out)
        should_doc = load_json(should_report_out)
        must_rows = report_rows(must_doc)
        must_ok, must_missing, must_failed_rows = evaluate_pack_set(MUST_PACKS, must_rows)
        should_rows = report_rows(should_doc)
        should_ok, should_missing, should_failed_rows = evaluate_pack_set(SHOULD_PACKS, should_rows)
        if not must_shards:
            must_shards = [
                {
                    "shard_index": 1,
                    "packs": list(MUST_PACKS),
                    "pack_count": len(MUST_PACKS),
                    "hint_elapsed_ms": sum_hint_elapsed_ms(MUST_PACKS, must_hint_override),
                    "observed_elapsed_ms": observed_elapsed_ms_from_doc(must_doc),
                    "observed_elapsed_source": "report_elapsed_fallback",
                    "returncode": int(must_rc),
                    "report_loaded": bool(isinstance(must_doc, dict)),
                    "report_path": str(must_report_out),
                }
            ]
        if not should_shards:
            should_shards = [
                {
                    "shard_index": 1,
                    "packs": list(SHOULD_PACKS),
                    "pack_count": len(SHOULD_PACKS),
                    "hint_elapsed_ms": sum_hint_elapsed_ms(SHOULD_PACKS, should_hint_override),
                    "observed_elapsed_ms": observed_elapsed_ms_from_doc(should_doc),
                    "observed_elapsed_source": "report_elapsed_fallback",
                    "returncode": int(should_rc),
                    "report_loaded": bool(isinstance(should_doc, dict)),
                    "report_path": str(should_report_out),
                }
            ]
    else:
        must_rc, must_stdout, must_stderr = run_pack_batch(root, MUST_PACKS, must_report_out)
        if must_stdout:
            print(must_stdout, end="")
        if must_stderr:
            print(must_stderr, end="", file=sys.stderr)
        must_doc = load_json(must_report_out)
        must_rows = report_rows(must_doc)
        must_ok, must_missing, must_failed_rows = evaluate_pack_set(MUST_PACKS, must_rows)
        if not must_shards:
            must_shards = [
                {
                    "shard_index": 1,
                    "packs": list(MUST_PACKS),
                    "pack_count": len(MUST_PACKS),
                    "hint_elapsed_ms": sum_hint_elapsed_ms(MUST_PACKS, must_hint_override),
                    "observed_elapsed_ms": observed_elapsed_ms_from_doc(must_doc),
                    "observed_elapsed_source": "report_elapsed_fallback",
                    "returncode": int(must_rc),
                    "report_loaded": bool(isinstance(must_doc, dict)),
                    "report_path": str(must_report_out),
                }
            ]

    criteria = [
        {
            "name": "age2_ssot_pack_contract_sync",
            "ok": ssot_sync_ok,
            "detail": (
                f"docs={len(ssot_source_docs)} missing_docs={len(ssot_missing_docs)} "
                f"ssot_must={len(ssot_must_expected)} ssot_should={len(ssot_should_expected)} "
                f"must_missing_from_gate={len(must_missing_from_gate)} should_missing_from_gate={len(should_missing_from_gate)} "
                f"must_extra_in_gate={len(must_extra_in_gate)} should_extra_in_gate={len(should_extra_in_gate)}"
            ),
        },
        {
            "name": "must_pack_set_pass",
            "ok": must_ok and must_rc == 0,
            "detail": f"exit={must_rc} packs={len(MUST_PACKS)} missing={len(must_missing)} failed={len(must_failed_rows)}",
        },
        {
            "name": "should_pack_set_pass",
            "ok": (should_ok and should_rc == 0) if run_should else True,
            "detail": (
                f"exit={should_rc} packs={len(SHOULD_PACKS)} missing={len(should_missing)} failed={len(should_failed_rows)}"
                if run_should
                else "skipped (must-only)"
            ),
        },
        {
            "name": "strict_should_gate",
            "ok": (should_ok and should_rc == 0) if strict_should else True,
            "detail": f"strict_should={int(strict_should)} run_should={int(run_should)}",
        },
    ]

    overall_ok = bool(criteria[0]["ok"]) and bool(criteria[1]["ok"]) and bool(criteria[3]["ok"])
    failure_digest: list[str] = []
    failure_codes: list[str] = []
    if not bool(criteria[0]["ok"]):
        if ssot_missing_docs:
            failure_digest.extend([f"ssot-doc-missing: {path}" for path in ssot_missing_docs[:8]])
        if must_missing_from_gate:
            failure_digest.extend([f"ssot-must-missing-from-gate: {pack}" for pack in must_missing_from_gate[:8]])
        if should_missing_from_gate:
            failure_digest.extend([f"ssot-should-missing-from-gate: {pack}" for pack in should_missing_from_gate[:8]])
        if must_extra_in_gate:
            failure_digest.extend([f"gate-must-extra-vs-ssot: {pack}" for pack in must_extra_in_gate[:8]])
        if should_extra_in_gate:
            failure_digest.extend([f"gate-should-extra-vs-ssot: {pack}" for pack in should_extra_in_gate[:8]])
    if not bool(criteria[1]["ok"]):
        failure_digest.extend(build_failure_digest("must", must_missing, must_failed_rows, code_sink=failure_codes))
        if must_rc != 0:
            failure_digest.append(f"must: run_pack_golden exit={must_rc}")
    if run_should and not bool(criteria[2]["ok"]):
        failure_digest.extend(build_failure_digest("should", should_missing, should_failed_rows, code_sink=failure_codes))
        if should_rc != 0:
            failure_digest.append(f"should: run_pack_golden exit={should_rc}")

    report = {
        "schema": "ddn.age2.completion_gate.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "overall_ok": overall_ok,
        "strict_should": strict_should,
        "run_should": run_should,
        "shard_policy": {
            "mode": dynamic_shard_mode,
            "source_report_path": str(dynamic_source_report_path),
            "source_report_loaded": bool(isinstance(dynamic_source_doc, dict)),
            "lookback": int(dynamic_shard_lookback),
            "source_reports_used": {
                "must": int(dynamic_source_reports_used_must),
                "should": int(dynamic_source_reports_used_should),
            },
            "requested": {
                "must": int(requested_must_shard_count),
                "should": int(requested_should_shard_count),
            },
            "effective": {
                "must": int(effective_must_shard_count),
                "should": int(effective_should_shard_count),
            },
            "decisions": list(dynamic_shard_decisions),
            "hint_policy": dynamic_hint_policy,
        },
        "must_packs": MUST_PACKS,
        "should_packs": SHOULD_PACKS,
        "must_shards": must_shards,
        "should_shards": should_shards if run_should else [],
        "ssot_pack_contract_sync": {
            "source_docs": ssot_source_docs,
            "missing_docs": ssot_missing_docs,
            "ssot_must_expected": ssot_must_expected,
            "ssot_should_expected": ssot_should_expected,
            "must_missing_from_gate": must_missing_from_gate,
            "should_missing_from_gate": should_missing_from_gate,
            "must_extra_in_gate": must_extra_in_gate,
            "should_extra_in_gate": should_extra_in_gate,
        },
        "criteria": criteria,
        "must_report_path": str(must_report_out),
        "should_report_path": str(should_report_out) if run_should else "-",
        "failure_digest": failure_digest[:16],
        "failure_codes": failure_codes[:16],
    }
    report_out.parent.mkdir(parents=True, exist_ok=True)
    report_out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    failed_count = sum(1 for row in criteria if not bool(row.get("ok", False)))
    print(
        f"[age2-completion] overall_ok={int(overall_ok)} strict_should={int(strict_should)} "
        f"criteria={len(criteria)} failed={failed_count} report={report_out}"
    )
    print(
        f"[age2-completion] shard_policy mode={dynamic_shard_mode} "
        f"must={effective_must_shard_count} should={effective_should_shard_count}"
    )
    for row in criteria:
        print(f" - {row['name']}: ok={int(bool(row.get('ok', False)))}")
    if not overall_ok:
        for line in failure_digest[:8]:
            print(f"   {line}")
        if failure_codes:
            print(f"   failure_codes={','.join(failure_codes[:8])}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
