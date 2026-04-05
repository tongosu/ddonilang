#!/usr/bin/env python
"""SSOT v20.18.0 보개 backend/profile skeleton smoke."""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
PACK_ROOT = ROOT / "pack"


def resolve_tmp_root_base() -> Path:
    candidates = [
        Path("I:/home/urihanl/ddn/codex/build/tmp"),
        Path("C:/ddn/codex/build/tmp"),
        ROOT / "build" / "tmp",
    ]
    for base in candidates:
        try:
            base.mkdir(parents=True, exist_ok=True)
            return base
        except OSError:
            continue
    fallback = ROOT / "build" / "tmp"
    fallback.mkdir(parents=True, exist_ok=True)
    return fallback


TMP_ROOT = resolve_tmp_root_base() / "bogae_backend_profile_smoke"
TMP_ROOT.mkdir(parents=True, exist_ok=True)

PACKS = [
    "bogae_grid2d_smoke_v1",
    "bogae_backend_parity_console_web_v1",
    "nurimaker_grid_replay_v1",
]

HASH_RE = re.compile(r"^(state_hash|trace_hash|bogae_hash)=(.+)$")


def sort_json(value):
    if isinstance(value, list):
        return [sort_json(item) for item in value]
    if not isinstance(value, dict):
        return value
    return {key: sort_json(value[key]) for key in sorted(value)}


def format_json(value) -> str:
    return json.dumps(sort_json(value), ensure_ascii=False, indent=2) + "\n"


def sha256_bytes(payload: bytes) -> str:
    return "sha256:" + hashlib.sha256(payload).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def resolve_teul_cli_bin() -> Path | None:
    suffix = ".exe" if os.name == "nt" else ""
    candidates = [
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        ROOT / "target" / "debug" / f"teul-cli{suffix}",
    ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return None


def build_teul_cli_cmd(args: list[str]) -> list[str]:
    teul_cli = resolve_teul_cli_bin()
    if teul_cli is not None:
        return [str(teul_cli), *args]
    return [
        "cargo",
        "run",
        "-q",
        "--manifest-path",
        str(ROOT / "tools" / "teul-cli" / "Cargo.toml"),
        "--",
        *args,
    ]


def clean_dir(path: Path) -> None:
    if path.exists():
        last_error = None
        for attempt in range(10):
            try:
                shutil.rmtree(path)
                last_error = None
                break
            except PermissionError as exc:
                last_error = exc
                time.sleep(0.05 * (attempt + 1))
        if last_error is not None:
            raise last_error
    path.mkdir(parents=True, exist_ok=True)


def make_run_dir(pack_name: str) -> Path:
    run_dir = TMP_ROOT / pack_name / str(time.time_ns())
    run_dir.mkdir(parents=True, exist_ok=True)
    return run_dir


def extract_hashes(stdout: str) -> dict[str, str]:
    out: dict[str, str] = {}
    for raw_line in stdout.splitlines():
        for token in raw_line.strip().split():
            match = HASH_RE.match(token)
            if not match:
                continue
            out[match.group(1)] = match.group(2).strip()
    return out


def extract_non_hash_stdout(stdout: str) -> list[str]:
    out = []
    for raw_line in stdout.splitlines():
        line = raw_line.rstrip("\n")
        stripped = line.strip()
        if not stripped:
            continue
        tokens = stripped.split()
        if tokens and all(HASH_RE.match(token) for token in tokens):
            continue
        out.append(line)
    return out


def run_teul_cli(label: str, args: list[str], work_dir: Path) -> dict:
    cmd = build_teul_cli_cmd(args)
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    stdout = proc.stdout or ""
    stderr = proc.stderr or ""
    if proc.returncode != 0:
        raise RuntimeError(
            "\n".join(
                [
                    f"[FAIL] {label}",
                    f"cmd: {' '.join(cmd)}",
                    f"exit_code={proc.returncode}",
                    stderr.strip(),
                    stdout.strip(),
                ]
            ).strip()
        )
    hashes = extract_hashes(stdout)
    return {
        "label": label,
        "stdout_lines": [line for line in stdout.splitlines() if line.strip()],
        "stderr_lines": [line for line in stderr.splitlines() if line.strip()],
        "stdout_digest": sha256_bytes(stdout.encode("utf-8")),
        "non_hash_stdout_digest": sha256_bytes(
            "\n".join(extract_non_hash_stdout(stdout)).encode("utf-8")
        ),
        "state_hash": hashes.get("state_hash"),
        "trace_hash": hashes.get("trace_hash"),
        "bogae_hash": hashes.get("bogae_hash"),
    }


def summarize_drawlist_json(path: Path) -> dict:
    payload = read_json(path)
    cmds = payload.get("cmds") or []

    def is_integral_number(value) -> bool:
        return isinstance(value, (int, float)) and float(value).is_integer()

    rect_sizes = sorted(
        {
            int(float(cmd["w"]))
            for cmd in cmds
            if isinstance(cmd, dict)
            and str(cmd.get("kind")) == "RectFill"
            and is_integral_number(cmd.get("w"))
            and is_integral_number(cmd.get("h"))
            and int(float(cmd["w"])) == int(float(cmd["h"]))
        }
    )
    int_pixels = True
    for cmd in cmds:
        if not isinstance(cmd, dict):
            continue
        for key, value in cmd.items():
            if key in {"aa", "kind", "text", "uri", "color", "tint"}:
                continue
            if isinstance(value, bool):
                continue
            if is_integral_number(value):
                continue
            int_pixels = False
    return {
        "size_digest": sha256_file(path),
        "canvas_w": payload.get("width_px"),
        "canvas_h": payload.get("height_px"),
        "cmd_count": len(cmds),
        "cmd_kinds": [str(cmd.get("kind")) for cmd in cmds if isinstance(cmd, dict)],
        "rect_square_sizes": rect_sizes,
        "integer_pixel_fields": int_pixels,
    }


def summarize_run_manifest(path: Path) -> dict:
    payload = read_json(path)
    return {
        "digest": sha256_file(path),
        "state_hash": payload.get("state_hash"),
        "trace_hash": payload.get("trace_hash"),
        "bogae_hash": payload.get("bogae_hash"),
        "ticks": payload.get("ticks"),
        "contract": payload.get("contract"),
    }


def summarize_playback_manifest(path: Path) -> dict:
    payload = read_json(path)
    frames = payload.get("frames") or []
    return {
        "digest": sha256_file(path),
        "codec": payload.get("codec"),
        "frame_count": len(frames),
        "state_hashes": [str(frame.get("state_hash")) for frame in frames],
        "bogae_hashes": [str(frame.get("bogae_hash")) for frame in frames],
        "frame_files": [str(frame.get("file")) for frame in frames],
    }


def summarize_overlay(path: Path) -> dict:
    payload = read_json(path)
    return {
        "digest": sha256_file(path),
        "grid": bool(payload.get("grid")),
        "bounds": bool(payload.get("bounds")),
        "delta": bool(payload.get("delta")),
    }


def compare_frame_files(base_dir: Path, other_dir: Path, files: list[str]) -> list[dict]:
    rows = []
    for relative_name in files:
        base_path = base_dir / relative_name
        other_path = other_dir / relative_name
        rows.append(
            {
                "file": relative_name,
                "base_digest": sha256_file(base_path),
                "other_digest": sha256_file(other_path),
                "same_bytes": base_path.read_bytes() == other_path.read_bytes(),
            }
        )
    return rows


def run_bogae_grid2d_smoke(pack_dir: Path) -> dict:
    work_dir = make_run_dir(pack_dir.name)
    input_path = pack_dir / "fixtures" / "input.ddn"

    base_dir = work_dir / "baseline"
    web_dir = work_dir / "web_overlay"
    base_dir.mkdir(parents=True, exist_ok=True)
    web_dir.mkdir(parents=True, exist_ok=True)

    base_manifest = base_dir / "run_manifest.detjson"
    web_manifest = web_dir / "run_manifest.detjson"

    baseline = run_teul_cli(
        "bogae_grid2d.baseline",
        [
            "run",
            str(input_path),
            "--madi",
            "1",
            "--bogae-out",
            str(base_dir),
            "--run-manifest",
            str(base_manifest),
        ],
        base_dir,
    )
    web_variant = run_teul_cli(
        "bogae_grid2d.web_overlay",
        [
            "run",
            str(input_path),
            "--madi",
            "1",
            "--bogae",
            "web",
            "--no-open",
            "--bogae-overlay",
            "grid,bounds,delta",
            "--bogae-out",
            str(web_dir),
            "--run-manifest",
            str(web_manifest),
        ],
        web_dir,
    )

    base_manifest_summary = summarize_run_manifest(base_manifest)
    web_manifest_summary = summarize_run_manifest(web_manifest)
    web_drawlist_summary = summarize_drawlist_json(web_dir / "drawlist.json")
    overlay_summary = summarize_overlay(web_dir / "overlay.detjson")

    return {
        "schema": "ddn.bogae.grid2d_smoke.v1",
        "pack": pack_dir.name,
        "baseline": {
            "cli": baseline,
            "run_manifest": base_manifest_summary,
            "detbin_digest": sha256_file(base_dir / "drawlist.bdl1"),
        },
        "web_overlay": {
            "cli": web_variant,
            "run_manifest": web_manifest_summary,
            "drawlist": web_drawlist_summary,
            "overlay": overlay_summary,
            "detbin_digest": sha256_file(web_dir / "drawlist.bdl1"),
        },
        "checks": {
            "same_state_hash": base_manifest_summary["state_hash"] == web_manifest_summary["state_hash"],
            "same_bogae_hash": base_manifest_summary["bogae_hash"] == web_manifest_summary["bogae_hash"],
            "same_detbin_bytes": (base_dir / "drawlist.bdl1").read_bytes()
            == (web_dir / "drawlist.bdl1").read_bytes(),
            "integer_pixel_fields": web_drawlist_summary["integer_pixel_fields"],
            "grid2d_square_cells_detected": web_drawlist_summary["rect_square_sizes"] == [32],
        },
    }


def run_bogae_backend_parity(pack_dir: Path) -> dict:
    work_dir = make_run_dir(pack_dir.name)
    input_path = pack_dir / "fixtures" / "input.ddn"

    console_dir = work_dir / "console_headless"
    web_dir = work_dir / "web2d"
    console_dir.mkdir(parents=True, exist_ok=True)
    web_dir.mkdir(parents=True, exist_ok=True)

    console_manifest = console_dir / "run_manifest.detjson"
    web_manifest = web_dir / "run_manifest.detjson"

    console_run = run_teul_cli(
        "bogae_backend_parity.console",
        [
            "run",
            str(input_path),
            "--madi",
            "1",
            "--bogae",
            "console",
            "--bogae-out",
            str(console_dir),
            "--run-manifest",
            str(console_manifest),
        ],
        console_dir,
    )
    web_run = run_teul_cli(
        "bogae_backend_parity.web",
        [
            "run",
            str(input_path),
            "--madi",
            "1",
            "--bogae",
            "web",
            "--no-open",
            "--bogae-out",
            str(web_dir),
            "--run-manifest",
            str(web_manifest),
        ],
        web_dir,
    )

    console_manifest_summary = summarize_run_manifest(console_manifest)
    web_manifest_summary = summarize_run_manifest(web_manifest)

    return {
        "schema": "ddn.bogae.backend_parity_console_web.v1",
        "pack": pack_dir.name,
        "console_headless": {
            "cli": console_run,
            "run_manifest": console_manifest_summary,
            "detbin_digest": sha256_file(console_dir / "drawlist.bdl1"),
        },
        "web2d": {
            "cli": web_run,
            "run_manifest": web_manifest_summary,
            "detbin_digest": sha256_file(web_dir / "drawlist.bdl1"),
            "drawlist": summarize_drawlist_json(web_dir / "drawlist.json"),
        },
        "checks": {
            "same_state_hash": console_manifest_summary["state_hash"] == web_manifest_summary["state_hash"],
            "same_bogae_hash": console_manifest_summary["bogae_hash"] == web_manifest_summary["bogae_hash"],
            "same_detbin_bytes": (console_dir / "drawlist.bdl1").read_bytes()
            == (web_dir / "drawlist.bdl1").read_bytes(),
        },
    }


def run_nurimaker_grid_replay(pack_dir: Path) -> dict:
    work_dir = make_run_dir(pack_dir.name)
    input_path = pack_dir / "fixtures" / "input.ddn"

    base_dir = work_dir / "playback_base"
    variant_dir = work_dir / "playback_overlay"
    base_dir.mkdir(parents=True, exist_ok=True)
    variant_dir.mkdir(parents=True, exist_ok=True)

    base_run = run_teul_cli(
        "nurimaker_grid_replay.base",
        [
            "run",
            str(input_path),
            "--madi",
            "4",
            "--bogae",
            "web",
            "--no-open",
            "--bogae-out",
            str(base_dir),
        ],
        base_dir,
    )
    variant_run = run_teul_cli(
        "nurimaker_grid_replay.overlay",
        [
            "run",
            str(input_path),
            "--madi",
            "4",
            "--bogae",
            "web",
            "--no-open",
            "--bogae-overlay",
            "grid,bounds,delta",
            "--bogae-out",
            str(variant_dir),
        ],
        variant_dir,
    )

    base_manifest = summarize_playback_manifest(base_dir / "manifest.detjson")
    variant_manifest = summarize_playback_manifest(variant_dir / "manifest.detjson")
    frame_files = base_manifest["frame_files"]
    frame_digests = compare_frame_files(base_dir, variant_dir, frame_files)

    return {
        "schema": "ddn.nurimaker.grid_replay_smoke.v1",
        "pack": pack_dir.name,
        "base": {
            "cli": base_run,
            "manifest": base_manifest,
        },
        "overlay_variant": {
            "cli": variant_run,
            "manifest": variant_manifest,
            "overlay": summarize_overlay(variant_dir / "viewer" / "overlay.detjson"),
        },
        "checks": {
            "same_manifest_digest": base_manifest["digest"] == variant_manifest["digest"],
            "same_state_hash_sequence": base_manifest["state_hashes"] == variant_manifest["state_hashes"],
            "same_bogae_hash_sequence": base_manifest["bogae_hashes"] == variant_manifest["bogae_hashes"],
            "same_frame_bytes": all(row["same_bytes"] for row in frame_digests),
            "replay_changes_state": len(set(base_manifest["state_hashes"])) > 1,
        },
        "frame_file_digests": frame_digests,
    }


def run_pack(pack_name: str, update: bool) -> bool:
    pack_dir = PACK_ROOT / pack_name
    expected_path = pack_dir / "expected" / "smoke.detjson"
    if pack_name == "bogae_grid2d_smoke_v1":
        summary = run_bogae_grid2d_smoke(pack_dir)
    elif pack_name == "bogae_backend_parity_console_web_v1":
        summary = run_bogae_backend_parity(pack_dir)
    elif pack_name == "nurimaker_grid_replay_v1":
        summary = run_nurimaker_grid_replay(pack_dir)
    else:
        raise ValueError(f"unknown pack: {pack_name}")

    actual_text = format_json(summary)
    if update:
        expected_path.parent.mkdir(parents=True, exist_ok=True)
        expected_path.write_text(actual_text, encoding="utf-8")
        print(f"[update] {pack_name}")
        return True

    expected_text = expected_path.read_text(encoding="utf-8")
    if expected_text != actual_text:
        print(f"[FAIL] {pack_name}: expected mismatch", file=sys.stderr)
        return False
    print(f"[ok] {pack_name}")
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description="보개 backend/profile skeleton smoke 검증")
    parser.add_argument("packs", nargs="*", help="실행할 pack 이름")
    parser.add_argument("--update", action="store_true", help="golden 갱신")
    args = parser.parse_args()

    selected = args.packs or PACKS
    ok = True
    parallel_pack_check = (not args.update) and len(selected) > 1 and resolve_teul_cli_bin() is not None
    if parallel_pack_check:
        with ThreadPoolExecutor(max_workers=len(selected)) as executor:
            future_map = {pack_name: executor.submit(run_pack, pack_name, args.update) for pack_name in selected}
            for pack_name in selected:
                try:
                    pack_ok = bool(future_map[pack_name].result())
                except Exception as exc:
                    print(f"[FAIL] {pack_name}: {exc}", file=sys.stderr)
                    pack_ok = False
                ok = pack_ok and ok
    else:
        for pack_name in selected:
            try:
                pack_ok = bool(run_pack(pack_name, args.update))
            except Exception as exc:
                print(f"[FAIL] {pack_name}: {exc}", file=sys.stderr)
                pack_ok = False
            ok = pack_ok and ok
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
