#!/usr/bin/env python
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import urlparse

from _ci_seamgrim_step_contract import SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME
from _selftest_exec_cache import mark_script_ok
from _seamgrim_ci_diag_lib import build_failure_digest, extract_diagnostics
from _seamgrim_parity_server_lib import start_parity_server, stop_parity_server


def safe_print(text: str) -> None:
    data = str(text)
    try:
        print(data)
        return
    except UnicodeEncodeError:
        pass
    encoded = data.encode(sys.stdout.encoding or "utf-8", errors="replace")
    print(encoded.decode(sys.stdout.encoding or "utf-8", errors="replace"))


def _run_step_once(
    root: Path,
    name: str,
    cmd: list[str],
    *,
    env_extra: dict[str, str] | None = None,
) -> dict[str, object]:
    started = time.perf_counter()
    env = None
    if env_extra:
        env = os.environ.copy()
        env.update({str(k): str(v) for k, v in env_extra.items()})
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
    )
    elapsed_ms = int((time.perf_counter() - started) * 1000)
    stdout = (proc.stdout or "").strip()
    stderr = (proc.stderr or "").strip()
    ok = proc.returncode == 0

    diagnostics = extract_diagnostics(name, stdout, stderr, ok)
    if ok:
        _maybe_mark_cmd_script_ok(root, cmd)

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


def _maybe_mark_cmd_script_ok(root: Path, cmd: list[str]) -> None:
    if len(cmd) < 2:
        return
    candidate = str(cmd[1]).strip()
    if not candidate:
        return
    lowered = candidate.lower()
    if not lowered.endswith(".py"):
        return
    script_path = (root / candidate).resolve()
    try:
        if script_path.exists():
            mark_script_ok(str(script_path.relative_to(root).as_posix()))
    except Exception:
        return


def _print_step_result(step: dict[str, object]) -> None:
    name = str(step.get("name", "-"))
    ok = bool(step.get("ok"))
    elapsed_ms = int(step.get("elapsed_ms", 0))
    stdout = str(step.get("stdout", "") or "").strip()
    stderr = str(step.get("stderr", "") or "").strip()
    print(f"[{name}] {'ok' if ok else 'fail'} ({elapsed_ms}ms)")
    if stdout:
        safe_print(stdout)
    if stderr:
        safe_print(stderr)


def run_step(
    root: Path,
    name: str,
    cmd: list[str],
    *,
    env_extra: dict[str, str] | None = None,
) -> dict[str, object]:
    step = _run_step_once(root, name, cmd, env_extra=env_extra)
    _print_step_result(step)
    return step


def run_steps_parallel(
    root: Path,
    step_defs: list[dict[str, object]],
    *,
    max_workers_override: int = 0,
) -> list[dict[str, object]]:
    if not step_defs:
        return []
    ordered: list[dict[str, object] | None] = [None] * len(step_defs)
    env_workers_text = str(os.environ.get("DDN_SEAMGRIM_CI_GATE_MAX_WORKERS", "")).strip()
    env_workers = 0
    if env_workers_text:
        try:
            env_workers = int(env_workers_text)
        except ValueError:
            env_workers = 0
    default_workers = 14
    requested_workers = (
        int(max_workers_override)
        if int(max_workers_override) > 0
        else (env_workers if env_workers > 0 else default_workers)
    )
    max_workers = max(1, min(requested_workers, len(step_defs)))
    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        pending = {}
        for idx, step_def in enumerate(step_defs):
            name = str(step_def.get("name", "")).strip()
            cmd = step_def.get("cmd")
            env_extra = step_def.get("env_extra")
            if not name or not isinstance(cmd, list):
                raise ValueError(f"invalid step def at index={idx}: {step_def!r}")
            pending[executor.submit(_run_step_once, root, name, cmd, env_extra=env_extra)] = idx
        for future in as_completed(pending):
            idx = pending[future]
            ordered[idx] = future.result()
    results: list[dict[str, object]] = []
    for step in ordered:
        if step is None:
            continue
        _print_step_result(step)
        results.append(step)
    return results


def _read_positive_int_env(name: str, default: int) -> int:
    text = str(os.environ.get(name, "")).strip()
    if not text:
        return max(1, int(default))
    try:
        value = int(text)
    except ValueError:
        return max(1, int(default))
    return max(1, value)


def _start_local_ddn_exec_server_prewarm(
    root: Path,
    base_url: str,
    *,
    profile: str = "release",
    timeout_sec: float = 5.0,
) -> subprocess.Popen[bytes] | None:
    base = str(base_url or "").strip()
    if not base:
        return None

    parsed = urlparse(base)
    host = str(parsed.hostname or "").strip().lower()
    if host not in {"127.0.0.1", "localhost"}:
        return None
    port = int(parsed.port or (443 if str(parsed.scheme).lower() == "https" else 80))
    previous_catalog_mode = os.environ.get("SEAMGRIM_LESSON_CATALOG_MODE")
    previous_allow_legacy = os.environ.get("SEAMGRIM_ALLOW_LEGACY_LESSONS")
    if str(profile).strip().lower() == "legacy":
        os.environ["SEAMGRIM_LESSON_CATALOG_MODE"] = "full"
        os.environ["SEAMGRIM_ALLOW_LEGACY_LESSONS"] = "1"
    else:
        os.environ["SEAMGRIM_LESSON_CATALOG_MODE"] = "reps_only"
        os.environ["SEAMGRIM_ALLOW_LEGACY_LESSONS"] = "0"
    try:
        _server_module, _resolved_base_url, proc = start_parity_server(
            root=root,
            module_name="seamgrim_ddn_exec_server_check_for_ci_gate",
            host=host,
            port=port,
            timeout_sec=max(1.0, float(timeout_sec)),
            require_existing_server=False,
        )
    except RuntimeError as exc:
        print(f"[ddn_exec_server_prewarm] skip base_url={base} detail={str(exc).strip()}")
        return None
    finally:
        if previous_catalog_mode is None:
            os.environ.pop("SEAMGRIM_LESSON_CATALOG_MODE", None)
        else:
            os.environ["SEAMGRIM_LESSON_CATALOG_MODE"] = previous_catalog_mode
        if previous_allow_legacy is None:
            os.environ.pop("SEAMGRIM_ALLOW_LEGACY_LESSONS", None)
        else:
            os.environ["SEAMGRIM_ALLOW_LEGACY_LESSONS"] = previous_allow_legacy
    if proc is not None:
        print(f"[ddn_exec_server_prewarm] ok base_url={base} profile={str(profile).strip().lower() or 'release'}")
    return proc


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
    parser.add_argument(
        "--profile",
        choices=["release", "legacy"],
        default="release",
        help="gate profile (default: release)",
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
        "--runtime-5min-skip-ui-common",
        action="store_true",
        help="runtime_5min: skip ui common/aux ui runners",
    )
    parser.add_argument(
        "--runtime-5min-skip-showcase-check",
        action="store_true",
        help="runtime_5min: skip pendulum+tetris showcase check",
    )
    parser.add_argument(
        "--runtime-5min-showcase-smoke",
        action="store_true",
        help="runtime_5min: run showcase check with non-dry smoke",
    )
    parser.add_argument(
        "--runtime-5min-showcase-smoke-madi-pendulum",
        type=int,
        default=20,
        help="runtime_5min: showcase smoke pendulum madi",
    )
    parser.add_argument(
        "--runtime-5min-showcase-smoke-madi-tetris",
        type=int,
        default=20,
        help="runtime_5min: showcase smoke tetris madi",
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
        "--pack-evidence-report-json-out",
        default="",
        help="optional path to write pack evidence tier runner report json",
    )
    parser.add_argument(
        "--lesson-warning-report-json-out",
        default="",
        help="optional path to write lesson warning token scan report json",
    )
    parser.add_argument(
        "--lesson-warning-require-zero",
        action="store_true",
        help="fail when lesson warning token total is non-zero",
    )
    parser.add_argument(
        "--require-preview-synced",
        action="store_true",
        help="fail when lesson preview/source dry-run reports would_apply > 0",
    )
    parser.add_argument(
        "--print-drilldown",
        action="store_true",
        help="print parsed diagnostics for failed steps",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    profile = str(args.profile or "release").strip().lower()
    release_profile = profile != "legacy"
    transport_contract_env = {"DDN_ASSUME_FAMILY_CONTRACT_PASSED": "1"}
    contract_prereq_env = {"DDN_ASSUME_CONTRACT_PREREQS_PASSED": "1"}
    browse_selection_json_out = str(args.browse_selection_json_out or "").strip()
    runtime_browse_selection_json_out = str(args.runtime_5min_browse_selection_json_out or "").strip()
    runtime_5min_json_out = str(args.runtime_5min_json_out or "").strip()
    pack_evidence_report_json_out = str(args.pack_evidence_report_json_out or "").strip()
    checklist_json_out = str(args.checklist_json_out or "").strip()
    checklist_markdown_out = str(args.checklist_markdown_out or "").strip()
    checklist_from_runtime_report = str(args.checklist_from_runtime_report or "").strip()
    lesson_warning_report_json_out = str(args.lesson_warning_report_json_out or "").strip()
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
    if not pack_evidence_report_json_out:
        default_pack_evidence_report = root / "build" / "reports" / "seamgrim_pack_evidence_tier_runner_check.detjson"
        pack_evidence_report_json_out = str(default_pack_evidence_report)

    legacy_only_parallel_steps = {
        "schema_gate",
        "seed_overlay_quality",
        "seed_meta_files",
        "featured_seed_catalog_sync",
        "featured_seed_catalog_autogen",
        "rewrite_overlay_quality",
        "featured_seed_quick_launch_logic",
    }
    legacy_only_gate_steps = {
        "seed_pendulum_export",
        "pendulum_runtime_visual",
        "seed_runtime_visual_pack",
        "pendulum_bogae_shape",
        "full_check",
    }

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
    lesson_warning_cmd = [py, "tests/run_seamgrim_lesson_warning_tokens_check.py"]
    lesson_migration_lint_cmd = [py, "tests/run_seamgrim_lesson_migration_lint_check.py"]
    lesson_migration_lint_preview_cmd = [py, "tests/run_seamgrim_lesson_migration_lint_preview_check.py"]
    lesson_preview_sync_cmd = [py, "tests/run_seamgrim_lesson_preview_sync_check.py"]
    lesson_migration_autofix_cmd = [py, "tests/run_seamgrim_lesson_migration_autofix_check.py"]
    pack_evidence_tier_cmd = [py, "tests/run_pack_evidence_tier_check.py"]
    pack_evidence_tier_report_check_cmd = [py, "tests/run_pack_evidence_tier_report_check.py"]
    if pack_evidence_report_json_out:
        pack_evidence_tier_cmd.extend(["--report-out", pack_evidence_report_json_out])
        pack_evidence_tier_report_check_cmd.extend(["--report-path", pack_evidence_report_json_out])
    if lesson_warning_report_json_out:
        lesson_warning_cmd.extend(["--report", lesson_warning_report_json_out])
    if args.lesson_warning_require_zero:
        lesson_warning_cmd.append("--require-zero")
    if args.require_preview_synced:
        lesson_preview_sync_cmd.append("--require-synced")
    ui_age3_cmd = [py, "tests/run_seamgrim_ui_age3_gate.py"]
    if args.ui_age3_json_out:
        ui_age3_cmd.extend(["--json-out", args.ui_age3_json_out])
    sim_core_cmd = [py, "tests/run_seamgrim_sim_core_contract_gate.py"]
    if args.sim_core_json_out:
        sim_core_cmd.extend(["--json-out", str(args.sim_core_json_out)])
    phase3_cleanup_cmd = [py, "tests/run_seamgrim_phase3_cleanup_gate.py"]
    if args.phase3_cleanup_json_out:
        phase3_cleanup_cmd.extend(["--json-out", str(args.phase3_cleanup_json_out)])
    browse_selection_cmd = [py, "tests/run_seamgrim_browse_selection_flow_check.py"]
    if browse_selection_json_out:
        browse_selection_cmd.extend(["--json-out", browse_selection_json_out])
    rewrite_overlay_cmd = [py, "tests/run_seamgrim_rewrite_overlay_quality_check.py"]
    if str(args.rewrite_overlay_json_out or "").strip():
        rewrite_overlay_cmd.extend(["--json-out", str(args.rewrite_overlay_json_out)])
    parallel_step_defs: list[dict[str, object]] = [
                {
                    "name": "schema_gate",
                    "cmd": schema_cmd,
                },
                {
                    "name": "lesson_warning_tokens",
                    "cmd": lesson_warning_cmd,
                },
                {
                    "name": "lesson_migration_lint",
                    "cmd": lesson_migration_lint_cmd,
                },
                {
                    "name": "lesson_migration_lint_preview",
                    "cmd": lesson_migration_lint_preview_cmd,
                },
                {
                    "name": "lesson_preview_sync",
                    "cmd": lesson_preview_sync_cmd,
                },
                {
                    "name": "lesson_migration_autofix",
                    "cmd": lesson_migration_autofix_cmd,
                },
                {
                    "name": "pack_evidence_tier",
                    "cmd": pack_evidence_tier_cmd,
                },
                {
                    "name": "stateful_sim_preview_upgrade",
                    "cmd": [py, "tests/run_seamgrim_stateful_sim_preview_upgrade_check.py"],
                },
                {
                    "name": "schema_realign_formula_compat",
                    "cmd": [py, "tests/run_seamgrim_lesson_schema_realign_formula_compat_check.py"],
                },
                {
                    "name": "schema_upgrade_formula_compat",
                    "cmd": [py, "tests/run_seamgrim_lesson_schema_upgrade_formula_compat_check.py"],
                },
                {
                    "name": "formula_compat",
                    "cmd": [py, "tests/run_seamgrim_formula_compat_check.py"],
                },
                {
                    "name": "ui_age3_gate",
                    "cmd": ui_age3_cmd,
                },
                {
                    "name": "sim_core_contract_gate",
                    "cmd": sim_core_cmd,
                },
                {
                    "name": "shape_fallback_mode",
                    "cmd": [py, "tests/run_seamgrim_shape_fallback_mode_check.py"],
                },
                {
                    "name": "runtime_view_source_strict",
                    "cmd": [py, SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME["seamgrim_runtime_view_source_strict_check"]],
                },
                {
                    "name": "view_only_state_hash_invariant",
                    "cmd": [
                        py,
                        SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME["seamgrim_view_only_state_hash_invariant_check"],
                    ],
                },
                {
                    "name": "run_legacy_autofix",
                    "cmd": [py, SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME["seamgrim_run_legacy_autofix_check"]],
                },
                {
                    "name": "space2d_primitive_source",
                    "cmd": [py, "tests/run_seamgrim_space2d_primitive_source_check.py"],
                },
                {
                    "name": "space2d_source_ui_gate",
                    "cmd": [py, "tests/run_seamgrim_space2d_source_ui_gate.py"],
                },
                {
                    "name": "phase3_cleanup_gate",
                    "cmd": phase3_cleanup_cmd,
                },
                {
                    "name": "lesson_path_fallback",
                    "cmd": [py, "tests/run_seamgrim_lesson_path_fallback_check.py"],
                },
                {
                    "name": "new_grammar_no_legacy_control_meta",
                    "cmd": [py, "tests/run_seamgrim_new_grammar_no_legacy_control_meta_check.py"],
                },
                {
                    "name": "visual_contract",
                    "cmd": [py, "tests/run_seamgrim_visual_contract_check.py"],
                },
                {
                    "name": "seed_overlay_quality",
                    "cmd": [py, "tests/run_seamgrim_seed_overlay_quality_check.py"],
                },
                {
                    "name": "seed_meta_files",
                    "cmd": [py, "tests/run_seamgrim_seed_meta_files_check.py"],
                },
                {
                    "name": "featured_seed_catalog_sync",
                    "cmd": [py, "tests/run_seamgrim_featured_seed_catalog_sync_check.py"],
                },
                {
                    "name": "featured_seed_catalog_autogen",
                    "cmd": [py, "tests/run_seamgrim_featured_seed_catalog_autogen_check.py"],
                },
                {
                    "name": "guideblock_keys_pack",
                    "cmd": [py, "tests/run_seamgrim_guideblock_keys_pack_check.py"],
                },
                {
                    "name": "moyang_view_boundary_pack",
                    "cmd": [py, "tests/run_seamgrim_moyang_view_boundary_pack_check.py"],
                },
                {
                    "name": "patent_b_state_view_hash_isolation",
                    "cmd": [py, "tests/run_patent_b_state_view_hash_isolation_check.py"],
                },
                {
                    "name": "dotbogi_view_meta_hash_pack",
                    "cmd": [py, "tests/run_dotbogi_view_meta_hash_pack_check.py"],
                },
                {
                    "name": "rewrite_overlay_quality",
                    "cmd": rewrite_overlay_cmd,
                },
                {
                    "name": "pendulum_surface_contract",
                    "cmd": [py, "tests/run_seamgrim_pendulum_surface_contract_check.py"],
                },
                {
                    "name": "control_exposure_policy",
                    "cmd": [py, "tests/run_seamgrim_control_exposure_policy_check.py"],
                },
                {
                    "name": "featured_seed_quick_launch_logic",
                    "cmd": [py, "tests/run_seamgrim_featured_seed_quick_launch_check.py"],
                },
                {
                    "name": "browse_selection_flow",
                    "cmd": browse_selection_cmd,
                },
                {
                    "name": "block_editor_smoke",
                    "cmd": [py, "tests/run_seamgrim_block_editor_smoke_check.py"],
                },
                {
                    "name": "playground_smoke",
                    "cmd": [py, "tests/run_seamgrim_playground_smoke_check.py"],
                },
            ]
    if release_profile:
        parallel_step_defs = [
            row for row in parallel_step_defs if str(row.get("name", "")).strip() not in legacy_only_parallel_steps
        ]
    steps.extend(run_steps_parallel(root, parallel_step_defs))
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
            "--server-check-profile",
            "legacy" if not release_profile else "release",
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
        if release_profile and not args.runtime_5min_skip_seed_cli:
            runtime_5min_cmd.append("--skip-seed-cli")
        if args.runtime_5min_skip_ui_common:
            runtime_5min_cmd.append("--skip-ui-common")
        if release_profile and not args.runtime_5min_skip_ui_common:
            runtime_5min_cmd.append("--skip-ui-common")
        if args.runtime_5min_skip_showcase_check:
            runtime_5min_cmd.append("--skip-showcase-check")
        if release_profile and not args.runtime_5min_skip_showcase_check:
            runtime_5min_cmd.append("--skip-showcase-check")
        if args.runtime_5min_showcase_smoke:
            runtime_5min_cmd.extend(
                [
                    "--showcase-smoke",
                    "--showcase-smoke-madi-pendulum",
                    str(max(1, int(args.runtime_5min_showcase_smoke_madi_pendulum))),
                    "--showcase-smoke-madi-tetris",
                    str(max(1, int(args.runtime_5min_showcase_smoke_madi_tetris))),
                ]
            )
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
                "overlay_session_wired_consistency",
                [py, "tests/run_seamgrim_overlay_session_wired_consistency_check.py"],
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
                [py, "tests/run_age5_close.py", "--strict"],
            )
        )

    steps.append(
        run_step(
            root,
            "export_graph_preprocess",
            [py, "tests/run_seamgrim_export_graph_preprocess_check.py"],
        )
    )
    ddn_exec_server_base_url = "http://127.0.0.1:18787"
    ddn_exec_server_proc = _start_local_ddn_exec_server_prewarm(
        root,
        ddn_exec_server_base_url,
        profile=profile,
    )
    shared_parity_server_args: list[str] = []
    if ddn_exec_server_proc is not None:
        parsed_shared_parity = urlparse(ddn_exec_server_base_url)
        shared_host = str(parsed_shared_parity.hostname or "127.0.0.1")
        shared_port = int(
            parsed_shared_parity.port or (443 if str(parsed_shared_parity.scheme).lower() == "https" else 80)
        )
        shared_parity_server_args = [
            "--host",
            shared_host,
            "--port",
            str(shared_port),
            "--require-existing-server",
        ]
    steps.append(
        run_step(
            root,
            "deploy_artifacts",
            [py, "tests/run_seamgrim_deploy_artifacts_check.py"],
        )
    )
    full_cmd = [py, "tests/run_seamgrim_full_gate_check.py"]
    if args.strict_graph:
        full_cmd.append("--strict-graph")
    gate_step_defs: list[dict[str, object]] = [
                {
                    "name": "seed_pendulum_export",
                    "cmd": [py, "tests/run_seamgrim_seed_pendulum_export_check.py"],
                },
                {
                    "name": "pendulum_runtime_visual",
                    "cmd": [py, "tests/run_seamgrim_pendulum_runtime_visual_check.py"],
                },
                {
                    "name": "seed_runtime_visual_pack",
                    "cmd": [py, "tests/run_seamgrim_seed_runtime_visual_pack_check.py"],
                },
                {
                    "name": "wasm_viewmeta_statehash_prereq",
                    "cmd": [
                        py,
                        "tests/run_seamgrim_wasm_smoke.py",
                        "seamgrim_wasm_viewmeta_statehash_v1",
                        "--skip-ui-common",
                        "--skip-ui-pendulum",
                        "--skip-wrapper",
                        "--skip-vm-runtime",
                        "--skip-space2d-source-gate",
                        "--skip-lesson-canon",
                    ],
                },
                {
                    "name": "state_hash_view_boundary_prereq",
                    "cmd": [py, "tests/run_pack_golden.py", "seamgrim_state_hash_view_boundary_smoke_v1"],
                },
                {
                    "name": "wasm_bridge_contract_prereq",
                    "cmd": [
                        py,
                        "tests/run_seamgrim_wasm_smoke.py",
                        "seamgrim_wasm_bridge_contract_v1",
                        "--skip-ui-common",
                        "--skip-ui-pendulum",
                        "--skip-wrapper",
                        "--skip-vm-runtime",
                        "--skip-space2d-source-gate",
                        "--skip-lesson-canon",
                    ],
                },
                {
                    "name": "wasm_web_smoke_contract",
                    "cmd": [py, "tests/run_seamgrim_wasm_web_smoke_contract_pack_check.py"],
                },
                {
                    "name": "seamgrim_ci_gate_wasm_web_smoke_step_check",
                    "cmd": [py, "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py"],
                },
                {
                    "name": "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
                    "cmd": [py, "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest.py"],
                },
                {
                    "name": "seamgrim_ci_gate_lesson_migration_lint_step_check",
                    "cmd": [py, "tests/run_seamgrim_ci_gate_lesson_migration_lint_step_check.py"],
                },
                {
                    "name": "seamgrim_ci_gate_lesson_migration_lint_step_check_selftest",
                    "cmd": [py, "tests/run_seamgrim_ci_gate_lesson_migration_lint_step_check_selftest.py"],
                },
                {
                    "name": "seamgrim_ci_gate_lesson_migration_autofix_step_check",
                    "cmd": [py, "tests/run_seamgrim_ci_gate_lesson_migration_autofix_step_check.py"],
                },
                {
                    "name": "seamgrim_ci_gate_lesson_migration_autofix_step_check_selftest",
                    "cmd": [py, "tests/run_seamgrim_ci_gate_lesson_migration_autofix_step_check_selftest.py"],
                },
                {
                    "name": "seamgrim_ci_gate_lesson_preview_sync_step_check",
                    "cmd": [py, "tests/run_seamgrim_ci_gate_lesson_preview_sync_step_check.py"],
                },
                {
                    "name": "seamgrim_ci_gate_lesson_preview_sync_step_check_selftest",
                    "cmd": [py, "tests/run_seamgrim_ci_gate_lesson_preview_sync_step_check_selftest.py"],
                },
                {
                    "name": "seamgrim_ci_gate_pack_evidence_tier_step_check",
                    "cmd": [py, "tests/run_seamgrim_ci_gate_pack_evidence_tier_step_check.py"],
                },
                {
                    "name": "seamgrim_ci_gate_pack_evidence_tier_step_check_selftest",
                    "cmd": [py, "tests/run_seamgrim_ci_gate_pack_evidence_tier_step_check_selftest.py"],
                },
                {
                    "name": "pack_evidence_tier_report_check",
                    "cmd": pack_evidence_tier_report_check_cmd,
                },
                {
                    "name": "pack_evidence_tier_report_check_selftest",
                    "cmd": [
                        py,
                        "tests/run_pack_evidence_tier_report_check_selftest.py",
                        "--verify-report",
                        pack_evidence_report_json_out,
                    ],
                },
                {
                    "name": "pack_evidence_tier_selftest",
                    "cmd": [py, "tests/run_pack_evidence_tier_check_selftest.py"],
                },
                {
                    "name": "graph_bridge_contract",
                    "cmd": [py, "tests/run_seamgrim_graph_golden.py"],
                },
                {
                    "name": "bridge_hash_cross_check",
                    "cmd": [py, "tests/run_seamgrim_bridge_check_selftest.py"],
                },
                {
                    "name": "graph_api_parity",
                    "cmd": [py, "tests/run_seamgrim_graph_api_parity_check.py", *shared_parity_server_args],
                },
                {
                    "name": "bridge_surface_api_parity",
                    "cmd": [py, "tests/run_seamgrim_bridge_surface_api_parity_check.py", *shared_parity_server_args],
                },
                {
                    "name": "space2d_api_parity",
                    "cmd": [py, "tests/run_seamgrim_space2d_api_parity_check.py", *shared_parity_server_args],
                },
                {
                    "name": "seamgrim_parity_server_lib_selftest",
                    "cmd": [py, "tests/run_seamgrim_parity_server_lib_selftest.py"],
                },
                {
                    "name": "ddn_exec_server_check",
                    "cmd": [py, "tests/run_seamgrim_ddn_exec_server_gate_check.py"]
                    + ([] if release_profile else ["--profile", "legacy"]),
                },
                {
                    "name": "pendulum_bogae_shape",
                    "cmd": [py, "tests/run_seamgrim_pendulum_bogae_shape_check.py"],
                },
                {
                    "name": "observe_output_contract",
                    "cmd": [py, SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME["seamgrim_observe_output_contract_check"]],
                },
                {
                    "name": "sam_seulgi_family_contract_selftest",
                    "cmd": [py, "tests/run_sam_seulgi_family_contract_selftest.py"],
                },
                {
                    "name": "full_check",
                    "cmd": full_cmd,
                },
            ]
    if release_profile:
        gate_step_defs = [row for row in gate_step_defs if str(row.get("name", "")).strip() not in legacy_only_gate_steps]
    steps.extend(run_steps_parallel(root, gate_step_defs))
    steps.extend(
        run_steps_parallel(
            root,
            [
                {
                    "name": "group_id_summary",
                    "cmd": [py, "tests/run_seamgrim_group_id_summary_check.py"],
                },
                {
                    "name": "runtime_fallback_metrics",
                    "cmd": [py, "tests/run_seamgrim_runtime_fallback_metrics_check.py"],
                },
                {
                    "name": "frontdoor_strict_all",
                    "cmd": [py, "tests/run_seamgrim_frontdoor_strict_all_check.py"],
                    "env_extra": {"DDN_ASSUME_WASM_CANON_PARITY_PASSED": "1"},
                },
                {
                    "name": "seamgrim_subject_representative_examples",
                    "cmd": [py, "tests/run_seamgrim_subject_representative_examples_check.py"],
                },
            ],
        )
    )
    steps.append(
        run_step(
            root,
            "runtime_fallback_policy",
            [py, "tests/run_seamgrim_runtime_fallback_policy_check.py"],
        )
    )
    steps.extend(
        run_steps_parallel(
            root,
            [
                {"name": "seamgrim_bridge_family_selftest", "cmd": [py, "tests/run_seamgrim_bridge_family_selftest.py"]},
                {
                    "name": "seamgrim_bridge_family_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_bridge_family_contract_selftest.py"],
                    "env_extra": contract_prereq_env,
                },
                {
                    "name": "seamgrim_bridge_family_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_bridge_family_contract_summary_selftest.py"],
                },
                {
                    "name": "seamgrim_bridge_family_transport_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_bridge_family_transport_contract_selftest.py"],
                    "env_extra": transport_contract_env,
                },
                {
                    "name": "seamgrim_bridge_family_transport_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_bridge_family_transport_contract_summary_selftest.py"],
                },
                {
                    "name": "state_view_hash_separation_family_selftest",
                    "cmd": [py, "tests/run_state_view_hash_separation_family_selftest.py"],
                },
                {
                    "name": "state_view_hash_separation_family_contract_selftest",
                    "cmd": [py, "tests/run_state_view_hash_separation_family_contract_selftest.py"],
                    "env_extra": contract_prereq_env,
                },
                {
                    "name": "state_view_hash_separation_family_contract_summary_selftest",
                    "cmd": [py, "tests/run_state_view_hash_separation_family_contract_summary_selftest.py"],
                },
                {
                    "name": "state_view_hash_separation_family_transport_contract_selftest",
                    "cmd": [py, "tests/run_state_view_hash_separation_family_transport_contract_selftest.py"],
                    "env_extra": transport_contract_env,
                },
                {
                    "name": "state_view_hash_separation_family_transport_contract_summary_selftest",
                    "cmd": [py, "tests/run_state_view_hash_separation_family_transport_contract_summary_selftest.py"],
                },
                {"name": "seamgrim_view_hash_family_selftest", "cmd": [py, "tests/run_seamgrim_view_hash_family_selftest.py"]},
                {
                    "name": "seamgrim_view_hash_family_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_view_hash_family_contract_selftest.py"],
                    "env_extra": contract_prereq_env,
                },
                {
                    "name": "seamgrim_view_hash_family_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_view_hash_family_contract_summary_selftest.py"],
                },
                {
                    "name": "seamgrim_view_hash_family_transport_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_view_hash_family_transport_contract_selftest.py"],
                    "env_extra": transport_contract_env,
                },
                {
                    "name": "seamgrim_view_hash_family_transport_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_view_hash_family_transport_contract_summary_selftest.py"],
                },
                {
                    "name": "seamgrim_state_view_boundary_family_selftest",
                    "cmd": [py, "tests/run_seamgrim_state_view_boundary_family_selftest.py"],
                },
                {
                    "name": "seamgrim_state_view_boundary_family_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_state_view_boundary_family_contract_selftest.py"],
                    "env_extra": contract_prereq_env,
                },
                {
                    "name": "seamgrim_state_view_boundary_family_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_state_view_boundary_family_contract_summary_selftest.py"],
                },
                {
                    "name": "seamgrim_state_view_boundary_family_transport_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_state_view_boundary_family_transport_contract_selftest.py"],
                    "env_extra": transport_contract_env,
                },
                {
                    "name": "seamgrim_state_view_boundary_family_transport_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_state_view_boundary_family_transport_contract_summary_selftest.py"],
                },
                {
                    "name": "seamgrim_consumer_surface_family_selftest",
                    "cmd": [py, "tests/run_seamgrim_consumer_surface_family_selftest.py"],
                },
                {
                    "name": "seamgrim_consumer_surface_family_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_consumer_surface_family_contract_selftest.py"],
                    "env_extra": contract_prereq_env,
                },
                {
                    "name": "seamgrim_consumer_surface_family_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_consumer_surface_family_contract_summary_selftest.py"],
                },
                {
                    "name": "seamgrim_consumer_surface_family_transport_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_consumer_surface_family_transport_contract_selftest.py"],
                    "env_extra": transport_contract_env,
                },
                {
                    "name": "seamgrim_consumer_surface_family_transport_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_consumer_surface_family_transport_contract_summary_selftest.py"],
                },
                {"name": "seamgrim_surface_family_selftest", "cmd": [py, "tests/run_seamgrim_surface_family_selftest.py"]},
                {
                    "name": "seamgrim_surface_family_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_surface_family_contract_selftest.py"],
                    "env_extra": contract_prereq_env,
                },
                {
                    "name": "seamgrim_surface_family_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_surface_family_contract_summary_selftest.py"],
                },
                {
                    "name": "seamgrim_surface_family_transport_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_surface_family_transport_contract_selftest.py"],
                    "env_extra": transport_contract_env,
                },
                {
                    "name": "seamgrim_surface_family_transport_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_surface_family_transport_contract_summary_selftest.py"],
                },
                {"name": "seamgrim_runtime_family_selftest", "cmd": [py, "tests/run_seamgrim_runtime_family_selftest.py"]},
                {
                    "name": "seamgrim_runtime_family_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_runtime_family_contract_selftest.py"],
                    "env_extra": contract_prereq_env,
                },
                {
                    "name": "seamgrim_runtime_family_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_runtime_family_contract_summary_selftest.py"],
                },
                {
                    "name": "seamgrim_runtime_family_transport_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_runtime_family_transport_contract_selftest.py"],
                    "env_extra": transport_contract_env,
                },
                {
                    "name": "seamgrim_runtime_family_transport_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_runtime_family_transport_contract_summary_selftest.py"],
                },
                {"name": "seamgrim_gate_family_selftest", "cmd": [py, "tests/run_seamgrim_gate_family_selftest.py"]},
                {
                    "name": "seamgrim_gate_family_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_gate_family_contract_selftest.py"],
                    "env_extra": contract_prereq_env,
                },
                {
                    "name": "seamgrim_gate_family_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_gate_family_contract_summary_selftest.py"],
                },
                {
                    "name": "seamgrim_gate_family_transport_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_gate_family_transport_contract_selftest.py"],
                    "env_extra": transport_contract_env,
                },
                {
                    "name": "seamgrim_gate_family_transport_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_gate_family_transport_contract_summary_selftest.py"],
                },
                {"name": "seamgrim_stack_family_selftest", "cmd": [py, "tests/run_seamgrim_stack_family_selftest.py"]},
                {
                    "name": "seamgrim_stack_family_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_stack_family_contract_selftest.py"],
                    "env_extra": contract_prereq_env,
                },
                {
                    "name": "seamgrim_stack_family_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_stack_family_contract_summary_selftest.py"],
                },
                {
                    "name": "seamgrim_stack_family_transport_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_stack_family_transport_contract_selftest.py"],
                    "env_extra": transport_contract_env,
                },
                {
                    "name": "seamgrim_stack_family_transport_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_stack_family_transport_contract_summary_selftest.py"],
                },
                {"name": "seamgrim_interaction_family_selftest", "cmd": [py, "tests/run_seamgrim_interaction_family_selftest.py"]},
                {
                    "name": "seamgrim_interaction_family_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_interaction_family_contract_selftest.py"],
                    "env_extra": {
                        **contract_prereq_env,
                        "DDN_ASSUME_INTERACTION_SMOKE_PASSED": "1",
                        "DDN_ASSUME_CONSUMER_SURFACE_TRANSPORT_PASSED": "1",
                    },
                },
                {
                    "name": "seamgrim_interaction_family_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_interaction_family_contract_summary_selftest.py"],
                },
                {
                    "name": "seamgrim_interaction_family_transport_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_interaction_family_transport_contract_selftest.py"],
                    "env_extra": transport_contract_env,
                },
                {
                    "name": "seamgrim_interaction_family_transport_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_interaction_family_transport_contract_summary_selftest.py"],
                },
                {"name": "seamgrim_application_family_selftest", "cmd": [py, "tests/run_seamgrim_application_family_selftest.py"]},
                {
                    "name": "seamgrim_application_family_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_application_family_contract_selftest.py"],
                    "env_extra": {**contract_prereq_env, "DDN_ASSUME_FAMILY_CONTRACT_PASSED": "1"},
                },
                {
                    "name": "seamgrim_application_family_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_application_family_contract_summary_selftest.py"],
                },
                {
                    "name": "seamgrim_application_family_transport_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_application_family_transport_contract_selftest.py"],
                    "env_extra": transport_contract_env,
                },
                {
                    "name": "seamgrim_application_family_transport_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_application_family_transport_contract_summary_selftest.py"],
                },
                {"name": "seamgrim_delivery_family_selftest", "cmd": [py, "tests/run_seamgrim_delivery_family_selftest.py"]},
                {
                    "name": "seamgrim_delivery_family_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_delivery_family_contract_selftest.py"],
                    "env_extra": {**contract_prereq_env, "DDN_ASSUME_FULL_GATE_PASSED": "1"},
                },
                {
                    "name": "seamgrim_delivery_family_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_delivery_family_contract_summary_selftest.py"],
                },
                {
                    "name": "seamgrim_delivery_family_transport_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_delivery_family_transport_contract_selftest.py"],
                    "env_extra": transport_contract_env,
                },
                {
                    "name": "seamgrim_delivery_family_transport_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_delivery_family_transport_contract_summary_selftest.py"],
                },
                {"name": "seamgrim_release_family_selftest", "cmd": [py, "tests/run_seamgrim_release_family_selftest.py"]},
                {
                    "name": "seamgrim_release_family_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_release_family_contract_selftest.py"],
                    "env_extra": contract_prereq_env,
                },
                {
                    "name": "seamgrim_release_family_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_release_family_contract_summary_selftest.py"],
                },
                {
                    "name": "seamgrim_release_family_transport_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_release_family_transport_contract_selftest.py"],
                    "env_extra": transport_contract_env,
                },
                {
                    "name": "seamgrim_release_family_transport_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_release_family_transport_contract_summary_selftest.py"],
                },
                {"name": "seamgrim_system_family_selftest", "cmd": [py, "tests/run_seamgrim_system_family_selftest.py"]},
                {
                    "name": "seamgrim_system_family_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_system_family_contract_selftest.py"],
                    "env_extra": contract_prereq_env,
                },
                {
                    "name": "seamgrim_system_family_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_system_family_contract_summary_selftest.py"],
                },
                {
                    "name": "seamgrim_system_family_transport_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_system_family_transport_contract_selftest.py"],
                    "env_extra": transport_contract_env,
                },
                {
                    "name": "seamgrim_system_family_transport_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_system_family_transport_contract_summary_selftest.py"],
                },
                {"name": "seamgrim_total_family_selftest", "cmd": [py, "tests/run_seamgrim_total_family_selftest.py"]},
                {
                    "name": "seamgrim_total_family_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_total_family_contract_selftest.py"],
                    "env_extra": {**contract_prereq_env, "DDN_ASSUME_FULL_GATE_PASSED": "1"},
                },
                {
                    "name": "seamgrim_total_family_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_total_family_contract_summary_selftest.py"],
                },
                {
                    "name": "seamgrim_total_family_transport_contract_selftest",
                    "cmd": [py, "tests/run_seamgrim_total_family_transport_contract_selftest.py"],
                    "env_extra": transport_contract_env,
                },
                {
                    "name": "seamgrim_total_family_transport_contract_summary_selftest",
                    "cmd": [py, "tests/run_seamgrim_total_family_transport_contract_summary_selftest.py"],
                },
            ],
            max_workers_override=_read_positive_int_env("DDN_SEAMGRIM_CI_GATE_FAMILY_MAX_WORKERS", 10),
        )
    )
    failed = [step for step in steps if not bool(step.get("ok"))]
    elapsed_total_ms = sum(int(step.get("elapsed_ms", 0)) for step in steps)
    result = {
        "schema": "seamgrim.ci_gate.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": len(failed) == 0,
        "profile": profile,
        "strict_graph": bool(args.strict_graph),
        "require_promoted": bool(args.require_promoted),
        "browse_selection_strict": bool(args.browse_selection_strict),
        "browse_selection_report_path": browse_selection_json_out,
        "runtime_5min_report_path": runtime_5min_json_out,
        "runtime_5min_browse_selection_report_path": runtime_browse_selection_json_out,
        "runtime_5min_skip_seed_cli": bool(args.runtime_5min_skip_seed_cli),
        "runtime_5min_skip_seed_cli_effective": bool(args.runtime_5min_skip_seed_cli or release_profile),
        "runtime_5min_skip_ui_common": bool(args.runtime_5min_skip_ui_common),
        "runtime_5min_skip_ui_common_effective": bool(args.runtime_5min_skip_ui_common or release_profile),
        "runtime_5min_skip_showcase_check": bool(args.runtime_5min_skip_showcase_check),
        "runtime_5min_skip_showcase_check_effective": bool(args.runtime_5min_skip_showcase_check or release_profile),
        "with_5min_checklist": bool(args.with_5min_checklist),
        "checklist_base_url": checklist_base_url,
        "checklist_from_runtime_report": checklist_from_runtime_report,
        "checklist_json_out": checklist_json_out,
        "checklist_markdown_out": checklist_markdown_out,
        "lesson_warning_report_path": lesson_warning_report_json_out,
        "lesson_warning_require_zero": bool(args.lesson_warning_require_zero),
        "require_preview_synced": bool(args.require_preview_synced),
        "ui_age3_report_path": str(args.ui_age3_json_out) if args.ui_age3_json_out else "",
        "sim_core_report_path": str(args.sim_core_json_out) if args.sim_core_json_out else "",
        "phase3_cleanup_report_path": str(args.phase3_cleanup_json_out) if args.phase3_cleanup_json_out else "",
        "rewrite_overlay_report_path": str(args.rewrite_overlay_json_out) if args.rewrite_overlay_json_out else "",
        "pack_evidence_report_path": pack_evidence_report_json_out,
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
        stop_parity_server(ddn_exec_server_proc)
        if args.print_drilldown:
            safe_print("[drilldown]")
            for step in failed:
                name = str(step.get("name", "-"))
                diagnostics = step.get("diagnostics") or []
                if not isinstance(diagnostics, list) or not diagnostics:
                    safe_print(f" - {name}: no parsed diagnostics")
                    continue
                for row in diagnostics:
                    if not isinstance(row, dict):
                        continue
                    kind = str(row.get("kind", "generic_error"))
                    target = str(row.get("target", "-"))
                    detail = str(row.get("detail", "")).strip()
                    safe_print(f" - {name}::{kind} target={target}")
                    if detail:
                        safe_print(f"   {detail}")
        names = ", ".join(str(step.get("name")) for step in failed)
        print(f"seamgrim ci gate failed: {names}")
        return 1

    stop_parity_server(ddn_exec_server_proc)
    print("seamgrim ci gate ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

