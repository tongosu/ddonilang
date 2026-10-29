#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

def _sanitize_for_console(text: str) -> str:
    return str(text or "").replace("\ufffd", "?")


def _safe_print(text: str) -> None:
    message = _sanitize_for_console(text)
    try:
        print(message)
    except UnicodeEncodeError:
        enc = getattr(sys.stdout, "encoding", None) or "utf-8"
        sys.stdout.write(message.encode(enc, errors="replace").decode(enc, errors="replace") + "\n")


def fail(detail: str) -> int:
    _safe_print(f"check=runtime_fallback_metrics detail={detail}")
    return 1


def load_seed_report_rows(report_path: Path) -> list[dict[str, str]]:
    doc = json.loads(report_path.read_text(encoding="utf-8"))
    if doc.get("schema") != "ddn.seamgrim.seed_runtime_visual_pack_report.v1":
        raise ValueError(f"seed_report_schema_mismatch:{doc.get('schema')}")
    if doc.get("ok") is not True:
        raise ValueError("seed_report_not_ok")
    cases = doc.get("cases")
    if not isinstance(cases, list) or not cases:
        raise ValueError("seed_report_cases_missing")
    out: list[dict[str, str]] = []
    for row in cases:
        if not isinstance(row, dict):
            continue
        out.append(
            {
                "seed_id": str(row.get("id", "")).strip(),
                "y_key": str(row.get("y", "")).strip(),
                "points": str(row.get("points", "")),
                "mode": str(row.get("mode", "")).strip(),
                "source": str(row.get("source", "")).strip(),
                "title": str(row.get("fallback_title", "")).strip(),
            }
        )
    return [row for row in out if row.get("seed_id")]


def main() -> int:
    parser = argparse.ArgumentParser(description="Seamgrim runtime fallback metrics check")
    parser.add_argument(
        "--json-out",
        default="build/reports/seamgrim_runtime_fallback_metrics.detjson",
        help="metrics report output path",
    )
    parser.add_argument(
        "--seed-report",
        default="build/reports/seamgrim_seed_runtime_visual_pack_report.detjson",
        help="seed runtime visual pack report path",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    seed_report_path = root / str(args.seed_report)
    if not seed_report_path.exists():
        runner = root / "tests" / "run_seamgrim_seed_runtime_visual_pack_check.py"
        if not runner.exists():
            return fail(f"runner_missing:{runner.as_posix()}")
        proc = subprocess.run(
            [sys.executable, str(runner), "--json-out", str(seed_report_path)],
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if proc.returncode != 0:
            detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or f"returncode={proc.returncode}"
            return fail(f"seed_runtime_visual_pack_failed:{detail}")

    try:
        rows = load_seed_report_rows(seed_report_path)
    except Exception as exc:
        return fail(f"seed_report_invalid:{exc}")
    if not rows:
        return fail("seed_report_rows_empty")

    mixed_rows = [
        row
        for row in rows
        if str(row.get("mode", "")).strip().lower() == "native"
        and "fallback" in str(row.get("source", row.get("title", ""))).strip().lower()
    ]
    if mixed_rows:
        top = mixed_rows[0]
        return fail(
            "native_fallback_label_mixed:"
            f"{top.get('seed_id', '-')}:"
            f"{top.get('source', top.get('title', '-'))}"
        )

    total = len(rows)
    fallback_rows = [
        row
        for row in rows
        if (
            str(row.get("mode", "")).strip().lower() == "fallback"
            or (
                not str(row.get("mode", "")).strip()
                and "fallback" in str(row.get("source", row.get("title", ""))).strip().lower()
            )
        )
    ]
    fallback_count = len(fallback_rows)
    native_count = total - fallback_count
    ratio = fallback_count / total if total > 0 else 0.0

    report = {
        "schema": "seamgrim.runtime_fallback_metrics.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": True,
        "total": total,
        "fallback_count": fallback_count,
        "native_count": native_count,
        "fallback_ratio": ratio,
        "rows": rows,
    }
    out_path = Path(args.json_out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    _safe_print(
        "[runtime-fallback] "
        f"total={total} fallback={fallback_count} native={native_count} ratio={ratio:.3f} "
        f"report={out_path.as_posix()}"
    )
    _safe_print("seamgrim runtime fallback metrics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
