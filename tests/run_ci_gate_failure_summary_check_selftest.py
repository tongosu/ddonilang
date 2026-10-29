#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from ci_check_error_codes import FAILURE_SUMMARY_CODES as CODES

def fail(msg: str) -> int:
    print(f"[ci-gate-failure-summary-selftest] fail: {msg}")
    return 1


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.rstrip() + "\n", encoding="utf-8")


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def run_check(summary: Path, index: Path, require_pass: bool = False) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_ci_gate_failure_summary_check.py",
        "--summary",
        str(summary),
        "--index",
        str(index),
    ]
    if require_pass:
        cmd.append("--require-pass")
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def make_index(path: Path, step_log_dir: Path, steps: list[dict]) -> None:
    write_json(
        path,
        {
            "schema": "ddn.ci.aggregate_gate.index.v1",
            "step_log_dir": str(step_log_dir),
            "steps": steps,
        },
    )


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="ci_gate_failure_summary_selftest_") as tmp:
        root = Path(tmp)
        logs = root / "logs"
        logs.mkdir(parents=True, exist_ok=True)
        pass_summary = root / "pass.summary.txt"
        pass_index = root / "pass.index.detjson"
        write_text(
            pass_summary,
            "\n".join(
                [
                    "[ci-gate-summary] PASS",
                    "[ci-gate-summary] failed_steps=(none)",
                ]
            ),
        )
        make_index(pass_index, logs, [{"name": "sample", "returncode": 0}])
        proc_pass = run_check(pass_summary, pass_index, require_pass=True)
        if proc_pass.returncode != 0:
            return fail(f"pass case failed: out={proc_pass.stdout} err={proc_pass.stderr}")

        fail_stdout = logs / "fail.stdout.txt"
        fail_stderr = logs / "fail.stderr.txt"
        write_text(fail_stdout, "stdout")
        write_text(fail_stderr, "stderr")
        fail_summary = root / "fail.summary.txt"
        fail_index = root / "fail.index.detjson"
        write_text(
            fail_summary,
            "\n".join(
                [
                    "[ci-gate-summary] FAIL",
                    "[ci-gate-summary] failed_steps=sample_fail",
                    "[ci-gate-summary] failed_step_detail=sample_fail rc=1 cmd=python sample.py",
                    f"[ci-gate-summary] failed_step_logs=sample_fail stdout={fail_stdout} stderr={fail_stderr}",
                ]
            ),
        )
        make_index(
            fail_index,
            logs,
            [
                {"name": "sample_fail", "returncode": 1},
            ],
        )
        proc_fail_ok = run_check(fail_summary, fail_index)
        if proc_fail_ok.returncode != 0:
            return fail(f"valid fail case failed: out={proc_fail_ok.stdout} err={proc_fail_ok.stderr}")

        dup_summary = root / "dup.summary.txt"
        write_text(
            dup_summary,
            "\n".join(
                [
                    "[ci-gate-summary] FAIL",
                    "[ci-gate-summary] failed_steps=sample_fail,sample_fail",
                    "[ci-gate-summary] failed_step_detail=sample_fail rc=1 cmd=python sample.py",
                    f"[ci-gate-summary] failed_step_logs=sample_fail stdout={fail_stdout} stderr={fail_stderr}",
                ]
            ),
        )
        proc_dup = run_check(dup_summary, fail_index)
        if proc_dup.returncode == 0:
            return fail("duplicate failed_steps must fail")
        if f"fail code={CODES['FAIL_FAILED_STEPS_DUPLICATE']}" not in proc_dup.stderr:
            return fail(f"duplicate case error code missing: err={proc_dup.stderr}")

        miss_detail_summary = root / "miss_detail.summary.txt"
        write_text(
            miss_detail_summary,
            "\n".join(
                [
                    "[ci-gate-summary] FAIL",
                    "[ci-gate-summary] failed_steps=sample_fail",
                    f"[ci-gate-summary] failed_step_logs=sample_fail stdout={fail_stdout} stderr={fail_stderr}",
                ]
            ),
        )
        proc_miss_detail = run_check(miss_detail_summary, fail_index)
        if proc_miss_detail.returncode == 0:
            return fail("missing detail row must fail")

        miss_logs_summary = root / "miss_logs.summary.txt"
        write_text(
            miss_logs_summary,
            "\n".join(
                [
                    "[ci-gate-summary] FAIL",
                    "[ci-gate-summary] failed_steps=sample_fail",
                    "[ci-gate-summary] failed_step_detail=sample_fail rc=1 cmd=python sample.py",
                ]
            ),
        )
        proc_miss_logs = run_check(miss_logs_summary, fail_index)
        if proc_miss_logs.returncode == 0:
            return fail("missing logs row with step_log_dir must fail")

        bad_format_summary = root / "bad_format.summary.txt"
        write_text(
            bad_format_summary,
            "\n".join(
                [
                    "[ci-gate-summary] FAIL",
                    "[ci-gate-summary] failed_steps=sample_fail",
                    "[ci-gate-summary] failed_step_detail=sample_fail rc=not_int cmd=python sample.py",
                    f"[ci-gate-summary] failed_step_logs=sample_fail stdout={fail_stdout} stderr={fail_stderr}",
                ]
            ),
        )
        proc_bad_format = run_check(bad_format_summary, fail_index)
        if proc_bad_format.returncode == 0:
            return fail("bad detail format must fail")
        if f"fail code={CODES['DETAIL_FORMAT_INVALID']}" not in proc_bad_format.stderr:
            return fail(f"bad format case error code missing: err={proc_bad_format.stderr}")

    print("[ci-gate-failure-summary-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
