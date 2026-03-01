#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path


def run_step(root: Path, name: str, cmd: list[str]) -> dict[str, object]:
    started = time.perf_counter()
    try:
        proc = subprocess.run(
            cmd,
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
    except FileNotFoundError as exc:
        elapsed_ms = int((time.perf_counter() - started) * 1000)
        return {
            "name": name,
            "ok": False,
            "elapsed_ms": elapsed_ms,
            "cmd": cmd,
            "returncode": 127,
            "stdout": "",
            "stderr": str(exc),
        }

    elapsed_ms = int((time.perf_counter() - started) * 1000)
    return {
        "name": name,
        "ok": proc.returncode == 0,
        "elapsed_ms": elapsed_ms,
        "cmd": cmd,
        "returncode": proc.returncode,
        "stdout": (proc.stdout or "").strip(),
        "stderr": (proc.stderr or "").strip(),
    }


def print_step(step: dict[str, object]) -> None:
    def clip_output(text: str, max_lines: int = 24, max_chars: int = 4000) -> str:
        if not text:
            return ""
        if len(text) > max_chars:
            text = text[:max_chars] + "\n... (truncated by chars)"
        lines = text.splitlines()
        if len(lines) > max_lines:
            head = lines[: max_lines - 1]
            return "\n".join(head + [f"... (truncated {len(lines) - (max_lines - 1)} lines)"])
        return text

    name = str(step["name"])
    ok = bool(step["ok"])
    elapsed_ms = int(step["elapsed_ms"])
    print(f"[{name}] {'ok' if ok else 'fail'} ({elapsed_ms}ms)")
    stdout = str(step.get("stdout") or "")
    stderr = str(step.get("stderr") or "")
    if stdout:
        print(clip_output(stdout))
    if stderr:
        print(clip_output(stderr))


def main() -> int:
    parser = argparse.ArgumentParser(description="Seamgrim 5-minute runtime validation scenario")
    parser.add_argument("--base-url", default="http://127.0.0.1:8787")
    parser.add_argument("--madi", type=int, default=30)
    parser.add_argument("--skip-seed-cli", action="store_true")
    parser.add_argument("--skip-ui-common", action="store_true")
    parser.add_argument("--browse-selection-json-out", default="")
    parser.add_argument(
        "--browse-selection-strict",
        action="store_true",
        help="require browse selection json report to exist and validate",
    )
    parser.add_argument("--json-out")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    madi = max(1, int(args.madi))

    steps: list[dict[str, object]] = []

    if not args.skip_seed_cli:
        steps.append(
            run_step(
                root,
                "seed_econ_inventory_price_feedback",
                [
                    "cargo",
                    "run",
                    "--manifest-path",
                    "tools/teul-cli/Cargo.toml",
                    "--",
                    "run",
                    "solutions/seamgrim_ui_mvp/seed_lessons_v1/econ_inventory_price_feedback_seed_v1/lesson.ddn",
                    "--madi",
                    str(madi),
                ],
            )
        )
        steps.append(
            run_step(
                root,
                "seed_bio_sir_transition",
                [
                    "cargo",
                    "run",
                    "--manifest-path",
                    "tools/teul-cli/Cargo.toml",
                    "--",
                    "run",
                    "solutions/seamgrim_ui_mvp/seed_lessons_v1/bio_sir_transition_seed_v1/lesson.ddn",
                    "--madi",
                    str(madi),
                ],
            )
        )
        steps.append(
            run_step(
                root,
                "seed_physics_pendulum_export",
                [
                    py,
                    "tests/run_seamgrim_seed_pendulum_export_check.py",
                ],
            )
        )
        steps.append(
            run_step(
                root,
                "seed_physics_pendulum_bogae_shape",
                [
                    py,
                    "tests/run_seamgrim_pendulum_bogae_shape_check.py",
                ],
            )
        )
        steps.append(
            run_step(
                root,
                "rewrite_motion_projectile_fallback",
                [
                    py,
                    "tests/run_seamgrim_motion_projectile_fallback_check.py",
                ],
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
                str(args.base_url),
            ],
        )
    )
    steps.append(
        run_step(
            root,
            "lesson_path_fallback",
            [py, "tests/run_seamgrim_lesson_path_fallback_check.py"],
        )
    )
    browse_selection_report_path = str(args.browse_selection_json_out or "").strip()
    if args.browse_selection_strict and not browse_selection_report_path:
        browse_selection_report_path = str(
            (root / "build" / "reports" / "seamgrim_browse_selection_flow_runtime.detjson").as_posix()
        )

    browse_selection_cmd = [py, "tests/run_seamgrim_browse_selection_flow_check.py"]
    if browse_selection_report_path:
        browse_selection_cmd.extend(["--json-out", browse_selection_report_path])
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
                    browse_selection_report_path,
                ],
            )
        )
    if not args.skip_ui_common:
        steps.append(run_step(root, "ui_common_runner", ["node", "tests/seamgrim_ui_common_runner.mjs"]))

    for step in steps:
        print_step(step)

    ok = all(bool(step.get("ok")) for step in steps)
    elapsed_ms_total = sum(int(step.get("elapsed_ms") or 0) for step in steps)
    payload = {
        "schema": "seamgrim.runtime_5min_check.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": ok,
        "elapsed_ms_total": elapsed_ms_total,
        "browse_selection_report_path": browse_selection_report_path,
        "step_count": len(steps),
        "steps": steps,
    }

    if args.json_out:
        out_path = Path(args.json_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"[runtime-5min] report={out_path}")

    print(f"[runtime-5min] ok={int(ok)} steps={len(steps)} elapsed_ms_total={elapsed_ms_total}")
    if not ok:
        failed = [str(step.get("name")) for step in steps if not bool(step.get("ok"))]
        print(f"runtime 5min check failed: {', '.join(failed)}")
        return 1
    print("runtime 5min check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
