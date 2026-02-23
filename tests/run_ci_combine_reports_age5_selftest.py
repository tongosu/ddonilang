#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[ci-combine-age5-selftest] fail: {msg}")
    return 1


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def read_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def run_combine(
    *,
    seamgrim: Path,
    age3: Path,
    age4: Path,
    age5: Path,
    oi: Path,
    out: Path,
    require_age4: bool,
    require_age5: bool,
) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tools/scripts/combine_ci_reports.py",
        "--seamgrim-report",
        str(seamgrim),
        "--age3-report",
        str(age3),
        "--oi-report",
        str(oi),
        "--age4-report",
        str(age4),
        "--age5-report",
        str(age5),
        "--out",
        str(out),
        "--fail-on-bad",
        "--require-age3",
    ]
    if require_age4:
        cmd.append("--require-age4")
    if require_age5:
        cmd.append("--require-age5")
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="ci_combine_age5_selftest_") as tmp:
        root = Path(tmp)
        seamgrim_report = root / "seamgrim.json"
        age3_report = root / "age3.detjson"
        age4_report = root / "age4.detjson"
        age5_report = root / "age5.detjson"
        oi_report = root / "oi.detjson"
        out_report = root / "aggregate.detjson"

        write_json(
            seamgrim_report,
            {
                "schema": "ddn.seamgrim.ci_gate_report.v1",
                "ok": True,
                "steps": [],
                "failure_digest": [],
            },
        )
        write_json(
            age3_report,
            {
                "schema": "ddn.seamgrim.age3_close_report.v1",
                "overall_ok": True,
                "criteria": [],
                "failure_digest": [],
            },
        )
        write_json(
            age4_report,
            {
                "schema": "ddn.age4_close_report.v1",
                "overall_ok": True,
                "criteria": [],
                "failure_digest": [],
            },
        )
        write_json(
            oi_report,
            {
                "schema": "ddn.oi405_406.close_report.v1",
                "overall_ok": True,
                "packs": [],
                "failure_digest": [],
            },
        )

        # case 1: require-age5 + valid age5 report => pass
        write_json(
            age5_report,
            {
                "schema": "ddn.age5_close_report.v1",
                "overall_ok": True,
                "criteria": [],
                "failure_digest": [],
            },
        )
        proc_ok = run_combine(
            seamgrim=seamgrim_report,
            age3=age3_report,
            age4=age4_report,
            age5=age5_report,
            oi=oi_report,
            out=out_report,
            require_age4=True,
            require_age5=True,
        )
        if proc_ok.returncode != 0:
            return fail(f"require-age5 pass case failed: out={proc_ok.stdout} err={proc_ok.stderr}")
        ok_doc = read_json(out_report)
        if not isinstance(ok_doc, dict) or not bool(ok_doc.get("overall_ok", False)):
            return fail("require-age5 pass case aggregate overall_ok mismatch")
        age5_row = ok_doc.get("age5")
        if not isinstance(age5_row, dict) or not bool(age5_row.get("ok", False)):
            return fail("require-age5 pass case age5.ok mismatch")

        # case 2: require-age5 + missing age5 report => fail
        age5_report.unlink(missing_ok=True)
        proc_missing = run_combine(
            seamgrim=seamgrim_report,
            age3=age3_report,
            age4=age4_report,
            age5=age5_report,
            oi=oi_report,
            out=out_report,
            require_age4=True,
            require_age5=True,
        )
        if proc_missing.returncode == 0:
            return fail("require-age5 missing case must fail")
        miss_doc = read_json(out_report)
        if not isinstance(miss_doc, dict):
            return fail("require-age5 missing case report not generated")
        if bool(miss_doc.get("overall_ok", True)):
            return fail("require-age5 missing case overall_ok must be false")
        miss_age5 = miss_doc.get("age5")
        if not isinstance(miss_age5, dict) or bool(miss_age5.get("ok", True)):
            return fail("require-age5 missing case age5.ok must be false")

        # case 3: optional age5 + missing age5 report => pass (skipped)
        proc_optional = run_combine(
            seamgrim=seamgrim_report,
            age3=age3_report,
            age4=age4_report,
            age5=age5_report,
            oi=oi_report,
            out=out_report,
            require_age4=True,
            require_age5=False,
        )
        if proc_optional.returncode != 0:
            return fail(f"optional-age5 missing case failed: out={proc_optional.stdout} err={proc_optional.stderr}")
        optional_doc = read_json(out_report)
        if not isinstance(optional_doc, dict) or not bool(optional_doc.get("overall_ok", False)):
            return fail("optional-age5 missing case overall_ok mismatch")
        optional_age5 = optional_doc.get("age5")
        if not isinstance(optional_age5, dict):
            return fail("optional-age5 missing case age5 block missing")
        if not bool(optional_age5.get("ok", False)) or not bool(optional_age5.get("skipped", False)):
            return fail("optional-age5 missing case skipped semantics mismatch")

    print("[ci-combine-age5-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
