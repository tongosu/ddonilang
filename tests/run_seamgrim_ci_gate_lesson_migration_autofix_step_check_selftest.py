#!/usr/bin/env python
from __future__ import annotations

import argparse
import importlib.util
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from types import ModuleType


CHECK_SCRIPT = Path(__file__).resolve().parent / "run_seamgrim_ci_gate_lesson_migration_autofix_step_check.py"
CHECK_REPORT_SCHEMA = "ddn.seamgrim_ci_gate_lesson_migration_autofix_step_check.v1"
EXPECTED_FILE_COUNT = 3
MISSING_TOKEN_REL_PATH = "tests/run_seamgrim_ci_gate.py"
MISSING_TOKEN_VALUE = '"seamgrim_ci_gate_lesson_migration_autofix_step_check_selftest"'


def fail(msg: str) -> int:
    print(f"[seamgrim-ci-gate-lesson-migration-autofix-step-check-selftest] fail {msg}", file=sys.stderr)
    return 1


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def load_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Selftest for lesson migration autofix step contract checker")
    parser.add_argument(
        "--verify-report",
        help="optional real report path emitted by run_seamgrim_ci_gate_lesson_migration_autofix_step_check.py",
    )
    return parser.parse_args()


def load_check_module() -> ModuleType:
    spec = importlib.util.spec_from_file_location("seamgrim_lesson_migration_autofix_step_check_mod", CHECK_SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("failed to create module spec")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def load_requirements() -> dict[str, tuple[str, ...]]:
    module = load_check_module()
    value = getattr(module, "FILE_TOKEN_REQUIREMENTS", None)
    if not isinstance(value, dict):
        raise RuntimeError("FILE_TOKEN_REQUIREMENTS must be a dict")
    requirements: dict[str, tuple[str, ...]] = {}
    for key, tokens in value.items():
        if not isinstance(key, str):
            raise RuntimeError("requirement key must be str")
        if not isinstance(tokens, tuple) or not all(isinstance(token, str) for token in tokens):
            raise RuntimeError(f"requirement tokens must be tuple[str]: {key}")
        requirements[key] = tokens
    return requirements


def write_ok_tree(root: Path, requirements: dict[str, tuple[str, ...]]) -> None:
    for rel_path, tokens in sorted(requirements.items()):
        write_text(root / rel_path, "\n".join(tokens) + "\n")


def run_check(repo_root: Path, *, report_out: Path | None = None) -> subprocess.CompletedProcess[str]:
    cmd = [sys.executable, str(CHECK_SCRIPT), "--repo-root", str(repo_root)]
    if report_out is not None:
        cmd.extend(["--report-out", str(report_out)])
    return subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def validate_real_report(path: Path, minimum_checked_files: int) -> str | None:
    if not path.exists():
        return f"verify-report path does not exist: {path}"
    try:
        report = load_json(path)
    except Exception as exc:
        return f"verify-report json parse failed path={path} err={exc}"
    schema = str(report.get("schema", "")).strip()
    if schema != CHECK_REPORT_SCHEMA:
        return f"verify-report schema mismatch expected={CHECK_REPORT_SCHEMA} actual={schema}"
    if str(report.get("status", "")).strip() != "pass":
        return f"verify-report status must be pass doc={report}"
    if bool(report.get("ok", False)) is not True:
        return f"verify-report ok must be true doc={report}"
    if str(report.get("code", "")).strip() != "OK":
        return f"verify-report code must be OK doc={report}"
    try:
        checked_files = int(report.get("checked_files", -1))
    except Exception:
        checked_files = -1
    if checked_files < minimum_checked_files:
        return f"verify-report checked_files too small checked_files={checked_files} minimum={minimum_checked_files}"
    try:
        missing_count = int(report.get("missing_count", -1))
    except Exception:
        missing_count = -1
    if missing_count != 0:
        return f"verify-report missing_count must be 0 doc={report}"
    missing_items = report.get("missing")
    if not isinstance(missing_items, list) or missing_items:
        return f"verify-report missing must be [] doc={report}"
    return None


def main() -> int:
    args = parse_args()
    with tempfile.TemporaryDirectory(prefix="seamgrim_lesson_migration_autofix_step_selftest_") as tmp:
        root = Path(tmp)
        requirements = load_requirements()
        if len(requirements) != EXPECTED_FILE_COUNT:
            return fail(f"requirement count mismatch count={len(requirements)} expected={EXPECTED_FILE_COUNT}")
        write_ok_tree(root, requirements)

        ok_report = root / "reports" / "ok.detjson"
        ok_proc = run_check(root, report_out=ok_report)
        if ok_proc.returncode != 0:
            return fail(f"ok case failed rc={ok_proc.returncode} out={ok_proc.stdout} err={ok_proc.stderr}")
        if f"checked_files={len(requirements)}" not in ok_proc.stdout:
            return fail(f"ok case missing checked_files marker out={ok_proc.stdout}")
        ok_doc = load_json(ok_report)
        if ok_doc.get("status") != "pass":
            return fail(f"ok report status must be pass doc={ok_doc}")
        if int(ok_doc.get("checked_files", -1)) != len(requirements):
            return fail(f"ok report checked_files mismatch doc={ok_doc}")
        if int(ok_doc.get("missing_count", -1)) != 0:
            return fail(f"ok report missing_count must be 0 doc={ok_doc}")

        if MISSING_TOKEN_REL_PATH not in requirements:
            return fail(f"missing-token target not found in requirements: {MISSING_TOKEN_REL_PATH}")
        if MISSING_TOKEN_VALUE not in requirements[MISSING_TOKEN_REL_PATH]:
            return fail(
                f"missing-token value not found in requirement target: {MISSING_TOKEN_REL_PATH} token={MISSING_TOKEN_VALUE}"
            )

        write_text(
            root / MISSING_TOKEN_REL_PATH,
            "\n".join(token for token in requirements[MISSING_TOKEN_REL_PATH] if token != MISSING_TOKEN_VALUE) + "\n",
        )
        miss_token_report = root / "reports" / "missing_token.detjson"
        miss_token_proc = run_check(root, report_out=miss_token_report)
        if miss_token_proc.returncode == 0:
            return fail("missing token case must fail")
        if f"{MISSING_TOKEN_REL_PATH}::token_missing::{MISSING_TOKEN_VALUE}" not in miss_token_proc.stdout:
            return fail(f"missing token marker not found out={miss_token_proc.stdout} err={miss_token_proc.stderr}")
        miss_token_doc = load_json(miss_token_report)
        if miss_token_doc.get("status") != "fail":
            return fail(f"missing token report status must be fail doc={miss_token_doc}")
        if int(miss_token_doc.get("missing_count", 0)) < 1:
            return fail(f"missing token report missing_count must be >=1 doc={miss_token_doc}")

    if args.verify_report:
        verify_error = validate_real_report(Path(args.verify_report), minimum_checked_files=EXPECTED_FILE_COUNT)
        if verify_error is not None:
            return fail(verify_error)

    print("[seamgrim-ci-gate-lesson-migration-autofix-step-check-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
