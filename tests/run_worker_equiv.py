#!/usr/bin/env python
import json
import subprocess
import sys
from pathlib import Path


def write_frame(stream, payload: dict) -> None:
    raw = json.dumps(payload, ensure_ascii=False, sort_keys=True).encode("utf-8")
    header = f"Content-Length: {len(raw)}\r\n\r\n".encode("utf-8")
    stream.write(header + raw)
    stream.flush()


def read_frame(stream) -> dict:
    length = None
    while True:
        line = stream.readline()
        if not line:
            raise RuntimeError("worker response EOF")
        line = line.strip()
        if not line:
            break
        text = line.decode("utf-8", errors="replace").lower()
        if text.startswith("content-length:"):
            length = int(text.split(":", 1)[1].strip())
    if length is None:
        raise RuntimeError("worker response missing content-length")
    body = stream.read(length)
    return json.loads(body.decode("utf-8"))


def normalize_lines(lines: list[str]) -> list[str]:
    out = []
    for line in lines:
        if not line:
            continue
        if line.startswith(("state_hash=", "trace_hash=", "bogae_hash=")):
            continue
        out.append(line)
    return out


def run_command(root: Path, input_path: Path) -> list[str]:
    manifest = root / "tools" / "teul-cli" / "Cargo.toml"
    cmd = [
        "cargo",
        "run",
        "-q",
        "--manifest-path",
        str(manifest),
        "--",
        "run",
        str(input_path),
    ]
    result = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or f"run failed: {result.returncode}")
    return normalize_lines([line.strip() for line in result.stdout.splitlines()])


def run_worker(root: Path, input_path: Path) -> list[str]:
    manifest = root / "tools" / "teul-cli" / "Cargo.toml"
    cmd = [
        "cargo",
        "run",
        "-q",
        "--manifest-path",
        str(manifest),
        "--",
        "worker",
    ]
    proc = subprocess.Popen(
        cmd,
        cwd=root,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    try:
        assert proc.stdin is not None
        assert proc.stdout is not None

        # reset negative (params not allowed)
        write_frame(
            proc.stdin,
            {"jsonrpc": "2.0", "id": 1, "method": "reset", "params": {"bad": True}},
        )
        resp = read_frame(proc.stdout)
        if "error" not in resp:
            raise RuntimeError("reset negative test failed: expected error response")

        # run_file
        write_frame(
            proc.stdin,
            {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "run_file",
                "params": {"path": str(input_path), "args": [], "mode": "inproc"},
            },
        )
        resp = read_frame(proc.stdout)
        if "error" in resp:
            raise RuntimeError(resp["error"].get("message", "worker error"))
        result = resp.get("result", {})
        if not result.get("ok", False):
            stderr_lines = result.get("stderr", [])
            message = "\n".join(stderr_lines) if stderr_lines else "worker run failed"
            raise RuntimeError(message)
        stdout_lines = [line.strip() for line in result.get("stdout", [])]
        return normalize_lines(stdout_lines)
    finally:
        if proc.poll() is None:
            proc.terminate()


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    input_path = root / "pack" / "age2_kernel_smoke_v1" / "input.ddn"
    run_lines = run_command(root, input_path)
    worker_lines = run_worker(root, input_path)
    if run_lines != worker_lines:
        print("worker vs run mismatch")
        print("run:", run_lines)
        print("worker:", worker_lines)
        return 1
    print("worker equivalence ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
