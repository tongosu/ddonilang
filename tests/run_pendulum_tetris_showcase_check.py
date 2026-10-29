#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCHEMA = "ddn.showcase.pendulum_tetris.v1"
SCRIPT = "tools/scripts/run_pendulum_tetris_showcase.py"
PROFILES = ("mini", "full_preprocessed")


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def run_showcase(
    *,
    py: str,
    profile: str,
    mode: str,
    dry_run: bool,
    report: Path,
    madi_pendulum: int,
    madi_tetris: int,
) -> tuple[int, str, str]:
    cmd = [
        py,
        SCRIPT,
        "--mode",
        str(mode),
        "--tetris-profile",
        str(profile),
        "--json-out",
        str(report),
    ]
    if dry_run:
        cmd.append("--dry-run")
    else:
        cmd.extend(
            [
                "--madi-pendulum",
                str(madi_pendulum),
                "--madi-tetris",
                str(madi_tetris),
            ]
        )
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return int(proc.returncode), proc.stdout or "", proc.stderr or ""


def expected_step_names(profile: str, mode: str) -> set[str]:
    tetris_prefix = "tetris_mini" if profile == "mini" else "tetris_full_preprocessed"
    rows: set[str] = set()
    if mode in {"web", "both"}:
        rows.update({"pendulum_web", f"{tetris_prefix}_web"})
    if mode in {"console", "both"}:
        rows.update({"pendulum_console", f"{tetris_prefix}_console"})
    return rows


def validate_payload(payload: dict, *, profile: str, mode: str, dry_run: bool) -> str | None:
    if str(payload.get("schema", "")) != SCHEMA:
        return "schema mismatch"
    if not bool(payload.get("ok", False)):
        return "report ok=false"
    if bool(payload.get("dry_run", False)) != bool(dry_run):
        return "dry_run mismatch"
    if str(payload.get("tetris_profile", "")) != str(profile):
        return f"{profile} profile mismatch"

    tetris_input = str(payload.get("tetris_input", ""))
    tetris_origin = str(payload.get("tetris_input_origin", ""))
    if profile == "mini":
        if "showcase_tetris_mini_v1/input.ddn" not in tetris_input:
            return "mini tetris_input mismatch"
    else:
        if "tetris_full_preprocessed.ddn" not in tetris_input:
            return "full profile input path mismatch"
        if "pack/game_maker_tetris_full/input.ddn" not in tetris_origin:
            return "full profile input origin mismatch"

    expected_names = expected_step_names(profile, mode)
    steps = payload.get("steps")
    if not isinstance(steps, list) or len(steps) != len(expected_names):
        return "step count mismatch"
    names = [str(step.get("name", "")) for step in steps if isinstance(step, dict)]
    if set(names) != expected_names:
        return "step names mismatch"
    expected_status = "dry_run" if dry_run else "pass"
    for step in steps:
        if not isinstance(step, dict):
            return "invalid step row"
        if str(step.get("status", "")) != expected_status:
            return "step status mismatch"
    return None


def run_and_validate(
    *,
    py: str,
    profile: str,
    mode: str,
    dry_run: bool,
    report: Path,
    madi_pendulum: int,
    madi_tetris: int,
    label: str,
) -> int:
    rc, stdout, stderr = run_showcase(
        py=py,
        profile=profile,
        mode=mode,
        dry_run=dry_run,
        report=report,
        madi_pendulum=madi_pendulum,
        madi_tetris=madi_tetris,
    )
    if rc != 0:
        if stdout:
            print(stdout, end="")
        if stderr:
            print(stderr, end="", file=sys.stderr)
        print(f"[showcase-check] {label} script failed", file=sys.stderr)
        return rc

    payload = load_json(report)
    if not isinstance(payload, dict):
        print(f"[showcase-check] {label} report parse failed", file=sys.stderr)
        return 1
    err = validate_payload(payload, profile=profile, mode=mode, dry_run=dry_run)
    if err:
        print(f"[showcase-check] {label} {err}", file=sys.stderr)
        return 1
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="pendulum+tetris showcase contract check")
    parser.add_argument(
        "--with-smoke",
        action="store_true",
        help="run non-dry smoke execution for mini/full_preprocessed profiles",
    )
    parser.add_argument(
        "--smoke-mode",
        choices=("web", "console", "both"),
        default="web",
        help="execution mode used when --with-smoke is enabled",
    )
    parser.add_argument("--smoke-madi-pendulum", type=int, default=20)
    parser.add_argument("--smoke-madi-tetris", type=int, default=20)
    args = parser.parse_args()

    py = sys.executable
    report_mini = ROOT / "build" / "reports" / "pendulum_tetris_showcase_check.detjson"
    report_full = ROOT / "build" / "reports" / "pendulum_tetris_showcase_full_check.detjson"

    rc = run_and_validate(
        py=py,
        profile="mini",
        mode="both",
        dry_run=True,
        report=report_mini,
        madi_pendulum=max(1, int(args.smoke_madi_pendulum)),
        madi_tetris=max(1, int(args.smoke_madi_tetris)),
        label="mini dry-run",
    )
    if rc != 0:
        return rc
    rc = run_and_validate(
        py=py,
        profile="full_preprocessed",
        mode="both",
        dry_run=True,
        report=report_full,
        madi_pendulum=max(1, int(args.smoke_madi_pendulum)),
        madi_tetris=max(1, int(args.smoke_madi_tetris)),
        label="full_preprocessed dry-run",
    )
    if rc != 0:
        return rc

    if args.with_smoke:
        smoke_mode = str(args.smoke_mode)
        smoke_madi_pendulum = max(1, int(args.smoke_madi_pendulum))
        smoke_madi_tetris = max(1, int(args.smoke_madi_tetris))
        for profile in PROFILES:
            smoke_report = ROOT / "build" / "reports" / f"pendulum_tetris_showcase_{profile}_smoke_check.detjson"
            rc = run_and_validate(
                py=py,
                profile=profile,
                mode=smoke_mode,
                dry_run=False,
                report=smoke_report,
                madi_pendulum=smoke_madi_pendulum,
                madi_tetris=smoke_madi_tetris,
                label=f"{profile} smoke",
            )
            if rc != 0:
                return rc

    print(f"[showcase-check] ok report={report_mini}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
