#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

from _teul_cli_freshness import build_teul_cli_cmd as build_teul_cli_cmd_with_freshness


STATIC_PACK_INPUT = "pack/bogae_web_viewer_v1/input.ddn"
SIM_PACK_INPUT = "pack/nurimaker_grid_replay_v1/fixtures/input.ddn"
SCHEMA = "ddn.bogae_geoul_visibility_smoke.v1"


def default_report_path(file_name: str) -> str:
    candidates = [
        Path("I:/home/urihanl/ddn/codex/build/reports"),
        Path("C:/ddn/codex/build/reports"),
        Path("build/reports"),
    ]
    for base in candidates:
        try:
            base.mkdir(parents=True, exist_ok=True)
            return str(base / file_name)
        except OSError:
            continue
    return str(Path("build/reports") / file_name)


def default_tmp_root() -> Path:
    candidates = [
        Path("I:/home/urihanl/ddn/codex/build/tmp"),
        Path("C:/ddn/codex/build/tmp"),
        Path("build/tmp"),
    ]
    for base in candidates:
        try:
            base.mkdir(parents=True, exist_ok=True)
            return base
        except OSError:
            continue
    fallback = Path("build/tmp")
    fallback.mkdir(parents=True, exist_ok=True)
    return fallback


def teul_cli_candidates(root: Path) -> list[Path]:
    suffix = ".exe" if os.name == "nt" else ""
    return [
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        root / "target" / "debug" / f"teul-cli{suffix}",
    ]


def build_teul_cli_cmd(root: Path, args: list[str]) -> list[str]:
    return build_teul_cli_cmd_with_freshness(
        root,
        args,
        candidates=teul_cli_candidates(root),
        include_which=True,
    )


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def load_jsonl(path: Path) -> list[dict]:
    rows: list[dict] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line:
            continue
        data = json.loads(line)
        if isinstance(data, dict):
            rows.append(data)
    return rows


def extract_case_summary(case_name: str, entry: str, madi: int, out_dir: Path, cmd: list[str], proc: subprocess.CompletedProcess[str]) -> dict:
    viewer_dir = out_dir / "viewer"
    diag_path = out_dir / "geoul.diag.jsonl"
    run_manifest_path = out_dir / "run_manifest.detjson"
    viewer_manifest_path = viewer_dir / "manifest.detjson"
    viewer_index_html = viewer_dir / "viewer" / "index.html"
    viewer_live_html = viewer_dir / "viewer" / "live.html"

    diag_rows: list[dict] = []
    viewer_manifest: dict = {}
    run_manifest: dict = {}
    parse_errors: list[str] = []

    if diag_path.exists():
        try:
            diag_rows = load_jsonl(diag_path)
        except Exception as exc:  # pragma: no cover - defensive parse guard
            parse_errors.append(f"diag jsonl parse failed: {exc}")
    if viewer_manifest_path.exists():
        try:
            viewer_manifest = load_json(viewer_manifest_path)
        except Exception as exc:  # pragma: no cover - defensive parse guard
            parse_errors.append(f"viewer manifest parse failed: {exc}")
    if run_manifest_path.exists():
        try:
            run_manifest = load_json(run_manifest_path)
        except Exception as exc:  # pragma: no cover - defensive parse guard
            parse_errors.append(f"run manifest parse failed: {exc}")

    frames = viewer_manifest.get("frames", []) if isinstance(viewer_manifest, dict) else []
    frame_files_exist = True
    frame_hash_fields_ok = True
    if isinstance(frames, list):
        for frame in frames:
            if not isinstance(frame, dict):
                frame_files_exist = False
                frame_hash_fields_ok = False
                continue
            rel_file = str(frame.get("file", "")).strip()
            if not rel_file:
                frame_files_exist = False
            elif not (viewer_dir / rel_file).exists():
                frame_files_exist = False
            if not str(frame.get("state_hash", "")).strip():
                frame_hash_fields_ok = False
            if not str(frame.get("bogae_hash", "")).strip():
                frame_hash_fields_ok = False
            try:
                if int(frame.get("cmd_count", 0)) <= 0:
                    frame_hash_fields_ok = False
            except Exception:
                frame_hash_fields_ok = False
    else:
        frame_files_exist = False
        frame_hash_fields_ok = False
        frames = []

    state_hashes = [str(frame.get("state_hash", "")).strip() for frame in frames if isinstance(frame, dict)]
    bogae_hashes = [str(frame.get("bogae_hash", "")).strip() for frame in frames if isinstance(frame, dict)]

    checks = {
        "teul_cli_run_ok": proc.returncode == 0,
        "geoul_diag_exists": diag_path.exists(),
        "geoul_diag_non_empty": len(diag_rows) > 0,
        "geoul_run_config_present": any(str(row.get("event", "")) == "run_config" for row in diag_rows),
        "run_manifest_has_hashes": bool(str(run_manifest.get("state_hash", "")).strip())
        and bool(str(run_manifest.get("bogae_hash", "")).strip()),
        "viewer_manifest_has_frames": viewer_manifest_path.exists()
        and str(viewer_manifest.get("kind", "")) == "bogae_web_playback_v1"
        and len(frames) > 0,
        "viewer_frame_files_exist": frame_files_exist,
        "viewer_frame_hash_fields_ok": frame_hash_fields_ok,
        "viewer_html_exists": viewer_index_html.exists() and viewer_live_html.exists(),
        "parse_errors_empty": len(parse_errors) == 0,
    }
    return {
        "case_name": case_name,
        "entry": entry,
        "madi": int(max(1, madi)),
        "command": cmd,
        "exit_code": int(proc.returncode),
        "out_dir": str(out_dir),
        "diag_path": str(diag_path),
        "run_manifest_path": str(run_manifest_path),
        "viewer_manifest_path": str(viewer_manifest_path),
        "diag_line_count": len(diag_rows),
        "viewer_frame_count": len(frames),
        "state_hashes": state_hashes,
        "bogae_hashes": bogae_hashes,
        "parse_errors": parse_errors,
        "stderr": (proc.stderr or "").strip()[:240],
        "stdout": (proc.stdout or "").strip()[:240],
        "checks": checks,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Bogae/Geoul visibility smoke check")
    parser.add_argument(
        "--report-out",
        default=default_report_path("bogae_geoul_visibility_smoke.detjson"),
        help="output JSON report path",
    )
    parser.add_argument(
        "--madi",
        type=int,
        default=2,
        help="teul-cli run madi value for static viewer case",
    )
    parser.add_argument(
        "--sim-madi",
        type=int,
        default=3,
        help="teul-cli run madi value for simulation viewer case",
    )
    parser.add_argument(
        "--entry-static",
        default=STATIC_PACK_INPUT,
        help="static viewer ddn entry path",
    )
    parser.add_argument(
        "--entry-sim",
        default=SIM_PACK_INPUT,
        help="simulation viewer ddn entry path",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    report_out = Path(args.report_out)
    run_root = default_tmp_root() / "bogae_geoul_visibility_smoke"
    run_root.mkdir(parents=True, exist_ok=True)

    out_dir = run_root / datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    static_out_dir = out_dir / "static"
    sim_out_dir = out_dir / "simulation"

    static_cmd = build_teul_cli_cmd(
        root,
        [
            "run",
            str(args.entry_static),
            "--madi",
            str(max(1, int(args.madi))),
            "--bogae",
            "web",
            "--no-open",
            "--bogae-out",
            str(static_out_dir / "viewer"),
            "--diag",
            str(static_out_dir / "geoul.diag.jsonl"),
            "--run-manifest",
            str(static_out_dir / "run_manifest.detjson"),
        ],
    )
    static_proc = subprocess.run(
        static_cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )

    sim_cmd = build_teul_cli_cmd(
        root,
        [
            "run",
            str(args.entry_sim),
            "--madi",
            str(max(1, int(args.sim_madi))),
            "--bogae",
            "web",
            "--no-open",
            "--bogae-out",
            str(sim_out_dir / "viewer"),
            "--diag",
            str(sim_out_dir / "geoul.diag.jsonl"),
            "--run-manifest",
            str(sim_out_dir / "run_manifest.detjson"),
        ],
    )
    sim_proc = subprocess.run(
        sim_cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )

    static_case = extract_case_summary(
        case_name="static_viewer",
        entry=str(args.entry_static),
        madi=int(args.madi),
        out_dir=static_out_dir,
        cmd=static_cmd,
        proc=static_proc,
    )
    sim_case = extract_case_summary(
        case_name="simulation_viewer",
        entry=str(args.entry_sim),
        madi=int(args.sim_madi),
        out_dir=sim_out_dir,
        cmd=sim_cmd,
        proc=sim_proc,
    )

    sim_state_hash_changes = len({h for h in sim_case["state_hashes"] if h}) > 1
    sim_bogae_hash_changes = len({h for h in sim_case["bogae_hashes"] if h}) > 1

    checks = []
    for key, ok in static_case["checks"].items():
        checks.append({"name": f"static_{key}", "ok": bool(ok)})
    for key, ok in sim_case["checks"].items():
        checks.append({"name": f"sim_{key}", "ok": bool(ok)})
    checks.extend(
        [
            {"name": "sim_frame_count_min_2", "ok": int(sim_case["viewer_frame_count"]) >= 2},
            {"name": "sim_state_hash_changes", "ok": sim_state_hash_changes},
            {"name": "sim_bogae_hash_changes", "ok": sim_bogae_hash_changes},
        ]
    )

    overall_ok = all(bool(row.get("ok", False)) for row in checks)
    failure_digest = [row["name"] for row in checks if not bool(row.get("ok", False))]
    if static_case["exit_code"] != 0:
        if static_case["stderr"]:
            failure_digest.append(f"static_stderr={static_case['stderr']}")
        elif static_case["stdout"]:
            failure_digest.append(f"static_stdout={static_case['stdout']}")
    if sim_case["exit_code"] != 0:
        if sim_case["stderr"]:
            failure_digest.append(f"sim_stderr={sim_case['stderr']}")
        elif sim_case["stdout"]:
            failure_digest.append(f"sim_stdout={sim_case['stdout']}")
    for err in list(static_case["parse_errors"])[:4]:
        failure_digest.append(f"static_{err}")
    for err in list(sim_case["parse_errors"])[:4]:
        failure_digest.append(f"sim_{err}")

    report = {
        "schema": SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "overall_ok": overall_ok,
        "entry_static": str(args.entry_static),
        "entry_sim": str(args.entry_sim),
        "madi_static": max(1, int(args.madi)),
        "madi_sim": max(1, int(args.sim_madi)),
        "out_dir": str(out_dir),
        "checks": checks,
        "cases": {
            "static_viewer": static_case,
            "simulation_viewer": sim_case,
        },
        "simulation_hash_delta": {
            "state_hash_changes": sim_state_hash_changes,
            "bogae_hash_changes": sim_bogae_hash_changes,
        },
        "failure_digest": failure_digest[:12],
    }
    report_out.parent.mkdir(parents=True, exist_ok=True)
    report_out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    print(
        f"[bogae-geoul-smoke] ok={int(overall_ok)} checks={len(checks)} "
        f"failed={len(failure_digest)} sim_frames={sim_case['viewer_frame_count']} report={report_out}"
    )
    if not overall_ok:
        for item in failure_digest[:8]:
            print(f" - {item}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
