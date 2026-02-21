#!/usr/bin/env python3
"""Quality gate for rewritten Seamgrim lessons (batch v1)."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
from pathlib import Path


ASSIGN_EQUAL_RE = re.compile(r"^[^#\n]*=[^=\n]*\.\s*$")


def has_assign_equal(text: str) -> bool:
    for line in text.splitlines():
        if ASSIGN_EQUAL_RE.match(line.strip()):
            return True
    return False


def run_parse_check(ddn_path: Path, manifest_path: Path) -> tuple[bool, str]:
    cmd = [
        "cargo",
        "run",
        "-q",
        "--manifest-path",
        str(manifest_path),
        "--",
        "run",
        str(ddn_path),
        "--madi",
        "1",
    ]
    proc = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")
    if proc.returncode != 0:
        stderr = (proc.stderr or "").strip().splitlines()
        return False, stderr[-1] if stderr else f"exit={proc.returncode}"
    for line in (proc.stdout or "").splitlines():
        if line.strip().startswith("state_hash="):
            return True, "ok"
    return False, "missing state_hash"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--manifest",
        default="solutions/seamgrim_ui_mvp/lessons_rewrite_v1/rewrite_manifest.detjson",
        help="Rewrite manifest path",
    )
    parser.add_argument(
        "--teul-manifest",
        default="tools/teul-cli/Cargo.toml",
        help="teul-cli Cargo manifest path",
    )
    parser.add_argument(
        "--out",
        default="build/reports/seamgrim_batch_quality_gate_v1.detjson",
        help="Output report path",
    )
    parser.add_argument(
        "--check-run",
        action="store_true",
        help="Run parse/execution check via teul-cli run --madi 1",
    )
    args = parser.parse_args()

    manifest_path = Path(args.manifest)
    doc = json.loads(manifest_path.read_text(encoding="utf-8"))
    rows = list(doc.get("generated", []))
    teul_manifest = Path(args.teul_manifest)

    result_rows = []
    pass_count = 0

    for row in rows:
        lesson_id = str(row.get("lesson_id", "")).strip()
        ddn_path = Path(str(row.get("generated_lesson_ddn", "")).strip())
        if not lesson_id or not ddn_path.exists():
            result_rows.append(
                {
                    "lesson_id": lesson_id or "<missing>",
                    "ok": False,
                    "checks": {"file_exists": False},
                }
            )
            continue

        text = ddn_path.read_text(encoding="utf-8")
        checks = {
            "has_control": "#control:" in text,
            "has_start_block": "(시작)할때" in text,
            "has_tick_block": "(매마디)마다" in text,
            "has_show": "보여주기." in text,
            "no_legacy_equal_assign": not has_assign_equal(text),
            "no_boim_block": "보임" not in text,
        }
        if args.check_run:
            ok_run, run_msg = run_parse_check(ddn_path, teul_manifest)
            checks["parse_run_ok"] = ok_run
            checks["parse_run_msg"] = run_msg
        ok = all(value is True for key, value in checks.items() if key != "parse_run_msg")
        if ok:
            pass_count += 1
        result_rows.append(
            {
                "lesson_id": lesson_id,
                "subject": row.get("subject", ""),
                "path": str(ddn_path.as_posix()),
                "ok": ok,
                "checks": checks,
            }
        )

    report = {
        "schema": "seamgrim.batch.quality_gate.v1",
        "manifest": str(manifest_path.as_posix()),
        "count": len(result_rows),
        "pass_count": pass_count,
        "fail_count": len(result_rows) - pass_count,
        "rows": result_rows,
    }
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"[ok] wrote {out_path} pass={pass_count}/{len(result_rows)}")
    return 0 if pass_count == len(result_rows) else 1


if __name__ == "__main__":
    raise SystemExit(main())
