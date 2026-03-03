#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from ci_check_error_codes import SYNC_READINESS_REPORT_CODES as CODES


VALID_STATUS = {"pass", "fail"}
VALID_FAIL_CODES = {
    "E_SYNC_READINESS_STEP_FAIL",
    "E_SYNC_READINESS_SANITY_CONTRACT_FAIL",
    "E_SYNC_READINESS_VALIDATE_ONLY_PATH_MISSING",
}
BASE_STEP_PREFIX = [
    "pipeline_emit_flags_check",
    "pipeline_emit_flags_selftest",
    "sanity_gate_diagnostics_check",
    "sanity_gate",
]
VALID_SANITY_PROFILES = {"full", "core_lang", "seamgrim"}


def fail(msg: str, code: str = "E_CHECK") -> int:
    print(f"[ci-sync-readiness-report-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8-sig"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate ci_sync_readiness.detjson contract")
    parser.add_argument("--report", required=True, help="path to ci_sync_readiness.detjson")
    parser.add_argument("--require-pass", action="store_true", help="require status=pass")
    parser.add_argument(
        "--sanity-profile",
        choices=("full", "core_lang", "seamgrim"),
        default="",
        help="optional expected sanity profile",
    )
    args = parser.parse_args()

    report_path = Path(args.report)
    if not report_path.exists():
        return fail(f"missing report: {report_path}", code=CODES["REPORT_MISSING"])
    doc = load_json(report_path)
    if not isinstance(doc, dict):
        return fail(f"invalid json: {report_path}", code=CODES["JSON_INVALID"])

    if str(doc.get("schema", "")).strip() != "ddn.ci.sync_readiness.v1":
        return fail(f"schema mismatch: {doc.get('schema')}", code=CODES["SCHEMA"])

    status = str(doc.get("status", "")).strip()
    if status not in VALID_STATUS:
        return fail(f"invalid status: {status}", code=CODES["STATUS"])
    if args.require_pass and status != "pass":
        return fail(f"require-pass set but status={status}", code=CODES["STATUS"])

    ok_value = doc.get("ok")
    if not isinstance(ok_value, bool):
        return fail("ok must be bool", code=CODES["OK_TYPE"])
    if ok_value != (status == "pass"):
        return fail(f"status/ok mismatch status={status} ok={ok_value}", code=CODES["STATUS_OK_MISMATCH"])

    sanity_profile = str(doc.get("sanity_profile", "")).strip() or "full"
    if sanity_profile not in VALID_SANITY_PROFILES:
        return fail(f"invalid sanity_profile: {sanity_profile}", code=CODES["STATUS"])
    expected_sanity_profile = args.sanity_profile.strip()
    if expected_sanity_profile and sanity_profile != expected_sanity_profile:
        return fail(
            f"sanity_profile mismatch expected={expected_sanity_profile} actual={sanity_profile}",
            code=CODES["STATUS_OK_MISMATCH"],
        )

    code = str(doc.get("code", "")).strip()
    step = str(doc.get("step", "")).strip()
    msg = str(doc.get("msg", "")).strip()
    if not code:
        return fail("code missing", code=CODES["CODE"])
    if not step:
        return fail("step missing", code=CODES["STEP"])
    if not msg:
        return fail("msg missing", code=CODES["MSG"])

    steps = doc.get("steps")
    if not isinstance(steps, list):
        return fail("steps must be list", code=CODES["STEPS_TYPE"])
    try:
        steps_count = int(doc.get("steps_count", -1))
    except Exception:
        return fail("steps_count must be int", code=CODES["STEPS_COUNT"])
    if steps_count != len(steps):
        return fail(f"steps_count mismatch report={steps_count} actual={len(steps)}", code=CODES["STEPS_COUNT"])

    names: list[str] = []
    for idx, row in enumerate(steps):
        if not isinstance(row, dict):
            return fail(f"steps[{idx}] must be object", code=CODES["ROW_TYPE"])
        name = str(row.get("name", "")).strip()
        if not name:
            return fail(f"steps[{idx}] name missing", code=CODES["ROW_NAME"])
        if not isinstance(row.get("ok"), bool):
            return fail(f"steps[{idx}] ok must be bool", code=CODES["ROW_OK_TYPE"])
        try:
            int(row.get("returncode", -1))
        except Exception:
            return fail(f"steps[{idx}] returncode must be int", code=CODES["ROW_RC_TYPE"])
        names.append(name)

    validate_only_path = str(doc.get("validate_only_sanity_json", "")).strip()
    has_contract_row = "sanity_gate_contract" in names
    if not has_contract_row:
        return fail("missing sanity_gate_contract row", code=CODES["MISSING_CONTRACT_ROW"])

    if status == "pass":
        if code != "OK" or step != "all" or msg != "-":
            return fail(
                f"pass fields invalid code={code} step={step} msg={msg}",
                code=CODES["PASS_STATUS_FIELDS"],
            )
        for idx, row in enumerate(steps):
            if not bool(row.get("ok", False)):
                return fail(f"pass row must be ok=1 idx={idx}", code=CODES["PASS_ROW_FAIL"])
            if int(row.get("returncode", 1)) != 0:
                return fail(
                    f"pass row returncode must be 0 idx={idx} rc={row.get('returncode')}",
                    code=CODES["PASS_ROW_FAIL"],
                )
    else:
        if code not in VALID_FAIL_CODES:
            return fail(f"invalid fail code: {code}", code=CODES["FAIL_STATUS_FIELDS"])
        if step == "all":
            return fail("fail step must not be all", code=CODES["FAIL_STATUS_FIELDS"])

    if validate_only_path:
        if len(steps) != 1 or names != ["sanity_gate_contract"]:
            return fail(
                f"validate-only shape invalid steps={names}",
                code=CODES["VALIDATE_ONLY_SHAPE"],
            )
    else:
        if status == "pass":
            if len(names) < 5:
                return fail(f"pass non-validate steps too small: {len(names)}", code=CODES["QUICK_BASE_STEPS"])
            if names[:4] != BASE_STEP_PREFIX:
                return fail(
                    f"base steps mismatch got={names[:4]}",
                    code=CODES["QUICK_BASE_STEPS"],
                )

    print(
        f"[ci-sync-readiness-report-check] ok report={report_path} "
        f"status={status} sanity_profile={sanity_profile} steps={len(steps)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
