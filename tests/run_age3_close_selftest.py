#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str, proc: subprocess.CompletedProcess[str] | None = None) -> int:
    print(f"[age3-close-selftest] fail: {msg}")
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


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


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


def make_seamgrim_doc(step_ok: bool = True, *, mode: str = "legacy") -> dict:
    if mode == "release":
        required_steps = [
            "schema_realign_formula_compat",
            "schema_upgrade_formula_compat",
            "formula_compat",
            "ui_age3_gate",
            "space2d_source_ui_gate",
            "export_graph_preprocess",
            "frontdoor_strict_all",
            "ci_gate_diagnostics",
        ]
        fail_target = "schema_realign_formula_compat"
    else:
        required_steps = [
            "schema_gate",
            "ui_age3_gate",
            "space2d_source_ui_gate",
            "export_graph_preprocess",
            "full_check",
            "ci_gate_diagnostics",
        ]
        fail_target = "schema_gate"
    rows = [{"name": name, "ok": True} for name in required_steps]
    if not step_ok:
        for row in rows:
            if str(row.get("name", "")) == fail_target:
                row["ok"] = False
                break
    return {
        "schema": "ddn.seamgrim.ci_gate.v1",
        "ok": step_ok,
        "require_promoted": True,
        "strict_graph": True,
        "steps": rows,
        "failure_digest": [] if step_ok else ["schema gate failed"],
    }


def make_ui_doc(ok: bool = True) -> dict:
    return {
        "schema": "ddn.seamgrim.ui_age3_gate.v1",
        "ok": ok,
        "checks": [{"name": "bundle", "ok": ok}],
        "failure_digest": [] if ok else ["ui gate failed"],
    }


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    py = sys.executable

    with tempfile.TemporaryDirectory(prefix="age3_close_selftest_") as tmp:
        report_dir = Path(tmp)
        seamgrim_report = report_dir / "seamgrim_ci_gate_report.detjson"
        ui_report = report_dir / "seamgrim_ui_age3_gate_report.detjson"
        close_report = report_dir / "age3_close_report.detjson"

        # case 1: positive flow (legacy surface)
        write_json(seamgrim_report, make_seamgrim_doc(step_ok=True, mode="legacy"))
        write_json(ui_report, make_ui_doc(ok=True))
        proc_ok = run(
            [
                py,
                "tests/run_age3_close.py",
                "--seamgrim-report",
                str(seamgrim_report),
                "--ui-age3-report",
                str(ui_report),
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
        if str(close_doc.get("schema", "")) != "ddn.seamgrim.age3_close_report.v1":
            return fail(f"schema mismatch: {close_doc.get('schema')}")
        if not bool(close_doc.get("overall_ok", False)):
            return fail("overall_ok must be true")
        required_true = [
            "seamgrim_gate_ok",
            "require_promoted_on",
            "strict_graph_on",
            "schema_gate_ok",
            "ui_age3_gate_step_ok",
            "space2d_source_ui_gate_ok",
            "ui_age3_report_ok",
            "export_graph_preprocess_ok",
            "full_check_ok",
            "ci_gate_diagnostics_ok",
        ]
        ok_map = criteria_map(close_doc.get("criteria"))
        missing = [name for name in required_true if name not in ok_map]
        if missing:
            return fail(f"missing criteria: {missing}")
        failed = [name for name in required_true if not ok_map.get(name, False)]
        if failed:
            return fail(f"criteria must pass: {failed}")

        # case 1a: positive flow (release surface)
        write_json(seamgrim_report, make_seamgrim_doc(step_ok=True, mode="release"))
        write_json(ui_report, make_ui_doc(ok=True))
        proc_release_ok = run(
            [
                py,
                "tests/run_age3_close.py",
                "--seamgrim-report",
                str(seamgrim_report),
                "--ui-age3-report",
                str(ui_report),
                "--report-out",
                str(close_report),
            ],
            root,
        )
        if proc_release_ok.returncode != 0:
            return fail("release surface positive flow must pass", proc_release_ok)
        release_doc = load_json(close_report)
        if not bool(release_doc.get("overall_ok", False)):
            return fail("release surface overall_ok must be true")
        release_map = criteria_map(release_doc.get("criteria"))
        failed_release = [name for name in required_true if not release_map.get(name, False)]
        if failed_release:
            return fail(f"release surface criteria must pass: {failed_release}")

        # case 1b: alias flags are exposed on CLI help
        proc_help = run(
            [
                py,
                "tests/run_age3_close.py",
                "--help",
            ],
            root,
        )
        if proc_help.returncode != 0:
            return fail("help flow must pass", proc_help)
        if "--run-age3" not in proc_help.stdout or "--run-seamgrim-ci-gate" not in proc_help.stdout:
            return fail("alias flags missing from help output")

        # case 2: negative flow
        write_json(seamgrim_report, make_seamgrim_doc(step_ok=False, mode="release"))
        write_json(ui_report, make_ui_doc(ok=True))
        proc_bad = run(
            [
                py,
                "tests/run_age3_close.py",
                "--seamgrim-report",
                str(seamgrim_report),
                "--ui-age3-report",
                str(ui_report),
                "--report-out",
                str(close_report),
            ],
            root,
        )
        if proc_bad.returncode == 0:
            return fail("negative flow must fail", proc_bad)
        bad_doc = load_json(close_report)
        if bool(bad_doc.get("overall_ok", True)):
            return fail("negative flow overall_ok must be false")
        bad_map = criteria_map(bad_doc.get("criteria"))
        if bad_map.get("schema_gate_ok", True):
            return fail("negative flow must fail schema_gate_ok criterion")
        if bad_map.get("seamgrim_gate_ok", True):
            return fail("negative flow must fail seamgrim_gate_ok criterion")
        digest = bad_doc.get("failure_digest")
        if not isinstance(digest, list) or not digest:
            return fail("negative flow failure_digest must be non-empty")

    print("[age3-close-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
