#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path


def _to_int(value: object) -> int:
    try:
        return int(value)  # type: ignore[arg-type]
    except (TypeError, ValueError):
        return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Print slowest steps from a ddn.ci.sanity_gate.v1 report",
    )
    parser.add_argument("--json", required=True, help="path to ci sanity json report")
    parser.add_argument("--top", type=int, default=15, help="number of rows to print")
    parser.add_argument("--min-ms", type=int, default=0, help="minimum elapsed ms to include")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report_path = Path(args.json)
    if not report_path.exists():
        print(f"[ci-sanity-slowest] missing report: {report_path.as_posix()}")
        return 1

    try:
        payload = json.loads(report_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        print(f"[ci-sanity-slowest] invalid json: {exc}")
        return 1

    schema = str(payload.get("schema", "")).strip()
    if schema != "ddn.ci.sanity_gate.v1":
        print(f"[ci-sanity-slowest] unsupported schema: {schema or '-'}")
        return 1

    rows = payload.get("steps")
    if not isinstance(rows, list):
        print("[ci-sanity-slowest] steps missing")
        return 1

    normalized = []
    for row in rows:
        if not isinstance(row, dict):
            continue
        step_name = str(row.get("step", "")).strip() or "-"
        elapsed_ms = _to_int(row.get("elapsed_ms"))
        rc = _to_int(row.get("returncode"))
        code = str(row.get("code", "")).strip() or "-"
        status = str(row.get("status", "")).strip() or "-"
        normalized.append(
            {
                "step": step_name,
                "elapsed_ms": elapsed_ms,
                "returncode": rc,
                "code": code,
                "status": status,
            }
        )

    filtered = [row for row in normalized if row["elapsed_ms"] >= max(0, args.min_ms)]
    filtered.sort(key=lambda row: row["elapsed_ms"], reverse=True)
    topn = max(1, args.top)
    selected = filtered[:topn]

    total_ms = _to_int(payload.get("total_elapsed_ms"))
    print(
        "[ci-sanity-slowest] "
        f"steps={len(rows)} total_elapsed_ms={total_ms} "
        f"min_ms={max(0, args.min_ms)} top={topn}"
    )
    for idx, row in enumerate(selected, start=1):
        print(
            f"{idx:02d}. step={row['step']} "
            f"elapsed_ms={row['elapsed_ms']} "
            f"status={row['status']} code={row['code']} returncode={row['returncode']}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

