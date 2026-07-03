#!/usr/bin/env python3
"""Validate STD_GRID_GAME_BOGAE_FINITE_LIVE_LOOP_V1 evidence."""

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
FINITE = "std_grid_game_bogae_finite_live_loop_v1"
FRAME_EFFECT = "std_grid_game_bogae_live_input_frame_effect_v1"
CLOSURE = "std_grid_game_bogae_finite_live_loop_closure_v1"
LIVE_ASSETS = "std_grid_game_bogae_live_web_assets_v1"
BROWSER_INPUT = "std_grid_game_bogae_browser_input_delivery_v1"
LIVE_BRIDGE = "std_grid_game_bogae_live_bridge_closure_v1"
NODE_OK = "std_grid_game_bogae_finite_live_loop: ok"
MADI_COUNT = 80
MADI_HZ = 10
RUN_TIMEOUT = 70.0
MAGIC = b"DDN_INPUT_TAPE_V1\n"
KEY_BITS = {
    "ArrowLeft": 1 << 0,
    "ArrowRight": 1 << 1,
    "ArrowDown": 1 << 2,
}


def fail(message: str) -> None:
    print(f"[std-grid-game-bogae-finite-live-loop] FAIL: {message}", file=sys.stderr)
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


def validate_tape(path: Path, *, expect_input: bool) -> tuple[int | None, list[tuple[int, int]]]:
    madi_hz, records = parse_input_tape(path)
    if madi_hz != MADI_HZ:
        fail(f"{path.name}: record-sam madi_hz mismatch: {madi_hz}")
    if not records:
        fail(f"{path.name}: record-sam tape has no records")
    madis = [madi for madi, _ in records]
    if madis != sorted(madis):
        fail(f"{path.name}: record-sam tape madi values are not chronological")
    nonzero = [(madi, mask) for madi, mask in records if mask != 0]
    if expect_input:
        if not nonzero:
            fail("input run record-sam tape never observed non-zero input")
        for name, bit in KEY_BITS.items():
            if not any(mask & bit for _, mask in records):
                fail(f"input run record-sam tape never observed {name}")
        first = nonzero[0][0]
        if first + 3 >= MADI_COUNT:
            fail(f"first non-zero input madi too late for comparison window: {first}")
        return first, records
    if nonzero:
        fail(f"control run record-sam tape unexpectedly observed input: {nonzero[:3]}")
    return None, records


def require_generated_assets(out_dir: Path, expected_frames: int = MADI_COUNT) -> dict:
    for rel in [
        "manifest.detjson",
        "viewer/index.html",
        "viewer/live.html",
        "viewer/viewer.js",
        "viewer/overlay.detjson",
        "viewer/skin.detjson",
    ]:
        require_file(out_dir / rel)
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
        if not isinstance(frame, dict):
            fail(f"{out_dir}: frame {idx} must be object")
        if frame.get("madi") != idx:
            fail(f"{out_dir}: frame {idx} madi mismatch")
        expected_file = f"frames/{idx:06d}.bdl1.detbin"
        if frame.get("file") != expected_file:
            fail(f"{out_dir}: frame {idx} file mismatch")
        if not str(frame.get("state_hash", "")).startswith("blake3:"):
            fail(f"{out_dir}: frame {idx} state_hash missing")
        if not str(frame.get("bogae_hash", "")).startswith("blake3:"):
            fail(f"{out_dir}: frame {idx} bogae_hash missing")
        require_file(out_dir / expected_file)
    return manifest


def wait_for_live_url(
    proc: subprocess.Popen[str],
    events: "queue.Queue[tuple[str, str]]",
    expected_url: str,
    stdout_lines: list[str],
    stderr_lines: list[str],
    timeout: float,
) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
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
                return
    fail(f"missing live input url stderr; stderr={stderr_lines} stdout={stdout_lines}")


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


def drain_events(events: "queue.Queue[tuple[str, str]]", stdout_lines: list[str], stderr_lines: list[str]) -> None:
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


def run_live_case(label: str, *, drive_browser: bool) -> tuple[Path, Path, dict, list[tuple[int, int]], int | None]:
    port = free_port()
    base = tmp_root() / "std_grid_game_bogae_finite_live_loop" / f"{label}_{time.time_ns()}"
    clean_dir(base)
    out_dir = base / "web"
    record_path = base / f"{label}.input.bin"
    expected_url = f"sam_live_input_url=http://127.0.0.1:{port}/input"
    input_url = f"http://127.0.0.1:{port}/input"
    cmd = build_teul_cli_cmd(
        [
            "run",
            str(ROOT / "pack" / LIVE_ASSETS / "input.ddn"),
            "--madi",
            str(MADI_COUNT),
            "--madi-hz",
            str(MADI_HZ),
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
        wait_for_live_url(proc, events, expected_url, stdout_lines, stderr_lines, timeout=30.0)
        if drive_browser:
            wait_for_assets(out_dir, proc)
            node_proc = run(
                [
                    "node",
                    "tests/std_grid_game_bogae_finite_live_loop_runner.mjs",
                    str(out_dir),
                    input_url,
                ],
                timeout=30,
            )
            if node_proc.stdout.strip() != NODE_OK:
                fail(f"node runner stdout mismatch: {node_proc.stdout.strip()}")

        try:
            code = proc.wait(timeout=RUN_TIMEOUT)
        except subprocess.TimeoutExpired:
            proc.kill()
            fail(f"{label} live process did not finish within {RUN_TIMEOUT:.0f}s")
        drain_events(events, stdout_lines, stderr_lines)
        if code != 0:
            fail(f"{label} live process failed code={code} stderr={stderr_lines} stdout={stdout_lines}")
        if not any(line.startswith("bogae_hash=blake3:") for line in stdout_lines):
            fail(f"{label} live process missing final bogae_hash")
        manifest = require_generated_assets(out_dir)
        first_nonzero, records = validate_tape(record_path, expect_input=drive_browser)
        return out_dir, record_path, manifest, records, first_nonzero
    finally:
        if proc.poll() is None:
            proc.kill()


def compare_frame_effect(control_manifest: dict, input_manifest: dict, first_nonzero_madi: int) -> None:
    control_frames = control_manifest["frames"]
    input_frames = input_manifest["frames"]
    start_madi = first_nonzero_madi + 3
    if start_madi >= len(input_frames):
        fail(f"input comparison window unavailable: start={start_madi}")
    state_diff = []
    bogae_diff = []
    for idx in range(start_madi, len(input_frames)):
        control_frame = control_frames[idx]
        input_frame = input_frames[idx]
        if control_frame["state_hash"] != input_frame["state_hash"]:
            state_diff.append(idx)
        if control_frame["bogae_hash"] != input_frame["bogae_hash"]:
            bogae_diff.append(idx)
    if not state_diff:
        fail("no state_hash difference observed after input + 3 ticks")
    if not bogae_diff:
        fail("no bogae_hash difference observed after input + 3 ticks")


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
    run(
        [
            "node",
            "-e",
            "const { chromium } = require('playwright'); chromium.launch({headless:true}).then(b=>b.close()).catch(e=>{console.error(e.message); process.exit(1)})",
        ],
        timeout=30,
    )


def require_pack_contracts() -> None:
    expected_stdout = {
        FINITE: [
            "std_grid_game_bogae_finite_live_loop_v1",
            "finite_madi=80",
            "madi_hz=10",
            "frame_count=80",
            "final_manifest_only_claim",
        ],
        FRAME_EFFECT: [
            "std_grid_game_bogae_live_input_frame_effect_v1",
            "control_then_input",
            "record_sam_nonzero",
            "state_hash_diff_after_input_plus_3_ticks",
            "bogae_hash_aux_diff",
        ],
        CLOSURE: [
            "std_grid_game_bogae_finite_live_loop_closure_v1",
            LIVE_BRIDGE,
            BROWSER_INPUT,
            FINITE,
            FRAME_EFFECT,
        ],
    }
    for pack, stdout in expected_stdout.items():
        pack_dir = ROOT / "pack" / pack
        require_dir(pack_dir)
        for rel in ["README.md", "input.ddn", "golden.jsonl", "contract.detjson"]:
            require_file(pack_dir / rel)
        rows = load_jsonl(pack_dir / "golden.jsonl")
        if len(rows) != 1 or rows[0].get("stdout") != stdout:
            fail(f"{pack} golden stdout mismatch")
        contract = load_json(pack_dir / "contract.detjson")
        if contract.get("id") != pack:
            fail(f"{pack} contract id mismatch")
        if contract.get("public_surface_delta") != "none":
            fail(f"{pack} public_surface_delta mismatch")

    finite_contract = load_json(ROOT / "pack" / FINITE / "contract.detjson")
    if finite_contract.get("schema") != "ddn.std_grid_game_bogae_finite_live_loop.pack.contract.v1":
        fail("finite live loop contract schema mismatch")
    if finite_contract.get("cli_flag") != "--bogae-out":
        fail("finite live loop cli flag mismatch")
    if finite_contract.get("live_cli") != {
        "bogae": "web",
        "bogae_live": True,
        "sam_live": "web",
        "finite_madi": MADI_COUNT,
        "madi_hz": MADI_HZ,
        "no_open": True,
    }:
        fail("finite live loop live_cli mismatch")
    if finite_contract.get("manifest_claim") != "final manifest contains all frame metadata":
        fail("finite live loop manifest claim mismatch")

    effect_contract = load_json(ROOT / "pack" / FRAME_EFFECT / "contract.detjson")
    if effect_contract.get("schema") != "ddn.std_grid_game_bogae_live_input_frame_effect.pack.contract.v1":
        fail("frame effect contract schema mismatch")
    if effect_contract.get("primary_compare_field") != "state_hash":
        fail("frame effect primary compare field mismatch")
    if effect_contract.get("compare_after_ticks") != 3:
        fail("frame effect compare_after_ticks mismatch")
    if effect_contract.get("run_order") != ["control", "input"]:
        fail("frame effect run_order mismatch")

    closure_contract = load_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure_contract.get("schema") != "ddn.std_grid_game_bogae_finite_live_loop_closure.pack.contract.v1":
        fail("closure contract schema mismatch")
    if closure_contract.get("bundled_packs") != [LIVE_BRIDGE, BROWSER_INPUT, FINITE, FRAME_EFFECT]:
        fail("closure bundled_packs mismatch")
    if "gaji/std_grid_game_bogae_finite_live_loop" not in closure_contract.get("gaji_skeletons", []):
        fail("closure missing gaji skeleton pointer")


def main() -> None:
    for path in [
        ROOT / "STD_GRID_GAME_BOGAE_BROWSER_INPUT_DELIVERY_V1.md",
        ROOT / "tests" / "run_std_grid_game_bogae_browser_input_delivery_check.py",
        ROOT / "pack" / BROWSER_INPUT / "contract.detjson",
        ROOT / "pack" / LIVE_ASSETS / "golden.jsonl",
        ROOT / "pack" / "std_grid_game_bogae_viewer_js_live_input_v1" / "golden.jsonl",
        ROOT / "pack" / "std_grid_game_bogae_viewer_js_playback_controls_v1" / "golden.jsonl",
        ROOT / "tests" / "std_grid_game_bogae_finite_live_loop_runner.mjs",
        ROOT / "STD_GRID_GAME_BOGAE_FINITE_LIVE_LOOP_V1.md",
        ROOT / "gaji" / "std_grid_game_bogae_finite_live_loop" / "README.md",
        ROOT / "gaji" / "std_grid_game_bogae_finite_live_loop" / "example.ddn",
    ]:
        require_file(path)
    require_node_and_playwright()
    require_pack_contracts()
    _, _, control_manifest, _, _ = run_live_case("control", drive_browser=False)
    _, _, input_manifest, _, first_nonzero = run_live_case("input", drive_browser=True)
    assert first_nonzero is not None
    compare_frame_effect(control_manifest, input_manifest, first_nonzero)
    run([sys.executable, "tests/run_pack_golden.py", FINITE, FRAME_EFFECT, CLOSURE])
    print("[std-grid-game-bogae-finite-live-loop] OK")


if __name__ == "__main__":
    main()
