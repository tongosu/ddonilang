#!/usr/bin/env python
from __future__ import annotations

import tempfile
from pathlib import Path

from _seamgrim_parity_server_lib import start_parity_server, stop_parity_server


def _write_fake_tools(root: Path, *, is_alive: bool, wait_ok: bool) -> None:
    tools_dir = root / "solutions" / "seamgrim_ui_mvp" / "tools"
    tools_dir.mkdir(parents=True, exist_ok=True)
    check_script = tools_dir / "ddn_exec_server_check.py"
    check_script.write_text(
        "\n".join(
            [
                "from __future__ import annotations",
                "",
                "def is_server_alive(base_url: str) -> bool:",
                f"    return {bool(is_alive)!r}",
                "",
                "def wait_for_server(base_url: str, timeout_sec: float) -> bool:",
                f"    return {bool(wait_ok)!r}",
                "",
            ]
        ),
        encoding="utf-8",
    )
    server_script = tools_dir / "ddn_exec_server.py"
    server_script.write_text(
        "\n".join(
            [
                "#!/usr/bin/env python",
                "from __future__ import annotations",
                "",
                "import time",
                "",
                "while True:",
                "    time.sleep(0.2)",
                "",
            ]
        ),
        encoding="utf-8",
    )


def _expect_runtime_error(fn, expected_fragment: str) -> None:
    try:
        fn()
    except RuntimeError as exc:
        detail = str(exc)
        if expected_fragment not in detail:
            raise AssertionError(f"unexpected runtime error: {detail}") from exc
        return
    raise AssertionError("RuntimeError expected but not raised")


def test_require_existing_server_fail() -> None:
    with tempfile.TemporaryDirectory(prefix="seamgrim_parity_server_lib_require_") as tmp:
        root = Path(tmp)
        _write_fake_tools(root, is_alive=False, wait_ok=True)
        _expect_runtime_error(
            lambda: start_parity_server(
                root=root,
                module_name="seamgrim_parity_server_lib_require_existing",
                host="127.0.0.1",
                port=18871,
                timeout_sec=1.0,
                require_existing_server=True,
            ),
            "ddn_exec_server not reachable",
        )


def test_spawn_and_stop() -> None:
    with tempfile.TemporaryDirectory(prefix="seamgrim_parity_server_lib_spawn_") as tmp:
        root = Path(tmp)
        _write_fake_tools(root, is_alive=False, wait_ok=True)
        _module, _base_url, proc = start_parity_server(
            root=root,
            module_name="seamgrim_parity_server_lib_spawn",
            host="127.0.0.1",
            port=18872,
            timeout_sec=1.0,
            require_existing_server=False,
        )
        if proc is None:
            raise AssertionError("expected spawned proc, got None")
        if proc.poll() is not None:
            raise AssertionError("spawned proc already exited")
        stop_parity_server(proc)
        if proc.poll() is None:
            raise AssertionError("spawned proc still alive after stop")


def test_start_timeout() -> None:
    with tempfile.TemporaryDirectory(prefix="seamgrim_parity_server_lib_timeout_") as tmp:
        root = Path(tmp)
        _write_fake_tools(root, is_alive=False, wait_ok=False)
        _expect_runtime_error(
            lambda: start_parity_server(
                root=root,
                module_name="seamgrim_parity_server_lib_timeout",
                host="127.0.0.1",
                port=18873,
                timeout_sec=1.0,
                require_existing_server=False,
            ),
            "ddn_exec_server start timeout",
        )


def test_already_alive_no_spawn() -> None:
    with tempfile.TemporaryDirectory(prefix="seamgrim_parity_server_lib_alive_") as tmp:
        root = Path(tmp)
        _write_fake_tools(root, is_alive=True, wait_ok=True)
        _module, _base_url, proc = start_parity_server(
            root=root,
            module_name="seamgrim_parity_server_lib_alive",
            host="127.0.0.1",
            port=18874,
            timeout_sec=1.0,
            require_existing_server=False,
        )
        if proc is not None:
            stop_parity_server(proc)
            raise AssertionError("expected no spawned proc when server is already alive")


def main() -> int:
    tests = [
        test_require_existing_server_fail,
        test_spawn_and_stop,
        test_start_timeout,
        test_already_alive_no_spawn,
    ]
    for test in tests:
        test()
    print("[seamgrim-parity-server-lib-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
