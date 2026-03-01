from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def safe_print(text: str, *, end: str = "\n", file=None) -> None:
    stream = file or sys.stdout
    data = str(text)
    try:
        print(data, end=end, file=stream)
        return
    except UnicodeEncodeError:
        pass
    encoded = data.encode(getattr(stream, "encoding", None) or "utf-8", errors="replace")
    decoded = encoded.decode(getattr(stream, "encoding", None) or "utf-8", errors="replace")
    print(decoded, end=end, file=stream)


def sanitize_step_name(value: str) -> str:
    raw = str(value).strip()
    if not raw:
        return "step"
    out_chars: list[str] = []
    for ch in raw:
        if ch.isalnum() or ch in ("-", "_", "."):
            out_chars.append(ch)
        else:
            out_chars.append("_")
    sanitized = "".join(out_chars).strip("._-")
    return sanitized or "step"


def output_line_count(stdout: str, stderr: str) -> int:
    return len(stdout.splitlines()) + len(stderr.splitlines())


def run_step(
    root: Path,
    name: str,
    cmd: list[str],
    quiet_success_logs: bool,
    compact_step_logs: bool,
    step_log_failed_only: bool,
    stdout_log_path: Path | None = None,
    stderr_log_path: Path | None = None,
) -> dict[str, object]:
    if stdout_log_path is not None:
        stdout_log_path.parent.mkdir(parents=True, exist_ok=True)
    if stderr_log_path is not None:
        stderr_log_path.parent.mkdir(parents=True, exist_ok=True)
    if compact_step_logs:
        print(f"[ci-gate] step={name} start")
    else:
        print(f"[ci-gate] step={name} cmd={' '.join(cmd)}")
    try:
        proc = subprocess.run(
            cmd,
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
    except OSError as exc:
        err_text = f"[ci-gate] step={name} launch_error={exc}\n"
        print(err_text.rstrip(), file=sys.stderr)
        if compact_step_logs:
            print(f"[ci-gate] step={name} cmd={' '.join(cmd)}")
        if stderr_log_path is not None:
            stderr_log_path.write_text(err_text, encoding="utf-8")
        print(f"[ci-gate] step={name} exit=127")
        return {
            "returncode": 127,
            "stdout_line_count": 0,
            "stderr_line_count": 1,
            "stdout_log_path": "",
            "stderr_log_path": str(stderr_log_path) if stderr_log_path is not None else "",
        }
    should_write_step_logs = not step_log_failed_only or proc.returncode != 0
    written_stdout_path = ""
    written_stderr_path = ""
    if should_write_step_logs and stdout_log_path is not None:
        stdout_log_path.write_text(proc.stdout or "", encoding="utf-8")
        written_stdout_path = str(stdout_log_path)
    if should_write_step_logs and stderr_log_path is not None:
        stderr_log_path.write_text(proc.stderr or "", encoding="utf-8")
        written_stderr_path = str(stderr_log_path)
    if compact_step_logs and proc.returncode != 0:
        print(f"[ci-gate] step={name} cmd={' '.join(cmd)}")
    if quiet_success_logs and proc.returncode == 0:
        line_count = output_line_count(proc.stdout or "", proc.stderr or "")
        if line_count > 0 and not compact_step_logs:
            print(f"[ci-gate] step={name} output_suppressed=1 line_count={line_count}")
    else:
        if proc.stdout:
            safe_print(proc.stdout, end="")
        if proc.stderr:
            safe_print(proc.stderr, end="", file=sys.stderr)
    print(f"[ci-gate] step={name} exit={proc.returncode}")
    return {
        "returncode": int(proc.returncode),
        "stdout_line_count": len((proc.stdout or "").splitlines()),
        "stderr_line_count": len((proc.stderr or "").splitlines()),
        "stdout_log_path": written_stdout_path,
        "stderr_log_path": written_stderr_path,
    }
