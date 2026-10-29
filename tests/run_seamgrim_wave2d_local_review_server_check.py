#!/usr/bin/env python
from __future__ import annotations

import socket
import subprocess
import sys
import time
import importlib.util
from pathlib import Path
from urllib.error import URLError
from urllib.request import Request, urlopen


def find_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def fetch(base_url: str, path: str) -> tuple[int, dict[str, str], bytes]:
    request = Request(f"{base_url}{path}", headers={"Cache-Control": "no-cache"})
    with urlopen(request, timeout=8) as response:
        headers = {str(key).lower(): str(value) for key, value in response.headers.items()}
        return int(response.status), headers, response.read()


def wait_ready(base_url: str, proc: subprocess.Popen[bytes]) -> None:
    deadline = time.monotonic() + 15
    last_error = ""
    while time.monotonic() < deadline:
      if proc.poll() is not None:
          raise RuntimeError(f"ddn_exec_server exited early with {proc.returncode}")
      try:
          status, headers, body = fetch(base_url, "/api/health")
          if status == 200 and b"ddn_exec_server" in body:
              cache_control = headers.get("cache-control", "")
              if "no-store" not in cache_control:
                  raise RuntimeError(f"/api/health missing no-store: {cache_control}")
              return
      except (OSError, URLError, TimeoutError) as exc:
          last_error = str(exc)
      time.sleep(0.15)
    raise RuntimeError(f"ddn_exec_server did not become ready: {last_error}")


def load_server_module(root: Path):
    path = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "ddn_exec_server.py"
    spec = importlib.util.spec_from_file_location("wave2d_ddn_exec_server", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"failed to load server module: {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def assert_no_store(headers: dict[str, str], path: str) -> None:
    cache_control = headers.get("cache-control", "")
    if "no-store" not in cache_control or "max-age=0" not in cache_control:
        raise AssertionError(f"{path} cache-control mismatch: {cache_control}")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    server_script = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "ddn_exec_server.py"
    server_module = load_server_module(root)
    smoke_dir = Path(server_module.resolve_build_dir()) / "reports"
    smoke_dir.mkdir(parents=True, exist_ok=True)
    smoke_name = f".tmp_wave2d_cache_smoke_{time.time_ns()}.js"
    smoke_path = smoke_dir / smoke_name
    smoke_url = f"/build/reports/{smoke_name}"
    port = find_free_port()
    base_url = f"http://127.0.0.1:{port}"
    proc: subprocess.Popen[bytes] | None = None
    try:
        proc = subprocess.Popen(
            [sys.executable, str(server_script), "--host", "127.0.0.1", "--port", str(port)],
            cwd=root,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        wait_ready(base_url, proc)
        paths = [
            "/",
            "/ui/app.js",
            "/api/health",
            "/api/lessons/inventory",
        ]
        for path in paths:
            status, headers, body = fetch(base_url, path)
            if status != 200:
                raise AssertionError(f"{path} status mismatch: {status}")
            if not body:
                raise AssertionError(f"{path} returned empty body")
            assert_no_store(headers, path)

        smoke_path.write_text("export const wave2dCacheSmoke = 'v1';\n", encoding="utf-8")
        status, headers, body = fetch(base_url, smoke_url)
        if status != 200 or b"'v1'" not in body:
            raise AssertionError(f"{smoke_url} v1 fetch mismatch: status={status} body={body[:80]!r}")
        assert_no_store(headers, smoke_url)

        smoke_path.write_text("export const wave2dCacheSmoke = 'v2';\n", encoding="utf-8")
        status, headers, body = fetch(base_url, smoke_url)
        if status != 200 or b"'v2'" not in body:
            raise AssertionError(f"{smoke_url} same-url reload did not see latest bytes: status={status} body={body[:80]!r}")
        assert_no_store(headers, smoke_url)
    finally:
        if proc is not None and proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=4)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=4)
        try:
            smoke_path.unlink(missing_ok=True)
        except OSError:
            pass

    print("seamgrim_wave2d_local_review_server: ok no-store latest-byte-smoke")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
