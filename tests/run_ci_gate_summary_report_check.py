#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from ci_check_error_codes import SUMMARY_REPORT_CODES as CODES

PASS_REQUIRED_KEYS = (
    "report_index",
    "summary_line",
    "ci_gate_result",
    "ci_gate_badge",
    "ci_fail_brief_hint",
    "ci_fail_brief_exists",
    "ci_fail_triage_hint",
    "ci_fail_triage_exists",
    "age3_status",
    "age4_status",
    "age5_status",
    "seamgrim_phase3_cleanup",
    "fixed64_threeway_report",
    "fixed64_threeway_status",
    "fixed64_threeway_ok",
)


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def parse_summary(path: Path) -> tuple[str | None, dict[str, str], list[str]]:
    lines = [line.strip() for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
    status: str | None = None
    kv: dict[str, str] = {}
    for line in lines:
        if not line.startswith("[ci-gate-summary] "):
            continue
        body = line[len("[ci-gate-summary] ") :]
        if body in {"PASS", "FAIL"}:
            status = body.lower()
            continue
        if "=" not in body:
            continue
        key, value = body.split("=", 1)
        key = key.strip()
        value = value.strip()
        if key:
            kv[key] = value
    return status, kv, lines


def fail(msg: str, code: str = "E_CHECK") -> int:
    print(f"[ci-gate-summary-report-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate ci_gate_summary.txt core key/value lines")
    parser.add_argument("--summary", required=True, help="path to ci_gate_summary.txt")
    parser.add_argument("--index", required=True, help="path to ci_gate_report_index.detjson")
    parser.add_argument("--require-pass", action="store_true", help="require summary PASS block")
    args = parser.parse_args()

    summary_path = Path(args.summary)
    index_path = Path(args.index)
    if not summary_path.exists():
        return fail(f"missing summary file: {summary_path}", code=CODES["SUMMARY_MISSING"])
    status, kv, lines = parse_summary(summary_path)
    if not lines:
        return fail("summary file is empty", code=CODES["SUMMARY_EMPTY"])
    if status not in {"pass", "fail"}:
        return fail("missing PASS/FAIL header line", code=CODES["SUMMARY_STATUS_MISSING"])
    if args.require_pass and status != "pass":
        return fail("require-pass set but summary status is not PASS", code=CODES["REQUIRE_PASS"])

    index_doc = load_json(index_path)
    if index_doc is None:
        return fail(f"invalid index json: {index_path}", code=CODES["INDEX_INVALID"])

    if status == "pass":
        for key in PASS_REQUIRED_KEYS:
            value = kv.get(key, "").strip()
            if not value:
                return fail(f"missing summary key: {key}", code=CODES["PASS_KEY_MISSING"])

        if kv.get("report_index") != str(index_path):
            return fail(
                f"report_index mismatch summary={kv.get('report_index')} index={index_path}",
                code=CODES["REPORT_INDEX_MISMATCH"],
            )

        reports = index_doc.get("reports")
        if not isinstance(reports, dict):
            return fail("index.reports is missing", code=CODES["INDEX_REPORTS_MISSING"])
        compare_map = {
            "summary_line": str(reports.get("summary_line", "")).strip(),
            "ci_gate_result": str(reports.get("ci_gate_result_json", "")).strip(),
            "ci_gate_badge": str(reports.get("ci_gate_badge_json", "")).strip(),
            "ci_fail_triage_hint": str(reports.get("ci_fail_triage_json", "")).strip(),
            "age3_status": str(reports.get("age3_close_status_json", "")).strip(),
            "age4_status": str(reports.get("age4_close", "")).strip(),
            "age5_status": str(reports.get("age5_close", "")).strip(),
            "seamgrim_phase3_cleanup": str(reports.get("seamgrim_phase3_cleanup", "")).strip(),
            "fixed64_threeway_report": str(reports.get("fixed64_threeway_gate", "")).strip(),
        }
        for key, expected in compare_map.items():
            if not expected:
                return fail(f"index missing expected path for {key}", code=CODES["INDEX_PATH_MISSING"])
            if kv.get(key) != expected:
                return fail(
                    f"{key} mismatch summary={kv.get(key)} index={expected}",
                    code=CODES["SUMMARY_INDEX_PATH_MISMATCH"],
                )
        fixed64_ok_text = kv.get("fixed64_threeway_ok", "").strip()
        if fixed64_ok_text not in {"0", "1"}:
            return fail(f"fixed64_threeway_ok invalid: {fixed64_ok_text}", code=CODES["PASS_KEY_MISSING"])
        if fixed64_ok_text != "1":
            return fail("PASS summary requires fixed64_threeway_ok=1", code=CODES["PASS_KEY_MISSING"])
        fixed64_status = kv.get("fixed64_threeway_status", "").strip()
        if not fixed64_status:
            return fail("fixed64_threeway_status is empty", code=CODES["PASS_KEY_MISSING"])

        hint = kv.get("ci_fail_brief_hint", "").strip()
        if not hint:
            return fail("ci_fail_brief_hint is empty", code=CODES["BRIEF_HINT_EMPTY"])
        exists_text = kv.get("ci_fail_brief_exists", "").strip()
        if exists_text not in {"0", "1"}:
            return fail(f"ci_fail_brief_exists invalid: {exists_text}", code=CODES["BRIEF_EXISTS_INVALID"])
        brief_exists_actual = 1 if Path(hint).exists() else 0
        if int(exists_text) != brief_exists_actual:
            return fail(
                f"ci_fail_brief_exists mismatch summary={exists_text} actual={brief_exists_actual}",
                code=CODES["BRIEF_EXISTS_MISMATCH"],
            )
        if brief_exists_actual != 1:
            return fail(f"PASS summary requires ci_fail_brief_exists=1 path={hint}", code=CODES["PASS_BRIEF_REQUIRED"])
        triage_hint = kv.get("ci_fail_triage_hint", "").strip()
        if not triage_hint:
            return fail("ci_fail_triage_hint is empty", code=CODES["TRIAGE_HINT_EMPTY"])
        triage_exists_text = kv.get("ci_fail_triage_exists", "").strip()
        if triage_exists_text not in {"0", "1"}:
            return fail(f"ci_fail_triage_exists invalid: {triage_exists_text}", code=CODES["TRIAGE_EXISTS_INVALID"])
        triage_exists_actual = 1 if Path(triage_hint).exists() else 0
        if int(triage_exists_text) != triage_exists_actual:
            return fail(
                f"ci_fail_triage_exists mismatch summary={triage_exists_text} actual={triage_exists_actual}",
                code=CODES["TRIAGE_EXISTS_MISMATCH"],
            )
        if triage_exists_actual != 1:
            return fail(
                f"PASS summary requires ci_fail_triage_exists=1 path={triage_hint}",
                code=CODES["PASS_TRIAGE_REQUIRED"],
            )

    print(
        f"[ci-gate-summary-report-check] ok status={status} "
        f"summary={summary_path} index={index_path}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
