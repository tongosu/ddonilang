#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "bogae_graph_prefix_v1"


def sort_json(value):
    if isinstance(value, list):
        return [sort_json(item) for item in value]
    if isinstance(value, dict):
        return {key: sort_json(value[key]) for key in sorted(value)}
    return value


def main() -> int:
    proc = subprocess.run(
        ["node", "tests/seamgrim_bogae_graph_prefix_runner.mjs"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=120,
    )
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip()
        print(f"[bogae-graph-prefix] fail: {detail}", file=sys.stderr)
        return 1
    try:
        actual = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        print(f"[bogae-graph-prefix] fail: invalid runner json: {exc}", file=sys.stderr)
        return 1
    expected = json.loads((PACK / "expected" / "bogae_graph_prefix.detjson").read_text(encoding="utf-8"))
    if sort_json(actual) != sort_json(expected):
        print("[bogae-graph-prefix] fail: expected mismatch", file=sys.stderr)
        print(json.dumps(actual, ensure_ascii=False, indent=2), file=sys.stderr)
        return 1
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    by_id = {case.get("id"): case for case in actual.get("cases", []) if isinstance(case, dict)}
    prefix_case = by_id.get("prefix_value_json_to_graph") or {}
    if prefix_case.get("series_count") != contract.get("expected_series_count"):
        print("[bogae-graph-prefix] fail: series count contract mismatch", file=sys.stderr)
        return 1
    if prefix_case.get("primary_points") != contract.get("expected_primary_points"):
        print("[bogae-graph-prefix] fail: primary points contract mismatch", file=sys.stderr)
        return 1
    if prefix_case.get("secondary_points") != contract.get("expected_secondary_points"):
        print("[bogae-graph-prefix] fail: secondary points contract mismatch", file=sys.stderr)
        return 1
    print("[bogae-graph-prefix] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
