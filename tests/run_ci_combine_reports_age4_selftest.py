#!/usr/bin/env python
from __future__ import annotations

import json
import io
from contextlib import contextmanager, redirect_stderr, redirect_stdout
import runpy
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[ci-combine-age4-selftest] fail: {msg}")
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


def run_cmd_inprocess(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    if len(cmd) < 2 or not str(cmd[1]).endswith(".py"):
        return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")
    script = str(cmd[1])
    argv = [script, *[str(arg) for arg in cmd[2:]]]
    stdout_buf = io.StringIO()
    stderr_buf = io.StringIO()
    old_argv = sys.argv
    returncode = 0
    try:
        sys.argv = argv
        with redirect_stdout(stdout_buf), redirect_stderr(stderr_buf):
            try:
                runpy.run_path(script, run_name="__main__")
            except SystemExit as exc:
                code = exc.code
                if code is None:
                    returncode = 0
                elif isinstance(code, int):
                    returncode = code
                else:
                    returncode = 1
                    stderr_buf.write(str(code))
            except Exception as exc:  # pragma: no cover - defensive fallback
                returncode = 1
                stderr_buf.write(f"{type(exc).__name__}: {exc}")
    finally:
        sys.argv = old_argv
    return subprocess.CompletedProcess(
        args=cmd,
        returncode=returncode,
        stdout=stdout_buf.getvalue(),
        stderr=stderr_buf.getvalue(),
    )


def run_combine(
    *,
    seamgrim: Path,
    age3: Path,
    age4: Path,
    age4_proof: Path,
    age5: Path,
    oi: Path,
    out: Path,
    require_age4: bool,
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
        "--age4-proof-report",
        str(age4_proof),
        "--age5-report",
        str(age5),
        "--out",
        str(out),
        "--fail-on-bad",
        "--require-age3",
    ]
    if require_age4:
        cmd.append("--require-age4")
    return run_cmd_inprocess(cmd)


def run_aggregate_digest(report: Path) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tools/scripts/print_ci_aggregate_digest.py",
        str(report),
        "--top",
        "1",
    ]
    return run_cmd_inprocess(cmd)


@contextmanager
def persistent_tmpdir(prefix: str):
    # Selftest speedup: skip TemporaryDirectory cleanup(rmtree) cost.
    yield tempfile.mkdtemp(prefix=prefix)


def main() -> int:
    with persistent_tmpdir("ci_combine_age4_selftest_") as tmp:
        root = Path(tmp)
        seamgrim_report = root / "seamgrim.json"
        age3_report = root / "age3.detjson"
        age4_report = root / "age4.detjson"
        age4_proof_report = root / "age4_proof.detjson"
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
            oi_report,
            {
                "schema": "ddn.oi405_406.close_report.v1",
                "overall_ok": True,
                "packs": [],
                "failure_digest": [],
            },
        )
        write_json(
            age5_report,
            {
                "schema": "ddn.age5_close_report.v1",
                "overall_ok": True,
                "criteria": [],
                "failure_digest": [],
            },
        )

        # case 1: require-age4 + valid age4 report => pass
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
            age4_proof_report,
            {
                "schema": "ddn.age4.proof_artifact_report.v1",
                "overall_ok": True,
                "criteria": [],
                "failure_digest": [],
                "proof_summary_path": str(root / "proof_artifact_summary.detjson"),
                "proof_summary_hash": "sha256:test",
            },
        )
        proc_ok = run_combine(
            seamgrim=seamgrim_report,
            age3=age3_report,
            age4=age4_report,
            age4_proof=age4_proof_report,
            age5=age5_report,
            oi=oi_report,
            out=out_report,
            require_age4=True,
        )
        if proc_ok.returncode != 0:
            return fail(f"require-age4 pass case failed: out={proc_ok.stdout} err={proc_ok.stderr}")
        ok_doc = read_json(out_report)
        if not isinstance(ok_doc, dict) or not bool(ok_doc.get("overall_ok", False)):
            return fail("require-age4 pass case aggregate overall_ok mismatch")
        age4_row = ok_doc.get("age4")
        if not isinstance(age4_row, dict) or not bool(age4_row.get("ok", False)):
            return fail("require-age4 pass case age4.ok mismatch")
        if not bool(age4_row.get("proof_artifact_ok", False)):
            return fail("require-age4 pass case age4 proof_artifact_ok mismatch")
        if str(age4_row.get("proof_artifact_summary_hash", "")).strip() != "sha256:test":
            return fail("require-age4 pass case proof summary hash mismatch")
        if str(age4_row.get("proof_artifact_failed_preview", "")).strip() != "-":
            return fail("require-age4 pass case proof failed preview mismatch")
        digest_ok = run_aggregate_digest(out_report)
        if digest_ok.returncode != 0:
            return fail(f"require-age4 pass case aggregate digest failed: out={digest_ok.stdout} err={digest_ok.stderr}")
        digest_ok_text = str(digest_ok.stdout or "")
        if "age4_proof_ok=1" not in digest_ok_text:
            return fail("require-age4 pass case aggregate digest age4_proof_ok mismatch")
        if "age4_proof_failed=0" not in digest_ok_text:
            return fail("require-age4 pass case aggregate digest age4_proof_failed mismatch")
        if "age4_proof_failed_preview=-" not in digest_ok_text:
            return fail("require-age4 pass case aggregate digest age4_proof_failed_preview mismatch")

        # case 2: require-age4 + failing age4 proof report => fail
        write_json(
            age4_proof_report,
            {
                "schema": "ddn.age4.proof_artifact_report.v1",
                "overall_ok": False,
                "criteria": [{"name": "proof_runtime_error_statehash_preserved", "ok": False}],
                "failed_criteria_preview": "proof_runtime_error_statehash_preserved",
                "failure_digest": ["proof_runtime_error_statehash_preserved: missing_state_hash"],
                "proof_summary_path": str(root / "proof_artifact_summary.detjson"),
                "proof_summary_hash": "sha256:bad",
            },
        )
        proc_proof_fail = run_combine(
            seamgrim=seamgrim_report,
            age3=age3_report,
            age4=age4_report,
            age4_proof=age4_proof_report,
            age5=age5_report,
            oi=oi_report,
            out=out_report,
            require_age4=True,
        )
        if proc_proof_fail.returncode == 0:
            return fail("require-age4 failing proof report case must fail")
        proof_fail_doc = read_json(out_report)
        if not isinstance(proof_fail_doc, dict) or bool(proof_fail_doc.get("overall_ok", True)):
            return fail("require-age4 failing proof report case overall_ok must be false")
        proof_fail_age4 = proof_fail_doc.get("age4")
        if not isinstance(proof_fail_age4, dict) or bool(proof_fail_age4.get("ok", True)):
            return fail("require-age4 failing proof report case age4.ok must be false")
        if bool(proof_fail_age4.get("proof_artifact_ok", True)):
            return fail("require-age4 failing proof report case proof_artifact_ok must be false")
        if str(proof_fail_age4.get("proof_artifact_failed_preview", "")).strip() != "proof_runtime_error_statehash_preserved":
            return fail("require-age4 failing proof report case proof failed preview mismatch")
        if "proof_artifact::proof_runtime_error_statehash_preserved" not in proof_fail_age4.get("failed_criteria", []):
            return fail("require-age4 failing proof report case failed_criteria missing child proof token")
        digest_proof_fail = run_aggregate_digest(out_report)
        if digest_proof_fail.returncode != 0:
            return fail(
                f"require-age4 failing proof report case aggregate digest failed: out={digest_proof_fail.stdout} err={digest_proof_fail.stderr}"
            )
        digest_proof_fail_text = str(digest_proof_fail.stdout or "")
        if "age4_proof_ok=0" not in digest_proof_fail_text:
            return fail("require-age4 failing proof report case aggregate digest age4_proof_ok mismatch")
        if "age4_proof_failed=1" not in digest_proof_fail_text:
            return fail("require-age4 failing proof report case aggregate digest age4_proof_failed mismatch")
        if "age4_proof_failed_preview=proof_runtime_error_statehash_preserved" not in digest_proof_fail_text:
            return fail("require-age4 failing proof report case aggregate digest age4_proof_failed_preview mismatch")

        write_json(
            age4_proof_report,
            {
                "schema": "ddn.age4.proof_artifact_report.v1",
                "overall_ok": True,
                "criteria": [],
                "failed_criteria_preview": "-",
                "failure_digest": [],
                "proof_summary_path": str(root / "proof_artifact_summary.detjson"),
                "proof_summary_hash": "sha256:test",
            },
        )

        # case 3: require-age4 + missing age4 report => fail
        age4_report.unlink(missing_ok=True)
        proc_missing = run_combine(
            seamgrim=seamgrim_report,
            age3=age3_report,
            age4=age4_report,
            age4_proof=age4_proof_report,
            age5=age5_report,
            oi=oi_report,
            out=out_report,
            require_age4=True,
        )
        if proc_missing.returncode == 0:
            return fail("require-age4 missing case must fail")
        miss_doc = read_json(out_report)
        if not isinstance(miss_doc, dict):
            return fail("require-age4 missing case report not generated")
        if bool(miss_doc.get("overall_ok", True)):
            return fail("require-age4 missing case overall_ok must be false")
        miss_age4 = miss_doc.get("age4")
        if not isinstance(miss_age4, dict) or bool(miss_age4.get("ok", True)):
            return fail("require-age4 missing case age4.ok must be false")

        # case 4: optional age4 + missing age4 report => pass (skipped)
        proc_optional = run_combine(
            seamgrim=seamgrim_report,
            age3=age3_report,
            age4=age4_report,
            age4_proof=age4_proof_report,
            age5=age5_report,
            oi=oi_report,
            out=out_report,
            require_age4=False,
        )
        if proc_optional.returncode != 0:
            return fail(f"optional-age4 missing case failed: out={proc_optional.stdout} err={proc_optional.stderr}")
        optional_doc = read_json(out_report)
        if not isinstance(optional_doc, dict) or not bool(optional_doc.get("overall_ok", False)):
            return fail("optional-age4 missing case overall_ok mismatch")
        optional_age4 = optional_doc.get("age4")
        if not isinstance(optional_age4, dict):
            return fail("optional-age4 missing case age4 block missing")
        if not bool(optional_age4.get("ok", False)) or not bool(optional_age4.get("skipped", False)):
            return fail("optional-age4 missing case skipped semantics mismatch")

    print("[ci-combine-age4-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
