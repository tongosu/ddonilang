#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import socket
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import urlparse


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


def strip_legacy_pragma_lines(text: str) -> str:
    source = str(text or "")
    lines = source.splitlines()
    filtered = [line for line in lines if not str(line).lstrip().startswith("#")]
    rendered = "\n".join(filtered)
    if source.endswith("\n"):
        rendered += "\n"
    return rendered


def run_seed_cli_step_with_pragma_strip(root: Path, name: str, lesson_rel: str, madi: int) -> dict[str, object]:
    lesson_path = root / str(lesson_rel)
    if not lesson_path.exists():
        return {
            "name": name,
            "ok": False,
            "elapsed_ms": 0,
            "cmd": [],
            "returncode": 127,
            "stdout": "",
            "stderr": f"lesson_missing:{lesson_path}",
        }
    source_text = lesson_path.read_text(encoding="utf-8")
    sanitized_text = strip_legacy_pragma_lines(source_text)
    with tempfile.TemporaryDirectory(prefix="seamgrim_runtime_5min_seed_") as temp_dir:
        lesson_copy = Path(temp_dir) / lesson_path.name
        lesson_copy.write_text(sanitized_text, encoding="utf-8")
        cmd = [
            "cargo",
            "run",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            "run",
            str(lesson_copy),
            "--madi",
            str(max(1, int(madi))),
        ]
        return run_step(root, name, cmd)


def print_step(step: dict[str, object]) -> None:
    def safe_console_text(text: str) -> str:
        encoding = (getattr(sys.stdout, "encoding", None) or "utf-8").strip() or "utf-8"
        return str(text).encode(encoding, errors="replace").decode(encoding, errors="replace")

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
    noisy_ok_steps = (
        name.startswith("seed_")
        or name == "rewrite_motion_projectile_fallback"
    )
    if ok and noisy_ok_steps:
        return

    if stdout:
        if ok:
            print(safe_console_text(clip_output(stdout, max_lines=6, max_chars=900)))
        else:
            print(safe_console_text(clip_output(stdout)))
    if stderr:
        if ok:
            print(safe_console_text(clip_output(stderr, max_lines=6, max_chars=900)))
        else:
            print(safe_console_text(clip_output(stderr)))


def _find_step(steps: list[dict[str, object]], name: str) -> dict[str, object] | None:
    for step in steps:
        if str(step.get("name", "")).strip() == str(name).strip():
            return step
    return None


def pick_free_local_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        return int(sock.getsockname()[1])


def resolve_server_check_base_url(raw_base_url: str) -> tuple[str, bool]:
    text = str(raw_base_url or "").strip()
    if text:
        return text.rstrip("/"), False
    port = pick_free_local_port()
    return f"http://127.0.0.1:{port}", True


def main() -> int:
    parser = argparse.ArgumentParser(description="Seamgrim 5-minute runtime validation scenario")
    parser.add_argument(
        "--base-url",
        default="",
        help="ddn_exec_server base url. 비우면 매 실행마다 전용 로컬 포트를 자동 할당한다.",
    )
    parser.add_argument(
        "--server-check-profile",
        default="release",
        choices=["release", "legacy"],
        help="ddn_exec_server_check profile (default: release)",
    )
    parser.add_argument("--madi", type=int, default=30)
    parser.add_argument("--skip-seed-cli", action="store_true")
    parser.add_argument("--skip-ui-common", action="store_true")
    parser.add_argument(
        "--skip-showcase-check",
        action="store_true",
        help="skip pendulum+tetris showcase check step",
    )
    parser.add_argument(
        "--showcase-smoke",
        action="store_true",
        help="run showcase check with real execution smoke for mini/full_preprocessed",
    )
    parser.add_argument("--showcase-smoke-madi-pendulum", type=int, default=20)
    parser.add_argument("--showcase-smoke-madi-tetris", type=int, default=20)
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
    server_check_base_url, server_check_isolated = resolve_server_check_base_url(str(args.base_url))

    steps: list[dict[str, object]] = []

    if not args.skip_seed_cli:
        steps.append(
            run_seed_cli_step_with_pragma_strip(
                root,
                "seed_econ_inventory_price_feedback",
                "solutions/seamgrim_ui_mvp/seed_lessons_v1/econ_inventory_price_feedback_seed_v1/lesson.ddn",
                madi,
            )
        )
        steps.append(
            run_seed_cli_step_with_pragma_strip(
                root,
                "seed_bio_sir_transition",
                "solutions/seamgrim_ui_mvp/seed_lessons_v1/bio_sir_transition_seed_v1/lesson.ddn",
                madi,
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
                server_check_base_url,
                "--profile",
                str(args.server_check_profile),
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
    steps.append(
        run_step(
            root,
            "runtime_view_source_strict",
            [py, "tests/run_seamgrim_runtime_view_source_strict_check.py"],
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
        steps.append(
            run_step(
                root,
                "nurimaker_grid_smoke",
                [py, "tests/run_nurimaker_grid_smoke_check.py"],
            )
        )
        steps.append(
            run_step(
                root,
                "rpgbox_block_smoke",
                [py, "tests/run_rpgbox_block_smoke_check.py"],
            )
        )
        steps.append(
            run_step(
                root,
                "block_editor_screen_smoke",
                [py, "tests/run_seamgrim_block_editor_smoke_check.py"],
            )
        )
        steps.append(run_step(root, "ui_common_runner", ["node", "tests/seamgrim_ui_common_runner.mjs"]))
        steps.append(
            run_step(
                root,
                "guideblock_keys_pack_check",
                [py, "tests/run_seamgrim_guideblock_keys_pack_check.py"],
            )
        )
        steps.append(
            run_step(
                root,
                "moyang_view_boundary_pack_check",
                [py, "tests/run_seamgrim_moyang_view_boundary_pack_check.py"],
            )
        )
        steps.append(run_step(root, "ui_pendulum_runner", ["node", "tests/seamgrim_pendulum_bogae_runner.mjs"]))
        steps.append(run_step(root, "wasm_vm_runtime_runner", ["node", "tests/seamgrim_wasm_vm_runtime_runner.mjs"]))
    if not args.skip_showcase_check:
        showcase_cmd = [py, "tests/run_pendulum_tetris_showcase_check.py"]
        if args.showcase_smoke:
            showcase_cmd.extend(
                [
                    "--with-smoke",
                    "--smoke-mode",
                    "web",
                    "--smoke-madi-pendulum",
                    str(max(1, int(args.showcase_smoke_madi_pendulum))),
                    "--smoke-madi-tetris",
                    str(max(1, int(args.showcase_smoke_madi_tetris))),
                ]
            )
        steps.append(
            run_step(
                root,
                "pendulum_tetris_showcase_check",
                showcase_cmd,
            )
        )

    for step in steps:
        print_step(step)

    ok = all(bool(step.get("ok")) for step in steps)
    elapsed_ms_total = sum(int(step.get("elapsed_ms") or 0) for step in steps)
    view_source_step = _find_step(steps, "runtime_view_source_strict")
    view_source_ok = bool(view_source_step and view_source_step.get("ok"))
    payload = {
        "schema": "seamgrim.runtime_5min_check.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": ok,
        "elapsed_ms_total": elapsed_ms_total,
        "view_source_strict_ok": view_source_ok,
        "server_check_base_url": server_check_base_url,
        "server_check_isolated": bool(server_check_isolated),
        "view_source_strict_step": {
            "name": str(view_source_step.get("name", "")) if view_source_step else "runtime_view_source_strict",
            "ok": bool(view_source_step.get("ok")) if view_source_step else False,
            "elapsed_ms": int(view_source_step.get("elapsed_ms") or 0) if view_source_step else 0,
            "returncode": int(view_source_step.get("returncode") or 0) if view_source_step else 127,
        },
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
