#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str, proc: subprocess.CompletedProcess[str] | None = None) -> int:
    print(f"[age2-close-selftest] fail: {msg}")
    if proc is not None:
        if proc.stdout:
            print(proc.stdout, end="")
        if proc.stderr:
            print(proc.stderr, end="", file=sys.stderr)
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


def load_json(path: Path) -> dict:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError(f"json root must be object: {path}")
    return data


def criteria_map(criteria: object) -> dict[str, bool]:
    out: dict[str, bool] = {}
    if not isinstance(criteria, list):
        return out
    for row in criteria:
        if not isinstance(row, dict):
            continue
        name = str(row.get("name", "")).strip()
        if not name:
            continue
        out[name] = bool(row.get("ok", False))
    return out


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    py = sys.executable

    with tempfile.TemporaryDirectory(prefix="age2_close_selftest_") as tmp:
        report_dir = Path(tmp)
        age2_report = report_dir / "age2_completion_gate.detjson"
        must_report = report_dir / "age2_completion_must_pack_report.detjson"
        should_report = report_dir / "age2_completion_should_pack_report.detjson"
        close_report = report_dir / "age2_close_report.detjson"

        # case 1: positive flow
        proc_ok = run(
            [
                py,
                "tests/run_age2_close.py",
                "--run-age2",
                "--age2-report",
                str(age2_report),
                "--must-report",
                str(must_report),
                "--should-report",
                str(should_report),
                "--report-out",
                str(close_report),
            ],
            root,
        )
        if proc_ok.returncode != 0:
            return fail("positive flow must pass", proc_ok)
        if not close_report.exists():
            return fail(f"close report missing: {close_report}")

        close_doc = load_json(close_report)
        if str(close_doc.get("schema", "")) != "ddn.age2_close_report.v1":
            return fail(f"schema mismatch: {close_doc.get('schema')}")
        if not bool(close_doc.get("overall_ok", False)):
            return fail("overall_ok must be true")
        required = [
            "age2_completion_gate_schema_ok",
            "age2_completion_gate_overall_ok",
            "age2_ssot_pack_contract_sync_ok",
            "age2_must_pack_set_pass",
            "age2_should_pack_set_pass",
            "age2_strict_should_gate_pass",
            "age2_must_report_exists",
            "age2_should_report_exists_when_enabled",
            "age2_completion_gate_selftest_pass",
        ]
        ok_map = criteria_map(close_doc.get("criteria"))
        missing = [name for name in required if name not in ok_map]
        if missing:
            return fail(f"missing criteria: {missing}")
        failed = [name for name in required if not ok_map.get(name, False)]
        if failed:
            return fail(f"criteria must pass: {failed}")

        for key in (
            "age2_completion_gate_report_path",
            "age2_completion_must_report_path",
            "age2_completion_should_report_path",
        ):
            path_text = str(close_doc.get(key, "")).strip()
            if not path_text:
                return fail(f"{key} missing")
            if not Path(path_text).exists():
                return fail(f"{key} path not found: {path_text}")

        # case 2: negative flow with bad precomputed completion report
        bad_age2_report = report_dir / "bad_age2_completion_gate.detjson"
        bad_close_report = report_dir / "bad_age2_close_report.detjson"
        bad_must_report = report_dir / "bad_age2_must.detjson"
        bad_should_report = report_dir / "bad_age2_should.detjson"
        bad_age2_doc = {
            "schema": "ddn.age2.completion_gate.v1",
            "overall_ok": False,
            "strict_should": True,
            "run_should": True,
            "criteria": [
                {"name": "age2_ssot_pack_contract_sync", "ok": False},
                {"name": "must_pack_set_pass", "ok": False},
                {"name": "should_pack_set_pass", "ok": False},
                {"name": "strict_should_gate", "ok": False},
            ],
            "failure_digest": ["synthetic failure"],
            "failure_codes": ["E_SYNTHETIC_SELFTEST"],
        }
        bad_age2_report.write_text(
            json.dumps(bad_age2_doc, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )

        proc_bad = run(
            [
                py,
                "tests/run_age2_close.py",
                "--age2-report",
                str(bad_age2_report),
                "--must-report",
                str(bad_must_report),
                "--should-report",
                str(bad_should_report),
                "--report-out",
                str(bad_close_report),
                "--skip-selftest",
            ],
            root,
        )
        if proc_bad.returncode == 0:
            return fail("negative flow must fail", proc_bad)
        if not bad_close_report.exists():
            return fail(f"bad close report missing: {bad_close_report}")
        bad_close_doc = load_json(bad_close_report)
        if str(bad_close_doc.get("schema", "")) != "ddn.age2_close_report.v1":
            return fail("bad flow schema mismatch")
        if bool(bad_close_doc.get("overall_ok", True)):
            return fail("bad flow overall_ok must be false")

        bad_ok_map = criteria_map(bad_close_doc.get("criteria"))
        if bad_ok_map.get("age2_completion_gate_overall_ok", True):
            return fail("bad flow must fail age2_completion_gate_overall_ok criterion")
        if bad_ok_map.get("age2_must_report_exists", True):
            return fail("bad flow must fail age2_must_report_exists criterion")
        failure_digest = bad_close_doc.get("failure_digest")
        if not isinstance(failure_digest, list) or not failure_digest:
            return fail("bad flow failure_digest must be non-empty list")

    print("[age2-close-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
