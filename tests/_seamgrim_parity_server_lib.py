from __future__ import annotations

import importlib.util
import subprocess
import sys
import time
from pathlib import Path


def load_module(module_name: str, path: Path):
    spec = importlib.util.spec_from_file_location(module_name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"module load failed: {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def start_parity_server(
    *,
    root: Path,
    module_name: str,
    host: str,
    port: int,
    timeout_sec: float,
    require_existing_server: bool,
):
    server_module = load_module(
        module_name,
        root / "solutions" / "seamgrim_ui_mvp" / "tools" / "ddn_exec_server_check.py",
    )
    server_script = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "ddn_exec_server.py"
    base_url = f"http://{host}:{port}"
    started_proc = None
    server_alive = bool(server_module.is_server_alive(base_url))
    if not server_alive:
        if require_existing_server:
            raise RuntimeError(f"ddn_exec_server not reachable: {base_url}")
        started_proc = subprocess.Popen(
            [sys.executable, str(server_script), "--host", host, "--port", str(port)],
            cwd=root,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    if not server_alive and not server_module.wait_for_server(base_url, timeout_sec=float(timeout_sec)):
        stop_parity_server(started_proc)
        raise RuntimeError("ddn_exec_server start timeout")
    return server_module, base_url, started_proc


def stop_parity_server(proc: subprocess.Popen[bytes] | None) -> None:
    if proc is None:
        return
    if proc.poll() is None:
        proc.terminate()
        try:
            proc.wait(timeout=2.0)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=2.0)
    else:
        time.sleep(0.05)
