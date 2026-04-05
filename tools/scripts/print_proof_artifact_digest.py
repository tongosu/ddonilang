#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def ensure_utf8_stdio() -> None:
    for stream_name in ("stdout", "stderr"):
        stream = getattr(sys, stream_name, None)
        if stream is None:
            continue
        reconfigure = getattr(stream, "reconfigure", None)
        if callable(reconfigure):
            try:
                reconfigure(encoding="utf-8")
            except Exception:
                pass


def load_payload(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def format_counter(rows: object, key: str) -> str:
    if not isinstance(rows, list):
        return "-"
    parts: list[str] = []
    for row in rows:
        if not isinstance(row, dict):
            continue
        label = str(row.get(key, "")).strip()
        count = int(row.get("count", 0) or 0)
        if not label:
            continue
        parts.append(f"{label}:{count}")
    return ",".join(parts) if parts else "-"


def main() -> int:
    ensure_utf8_stdio()
    parser = argparse.ArgumentParser(description="Print digest from ddn.proof_artifact_summary.v1")
    parser.add_argument("report", help="path to proof artifact summary detjson")
    parser.add_argument("--top", type=int, default=8, help="max failure digest lines")
    parser.add_argument("--only-failed", action="store_true", help="print digest only when unverified_count>0")
    args = parser.parse_args()

    path = Path(args.report)
    payload = load_payload(path)
    if payload is None:
        print(f"[proof-artifact] report missing_or_invalid: {path}")
        return 0

    artifact_count = int(payload.get("artifact_count", 0) or 0)
    verified_count = int(payload.get("verified_count", 0) or 0)
    unverified_count = int(payload.get("unverified_count", 0) or 0)
    invalid_count = int(payload.get("invalid_artifact_count", 0) or 0)
    runtime_error_artifact_count = int(payload.get("runtime_error_artifact_count", 0) or 0)
    runtime_error_state_hash_present_count = int(payload.get("runtime_error_state_hash_present_count", 0) or 0)
    runtime_error_text = format_counter(payload.get("runtime_error_counts"), "code")
    proof_block_text = format_counter(payload.get("proof_block_result_counts"), "result")
    solver_runtime_text = format_counter(payload.get("solver_runtime_counts"), "operation")
    summary_hash = str(payload.get("summary_hash", "")).strip() or "-"
    print(
        f"[proof-artifact] artifacts={artifact_count} verified={verified_count} unverified={unverified_count} "
        f"invalid={invalid_count} runtime_errors={runtime_error_text} "
        f"runtime_error_statehash={runtime_error_state_hash_present_count}/{runtime_error_artifact_count} "
        f"proof_blocks={proof_block_text} solver_runtime={solver_runtime_text} "
        f"summary_hash={summary_hash} report={path}"
    )
    if args.only_failed and unverified_count == 0:
        return 0

    digest = payload.get("failure_digest")
    if isinstance(digest, list) and digest:
        for line in digest[: max(1, int(args.top))]:
            print(f" - {line}")
    else:
        print(" - failure_digest=(none)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
