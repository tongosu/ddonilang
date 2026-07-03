#!/usr/bin/env python3
"""Validate deterministic web output for STD_GRID_GAME_BOGAE_WEB_SHOWCASE_7DAY_V1."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

from _teul_cli_freshness import build_teul_cli_cmd as shared_build_teul_cli_cmd


ROOT = Path(__file__).resolve().parents[1]
PACK = "std_grid_game_bogae_web_out_determinism_v1"
PACK_DIR = ROOT / "pack" / PACK
INPUT = PACK_DIR / "input.ddn"
GOLDEN = PACK_DIR / "golden" / "webout_001_manifest_hash.test.json"


def fail(message: str) -> None:
    print(f"[std-grid-game-bogae-web-output] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def sha256_file(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def teul_cli_candidates() -> list[Path]:
    suffix = ".exe" if os.name == "nt" else ""
    return [
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        ROOT / "target" / "debug" / f"teul-cli{suffix}",
    ]


def build_teul_cli_cmd(args: list[str]) -> list[str]:
    return shared_build_teul_cli_cmd(
        ROOT,
        args,
        candidates=teul_cli_candidates(),
        include_which=False,
        manifest_path=ROOT / "tools" / "teul-cli" / "Cargo.toml",
    )


def tmp_root() -> Path:
    candidates = [
        Path("I:/home/urihanl/ddn/codex/build/tmp"),
        Path("C:/ddn/codex/build/tmp"),
        ROOT / "build" / "tmp",
    ]
    for candidate in candidates:
        try:
            candidate.mkdir(parents=True, exist_ok=True)
            return candidate
        except OSError:
            continue
    fallback = ROOT / "build" / "tmp"
    fallback.mkdir(parents=True, exist_ok=True)
    return fallback


def clean_dir(path: Path) -> None:
    if path.exists():
        last_error: Exception | None = None
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


def run_once(label: str, out_dir: Path) -> dict:
    clean_dir(out_dir)
    cmd = build_teul_cli_cmd(
        [
            "run",
            str(INPUT),
            "--madi",
            "4",
            "--bogae",
            "web",
            "--bogae-out",
            str(out_dir),
            "--no-open",
        ]
    )
    env = os.environ.copy()
    env.setdefault("RUST_MIN_STACK", str(64 * 1024 * 1024))
    env.setdefault(
        "CARGO_TARGET_DIR",
        str((ROOT / "build" / "cargo-target-pack-golden").resolve()),
    )
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
    )
    if proc.returncode != 0:
        fail(
            "\n".join(
                [
                    f"{label}: command failed",
                    "cmd: " + " ".join(cmd),
                    f"exit_code={proc.returncode}",
                    (proc.stderr or "").strip(),
                    (proc.stdout or "").strip(),
                ]
            ).strip()
        )
    stdout_lines = [line.strip() for line in (proc.stdout or "").splitlines() if line.strip()]
    if not stdout_lines or not stdout_lines[-1].startswith("bogae_hash=blake3:"):
        fail(f"{label}: missing final bogae_hash stdout")
    if "cmd_count=65" not in stdout_lines[-1] or "codec=BDL1" not in stdout_lines[-1]:
        fail(f"{label}: unexpected final stdout: {stdout_lines[-1]}")
    return summarize_output(label, out_dir)


def summarize_output(label: str, out_dir: Path) -> dict:
    manifest_path = out_dir / "manifest.detjson"
    if not manifest_path.is_file():
        fail(f"{label}: missing manifest.detjson")
    for rel in [
        "viewer/index.html",
        "viewer/live.html",
        "viewer/overlay.detjson",
        "viewer/viewer.js",
    ]:
        if not (out_dir / rel).is_file():
            fail(f"{label}: missing {rel}")

    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("kind") != "bogae_web_playback_v1":
        fail(f"{label}: manifest kind mismatch")
    if manifest.get("codec") != "BDL1":
        fail(f"{label}: manifest codec mismatch")
    if manifest.get("start_madi") != 0 or manifest.get("end_madi") != 4:
        fail(f"{label}: manifest madi range mismatch")
    frames = manifest.get("frames")
    if not isinstance(frames, list) or len(frames) != 4:
        fail(f"{label}: expected 4 frames")

    frame_paths: list[str] = []
    frame_hashes: dict[str, str] = {}
    for idx, frame in enumerate(frames):
        if not isinstance(frame, dict):
            fail(f"{label}: frame {idx} is not an object")
        expected_rel = f"frames/{idx:06d}.bdl1.detbin"
        if frame.get("file") != expected_rel:
            fail(f"{label}: frame {idx} file mismatch")
        if frame.get("madi") != idx:
            fail(f"{label}: frame {idx} madi mismatch")
        if frame.get("cmd_count") != 65:
            fail(f"{label}: frame {idx} cmd_count mismatch")
        if not str(frame.get("state_hash", "")).startswith("blake3:"):
            fail(f"{label}: frame {idx} missing state_hash")
        if not str(frame.get("bogae_hash", "")).startswith("blake3:"):
            fail(f"{label}: frame {idx} missing bogae_hash")
        frame_path = out_dir / expected_rel
        if not frame_path.is_file():
            fail(f"{label}: missing {expected_rel}")
        frame_paths.append(expected_rel)
        frame_hashes[expected_rel] = sha256_file(frame_path)

    return {
        "manifest_sha256": sha256_file(manifest_path),
        "frame_paths": frame_paths,
        "frame_hashes": frame_hashes,
    }


def expected_observations() -> tuple[str, dict[str, str]]:
    if not GOLDEN.is_file():
        fail(f"missing golden: {GOLDEN.relative_to(ROOT)}")
    payload = json.loads(GOLDEN.read_text(encoding="utf-8"))
    if payload.get("name") != "std_grid_game_bogae_webout_001_manifest_hash":
        fail("golden name mismatch")
    if payload.get("max_madi") != 4:
        fail("golden max_madi mismatch")
    grid = payload.get("grid")
    if grid != {"width": 8, "height": 8, "cell_px": 4, "canvas_w": 32, "canvas_h": 32}:
        fail("golden grid contract mismatch")
    det_expected = payload.get("det_expected", {})
    if det_expected.get("no_fault") is not True or det_expected.get("frame_count") != 4:
        fail("golden det_expected mismatch")

    manifest_hash: str | None = None
    frame_hashes: dict[str, str] = {}
    observations = det_expected.get("expected_observations")
    if not isinstance(observations, list):
        fail("golden expected_observations must be a list")
    for item in observations:
        if not isinstance(item, str):
            fail("golden observation must be a string")
        if item.startswith("manifest_sha256="):
            manifest_hash = item.removeprefix("manifest_sha256=")
        elif item.startswith("frame_sha256 "):
            path_part, hash_part = item.removeprefix("frame_sha256 ").split("=", 1)
            frame_hashes[path_part] = hash_part
    if manifest_hash is None:
        fail("golden missing manifest_sha256 observation")
    if sorted(frame_hashes) != [f"frames/{idx:06d}.bdl1.detbin" for idx in range(4)]:
        fail("golden frame_sha256 observations mismatch")
    return manifest_hash, frame_hashes


def main() -> None:
    if not INPUT.is_file():
        fail(f"missing input: {INPUT.relative_to(ROOT)}")
    expected_manifest_hash, expected_frame_hashes = expected_observations()
    base = tmp_root() / "std_grid_game_bogae_web_output_determinism" / str(time.time_ns())
    first = run_once("run-a", base / "a")
    second = run_once("run-b", base / "b")

    if first["manifest_sha256"] != second["manifest_sha256"]:
        fail("manifest sha256 differs across runs")
    if first["frame_paths"] != second["frame_paths"]:
        fail("frame path list differs across runs")
    if first["frame_hashes"] != second["frame_hashes"]:
        fail("frame sha256 differs across runs")
    if first["manifest_sha256"] != expected_manifest_hash:
        fail("manifest sha256 does not match golden observation")
    if first["frame_hashes"] != expected_frame_hashes:
        fail("frame sha256 values do not match golden observations")

    print("[std-grid-game-bogae-web-output] OK frames=4 canvas=32x32 cell=4")


if __name__ == "__main__":
    main()
