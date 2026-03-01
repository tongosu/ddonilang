#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path


def fail(detail: str) -> int:
    print(f"check=runtime_fallback_policy detail={detail}")
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Seamgrim runtime fallback policy check")
    parser.add_argument(
        "--metrics",
        default="build/reports/seamgrim_runtime_fallback_metrics.detjson",
        help="runtime fallback metrics report path",
    )
    parser.add_argument(
        "--max-ratio",
        type=float,
        default=0.2,
        help="maximum allowed fallback ratio (default: 0.2)",
    )
    args = parser.parse_args()

    metrics_path = Path(args.metrics)
    if not metrics_path.exists():
        return fail(f"metrics_missing:{metrics_path.as_posix()}")
    try:
        payload = json.loads(metrics_path.read_text(encoding="utf-8"))
    except Exception as exc:
        return fail(f"metrics_parse_failed:{exc}")

    if not isinstance(payload, dict):
        return fail("metrics_invalid_payload")
    ratio_raw = payload.get("fallback_ratio")
    total_raw = payload.get("total")
    fallback_raw = payload.get("fallback_count")
    native_raw = payload.get("native_count")
    try:
        ratio = float(ratio_raw)
        total = int(total_raw)
        fallback_count = int(fallback_raw)
        native_count = int(native_raw)
    except Exception:
        return fail("metrics_required_fields_invalid")

    max_ratio = float(args.max_ratio)
    if ratio > max_ratio:
        return fail(
            f"ratio_exceeds:max={max_ratio:.3f}:ratio={ratio:.3f}:"
            f"fallback={fallback_count}:native={native_count}:total={total}"
        )

    print(
        "[runtime-fallback-policy] "
        f"status=pass max_ratio={max_ratio:.3f} ratio={ratio:.3f} "
        f"fallback={fallback_count} native={native_count} total={total}"
    )
    print("seamgrim runtime fallback policy check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
