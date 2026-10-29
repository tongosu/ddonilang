#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from _ci_age5_combined_heavy_contract import (
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
    build_age5_combined_heavy_policy_origin_trace_contract_compact_failure_reason,
    build_age5_combined_heavy_policy_origin_trace_contract_compact_reason,
    build_age5_combined_heavy_policy_origin_trace,
    build_age5_combined_heavy_policy_origin_trace_text,
    build_age5_combined_heavy_full_real_source_trace,
    build_age5_combined_heavy_full_real_source_trace_text,
    build_age5_close_digest_selftest_default_field,
    build_age5_combined_heavy_child_summary_default_text_transport_fields,
)


def fail(msg: str) -> int:
    print(f"[ci-combine-age5-selftest] fail: {msg}")
    return 1


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def read_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def run_combine(
    *,
    seamgrim: Path,
    age3: Path,
    age4: Path,
    age5: Path,
    age5_policy_report: Path,
    age5_policy_text: Path,
    age5_policy_summary: Path,
    oi: Path,
    out: Path,
    require_age4: bool,
    require_age5: bool,
) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tools/scripts/combine_ci_reports.py",
        "--seamgrim-report",
        str(seamgrim),
        "--age3-report",
        str(age3),
        "--oi-report",
        str(oi),
        "--age4-report",
        str(age4),
        "--age5-report",
        str(age5),
        "--age5-combined-heavy-policy-report",
        str(age5_policy_report),
        "--age5-combined-heavy-policy-text",
        str(age5_policy_text),
        "--age5-combined-heavy-policy-summary",
        str(age5_policy_summary),
        "--out",
        str(out),
        "--fail-on-bad",
        "--require-age3",
    ]
    if require_age4:
        cmd.append("--require-age4")
    if require_age5:
        cmd.append("--require-age5")
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def run_combine_print_summary(
    *,
    seamgrim: Path,
    age3: Path,
    age4: Path,
    age5: Path,
    age5_policy_report: Path,
    age5_policy_text: Path,
    age5_policy_summary: Path,
    oi: Path,
    out: Path,
    require_age4: bool,
    require_age5: bool,
) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tools/scripts/combine_ci_reports.py",
        "--seamgrim-report",
        str(seamgrim),
        "--age3-report",
        str(age3),
        "--oi-report",
        str(oi),
        "--age4-report",
        str(age4),
        "--age5-report",
        str(age5),
        "--age5-combined-heavy-policy-report",
        str(age5_policy_report),
        "--age5-combined-heavy-policy-text",
        str(age5_policy_text),
        "--age5-combined-heavy-policy-summary",
        str(age5_policy_summary),
        "--out",
        str(out),
        "--fail-on-bad",
        "--require-age3",
        "--print-summary",
    ]
    if require_age4:
        cmd.append("--require-age4")
    if require_age5:
        cmd.append("--require-age5")
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def run_aggregate_digest(report: Path) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tools/scripts/print_ci_aggregate_digest.py",
        str(report),
        "--top",
        "1",
    ]
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def main() -> int:
    expected_default_transport = build_age5_combined_heavy_child_summary_default_text_transport_fields()
    expected_digest_default_field = build_age5_close_digest_selftest_default_field()
    expected_full_real_source_trace = build_age5_combined_heavy_full_real_source_trace(
        smoke_check_script_exists=True,
        smoke_check_selftest_script_exists=True,
    )
    expected_full_real_source_trace_text = build_age5_combined_heavy_full_real_source_trace_text(
        expected_full_real_source_trace
    )
    with tempfile.TemporaryDirectory(prefix="ci_combine_age5_selftest_") as tmp:
        root = Path(tmp)
        seamgrim_report = root / "seamgrim.json"
        age3_report = root / "age3.detjson"
        age4_report = root / "age4.detjson"
        age5_report = root / "age5.detjson"
        age5_policy_report = root / "age5_combined_heavy_policy.detjson"
        age5_policy_text = root / "age5_combined_heavy_policy.txt"
        age5_policy_summary = root / "age5_combined_heavy_policy_summary.txt"
        oi_report = root / "oi.detjson"
        out_report = root / "aggregate.detjson"

        write_json(
            seamgrim_report,
            {
                "schema": "ddn.seamgrim.ci_gate_report.v1",
                "ok": True,
                "steps": [],
                "failure_digest": [],
            },
        )
        write_json(
            age3_report,
            {
                "schema": "ddn.seamgrim.age3_close_report.v1",
                "overall_ok": True,
                "criteria": [],
                "failure_digest": [],
            },
        )
        write_json(
            age4_report,
            {
                "schema": "ddn.age4_close_report.v1",
                "overall_ok": True,
                "criteria": [],
                "failure_digest": [],
            },
        )
        write_json(
            age5_policy_report,
            {
                "schema": "ddn.ci.age5_combined_heavy_policy.v1",
                "combined_digest_selftest_default_field": expected_digest_default_field,
                AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
            },
        )
        age5_policy_text.write_text(
            "[age5-combined-heavy-policy] "
            f"{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}={AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT} "
            "combined_digest_selftest_default_field="
            + json.dumps(expected_digest_default_field, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            + "\n",
            encoding="utf-8",
        )
        age5_policy_summary.write_text(
            f"[age5-combined-heavy-policy] age5_combined_heavy_policy_report_path={age5_policy_report} "
            f"age5_combined_heavy_policy_text_path={age5_policy_text} "
            "age5_combined_heavy_policy_report_exists=1 "
            "age5_combined_heavy_policy_text_exists=1 "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT} "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT} "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}="
            f"{build_age5_combined_heavy_policy_origin_trace_contract_compact_reason()} "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}=ok "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}=1 "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}=report_path={age5_policy_report}|report_exists=1|text_path={age5_policy_text}|text_exists=1|summary_path={age5_policy_summary}|summary_exists=1 "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY}="
            + json.dumps(
                {
                    "report_exists": "1",
                    "report_path": str(age5_policy_report),
                    "summary_exists": "1",
                    "summary_path": str(age5_policy_summary),
                    "text_exists": "1",
                    "text_path": str(age5_policy_text),
                },
                ensure_ascii=False,
                sort_keys=True,
                separators=(",", ":"),
            ),
            encoding="utf-8",
        )
        expected_policy_origin_trace = build_age5_combined_heavy_policy_origin_trace(
            report_path=str(age5_policy_report),
            report_exists=True,
            text_path=str(age5_policy_text),
            text_exists=True,
            summary_path=str(age5_policy_summary),
            summary_exists=True,
        )
        expected_policy_origin_trace_text = build_age5_combined_heavy_policy_origin_trace_text(
            expected_policy_origin_trace
        )
        write_json(
            oi_report,
            {
                "schema": "ddn.oi405_406.close_report.v1",
                "overall_ok": True,
                "packs": [],
                "failure_digest": [],
            },
        )

        # case 1: require-age5 + valid age5 report => pass
        write_json(
            age5_report,
            {
                "schema": "ddn.age5_close_report.v1",
                "overall_ok": True,
                "criteria": [],
                "failure_digest": [],
                "age5_close_digest_selftest_ok": 1,
                AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
                "combined_digest_selftest_default_field": expected_digest_default_field,
                "age5_combined_heavy_full_real_status": "pass",
                "age5_combined_heavy_runtime_helper_negative_status": "skipped",
                "age5_combined_heavy_group_id_summary_negative_status": "skipped",
                "combined_heavy_child_timeout_sec": 0,
                "age5_combined_heavy_timeout_mode": "disabled",
                "age5_combined_heavy_timeout_present": "0",
                "age5_combined_heavy_timeout_targets": "-",
                "full_real_source_trace": expected_full_real_source_trace,
                "full_real_source_trace_text": expected_full_real_source_trace_text,
                **expected_default_transport,
            },
        )
        proc_ok = run_combine(
            seamgrim=seamgrim_report,
            age3=age3_report,
            age4=age4_report,
            age5=age5_report,
            age5_policy_report=age5_policy_report,
            age5_policy_text=age5_policy_text,
            age5_policy_summary=age5_policy_summary,
            oi=oi_report,
            out=out_report,
            require_age4=True,
            require_age5=True,
        )
        if proc_ok.returncode != 0:
            return fail(f"require-age5 pass case failed: out={proc_ok.stdout} err={proc_ok.stderr}")
        ok_doc = read_json(out_report)
        if not isinstance(ok_doc, dict) or not bool(ok_doc.get("overall_ok", False)):
            return fail("require-age5 pass case aggregate overall_ok mismatch")
        age5_row = ok_doc.get("age5")
        if not isinstance(age5_row, dict) or not bool(age5_row.get("ok", False)):
            return fail("require-age5 pass case age5.ok mismatch")
        if str(age5_row.get("age5_combined_heavy_full_real_status", "")).strip() != "pass":
            return fail("require-age5 pass case full_real child status mismatch")
        if str(age5_row.get("age5_combined_heavy_runtime_helper_negative_status", "")).strip() != "skipped":
            return fail("require-age5 pass case runtime-helper child status mismatch")
        if str(age5_row.get("age5_combined_heavy_group_id_summary_negative_status", "")).strip() != "skipped":
            return fail("require-age5 pass case group-id child status mismatch")
        if str(age5_row.get("combined_heavy_child_timeout_sec", "")).strip() != "0":
            return fail("require-age5 pass case timeout sec mismatch")
        if str(age5_row.get("age5_combined_heavy_timeout_mode", "")).strip() != "disabled":
            return fail("require-age5 pass case timeout mode mismatch")
        if str(age5_row.get("age5_combined_heavy_timeout_present", "")).strip() != "0":
            return fail("require-age5 pass case timeout present mismatch")
        if str(age5_row.get("age5_combined_heavy_timeout_targets", "")).strip() != "-":
            return fail("require-age5 pass case timeout targets mismatch")
        if dict(age5_row.get("full_real_source_trace", {})) != expected_full_real_source_trace:
            return fail("require-age5 pass case full_real_source_trace mismatch")
        if str(age5_row.get("full_real_source_trace_text", "")).strip() != expected_full_real_source_trace_text:
            return fail("require-age5 pass case full_real_source_trace_text mismatch")
        if str(age5_row.get("age5_close_digest_selftest_ok", "")).strip() != "1":
            return fail("require-age5 pass case digest selftest status mismatch")
        if str(age5_row.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
            return fail("require-age5 pass case digest selftest default text mismatch")
        if dict(age5_row.get("combined_digest_selftest_default_field", {})) != expected_digest_default_field:
            return fail("require-age5 pass case digest selftest default field mismatch")
        if str(age5_row.get("age5_policy_combined_digest_selftest_default_field_text", "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
            return fail("require-age5 pass case policy digest selftest default text mismatch")
        if dict(age5_row.get("age5_policy_combined_digest_selftest_default_field", {})) != expected_digest_default_field:
            return fail("require-age5 pass case policy digest selftest default field mismatch")
        if not bool(age5_row.get("age5_combined_heavy_policy_report_exists", False)):
            return fail("require-age5 pass case policy report exists mismatch")
        if not bool(age5_row.get("age5_combined_heavy_policy_text_exists", False)):
            return fail("require-age5 pass case policy text exists mismatch")
        if str(age5_row.get("age5_combined_heavy_policy_summary_path", "")).strip() != str(age5_policy_summary):
            return fail("require-age5 pass case policy summary path mismatch")
        if not bool(age5_row.get("age5_combined_heavy_policy_summary_exists", False)):
            return fail("require-age5 pass case policy summary exists mismatch")
        if (
            str(age5_row.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY, "")).strip()
            != AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT
        ):
            return fail("require-age5 pass case policy origin trace contract issue mismatch")
        if (
            str(age5_row.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY, "")).strip()
            != AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT
        ):
            return fail("require-age5 pass case policy origin trace source issue mismatch")
        if (
            str(age5_row.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY, "")).strip()
            != build_age5_combined_heavy_policy_origin_trace_contract_compact_reason()
        ):
            return fail("require-age5 pass case policy origin trace compact reason mismatch")
        if (
            str(
                age5_row.get(
                    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY, ""
                )
            ).strip()
            != build_age5_combined_heavy_policy_origin_trace_contract_compact_failure_reason()
        ):
            return fail("require-age5 pass case policy origin trace compact failure reason mismatch")
        if str(age5_row.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY, "")).strip() != "ok":
            return fail("require-age5 pass case policy origin trace contract status mismatch")
        if not bool(age5_row.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY, False)):
            return fail("require-age5 pass case policy origin trace contract ok mismatch")
        if dict(age5_row.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY, {})) != expected_policy_origin_trace:
            return fail("require-age5 pass case policy origin trace mismatch")
        if str(age5_row.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY, "")).strip() != expected_policy_origin_trace_text:
            return fail("require-age5 pass case policy origin trace text mismatch")
        for key, expected in expected_default_transport.items():
            if str(age5_row.get(key, "")).strip() != expected:
                return fail(f"require-age5 pass case default transport mismatch: {key}")
        summary_ok = run_combine_print_summary(
            seamgrim=seamgrim_report,
            age3=age3_report,
            age4=age4_report,
            age5=age5_report,
            age5_policy_report=age5_policy_report,
            age5_policy_text=age5_policy_text,
            age5_policy_summary=age5_policy_summary,
            oi=oi_report,
            out=out_report,
            require_age4=True,
            require_age5=True,
        )
        if summary_ok.returncode != 0:
            return fail(f"require-age5 pass case print-summary failed: out={summary_ok.stdout} err={summary_ok.stderr}")
        summary_ok_text = str(summary_ok.stdout or "")
        if "age5_close_digest_selftest_ok=1" not in summary_ok_text:
            return fail("require-age5 pass case print-summary digest selftest mismatch")
        if "combined_heavy_child_timeout_sec=0" not in summary_ok_text:
            return fail("require-age5 pass case print-summary timeout sec mismatch")
        if "age5_combined_heavy_timeout_mode=disabled" not in summary_ok_text:
            return fail("require-age5 pass case print-summary timeout mode mismatch")
        if "age5_combined_heavy_timeout_present=0" not in summary_ok_text:
            return fail("require-age5 pass case print-summary timeout present mismatch")
        if "age5_combined_heavy_timeout_targets=-" not in summary_ok_text:
            return fail("require-age5 pass case print-summary timeout targets mismatch")
        if f"{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}={AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT}" not in summary_ok_text:
            return fail("require-age5 pass case print-summary digest selftest default text mismatch")
        if (
            "combined_digest_selftest_default_field="
            + json.dumps(expected_digest_default_field, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            not in summary_ok_text
        ):
            return fail("require-age5 pass case print-summary digest selftest default field mismatch")
        if (
            "age5_policy_combined_digest_selftest_default_field_text=" + AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT
            not in summary_ok_text
        ):
            return fail("require-age5 pass case print-summary policy digest selftest default text mismatch")
        if (
            "age5_policy_combined_digest_selftest_default_field="
            + json.dumps(expected_digest_default_field, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            not in summary_ok_text
        ):
            return fail("require-age5 pass case print-summary policy digest selftest default field mismatch")
        if f"policy_report={age5_policy_report}" not in summary_ok_text:
            return fail("require-age5 pass case print-summary policy_report mismatch")
        if "policy_report_exists=1" not in summary_ok_text:
            return fail("require-age5 pass case print-summary policy_report_exists mismatch")
        if f"policy_text={age5_policy_text}" not in summary_ok_text:
            return fail("require-age5 pass case print-summary policy_text mismatch")
        if "policy_text_exists=1" not in summary_ok_text:
            return fail("require-age5 pass case print-summary policy_text_exists mismatch")
        if f"policy_summary={age5_policy_summary}" not in summary_ok_text:
            return fail("require-age5 pass case print-summary policy_summary mismatch")
        if "policy_summary_exists=1" not in summary_ok_text:
            return fail("require-age5 pass case print-summary policy_summary_exists mismatch")
        if (
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT}"
            not in summary_ok_text
        ):
            return fail("require-age5 pass case print-summary policy origin trace contract issue mismatch")
        if (
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT}"
            not in summary_ok_text
        ):
            return fail("require-age5 pass case print-summary policy origin trace source issue mismatch")
        if (
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}="
            f"{build_age5_combined_heavy_policy_origin_trace_contract_compact_reason()}"
            not in summary_ok_text
        ):
            return fail("require-age5 pass case print-summary policy origin trace compact reason mismatch")
        if (
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY}="
            f"{build_age5_combined_heavy_policy_origin_trace_contract_compact_failure_reason()}"
            not in summary_ok_text
        ):
            return fail("require-age5 pass case print-summary policy origin trace compact failure reason mismatch")
        if f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}=ok" not in summary_ok_text:
            return fail("require-age5 pass case print-summary policy origin trace contract status mismatch")
        if f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}=1" not in summary_ok_text:
            return fail("require-age5 pass case print-summary policy origin trace contract ok mismatch")
        if f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}={expected_policy_origin_trace_text}" not in summary_ok_text:
            return fail("require-age5 pass case print-summary policy_origin_trace_text mismatch")
        if (
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY}="
            + json.dumps(expected_policy_origin_trace, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            not in summary_ok_text
        ):
            return fail("require-age5 pass case print-summary policy_origin_trace mismatch")
        if (
            f"child_summary_defaults={expected_default_transport['ci_sanity_age5_combined_heavy_child_summary_default_fields']}"
            not in summary_ok_text
        ):
            return fail("require-age5 pass case print-summary child_summary_defaults mismatch")
        if (
            "sync_child_summary_defaults="
            + expected_default_transport["ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"]
            not in summary_ok_text
        ):
            return fail("require-age5 pass case print-summary sync_child_summary_defaults mismatch")
        digest_ok = run_aggregate_digest(out_report)
        if digest_ok.returncode != 0:
            return fail(f"require-age5 pass case aggregate digest failed: out={digest_ok.stdout} err={digest_ok.stderr}")
        digest_ok_text = str(digest_ok.stdout or "")
        if "age5_close_digest_selftest_ok=1" not in digest_ok_text:
            return fail("require-age5 pass case aggregate digest selftest mismatch")
        if "age5_combined_heavy_child_timeout_sec=0" not in digest_ok_text:
            return fail("require-age5 pass case aggregate digest timeout sec mismatch")
        if "age5_combined_heavy_timeout_mode=disabled" not in digest_ok_text:
            return fail("require-age5 pass case aggregate digest timeout mode mismatch")
        if "age5_combined_heavy_timeout_present=0" not in digest_ok_text:
            return fail("require-age5 pass case aggregate digest timeout present mismatch")
        if "age5_combined_heavy_timeout_targets=-" not in digest_ok_text:
            return fail("require-age5 pass case aggregate digest timeout targets mismatch")
        if f"{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}={AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT}" not in digest_ok_text:
            return fail("require-age5 pass case aggregate digest selftest default text mismatch")
        if (
            "combined_digest_selftest_default_field="
            + json.dumps(expected_digest_default_field, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            not in digest_ok_text
        ):
            return fail("require-age5 pass case aggregate digest selftest default field mismatch")
        if (
            "age5_policy_combined_digest_selftest_default_field_text=" + AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT
            not in digest_ok_text
        ):
            return fail("require-age5 pass case aggregate digest policy default text mismatch")
        if (
            "age5_policy_combined_digest_selftest_default_field="
            + json.dumps(expected_digest_default_field, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            not in digest_ok_text
        ):
            return fail("require-age5 pass case aggregate digest policy default field mismatch")
        if f"age5_policy_report={age5_policy_report}" not in digest_ok_text:
            return fail("require-age5 pass case aggregate digest policy_report mismatch")
        if "age5_policy_report_exists=1" not in digest_ok_text:
            return fail("require-age5 pass case aggregate digest policy_report_exists mismatch")
        if f"age5_policy_text={age5_policy_text}" not in digest_ok_text:
            return fail("require-age5 pass case aggregate digest policy_text mismatch")
        if "age5_policy_text_exists=1" not in digest_ok_text:
            return fail("require-age5 pass case aggregate digest policy_text_exists mismatch")
        if f"age5_policy_summary={age5_policy_summary}" not in digest_ok_text:
            return fail("require-age5 pass case aggregate digest policy_summary mismatch")
        if "age5_policy_summary_exists=1" not in digest_ok_text:
            return fail("require-age5 pass case aggregate digest policy_summary_exists mismatch")
        if (
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT}"
            not in digest_ok_text
        ):
            return fail("require-age5 pass case aggregate digest policy origin trace contract issue mismatch")
        if (
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT}"
            not in digest_ok_text
        ):
            return fail("require-age5 pass case aggregate digest policy origin trace source issue mismatch")
        if (
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}="
            f"{build_age5_combined_heavy_policy_origin_trace_contract_compact_reason()}"
            not in digest_ok_text
        ):
            return fail("require-age5 pass case aggregate digest policy origin trace compact reason mismatch")
        if (
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY}="
            f"{build_age5_combined_heavy_policy_origin_trace_contract_compact_failure_reason()}"
            not in digest_ok_text
        ):
            return fail("require-age5 pass case aggregate digest policy origin trace compact failure reason mismatch")
        if f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}=ok" not in digest_ok_text:
            return fail("require-age5 pass case aggregate digest policy origin trace contract status mismatch")
        if f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}=1" not in digest_ok_text:
            return fail("require-age5 pass case aggregate digest policy origin trace contract ok mismatch")
        if f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}={expected_policy_origin_trace_text}" not in digest_ok_text:
            return fail("require-age5 pass case aggregate digest policy_origin_trace_text mismatch")
        if (
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY}="
            + json.dumps(expected_policy_origin_trace, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            not in digest_ok_text
        ):
            return fail("require-age5 pass case aggregate digest policy_origin_trace mismatch")
        if (
            f"age5_child_summary_defaults={expected_default_transport['ci_sanity_age5_combined_heavy_child_summary_default_fields']}"
            not in digest_ok_text
        ):
            return fail("require-age5 pass case aggregate digest child_summary_defaults mismatch")
        if (
            "age5_sync_child_summary_defaults="
            + expected_default_transport["ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"]
            not in digest_ok_text
        ):
            return fail("require-age5 pass case aggregate digest sync_child_summary_defaults mismatch")

        # case 1b: summary payload origin trace mismatch => fail
        age5_policy_summary.write_text(
            f"[age5-combined-heavy-policy] "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT} "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT} "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}=ok "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}=1 "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}=report_path=bad|report_exists=1|text_path=bad|text_exists=1|summary_path=bad|summary_exists=1 "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY}="
            + json.dumps(
                {
                    "report_exists": "1",
                    "report_path": "bad",
                    "summary_exists": "1",
                    "summary_path": "bad",
                    "text_exists": "1",
                    "text_path": "bad",
                },
                ensure_ascii=False,
                sort_keys=True,
                separators=(",", ":"),
            ),
            encoding="utf-8",
        )
        proc_mismatch = run_combine(
            seamgrim=seamgrim_report,
            age3=age3_report,
            age4=age4_report,
            age5=age5_report,
            age5_policy_report=age5_policy_report,
            age5_policy_text=age5_policy_text,
            age5_policy_summary=age5_policy_summary,
            oi=oi_report,
            out=out_report,
            require_age4=True,
            require_age5=True,
        )
        if proc_mismatch.returncode == 0:
            return fail("policy summary origin trace mismatch case must fail")
        mismatch_doc = read_json(out_report)
        if not isinstance(mismatch_doc, dict):
            return fail("policy summary origin trace mismatch case aggregate report missing")
        mismatch_age5 = mismatch_doc.get("age5")
        if not isinstance(mismatch_age5, dict) or bool(mismatch_age5.get("ok", True)):
            return fail("policy summary origin trace mismatch case age5.ok mismatch")
        mismatch_digest = [str(item) for item in mismatch_age5.get("failure_digest", [])]
        if not mismatch_digest or "policy_summary_origin_trace_mismatch" not in mismatch_digest[0]:
            return fail("policy summary origin trace mismatch case failure digest mismatch")
        if len(mismatch_digest) < 2 or "policy_summary_origin_trace_contract_mismatch" not in mismatch_digest[1]:
            return fail("policy summary origin trace mismatch case contract failure digest mismatch")
        if str(mismatch_age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY, "")).strip() != "mismatch":
            return fail("policy summary origin trace mismatch case contract status mismatch")
        if bool(mismatch_age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY, True)):
            return fail("policy summary origin trace mismatch case contract ok mismatch")
        if (
            str(mismatch_age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY, "")).strip()
            != "policy_summary_origin_trace_contract_mismatch"
        ):
            return fail("policy summary origin trace mismatch case contract issue mismatch")
        if (
            str(mismatch_age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY, "")).strip()
            != AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT
        ):
            return fail("policy summary origin trace mismatch case source issue mismatch")
        if (
            str(mismatch_age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY, "")).strip()
            != "issue=policy_summary_origin_trace_contract_mismatch"
        ):
            return fail("policy summary origin trace mismatch case compact reason mismatch")
        if (
            str(
                mismatch_age5.get(
                    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY, ""
                )
            ).strip()
            != "policy_summary_origin_trace_contract_mismatch"
        ):
            return fail("policy summary origin trace mismatch case compact failure reason mismatch")
        age5_policy_summary.write_text(
            f"[age5-combined-heavy-policy] age5_combined_heavy_policy_report_path={age5_policy_report} "
            f"age5_combined_heavy_policy_text_path={age5_policy_text} "
            "age5_combined_heavy_policy_report_exists=1 "
            "age5_combined_heavy_policy_text_exists=1 "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}="
            "policy_summary_origin_trace_contract_mismatch "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT} "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}="
            "issue=policy_summary_origin_trace_contract_mismatch "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}=ok "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}=1 "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}={expected_policy_origin_trace_text} "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY}="
            f'{json.dumps(expected_policy_origin_trace, ensure_ascii=False, sort_keys=True, separators=(",", ":"))}',
            encoding="utf-8",
        )

        # case 1c: summary payload issue token mismatch => fail
        age5_policy_summary.write_text(
            f"[age5-combined-heavy-policy] age5_combined_heavy_policy_report_path={age5_policy_report} "
            f"age5_combined_heavy_policy_text_path={age5_policy_text} "
            "age5_combined_heavy_policy_report_exists=1 "
            "age5_combined_heavy_policy_text_exists=1 "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}=BROKEN "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}=BROKEN "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}="
            "issue=policy_summary_origin_trace_contract_issue_mismatch|source=BROKEN "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}=ok "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}=1 "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}={expected_policy_origin_trace_text} "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY}="
            f'{json.dumps(expected_policy_origin_trace, ensure_ascii=False, sort_keys=True, separators=(",", ":"))}',
            encoding="utf-8",
        )
        proc_issue_mismatch = run_combine(
            seamgrim=seamgrim_report,
            age3=age3_report,
            age4=age4_report,
            age5=age5_report,
            age5_policy_report=age5_policy_report,
            age5_policy_text=age5_policy_text,
            age5_policy_summary=age5_policy_summary,
            oi=oi_report,
            out=out_report,
            require_age4=True,
            require_age5=True,
        )
        if proc_issue_mismatch.returncode == 0:
            return fail("policy summary origin trace issue mismatch case must fail")
        issue_mismatch_doc = read_json(out_report)
        if not isinstance(issue_mismatch_doc, dict):
            return fail("policy summary origin trace issue mismatch case aggregate report missing")
        issue_mismatch_age5 = issue_mismatch_doc.get("age5")
        if not isinstance(issue_mismatch_age5, dict) or bool(issue_mismatch_age5.get("ok", True)):
            return fail("policy summary origin trace issue mismatch case age5.ok mismatch")
        issue_mismatch_digest = [str(item) for item in issue_mismatch_age5.get("failure_digest", [])]
        if not issue_mismatch_digest or "policy_summary_origin_trace_contract_issue_mismatch" not in issue_mismatch_digest[0]:
            return fail("policy summary origin trace issue mismatch case failure digest mismatch")
        if (
            str(issue_mismatch_age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY, "")).strip()
            != "policy_summary_origin_trace_contract_issue_mismatch"
        ):
            return fail("policy summary origin trace issue mismatch case contract issue mismatch")
        if (
            str(issue_mismatch_age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY, "")).strip()
            != "BROKEN"
        ):
            return fail("policy summary origin trace issue mismatch case source issue mismatch")
        if (
            str(issue_mismatch_age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY, "")).strip()
            != "issue=policy_summary_origin_trace_contract_issue_mismatch|source=BROKEN"
        ):
            return fail("policy summary origin trace issue mismatch case compact reason mismatch")
        if (
            str(
                issue_mismatch_age5.get(
                    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY, ""
                )
            ).strip()
            != "policy_summary_origin_trace_contract_issue_mismatch"
        ):
            return fail("policy summary origin trace issue mismatch case compact failure reason mismatch")
        if str(issue_mismatch_age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY, "")).strip() != "ok":
            return fail("policy summary origin trace issue mismatch case contract status mismatch")
        if not bool(issue_mismatch_age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY, False)):
            return fail("policy summary origin trace issue mismatch case contract ok mismatch")

        # case 1d: summary payload compact reason mismatch => fail
        age5_policy_summary.write_text(
            f"[age5-combined-heavy-policy] age5_combined_heavy_policy_report_path={age5_policy_report} "
            f"age5_combined_heavy_policy_text_path={age5_policy_text} "
            "age5_combined_heavy_policy_report_exists=1 "
            "age5_combined_heavy_policy_text_exists=1 "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT} "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT} "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}=source=BROKEN "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}=ok "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}=1 "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}={expected_policy_origin_trace_text} "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY}="
            f'{json.dumps(expected_policy_origin_trace, ensure_ascii=False, sort_keys=True, separators=(",", ":"))}',
            encoding="utf-8",
        )
        proc_compact_reason_mismatch = run_combine(
            seamgrim=seamgrim_report,
            age3=age3_report,
            age4=age4_report,
            age5=age5_report,
            age5_policy_report=age5_policy_report,
            age5_policy_text=age5_policy_text,
            age5_policy_summary=age5_policy_summary,
            oi=oi_report,
            out=out_report,
            require_age4=True,
            require_age5=True,
        )
        if proc_compact_reason_mismatch.returncode == 0:
            return fail("policy summary origin trace compact reason mismatch case must fail")
        compact_reason_mismatch_doc = read_json(out_report)
        if not isinstance(compact_reason_mismatch_doc, dict):
            return fail("policy summary origin trace compact reason mismatch case aggregate report missing")
        compact_reason_mismatch_age5 = compact_reason_mismatch_doc.get("age5")
        if not isinstance(compact_reason_mismatch_age5, dict) or bool(compact_reason_mismatch_age5.get("ok", True)):
            return fail("policy summary origin trace compact reason mismatch case age5.ok mismatch")
        compact_reason_mismatch_digest = [
            str(item) for item in compact_reason_mismatch_age5.get("failure_digest", [])
        ]
        if (
            not compact_reason_mismatch_digest
            or "policy_summary_origin_trace_contract_compact_reason_mismatch"
            not in compact_reason_mismatch_digest[0]
        ):
            return fail("policy summary origin trace compact reason mismatch case failure digest mismatch")
        if (
            str(
                compact_reason_mismatch_age5.get(
                    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY, ""
                )
            ).strip()
            != "policy_summary_origin_trace_contract_compact_reason_mismatch"
        ):
            return fail("policy summary origin trace compact reason mismatch case contract issue mismatch")
        if (
            str(
                compact_reason_mismatch_age5.get(
                    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY, ""
                )
            ).strip()
            != AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT
        ):
            return fail("policy summary origin trace compact reason mismatch case source issue mismatch")
        if (
            str(
                compact_reason_mismatch_age5.get(
                    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY, ""
                )
            ).strip()
            != "issue=policy_summary_origin_trace_contract_compact_reason_mismatch"
        ):
            return fail("policy summary origin trace compact reason mismatch case compact reason mismatch")
        if (
            str(
                compact_reason_mismatch_age5.get(
                    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY, ""
                )
            ).strip()
            != "policy_summary_origin_trace_contract_compact_reason_mismatch"
        ):
            return fail("policy summary origin trace compact reason mismatch case compact failure reason mismatch")
        if str(compact_reason_mismatch_age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY, "")).strip() != "ok":
            return fail("policy summary origin trace compact reason mismatch case contract status mismatch")
        if not bool(compact_reason_mismatch_age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY, False)):
            return fail("policy summary origin trace compact reason mismatch case contract ok mismatch")
        age5_policy_summary.write_text(
            f"[age5-combined-heavy-policy] age5_combined_heavy_policy_report_path={age5_policy_report} "
            f"age5_combined_heavy_policy_text_path={age5_policy_text} "
            "age5_combined_heavy_policy_report_exists=1 "
            "age5_combined_heavy_policy_text_exists=1 "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT} "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT} "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}="
            f"{build_age5_combined_heavy_policy_origin_trace_contract_compact_reason()} "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}=ok "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}=1 "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}={expected_policy_origin_trace_text} "
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY}="
            f'{json.dumps(expected_policy_origin_trace, ensure_ascii=False, sort_keys=True, separators=(",", ":"))}',
            encoding="utf-8",
        )

        # case 2: require-age5 + missing age5 report => fail
        age5_report.unlink(missing_ok=True)
        proc_missing = run_combine(
            seamgrim=seamgrim_report,
            age3=age3_report,
            age4=age4_report,
            age5=age5_report,
            age5_policy_report=age5_policy_report,
            age5_policy_text=age5_policy_text,
            age5_policy_summary=age5_policy_summary,
            oi=oi_report,
            out=out_report,
            require_age4=True,
            require_age5=True,
        )
        if proc_missing.returncode == 0:
            return fail("require-age5 missing case must fail")
        miss_doc = read_json(out_report)
        if not isinstance(miss_doc, dict):
            return fail("require-age5 missing case report not generated")
        if bool(miss_doc.get("overall_ok", True)):
            return fail("require-age5 missing case overall_ok must be false")
        miss_age5 = miss_doc.get("age5")
        if not isinstance(miss_age5, dict) or bool(miss_age5.get("ok", True)):
            return fail("require-age5 missing case age5.ok must be false")
        if str(miss_age5.get("age5_combined_heavy_full_real_status", "")).strip() != "fail":
            return fail("require-age5 missing case full_real child status mismatch")
        if str(miss_age5.get("age5_combined_heavy_runtime_helper_negative_status", "")).strip() != "fail":
            return fail("require-age5 missing case runtime-helper child status mismatch")
        if str(miss_age5.get("age5_combined_heavy_group_id_summary_negative_status", "")).strip() != "fail":
            return fail("require-age5 missing case group-id child status mismatch")
        if str(miss_age5.get("age5_close_digest_selftest_ok", "")).strip() != "0":
            return fail("require-age5 missing case digest selftest status mismatch")
        if str(miss_age5.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
            return fail("require-age5 missing case digest selftest default text mismatch")
        if dict(miss_age5.get("combined_digest_selftest_default_field", {})) != expected_digest_default_field:
            return fail("require-age5 missing case digest selftest default field mismatch")
        if str(miss_age5.get("age5_policy_combined_digest_selftest_default_field_text", "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
            return fail("require-age5 missing case policy digest default text mismatch")
        if dict(miss_age5.get("age5_policy_combined_digest_selftest_default_field", {})) != expected_digest_default_field:
            return fail("require-age5 missing case policy digest default field mismatch")
        if dict(miss_age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY, {})) != expected_policy_origin_trace:
            return fail("require-age5 missing case policy origin trace mismatch")
        if str(miss_age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY, "")).strip() != expected_policy_origin_trace_text:
            return fail("require-age5 missing case policy origin trace text mismatch")
        for key, expected in expected_default_transport.items():
            if str(miss_age5.get(key, "")).strip() != expected:
                return fail(f"require-age5 missing case default transport mismatch: {key}")
        summary_missing = run_combine_print_summary(
            seamgrim=seamgrim_report,
            age3=age3_report,
            age4=age4_report,
            age5=age5_report,
            age5_policy_report=age5_policy_report,
            age5_policy_text=age5_policy_text,
            age5_policy_summary=age5_policy_summary,
            oi=oi_report,
            out=out_report,
            require_age4=True,
            require_age5=True,
        )
        if summary_missing.returncode == 0:
            return fail("require-age5 missing case print-summary must fail")
        summary_missing_text = str(summary_missing.stdout or "")
        if "age5_close_digest_selftest_ok=0" not in summary_missing_text:
            return fail("require-age5 missing case print-summary digest selftest mismatch")
        if f"{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}={AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT}" not in summary_missing_text:
            return fail("require-age5 missing case print-summary digest selftest default text mismatch")
        if (
            "combined_digest_selftest_default_field="
            + json.dumps(expected_digest_default_field, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            not in summary_missing_text
        ):
            return fail("require-age5 missing case print-summary digest selftest default field mismatch")
        if (
            "age5_policy_combined_digest_selftest_default_field_text=" + AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT
            not in summary_missing_text
        ):
            return fail("require-age5 missing case print-summary policy digest default text mismatch")
        if (
            "age5_policy_combined_digest_selftest_default_field="
            + json.dumps(expected_digest_default_field, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            not in summary_missing_text
        ):
            return fail("require-age5 missing case print-summary policy digest default field mismatch")
        if f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}={expected_policy_origin_trace_text}" not in summary_missing_text:
            return fail("require-age5 missing case print-summary policy origin trace text mismatch")
        if (
            f"child_summary_defaults={expected_default_transport['ci_sanity_age5_combined_heavy_child_summary_default_fields']}"
            not in summary_missing_text
        ):
            return fail("require-age5 missing case print-summary child_summary_defaults mismatch")
        if (
            "sync_child_summary_defaults="
            + expected_default_transport["ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"]
            not in summary_missing_text
        ):
            return fail("require-age5 missing case print-summary sync_child_summary_defaults mismatch")
        digest_missing = run_aggregate_digest(out_report)
        if digest_missing.returncode != 0:
            return fail(
                f"require-age5 missing case aggregate digest failed: out={digest_missing.stdout} err={digest_missing.stderr}"
            )
        digest_missing_text = str(digest_missing.stdout or "")
        if "age5_full_real_source_check=0" not in digest_missing_text:
            return fail("require-age5 missing case aggregate digest full-real source check mismatch")
        if "age5_full_real_source_selftest=0" not in digest_missing_text:
            return fail("require-age5 missing case aggregate digest full-real source selftest mismatch")
        if "age5_close_digest_selftest_ok=0" not in digest_missing_text:
            return fail("require-age5 missing case aggregate digest selftest mismatch")
        if f"{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}={AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT}" not in digest_missing_text:
            return fail("require-age5 missing case aggregate digest selftest default text mismatch")
        if (
            "combined_digest_selftest_default_field="
            + json.dumps(expected_digest_default_field, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            not in digest_missing_text
        ):
            return fail("require-age5 missing case aggregate digest selftest default field mismatch")
        if (
            "age5_policy_combined_digest_selftest_default_field_text=" + AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT
            not in digest_missing_text
        ):
            return fail("require-age5 missing case aggregate digest policy default text mismatch")
        if (
            "age5_policy_combined_digest_selftest_default_field="
            + json.dumps(expected_digest_default_field, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            not in digest_missing_text
        ):
            return fail("require-age5 missing case aggregate digest policy default field mismatch")
        if f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}={expected_policy_origin_trace_text}" not in digest_missing_text:
            return fail("require-age5 missing case aggregate digest policy origin trace text mismatch")
        if (
            f"age5_child_summary_defaults={expected_default_transport['ci_sanity_age5_combined_heavy_child_summary_default_fields']}"
            not in digest_missing_text
        ):
            return fail("require-age5 missing case aggregate digest child_summary_defaults mismatch")
        if (
            "age5_sync_child_summary_defaults="
            + expected_default_transport["ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"]
            not in digest_missing_text
        ):
            return fail("require-age5 missing case aggregate digest sync_child_summary_defaults mismatch")

        # case 3: optional age5 + missing age5 report => pass (skipped)
        proc_optional = run_combine(
            seamgrim=seamgrim_report,
            age3=age3_report,
            age4=age4_report,
            age5=age5_report,
            age5_policy_report=age5_policy_report,
            age5_policy_text=age5_policy_text,
            age5_policy_summary=age5_policy_summary,
            oi=oi_report,
            out=out_report,
            require_age4=True,
            require_age5=False,
        )
        if proc_optional.returncode != 0:
            return fail(f"optional-age5 missing case failed: out={proc_optional.stdout} err={proc_optional.stderr}")
        optional_doc = read_json(out_report)
        if not isinstance(optional_doc, dict) or not bool(optional_doc.get("overall_ok", False)):
            return fail("optional-age5 missing case overall_ok mismatch")
        optional_age5 = optional_doc.get("age5")
        if not isinstance(optional_age5, dict):
            return fail("optional-age5 missing case age5 block missing")
        if not bool(optional_age5.get("ok", False)) or not bool(optional_age5.get("skipped", False)):
            return fail("optional-age5 missing case skipped semantics mismatch")
        if str(optional_age5.get("age5_combined_heavy_full_real_status", "")).strip() != "skipped":
            return fail("optional-age5 missing case full_real child status mismatch")
        if str(optional_age5.get("age5_combined_heavy_runtime_helper_negative_status", "")).strip() != "skipped":
            return fail("optional-age5 missing case runtime-helper child status mismatch")
        if str(optional_age5.get("age5_combined_heavy_group_id_summary_negative_status", "")).strip() != "skipped":
            return fail("optional-age5 missing case group-id child status mismatch")
        if str(optional_age5.get("age5_close_digest_selftest_ok", "")).strip() != "0":
            return fail("optional-age5 missing case digest selftest status mismatch")
        if str(optional_age5.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
            return fail("optional-age5 missing case digest selftest default text mismatch")
        if dict(optional_age5.get("combined_digest_selftest_default_field", {})) != expected_digest_default_field:
            return fail("optional-age5 missing case digest selftest default field mismatch")
        if str(optional_age5.get("age5_policy_combined_digest_selftest_default_field_text", "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
            return fail("optional-age5 missing case policy digest default text mismatch")
        if dict(optional_age5.get("age5_policy_combined_digest_selftest_default_field", {})) != expected_digest_default_field:
            return fail("optional-age5 missing case policy digest default field mismatch")
        if dict(optional_age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY, {})) != expected_policy_origin_trace:
            return fail("optional-age5 missing case policy origin trace mismatch")
        if str(optional_age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY, "")).strip() != expected_policy_origin_trace_text:
            return fail("optional-age5 missing case policy origin trace text mismatch")
        for key, expected in expected_default_transport.items():
            if str(optional_age5.get(key, "")).strip() != expected:
                return fail(f"optional-age5 missing case default transport mismatch: {key}")
        summary_optional = run_combine_print_summary(
            seamgrim=seamgrim_report,
            age3=age3_report,
            age4=age4_report,
            age5=age5_report,
            age5_policy_report=age5_policy_report,
            age5_policy_text=age5_policy_text,
            age5_policy_summary=age5_policy_summary,
            oi=oi_report,
            out=out_report,
            require_age4=True,
            require_age5=False,
        )
        if summary_optional.returncode != 0:
            return fail(
                f"optional-age5 missing case print-summary failed: out={summary_optional.stdout} err={summary_optional.stderr}"
            )
        summary_optional_text = str(summary_optional.stdout or "")
        if "age5_close_digest_selftest_ok=0" not in summary_optional_text:
            return fail("optional-age5 missing case print-summary digest selftest mismatch")
        if f"{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}={AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT}" not in summary_optional_text:
            return fail("optional-age5 missing case print-summary digest selftest default text mismatch")
        if (
            "combined_digest_selftest_default_field="
            + json.dumps(expected_digest_default_field, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            not in summary_optional_text
        ):
            return fail("optional-age5 missing case print-summary digest selftest default field mismatch")
        if (
            "age5_policy_combined_digest_selftest_default_field_text=" + AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT
            not in summary_optional_text
        ):
            return fail("optional-age5 missing case print-summary policy digest default text mismatch")
        if (
            "age5_policy_combined_digest_selftest_default_field="
            + json.dumps(expected_digest_default_field, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            not in summary_optional_text
        ):
            return fail("optional-age5 missing case print-summary policy digest default field mismatch")
        if f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}={expected_policy_origin_trace_text}" not in summary_optional_text:
            return fail("optional-age5 missing case print-summary policy origin trace text mismatch")
        if (
            f"child_summary_defaults={expected_default_transport['ci_sanity_age5_combined_heavy_child_summary_default_fields']}"
            not in summary_optional_text
        ):
            return fail("optional-age5 missing case print-summary child_summary_defaults mismatch")
        if (
            "sync_child_summary_defaults="
            + expected_default_transport["ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"]
            not in summary_optional_text
        ):
            return fail("optional-age5 missing case print-summary sync_child_summary_defaults mismatch")
        digest_optional = run_aggregate_digest(out_report)
        if digest_optional.returncode != 0:
            return fail(
                f"optional-age5 missing case aggregate digest failed: out={digest_optional.stdout} err={digest_optional.stderr}"
            )
        digest_optional_text = str(digest_optional.stdout or "")
        if "age5_full_real_source_check=0" not in digest_optional_text:
            return fail("optional-age5 missing case aggregate digest full-real source check mismatch")
        if "age5_full_real_source_selftest=0" not in digest_optional_text:
            return fail("optional-age5 missing case aggregate digest full-real source selftest mismatch")
        if "age5_close_digest_selftest_ok=0" not in digest_optional_text:
            return fail("optional-age5 missing case aggregate digest selftest mismatch")
        if f"{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}={AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT}" not in digest_optional_text:
            return fail("optional-age5 missing case aggregate digest selftest default text mismatch")
        if (
            "combined_digest_selftest_default_field="
            + json.dumps(expected_digest_default_field, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            not in digest_optional_text
        ):
            return fail("optional-age5 missing case aggregate digest selftest default field mismatch")
        if (
            "age5_policy_combined_digest_selftest_default_field_text=" + AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT
            not in digest_optional_text
        ):
            return fail("optional-age5 missing case aggregate digest policy default text mismatch")
        if (
            "age5_policy_combined_digest_selftest_default_field="
            + json.dumps(expected_digest_default_field, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            not in digest_optional_text
        ):
            return fail("optional-age5 missing case aggregate digest policy default field mismatch")
        if f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}={expected_policy_origin_trace_text}" not in digest_optional_text:
            return fail("optional-age5 missing case aggregate digest policy origin trace text mismatch")
        if (
            f"age5_child_summary_defaults={expected_default_transport['ci_sanity_age5_combined_heavy_child_summary_default_fields']}"
            not in digest_optional_text
        ):
            return fail("optional-age5 missing case aggregate digest child_summary_defaults mismatch")
        if (
            "age5_sync_child_summary_defaults="
            + expected_default_transport["ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"]
            not in digest_optional_text
        ):
            return fail("optional-age5 missing case aggregate digest sync_child_summary_defaults mismatch")

    print("[ci-combine-age5-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
