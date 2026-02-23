#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path


def clip(text: str, limit: int = 180) -> str:
    normalized = " ".join(str(text).split())
    if not normalized:
        return "-"
    if len(normalized) <= limit:
        return normalized
    return normalized[: limit - 3] + "..."


def run_step(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run sync-readiness CI checks (pipeline flags/selftests/sanity/aggregate) in one command"
    )
    parser.add_argument("--report-dir", default="build/reports", help="report directory")
    parser.add_argument("--report-prefix", default="dev_sync_readiness", help="report prefix for aggregate gate")
    parser.add_argument("--json-out", default="", help="optional path for sync-readiness report json")
    parser.add_argument(
        "--skip-aggregate",
        action="store_true",
        help="skip aggregate gate run (quick mode)",
    )
    args = parser.parse_args()

    py = sys.executable
    report_dir = Path(args.report_dir)
    report_dir.mkdir(parents=True, exist_ok=True)
    prefix = args.report_prefix.strip() or "dev_sync_readiness"
    sanity_json = report_dir / f"{prefix}.ci_sanity_gate.detjson"

    steps: list[tuple[str, list[str]]] = [
        ("pipeline_emit_flags_check", [py, "tests/run_ci_pipeline_emit_flags_check.py"]),
        ("pipeline_emit_flags_selftest", [py, "tests/run_ci_pipeline_emit_flags_check_selftest.py"]),
        ("sanity_gate_diagnostics_check", [py, "tests/run_ci_sanity_gate_diagnostics_check.py"]),
        ("sanity_gate", [py, "tests/run_ci_sanity_gate.py", "--json-out", str(sanity_json)]),
    ]
    if not args.skip_aggregate:
        steps.append(
            (
                "aggregate_gate",
                [
                    py,
                    "tests/run_ci_aggregate_gate.py",
                    "--report-dir",
                    str(report_dir),
                    "--report-prefix",
                    prefix,
                    "--skip-core-tests",
                    "--fast-fail",
                    "--backup-hygiene",
                    "--clean-prefixed-reports",
                    "--quiet-success-logs",
                    "--compact-step-logs",
                    "--step-log-dir",
                    str(report_dir),
                    "--step-log-failed-only",
                    "--checklist-skip-seed-cli",
                    "--checklist-skip-ui-common",
                ],
            )
        )

    rows: list[dict[str, object]] = []
    all_ok = True
    started = datetime.now(timezone.utc).isoformat()
    for name, cmd in steps:
        tick = time.perf_counter()
        proc = run_step(cmd)
        elapsed_ms = int((time.perf_counter() - tick) * 1000)
        stdout = proc.stdout or ""
        stderr = proc.stderr or ""
        if stdout.strip():
            print(stdout, end="")
        if stderr.strip():
            print(stderr, end="", file=sys.stderr)
        ok = proc.returncode == 0
        rows.append(
            {
                "name": name,
                "ok": ok,
                "returncode": int(proc.returncode),
                "elapsed_ms": elapsed_ms,
                "cmd": cmd,
                "stdout_head": clip(stdout, 220),
                "stderr_head": clip(stderr, 220),
            }
        )
        if not ok:
            all_ok = False
            break

    total_elapsed_ms = sum(int(row.get("elapsed_ms", 0)) for row in rows)
    payload = {
        "schema": "ddn.ci.sync_readiness.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "started_at_utc": started,
        "ok": all_ok,
        "report_dir": str(report_dir),
        "report_prefix": prefix,
        "skip_aggregate": bool(args.skip_aggregate),
        "steps": rows,
        "steps_count": len(rows),
        "total_elapsed_ms": total_elapsed_ms,
    }

    out_path = Path(args.json_out) if args.json_out.strip() else (report_dir / f"{prefix}.ci_sync_readiness.detjson")
    write_json(out_path, payload)
    status = "pass" if all_ok else "fail"
    print(
        f'ci_sync_readiness_status={status} ok={1 if all_ok else 0} '
        f'steps={len(rows)} total_elapsed_ms={total_elapsed_ms} report="{out_path}"'
    )
    return 0 if all_ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
