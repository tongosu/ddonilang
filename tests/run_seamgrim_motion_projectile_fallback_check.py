#!/usr/bin/env python
from __future__ import annotations

import argparse
import re
import shutil
import subprocess
from pathlib import Path


def fail(detail: str) -> int:
    print(f"check=motion_projectile_fallback detail={detail}")
    return 1


def resolve_teul_cli_bin(root: Path) -> Path | None:
    suffix = ".exe" if __import__("os").name == "nt" else ""
    candidates = [
        root / "target" / "debug" / f"teul-cli{suffix}",
        root / "target" / "release" / f"teul-cli{suffix}",
        Path("I:/home/urihanl/ddn/codex/target/debug/teul-cli.exe"),
    ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    which = shutil.which("teul-cli")
    return Path(which) if which else None


def parse_numeric_lines(stdout: str) -> list[float]:
    out: list[float] = []
    for raw in (stdout or "").splitlines():
        line = raw.strip()
        if not line:
            continue
        if line.startswith("state_hash=") or line.startswith("trace_hash=") or line.startswith("bogae_hash="):
            continue
        try:
            out.append(float(line))
        except Exception:
            continue
    return out


def split_output_lines(stdout: str) -> list[str]:
    return [raw.strip() for raw in (stdout or "").splitlines() if raw.strip()]


def extract_series_points(lines: list[str], series_id: str) -> list[tuple[float, float]]:
    target = f"series:{str(series_id or '').strip().lower()}"
    if not target or target == "series:":
        return []
    out: list[tuple[float, float]] = []
    size = len(lines)
    i = 0
    while i < size:
        line = str(lines[i] or "").strip()
        if line.lower() != target:
            i += 1
            continue
        values: list[float] = []
        j = i + 1
        while j < size and len(values) < 2:
            next_line = str(lines[j] or "").strip()
            lower = next_line.lower()
            if lower.startswith("series:"):
                break
            if lower in {"space2d", "space2d.shape", "space2d_shape", "shape2d"}:
                break
            try:
                values.append(float(next_line))
            except Exception:
                pass
            j += 1
        if len(values) >= 2:
            out.append((values[0], values[1]))
        i = j
    return out


def run_lesson_output(root: Path, teul_cli: Path, lesson: Path, madi: int) -> tuple[list[float], list[str]]:
    cmd = [str(teul_cli), "run", str(lesson), "--madi", str(max(1, int(madi)))]
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or f"returncode={proc.returncode}"
        raise RuntimeError(detail)
    return parse_numeric_lines(proc.stdout), split_output_lines(proc.stdout)


def check_runjs_fallback_tokens(run_js: Path) -> tuple[bool, str]:
    text = run_js.read_text(encoding="utf-8")
    required = [
        "function synthesizePointSpace2dFromObservation(",
        '"x-observation-fallback"',
        '"xy-observation-fallback"',
        '["x", "x_pos", "pos_x", "px", "위치x"]',
        '["y", "y_pos", "pos_y", "py", "위치y"]',
        "synthesizePendulumSpace2dFromObservation(observation) ??",
        "synthesizePointSpace2dFromObservation(observation)",
    ]
    for token in required:
        if token not in text:
            return (False, token)
    return (True, "")


def check_motion_series(numbers: list[float], lines: list[str]) -> tuple[bool, str]:
    x_points = extract_series_points(lines, "x")
    v_points = extract_series_points(lines, "v")
    if x_points and v_points:
        count = min(len(x_points), len(v_points))
        if count < 30:
            return (False, f"motion_rows_too_few:{count}")
        t_values = [x_points[i][0] for i in range(count)]
        x_values = [x_points[i][1] for i in range(count)]
        v_values = [v_points[i][1] for i in range(count)]
    else:
        if len(numbers) < 120:
            return (False, f"motion_numbers_too_few:{len(numbers)}")
        usable = (len(numbers) // 3) * 3
        rows = [(numbers[i], numbers[i + 1], numbers[i + 2]) for i in range(0, usable, 3)]
        if len(rows) < 30:
            return (False, f"motion_rows_too_few:{len(rows)}")
        t_values = [row[0] for row in rows]
        x_values = [row[1] for row in rows]
        v_values = [row[2] for row in rows]

    monotonic_break = next((idx for idx in range(1, len(t_values)) if t_values[idx] < t_values[idx - 1]), -1)
    if monotonic_break >= 0:
        return (False, f"motion_t_not_monotonic:index={monotonic_break}")
    if max(x_values) - min(x_values) < 0.2:
        return (False, "motion_x_span_too_small")
    if max(v_values) - min(v_values) <= 0:
        return (False, "motion_v_span_too_small")
    return (True, "")


def check_projectile_series(numbers: list[float], lines: list[str]) -> tuple[bool, str]:
    xy_points = extract_series_points(lines, "xy")
    if xy_points:
        if len(xy_points) < 40:
            return (False, f"projectile_rows_too_few:{len(xy_points)}")
        x_values = [row[0] for row in xy_points]
        y_values = [row[1] for row in xy_points]
    else:
        if len(numbers) < 120:
            return (False, f"projectile_numbers_too_few:{len(numbers)}")
        usable = (len(numbers) // 2) * 2
        rows = [(numbers[i], numbers[i + 1]) for i in range(0, usable, 2)]
        if len(rows) < 40:
            return (False, f"projectile_rows_too_few:{len(rows)}")
        x_values = [row[0] for row in rows]
        y_values = [row[1] for row in rows]

    if max(x_values) - min(x_values) < 0.2:
        return (False, "projectile_x_span_too_small")
    if max(y_values) - min(y_values) < 0.02:
        return (False, "projectile_y_span_too_small")
    return (True, "")


def check_shape_block(lesson_text: str, lesson_id: str) -> tuple[bool, str]:
    if re.search(r"^\s*(보개|모양)\s*:?\s*\{", str(lesson_text or ""), re.MULTILINE):
        return (True, "")
    return (False, f"shape_block_missing:{lesson_id}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Seamgrim motion/projectile x/y fallback smoke check")
    parser.add_argument(
        "--motion-lesson",
        default="solutions/seamgrim_ui_mvp/lessons_rewrite_v1/physics_motion/lesson.ddn",
        help="motion lesson path",
    )
    parser.add_argument(
        "--projectile-lesson",
        default="solutions/seamgrim_ui_mvp/lessons_rewrite_v1/physics_projectile/lesson.ddn",
        help="projectile lesson path",
    )
    parser.add_argument("--motion-madi", type=int, default=120)
    parser.add_argument("--projectile-madi", type=int, default=220)
    parser.add_argument(
        "--run-js",
        default="solutions/seamgrim_ui_mvp/ui/screens/run.js",
        help="run.js path for fallback token check",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    motion_lesson = root / str(args.motion_lesson)
    projectile_lesson = root / str(args.projectile_lesson)
    run_js = root / str(args.run_js)

    if not motion_lesson.exists():
        return fail(f"motion_lesson_missing:{motion_lesson.as_posix()}")
    if not projectile_lesson.exists():
        return fail(f"projectile_lesson_missing:{projectile_lesson.as_posix()}")
    if not run_js.exists():
        return fail(f"run_js_missing:{run_js.as_posix()}")

    ok_tokens, token_detail = check_runjs_fallback_tokens(run_js)
    if not ok_tokens:
        return fail(f"runjs_fallback_token_missing:{token_detail}")

    motion_text = motion_lesson.read_text(encoding="utf-8")
    projectile_text = projectile_lesson.read_text(encoding="utf-8")
    ok_motion_shape, motion_shape_detail = check_shape_block(motion_text, "physics_motion")
    if not ok_motion_shape:
        return fail(motion_shape_detail)
    ok_projectile_shape, projectile_shape_detail = check_shape_block(projectile_text, "physics_projectile")
    if not ok_projectile_shape:
        return fail(projectile_shape_detail)

    teul_cli = resolve_teul_cli_bin(root)
    if teul_cli is None:
        return fail("teul_cli_missing")

    try:
        motion_numbers, motion_lines = run_lesson_output(root, teul_cli, motion_lesson, int(args.motion_madi))
    except Exception as exc:
        return fail(f"motion_run_failed:{exc}")
    ok_motion, motion_detail = check_motion_series(motion_numbers, motion_lines)
    if not ok_motion:
        return fail(motion_detail)

    try:
        projectile_numbers, projectile_lines = run_lesson_output(
            root,
            teul_cli,
            projectile_lesson,
            int(args.projectile_madi),
        )
    except Exception as exc:
        return fail(f"projectile_run_failed:{exc}")
    ok_projectile, projectile_detail = check_projectile_series(projectile_numbers, projectile_lines)
    if not ok_projectile:
        return fail(projectile_detail)

    print(
        "motion/projectile fallback check ok "
        f"motion_samples={len(motion_numbers)} projectile_samples={len(projectile_numbers)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
