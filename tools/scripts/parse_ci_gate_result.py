#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path


EXPECTED_SCHEMA = "ddn.ci.gate_result.v1"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def compact_line(doc: dict) -> str:
    status = str(doc.get("status", "fail")).strip() or "fail"
    ok = 1 if bool(doc.get("ok", False)) else 0
    overall_ok = 1 if bool(doc.get("overall_ok", False)) else 0
    failed_steps = int(doc.get("failed_steps", -1))
    aggregate_status = str(doc.get("aggregate_status", "fail")).strip() or "fail"
    reason = str(doc.get("reason", "-")).strip() or "-"
    return (
        f"ci_gate_result_status={status} ok={ok} overall_ok={overall_ok} "
        f"failed_steps={failed_steps} aggregate_status={aggregate_status} reason={reason}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Parse ci_gate_result.detjson and print compact status line")
    parser.add_argument("--result", required=True, help="path to ci_gate_result.detjson")
    parser.add_argument("--json-out", help="optional output parse detjson path")
    parser.add_argument("--compact-out", help="optional output compact line txt path")
    parser.add_argument("--fail-on-invalid", action="store_true", help="return non-zero when parse/validation fails")
    parser.add_argument("--fail-on-fail", action="store_true", help="return non-zero when status is fail")
    args = parser.parse_args()

    result_path = Path(args.result)
    doc = load_json(result_path)
    if not isinstance(doc, dict):
        print(f"[ci-gate-result-parse] invalid reason=missing_or_invalid_result path={result_path}")
        if args.fail_on_invalid:
            return 1
        return 0
    if doc.get("schema") != EXPECTED_SCHEMA:
        print(
            "[ci-gate-result-parse] invalid "
            f"reason=schema_mismatch schema={doc.get('schema')} expected={EXPECTED_SCHEMA}"
        )
        if args.fail_on_invalid:
            return 1
        return 0
    status = str(doc.get("status", "fail")).strip() or "fail"
    aggregate_status = str(doc.get("aggregate_status", "fail")).strip() or "fail"
    if status not in {"pass", "fail"} or aggregate_status not in {"pass", "fail"}:
        print(
            "[ci-gate-result-parse] invalid "
            f"reason=status_field_invalid status={status} aggregate_status={aggregate_status}"
        )
        if args.fail_on_invalid:
            return 1
        return 0
    summary_line = str(doc.get("summary_line", "")).strip()
    if not summary_line:
        print("[ci-gate-result-parse] invalid reason=summary_line_missing")
        if args.fail_on_invalid:
            return 1
        return 0

    compact = compact_line(doc)
    print(f"[ci-gate-result-parse] {compact}")

    if args.json_out:
        out = Path(args.json_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "schema": "ddn.ci.gate_result_parse.v1",
            "result_path": str(result_path),
            "parsed": {
                "status": status,
                "ok": bool(doc.get("ok", False)),
                "overall_ok": bool(doc.get("overall_ok", False)),
                "failed_steps": int(doc.get("failed_steps", -1)),
                "aggregate_status": aggregate_status,
                "reason": str(doc.get("reason", "-")),
            },
            "compact_line": compact,
        }
        out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    if args.compact_out:
        out = Path(args.compact_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(compact + "\n", encoding="utf-8")

    if args.fail_on_fail and status != "pass":
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
