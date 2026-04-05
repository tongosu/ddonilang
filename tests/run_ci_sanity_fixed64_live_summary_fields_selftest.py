#!/usr/bin/env python
from __future__ import annotations

import argparse
import importlib.util
import json
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
LIVE_SCHEMA = "ddn.fixed64.darwin_real_report_live_check.v1"
MODULE_NAME = "_ddn_ci_sanity_gate_fixed64_live_summary_selftest"


def load_ci_sanity_gate_module():
    script_path = ROOT / "tests" / "run_ci_sanity_gate.py"
    spec = importlib.util.spec_from_file_location(MODULE_NAME, script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"unable to load module: {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def make_live_report(*, status: str, resolved_status: str, resolved_source: str, invalid_hits: list[str]) -> dict:
    return {
        "schema": LIVE_SCHEMA,
        "generated_at_utc": "2026-03-27T00:00:00Z",
        "ok": status in {"pass_3way", "pass_contract_only", "skip_disabled"},
        "status": status,
        "reason": "-",
        "enabled_env_key": "DDN_ENABLE_DARWIN_PROBE",
        "enabled_env_value": "1",
        "enabled": True,
        "allow_pass_contract_only": False,
        "reports": {
            "darwin": "build/reports/fixed64_cross_platform_probe_darwin.detjson",
            "windows": "build/reports/fixed64_cross_platform_probe_windows.detjson",
            "linux": "build/reports/fixed64_cross_platform_probe_linux.detjson",
            "inputs": "build/reports/fixed64_threeway_inputs.live.detjson",
            "resolve_out": "build/reports/fixed64_threeway_inputs.live.detjson",
            "readiness_out": "build/reports/fixed64_darwin_real_report_live_check.readiness.detjson",
        },
        "resolve_candidates": [],
        "resolved_status": resolved_status,
        "resolved_source": resolved_source,
        "resolve_invalid_hits": invalid_hits,
        "readiness": {},
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="ci_sanity fixed64 live summary fields selftest")
    parser.add_argument("--report", required=True, help="fixed64 live report path produced by ci_sanity step")
    args = parser.parse_args()

    report_path = Path(args.report).resolve()
    report_doc = load_json(report_path)
    if not isinstance(report_doc, dict):
        print("[ci-sanity-fixed64-live-summary-selftest] report missing/invalid", file=sys.stderr)
        return 1
    if str(report_doc.get("schema", "")).strip() != LIVE_SCHEMA:
        print("[ci-sanity-fixed64-live-summary-selftest] report schema mismatch", file=sys.stderr)
        return 1

    module = load_ci_sanity_gate_module()
    rows = [
        {
            "step": "fixed64_darwin_real_report_live_check",
            "ok": True,
            "returncode": 0,
        }
    ]
    completion_gate_reports = {"fixed64_live_report": str(report_path)}
    fields = module.build_sanity_summary_fields("core_lang", rows, completion_gate_reports)
    expected_status = str(report_doc.get("status", "")).strip() or "-"
    expected_resolved_status = str(report_doc.get("resolved_status", "")).strip() or "-"
    expected_resolved_source = str(report_doc.get("resolved_source", "")).strip() or "-"
    expected_invalid_count = str(len([str(item).strip() for item in list(report_doc.get("resolve_invalid_hits", [])) if str(item).strip()]))
    expected_zip = "1" if ".zip!" in expected_resolved_source else "0"

    checks = {
        "ci_sanity_fixed64_darwin_real_report_live_report_path": str(report_path),
        "ci_sanity_fixed64_darwin_real_report_live_report_exists": "1",
        "ci_sanity_fixed64_darwin_real_report_live_status": expected_status,
        "ci_sanity_fixed64_darwin_real_report_live_resolved_status": expected_resolved_status,
        "ci_sanity_fixed64_darwin_real_report_live_resolved_source": expected_resolved_source or "-",
        "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count": expected_invalid_count,
        "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip": expected_zip,
        "ci_sanity_fixed64_darwin_real_report_live_check_selftest_ok": "pending",
    }
    for key, expected in checks.items():
        actual = str(fields.get(key, "")).strip()
        if actual != str(expected):
            print(f"[ci-sanity-fixed64-live-summary-selftest] mismatch {key}: {actual!r} != {expected!r}", file=sys.stderr)
            return 1

    # pending path contract
    pending_fields = module.build_sanity_summary_fields("core_lang", [], completion_gate_reports)
    pending_checks = {
        "ci_sanity_fixed64_darwin_real_report_live_report_path": str(report_path),
        "ci_sanity_fixed64_darwin_real_report_live_report_exists": "pending",
        "ci_sanity_fixed64_darwin_real_report_live_status": "pending",
        "ci_sanity_fixed64_darwin_real_report_live_resolved_status": "pending",
        "ci_sanity_fixed64_darwin_real_report_live_resolved_source": "pending",
        "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count": "pending",
        "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip": "pending",
        "ci_sanity_fixed64_darwin_real_report_live_check_selftest_ok": "pending",
    }
    for key, expected in pending_checks.items():
        actual = str(pending_fields.get(key, "")).strip()
        if actual != str(expected):
            print(f"[ci-sanity-fixed64-live-summary-selftest] pending mismatch {key}: {actual!r} != {expected!r}", file=sys.stderr)
            return 1

    # zip marker regression with isolated fixture
    with tempfile.TemporaryDirectory(prefix="ci_sanity_fixed64_live_summary_selftest_") as tmp:
        tmp_report = Path(tmp) / "live_report_zip.detjson"
        write_json(
            tmp_report,
            make_live_report(
                status="pass_3way",
                resolved_status="staged",
                resolved_source=str(Path(tmp) / "fixed64_darwin_probe_artifact.zip") + "!nested/fixed64_cross_platform_probe_darwin.detjson",
                invalid_hits=[],
            ),
        )
        zip_fields = module.build_sanity_summary_fields(
            "core_lang",
            rows,
            {"fixed64_live_report": str(tmp_report)},
        )
        if str(zip_fields.get("ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip", "")).strip() != "1":
            print("[ci-sanity-fixed64-live-summary-selftest] zip marker regression", file=sys.stderr)
            return 1

    # selftest step summary flag regression
    selftest_fields = module.build_sanity_summary_fields(
        "core_lang",
        [
            {"step": "fixed64_darwin_real_report_live_check", "ok": True, "returncode": 0},
            {"step": "fixed64_darwin_real_report_live_check_selftest", "ok": True, "returncode": 0},
        ],
        completion_gate_reports,
    )
    if str(selftest_fields.get("ci_sanity_fixed64_darwin_real_report_live_check_selftest_ok", "")).strip() != "1":
        print("[ci-sanity-fixed64-live-summary-selftest] live selftest summary flag regression", file=sys.stderr)
        return 1

    print("[ci-sanity-fixed64-live-summary-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
