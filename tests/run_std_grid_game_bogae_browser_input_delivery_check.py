#!/usr/bin/env python3
"""Validate STD_GRID_GAME_BOGAE_BROWSER_INPUT_DELIVERY_V1 evidence."""

from __future__ import annotations

import json
import os
import queue
import shutil
import socket
import struct
import subprocess
import sys
import threading
import time
from pathlib import Path

from _teul_cli_freshness import build_teul_cli_cmd as shared_build_teul_cli_cmd


ROOT = Path(__file__).resolve().parents[1]
PACK = "std_grid_game_bogae_browser_input_delivery_v1"
LIVE_ASSETS = "std_grid_game_bogae_live_web_assets_v1"
NODE_OK = "std_grid_game_bogae_browser_input_delivery: ok"
MAGIC = b"DDN_INPUT_TAPE_V1\n"
KEY_BITS = {
    "ArrowLeft": 1 << 0,
    "ArrowRight": 1 << 1,
    "ArrowDown": 1 << 2,
}


def fail(message: str) -> None:
    print(f"[std-grid-game-bogae-browser-input] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def require_dir(path: Path) -> None:
    if not path.is_dir():
        fail(f"missing dir: {path.relative_to(ROOT)}")


def run(args: list[str], *, timeout: float | None = None) -> subprocess.CompletedProcess[str]:
    resolved = shutil.which(args[0]) or args[0]
    proc = subprocess.run(
        [resolved, *args[1:]],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
    )
    if proc.returncode != 0:
        print(proc.stdout, end="")
        print(proc.stderr, end="", file=sys.stderr)
        fail(f"command failed: {' '.join(args)}")
    return proc


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


def free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def tmp_root() -> Path:
    for candidate in [
        Path("I:/home/urihanl/ddn/codex/build/tmp"),
        Path("C:/ddn/codex/build/tmp"),
        ROOT / "build" / "tmp",
    ]:
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


def pipe_reader(name: str, pipe, events: "queue.Queue[tuple[str, str]]") -> None:
    try:
        for line in pipe:
            events.put((name, line.rstrip("\r\n")))
    finally:
        events.put((name, "__EOF__"))


def require_node_and_playwright() -> None:
    node = run(["node", "--version"]).stdout.strip()
    if not node.startswith("v") or int(node[1:].split(".", 1)[0]) < 22:
        fail(f"node v22+ required, got {node}")
    npm = run(["npm", "--version"]).stdout.strip()
    if not npm:
        fail("npm version is empty")
    pkg = load_json(ROOT / "package.json")
    dev_deps = pkg.get("devDependencies")
    if not isinstance(dev_deps, dict) or dev_deps.get("playwright") != "1.44.0":
        fail("package.json must pin playwright to 1.44.0")
    require_file(ROOT / "package-lock.json")
    resolved = run(["node", "-e", "console.log(require.resolve('playwright'))"]).stdout.strip()
    if "node_modules" not in resolved.replace("\\", "/"):
        fail(f"unexpected playwright resolve path: {resolved}")
    run(
        [
            "node",
            "-e",
            "const { chromium } = require('playwright'); chromium.launch({headless:true}).then(b=>b.close()).catch(e=>{console.error(e.message); process.exit(1)})",
        ],
        timeout=30,
    )


def require_pack() -> None:
    pack_dir = ROOT / "pack" / PACK
    require_dir(pack_dir)
    for rel in ["README.md", "input.ddn", "golden.jsonl", "contract.detjson"]:
        require_file(pack_dir / rel)
    expected = [
        "std_grid_game_bogae_browser_input_delivery_v1",
        "playwright_chromium",
        "keyboard_to_sam_live_endpoint",
        "repeat_keydown_no_duplicate",
        "record_sam_tape_observed",
    ]
    rows = load_jsonl(pack_dir / "golden.jsonl")
    if len(rows) != 1 or rows[0].get("stdout") != expected:
        fail("browser input delivery golden stdout mismatch")
    contract = load_json(pack_dir / "contract.detjson")
    if contract.get("schema") != "ddn.std_grid_game_bogae_browser_input_delivery.pack.contract.v1":
        fail("browser input delivery contract schema mismatch")
    if contract.get("playwright_version") != "1.44.0":
        fail("browser input delivery playwright_version mismatch")
    if contract.get("runner") != "tests/std_grid_game_bogae_browser_input_delivery_runner.mjs":
        fail("browser input delivery runner mismatch")
    if contract.get("checker") != "tests/run_std_grid_game_bogae_browser_input_delivery_check.py":
        fail("browser input delivery checker mismatch")
    if contract.get("generated_asset_dependency") != LIVE_ASSETS:
        fail("browser input delivery generated asset dependency mismatch")


def require_generated_assets(out_dir: Path) -> None:
    for rel in [
        "manifest.detjson",
        "viewer/live.html",
        "viewer/viewer.js",
        "viewer/overlay.detjson",
        "viewer/skin.detjson",
    ]:
        require_file(out_dir / rel)
    manifest = load_json(out_dir / "manifest.detjson")
    if manifest.get("kind") != "bogae_web_playback_v1":
        fail("generated manifest kind mismatch")
    if manifest.get("codec") != "BDL1":
        fail("generated manifest codec mismatch")
    frames = manifest.get("frames")
    if not isinstance(frames, list) or not frames:
        fail("generated manifest frames missing")
    first = frames[0]
    if not isinstance(first, dict) or not str(first.get("file", "")).startswith("frames/"):
        fail("generated first frame file mismatch")
    require_file(out_dir / str(first["file"]))


def read_u32(data: bytes, offset: int) -> tuple[int, int]:
    if offset + 4 > len(data):
        fail("input tape truncated")
    return struct.unpack_from("<I", data, offset)[0], offset + 4


def parse_input_tape(path: Path) -> tuple[int, list[tuple[int, int]]]:
    require_file(path)
    data = path.read_bytes()
    offset = 0
    if not data.startswith(MAGIC):
        fail("input tape magic mismatch")
    offset += len(MAGIC)
    version, offset = read_u32(data, offset)
    if version != 1:
        fail(f"input tape version mismatch: {version}")
    registry_len, offset = read_u32(data, offset)
    registry = data[offset : offset + registry_len]
    offset += registry_len
    if registry != b"KEY_REGISTRY_V1_MIN":
        fail("input tape registry id mismatch")
    offset += 32
    madi_hz, offset = read_u32(data, offset)
    record_count, offset = read_u32(data, offset)
    records: list[tuple[int, int]] = []
    for _ in range(record_count):
        madi, offset = read_u32(data, offset)
        mask_len, offset = read_u32(data, offset)
        mask_bytes = data[offset : offset + mask_len]
        offset += mask_len
        if len(mask_bytes) != mask_len:
            fail("input tape mask truncated")
        if mask_len < 1 or mask_len > 2:
            fail(f"input tape mask length mismatch: {mask_len}")
        raw = bytearray(2)
        raw[:mask_len] = mask_bytes
        records.append((madi, struct.unpack("<H", raw)[0]))
    if offset != len(data):
        fail("input tape has trailing bytes")
    return madi_hz, records


def find_first(records: list[tuple[int, int]], start: int, predicate) -> int | None:
    for idx in range(start, len(records)):
        if predicate(records[idx][1]):
            return idx
    return None


def require_tape_sequence(path: Path) -> None:
    madi_hz, records = parse_input_tape(path)
    if madi_hz != 10:
        fail(f"record-sam madi_hz mismatch: {madi_hz}")
    if not records:
        fail("record-sam tape has no records")
    madis = [madi for madi, _ in records]
    if madis != sorted(madis):
        fail("record-sam tape madi values are not chronological")

    left = KEY_BITS["ArrowLeft"]
    right = KEY_BITS["ArrowRight"]
    down = KEY_BITS["ArrowDown"]
    left_idx = find_first(records, 0, lambda mask: bool(mask & left))
    if left_idx is None:
        fail("record-sam tape never observed ArrowLeft")
    zero_after_left = find_first(records, left_idx + 1, lambda mask: (mask & left) == 0 and mask == 0)
    if zero_after_left is None:
        fail("record-sam tape never cleared ArrowLeft")
    right_idx = find_first(records, zero_after_left + 1, lambda mask: bool(mask & right))
    if right_idx is None:
        fail("record-sam tape never observed ArrowRight after left clear")
    zero_after_right = find_first(records, right_idx + 1, lambda mask: (mask & right) == 0 and mask == 0)
    if zero_after_right is None:
        fail("record-sam tape never cleared ArrowRight")
    down_idx = find_first(records, zero_after_right + 1, lambda mask: bool(mask & down))
    if down_idx is None:
        fail("record-sam tape never observed ArrowDown after right clear")
    zero_after_down = find_first(records, down_idx + 1, lambda mask: (mask & down) == 0 and mask == 0)
    if zero_after_down is None:
        fail("record-sam tape never cleared ArrowDown")


def wait_for_assets(out_dir: Path, proc: subprocess.Popen[str], timeout: float = 30.0) -> None:
    deadline = time.monotonic() + timeout
    required = [
        out_dir / "manifest.detjson",
        out_dir / "viewer" / "live.html",
        out_dir / "viewer" / "viewer.js",
        out_dir / "viewer" / "overlay.detjson",
        out_dir / "viewer" / "skin.detjson",
        out_dir / "frames" / "000000.bdl1.detbin",
    ]
    while time.monotonic() < deadline:
        if all(path.is_file() for path in required):
            return
        if proc.poll() is not None:
            fail("live process exited before generated assets were ready")
        time.sleep(0.05)
    fail("timed out waiting for generated live assets")


def run_live_browser_delivery() -> None:
    port = free_port()
    base = tmp_root() / "std_grid_game_bogae_browser_input_delivery" / str(time.time_ns())
    clean_dir(base)
    out_dir = base / "web"
    record_path = base / "browser_input.input.bin"
    expected_url = f"sam_live_input_url=http://127.0.0.1:{port}/input"
    input_url = f"http://127.0.0.1:{port}/input"
    cmd = build_teul_cli_cmd(
        [
            "run",
            str(ROOT / "pack" / LIVE_ASSETS / "input.ddn"),
            "--madi",
            "80",
            "--madi-hz",
            "10",
            "--record-sam",
            str(record_path),
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
    events: "queue.Queue[tuple[str, str]]" = queue.Queue()
    assert proc.stdout is not None
    assert proc.stderr is not None
    threading.Thread(target=pipe_reader, args=("stdout", proc.stdout, events), daemon=True).start()
    threading.Thread(target=pipe_reader, args=("stderr", proc.stderr, events), daemon=True).start()
    stdout_lines: list[str] = []
    stderr_lines: list[str] = []
    try:
        saw_url = False
        deadline = time.monotonic() + 90.0
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
            fail(f"missing live input url stderr; stderr={stderr_lines} stdout={stdout_lines}")

        wait_for_assets(out_dir, proc)
        node_proc = run(
            [
                "node",
                "tests/std_grid_game_bogae_browser_input_delivery_runner.mjs",
                str(out_dir),
                input_url,
            ],
            timeout=30,
        )
        if node_proc.stdout.strip() != NODE_OK:
            fail(f"node runner stdout mismatch: {node_proc.stdout.strip()}")

        try:
            code = proc.wait(timeout=20)
        except subprocess.TimeoutExpired:
            proc.kill()
            fail("live process did not finish")
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
            fail(f"live process failed code={code} stderr={stderr_lines} stdout={stdout_lines}")
        if not any(line.startswith("bogae_hash=blake3:") for line in stdout_lines):
            fail("live process missing final bogae_hash")
        require_generated_assets(out_dir)
        require_tape_sequence(record_path)
    finally:
        if proc.poll() is None:
            proc.kill()


def main() -> None:
    for path in [
        ROOT / "STD_GRID_GAME_BOGAE_BROWSER_DOM_SMOKE_V1.md",
        ROOT / "tests" / "run_std_grid_game_bogae_browser_dom_smoke_check.py",
        ROOT / "pack" / "std_grid_game_bogae_browser_dom_smoke_v1" / "contract.detjson",
        ROOT / "STD_GRID_GAME_BOGAE_LIVE_INPUT_BRIDGE_7DAY_V1.md",
        ROOT / "tests" / "run_std_grid_game_bogae_live_bridge_check.py",
        ROOT / "pack" / LIVE_ASSETS / "golden.jsonl",
        ROOT / "tests" / "std_grid_game_bogae_browser_input_delivery_runner.mjs",
        ROOT / "STD_GRID_GAME_BOGAE_BROWSER_INPUT_DELIVERY_V1.md",
        ROOT / "gaji" / "std_grid_game_bogae_browser_input_delivery" / "README.md",
        ROOT / "gaji" / "std_grid_game_bogae_browser_input_delivery" / "example.ddn",
    ]:
        require_file(path)
    require_node_and_playwright()
    require_pack()
    run_live_browser_delivery()
    run([sys.executable, "tests/run_pack_golden.py", PACK])
    print("[std-grid-game-bogae-browser-input] OK")


if __name__ == "__main__":
    main()
