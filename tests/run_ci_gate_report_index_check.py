#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from ci_check_error_codes import GATE_REPORT_INDEX_CODES as CODES

INDEX_SCHEMA = "ddn.ci.aggregate_gate.index.v1"

REQUIRED_REPORT_PATH_KEYS = (
    "summary",
    "summary_line",
    "ci_gate_result_json",
    "ci_gate_badge_json",
    "ci_fail_brief_txt",
    "ci_fail_triage_json",
    "ci_sanity_gate",
    "ci_sync_readiness",
    "seamgrim_wasm_cli_diag_parity",
)

ARTIFACT_SCHEMA_MAP = {
    "ci_gate_result_json": "ddn.ci.gate_result.v1",
    "ci_sanity_gate": "ddn.ci.sanity_gate.v1",
    "ci_sync_readiness": "ddn.ci.sync_readiness.v1",
    "seamgrim_wasm_cli_diag_parity": "ddn.seamgrim.wasm_cli_diag_parity.v1",
}


def fail(msg: str, code: str) -> int:
    print(f"[ci-gate-report-index-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def resolve_report_path(index_doc: dict, key: str) -> Path | None:
    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return None
    raw = str(reports.get(key, "")).strip()
    if not raw:
        return None
    return Path(raw.replace("\\", "/"))


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate aggregate gate report-index schema and report paths")
    parser.add_argument("--index", required=True, help="path to ci_gate_report_index.detjson")
    parser.add_argument(
        "--required-step",
        action="append",
        default=[],
        help="required step name in index.steps (can be repeated)",
    )
    args = parser.parse_args()

    index_path = Path(args.index)
    if not index_path.exists():
        return fail(f"missing index file: {index_path}", CODES["INDEX_MISSING"])
    index_doc = load_json(index_path)
    if not isinstance(index_doc, dict):
        return fail(f"invalid index json: {index_path}", CODES["INDEX_JSON_INVALID"])
    if str(index_doc.get("schema", "")).strip() != INDEX_SCHEMA:
        return fail(
            f"index schema mismatch: {index_doc.get('schema')}",
            CODES["INDEX_SCHEMA"],
        )

    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return fail("index.reports is missing", CODES["INDEX_REPORTS_MISSING"])

    steps = index_doc.get("steps")
    if steps is None:
        return fail("index.steps is missing", CODES["STEPS_MISSING"])
    if not isinstance(steps, list):
        return fail("index.steps must be list", CODES["STEPS_TYPE"])
    seen_step_names: set[str] = set()
    for idx, row in enumerate(steps):
        if not isinstance(row, dict):
            return fail(f"index.steps[{idx}] must be object", CODES["STEP_ROW_TYPE"])
        step_name = str(row.get("name", "")).strip()
        if not step_name:
            return fail(f"index.steps[{idx}].name missing", CODES["STEP_NAME"])
        if step_name in seen_step_names:
            return fail(f"index.steps duplicate name: {step_name}", CODES["STEP_DUP"])
        seen_step_names.add(step_name)
        ok_value = row.get("ok")
        if not isinstance(ok_value, bool):
            return fail(f"index.steps[{idx}].ok must be bool", CODES["STEP_OK_TYPE"])
        try:
            int(row.get("returncode"))
        except Exception:
            return fail(f"index.steps[{idx}].returncode must be int", CODES["STEP_RC_TYPE"])
        cmd_value = row.get("cmd")
        if not isinstance(cmd_value, list):
            return fail(f"index.steps[{idx}].cmd must be list", CODES["STEP_CMD_TYPE"])

    for key in REQUIRED_REPORT_PATH_KEYS:
        path = resolve_report_path(index_doc, key)
        if path is None:
            return fail(f"missing index reports key/path: {key}", CODES["REPORT_KEY_MISSING"])
        if not path.exists():
            return fail(f"missing report path for {key}: {path}", CODES["REPORT_PATH_MISSING"])

    for key, expected_schema in ARTIFACT_SCHEMA_MAP.items():
        artifact_path = resolve_report_path(index_doc, key)
        if artifact_path is None:
            return fail(f"missing artifact path key: {key}", CODES["REPORT_KEY_MISSING"])
        artifact_doc = load_json(artifact_path)
        if not isinstance(artifact_doc, dict):
            return fail(
                f"artifact json invalid key={key} path={artifact_path}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
        actual_schema = str(artifact_doc.get("schema", "")).strip()
        if actual_schema != expected_schema:
            return fail(
                f"artifact schema mismatch key={key} schema={actual_schema} expected={expected_schema}",
                CODES["ARTIFACT_SCHEMA_MISMATCH"],
            )

    required_steps = [str(item).strip() for item in args.required_step if str(item).strip()]
    missing_required_steps = [name for name in required_steps if name not in seen_step_names]
    if missing_required_steps:
        return fail(
            f"missing required index step(s): {','.join(missing_required_steps)}",
            CODES["REQUIRED_STEP_MISSING"],
        )

    print(f"[ci-gate-report-index-check] ok index={index_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
