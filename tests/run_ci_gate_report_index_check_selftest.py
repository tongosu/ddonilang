#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from ci_check_error_codes import GATE_REPORT_INDEX_CODES as CODES

REQUIRED_STEPS = (
    "ci_sanity_gate",
    "ci_sync_readiness_report_generate",
    "ci_sync_readiness_report_check",
    "seamgrim_wasm_cli_diag_parity_check",
)


def fail(msg: str) -> int:
    print(f"[ci-gate-report-index-selftest] fail: {msg}")
    return 1


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.rstrip() + "\n", encoding="utf-8")


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def run_check(index: Path, required_steps: tuple[str, ...] = ()) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_ci_gate_report_index_check.py",
        "--index",
        str(index),
    ]
    for step in required_steps:
        cmd.extend(["--required-step", step])
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def build_index_case(root: Path, case_name: str) -> Path:
    case_dir = root / case_name
    case_dir.mkdir(parents=True, exist_ok=True)

    summary = case_dir / "ci_gate_summary.txt"
    summary_line = case_dir / "ci_gate_summary_line.txt"
    result = case_dir / "ci_gate_result.detjson"
    badge = case_dir / "ci_gate_badge.detjson"
    brief = case_dir / "ci_fail_brief.txt"
    triage = case_dir / "ci_fail_triage.detjson"
    sanity = case_dir / "ci_sanity_gate.detjson"
    sync = case_dir / "ci_sync_readiness.detjson"
    parity = case_dir / "seamgrim_wasm_cli_diag_parity_report.detjson"
    index = case_dir / "ci_gate_report_index.detjson"

    write_text(summary, "[ci-gate-summary] PASS")
    write_text(summary_line, "status=pass reason=ok failed_steps=0")
    write_json(result, {"schema": "ddn.ci.gate_result.v1", "status": "pass", "reason": "ok", "failed_steps": 0})
    write_json(badge, {"schema": "ddn.ci.gate_badge.v1", "status": "pass", "label": "ci:pass"})
    write_text(brief, "status=pass reason=ok failed_steps_count=0")
    write_json(triage, {"schema": "ddn.ci.fail_triage.v1", "status": "pass", "reason": "ok"})
    write_json(sanity, {"schema": "ddn.ci.sanity_gate.v1", "status": "pass", "code": "OK", "step": "all", "steps": []})
    write_json(sync, {"schema": "ddn.ci.sync_readiness.v1", "status": "pass", "ok": True, "code": "OK", "step": "all", "steps": []})
    write_json(
        parity,
        {"schema": "ddn.seamgrim.wasm_cli_diag_parity.v1", "status": "pass", "ok": True, "code": "OK", "step": "all", "steps": []},
    )

    write_json(
        index,
        {
            "schema": "ddn.ci.aggregate_gate.index.v1",
            "reports": {
                "summary": str(summary),
                "summary_line": str(summary_line),
                "ci_gate_result_json": str(result),
                "ci_gate_badge_json": str(badge),
                "ci_fail_brief_txt": str(brief),
                "ci_fail_triage_json": str(triage),
                "ci_sanity_gate": str(sanity),
                "ci_sync_readiness": str(sync),
                "seamgrim_wasm_cli_diag_parity": str(parity),
            },
            "steps": [
                {"name": "ci_sanity_gate", "returncode": 0, "ok": True, "cmd": ["python", "tests/run_ci_sanity_gate.py"]},
                {
                    "name": "ci_sync_readiness_report_generate",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_sync_readiness_check.py"],
                },
                {
                    "name": "ci_sync_readiness_report_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_sync_readiness_report_check.py"],
                },
                {
                    "name": "seamgrim_wasm_cli_diag_parity_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_seamgrim_wasm_cli_diag_parity_check.py"],
                },
            ],
        },
    )
    return index


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="ci_gate_report_index_selftest_") as td:
        root = Path(td)

        ok_index = build_index_case(root, "ok")
        ok_proc = run_check(ok_index, REQUIRED_STEPS)
        if ok_proc.returncode != 0:
            return fail(f"ok case failed: out={ok_proc.stdout} err={ok_proc.stderr}")

        missing_key_index = build_index_case(root, "missing_key")
        missing_key_doc = json.loads(missing_key_index.read_text(encoding="utf-8"))
        missing_key_doc["reports"].pop("seamgrim_wasm_cli_diag_parity", None)
        write_json(missing_key_index, missing_key_doc)
        missing_key_proc = run_check(missing_key_index, REQUIRED_STEPS)
        if missing_key_proc.returncode == 0:
            return fail("missing key case must fail")
        if f"fail code={CODES['REPORT_KEY_MISSING']}" not in missing_key_proc.stderr:
            return fail(f"missing key code mismatch: err={missing_key_proc.stderr}")

        missing_path_index = build_index_case(root, "missing_path")
        missing_path_doc = json.loads(missing_path_index.read_text(encoding="utf-8"))
        missing_path_doc["reports"]["seamgrim_wasm_cli_diag_parity"] = str(root / "missing" / "parity.detjson")
        write_json(missing_path_index, missing_path_doc)
        missing_path_proc = run_check(missing_path_index, REQUIRED_STEPS)
        if missing_path_proc.returncode == 0:
            return fail("missing path case must fail")
        if f"fail code={CODES['REPORT_PATH_MISSING']}" not in missing_path_proc.stderr:
            return fail(f"missing path code mismatch: err={missing_path_proc.stderr}")

        bad_schema_index = build_index_case(root, "bad_schema")
        bad_schema_doc = json.loads(bad_schema_index.read_text(encoding="utf-8"))
        parity_path = Path(str(bad_schema_doc["reports"]["seamgrim_wasm_cli_diag_parity"]))
        write_json(parity_path, {"schema": "wrong.schema"})
        bad_schema_proc = run_check(bad_schema_index, REQUIRED_STEPS)
        if bad_schema_proc.returncode == 0:
            return fail("bad schema case must fail")
        if f"fail code={CODES['ARTIFACT_SCHEMA_MISMATCH']}" not in bad_schema_proc.stderr:
            return fail(f"bad schema code mismatch: err={bad_schema_proc.stderr}")

        missing_required_step_index = build_index_case(root, "missing_required_step")
        missing_required_step_doc = json.loads(missing_required_step_index.read_text(encoding="utf-8"))
        missing_required_step_doc["steps"] = [
            row for row in missing_required_step_doc["steps"] if row.get("name") != "ci_sync_readiness_report_check"
        ]
        write_json(missing_required_step_index, missing_required_step_doc)
        missing_required_step_proc = run_check(missing_required_step_index, REQUIRED_STEPS)
        if missing_required_step_proc.returncode == 0:
            return fail("missing required step case must fail")
        if f"fail code={CODES['REQUIRED_STEP_MISSING']}" not in missing_required_step_proc.stderr:
            return fail(f"missing required step code mismatch: err={missing_required_step_proc.stderr}")

        bad_step_shape_index = build_index_case(root, "bad_step_shape")
        bad_step_shape_doc = json.loads(bad_step_shape_index.read_text(encoding="utf-8"))
        bad_step_shape_doc["steps"][0] = "oops"
        write_json(bad_step_shape_index, bad_step_shape_doc)
        bad_step_shape_proc = run_check(bad_step_shape_index, REQUIRED_STEPS)
        if bad_step_shape_proc.returncode == 0:
            return fail("bad step shape case must fail")
        if f"fail code={CODES['STEP_ROW_TYPE']}" not in bad_step_shape_proc.stderr:
            return fail(f"bad step shape code mismatch: err={bad_step_shape_proc.stderr}")

    print("[ci-gate-report-index-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
