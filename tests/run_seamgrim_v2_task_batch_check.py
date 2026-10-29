#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path


TASK_ORDER: tuple[str, ...] = ("T1", "T2", "T3", "T4", "T5")

TASK_CHECKS: dict[str, tuple[tuple[str, tuple[str, ...]], ...]] = {
    "T1": (
        ("seamgrim_ui_age3_gate", ("tests/run_seamgrim_ui_age3_gate.py",)),
    ),
    "T2": (
        ("seamgrim_observe_output_contract_check", ("tests/run_seamgrim_observe_output_contract_check.py",)),
        ("seamgrim_runtime_view_source_strict_check", ("tests/run_seamgrim_runtime_view_source_strict_check.py",)),
        ("seamgrim_run_legacy_autofix_check", ("tests/run_seamgrim_run_legacy_autofix_check.py",)),
        ("seamgrim_playground_smoke_check", ("tests/run_seamgrim_playground_smoke_check.py",)),
    ),
    "T3": (
        ("seamgrim_browse_selection_flow_check", ("tests/run_seamgrim_browse_selection_flow_check.py",)),
        ("seamgrim_playground_smoke_check", ("tests/run_seamgrim_playground_smoke_check.py",)),
        ("seamgrim_view_only_state_hash_invariant_check", ("tests/run_seamgrim_view_only_state_hash_invariant_check.py",)),
    ),
    "T4": (
        ("seamgrim_browse_selection_flow_check", ("tests/run_seamgrim_browse_selection_flow_check.py",)),
        ("seamgrim_view_only_state_hash_invariant_check", ("tests/run_seamgrim_view_only_state_hash_invariant_check.py",)),
    ),
    "T5": (
        ("seamgrim_motion_projectile_fallback_check", ("tests/run_seamgrim_motion_projectile_fallback_check.py",)),
        ("seamgrim_rewrite_overlay_quality_check", ("tests/run_seamgrim_rewrite_overlay_quality_check.py",)),
        ("seamgrim_new_grammar_no_legacy_control_meta_check", ("tests/run_seamgrim_new_grammar_no_legacy_control_meta_check.py",)),
        ("seamgrim_lesson_path_fallback_check", ("tests/run_seamgrim_lesson_path_fallback_check.py",)),
        ("seamgrim_lesson_migration_lint_check", ("tests/run_seamgrim_lesson_migration_lint_check.py",)),
        ("seamgrim_lesson_migration_autofix_check", ("tests/run_seamgrim_lesson_migration_autofix_check.py",)),
    ),
}


def now_utc_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def write_json(path: str, payload: dict[str, object]) -> None:
    if not path:
        return
    target = Path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8")


def select_tasks(only_task: str, from_task: str) -> list[str]:
    if only_task and from_task:
        raise ValueError("--only-task and --from-task cannot be used together")
    if only_task:
        if only_task not in TASK_ORDER:
            raise ValueError(f"unknown --only-task: {only_task}")
        return [only_task]
    if from_task:
        if from_task not in TASK_ORDER:
            raise ValueError(f"unknown --from-task: {from_task}")
        start = TASK_ORDER.index(from_task)
        return list(TASK_ORDER[start:])
    return list(TASK_ORDER)


def run_check(root: Path, name: str, script: tuple[str, ...]) -> tuple[int, float]:
    cmd = [sys.executable, *script]
    started = time.perf_counter()
    proc = subprocess.run(
        cmd,
        cwd=str(root),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    if proc.stdout.strip():
        emit_text_safely(proc.stdout.rstrip() + "\n", sys.stdout)
    if proc.stderr.strip():
        emit_text_safely(proc.stderr.rstrip() + "\n", sys.stderr)
    if proc.returncode == 0:
        print(f"[{name}] ok ({int(round(elapsed_ms))}ms)")
    else:
        print(f"[{name}] fail rc={proc.returncode} ({int(round(elapsed_ms))}ms)")
    return proc.returncode, elapsed_ms


def emit_text_safely(text: str, stream) -> None:
    if not text:
        return
    try:
        stream.write(text)
    except UnicodeEncodeError:
        encoding = getattr(stream, "encoding", None) or "utf-8"
        data = text.encode(encoding, errors="replace")
        buffer = getattr(stream, "buffer", None)
        if buffer is not None:
            buffer.write(data)
        else:
            stream.write(data.decode(encoding, errors="replace"))
    stream.flush()


def main() -> int:
    parser = argparse.ArgumentParser(description="Run Seamgrim V2 T1~T5 task batch checks")
    parser.add_argument("--only-task", default="", help="Run only one task (T1~T5)")
    parser.add_argument("--from-task", default="", help="Run from a task to T5 (T1~T5)")
    parser.add_argument("--continue-on-fail", action="store_true", help="Continue even when a check fails")
    parser.add_argument("--json-out", default="", help="Optional report path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    try:
        tasks = select_tasks(str(args.only_task).strip(), str(args.from_task).strip())
    except ValueError as exc:
        print(str(exc), file=sys.stderr)
        return 2

    rows: list[dict[str, object]] = []
    failed = 0
    started_all = time.perf_counter()

    for task in tasks:
        checks = TASK_CHECKS.get(task, ())
        print(f"seamgrim_v2_task_current={task}")
        task_checks: list[dict[str, object]] = []
        task_failed = 0
        for check_name, script in checks:
            rc, elapsed_ms = run_check(root, check_name, script)
            check_row = {
                "name": check_name,
                "script": list(script),
                "ok": rc == 0,
                "returncode": int(rc),
                "elapsed_ms": int(round(elapsed_ms)),
            }
            task_checks.append(check_row)
            if rc != 0:
                failed += 1
                task_failed += 1
                if not args.continue_on_fail:
                    rows.append(
                        {
                            "task": task,
                            "ok": False,
                            "failed_checks": task_failed,
                            "checks": task_checks,
                        }
                    )
                    payload = {
                        "schema": "ddn.seamgrim_v2_task_batch_check.v1",
                        "generated_at_utc": now_utc_iso(),
                        "status": "fail",
                        "code": "E_SEAMGRIM_V2_TASK_CHECK_FAIL",
                        "task": task,
                        "failed_checks": failed,
                        "tasks": rows,
                        "total_elapsed_ms": int(round((time.perf_counter() - started_all) * 1000.0)),
                    }
                    write_json(str(args.json_out).strip(), payload)
                    print(
                        f"check=seamgrim_v2_task_batch detail=fail:task={task}:failed_checks={failed}:continue_on_fail=0"
                    )
                    return 1
        task_ok = task_failed == 0
        rows.append(
            {
                "task": task,
                "ok": task_ok,
                "failed_checks": task_failed,
                "checks": task_checks,
            }
        )
        print(
            f"seamgrim_v2_task_done={task} status={'ok' if task_ok else 'fail'} checks={len(task_checks)} failed={task_failed}"
        )

    status = "pass" if failed == 0 else "fail"
    code = "OK" if failed == 0 else "E_SEAMGRIM_V2_TASK_CHECK_FAIL"
    payload = {
        "schema": "ddn.seamgrim_v2_task_batch_check.v1",
        "generated_at_utc": now_utc_iso(),
        "status": status,
        "code": code,
        "task": tasks[-1] if tasks else "-",
        "failed_checks": failed,
        "tasks": rows,
        "total_elapsed_ms": int(round((time.perf_counter() - started_all) * 1000.0)),
    }
    write_json(str(args.json_out).strip(), payload)
    if failed:
        print(f"check=seamgrim_v2_task_batch detail=fail:failed_checks={failed}:tasks={','.join(tasks)}")
        return 1

    print(f"check=seamgrim_v2_task_batch detail=ok:tasks={','.join(tasks)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
