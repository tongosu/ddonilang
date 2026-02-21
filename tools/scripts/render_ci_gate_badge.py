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


def build_badge(result_path: Path, result_doc: dict | None) -> tuple[dict, bool]:
    if not isinstance(result_doc, dict):
        payload = {
            "schema": "ddn.ci.gate_badge.v1",
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "label": "ci-gate",
            "message": "invalid-result",
            "color": "lightgray",
            "status": "fail",
            "ok": False,
            "reason": "invalid_or_missing_result",
            "result_path": str(result_path),
        }
        return payload, False

    status = str(result_doc.get("status", "fail")).strip() or "fail"
    ok = bool(result_doc.get("ok", False))
    failed_steps = int(result_doc.get("failed_steps", -1))
    if ok and status == "pass":
        message = "pass"
        color = "brightgreen"
    else:
        message = f"fail ({failed_steps})" if failed_steps >= 0 else "fail"
        color = "red"
    payload = {
        "schema": "ddn.ci.gate_badge.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "label": "ci-gate",
        "message": message,
        "color": color,
        "status": status,
        "ok": ok,
        "reason": str(result_doc.get("reason", "-")).strip() or "-",
        "result_path": str(result_path),
    }
    return payload, (ok and status == "pass")


def main() -> int:
    parser = argparse.ArgumentParser(description="Render CI gate badge json")
    parser.add_argument("result_json", help="path to ci_gate_result.detjson")
    parser.add_argument("--out", required=True, help="output badge detjson path")
    parser.add_argument("--fail-on-bad", action="store_true", help="return non-zero when badge status is fail")
    args = parser.parse_args()

    result_path = Path(args.result_json)
    out_path = Path(args.out)
    result_doc = load_json(result_path)
    payload, ok = build_badge(result_path, result_doc)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"[ci-gate-badge] out={out_path} status={payload.get('status')} ok={int(ok)}")
    if args.fail_on_bad and not ok:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
