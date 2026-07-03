#!/usr/bin/env python3
"""Validate STD_GRID_GAME_BOGAE_LIVE_INPUT_BRIDGE_7DAY_V1 evidence."""

from __future__ import annotations

import json
import os
import queue
import shutil
import socket
import subprocess
import sys
import threading
import time
from pathlib import Path

from _teul_cli_freshness import build_teul_cli_cmd as shared_build_teul_cli_cmd


ROOT = Path(__file__).resolve().parents[1]
ASSETS = "std_grid_game_bogae_live_web_assets_v1"
INPUT_CONTRACT = "std_grid_game_bogae_live_input_contract_v1"
CLOSURE = "std_grid_game_bogae_live_bridge_closure_v1"
PLAYBACK_CLOSURE = "std_grid_game_bogae_web_playback_closure_v1"
ALL_PACKS = [ASSETS, INPUT_CONTRACT, CLOSURE]

EXPECTED_BOGAE_STDOUT = [
    "bogae_hash=blake3:dc4856b1baf4c5217660788764bfe0c81be70644c3260311f766b55e340387e2"
]
EXPECTED_INPUT_STDOUT = [
    "std_grid_game_bogae_live_input_contract_v1",
    "sam_live_input_url=http://127.0.0.1:<port>/input",
    "/input?code=ArrowLeft&kind=down",
    "/input?code=ArrowLeft&kind=up",
    "/input?kind=clear",
    "OPTIONS /input",
    "200 OK / 204 No Content + CORS",
]
EXPECTED_CLOSURE_STDOUT = [
    "std_grid_game_bogae_live_bridge_closure_v1",
    "8",
    "8",
    "4",
    "4",
    "BDL1",
    "bogae_web_playback_v1",
    "--bogae-live",
    "--sam-live web",
    "sam_live_input_url",
]
REQUIRED_ASSETS = [
    "manifest.detjson",
    "frames/000000.bdl1.detbin",
    "frames/000001.bdl1.detbin",
    "frames/000002.bdl1.detbin",
    "frames/000003.bdl1.detbin",
    "viewer/index.html",
    "viewer/live.html",
    "viewer/viewer.js",
    "viewer/overlay.detjson",
]


def fail(message: str) -> None:
    print(f"[std-grid-game-bogae-live-bridge] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def require_dir(path: Path) -> None:
    if not path.is_dir():
        fail(f"missing dir: {path.relative_to(ROOT)}")


def load_json(path: Path) -> dict:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(data, dict):
        fail(f"{path.relative_to(ROOT)} root must be object")
    return data


def load_jsonl(path: Path) -> list[dict]:
    rows: list[dict] = []
    for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.strip():
            continue
        try:
            row = json.loads(line)
        except json.JSONDecodeError as exc:
            fail(f"{path.relative_to(ROOT)}:{lineno}: invalid JSONL: {exc}")
        if not isinstance(row, dict):
            fail(f"{path.relative_to(ROOT)}:{lineno}: row must be object")
        rows.append(row)
    return rows


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
        shutil.rmtree(path)
    path.mkdir(parents=True, exist_ok=True)


def free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def require_pack(pack: str) -> None:
    pack_dir = ROOT / "pack" / pack
    require_dir(pack_dir)
    require_file(pack_dir / "README.md")
    require_file(pack_dir / "input.ddn")
    require_file(pack_dir / "golden.jsonl")
    rows = load_jsonl(pack_dir / "golden.jsonl")
    if len(rows) != 1:
        fail(f"pack/{pack}/golden.jsonl must have exactly one row")
    row = rows[0]
    if pack == ASSETS:
        if row.get("stdout") != EXPECTED_BOGAE_STDOUT:
            fail("live assets golden stdout mismatch")
        if row.get("stderr") != ["sam_live_input_url=http://127.0.0.1:56101/input"]:
            fail("live assets golden stderr mismatch")
    elif pack == INPUT_CONTRACT:
        if row.get("stdout") != EXPECTED_INPUT_STDOUT:
            fail("input contract golden stdout mismatch")
    elif pack == CLOSURE:
        if row.get("stdout") != EXPECTED_CLOSURE_STDOUT:
            fail("closure golden stdout mismatch")


def require_contract() -> None:
    contract = load_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if contract.get("schema") != "ddn.std_grid_game_bogae_live_bridge_closure.pack.contract.v1":
        fail("closure contract schema mismatch")
    if contract.get("public_surface_delta") != "none":
        fail("closure public_surface_delta mismatch")
    if contract.get("bundled_packs") != [ASSETS, INPUT_CONTRACT, PLAYBACK_CLOSURE]:
        fail("closure bundled_packs mismatch")
    if contract.get("manifest") != {"kind": "bogae_web_playback_v1", "codec": "BDL1"}:
        fail("closure manifest mismatch")
    if contract.get("web_sample") != {
        "grid_width": 8,
        "grid_height": 8,
        "cell_px": 4,
        "canvas_w": 32,
        "canvas_h": 32,
        "frame_count": 4,
    }:
        fail("closure web_sample mismatch")
    endpoint = contract.get("input_endpoint")
    if not isinstance(endpoint, dict) or endpoint.get("cors") is not True:
        fail("closure input_endpoint mismatch")
    if endpoint.get("routes") != [
        "/input?code=ArrowLeft&kind=down",
        "/input?code=ArrowLeft&kind=up",
        "/input?kind=clear",
        "OPTIONS /input",
    ]:
        fail("closure input routes mismatch")
    if contract.get("required_assets") != REQUIRED_ASSETS:
        fail("closure required_assets mismatch")
    if "gaji/std_grid_game_bogae_live_bridge" not in contract.get("gaji_skeletons", []):
        fail("closure missing gaji skeleton pointer")


def run(args: list[str]) -> None:
    proc = subprocess.run(args, cwd=ROOT, text=True, encoding="utf-8", errors="replace")
    if proc.returncode != 0:
        fail(f"command failed: {' '.join(args)}")


def require_generated_assets(out_dir: Path, expected_frames: int = 4) -> None:
    require_dir(out_dir)
    for rel in [
        "manifest.detjson",
        "viewer/index.html",
        "viewer/live.html",
        "viewer/viewer.js",
        "viewer/overlay.detjson",
    ]:
        require_file(out_dir / rel)
    for idx in range(expected_frames):
        require_file(out_dir / f"frames/{idx:06d}.bdl1.detbin")
    manifest = load_json(out_dir / "manifest.detjson")
    if manifest.get("kind") != "bogae_web_playback_v1":
        fail(f"{out_dir}: manifest kind mismatch")
    if manifest.get("codec") != "BDL1":
        fail(f"{out_dir}: manifest codec mismatch")
    if manifest.get("start_madi") != 0 or manifest.get("end_madi") != expected_frames:
        fail(f"{out_dir}: manifest madi range mismatch")
    frames = manifest.get("frames")
    if not isinstance(frames, list) or len(frames) != expected_frames:
        fail(f"{out_dir}: frame count mismatch")
    for idx, frame in enumerate(frames):
        if frame.get("madi") != idx:
            fail(f"{out_dir}: frame {idx} madi mismatch")
        if frame.get("file") != f"frames/{idx:06d}.bdl1.detbin":
            fail(f"{out_dir}: frame {idx} file mismatch")
        if not str(frame.get("state_hash", "")).startswith("blake3:"):
            fail(f"{out_dir}: frame {idx} state_hash missing")
        if not str(frame.get("bogae_hash", "")).startswith("blake3:"):
            fail(f"{out_dir}: frame {idx} bogae_hash missing")
    index_html = (out_dir / "viewer" / "index.html").read_text(encoding="utf-8")
    live_html = (out_dir / "viewer" / "live.html").read_text(encoding="utf-8")
    viewer_js = (out_dir / "viewer" / "viewer.js").read_text(encoding="utf-8")
    if "Bogae Playback Viewer" not in index_html:
        fail(f"{out_dir}: index title mismatch")
    if "Bogae Live Viewer" not in live_html or "data-live=\"1\"" not in live_html:
        fail(f"{out_dir}: live title/marker mismatch")
    for snippet in ["setupInputCapture", "../manifest.detjson", "frameMeta.file"]:
        if snippet not in viewer_js:
            fail(f"{out_dir}: viewer.js missing {snippet}")


def pipe_reader(name: str, pipe, events: queue.Queue[tuple[str, str]]) -> None:
    try:
        for line in pipe:
            events.put((name, line.rstrip("\r\n")))
    finally:
        events.put((name, "__EOF__"))


def http_request(port: int, request: str) -> str:
    last_error: OSError | None = None
    for _ in range(5):
        chunks: list[bytes] = []
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=2.0) as sock:
                sock.settimeout(2.0)
                sock.sendall(request.encode("ascii"))
                while True:
                    try:
                        chunk = sock.recv(4096)
                    except ConnectionAbortedError:
                        break
                    if not chunk:
                        break
                    chunks.append(chunk)
            if chunks:
                return b"".join(chunks).decode("utf-8", errors="replace")
        except OSError as exc:
            last_error = exc
            time.sleep(0.05)
    if last_error is not None:
        fail(f"http request failed: {last_error}")
    fail("http request returned empty response")


def require_http_response(label: str, response: str, status: str) -> None:
    if not response.startswith(f"HTTP/1.1 {status}"):
        fail(f"{label}: status mismatch: {response.splitlines()[:1]}")
    if "Access-Control-Allow-Origin: *" not in response:
        fail(f"{label}: missing CORS origin")
    if "Access-Control-Allow-Methods: GET, OPTIONS" not in response:
        fail(f"{label}: missing CORS methods")


def require_dynamic_input_endpoint() -> None:
    port = free_port()
    out_dir = tmp_root() / "std_grid_game_bogae_live_input_contract" / str(time.time_ns())
    clean_dir(out_dir)
    cmd = build_teul_cli_cmd(
        [
            "run",
            str(ROOT / "pack" / ASSETS / "input.ddn"),
            "--madi",
            "30",
            "--madi-hz",
            "10",
            "--bogae",
            "web",
            "--bogae-live",
            "--sam-live",
            "web",
            "--sam-live-port",
            str(port),
            "--bogae-out",
            str(out_dir),
            "--no-open",
        ]
    )
    env = os.environ.copy()
    env.setdefault("RUST_MIN_STACK", str(64 * 1024 * 1024))
    env.setdefault("CARGO_TARGET_DIR", str((ROOT / "build" / "cargo-target-pack-golden").resolve()))
    proc = subprocess.Popen(
        cmd,
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
    )
    events: queue.Queue[tuple[str, str]] = queue.Queue()
    assert proc.stdout is not None
    assert proc.stderr is not None
    threading.Thread(target=pipe_reader, args=("stdout", proc.stdout, events), daemon=True).start()
    threading.Thread(target=pipe_reader, args=("stderr", proc.stderr, events), daemon=True).start()

    stdout_lines: list[str] = []
    stderr_lines: list[str] = []
    expected_url = f"sam_live_input_url=http://127.0.0.1:{port}/input"
    deadline = time.monotonic() + 30.0
    saw_url = False
    while time.monotonic() < deadline and not saw_url:
        try:
            name, line = events.get(timeout=0.2)
        except queue.Empty:
            if proc.poll() is not None:
                break
            continue
        if line == "__EOF__":
            continue
        if name == "stdout":
            stdout_lines.append(line)
        else:
            stderr_lines.append(line)
            if line.strip() == expected_url:
                saw_url = True
    if not saw_url:
        proc.kill()
        fail(f"missing live input url stderr; stderr={stderr_lines} stdout={stdout_lines}")

    require_http_response(
        "down",
        http_request(port, "GET /input?code=ArrowLeft&kind=down HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n"),
        "200 OK",
    )
    require_http_response(
        "up",
        http_request(port, "GET /input?code=ArrowLeft&kind=up HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n"),
        "200 OK",
    )
    require_http_response(
        "clear",
        http_request(port, "GET /input?kind=clear HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n"),
        "200 OK",
    )
    require_http_response(
        "options",
        http_request(port, "OPTIONS /input HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n"),
        "204 No Content",
    )

    try:
        code = proc.wait(timeout=10)
    except subprocess.TimeoutExpired:
        proc.kill()
        fail("dynamic live command did not finish")
    while True:
        try:
            name, line = events.get_nowait()
        except queue.Empty:
            break
        if line == "__EOF__":
            continue
        if name == "stdout":
            stdout_lines.append(line)
        else:
            stderr_lines.append(line)
    if code != 0:
        fail(f"dynamic live command failed code={code} stderr={stderr_lines} stdout={stdout_lines}")
    if not any(line.startswith("bogae_hash=blake3:") for line in stdout_lines):
        fail("dynamic live command missing final bogae_hash")
    require_generated_assets(out_dir, expected_frames=30)


def main() -> None:
    require_file(ROOT / "STD_GRID_GAME_BOGAE_WEB_PLAYBACK_CONTROLS_6DAY_V1.md")
    require_file(ROOT / "STD_GRID_GAME_BOGAE_LIVE_INPUT_BRIDGE_7DAY_V1.md")
    require_file(ROOT / "tests" / "run_std_grid_game_bogae_web_playback_controls_check.py")
    require_file(ROOT / "pack" / PLAYBACK_CLOSURE / "contract.detjson")
    require_file(ROOT / "gaji" / "std_grid_game_bogae_live_bridge" / "README.md")
    require_file(ROOT / "gaji" / "std_grid_game_bogae_live_bridge" / "example.ddn")
    for pack in ALL_PACKS:
        require_pack(pack)
    require_contract()
    run([sys.executable, "tests/run_pack_golden.py", *ALL_PACKS])
    require_generated_assets(ROOT / "build" / ASSETS)
    require_dynamic_input_endpoint()
    print("[std-grid-game-bogae-live-bridge] OK")


if __name__ == "__main__":
    main()
