#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


DETAIL_RE = re.compile(r"detail=(.+)$")


def _sanitize_for_console(text: str) -> str:
    return str(text or "").replace("\ufffd", "?")


def _safe_print(text: str) -> None:
    message = _sanitize_for_console(text)
    try:
        print(message)
    except UnicodeEncodeError:
        enc = getattr(sys.stdout, "encoding", None) or "utf-8"
        sys.stdout.write(message.encode(enc, errors="replace").decode(enc, errors="replace") + "\n")


def fail(detail: str) -> int:
    _safe_print(f"check=runtime_fallback_metrics detail={detail}")
    return 1


def parse_pack_detail(stdout: str) -> list[dict[str, str]]:
    text = str(stdout or "")
    line = next((row.strip() for row in text.splitlines() if "detail=" in row), "")
    if not line:
        return []
    m = DETAIL_RE.search(line)
    if not m:
        return []
    payload = m.group(1).strip()
    if not payload:
        return []

    out: list[dict[str, str]] = []
    for chunk in [row.strip() for row in payload.split(",") if row.strip()]:
        parts = [part.strip() for part in chunk.split(":")]
        if len(parts) < 4:
            continue
        mode = ""
        title_start = 3
        if len(parts) >= 5 and parts[3] in {"native", "fallback"}:
            mode = parts[3]
            title_start = 4
        out.append(
            {
                "seed_id": parts[0],
                "y_key": parts[1],
                "points": parts[2],
                "mode": mode,
                "source": ":".join(parts[title_start:]).strip(),
                "title": ":".join(parts[title_start:]).strip(),
            }
        )
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description="Seamgrim runtime fallback metrics check")
    parser.add_argument(
        "--json-out",
        default="build/reports/seamgrim_runtime_fallback_metrics.detjson",
        help="metrics report output path",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    runner = root / "tests" / "run_seamgrim_seed_runtime_visual_pack_check.py"
    if not runner.exists():
        return fail(f"runner_missing:{runner.as_posix()}")

    proc = subprocess.run(
        [sys.executable, str(runner)],
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or f"returncode={proc.returncode}"
        return fail(f"seed_runtime_visual_pack_failed:{detail}")

    rows = parse_pack_detail(proc.stdout)
    if not rows:
        return fail("pack_detail_parse_failed")

    mixed_rows = [
        row
        for row in rows
        if str(row.get("mode", "")).strip().lower() == "native"
        and "fallback" in str(row.get("source", row.get("title", ""))).strip().lower()
    ]
    if mixed_rows:
        top = mixed_rows[0]
        return fail(
            "native_fallback_label_mixed:"
            f"{top.get('seed_id', '-')}:"
            f"{top.get('source', top.get('title', '-'))}"
        )

    total = len(rows)
    fallback_rows = [
        row
        for row in rows
        if (
            str(row.get("mode", "")).strip().lower() == "fallback"
            or (
                not str(row.get("mode", "")).strip()
                and "fallback" in str(row.get("source", row.get("title", ""))).strip().lower()
            )
        )
    ]
    fallback_count = len(fallback_rows)
    native_count = total - fallback_count
    ratio = fallback_count / total if total > 0 else 0.0

    report = {
        "schema": "seamgrim.runtime_fallback_metrics.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": True,
        "total": total,
        "fallback_count": fallback_count,
        "native_count": native_count,
        "fallback_ratio": ratio,
        "rows": rows,
    }
    out_path = Path(args.json_out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    _safe_print(
        "[runtime-fallback] "
        f"total={total} fallback={fallback_count} native={native_count} ratio={ratio:.3f} "
        f"report={out_path.as_posix()}"
    )
    _safe_print("seamgrim runtime fallback metrics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
