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


def bool_mark(value: object) -> str:
    return "PASS" if bool(value) else "FAIL"


def safe(text: object) -> str:
    return str(text).replace("\n", " ").strip()


def build_markdown(report_path: Path, payload: dict | None) -> tuple[str, bool]:
    lines: list[str] = []
    lines.append("# AGE3 Close Summary")
    lines.append("")
    lines.append(f"- report: `{report_path}`")
    if not isinstance(payload, dict):
        lines.append("- overall: `FAIL`")
        lines.append("- reason: invalid or missing report payload")
        lines.append("")
        return "\n".join(lines).rstrip() + "\n", False

    overall_ok = bool(payload.get("overall_ok", False))
    lines.append(f"- schema: `{safe(payload.get('schema', '-'))}`")
    lines.append(f"- generated_at_utc: `{safe(payload.get('generated_at_utc', '-'))}`")
    lines.append(f"- overall: `{bool_mark(overall_ok)}`")
    lines.append(f"- seamgrim_report_path: `{safe(payload.get('seamgrim_report_path', '-'))}`")
    lines.append(f"- ui_age3_report_path: `{safe(payload.get('ui_age3_report_path', '-'))}`")
    lines.append("")

    criteria = payload.get("criteria")
    lines.append("## Criteria")
    lines.append("")
    lines.append("| Name | Result | Detail |")
    lines.append("| --- | --- | --- |")
    if isinstance(criteria, list) and criteria:
        for row in criteria:
            if not isinstance(row, dict):
                continue
            name = safe(row.get("name", "-"))
            mark = bool_mark(row.get("ok", False))
            detail = safe(row.get("detail", "-")).replace("|", "\\|")
            lines.append(f"| `{name}` | `{mark}` | {detail} |")
    else:
        lines.append("| `criteria` | `FAIL` | criteria missing |")
    lines.append("")

    digest = payload.get("failure_digest")
    lines.append("## Failure Digest")
    lines.append("")
    if isinstance(digest, list) and digest:
        for line in digest:
            lines.append(f"- {safe(line)}")
    else:
        lines.append("- (none)")
    lines.append("")
    return "\n".join(lines).rstrip() + "\n", overall_ok


def main() -> int:
    parser = argparse.ArgumentParser(description="Render AGE3 close report into markdown summary")
    parser.add_argument("report", help="path to ddn.seamgrim.age3_close_report.v1")
    parser.add_argument("--out", required=True, help="output markdown path")
    parser.add_argument("--fail-on-bad", action="store_true", help="return non-zero when overall is FAIL")
    args = parser.parse_args()

    report_path = Path(args.report)
    out_path = Path(args.out)
    payload = load_payload(report_path)
    content, overall_ok = build_markdown(report_path, payload)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(content, encoding="utf-8")
    print(f"[age3-close-md] out={out_path} overall_ok={int(overall_ok)}")
    if args.fail_on_bad and not overall_ok:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
