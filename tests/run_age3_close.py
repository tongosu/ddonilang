#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


def safe_print(text: str, *, to_stderr: bool = False) -> None:
    stream = sys.stderr if to_stderr else sys.stdout
    data = str(text)
    try:
        print(data, end="", file=stream)
        return
    except UnicodeEncodeError:
        pass
    encoded = data.encode(stream.encoding or "utf-8", errors="replace")
    print(encoded.decode(stream.encoding or "utf-8", errors="replace"), end="", file=stream)


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def clip(text: str, limit: int = 140) -> str:
    normalized = " ".join(text.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def default_report_path(file_name: str) -> str:
    preferred = Path("I:/home/urihanl/ddn/codex/build/reports")
    if os.name == "nt":
        try:
            preferred.mkdir(parents=True, exist_ok=True)
        except OSError:
            pass
        return str(preferred / file_name)
    return f"build/reports/{file_name}"


def run_seamgrim_gate(root: Path, seamgrim_report: Path, ui_report: Path) -> int:
    cmd = [
        sys.executable,
        "tests/run_seamgrim_ci_gate.py",
        "--strict-graph",
        "--require-promoted",
        "--json-out",
        str(seamgrim_report),
        "--ui-age3-json-out",
        str(ui_report),
    ]
    print(f"[age3-close] run: {' '.join(cmd)}")
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.stdout:
        safe_print(proc.stdout)
    if proc.stderr:
        safe_print(proc.stderr, to_stderr=True)
    print(f"[age3-close] seamgrim_exit={proc.returncode}")
    return int(proc.returncode)


def find_step_ok(seamgrim_doc: dict, step_name: str) -> bool:
    steps = seamgrim_doc.get("steps")
    if not isinstance(steps, list):
        return False
    for row in steps:
        if isinstance(row, dict) and str(row.get("name", "")) == step_name:
            return bool(row.get("ok", False))
    return False


def find_step_row(seamgrim_doc: dict, step_name: str) -> dict | None:
    steps = seamgrim_doc.get("steps")
    if not isinstance(steps, list):
        return None
    for row in steps:
        if isinstance(row, dict) and str(row.get("name", "")) == step_name:
            return row
    return None


def resolve_schema_gate_ok(seamgrim_doc: dict | None) -> tuple[bool, str]:
    if not isinstance(seamgrim_doc, dict):
        return (False, "step:schema_gate missing")
    legacy_row = find_step_row(seamgrim_doc, "schema_gate")
    if isinstance(legacy_row, dict):
        ok = bool(legacy_row.get("ok", False))
        return (ok, f"step:schema_gate ok={int(ok)}")
    release_names = (
        "schema_realign_formula_compat",
        "schema_upgrade_formula_compat",
        "formula_compat",
    )
    rows: list[dict] = []
    for name in release_names:
        row = find_step_row(seamgrim_doc, name)
        if not isinstance(row, dict):
            return (False, f"steps:{'+'.join(release_names)} missing={name}")
        rows.append(row)
    ok = all(bool(row.get("ok", False)) for row in rows)
    return (ok, f"steps:{'+'.join(release_names)} ok={int(ok)}")


def resolve_full_check_ok(seamgrim_doc: dict | None) -> tuple[bool, str]:
    if not isinstance(seamgrim_doc, dict):
        return (False, "step:full_check missing")
    legacy_row = find_step_row(seamgrim_doc, "full_check")
    if isinstance(legacy_row, dict):
        ok = bool(legacy_row.get("ok", False))
        return (ok, f"step:full_check ok={int(ok)}")
    frontdoor_row = find_step_row(seamgrim_doc, "frontdoor_strict_all")
    if not isinstance(frontdoor_row, dict):
        return (False, "step:full_check/frontdoor_strict_all missing")
    ok = bool(frontdoor_row.get("ok", False))
    return (ok, f"step:frontdoor_strict_all ok={int(ok)}")


def build_report(
    seamgrim_doc: dict | None,
    ui_doc: dict | None,
    seamgrim_report: Path,
    ui_report: Path,
) -> dict:
    seamgrim_ok = bool(seamgrim_doc.get("ok", False)) if isinstance(seamgrim_doc, dict) else False
    schema_gate_ok, schema_gate_detail = resolve_schema_gate_ok(seamgrim_doc)
    ui_gate_step_ok = find_step_ok(seamgrim_doc, "ui_age3_gate") if isinstance(seamgrim_doc, dict) else False
    space2d_source_ui_gate_ok = (
        find_step_ok(seamgrim_doc, "space2d_source_ui_gate") if isinstance(seamgrim_doc, dict) else False
    )
    full_check_ok, full_check_detail = resolve_full_check_ok(seamgrim_doc)
    export_pre_ok = find_step_ok(seamgrim_doc, "export_graph_preprocess") if isinstance(seamgrim_doc, dict) else False
    diag_ok = find_step_ok(seamgrim_doc, "ci_gate_diagnostics") if isinstance(seamgrim_doc, dict) else False
    require_promoted = bool(seamgrim_doc.get("require_promoted", False)) if isinstance(seamgrim_doc, dict) else False
    strict_graph = bool(seamgrim_doc.get("strict_graph", False)) if isinstance(seamgrim_doc, dict) else False

    ui_report_ok = bool(ui_doc.get("ok", False)) if isinstance(ui_doc, dict) else False
    ui_check_rows = ui_doc.get("checks") if isinstance(ui_doc, dict) else None
    ui_failed_checks = 0
    if isinstance(ui_check_rows, list):
        ui_failed_checks = sum(1 for row in ui_check_rows if isinstance(row, dict) and not bool(row.get("ok", False)))

    criteria = [
        {
            "name": "seamgrim_gate_ok",
            "ok": seamgrim_ok,
            "detail": f"seamgrim.ci_gate.v1 ok={int(seamgrim_ok)}",
        },
        {
            "name": "require_promoted_on",
            "ok": require_promoted,
            "detail": f"require_promoted={int(require_promoted)}",
        },
        {
            "name": "strict_graph_on",
            "ok": strict_graph,
            "detail": f"strict_graph={int(strict_graph)}",
        },
        {
            "name": "schema_gate_ok",
            "ok": schema_gate_ok,
            "detail": schema_gate_detail,
        },
        {
            "name": "ui_age3_gate_step_ok",
            "ok": ui_gate_step_ok,
            "detail": f"step:ui_age3_gate ok={int(ui_gate_step_ok)}",
        },
        {
            "name": "space2d_source_ui_gate_ok",
            "ok": space2d_source_ui_gate_ok,
            "detail": f"step:space2d_source_ui_gate ok={int(space2d_source_ui_gate_ok)}",
        },
        {
            "name": "ui_age3_report_ok",
            "ok": ui_report_ok,
            "detail": f"seamgrim.ui_age3_gate.v1 ok={int(ui_report_ok)} failed_checks={ui_failed_checks}",
        },
        {
            "name": "export_graph_preprocess_ok",
            "ok": export_pre_ok,
            "detail": f"step:export_graph_preprocess ok={int(export_pre_ok)}",
        },
        {
            "name": "full_check_ok",
            "ok": full_check_ok,
            "detail": full_check_detail,
        },
        {
            "name": "ci_gate_diagnostics_ok",
            "ok": diag_ok,
            "detail": f"step:ci_gate_diagnostics ok={int(diag_ok)}",
        },
    ]
    overall_ok = all(bool(row["ok"]) for row in criteria)

    failure_digest: list[str] = []
    for row in criteria:
        if bool(row["ok"]):
            continue
        failure_digest.append(f"{row['name']}: {clip(str(row['detail']))}")

    seamgrim_digest = seamgrim_doc.get("failure_digest") if isinstance(seamgrim_doc, dict) else None
    if isinstance(seamgrim_digest, list):
        for line in seamgrim_digest[:3]:
            failure_digest.append(f"seamgrim: {clip(str(line))}")

    ui_digest = ui_doc.get("failure_digest") if isinstance(ui_doc, dict) else None
    if isinstance(ui_digest, list):
        for line in ui_digest[:3]:
            failure_digest.append(f"ui_age3: {clip(str(line))}")

    return {
        "schema": "ddn.seamgrim.age3_close_report.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "overall_ok": overall_ok,
        "criteria": criteria,
        "seamgrim_report_path": str(seamgrim_report),
        "ui_age3_report_path": str(ui_report),
        "failure_digest": failure_digest[:16],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Evaluate AGE3 close criteria from seamgrim reports")
    parser.add_argument(
        "--seamgrim-report",
        default=default_report_path("seamgrim_ci_gate_report.json"),
        help="path to seamgrim.ci_gate.v1 report",
    )
    parser.add_argument(
        "--ui-age3-report",
        default=default_report_path("seamgrim_ui_age3_gate_report.detjson"),
        help="path to seamgrim.ui_age3_gate.v1 report",
    )
    parser.add_argument(
        "--report-out",
        default=default_report_path("age3_close_report.detjson"),
        help="output age3 close report path",
    )
    parser.add_argument(
        "--run-seamgrim",
        action="store_true",
        help="run seamgrim gate first and then evaluate close criteria",
    )
    parser.add_argument(
        "--run-age3",
        action="store_true",
        help="alias of --run-seamgrim",
    )
    parser.add_argument(
        "--run-seamgrim-ci-gate",
        action="store_true",
        help="alias of --run-seamgrim",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    seamgrim_report = Path(args.seamgrim_report)
    ui_report = Path(args.ui_age3_report)
    report_out = Path(args.report_out)

    run_gate = bool(args.run_seamgrim or args.run_age3 or args.run_seamgrim_ci_gate)
    if run_gate:
        rc = run_seamgrim_gate(root, seamgrim_report, ui_report)
        if rc != 0:
            print("[age3-close] seamgrim gate returned non-zero", file=sys.stderr)

    seamgrim_doc = load_json(seamgrim_report)
    ui_doc = load_json(ui_report)
    report = build_report(seamgrim_doc, ui_doc, seamgrim_report, ui_report)
    report_out.parent.mkdir(parents=True, exist_ok=True)
    report_out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    criteria = report.get("criteria", [])
    failed_count = sum(1 for row in criteria if isinstance(row, dict) and not bool(row.get("ok", False)))
    print(
        f"[age3-close] overall_ok={int(bool(report.get('overall_ok', False)))} "
        f"criteria={len(criteria)} failed={failed_count} report={report_out}"
    )
    for row in criteria:
        if not isinstance(row, dict):
            continue
        print(f" - {row.get('name')}: ok={int(bool(row.get('ok', False)))}")
    if not bool(report.get("overall_ok", False)):
        digest = report.get("failure_digest")
        if isinstance(digest, list):
            for line in digest[:8]:
                print(f"   {line}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
