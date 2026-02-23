#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path


def safe_print(text: str) -> None:
    data = str(text)
    try:
        print(data)
        return
    except UnicodeEncodeError:
        pass
    encoded = data.encode(sys.stdout.encoding or "utf-8", errors="replace")
    print(encoded.decode(sys.stdout.encoding or "utf-8", errors="replace"))


def extract_diagnostics(name: str, stdout: str, stderr: str, ok: bool) -> list[dict[str, str]]:
    lines = [line.strip() for line in (stdout + "\n" + stderr).splitlines() if line.strip()]
    out: list[dict[str, str]] = []

    if name == "full_check":
        graph_export = re.compile(r"^graph export failed for (.+?):\s*(.+)$")
        graph_mismatch = re.compile(r"^graph json mismatch:\s*(.+)$")
        for line in lines:
            m1 = graph_export.match(line)
            if m1:
                out.append(
                    {
                        "kind": "graph_export_failed",
                        "target": m1.group(1).strip(),
                        "detail": m1.group(2).strip(),
                    }
                )
                continue
            m2 = graph_mismatch.match(line)
            if m2:
                out.append(
                    {
                        "kind": "graph_json_mismatch",
                        "target": m2.group(1).strip(),
                        "detail": line,
                    }
                )
                continue
    elif name == "schema_gate":
        for line in lines:
            if line.startswith("missing status file:"):
                out.append({"kind": "missing_status_file", "target": "schema_status", "detail": line})
            elif "schema_status.json drift detected" in line:
                out.append({"kind": "schema_status_drift", "target": "schema_status", "detail": line})
            elif "promote report has pending source updates" in line:
                out.append({"kind": "promote_pending", "target": "lessons", "detail": line})
            elif "promote report has missing preview" in line:
                out.append({"kind": "missing_preview", "target": "lessons", "detail": line})
            elif line.startswith("non-age3 profiles found"):
                out.append({"kind": "non_age3_profile", "target": "lessons", "detail": line})
            elif line.startswith("committed schema has non-age3 profiles"):
                out.append({"kind": "non_age3_profile", "target": "schema_status", "detail": line})
            elif line.startswith("committed schema has lesson without preview"):
                out.append({"kind": "missing_preview", "target": "schema_status", "detail": line})
    elif name == "ui_age3_gate":
        for line in lines:
            if line.startswith("missing ui html:"):
                out.append({"kind": "ui_html_missing", "target": "ui/index.html", "detail": line})
            elif line.startswith("missing ui js:"):
                out.append({"kind": "ui_js_missing", "target": "ui/app.js", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "age3_feature_missing", "target": "age3_ui", "detail": line})
            elif line.startswith("age3 ui gate failed:"):
                out.append({"kind": "age3_gate_failed", "target": "age3_ui", "detail": line})
    elif name == "space2d_source_ui_gate":
        for line in lines:
            if line.startswith("missing ui file:"):
                out.append({"kind": "space2d_ui_file_missing", "target": "playground_or_smoke_ui", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "space2d_feature_missing", "target": "space2d_source_ui", "detail": line})
            elif line.startswith("space2d source ui gate failed:"):
                out.append({"kind": "space2d_gate_failed", "target": "space2d_source_ui", "detail": line})
    elif name == "lesson_path_fallback":
        for line in lines:
            if line.startswith("missing ui js:"):
                out.append({"kind": "ui_js_missing", "target": "ui/app.js", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "lesson_path_fallback_missing", "target": "lesson_path_fallback", "detail": line})
            elif line.startswith("seamgrim lesson path fallback check failed:"):
                out.append({"kind": "lesson_path_fallback_failed", "target": "lesson_path_fallback", "detail": line})
    elif name == "overlay_compare_pack":
        for line in lines:
            if line.startswith("missing pack root:"):
                out.append({"kind": "overlay_pack_root_missing", "target": "overlay_compare_pack", "detail": line})
            elif line.startswith("missing pack case:"):
                out.append({"kind": "overlay_pack_case_missing", "target": "overlay_compare_pack", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "overlay_compare_case_failed", "target": "overlay_compare_pack", "detail": line})
            elif line.startswith("[FAIL] pack="):
                out.append({"kind": "overlay_compare_case_failed", "target": "overlay_compare_pack", "detail": line})
            elif line.startswith("overlay compare pack failed:"):
                out.append({"kind": "overlay_compare_pack_failed", "target": "overlay_compare_pack", "detail": line})
    elif name == "overlay_session_pack":
        for line in lines:
            if line.startswith("missing pack root:"):
                out.append({"kind": "overlay_session_pack_root_missing", "target": "overlay_session_pack", "detail": line})
            elif line.startswith("missing pack case:"):
                out.append({"kind": "overlay_session_pack_case_missing", "target": "overlay_session_pack", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "overlay_session_case_failed", "target": "overlay_session_pack", "detail": line})
            elif line.startswith("[FAIL] pack="):
                out.append({"kind": "overlay_session_case_failed", "target": "overlay_session_pack", "detail": line})
            elif line.startswith("overlay session pack failed:"):
                out.append({"kind": "overlay_session_pack_failed", "target": "overlay_session_pack", "detail": line})
    elif name == "overlay_session_contract":
        for line in lines:
            if line.startswith("overlay session contract failed"):
                out.append({"kind": "overlay_session_contract_failed", "target": "overlay_session_contract", "detail": line})
            elif line.startswith("[overlay-session-contract]"):
                out.append({"kind": "overlay_session_contract_log", "target": "overlay_session_contract", "detail": line})
    elif name == "age5_close":
        for line in lines:
            if line.startswith("[age5-close] overall_ok=0"):
                out.append({"kind": "age5_close_failed", "target": "age5_close", "detail": line})
            elif line.startswith(" - ") and "ok=0" in line:
                out.append({"kind": "age5_criteria_failed", "target": "age5_close", "detail": line})
    elif name == "export_graph_preprocess":
        for line in lines:
            if line.startswith("seamgrim export_graph preprocess check failed:"):
                out.append({"kind": "preprocess_check_failed", "target": "export_graph", "detail": line})
    elif name == "deploy_artifacts":
        for line in lines:
            if line.startswith("missing deploy file:"):
                out.append({"kind": "deploy_file_missing", "target": "deploy_artifacts", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "deploy_artifact_mismatch", "target": "deploy_artifacts", "detail": line})
            elif line.startswith("seamgrim deploy artifacts check failed:"):
                out.append({"kind": "deploy_check_failed", "target": "deploy_artifacts", "detail": line})
    elif name == "ddn_exec_server_check":
        for line in lines:
            if line.startswith("check="):
                out.append({"kind": "ddn_exec_server_check_failed", "target": "ddn_exec_server_check", "detail": line})
            elif line.startswith("ddn exec server failed to start"):
                out.append({"kind": "ddn_exec_server_start_failed", "target": "ddn_exec_server_check", "detail": line})
    elif name == "workflow_contract":
        for line in lines:
            if line.startswith("missing workflow file:"):
                out.append({"kind": "workflow_file_missing", "target": "workflow_contract", "detail": line})
            elif line.startswith("missing branch protection file:"):
                out.append(
                    {"kind": "branch_protection_file_missing", "target": "workflow_contract", "detail": line}
                )
            elif line.startswith("check="):
                out.append({"kind": "workflow_contract_mismatch", "target": "workflow_contract", "detail": line})
            elif line.startswith("seamgrim workflow contract check failed:"):
                out.append({"kind": "workflow_contract_failed", "target": "workflow_contract", "detail": line})
    elif name == "formula_compat":
        for line in lines:
            if line.startswith("missing target root:"):
                out.append({"kind": "formula_scope_root_missing", "target": "seamgrim_formula", "detail": line})
            elif line.startswith("no lesson files found under target:"):
                out.append({"kind": "formula_scope_files_missing", "target": "seamgrim_formula", "detail": line})
            elif line.startswith("check="):
                out.append({"kind": "formula_incompat", "target": "seamgrim_formula", "detail": line})
            elif line.startswith("seamgrim formula compat check failed:"):
                out.append({"kind": "formula_compat_failed", "target": "seamgrim_formula", "detail": line})
    elif name == "schema_realign_formula_compat":
        for line in lines:
            if line.startswith("schema realign compat check failed:"):
                out.append(
                    {"kind": "schema_realign_formula_compat_failed", "target": "lesson_schema_realign", "detail": line}
                )
    elif name == "schema_upgrade_formula_compat":
        for line in lines:
            if line.startswith("schema upgrade formula compat check failed:"):
                out.append(
                    {"kind": "schema_upgrade_formula_compat_failed", "target": "lesson_schema_upgrade", "detail": line}
                )

    if not out and not ok:
        for line in lines[:5]:
            out.append({"kind": "generic_error", "target": name, "detail": line})
    return out


def build_failure_digest(steps: list[dict[str, object]], limit: int = 8) -> list[str]:
    out: list[str] = []
    for step in steps:
        if bool(step.get("ok", False)):
            continue
        name = str(step.get("name", "-"))
        diagnostics = step.get("diagnostics")
        if isinstance(diagnostics, list) and diagnostics:
            first = diagnostics[0] if isinstance(diagnostics[0], dict) else {}
            kind = str(first.get("kind", "generic_error"))
            target = str(first.get("target", "-"))
            detail = str(first.get("detail", "")).strip()
            detail = " ".join(detail.split())
            if len(detail) > 120:
                detail = detail[:120] + "..."
            row = f"step={name} kind={kind} target={target}"
            if detail:
                row += f" detail={detail}"
            out.append(row)
        else:
            stderr = str(step.get("stderr") or "").strip()
            stdout = str(step.get("stdout") or "").strip()
            detail = stderr or stdout
            detail = " ".join(detail.split())
            if len(detail) > 120:
                detail = detail[:120] + "..."
            row = f"step={name}"
            if detail:
                row += f" detail={detail}"
            out.append(row)
        if len(out) >= limit:
            break
    return out


def run_step(root: Path, name: str, cmd: list[str]) -> dict[str, object]:
    started = time.perf_counter()
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    elapsed_ms = int((time.perf_counter() - started) * 1000)
    stdout = (proc.stdout or "").strip()
    stderr = (proc.stderr or "").strip()
    ok = proc.returncode == 0

    print(f"[{name}] {'ok' if ok else 'fail'} ({elapsed_ms}ms)")
    if stdout:
        safe_print(stdout)
    if stderr:
        safe_print(stderr)
    diagnostics = extract_diagnostics(name, stdout, stderr, ok)

    return {
        "name": name,
        "ok": ok,
        "returncode": proc.returncode,
        "elapsed_ms": elapsed_ms,
        "cmd": cmd,
        "stdout": stdout,
        "stderr": stderr,
        "diagnostics": diagnostics,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Run seamgrim CI gate checks as one entrypoint")
    parser.add_argument(
        "--require-promoted",
        action="store_true",
        help="require source==preview promotion on schema gate",
    )
    parser.add_argument(
        "--strict-graph",
        action="store_true",
        help="force full check graph export failures to fail",
    )
    parser.add_argument("--json-out", help="write gate result json")
    parser.add_argument(
        "--with-overlay-checks",
        action="store_true",
        help="include removed overlay/session legacy checks",
    )
    parser.add_argument(
        "--with-age5-close",
        action="store_true",
        help="include age5 close check",
    )
    parser.add_argument(
        "--ui-age3-json-out",
        help="optional path to write ui age3 gate report json",
    )
    parser.add_argument(
        "--print-drilldown",
        action="store_true",
        help="print parsed diagnostics for failed steps",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    steps: list[dict[str, object]] = []
    steps.append(
        run_step(
            root,
            "ci_gate_diagnostics",
            [py, "tests/run_seamgrim_ci_gate_diagnostics_check.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "workflow_contract",
            [py, "tests/run_seamgrim_workflow_contract_check.py"],
        )
    )

    schema_cmd = [py, "tests/run_seamgrim_lesson_schema_gate.py"]
    if args.require_promoted:
        schema_cmd.append("--require-promoted")
    steps.append(run_step(root, "schema_gate", schema_cmd))
    steps.append(
        run_step(
            root,
            "schema_realign_formula_compat",
            [py, "tests/run_seamgrim_lesson_schema_realign_formula_compat_check.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "schema_upgrade_formula_compat",
            [py, "tests/run_seamgrim_lesson_schema_upgrade_formula_compat_check.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "formula_compat",
            [py, "tests/run_seamgrim_formula_compat_check.py"],
        )
    )
    ui_age3_cmd = [py, "tests/run_seamgrim_ui_age3_gate.py"]
    if args.ui_age3_json_out:
        ui_age3_cmd.extend(["--json-out", args.ui_age3_json_out])
    steps.append(
        run_step(
            root,
            "ui_age3_gate",
            ui_age3_cmd,
        )
    )
    steps.append(
        run_step(
            root,
            "space2d_source_ui_gate",
            [py, "tests/run_seamgrim_space2d_source_ui_gate.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "lesson_path_fallback",
            [py, "tests/run_seamgrim_lesson_path_fallback_check.py"],
        )
    )
    if args.with_overlay_checks:
        steps.append(
            run_step(
                root,
                "overlay_compare_pack",
                [py, "tests/run_seamgrim_overlay_compare_pack.py"],
            )
        )
        steps.append(
            run_step(
                root,
                "overlay_session_pack",
                [py, "tests/run_seamgrim_overlay_session_pack.py"],
            )
        )
        steps.append(
            run_step(
                root,
                "overlay_session_contract",
                [py, "tests/run_seamgrim_overlay_session_contract.py"],
            )
        )
    if args.with_age5_close:
        steps.append(
            run_step(
                root,
                "age5_close",
                [py, "tests/run_age5_close.py"],
            )
        )

    steps.append(
        run_step(
            root,
            "export_graph_preprocess",
            [py, "tests/run_seamgrim_export_graph_preprocess_check.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "deploy_artifacts",
            [py, "tests/run_seamgrim_deploy_artifacts_check.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "ddn_exec_server_check",
            [
                py,
                "solutions/seamgrim_ui_mvp/tools/ddn_exec_server_check.py",
                "--base-url",
                "http://127.0.0.1:18787",
                "--timeout-sec",
                "15",
            ],
        )
    )

    full_cmd = [py, "tests/run_seamgrim_full_check.py", "--skip-schema-gate"]
    if args.strict_graph:
        full_cmd.append("--strict-graph")
    steps.append(run_step(root, "full_check", full_cmd))

    failed = [step for step in steps if not bool(step.get("ok"))]
    elapsed_total_ms = sum(int(step.get("elapsed_ms", 0)) for step in steps)
    result = {
        "schema": "seamgrim.ci_gate.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": len(failed) == 0,
        "strict_graph": bool(args.strict_graph),
        "require_promoted": bool(args.require_promoted),
        "ui_age3_report_path": str(args.ui_age3_json_out) if args.ui_age3_json_out else "",
        "elapsed_total_ms": elapsed_total_ms,
        "failure_digest": build_failure_digest(steps),
        "steps": steps,
    }
    if args.json_out:
        out = Path(args.json_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"[report] {out}")

    if failed:
        if args.print_drilldown:
            print("[drilldown]")
            for step in failed:
                name = str(step.get("name", "-"))
                diagnostics = step.get("diagnostics") or []
                if not isinstance(diagnostics, list) or not diagnostics:
                    print(f" - {name}: no parsed diagnostics")
                    continue
                for row in diagnostics:
                    if not isinstance(row, dict):
                        continue
                    kind = str(row.get("kind", "generic_error"))
                    target = str(row.get("target", "-"))
                    detail = str(row.get("detail", "")).strip()
                    print(f" - {name}::{kind} target={target}")
                    if detail:
                        print(f"   {detail}")
        names = ", ".join(str(step.get("name")) for step in failed)
        print(f"seamgrim ci gate failed: {names}")
        return 1

    print("seamgrim ci gate ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
