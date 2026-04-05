#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[ci-emit-artifacts-sanity-contract-check-selftest] fail: {msg}")
    return 1


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    checker = root / "tests" / "run_ci_emit_artifacts_sanity_contract_check.py"
    emit_check = root / "tests" / "run_ci_emit_artifacts_check.py"
    emit_selftest = root / "tests" / "run_ci_emit_artifacts_check_selftest.py"
    code_map = root / "tests" / "ci_check_error_codes.py"

    proc_ok = run([sys.executable, str(checker)], root)
    if proc_ok.returncode != 0:
        return fail(f"default pass case failed: err={proc_ok.stderr}")

    emit_check_text = emit_check.read_text(encoding="utf-8")
    emit_selftest_text = emit_selftest.read_text(encoding="utf-8")
    code_map_text = code_map.read_text(encoding="utf-8")

    with tempfile.TemporaryDirectory(prefix="ci_emit_artifacts_sanity_contract_selftest_") as td:
        tmp = Path(td)
        emit_check_copy = tmp / "run_ci_emit_artifacts_check.py"
        emit_selftest_copy = tmp / "run_ci_emit_artifacts_check_selftest.py"
        code_map_copy = tmp / "ci_check_error_codes.py"

        write_text(emit_check_copy, emit_check_text)
        write_text(code_map_copy, code_map_text)

        mutated_selftest = emit_selftest_text.replace(
            "fail code={CODES['REPORT_DIR_MISSING']}",
            "fail code={CODES['REPORT_DIR_MISSING_BROKEN']}",
            1,
        )
        if mutated_selftest == emit_selftest_text:
            return fail("selftest mutation anchor missing")
        write_text(emit_selftest_copy, mutated_selftest)

        proc_missing_selftest_code = run(
            [
                sys.executable,
                str(checker),
                "--emit-check",
                str(emit_check_copy),
                "--emit-selftest",
                str(emit_selftest_copy),
                "--code-map",
                str(code_map_copy),
            ],
            root,
        )
        merged_missing_selftest_code = (
            (proc_missing_selftest_code.stdout or "") + "\n" + (proc_missing_selftest_code.stderr or "")
        )
        if proc_missing_selftest_code.returncode == 0:
            return fail("missing selftest fail-code case must fail")
        if "emit_selftest:fail_code_missing:REPORT_DIR_MISSING" not in merged_missing_selftest_code:
            return fail(f"missing selftest fail-code token not found: out={merged_missing_selftest_code}")

        mutated_code_map = code_map_text.replace(
            '    "REPORT_DIR_MISSING": "E_REPORT_DIR_MISSING",\n',
            "",
            1,
        )
        if mutated_code_map == code_map_text:
            return fail("code-map mutation anchor missing")
        write_text(code_map_copy, mutated_code_map)
        write_text(emit_selftest_copy, emit_selftest_text)

        proc_missing_map_code = run(
            [
                sys.executable,
                str(checker),
                "--emit-check",
                str(emit_check_copy),
                "--emit-selftest",
                str(emit_selftest_copy),
                "--code-map",
                str(code_map_copy),
            ],
            root,
        )
        merged_missing_map_code = (proc_missing_map_code.stdout or "") + "\n" + (proc_missing_map_code.stderr or "")
        if proc_missing_map_code.returncode == 0:
            return fail("missing code-map code case must fail")
        if "code_map:emit_code_missing:REPORT_DIR_MISSING" not in merged_missing_map_code:
            return fail(f"missing code-map token not found: out={merged_missing_map_code}")

    print("[ci-emit-artifacts-sanity-contract-check-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
