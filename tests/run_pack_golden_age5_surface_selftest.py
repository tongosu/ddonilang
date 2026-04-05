#!/usr/bin/env python
from __future__ import annotations

import importlib
import io
import json
import os
import subprocess
import sys
import time
from contextlib import redirect_stderr, redirect_stdout
from datetime import datetime, timezone
from pathlib import Path

PROGRESS_ENV_KEY = "DDN_CI_PACK_GOLDEN_AGE5_SURFACE_SELFTEST_PROGRESS_JSON"
PASS_BATCH_GROUP_COUNT_ENV_KEY = "DDN_AGE5_SURFACE_PASS_BATCH_GROUP_COUNT"
SELFTEST_REPORT_ENV_KEY = "DDN_AGE5_SURFACE_SELFTEST_REPORT_JSON"
PASS_BATCH_DYNAMIC_GROUP_MODE_ENV_KEY = "DDN_AGE5_SURFACE_DYNAMIC_GROUP_MODE"
PASS_BATCH_DYNAMIC_GROUP_SOURCE_REPORT_ENV_KEY = "DDN_AGE5_SURFACE_DYNAMIC_GROUP_SOURCE_REPORT"
PASS_BATCH_DYNAMIC_GROUP_LOOKBACK_ENV_KEY = "DDN_AGE5_SURFACE_DYNAMIC_GROUP_LOOKBACK"
PASS_BATCH_DYNAMIC_GROUP_MODE_OFF = "off"
PASS_BATCH_DYNAMIC_GROUP_MODE_PREVIOUS = "previous"
_RUN_PACK_GOLDEN_MODULE = None
# 2026-03-26 재측정 기준: 단독 벤치는 3그룹이 빠르지만 core_lang 체인에서는 4그룹이 더 안정적.
PASS_BATCH_GROUP_COUNT = 4
PASS_PACK_ELAPSED_HINT_MS = {
    "seamgrim_moyang_template_instance_view_boundary_v1": 1029,
    "seamgrim_guseong_flatten_diag_v1": 655,
    "seamgrim_event_surface_canon_v1": 282,
    "seamgrim_jjaim_block_stub_canon_v1": 265,
    "block_header_no_colon": 219,
    "seamgrim_guseong_flatten_ir_v1": 154,
    "seamgrim_event_model_ir_v1": 136,
}
NEG_WARNING_PACK_NAME = "_tmp_age5_surface_selftest_warning"
NEG_CONTRACT_PACK_NAME = "_tmp_age5_surface_selftest_contract"


def ascii_safe(text: str) -> str:
    return text.encode("ascii", errors="backslashreplace").decode("ascii")


def fail(msg: str) -> int:
    print(f"[pack-golden-age5-surface-selftest] fail: {ascii_safe(msg)}")
    return 1


def write_json_atomic(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = path.with_name(f"{path.name}.{time.time_ns()}.tmp")
    tmp_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for attempt in range(10):
        try:
            tmp_path.replace(path)
            return
        except PermissionError:
            if attempt >= 9:
                raise
            time.sleep(0.01 * (attempt + 1))


def write_progress_snapshot(
    path_text: str,
    *,
    status: str,
    current_case: str,
    last_completed_case: str,
    current_probe: str,
    last_completed_probe: str,
    total_elapsed_ms: int,
) -> None:
    if not str(path_text).strip():
        return
    out = Path(path_text)
    payload = {
        "schema": "ddn.ci.pack_golden_age5_surface_selftest.progress.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": status,
        "current_case": current_case,
        "last_completed_case": last_completed_case,
        "current_probe": current_probe,
        "last_completed_probe": last_completed_probe,
        "total_elapsed_ms": int(total_elapsed_ms),
    }
    write_json_atomic(out, payload)


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


def resolve_selftest_report_path() -> Path:
    raw = str(os.environ.get(SELFTEST_REPORT_ENV_KEY, "")).strip()
    if raw:
        return Path(raw)
    return Path(default_report_path("pack_golden_age5_surface_selftest.detjson"))


def resolve_dynamic_group_mode() -> str:
    raw = str(os.environ.get(PASS_BATCH_DYNAMIC_GROUP_MODE_ENV_KEY, "")).strip().lower()
    if raw in ("", "0", "off", "false", "no"):
        return PASS_BATCH_DYNAMIC_GROUP_MODE_OFF
    if raw in ("1", "on", "true", "yes", "previous"):
        return PASS_BATCH_DYNAMIC_GROUP_MODE_PREVIOUS
    return PASS_BATCH_DYNAMIC_GROUP_MODE_OFF


def resolve_dynamic_group_source_report(report_out: Path) -> Path:
    raw = str(os.environ.get(PASS_BATCH_DYNAMIC_GROUP_SOURCE_REPORT_ENV_KEY, "")).strip()
    if raw:
        return Path(raw)
    return report_out


def resolve_dynamic_group_lookback() -> int:
    raw = str(os.environ.get(PASS_BATCH_DYNAMIC_GROUP_LOOKBACK_ENV_KEY, "")).strip()
    if not raw:
        return 3
    try:
        parsed = int(raw)
    except ValueError:
        return 3
    return min(max(1, parsed), 20)


def clamp_group_count(value: int, pack_count: int) -> int:
    if pack_count <= 0:
        return 0
    return min(max(1, int(value)), int(pack_count))


def sum_hint_elapsed_ms(pack_names: tuple[str, ...]) -> int:
    total = 0
    for name in pack_names:
        total += int(PASS_PACK_ELAPSED_HINT_MS.get(name, 200))
    return total


def extract_previous_batch_groups(report_doc: dict | None) -> list[dict]:
    if not isinstance(report_doc, dict):
        return []
    rows = report_doc.get("pass_batch_groups")
    if not isinstance(rows, list):
        return []
    out: list[dict] = []
    for row in rows:
        if isinstance(row, dict):
            out.append(row)
    return out


def build_recent_source_report_glob_pattern(reference_report_path: Path) -> str:
    name = reference_report_path.name
    if name.endswith(".dynamic.last.detjson"):
        prefix = name[: -len(".dynamic.last.detjson")]
        return f"{prefix}.*.detjson"
    if name.endswith(".detjson"):
        stem = name[: -len(".detjson")]
        parts = stem.split(".")
        if len(parts) >= 4 and parts[-1].isdigit() and parts[-2].isdigit():
            prefix = ".".join(parts[:-2])
            return f"{prefix}.*.detjson"
    return name


def load_recent_source_reports(reference_report_path: Path, lookback: int) -> list[dict]:
    def is_timestamped_selftest_report(path: Path) -> bool:
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
        if not is_timestamped_selftest_report(candidate):
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


def median_float(values: list[float]) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    n = len(ordered)
    mid = n // 2
    if n % 2 == 1:
        return float(ordered[mid])
    return float((ordered[mid - 1] + ordered[mid]) / 2.0)


def peak_ratio_from_groups(previous_groups: list[dict]) -> float:
    observed: list[int] = []
    for row in previous_groups:
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


def suggest_group_count_from_previous(
    previous_source_docs: list[dict],
    current: int,
    pack_count: int,
) -> tuple[int, str, int]:
    safe_current = clamp_group_count(current, pack_count)
    if pack_count <= 0:
        return 0, "no-pack", 0
    ratios: list[float] = []
    for doc in previous_source_docs:
        ratio = peak_ratio_from_groups(extract_previous_batch_groups(doc))
        if ratio > 0.0:
            ratios.append(ratio)
    if not ratios:
        return safe_current, "no-observed-groups", 0
    peak_ratio = median_float(ratios)
    if peak_ratio >= 0.50 and safe_current < pack_count:
        return safe_current + 1, f"increase-hotspot ratio={peak_ratio:.3f}", len(ratios)
    # core_lang 체인에서는 4그룹 기본값이 더 안정적이라 축소는 강한 underload에서만 허용한다.
    if peak_ratio <= 0.20 and safe_current > 1:
        return safe_current - 1, f"decrease-underload ratio={peak_ratio:.3f}", len(ratios)
    return safe_current, f"keep ratio={peak_ratio:.3f}", len(ratios)


def spawn_pack(root: Path, *pack_names: str, env_patch: dict[str, str] | None = None) -> subprocess.Popen[str]:
    if not pack_names:
        raise ValueError("spawn_pack requires at least one pack name")
    cmd = [sys.executable, "-S", "tests/run_pack_golden.py", *pack_names]
    env = dict(os.environ)
    if env_patch:
        env.update(env_patch)
    return subprocess.Popen(
        cmd,
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
    )


def collect_pack_process(proc: subprocess.Popen[str]) -> subprocess.CompletedProcess[str]:
    stdout, stderr = proc.communicate()
    return subprocess.CompletedProcess(
        args=proc.args,
        returncode=proc.returncode,
        stdout=stdout,
        stderr=stderr,
    )


def collect_pack_processes(procs: list[subprocess.Popen[str]]) -> list[subprocess.CompletedProcess[str]]:
    return [collect_pack_process(proc) for proc in procs]


def collect_pack_processes_with_observed_elapsed(
    procs: list[subprocess.Popen[str]],
    started_at: list[float],
) -> tuple[list[subprocess.CompletedProcess[str]], list[int]]:
    if len(procs) != len(started_at):
        raise ValueError("collect_pack_processes_with_observed_elapsed requires matching lengths")
    results: list[subprocess.CompletedProcess[str] | None] = [None] * len(procs)
    elapsed_ms: list[int] = [0] * len(procs)
    pending = set(range(len(procs)))
    while pending:
        progressed = False
        for idx in list(pending):
            proc = procs[idx]
            if proc.poll() is None:
                continue
            finished_at = time.perf_counter()
            stdout, stderr = proc.communicate()
            results[idx] = subprocess.CompletedProcess(
                args=proc.args,
                returncode=proc.returncode,
                stdout=stdout,
                stderr=stderr,
            )
            elapsed_ms[idx] = max(0, int(round((finished_at - float(started_at[idx])) * 1000.0)))
            pending.remove(idx)
            progressed = True
        if pending and not progressed:
            time.sleep(0.005)
    return [row for row in results if isinstance(row, subprocess.CompletedProcess)], elapsed_ms


def split_pack_groups(pack_names: tuple[str, ...], group_count: int = 2) -> list[tuple[str, ...]]:
    if not pack_names:
        return []
    original_order = list(pack_names)
    original_index = {name: idx for idx, name in enumerate(original_order)}
    actual_count = min(max(1, int(group_count)), len(original_order))
    groups: list[list[str]] = [[] for _ in range(actual_count)]
    loads = [0 for _ in range(actual_count)]
    weighted = sorted(
        original_order,
        key=lambda name: (-int(PASS_PACK_ELAPSED_HINT_MS.get(name, 200)), name),
    )
    for name in weighted:
        idx = min(range(actual_count), key=lambda i: loads[i])
        groups[idx].append(name)
        loads[idx] += int(PASS_PACK_ELAPSED_HINT_MS.get(name, 200))

    # Greedy 이후 이동/교환 기반 로컬 리밸런싱으로 최대 그룹 부하를 완화한다.
    def current_max_load() -> int:
        return max(loads) if loads else 0

    while True:
        best_kind = ""
        best_src = -1
        best_dst = -1
        best_i = -1
        best_j = -1
        best_value = current_max_load()

        for src in range(actual_count):
            if len(groups[src]) <= 1:
                continue
            for i, name in enumerate(groups[src]):
                w = int(PASS_PACK_ELAPSED_HINT_MS.get(name, 200))
                for dst in range(actual_count):
                    if src == dst:
                        continue
                    next_src = loads[src] - w
                    next_dst = loads[dst] + w
                    next_max = max(
                        next_src if k == src else next_dst if k == dst else loads[k]
                        for k in range(actual_count)
                    )
                    if next_max < best_value:
                        best_kind = "move"
                        best_src = src
                        best_dst = dst
                        best_i = i
                        best_j = -1
                        best_value = next_max

        for left in range(actual_count):
            if not groups[left]:
                continue
            for right in range(left + 1, actual_count):
                if not groups[right]:
                    continue
                for i, left_name in enumerate(groups[left]):
                    wl = int(PASS_PACK_ELAPSED_HINT_MS.get(left_name, 200))
                    for j, right_name in enumerate(groups[right]):
                        wr = int(PASS_PACK_ELAPSED_HINT_MS.get(right_name, 200))
                        next_left = loads[left] - wl + wr
                        next_right = loads[right] - wr + wl
                        next_max = max(
                            next_left if k == left else next_right if k == right else loads[k]
                            for k in range(actual_count)
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
            moved = groups[best_src].pop(best_i)
            wm = int(PASS_PACK_ELAPSED_HINT_MS.get(moved, 200))
            groups[best_dst].append(moved)
            loads[best_src] -= wm
            loads[best_dst] += wm
            continue

        left_name = groups[best_src][best_i]
        right_name = groups[best_dst][best_j]
        wl = int(PASS_PACK_ELAPSED_HINT_MS.get(left_name, 200))
        wr = int(PASS_PACK_ELAPSED_HINT_MS.get(right_name, 200))
        groups[best_src][best_i] = right_name
        groups[best_dst][best_j] = left_name
        loads[best_src] = loads[best_src] - wl + wr
        loads[best_dst] = loads[best_dst] - wr + wl

    for idx in range(actual_count):
        groups[idx] = sorted(groups[idx], key=lambda name: original_index.get(name, 0))
    return [tuple(group) for group in groups if group]


def pick_first_pass_pack(pass_pack_names: tuple[str, ...]) -> str:
    if not pass_pack_names:
        raise ValueError("pick_first_pass_pack requires at least one pack")
    return min(
        pass_pack_names,
        key=lambda name: (-int(PASS_PACK_ELAPSED_HINT_MS.get(name, 200)), name),
    )


def resolve_pass_batch_group_count() -> int:
    raw = str(os.environ.get(PASS_BATCH_GROUP_COUNT_ENV_KEY, "")).strip()
    if not raw:
        return PASS_BATCH_GROUP_COUNT
    try:
        parsed = int(raw)
    except ValueError:
        return PASS_BATCH_GROUP_COUNT
    if parsed <= 0:
        return PASS_BATCH_GROUP_COUNT
    return parsed


def run_pack(root: Path, *pack_names: str) -> subprocess.CompletedProcess[str]:
    if not pack_names:
        raise ValueError("run_pack requires at least one pack name")
    cmd = [sys.executable, "-S", "tests/run_pack_golden.py", *pack_names]
    global _RUN_PACK_GOLDEN_MODULE
    if _RUN_PACK_GOLDEN_MODULE is None:
        _RUN_PACK_GOLDEN_MODULE = importlib.import_module("run_pack_golden")

    argv = ["tests/run_pack_golden.py", *pack_names]
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
    return subprocess.CompletedProcess(
        args=cmd,
        returncode=returncode,
        stdout=stdout_buf.getvalue(),
        stderr=stderr_buf.getvalue(),
    )


def write_json(path: Path, payload: dict) -> None:
    write_json_atomic(path, payload)


def build_negative_warning_pack(root: Path, pack_name: str) -> Path:
    pack_dir = root / "pack" / pack_name
    case_dir = pack_dir / "c01_warning_mismatch"
    case_dir.mkdir(parents=True, exist_ok=True)
    input_path = case_dir / "input.ddn"
    input_path.write_text("채비: {\n  값:수 <- 1.\n}.\n", encoding="utf-8", newline="\n")
    golden_path = pack_dir / "golden.jsonl"
    golden_path.write_text(
        json.dumps(
            {
                "cmd": ["canon", f"pack/{pack_name}/c01_warning_mismatch/input.ddn"],
                "expected_warning_code": "W_AGE5_SURFACE_SELFTEST_NON_EXISTENT",
            },
            ensure_ascii=False,
        )
        + "\n",
        encoding="utf-8",
    )
    return pack_dir


def build_invalid_contract_pack(root: Path, pack_name: str) -> Path:
    pack_dir = root / "pack" / pack_name
    case_dir = pack_dir / "c01_invalid_contract"
    case_dir.mkdir(parents=True, exist_ok=True)
    input_path = case_dir / "input.ddn"
    input_path.write_text('"KIND"라는 소식이 오면 {\n  x <- 1.\n}.\n', encoding="utf-8", newline="\n")
    golden_path = pack_dir / "golden.jsonl"
    golden_path.write_text(
        json.dumps(
            {
                "cmd": ["run", f"pack/{pack_name}/c01_invalid_contract/input.ddn"],
                "exit_code": 1,
                "expected_warning_code": "W_AGE5_SURFACE_SELFTEST_DUMMY",
            },
            ensure_ascii=False,
        )
        + "\n",
        encoding="utf-8",
    )
    return pack_dir


def ensure_pack_pass(root: Path, pack_name: str) -> int:
    proc = run_pack(root, pack_name)
    if proc.returncode != 0:
        return fail(f"{pack_name} must pass: out={proc.stdout} err={proc.stderr}")
    if "pack golden ok" not in (proc.stdout or ""):
        return fail(f"{pack_name} pass marker missing")
    return 0


def validate_pack_pass_returncode(proc: subprocess.CompletedProcess[str], pack_name: str) -> int:
    if proc.returncode != 0:
        return fail(f"{pack_name} must pass: out={proc.stdout} err={proc.stderr}")
    return 0


def validate_pack_pass_marker(proc: subprocess.CompletedProcess[str], pack_name: str) -> int:
    if "pack golden ok" not in (proc.stdout or ""):
        return fail(f"{pack_name} pass marker missing")
    return 0


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    progress_path = str(os.environ.get(PROGRESS_ENV_KEY, "")).strip()
    report_out = resolve_selftest_report_path()
    dynamic_group_mode = resolve_dynamic_group_mode()
    dynamic_group_source_report = resolve_dynamic_group_source_report(report_out)
    dynamic_group_lookback = resolve_dynamic_group_lookback()
    dynamic_group_source_docs: list[dict] = []
    dynamic_group_source_doc: dict | None = None
    started_at = time.perf_counter()
    current_case = "-"
    last_completed_case = "-"
    current_probe = "-"
    last_completed_probe = "-"
    dynamic_group_decisions: list[str] = []
    dynamic_group_source_reports_used = 0
    first_pass_elapsed_ms = 0
    pass_batch_groups_report: list[dict] = []

    def update_progress(status: str) -> None:
        write_progress_snapshot(
            progress_path,
            status=status,
            current_case=current_case,
            last_completed_case=last_completed_case,
            current_probe=current_probe,
            last_completed_probe=last_completed_probe,
            total_elapsed_ms=int((time.perf_counter() - started_at) * 1000),
        )

    def start_case(name: str) -> None:
        nonlocal current_case, current_probe
        current_case = name
        current_probe = "-"
        update_progress("running")

    def complete_case(name: str) -> None:
        nonlocal current_case, current_probe, last_completed_case
        current_case = "-"
        current_probe = "-"
        last_completed_case = name
        update_progress("running")

    def start_probe(name: str) -> None:
        nonlocal current_probe
        current_probe = name
        update_progress("running")

    def complete_probe(name: str) -> None:
        nonlocal current_probe, last_completed_probe
        current_probe = "-"
        last_completed_probe = name
        update_progress("running")

    update_progress("running")

    pass_pack_names = (
        "seamgrim_event_model_ir_v1",
        "seamgrim_moyang_template_instance_view_boundary_v1",
        "seamgrim_jjaim_block_stub_canon_v1",
        "seamgrim_guseong_flatten_ir_v1",
        "seamgrim_guseong_flatten_diag_v1",
        "seamgrim_event_surface_canon_v1",
        "block_header_no_colon",
        "seamgrim_bogae_madang_alias_v1",
    )
    first_name = pick_first_pass_pack(pass_pack_names)
    grouped_pass_packs: list[tuple[str, ...]] = []
    batch_procs: list[subprocess.Popen[str]] = []
    batch_jobs: list[dict] = []
    remaining_pass_packs = tuple(name for name in pass_pack_names if name != first_name)
    requested_batch_group_count = resolve_pass_batch_group_count()
    effective_batch_group_count = clamp_group_count(requested_batch_group_count, len(remaining_pass_packs))
    if dynamic_group_mode == PASS_BATCH_DYNAMIC_GROUP_MODE_PREVIOUS:
        dynamic_group_source_docs = load_recent_source_reports(dynamic_group_source_report, dynamic_group_lookback)
        if dynamic_group_source_docs:
            dynamic_group_source_doc = dynamic_group_source_docs[0]
        suggested_count, reason, used_reports = suggest_group_count_from_previous(
            dynamic_group_source_docs,
            effective_batch_group_count,
            len(remaining_pass_packs),
        )
        effective_batch_group_count = clamp_group_count(suggested_count, len(remaining_pass_packs))
        dynamic_group_decisions.append(reason)
        dynamic_group_source_reports_used = int(used_reports)
    else:
        dynamic_group_source_doc = load_json(dynamic_group_source_report)
    if remaining_pass_packs:
        grouped_pass_packs = split_pack_groups(
            remaining_pass_packs,
            group_count=effective_batch_group_count,
        )
    start_case(f"pass.{first_name}")
    start_probe("ensure_pack_pass.run_pack.spawn_process")
    if grouped_pass_packs:
        for idx, group in enumerate(grouped_pass_packs, 1):
            proc = spawn_pack(root, *group)
            batch_procs.append(proc)
            batch_jobs.append(
                {
                    "group_index": idx,
                    "packs": tuple(group),
                    "hint_elapsed_ms": sum_hint_elapsed_ms(tuple(group)),
                    "started_at": time.perf_counter(),
                    "proc": proc,
                }
            )
    proc_ok = None
    complete_probe("ensure_pack_pass.run_pack.spawn_process")
    start_probe("ensure_pack_pass.run_pack.wait_exit")
    first_started = time.perf_counter()
    proc_ok_completed = run_pack(root, first_name)
    first_pass_elapsed_ms = int(round((time.perf_counter() - first_started) * 1000.0))
    complete_probe("ensure_pack_pass.run_pack.wait_exit")
    start_probe("ensure_pack_pass.run_pack.collect_output")
    proc_ok = proc_ok_completed
    complete_probe("ensure_pack_pass.run_pack.collect_output")
    start_probe("ensure_pack_pass.validate_returncode")
    rc = validate_pack_pass_returncode(proc_ok, first_name)
    complete_probe("ensure_pack_pass.validate_returncode")
    if rc != 0:
        update_progress("fail")
        return rc
    start_probe("ensure_pack_pass.validate_pass_marker")
    rc = validate_pack_pass_marker(proc_ok, first_name)
    complete_probe("ensure_pack_pass.validate_pass_marker")
    if rc != 0:
        if batch_procs:
            _ = collect_pack_processes(batch_procs)
        update_progress("fail")
        return rc
    complete_case(f"pass.{first_name}")

    if grouped_pass_packs:
        start_case("pass.batch_remaining")
        start_probe("ensure_pack_pass_batch.wait_exit_and_collect")
        batch_results, batch_elapsed_ms = collect_pack_processes_with_observed_elapsed(
            batch_procs,
            [float(job.get("started_at", started_at)) for job in batch_jobs],
        )
        for job, proc_batch, observed_elapsed_ms in zip(batch_jobs, batch_results, batch_elapsed_ms):
            pass_batch_groups_report.append(
                {
                    "group_index": int(job.get("group_index", 0)),
                    "packs": list(job.get("packs", ())),
                    "pack_count": len(list(job.get("packs", ()))),
                    "hint_elapsed_ms": int(job.get("hint_elapsed_ms", 0)),
                    "observed_elapsed_ms": max(0, observed_elapsed_ms),
                    "returncode": int(proc_batch.returncode),
                }
            )
        pass_batch_groups_report.sort(key=lambda row: int(row.get("group_index", 0)))
        complete_probe("ensure_pack_pass_batch.wait_exit_and_collect")
        start_probe("ensure_pack_pass_batch.validate_returncode")
        for group, proc_batch in zip(grouped_pass_packs, batch_results):
            remaining_label = ",".join(group)
            rc = validate_pack_pass_returncode(proc_batch, remaining_label)
            if rc != 0:
                complete_probe("ensure_pack_pass_batch.validate_returncode")
                update_progress("fail")
                return rc
        complete_probe("ensure_pack_pass_batch.validate_returncode")
        start_probe("ensure_pack_pass_batch.validate_pass_marker")
        for group, proc_batch in zip(grouped_pass_packs, batch_results):
            remaining_label = ",".join(group)
            rc = validate_pack_pass_marker(proc_batch, remaining_label)
            if rc != 0:
                complete_probe("ensure_pack_pass_batch.validate_pass_marker")
                update_progress("fail")
                return rc
        complete_probe("ensure_pack_pass_batch.validate_pass_marker")
        complete_case("pass.batch_remaining")

    # case 1: warning mismatch should fail
    temp_name = NEG_WARNING_PACK_NAME
    start_case("fail.warning_mismatch")
    start_probe("build_negative_warning_pack")
    build_negative_warning_pack(root, temp_name)
    complete_probe("build_negative_warning_pack")
    start_probe("run_pack")
    proc_fail = run_pack(root, temp_name)
    complete_probe("run_pack")
    if proc_fail.returncode == 0:
        update_progress("fail")
        return fail("negative warning mismatch pack must fail")
    merged = (proc_fail.stdout or "") + "\n" + (proc_fail.stderr or "")
    if "pack golden failed" not in merged:
        update_progress("fail")
        return fail(f"negative warning mismatch marker missing: out={proc_fail.stdout} err={proc_fail.stderr}")
    if "[FAIL] pack=" not in merged:
        update_progress("fail")
        return fail("negative warning mismatch digest missing [FAIL] pack line")
    complete_case("fail.warning_mismatch")

    # case 2: invalid exit_code contract should fail before execution
    temp_name_contract = NEG_CONTRACT_PACK_NAME
    start_case("fail.invalid_contract")
    start_probe("build_invalid_contract_pack")
    build_invalid_contract_pack(root, temp_name_contract)
    complete_probe("build_invalid_contract_pack")
    start_probe("run_pack")
    proc_contract_fail = run_pack(root, temp_name_contract)
    complete_probe("run_pack")
    if proc_contract_fail.returncode == 0:
        update_progress("fail")
        return fail("invalid contract pack must fail")
    merged_contract = (proc_contract_fail.stdout or "") + "\n" + (proc_contract_fail.stderr or "")
    if "non-zero exit_code requires expected_error_code" not in merged_contract:
        update_progress("fail")
        return fail(
            "invalid contract failure marker missing: non-zero exit_code requires expected_error_code"
        )
    complete_case("fail.invalid_contract")

    report_payload = {
        "schema": "ddn.ci.pack_golden_age5_surface_selftest.report.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "overall_ok": True,
        "first_pass_pack": first_name,
        "first_pass_elapsed_ms": int(first_pass_elapsed_ms),
        "pass_pack_names": list(pass_pack_names),
        "pass_batch_group_count_requested": int(requested_batch_group_count),
        "pass_batch_group_count_effective": int(effective_batch_group_count),
        "dynamic_group_policy": {
            "mode": dynamic_group_mode,
            "source_report_path": str(dynamic_group_source_report),
            "source_report_loaded": bool(len(dynamic_group_source_docs) > 0) or bool(isinstance(dynamic_group_source_doc, dict)),
            "lookback": int(dynamic_group_lookback),
            "source_reports_used": int(dynamic_group_source_reports_used),
            "decisions": list(dynamic_group_decisions),
        },
        "pass_batch_groups": pass_batch_groups_report,
        "negative_cases": [
            "fail.warning_mismatch",
            "fail.invalid_contract",
        ],
        "total_elapsed_ms": int((time.perf_counter() - started_at) * 1000),
    }
    write_json_atomic(report_out, report_payload)

    update_progress("pass")
    print("[pack-golden-age5-surface-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
