#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path


def load_payload(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def clip(text: str, limit: int = 160) -> str:
    normalized = " ".join(text.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def resolve_path(base_report: Path, raw: str) -> Path:
    candidate = Path(raw)
    if candidate.is_absolute():
        return candidate
    # 기본은 현재 작업 디렉터리 기준, 없으면 aggregate report 기준으로 재해석.
    if candidate.exists():
        return candidate
    if candidate.parent != Path("."):
        return candidate.resolve()
    return (base_report.parent / candidate).resolve()


def main() -> int:
    parser = argparse.ArgumentParser(description="Print top digest lines from ci_aggregate_report.detjson")
    parser.add_argument("report", help="path to ci_aggregate_report.detjson")
    parser.add_argument("--top", type=int, default=1, help="number of digest lines to print")
    parser.add_argument("--only-failed", action="store_true", help="print digest only when overall_ok=false")
    parser.add_argument(
        "--show-steps",
        action="store_true",
        help="print failed step names from gate index when available",
    )
    args = parser.parse_args()

    path = Path(args.report)
    payload = load_payload(path)
    if payload is None:
        print(f"[ci-aggregate] report missing_or_invalid: {path}")
        return 0

    overall_ok = bool(payload.get("overall_ok", False))
    digest_raw = payload.get("failure_digest")
    digest = [str(item) for item in digest_raw] if isinstance(digest_raw, list) else []
    top = max(1, int(args.top))

    seamgrim = payload.get("seamgrim") if isinstance(payload.get("seamgrim"), dict) else {}
    age3 = payload.get("age3") if isinstance(payload.get("age3"), dict) else {}
    age4 = payload.get("age4") if isinstance(payload.get("age4"), dict) else {}
    oi = payload.get("oi405_406") if isinstance(payload.get("oi405_406"), dict) else {}
    seamgrim_failed = len(seamgrim.get("failed_steps", [])) if isinstance(seamgrim, dict) else 0
    age3_failed = len(age3.get("failed_criteria", [])) if isinstance(age3, dict) else 0
    age4_failed = len(age4.get("failed_criteria", [])) if isinstance(age4, dict) else 0
    oi_failed = len(oi.get("failed_packs", [])) if isinstance(oi, dict) else 0
    print(
        f"[ci-aggregate] overall_ok={int(overall_ok)} seamgrim_failed={seamgrim_failed} "
        f"age3_failed={age3_failed} age4_failed={age4_failed} "
        f"oi405_406_failed={oi_failed} report={path}"
    )
    gate_index_raw = payload.get("gate_index_report_path")
    age3_status_raw = payload.get("age3_status_report_path")
    age3_status_line_raw = payload.get("age3_status_line")
    age3_status_line_path_raw = payload.get("age3_status_line_path")
    age3_badge_path_raw = payload.get("age3_badge_path")
    should_print_steps = bool(args.show_steps and (not args.only_failed or not overall_ok))
    gate_steps: list[dict] = []
    if isinstance(gate_index_raw, str) and gate_index_raw.strip():
        gate_index_path = resolve_path(path, gate_index_raw.strip())
        index_doc = load_payload(gate_index_path)
        index_ok = bool(index_doc.get("overall_ok", False)) if isinstance(index_doc, dict) else False
        if isinstance(index_doc, dict) and isinstance(index_doc.get("steps"), list):
            gate_steps = [row for row in index_doc.get("steps", []) if isinstance(row, dict)]
        step_count = len(gate_steps)
        print(
            f"[ci-aggregate] gate_index_path={gate_index_path} "
            f"exists={int(gate_index_path.exists())} index_ok={int(index_ok)} step_count={step_count}"
        )
        if should_print_steps:
            failed_rows = [row for row in gate_steps if not bool(row.get("ok", False))]
            if failed_rows:
                names = ", ".join(str(row.get("name", "-")) for row in failed_rows[:8])
                if len(failed_rows) > 8:
                    names = f"{names}, ..."
                print(f"[ci-aggregate] failed_steps={names}")
            elif gate_steps:
                print("[ci-aggregate] failed_steps=(none)")
            else:
                print("[ci-aggregate] failed_steps=(index_has_no_steps)")
    elif should_print_steps:
        print("[ci-aggregate] failed_steps=(gate_index_missing)")

    if isinstance(age3_status_raw, str) and age3_status_raw.strip():
        age3_status_path = resolve_path(path, age3_status_raw.strip())
        age3_status_doc = load_payload(age3_status_path)
        status_value = (
            str(age3_status_doc.get("status", "-"))
            if isinstance(age3_status_doc, dict)
            else "-"
        )
        status_ok = bool(age3_status_doc.get("overall_ok", False)) if isinstance(age3_status_doc, dict) else False
        print(
            f"[ci-aggregate] age3_status_path={age3_status_path} "
            f"exists={int(age3_status_path.exists())} status={status_value} ok={int(status_ok)}"
        )
    if isinstance(age3_status_line_path_raw, str) and age3_status_line_path_raw.strip():
        age3_status_line_path = resolve_path(path, age3_status_line_path_raw.strip())
        print(
            f"[ci-aggregate] age3_status_line_path={age3_status_line_path} "
            f"exists={int(age3_status_line_path.exists())}"
        )
    if isinstance(age3_status_line_raw, str) and age3_status_line_raw.strip():
        print(f"[ci-aggregate] age3_status_line={clip(age3_status_line_raw, 200)}")
    if isinstance(age3_badge_path_raw, str) and age3_badge_path_raw.strip():
        age3_badge_path = resolve_path(path, age3_badge_path_raw.strip())
        age3_badge_doc = load_payload(age3_badge_path)
        badge_status = str(age3_badge_doc.get("status", "-")) if isinstance(age3_badge_doc, dict) else "-"
        badge_color = str(age3_badge_doc.get("color", "-")) if isinstance(age3_badge_doc, dict) else "-"
        print(
            f"[ci-aggregate] age3_badge_path={age3_badge_path} "
            f"exists={int(age3_badge_path.exists())} status={badge_status} color={badge_color}"
        )

    if args.only_failed and overall_ok:
        return 0

    for idx, line in enumerate(digest[:top], 1):
        print(f" - top{idx}: {clip(line)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
