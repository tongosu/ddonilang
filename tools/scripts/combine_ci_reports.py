#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def load_text(path: Path) -> str | None:
    if not path.exists():
        return None
    try:
        return path.read_text(encoding="utf-8").strip()
    except Exception:
        return None


def clip(text: str, limit: int = 140) -> str:
    normalized = " ".join(text.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def seamgrim_summary(doc: dict | None, path: Path) -> dict[str, object]:
    if not isinstance(doc, dict):
        return {
            "ok": False,
            "report_path": str(path),
            "error": "missing_or_invalid_report",
            "failed_steps": [],
            "failure_digest": [f"seamgrim report missing_or_invalid: {path}"],
        }
    steps = doc.get("steps")
    failed_steps: list[str] = []
    if isinstance(steps, list):
        for row in steps:
            if isinstance(row, dict) and not bool(row.get("ok", False)):
                failed_steps.append(str(row.get("name", "-")))
    digest = doc.get("failure_digest")
    failure_digest = [str(item) for item in digest] if isinstance(digest, list) else []
    if not failure_digest and failed_steps:
        failure_digest = [f"step={name}" for name in failed_steps]
    return {
        "ok": bool(doc.get("ok", False)),
        "report_path": str(path),
        "schema": doc.get("schema"),
        "failed_steps": failed_steps,
        "failure_digest": failure_digest,
        "elapsed_total_ms": int(doc.get("elapsed_total_ms", 0)),
    }


def oi_summary(doc: dict | None, path: Path) -> dict[str, object]:
    if not isinstance(doc, dict):
        return {
            "ok": False,
            "report_path": str(path),
            "error": "missing_or_invalid_report",
            "failed_packs": [],
            "failure_digest": [f"oi report missing_or_invalid: {path}"],
        }
    packs = doc.get("packs")
    failed_packs: list[str] = []
    if isinstance(packs, list):
        for row in packs:
            if isinstance(row, dict) and not bool(row.get("ok", False)):
                failed_packs.append(str(row.get("pack", "-")))
    digest = doc.get("failure_digest")
    failure_digest = [str(item) for item in digest] if isinstance(digest, list) else []
    if not failure_digest and failed_packs:
        failure_digest = [f"pack={name}" for name in failed_packs]
    return {
        "ok": bool(doc.get("overall_ok", False)),
        "report_path": str(path),
        "schema": doc.get("schema"),
        "failed_packs": failed_packs,
        "failure_digest": failure_digest,
    }


def age3_summary(doc: dict | None, path: Path, require_age3: bool) -> dict[str, object]:
    if doc is None and not require_age3:
        return {
            "ok": True,
            "skipped": True,
            "report_path": str(path),
            "failed_criteria": [],
            "failure_digest": [],
        }
    if not isinstance(doc, dict):
        return {
            "ok": False,
            "report_path": str(path),
            "error": "missing_or_invalid_report",
            "failed_criteria": [],
            "failure_digest": [f"age3 report missing_or_invalid: {path}"],
        }
    criteria = doc.get("criteria")
    failed_criteria: list[str] = []
    if isinstance(criteria, list):
        for row in criteria:
            if isinstance(row, dict) and not bool(row.get("ok", False)):
                failed_criteria.append(str(row.get("name", "-")))
    digest = doc.get("failure_digest")
    failure_digest = [str(item) for item in digest] if isinstance(digest, list) else []
    if not failure_digest and failed_criteria:
        failure_digest = [f"criteria={name}" for name in failed_criteria]
    return {
        "ok": bool(doc.get("overall_ok", False)),
        "report_path": str(path),
        "schema": doc.get("schema"),
        "failed_criteria": failed_criteria,
        "failure_digest": failure_digest,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Combine seamgrim/oi close reports into one detjson report")
    parser.add_argument(
        "--seamgrim-report",
        default="build/reports/seamgrim_ci_gate_report.json",
        help="path to seamgrim ci gate report",
    )
    parser.add_argument(
        "--oi-report",
        default="build/reports/oi405_406_close_report.detjson",
        help="path to oi405/406 close report",
    )
    parser.add_argument(
        "--age3-report",
        default="build/reports/age3_close_report.detjson",
        help="path to age3 close report",
    )
    parser.add_argument(
        "--require-age3",
        action="store_true",
        help="require age3 close report to exist and pass",
    )
    parser.add_argument(
        "--age3-status",
        default="build/reports/age3_close_status.detjson",
        help="optional path to age3 close status json (link metadata)",
    )
    parser.add_argument(
        "--age3-status-line",
        default="build/reports/age3_close_status_line.txt",
        help="optional path to one-line age3 status text (link metadata)",
    )
    parser.add_argument(
        "--age3-badge",
        default="build/reports/age3_close_badge.detjson",
        help="optional path to age3 close badge json (link metadata)",
    )
    parser.add_argument(
        "--out",
        default="build/reports/ci_aggregate_report.detjson",
        help="output aggregate report path",
    )
    parser.add_argument(
        "--index-report-path",
        default="",
        help="optional aggregate gate index report path to embed as link metadata",
    )
    parser.add_argument("--print-summary", action="store_true", help="print aggregate summary")
    parser.add_argument("--fail-on-bad", action="store_true", help="return non-zero when any check failed")
    args = parser.parse_args()

    seamgrim_path = Path(args.seamgrim_report)
    age3_path = Path(args.age3_report)
    age3_status_path = Path(args.age3_status) if args.age3_status.strip() else None
    age3_status_line_path = Path(args.age3_status_line) if args.age3_status_line.strip() else None
    age3_badge_path = Path(args.age3_badge) if args.age3_badge.strip() else None
    oi_path = Path(args.oi_report)
    out_path = Path(args.out)
    index_report_path = Path(args.index_report_path) if args.index_report_path.strip() else None

    seamgrim = seamgrim_summary(load_json(seamgrim_path), seamgrim_path)
    age3 = age3_summary(load_json(age3_path), age3_path, bool(args.require_age3))
    oi = oi_summary(load_json(oi_path), oi_path)
    overall_ok = bool(seamgrim.get("ok", False)) and bool(age3.get("ok", False)) and bool(oi.get("ok", False))

    failure_digest: list[str] = []
    for item in seamgrim.get("failure_digest", []):
        failure_digest.append(f"seamgrim: {clip(str(item))}")
    for item in age3.get("failure_digest", []):
        failure_digest.append(f"age3: {clip(str(item))}")
    for item in oi.get("failure_digest", []):
        failure_digest.append(f"oi405_406: {clip(str(item))}")

    payload = {
        "schema": "ddn.ci.aggregate_report.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "overall_ok": overall_ok,
        "seamgrim": seamgrim,
        "age3": age3,
        "oi405_406": oi,
        "failure_digest": failure_digest[:16],
    }
    if age3_status_path is not None:
        payload["age3_status_report_path"] = str(age3_status_path)
        age3_status_doc = load_json(age3_status_path)
        payload["age3_status_ok"] = bool(age3_status_doc.get("overall_ok", False)) if isinstance(age3_status_doc, dict) else False
    if age3_status_line_path is not None:
        payload["age3_status_line_path"] = str(age3_status_line_path)
        age3_status_line = load_text(age3_status_line_path)
        payload["age3_status_line_exists"] = age3_status_line is not None
        payload["age3_status_line"] = age3_status_line or ""
    if age3_badge_path is not None:
        payload["age3_badge_path"] = str(age3_badge_path)
        age3_badge_doc = load_json(age3_badge_path)
        payload["age3_badge_exists"] = age3_badge_doc is not None
        payload["age3_badge_status"] = str(age3_badge_doc.get("status", "-")) if isinstance(age3_badge_doc, dict) else "-"
        payload["age3_badge_color"] = str(age3_badge_doc.get("color", "-")) if isinstance(age3_badge_doc, dict) else "-"
    if index_report_path is not None:
        payload["gate_index_report_path"] = str(index_report_path)

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    if args.print_summary:
        print(f"[ci-aggregate] overall_ok={int(overall_ok)} out={out_path}")
        print(
            f" - seamgrim: ok={int(bool(seamgrim.get('ok', False)))} "
            f"failed_steps={len(seamgrim.get('failed_steps', []))}"
        )
        print(
            f" - age3: ok={int(bool(age3.get('ok', False)))} "
            f"failed_criteria={len(age3.get('failed_criteria', []))}"
        )
        if age3_status_path is not None:
            print(
                f" - age3_status_path: {age3_status_path} "
                f"ok={int(bool(payload.get('age3_status_ok', False)))}"
            )
        if age3_status_line_path is not None:
            print(
                f" - age3_status_line_path: {age3_status_line_path} "
                f"exists={int(bool(payload.get('age3_status_line_exists', False)))}"
            )
            age3_status_line_print = str(payload.get("age3_status_line", "")).strip()
            if age3_status_line_print:
                print(f"   {clip(age3_status_line_print, 180)}")
        if age3_badge_path is not None:
            print(
                f" - age3_badge_path: {age3_badge_path} "
                f"exists={int(bool(payload.get('age3_badge_exists', False)))} "
                f"status={payload.get('age3_badge_status', '-')}"
            )
        print(
            f" - oi405_406: ok={int(bool(oi.get('ok', False)))} "
            f"failed_packs={len(oi.get('failed_packs', []))}"
        )
        if index_report_path is not None:
            print(f" - gate_index_path: {index_report_path}")
        for line in payload["failure_digest"][:6]:
            print(f"   {line}")

    if args.fail_on_bad and not overall_ok:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
