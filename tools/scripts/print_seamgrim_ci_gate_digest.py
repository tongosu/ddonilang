#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys


def _configure_stdout() -> None:
    reconfigure = getattr(sys.stdout, "reconfigure", None)
    if callable(reconfigure):
        try:
            reconfigure(errors="backslashreplace")
        except OSError:
            pass
        except ValueError:
            pass


def main() -> int:
    _configure_stdout()
    parser = argparse.ArgumentParser(description="Print seamgrim CI gate digest from report json")
    parser.add_argument("report", help="path to seamgrim_ci_gate_report.json")
    parser.add_argument("--only-failed", action="store_true", help="print only failed steps")
    parser.add_argument(
        "--top-slowest",
        type=int,
        default=0,
        help="print top N slowest steps (0 keeps report order)",
    )
    parser.add_argument("--max-drilldown", type=int, default=3, help="max diagnostics per failed step")
    args = parser.parse_args()

    report_path = Path(args.report)
    if not report_path.exists():
        print(f"[seamgrim-ci] report not found: {report_path}")
        return 0

    payload = json.loads(report_path.read_text(encoding="utf-8"))
    steps = payload.get("steps", [])
    if not isinstance(steps, list):
        print(f"[seamgrim-ci] invalid steps payload: {type(steps).__name__}")
        return 1
    ok = bool(payload.get("ok", False))
    print(f"[seamgrim-ci] ok={int(ok)} steps={len(steps)} report={report_path}")

    ordered_steps = steps
    if int(args.top_slowest) > 0:
        ordered_steps = sorted(
            steps,
            key=lambda row: int(row.get("elapsed_ms", 0)) if isinstance(row, dict) else 0,
            reverse=True,
        )[: max(1, int(args.top_slowest))]
        print(f"[seamgrim-ci] top_slowest={max(1, int(args.top_slowest))}")

    for row in ordered_steps:
        if not isinstance(row, dict):
            continue
        row_ok = bool(row.get("ok"))
        if args.only_failed and row_ok:
            continue
        name = row.get("name", "-")
        elapsed = row.get("elapsed_ms", 0)
        code = row.get("returncode", 0)
        print(f" - {name}: ok={int(row_ok)} code={code} elapsed_ms={elapsed}")
        if not row_ok:
            diagnostics = row.get("diagnostics")
            if isinstance(diagnostics, list) and diagnostics:
                limit = max(1, int(args.max_drilldown))
                shown = 0
                for diag in diagnostics:
                    if shown >= limit or not isinstance(diag, dict):
                        break
                    kind = str(diag.get("kind") or "generic_error")
                    target = str(diag.get("target") or "-")
                    detail = str(diag.get("detail") or "").strip()
                    print(f"   drilldown[{shown + 1}] kind={kind} target={target}")
                    if detail:
                        print(f"     {detail.splitlines()[0]}")
                    shown += 1
            stderr = str(row.get("stderr") or "").strip()
            stdout = str(row.get("stdout") or "").strip()
            detail = stderr or stdout
            if detail:
                first = detail.splitlines()[0]
                print(f"   detail: {first}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
