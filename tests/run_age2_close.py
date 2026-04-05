#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REUSE_REPORT_ENV_KEY = "DDN_AGE2_COMPLETION_GATE_SELFTEST_REUSE_REPORT"
REPORT_PATH_ENV_KEY = "DDN_AGE2_COMPLETION_GATE_REPORT_JSON"
MUST_REPORT_PATH_ENV_KEY = "DDN_AGE2_COMPLETION_GATE_MUST_REPORT_JSON"
SHOULD_REPORT_PATH_ENV_KEY = "DDN_AGE2_COMPLETION_GATE_SHOULD_REPORT_JSON"


def clip(text: str, limit: int = 140) -> str:
    normalized = " ".join(text.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def default_report_path(file_name: str) -> str:
    candidates = [
        Path("I:/home/urihanl/ddn/codex/build/reports"),
        Path("C:/ddn/codex/build/reports"),
        Path("build/reports"),
    ]
    for base in candidates:
        try:
            base.mkdir(parents=True, exist_ok=True)
            return str(base / file_name)
        except OSError:
            continue
    return str(Path("build/reports") / file_name)


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def run_age2_gate(root: Path, report: Path, must_report: Path, should_report: Path) -> int:
    cmd = [
        sys.executable,
        "tests/run_age2_completion_gate.py",
        "--report-out",
        str(report),
        "--must-report-out",
        str(must_report),
        "--should-report-out",
        str(should_report),
    ]
    print(f"[age2-close] run: {' '.join(cmd)}")
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.stdout:
        print(proc.stdout, end="")
    if proc.stderr:
        print(proc.stderr, end="", file=sys.stderr)
    print(f"[age2-close] age2_gate_exit={proc.returncode}")
    return int(proc.returncode)


def run_age2_selftest(root: Path, report: Path, must_report: Path, should_report: Path) -> tuple[int, str]:
    cmd = [sys.executable, "tests/run_age2_completion_gate_selftest.py"]
    env = dict(os.environ)
    env[REUSE_REPORT_ENV_KEY] = "1"
    env[REPORT_PATH_ENV_KEY] = str(report)
    env[MUST_REPORT_PATH_ENV_KEY] = str(must_report)
    env[SHOULD_REPORT_PATH_ENV_KEY] = str(should_report)
    proc = subprocess.run(
        cmd,
        cwd=root,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.stdout:
        print(proc.stdout, end="")
    if proc.stderr:
        print(proc.stderr, end="", file=sys.stderr)
    digest = clip((proc.stdout or "") + " " + (proc.stderr or ""))
    return int(proc.returncode), digest


def find_criteria_ok(doc: dict | None, name: str) -> bool:
    if not isinstance(doc, dict):
        return False
    rows = doc.get("criteria")
    if not isinstance(rows, list):
        return False
    for row in rows:
        if not isinstance(row, dict):
            continue
        if str(row.get("name", "")).strip() == name:
            return bool(row.get("ok", False))
    return False


def build_report(
    gate_doc: dict | None,
    gate_report_path: Path,
    must_report_path: Path,
    should_report_path: Path,
    selftest_rc: int | None,
    selftest_digest: str,
) -> dict:
    schema_ok = isinstance(gate_doc, dict) and str(gate_doc.get("schema", "")) == "ddn.age2.completion_gate.v1"
    overall_ok = bool(gate_doc.get("overall_ok", False)) if isinstance(gate_doc, dict) else False
    run_should = bool(gate_doc.get("run_should", True)) if isinstance(gate_doc, dict) else True
    strict_should = bool(gate_doc.get("strict_should", True)) if isinstance(gate_doc, dict) else True

    ssot_ok = find_criteria_ok(gate_doc, "age2_ssot_pack_contract_sync")
    must_ok = find_criteria_ok(gate_doc, "must_pack_set_pass")
    should_ok = find_criteria_ok(gate_doc, "should_pack_set_pass") if run_should else True
    strict_should_ok = find_criteria_ok(gate_doc, "strict_should_gate") if strict_should else True

    must_report_exists = must_report_path.exists()
    should_report_exists = should_report_path.exists() if run_should else True

    selftest_enabled = selftest_rc is not None
    selftest_ok = (selftest_rc == 0) if selftest_enabled else True

    criteria = [
        {
            "name": "age2_completion_gate_schema_ok",
            "ok": schema_ok,
            "detail": f"schema={str(gate_doc.get('schema', '-')) if isinstance(gate_doc, dict) else '-'}",
        },
        {
            "name": "age2_completion_gate_overall_ok",
            "ok": overall_ok,
            "detail": f"overall_ok={int(overall_ok)}",
        },
        {
            "name": "age2_ssot_pack_contract_sync_ok",
            "ok": ssot_ok,
            "detail": f"criteria:age2_ssot_pack_contract_sync={int(ssot_ok)}",
        },
        {
            "name": "age2_must_pack_set_pass",
            "ok": must_ok,
            "detail": f"criteria:must_pack_set_pass={int(must_ok)}",
        },
        {
            "name": "age2_should_pack_set_pass",
            "ok": should_ok,
            "detail": f"run_should={int(run_should)} criteria:should_pack_set_pass={int(should_ok)}",
        },
        {
            "name": "age2_strict_should_gate_pass",
            "ok": strict_should_ok,
            "detail": f"strict_should={int(strict_should)} criteria:strict_should_gate={int(strict_should_ok)}",
        },
        {
            "name": "age2_must_report_exists",
            "ok": must_report_exists,
            "detail": f"path={must_report_path}",
        },
        {
            "name": "age2_should_report_exists_when_enabled",
            "ok": should_report_exists,
            "detail": f"run_should={int(run_should)} path={should_report_path}",
        },
        {
            "name": "age2_completion_gate_selftest_pass",
            "ok": selftest_ok,
            "detail": (
                f"enabled={int(selftest_enabled)} rc={selftest_rc if selftest_rc is not None else '-'} "
                f"digest={selftest_digest or '-'}"
            ),
        },
    ]

    close_ok = all(bool(row.get("ok", False)) for row in criteria)
    failure_digest: list[str] = []
    for row in criteria:
        if bool(row.get("ok", False)):
            continue
        failure_digest.append(f"{row.get('name')}: {clip(str(row.get('detail', '-')))}")

    gate_failure_digest = gate_doc.get("failure_digest") if isinstance(gate_doc, dict) else None
    if isinstance(gate_failure_digest, list):
        for line in gate_failure_digest[:4]:
            failure_digest.append(f"age2_completion: {clip(str(line))}")

    gate_failure_codes = gate_doc.get("failure_codes") if isinstance(gate_doc, dict) else None
    failure_codes: list[str] = []
    if isinstance(gate_failure_codes, list):
        for raw in gate_failure_codes:
            code = str(raw).strip()
            if not code or code in failure_codes:
                continue
            failure_codes.append(code)
            if len(failure_codes) >= 16:
                break

    return {
        "schema": "ddn.age2_close_report.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "overall_ok": close_ok,
        "criteria": criteria,
        "age2_completion_gate_report_path": str(gate_report_path),
        "age2_completion_must_report_path": str(must_report_path),
        "age2_completion_should_report_path": str(should_report_path),
        "failure_digest": failure_digest[:16],
        "failure_codes": failure_codes,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Evaluate AGE2 close criteria from age2 completion gate report")
    parser.add_argument(
        "--age2-report",
        default=default_report_path("age2_completion_gate.detjson"),
        help="path to ddn.age2.completion_gate.v1 report",
    )
    parser.add_argument(
        "--must-report",
        default=default_report_path("age2_completion_must_pack_report.detjson"),
        help="path to ddn.pack.golden.report.v1 MUST report",
    )
    parser.add_argument(
        "--should-report",
        default=default_report_path("age2_completion_should_pack_report.detjson"),
        help="path to ddn.pack.golden.report.v1 SHOULD report",
    )
    parser.add_argument(
        "--report-out",
        default=default_report_path("age2_close_report.detjson"),
        help="output age2 close report path",
    )
    parser.add_argument(
        "--run-age2",
        action="store_true",
        help="run age2 completion gate before evaluating close criteria",
    )
    parser.add_argument(
        "--skip-selftest",
        action="store_true",
        help="skip age2 completion gate selftest execution",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    age2_report = Path(args.age2_report)
    must_report = Path(args.must_report)
    should_report = Path(args.should_report)
    report_out = Path(args.report_out)

    need_run_age2 = bool(args.run_age2) or (not age2_report.exists())
    if need_run_age2:
        rc = run_age2_gate(root, age2_report, must_report, should_report)
        if rc != 0:
            print("[age2-close] age2 completion gate returned non-zero", file=sys.stderr)

    selftest_rc: int | None = None
    selftest_digest = ""
    if not bool(args.skip_selftest):
        selftest_rc, selftest_digest = run_age2_selftest(root, age2_report, must_report, should_report)
        print(f"[age2-close] selftest_exit={selftest_rc}")

    gate_doc = load_json(age2_report)
    report = build_report(gate_doc, age2_report, must_report, should_report, selftest_rc, selftest_digest)
    report_out.parent.mkdir(parents=True, exist_ok=True)
    report_out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    criteria = report.get("criteria", [])
    failed_count = sum(1 for row in criteria if isinstance(row, dict) and not bool(row.get("ok", False)))
    print(
        f"[age2-close] overall_ok={int(bool(report.get('overall_ok', False)))} "
        f"criteria={len(criteria)} failed={failed_count} report={report_out}"
    )
    for row in criteria:
        if not isinstance(row, dict):
            continue
        print(f" - {row.get('name')}: ok={int(bool(row.get('ok', False)))}")
    if not bool(report.get("overall_ok", False)):
        digest = report.get("failure_digest")
        if isinstance(digest, list):
            for line in digest[:8]:
                print(f"   {line}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
