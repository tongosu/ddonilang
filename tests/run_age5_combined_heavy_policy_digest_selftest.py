#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from _ci_age5_combined_heavy_contract import (
    AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
    AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
    AGE5_COMBINED_HEAVY_POLICY_DIGEST_SCRIPT,
    AGE5_COMBINED_HEAVY_POLICY_SCHEMA,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_TEXT,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_DEFAULT,
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT,
    AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT,
    build_age4_proof_snapshot,
    build_age4_proof_snapshot_text,
    build_age4_proof_source_snapshot_fields,
    build_age5_combined_heavy_policy_origin_trace,
    build_age5_combined_heavy_policy_origin_trace_text,
)

AGE5_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY = "age5_policy_summary_origin_trace_contract_issue"
# split-contract token anchor: age5_policy_summary_origin_trace_contract_source_issue
# split-contract token anchor: age5_policy_summary_origin_trace_contract_compact_reason
# split-contract token anchor: age5_policy_summary_origin_trace_contract_compact_failure_reason


def fail(msg: str) -> int:
    print(f"[age5-combined-heavy-policy-digest-selftest] fail: {msg}")
    return 1


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.rstrip() + "\n", encoding="utf-8")


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    script = root / AGE5_COMBINED_HEAVY_POLICY_DIGEST_SCRIPT

    with tempfile.TemporaryDirectory(prefix="age5_combined_heavy_policy_digest_") as td:
        temp_root = Path(td)
        report = temp_root / "age5_combined_heavy_policy.detjson"
        text_path = temp_root / "age5_combined_heavy_policy.txt"
        summary_path = temp_root / "age5_combined_heavy_policy_summary.txt"
        write_json(
            report,
            {
                "schema": AGE5_COMBINED_HEAVY_POLICY_SCHEMA,
                "provider": "gitlab",
                "enabled": True,
                "reason": "manual_optin",
                "scope": "nightly_manual_only",
                "combined_timeout_mode_default": AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED,
                "combined_timeout_mode_allowed_values": AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_TEXT,
                "combined_timeout_mode_preview_only": AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_DEFAULT,
                "combined_timeout_mode_scope": "nightly_manual_only",
                "combined_timeout_requires_combined_optin": AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT,
                "combined_timeout_policy_reason": AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT,
                **build_age4_proof_snapshot(),
                **build_age4_proof_source_snapshot_fields(top_snapshot=build_age4_proof_snapshot()),
                "age4_proof_snapshot_fields_text": AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
                "age4_proof_source_snapshot_fields_text": AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
                "age4_proof_snapshot_text": build_age4_proof_snapshot_text(build_age4_proof_snapshot()),
                "combined_digest_selftest_default_field_text": AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
                "combined_digest_selftest_default_field": {
                    "age5_close_digest_selftest_ok": "0",
                },
            },
        )
        write_text(text_path, "provider=gitlab status=enabled")

        proc_ok = run(
            [py, str(script), str(report), "--policy-text", str(text_path), "--summary-out", str(summary_path)],
            cwd=root,
        )
        if proc_ok.returncode != 0:
            return fail(f"ok case rc={proc_ok.returncode} err={proc_ok.stderr}")
        line = str(proc_ok.stdout or "").strip()
        origin_trace = build_age5_combined_heavy_policy_origin_trace(
            report_path=report,
            report_exists=True,
            text_path=text_path,
            text_exists=True,
            summary_path=summary_path,
            summary_exists=False,
        )
        for token in (
            "[age5-combined-heavy-policy] provider=gitlab enabled=1 reason=manual_optin scope=nightly_manual_only",
            f"age5_policy_combined_timeout_mode_default={AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED}",
            f"age5_policy_combined_timeout_mode_allowed_values={AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_TEXT}",
            f"age5_policy_combined_timeout_mode_preview_only={AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_DEFAULT}",
            "age5_policy_combined_timeout_mode_scope=nightly_manual_only",
            f"age5_policy_combined_timeout_requires_optin={AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT}",
            f"age5_policy_combined_timeout_policy_reason={AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT}",
            f"age5_policy_age4_proof_snapshot_fields_text={AGE4_PROOF_SNAPSHOT_FIELDS_TEXT}",
            f"age5_policy_age4_proof_source_snapshot_fields_text={AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT}",
            "age5_policy_age4_proof_snapshot_text=" + build_age4_proof_snapshot_text(build_age4_proof_snapshot()),
            "age5_policy_age4_proof_gate_result_present=0",
            "age5_policy_age4_proof_gate_result_parity=0",
            "age5_policy_age4_proof_final_status_parse_present=0",
            "age5_policy_age4_proof_final_status_parse_parity=0",
            f"age5_policy_combined_digest_selftest_default_field_text={AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT}",
            'age5_policy_combined_digest_selftest_default_field={"age5_close_digest_selftest_ok":"0"}',
            f"age5_combined_heavy_policy_report_path={report}",
            "age5_combined_heavy_policy_report_exists=1",
            f"age5_combined_heavy_policy_text_path={text_path}",
            "age5_combined_heavy_policy_text_exists=1",
            f"age5_combined_heavy_policy_summary_path={summary_path}",
            "age5_combined_heavy_policy_summary_exists=0",
            f"{AGE5_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT}",
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT}",
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT}",
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}=ok",
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}=1",
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}="
            f"{build_age5_combined_heavy_policy_origin_trace_text(origin_trace)}",
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY}="
            f'{json.dumps(origin_trace, ensure_ascii=False, sort_keys=True, separators=(",", ":"))}',
        ):
            if token not in line:
                return fail(f"ok case missing token: {token}")

        missing_text = temp_root / "missing.txt"
        proc_missing_text = run(
            [py, str(script), str(report), "--policy-text", str(missing_text), "--summary-out", str(summary_path)],
            cwd=root,
        )
        if proc_missing_text.returncode != 0:
            return fail(f"missing_text case rc={proc_missing_text.returncode} err={proc_missing_text.stderr}")
        line_missing = str(proc_missing_text.stdout or "").strip()
        if "age5_combined_heavy_policy_text_exists=0" not in line_missing:
            return fail("missing_text case missing text_exists=0")
        if f"age5_combined_heavy_policy_summary_path={summary_path}" not in line_missing:
            return fail("missing_text case missing summary_path")
        if f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}=report_path={report}" not in line_missing:
            return fail("missing_text case missing origin_trace_text prefix")

        summary_path.write_text(line + "\n", encoding="utf-8")
        proc_existing_summary = run(
            [py, str(script), str(report), "--policy-text", str(text_path), "--summary-out", str(summary_path)],
            cwd=root,
        )
        if proc_existing_summary.returncode != 0:
            return fail(f"existing_summary case rc={proc_existing_summary.returncode} err={proc_existing_summary.stderr}")
        line_existing = str(proc_existing_summary.stdout or "").strip()
        if "age5_combined_heavy_policy_summary_exists=1" not in line_existing:
            return fail("existing_summary case missing summary_exists=1")
        if (
            f"{AGE5_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT}"
            not in line_existing
        ):
            return fail("existing_summary case missing contract_issue=-")
        if (
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT}"
            not in line_existing
        ):
            return fail("existing_summary case missing contract_source_issue=-")
        if (
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}="
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT}"
            not in line_existing
        ):
            return fail("existing_summary case missing compact_reason=-")
        if f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}=ok" not in line_existing:
            return fail("existing_summary case missing contract_status=ok")
        if f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}=1" not in line_existing:
            return fail("existing_summary case missing contract_ok=1")
        if f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}=report_path={report}" not in line_existing:
            return fail("existing_summary case missing origin_trace_text prefix")

        bad_report = temp_root / "bad.detjson"
        write_text(bad_report, "{broken")
        proc_bad = run(
            [py, str(script), str(bad_report), "--policy-text", str(text_path), "--summary-out", str(summary_path)],
            cwd=root,
        )
        if proc_bad.returncode == 0:
            return fail("bad report case must fail")

    print("[age5-combined-heavy-policy-digest-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
