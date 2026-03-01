#!/usr/bin/env python
from __future__ import annotations

import argparse
import shutil
import subprocess
import tempfile
from pathlib import Path

import sys


def fail(detail: str) -> int:
    print(f"check=seed_pendulum_export detail={detail}")
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


def main() -> int:
    parser = argparse.ArgumentParser(description="Seamgrim seed pendulum runtime smoke check")
    parser.add_argument(
        "--lesson",
        default="solutions/seamgrim_ui_mvp/seed_lessons_v1/physics_pendulum_seed_v1/lesson.ddn",
        help="pendulum seed lesson path",
    )
    parser.add_argument("--madi", type=int, default=420, help="madi count for teul-cli run")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    tools_dir = root / "solutions" / "seamgrim_ui_mvp" / "tools"
    if tools_dir.exists():
        sys.path.insert(0, str(tools_dir))
    try:
        from export_graph import preprocess_ddn_for_teul  # type: ignore
    except Exception:
        return fail("preprocess_import_failed")

    lesson = root / str(args.lesson)
    if not lesson.exists():
        return fail(f"lesson_missing:{lesson.as_posix()}")

    teul_cli = resolve_teul_cli_bin(root)
    if teul_cli is None:
        return fail("teul_cli_missing")

    madi = max(1, int(args.madi))
    original = lesson.read_text(encoding="utf-8")
    preprocessed = preprocess_ddn_for_teul(original, strip_draw=True)
    tmp_path = None
    if preprocessed != original:
        fd, tmp_name = tempfile.mkstemp(prefix="seed_pendulum_", suffix=".ddn")
        __import__("os").close(fd)
        tmp_path = Path(tmp_name)
        tmp_path.write_text(preprocessed, encoding="utf-8")
    run_target = tmp_path if tmp_path else lesson

    try:
        cmd = [str(teul_cli), "run", str(run_target), "--madi", str(madi)]
        proc = subprocess.run(
            cmd,
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
    finally:
        if tmp_path and tmp_path.exists():
            tmp_path.unlink()
    if proc.returncode != 0:
        stderr = (proc.stderr or "").strip()
        stdout = (proc.stdout or "").strip()
        detail = stderr or stdout or f"returncode={proc.returncode}"
        return fail(f"teul_run_failed:{detail}")

    lines = split_output_lines(proc.stdout)
    theta_pairs = extract_series_points(lines, "theta")
    if not theta_pairs:
        numbers = parse_numeric_lines(proc.stdout)
        if len(numbers) < 1200:
            return fail(f"numbers_too_few:{len(numbers)}")
        if len(numbers) % 2 != 0:
            numbers = numbers[:-1]
        pairs = [(numbers[i], numbers[i + 1]) for i in range(0, len(numbers), 2)]
        if len(pairs) < 600:
            return fail(f"pairs_too_few:{len(pairs)}")
        # 레거시 출력 순서: (t,theta), (t,omega), (t,energy) 반복
        theta_pairs = [pairs[i] for i in range(0, len(pairs), 3)]

    if len(theta_pairs) < 200:
        return fail(f"theta_pairs_too_few:{len(theta_pairs)}")

    x_values = [x for x, _ in theta_pairs]
    y_values = [y for _, y in theta_pairs]
    x_min = min(x_values)
    x_max = max(x_values)
    if x_min > 0.0:
        return fail(f"x_min_positive:{x_min}")
    if x_max < 4.0:
        return fail(f"x_max_too_small:{x_max}")
    if max(y_values) <= 0.0 or min(y_values) >= 0.0:
        return fail("theta_sign_span_missing")

    monotonic_break = next(
        (idx for idx in range(1, len(x_values)) if x_values[idx] < x_values[idx - 1]),
        -1,
    )
    if monotonic_break >= 0:
        return fail(f"x_not_monotonic:index={monotonic_break}")

    print(
        "seed pendulum runtime check ok "
        f"theta_points={len(theta_pairs)} x_min={x_min:.4f} x_max={x_max:.4f} "
        f"theta_min={min(y_values):.4f} theta_max={max(y_values):.4f}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
