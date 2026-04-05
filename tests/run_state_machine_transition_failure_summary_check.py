#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


PACK_SPECS = [
    {
        "pack_name": "state_machine_transition_failure_report_v1",
        "expected_code": "E_STATE_TRANSITION_CHECK_FAILED",
        "expected_kind": "check_failed",
        "category": "check",
        "field_name": "check_name",
        "field_value": "전이_실패",
    },
    {
        "pack_name": "state_machine_transition_check_unresolved_report_v1",
        "expected_code": "E_STATE_TRANSITION_CHECK_UNRESOLVED",
        "expected_kind": "check_unresolved",
        "category": "check",
        "field_name": "check_name",
        "field_value": "전이_안전",
    },
    {
        "pack_name": "state_machine_transition_guard_rejected_report_v1",
        "expected_code": "E_STATE_TRANSITION_GUARD_REJECTED",
        "expected_kind": "guard_rejected",
        "category": "guard",
        "field_name": "state_name",
        "field_value": "빨강",
    },
    {
        "pack_name": "state_machine_transition_guard_unresolved_report_v1",
        "expected_code": "E_STATE_TRANSITION_GUARD_UNRESOLVED",
        "expected_kind": "guard_unresolved",
        "category": "guard",
        "field_name": "guard_name",
        "field_value": "없는검사",
    },
    {
        "pack_name": "state_machine_transition_action_failure_report_v1",
        "expected_code": "E_STATE_TRANSITION_ACTION_UNRESOLVED",
        "expected_kind": "action_unresolved",
        "category": "action",
        "field_name": "action_name",
        "field_value": "기록",
    },
    {
        "pack_name": "state_machine_transition_action_aborted_report_v1",
        "expected_code": "E_STATE_TRANSITION_ACTION_ABORTED",
        "expected_kind": "action_aborted",
        "category": "action",
        "field_name": "action_name",
        "field_value": "기록",
    },
    {
        "pack_name": "state_machine_transition_action_arg_unresolved_report_v1",
        "expected_code": "E_STATE_TRANSITION_ACTION_ARG_UNRESOLVED",
        "expected_kind": "action_arg_unresolved",
        "category": "action",
        "field_name": "param_name",
        "field_value": "없는상태",
    },
]


def fail(msg: str) -> int:
    print(f"[state-transition-failure-summary] fail: {msg}")
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def validate_expected_report(root: Path, spec: dict) -> dict:
    report_path = (
        root
        / "pack"
        / str(spec["pack_name"])
        / "expected"
        / "state_transition_failures.detjson"
    )
    if not report_path.exists():
        raise RuntimeError(f"missing expected report: {report_path}")
    doc = load_json(report_path)
    if doc.get("schema") != "ddn.state_transition_failure_report.v1":
        raise RuntimeError(f"{report_path}: schema mismatch")
    failures = doc.get("failures")
    if not isinstance(failures, list) or len(failures) != 1:
        raise RuntimeError(f"{report_path}: failures must be single-row list")
    row = failures[0]
    if row.get("code") != spec["expected_code"]:
        raise RuntimeError(f"{report_path}: code mismatch {row.get('code')}")
    if row.get("kind") != spec["expected_kind"]:
        raise RuntimeError(f"{report_path}: kind mismatch {row.get('kind')}")
    field_name = str(spec["field_name"])
    if row.get(field_name) != spec["field_value"]:
        raise RuntimeError(
            f"{report_path}: {field_name} mismatch {row.get(field_name)!r}"
        )
    return {
        "pack_name": spec["pack_name"],
        "report_path": str(report_path),
        "code": row["code"],
        "kind": row["kind"],
        "category": spec["category"],
        "field_name": field_name,
        "field_value": row[field_name],
        "message": row.get("message", ""),
    }


def build_summary(rows: list[dict]) -> dict:
    category_counts: dict[str, int] = {}
    kind_counts: dict[str, int] = {}
    for row in rows:
        category = str(row["category"])
        kind = str(row["kind"])
        category_counts[category] = category_counts.get(category, 0) + 1
        kind_counts[kind] = kind_counts.get(kind, 0) + 1
    return {
        "schema": "ddn.state_transition_failure_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": True,
        "total_pack_count": len(rows),
        "total_failure_count": len(rows),
        "category_counts": category_counts,
        "kind_counts": kind_counts,
        "packs": rows,
    }


def run_pack_golden(root: Path) -> None:
    cmd = [
        sys.executable,
        "tests/run_pack_golden.py",
        "--manifest-path",
        "tool/Cargo.toml",
        *[str(spec["pack_name"]) for spec in PACK_SPECS],
    ]
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.stdout:
        print(proc.stdout.strip())
    if proc.stderr:
        print(proc.stderr.strip())
    if proc.returncode != 0:
        raise RuntimeError((proc.stderr or proc.stdout or "").strip() or "run_pack_golden failed")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Summarize state-machine transition failure detjson contracts"
    )
    parser.add_argument("--json-out", default="", help="optional summary json output path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    try:
        run_pack_golden(root)
        rows = [validate_expected_report(root, spec) for spec in PACK_SPECS]
    except RuntimeError as exc:
        return fail(str(exc))

    summary = build_summary(rows)
    if summary["category_counts"] != {"check": 2, "guard": 2, "action": 3}:
        return fail(f"category counts mismatch: {summary['category_counts']}")
    if len(summary["kind_counts"]) != len(PACK_SPECS):
        return fail(f"kind count coverage mismatch: {summary['kind_counts']}")

    if args.json_out:
        out = Path(args.json_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    print("state machine transition failure summary check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
