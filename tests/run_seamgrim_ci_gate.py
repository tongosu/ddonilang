#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

from _seamgrim_ci_diag_lib import build_failure_digest, extract_diagnostics


def safe_print(text: str) -> None:
    data = str(text)
    try:
        print(data)
        return
    except UnicodeEncodeError:
        pass
    encoded = data.encode(sys.stdout.encoding or "utf-8", errors="replace")
    print(encoded.decode(sys.stdout.encoding or "utf-8", errors="replace"))


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
        "--with-runtime-5min",
        action="store_true",
        help="include 5-minute runtime scenario check",
    )
    parser.add_argument(
        "--runtime-5min-base-url",
        default="http://127.0.0.1:18787",
        help="base url for runtime_5min step (default: http://127.0.0.1:18787)",
    )
    parser.add_argument(
        "--runtime-5min-json-out",
        default="",
        help="optional runtime_5min json report path",
    )
    parser.add_argument(
        "--runtime-5min-skip-seed-cli",
        action="store_true",
        help="runtime_5min: skip seed teul-cli runs",
    )
    parser.add_argument(
        "--with-5min-checklist",
        action="store_true",
        help="include human-readable 5-minute checklist wrapper check",
    )
    parser.add_argument(
        "--checklist-base-url",
        default="",
        help="base url for 5min checklist (defaults to --runtime-5min-base-url)",
    )
    parser.add_argument(
        "--checklist-json-out",
        default="",
        help="optional 5min checklist json output path",
    )
    parser.add_argument(
        "--checklist-markdown-out",
        default="",
        help="optional 5min checklist markdown output path",
    )
    parser.add_argument(
        "--checklist-from-runtime-report",
        default="",
        help="optional existing runtime_5min report path for checklist rendering",
    )
    parser.add_argument(
        "--checklist-skip-seed-cli",
        action="store_true",
        help="5min checklist: skip seed teul-cli runs when checklist executes runtime internally",
    )
    parser.add_argument(
        "--checklist-skip-ui-common",
        action="store_true",
        help="5min checklist: skip ui common runner when checklist executes runtime internally",
    )
    parser.add_argument(
        "--browse-selection-json-out",
        default="",
        help="optional browse_selection_flow json report path",
    )
    parser.add_argument(
        "--runtime-5min-browse-selection-json-out",
        default="",
        help="optional runtime_5min browse_selection_flow json report path",
    )
    parser.add_argument(
        "--browse-selection-strict",
        action="store_true",
        help="fail when browse_selection_flow report is missing/invalid",
    )
    parser.add_argument(
        "--ui-age3-json-out",
        help="optional path to write ui age3 gate report json",
    )
    parser.add_argument(
        "--sim-core-json-out",
        help="optional path to write sim core contract gate report json",
    )
    parser.add_argument(
        "--phase3-cleanup-json-out",
        help="optional path to write phase3 cleanup gate report json",
    )
    parser.add_argument(
        "--rewrite-overlay-json-out",
        default="",
        help="optional path to write rewrite overlay quality report json",
    )
    parser.add_argument(
        "--print-drilldown",
        action="store_true",
        help="print parsed diagnostics for failed steps",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    browse_selection_json_out = str(args.browse_selection_json_out or "").strip()
    runtime_browse_selection_json_out = str(args.runtime_5min_browse_selection_json_out or "").strip()
    runtime_5min_json_out = str(args.runtime_5min_json_out or "").strip()
    checklist_json_out = str(args.checklist_json_out or "").strip()
    checklist_markdown_out = str(args.checklist_markdown_out or "").strip()
    checklist_from_runtime_report = str(args.checklist_from_runtime_report or "").strip()
    checklist_base_url = str(args.checklist_base_url or args.runtime_5min_base_url or "").strip()
    if not checklist_base_url:
        checklist_base_url = "http://127.0.0.1:18787"

    if args.browse_selection_strict and not browse_selection_json_out:
        default_report = root / "build" / "reports" / "seamgrim_browse_selection_flow_report.detjson"
        browse_selection_json_out = str(default_report)
        print(f"[browse-selection-strict] default report path applied: {browse_selection_json_out}")
    if args.with_5min_checklist and args.with_runtime_5min and not runtime_5min_json_out:
        default_runtime_report = root / "build" / "reports" / "seamgrim_runtime_5min_report.detjson"
        runtime_5min_json_out = str(default_runtime_report)
        print(f"[5min-checklist] runtime report path applied: {runtime_5min_json_out}")

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
    sim_core_cmd = [py, "tests/run_seamgrim_sim_core_contract_gate.py"]
    if args.sim_core_json_out:
        sim_core_cmd.extend(["--json-out", str(args.sim_core_json_out)])
    steps.append(
        run_step(
            root,
            "sim_core_contract_gate",
            sim_core_cmd,
        )
    )
    steps.append(
        run_step(
            root,
            "shape_fallback_mode",
            [py, "tests/run_seamgrim_shape_fallback_mode_check.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "space2d_primitive_source",
            [py, "tests/run_seamgrim_space2d_primitive_source_check.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "space2d_source_ui_gate",
            [py, "tests/run_seamgrim_space2d_source_ui_gate.py"],
        )
    )
    phase3_cleanup_cmd = [py, "tests/run_seamgrim_phase3_cleanup_gate.py"]
    if args.phase3_cleanup_json_out:
        phase3_cleanup_cmd.extend(["--json-out", str(args.phase3_cleanup_json_out)])
    steps.append(
        run_step(
            root,
            "phase3_cleanup_gate",
            phase3_cleanup_cmd,
        )
    )
    steps.append(
        run_step(
            root,
            "lesson_path_fallback",
            [py, "tests/run_seamgrim_lesson_path_fallback_check.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "new_grammar_no_legacy_control_meta",
            [py, "tests/run_seamgrim_new_grammar_no_legacy_control_meta_check.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "visual_contract",
            [py, "tests/run_seamgrim_visual_contract_check.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "seed_overlay_quality",
            [py, "tests/run_seamgrim_seed_overlay_quality_check.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "rewrite_overlay_quality",
            (
                [py, "tests/run_seamgrim_rewrite_overlay_quality_check.py"]
                + (
                    ["--json-out", str(args.rewrite_overlay_json_out)]
                    if str(args.rewrite_overlay_json_out or "").strip()
                    else []
                )
            ),
        )
    )
    steps.append(
        run_step(
            root,
            "pendulum_surface_contract",
            [py, "tests/run_seamgrim_pendulum_surface_contract_check.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "control_exposure_policy",
            [py, "tests/run_seamgrim_control_exposure_policy_check.py"],
        )
    )
    browse_selection_cmd = [py, "tests/run_seamgrim_browse_selection_flow_check.py"]
    if browse_selection_json_out:
        browse_selection_cmd.extend(["--json-out", browse_selection_json_out])
    steps.append(
        run_step(
            root,
            "browse_selection_flow",
            browse_selection_cmd,
        )
    )
    if args.browse_selection_strict:
        steps.append(
            run_step(
                root,
                "browse_selection_report",
                [
                    py,
                    "tests/run_seamgrim_browse_selection_report_check.py",
                    "--report",
                    browse_selection_json_out,
                ],
            )
        )
    if args.with_runtime_5min:
        runtime_5min_cmd = [
            py,
            "tests/run_seamgrim_runtime_5min_check.py",
            "--base-url",
            str(args.runtime_5min_base_url),
        ]
        if runtime_5min_json_out:
            runtime_5min_cmd.extend(["--json-out", runtime_5min_json_out])
        if not runtime_browse_selection_json_out:
            runtime_browse_selection_json_out = browse_selection_json_out
        if runtime_browse_selection_json_out:
            runtime_5min_cmd.extend(["--browse-selection-json-out", runtime_browse_selection_json_out])
        if args.browse_selection_strict:
            runtime_5min_cmd.append("--browse-selection-strict")
        if args.runtime_5min_skip_seed_cli:
            runtime_5min_cmd.append("--skip-seed-cli")
        steps.append(run_step(root, "runtime_5min", runtime_5min_cmd))
    if args.with_5min_checklist:
        checklist_cmd = [
            py,
            "tests/run_seamgrim_5min_checklist.py",
            "--base-url",
            checklist_base_url,
        ]
        checklist_runtime_report = checklist_from_runtime_report
        if not checklist_runtime_report and args.with_runtime_5min and runtime_5min_json_out:
            checklist_runtime_report = runtime_5min_json_out
        if checklist_runtime_report:
            checklist_cmd.extend(["--from-runtime-report", checklist_runtime_report])
        if checklist_json_out:
            checklist_cmd.extend(["--json-out", checklist_json_out])
        if checklist_markdown_out:
            checklist_cmd.extend(["--markdown-out", checklist_markdown_out])
        if args.checklist_skip_seed_cli:
            checklist_cmd.append("--skip-seed-cli")
        if args.checklist_skip_ui_common:
            checklist_cmd.append("--skip-ui-common")
        steps.append(run_step(root, "runtime_5min_checklist", checklist_cmd))
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
            "seed_pendulum_export",
            [py, "tests/run_seamgrim_seed_pendulum_export_check.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "pendulum_runtime_visual",
            [py, "tests/run_seamgrim_pendulum_runtime_visual_check.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "seed_runtime_visual_pack",
            [py, "tests/run_seamgrim_seed_runtime_visual_pack_check.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "runtime_fallback_metrics",
            [py, "tests/run_seamgrim_runtime_fallback_metrics_check.py"],
        )
    )
    steps.append(
        run_step(
            root,
            "runtime_fallback_policy",
            [py, "tests/run_seamgrim_runtime_fallback_policy_check.py"],
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
    steps.append(
        run_step(
            root,
            "pendulum_bogae_shape",
            [py, "tests/run_seamgrim_pendulum_bogae_shape_check.py"],
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
        "browse_selection_strict": bool(args.browse_selection_strict),
        "browse_selection_report_path": browse_selection_json_out,
        "runtime_5min_report_path": runtime_5min_json_out,
        "runtime_5min_browse_selection_report_path": runtime_browse_selection_json_out,
        "with_5min_checklist": bool(args.with_5min_checklist),
        "checklist_base_url": checklist_base_url,
        "checklist_from_runtime_report": checklist_from_runtime_report,
        "checklist_json_out": checklist_json_out,
        "checklist_markdown_out": checklist_markdown_out,
        "ui_age3_report_path": str(args.ui_age3_json_out) if args.ui_age3_json_out else "",
        "sim_core_report_path": str(args.sim_core_json_out) if args.sim_core_json_out else "",
        "phase3_cleanup_report_path": str(args.phase3_cleanup_json_out) if args.phase3_cleanup_json_out else "",
        "rewrite_overlay_report_path": str(args.rewrite_overlay_json_out) if args.rewrite_overlay_json_out else "",
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

