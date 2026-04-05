from __future__ import annotations

import io
import json
import sys
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path


def run_aggregate_gate_with_mock_failure(
    *,
    report_dir: Path,
    report_prefix: str,
    gate_args: list[str],
    fail_step_name: str,
    fail_returncode: int,
    observed_step_name: str,
    suppress_output: bool = True,
) -> dict[str, object]:
    import run_ci_aggregate_gate as gate

    observed_cmd: list[str] | None = None

    def fake_run_step(root, name, cmd, **kwargs):
        nonlocal observed_cmd
        if name == observed_step_name:
            observed_cmd = [str(part).strip() for part in cmd if str(part).strip()]
        rc = int(fail_returncode) if name == fail_step_name else 0
        return {
            "returncode": rc,
            "stdout_line_count": 0,
            "stderr_line_count": 0,
            "stdout_log_path": "",
            "stderr_log_path": "",
        }

    original_run_step = gate.run_step
    original_argv = list(sys.argv)
    captured_stdout = ""
    captured_stderr = ""
    try:
        gate.run_step = fake_run_step
        sys.argv = ["run_ci_aggregate_gate.py", *gate_args]
        if suppress_output:
            stdout_buffer = io.StringIO()
            stderr_buffer = io.StringIO()
            with redirect_stdout(stdout_buffer), redirect_stderr(stderr_buffer):
                run_rc = int(gate.main())
            captured_stdout = stdout_buffer.getvalue()
            captured_stderr = stderr_buffer.getvalue()
        else:
            run_rc = int(gate.main())
    finally:
        gate.run_step = original_run_step
        sys.argv = original_argv

    index_path = report_dir / f"{report_prefix}.ci_gate_report_index.detjson"
    index_doc: dict[str, object] = {}
    if index_path.exists():
        try:
            loaded = json.loads(index_path.read_text(encoding="utf-8"))
            if isinstance(loaded, dict):
                index_doc = loaded
        except json.JSONDecodeError:
            index_doc = {}

    return {
        "returncode": run_rc,
        "observed_cmd": observed_cmd or [],
        "index_path": index_path,
        "index_doc": index_doc,
        "captured_stdout": captured_stdout,
        "captured_stderr": captured_stderr,
    }


def build_step_map(index_doc: dict[str, object]) -> dict[str, dict[str, object]]:
    rows = index_doc.get("steps")
    if not isinstance(rows, list):
        return {}
    step_map: dict[str, dict[str, object]] = {}
    for row in rows:
        if not isinstance(row, dict):
            continue
        name = str(row.get("name", "")).strip()
        if not name:
            continue
        step_map[name] = row
    return step_map
