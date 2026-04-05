#!/usr/bin/env python
from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

from _selftest_exec_cache import is_script_cached, mark_script_ok

ROOT = Path(__file__).resolve().parents[1]
SELF_SCRIPT_PATH = "tests/run_age4_proof_transport_contract_selftest.py"
PROGRESS_ENV_KEY = "DDN_AGE4_PROOF_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON"
MAX_WORKERS_ENV_KEY = "DDN_AGE4_PROOF_TRANSPORT_MAX_WORKERS"
SELFTEST_REPORT_ENV_KEY = "DDN_AGE4_PROOF_TRANSPORT_CONTRACT_SELFTEST_REPORT_JSON"
DYNAMIC_WORKER_MODE_ENV_KEY = "DDN_AGE4_PROOF_TRANSPORT_DYNAMIC_WORKER_MODE"
DYNAMIC_WORKER_SOURCE_REPORT_ENV_KEY = "DDN_AGE4_PROOF_TRANSPORT_DYNAMIC_WORKER_SOURCE_REPORT"
DYNAMIC_WORKER_LOOKBACK_ENV_KEY = "DDN_AGE4_PROOF_TRANSPORT_DYNAMIC_WORKER_LOOKBACK"
DYNAMIC_WORKER_MODE_OFF = "off"
DYNAMIC_WORKER_MODE_PREVIOUS = "previous"
CHECKS: tuple[tuple[str, tuple[str, ...]], ...] = (
    ("proof_artifact_digest", ("tests/run_proof_artifact_digest_selftest.py",)),
    ("proof_artifact_report", ("tests/run_age4_proof_artifact_report_selftest.py",)),
    ("aggregate_combine", ("tests/run_ci_combine_reports_age4_selftest.py",)),
    ("aggregate_status_line", ("tests/run_ci_aggregate_status_line_selftest.py",)),
    ("gate_summary_report", ("tests/run_ci_gate_summary_report_check_selftest.py",)),
    ("emit_artifacts", ("tests/run_ci_emit_artifacts_check_selftest.py",)),
    ("final_line_emitter", ("tests/run_ci_final_line_emitter_check.py",)),
    ("report_index", ("tests/run_ci_gate_report_index_check_selftest.py",)),
)
CHECKS_TEXT = ",".join(name for name, _ in CHECKS)


def clip(text: str, limit: int = 200) -> str:
    normalized = " ".join(str(text).split())
    if not normalized:
        return "-"
    if len(normalized) <= limit:
        return normalized
    return normalized[: limit - 3] + "..."


def fail(detail: str) -> int:
    print(f"[age4-proof-transport-selftest] fail: {detail}")
    return 1


def progress_path() -> Path | None:
    raw = os.environ.get(PROGRESS_ENV_KEY, "").strip()
    if not raw:
        return None
    return Path(raw)


def write_progress(
    *,
    status: str,
    current_probe: str,
    last_completed_probe: str,
    completed_checks: int,
    current_check: str,
    last_completed_check: str,
    failed_check: str = "-",
    total_elapsed_ms: int = 0,
) -> None:
    path = progress_path()
    if path is None:
        return
    payload = {
        "schema": "ddn.ci.age4_proof_transport_contract_selftest.progress.v1",
        "status": status,
        "current_probe": current_probe,
        "last_completed_probe": last_completed_probe,
        "completed_checks": completed_checks,
        "total_checks": len(CHECKS),
        "checks_text": CHECKS_TEXT,
        "current_check": current_check,
        "last_completed_check": last_completed_check,
        "failed_check": failed_check,
        "total_elapsed_ms": total_elapsed_ms,
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(payload, ensure_ascii=False, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


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


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def resolve_report_out() -> Path:
    raw = os.environ.get(SELFTEST_REPORT_ENV_KEY, "").strip()
    if raw:
        return Path(raw)
    return Path(default_report_path("age4_proof_transport_contract_selftest.detjson"))


def resolve_dynamic_worker_mode() -> str:
    raw = os.environ.get(DYNAMIC_WORKER_MODE_ENV_KEY, "").strip().lower()
    if raw in ("", "0", "off", "false", "no"):
        return DYNAMIC_WORKER_MODE_OFF
    if raw in ("1", "on", "true", "yes", "previous"):
        return DYNAMIC_WORKER_MODE_PREVIOUS
    return DYNAMIC_WORKER_MODE_OFF


def resolve_dynamic_worker_source_report(report_out: Path) -> Path:
    raw = os.environ.get(DYNAMIC_WORKER_SOURCE_REPORT_ENV_KEY, "").strip()
    if raw:
        return Path(raw)
    return report_out


def resolve_dynamic_worker_lookback() -> int:
    raw = os.environ.get(DYNAMIC_WORKER_LOOKBACK_ENV_KEY, "").strip()
    if not raw:
        return 3
    try:
        parsed = int(raw)
    except ValueError:
        return 3
    return max(1, min(20, parsed))


def clamp_workers(value: int, script_count: int) -> int:
    if script_count <= 0:
        return 1
    return min(max(1, int(value)), int(script_count))


def extract_script_runs(report_doc: dict | None) -> list[dict]:
    if not isinstance(report_doc, dict):
        return []
    rows = report_doc.get("script_runs")
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


def median_float(values: list[float]) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    n = len(ordered)
    mid = n // 2
    if n % 2 == 1:
        return float(ordered[mid])
    return float((ordered[mid - 1] + ordered[mid]) / 2.0)


def peak_ratio_from_runs(previous_runs: list[dict]) -> float:
    observed: list[int] = []
    for row in previous_runs:
        if bool(row.get("cached", False)):
            continue
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


def suggest_workers_from_previous(previous_docs: list[dict], current: int, script_count: int) -> tuple[int, str, int]:
    safe_current = clamp_workers(current, script_count)
    ratios: list[float] = []
    for doc in previous_docs:
        ratio = peak_ratio_from_runs(extract_script_runs(doc))
        if ratio > 0.0:
            ratios.append(ratio)
    if not ratios:
        return safe_current, "no-observed-runs", 0
    peak_ratio = median_float(ratios)
    if peak_ratio >= 0.45 and safe_current < script_count:
        return safe_current + 1, f"increase-hotspot ratio={peak_ratio:.3f}", len(ratios)
    # 단독 실측(2026-03-26)에서 workers=8이 workers=7보다 유의미하게 빨라
    # 축소는 매우 강한 underload 구간에서만 허용한다.
    if peak_ratio <= 0.18 and safe_current > 1:
        return safe_current - 1, f"decrease-underload ratio={peak_ratio:.3f}", len(ratios)
    return safe_current, f"keep ratio={peak_ratio:.3f}", len(ratios)


def run_check(script_rel: str) -> subprocess.CompletedProcess[bytes]:
    cmd = [sys.executable, "-S", script_rel]
    first = subprocess.run(
        cmd,
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    if first.returncode == 0:
        return subprocess.CompletedProcess(
            args=cmd,
            returncode=0,
            stdout=b"",
            stderr=b"",
        )
    return subprocess.run(
        cmd,
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
    )


def decode_stderr(proc: subprocess.CompletedProcess[bytes]) -> str:
    raw = proc.stderr or b""
    if isinstance(raw, bytes):
        return raw.decode("utf-8", errors="replace")
    return str(raw)


def run_check_timed(script_rel: str) -> tuple[subprocess.CompletedProcess[bytes], int]:
    started = time.perf_counter()
    proc = run_check(script_rel)
    elapsed_ms = int(round((time.perf_counter() - started) * 1000.0))
    return proc, max(0, elapsed_ms)


def resolve_max_workers(script_count: int) -> int:
    if script_count <= 0:
        return 1
    raw = os.environ.get(MAX_WORKERS_ENV_KEY, "").strip()
    if raw:
        try:
            parsed = int(raw)
        except ValueError:
            parsed = 0
        if parsed > 0:
            return max(1, min(parsed, script_count))
    cpu_workers = os.cpu_count() or 4
    return max(1, min(8, cpu_workers, script_count))


def build_report_payload(
    *,
    overall_ok: bool,
    started: float,
    failed_check: str,
    requested_workers: int,
    effective_workers: int,
    dynamic_worker_mode: str,
    dynamic_worker_source_report: Path,
    dynamic_worker_source_loaded: bool,
    dynamic_worker_lookback: int,
    dynamic_worker_source_reports_used: int,
    dynamic_worker_decisions: list[str],
    script_runs: list[dict],
    failure_detail: str,
) -> dict:
    return {
        "schema": "ddn.ci.age4_proof_transport_contract_selftest.report.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "overall_ok": bool(overall_ok),
        "checks_text": CHECKS_TEXT,
        "total_checks": len(CHECKS),
        "failed_check": failed_check,
        "max_workers_requested": int(requested_workers),
        "max_workers_effective": int(effective_workers),
        "dynamic_worker_policy": {
            "mode": dynamic_worker_mode,
            "source_report_path": str(dynamic_worker_source_report),
            "source_report_loaded": bool(dynamic_worker_source_loaded),
            "lookback": int(dynamic_worker_lookback),
            "source_reports_used": int(dynamic_worker_source_reports_used),
            "decisions": list(dynamic_worker_decisions),
        },
        "script_runs": list(script_runs),
        "failure_detail": failure_detail,
        "total_elapsed_ms": int((time.perf_counter() - started) * 1000),
    }


def main() -> int:
    started = time.perf_counter()
    last_completed = "-"
    script_futures: dict[str, object] = {}
    uncached_scripts: list[str] = []
    script_cached: dict[str, bool] = {}
    for _, scripts in CHECKS:
        for script_rel in scripts:
            cached = is_script_cached(script_rel)
            script_cached[script_rel] = cached
            if cached:
                continue
            if script_rel not in script_futures:
                script_futures[script_rel] = None
                uncached_scripts.append(script_rel)
    report_out = resolve_report_out()
    dynamic_worker_mode = resolve_dynamic_worker_mode()
    dynamic_worker_source_report = resolve_dynamic_worker_source_report(report_out)
    dynamic_worker_lookback = resolve_dynamic_worker_lookback()
    dynamic_worker_source_docs: list[dict] = []
    requested_workers = resolve_max_workers(len(uncached_scripts))
    effective_workers = clamp_workers(requested_workers, len(uncached_scripts))
    dynamic_worker_decisions: list[str] = []
    dynamic_worker_source_reports_used = 0
    if dynamic_worker_mode == DYNAMIC_WORKER_MODE_PREVIOUS and len(uncached_scripts) > 0:
        dynamic_worker_source_docs = load_recent_source_reports(dynamic_worker_source_report, dynamic_worker_lookback)
        suggested_workers, reason, used_reports = suggest_workers_from_previous(
            dynamic_worker_source_docs,
            effective_workers,
            len(uncached_scripts),
        )
        effective_workers = clamp_workers(suggested_workers, len(uncached_scripts))
        dynamic_worker_decisions.append(reason)
        dynamic_worker_source_reports_used = int(used_reports)
    script_runs: dict[str, dict] = {}

    write_progress(
        status="running",
        current_probe="spawn_process",
        last_completed_probe="-",
        completed_checks=0,
        current_check="-",
        last_completed_check="-",
        total_elapsed_ms=0,
    )

    with ThreadPoolExecutor(max_workers=effective_workers) as executor:
        for script_rel in uncached_scripts:
            script_futures[script_rel] = executor.submit(run_check_timed, script_rel)

        for index, (name, scripts) in enumerate(CHECKS, start=1):
            elapsed_ms = int((time.perf_counter() - started) * 1000)
            write_progress(
                status="running",
                current_probe=name,
                last_completed_probe=last_completed,
                completed_checks=index - 1,
                current_check=name,
                last_completed_check=last_completed,
                total_elapsed_ms=elapsed_ms,
            )
            for script_rel in scripts:
                if script_cached.get(script_rel, False):
                    print(f"[age4-proof-transport-selftest] cache-hit check={name} script={script_rel}")
                    script_runs[script_rel] = {
                        "check": name,
                        "script": script_rel,
                        "cached": True,
                        "observed_elapsed_ms": 0,
                        "returncode": 0,
                    }
                    continue
                future = script_futures.get(script_rel)
                if future is None:
                    continue
                proc, observed_elapsed_ms = future.result()
                script_runs[script_rel] = {
                    "check": name,
                    "script": script_rel,
                    "cached": False,
                    "observed_elapsed_ms": int(observed_elapsed_ms),
                    "returncode": int(proc.returncode),
                }
                if proc.returncode != 0:
                    stderr_text = decode_stderr(proc)
                    write_progress(
                        status="failed",
                        current_probe=name,
                        last_completed_probe=last_completed,
                        completed_checks=index - 1,
                        current_check=name,
                        last_completed_check=last_completed,
                        failed_check=name,
                        total_elapsed_ms=int((time.perf_counter() - started) * 1000),
                    )
                    write_json_atomic(
                        report_out,
                        build_report_payload(
                            overall_ok=False,
                            started=started,
                            failed_check=name,
                            requested_workers=requested_workers,
                            effective_workers=effective_workers,
                            dynamic_worker_mode=dynamic_worker_mode,
                            dynamic_worker_source_report=dynamic_worker_source_report,
                            dynamic_worker_source_loaded=len(dynamic_worker_source_docs) > 0,
                            dynamic_worker_lookback=dynamic_worker_lookback,
                            dynamic_worker_source_reports_used=dynamic_worker_source_reports_used,
                            dynamic_worker_decisions=dynamic_worker_decisions,
                            script_runs=list(script_runs.values()),
                            failure_detail=clip(stderr_text),
                        ),
                    )
                    return fail(
                        f"{name}: rc={proc.returncode} stderr={clip(stderr_text)}"
                    )
                mark_script_ok(script_rel)
                script_cached[script_rel] = True
            last_completed = name

    elapsed_ms = int((time.perf_counter() - started) * 1000)
    write_progress(
        status="completed",
        current_probe="-",
        last_completed_probe=last_completed,
        completed_checks=len(CHECKS),
        current_check="-",
        last_completed_check=last_completed,
        total_elapsed_ms=elapsed_ms,
    )
    write_json_atomic(
        report_out,
        build_report_payload(
            overall_ok=True,
            started=started,
            failed_check="-",
            requested_workers=requested_workers,
            effective_workers=effective_workers,
            dynamic_worker_mode=dynamic_worker_mode,
            dynamic_worker_source_report=dynamic_worker_source_report,
            dynamic_worker_source_loaded=len(dynamic_worker_source_docs) > 0,
            dynamic_worker_lookback=dynamic_worker_lookback,
            dynamic_worker_source_reports_used=dynamic_worker_source_reports_used,
            dynamic_worker_decisions=dynamic_worker_decisions,
            script_runs=list(script_runs.values()),
            failure_detail="-",
        ),
    )
    mark_script_ok(SELF_SCRIPT_PATH)
    print(
        "[age4-proof-transport-selftest] "
        f"completed_checks={len(CHECKS)} total_checks={len(CHECKS)} "
        f"last_completed_probe={last_completed} checks_text={CHECKS_TEXT}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
