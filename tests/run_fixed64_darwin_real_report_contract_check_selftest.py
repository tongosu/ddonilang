#!/usr/bin/env python
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from datetime import datetime, timedelta, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
REPORT_SCHEMA = "ddn.fixed64.cross_platform_probe.v1"
VECTOR_SCHEMA = "ddn.fixed64.determinism_vector.v1"
CONTRACT_SCHEMA = "ddn.fixed64.darwin_real_report_contract.v1"


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def make_probe(
    system: str,
    generated_at: datetime | None = None,
    *,
    release: str = "23.6.0",
    version: str = "Darwin Kernel Version 23.6.0",
    machine: str = "arm64",
) -> dict:
    stamp = generated_at or datetime.now(timezone.utc)
    raw_values = [1, 2, 3]
    blake3_hex = "3b4f486e7a86e1ba6b45a8fa89ee9998f2a05f503ae1e9c2ba1722726307e7ed"
    return {
        "schema": REPORT_SCHEMA,
        "generated_at_utc": stamp.isoformat(),
        "ok": True,
        "errors": [],
        "platform": {
            "system": system,
            "release": release,
            "version": version,
            "machine": machine,
            "python": "3.13.0",
        },
        "probe": {
            "schema": VECTOR_SCHEMA,
            "status": "pass",
            "blake3": blake3_hex,
            "raw_i64": raw_values,
            "expected_raw_i64": raw_values,
        },
        "cmd": ["cargo", "run", "-q", "-p", "ddonirang-core", "--example", "fixed64_determinism_vector"],
        "returncode": 0,
        "stdout": [
            f"schema={VECTOR_SCHEMA}",
            "status=pass",
            f"blake3={blake3_hex}",
            "raw_i64=1,2,3",
            "expected_raw_i64=1,2,3",
        ],
        "stderr": [],
    }


def run_check(
    *,
    report_path: Path,
    inputs_path: Path,
    json_out: Path,
    enabled: bool,
    max_age_minutes: float = 0.0,
    resolve_threeway_inputs: bool = False,
    resolve_inputs_json_out: Path | None = None,
    resolve_inputs_strict_invalid: bool = False,
    resolve_inputs_require_when_env: str = "",
    resolve_input_candidates: list[Path] | None = None,
) -> tuple[int, str, str]:
    cmd = [
        sys.executable,
        "tests/run_fixed64_darwin_real_report_contract_check.py",
        "--report",
        str(report_path),
        "--inputs-report",
        str(inputs_path),
        "--json-out",
        str(json_out),
        "--max-age-minutes",
        str(float(max_age_minutes)),
    ]
    if resolve_threeway_inputs:
        cmd.append("--resolve-threeway-inputs")
    if resolve_inputs_json_out is not None:
        cmd.extend(["--resolve-inputs-json-out", str(resolve_inputs_json_out)])
    if resolve_inputs_strict_invalid:
        cmd.append("--resolve-inputs-strict-invalid")
    if resolve_inputs_require_when_env.strip():
        cmd.extend(["--resolve-inputs-require-when-env", resolve_inputs_require_when_env.strip()])
    for candidate in resolve_input_candidates or []:
        cmd.extend(["--resolve-input-candidate", str(candidate)])

    env = os.environ.copy()
    env["DDN_ENABLE_DARWIN_PROBE"] = "1" if enabled else "0"
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return int(proc.returncode), (proc.stdout or ""), (proc.stderr or "")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="fixed64_darwin_real_report_contract_selftest_") as tmp:
        base = Path(tmp)
        report_path = base / "fixed64_cross_platform_probe_darwin.detjson"
        inputs_path = base / "fixed64_threeway_inputs.detjson"
        json_out = base / "fixed64_darwin_real_report_contract.detjson"

        write_json(inputs_path, {"schema": "ddn.fixed64.threeway_inputs.v1", "target_report": str(report_path)})

        # 1) disabled -> skip
        rc_skip, stdout_skip, _ = run_check(
            report_path=report_path,
            inputs_path=inputs_path,
            json_out=json_out,
            enabled=False,
        )
        if rc_skip != 0:
            print("[fixed64-darwin-real-report-selftest] disabled skip case failed", file=sys.stderr)
            return 1
        if "[fixed64-darwin-real-report] skip" not in stdout_skip:
            print("[fixed64-darwin-real-report-selftest] disabled skip marker missing", file=sys.stderr)
            return 1
        skip_doc = load_json(json_out)
        if not isinstance(skip_doc, dict) or str(skip_doc.get("schema", "")) != CONTRACT_SCHEMA:
            print("[fixed64-darwin-real-report-selftest] disabled skip report invalid", file=sys.stderr)
            return 1
        if str(skip_doc.get("status", "")) != "skip_disabled":
            print("[fixed64-darwin-real-report-selftest] disabled skip status mismatch", file=sys.stderr)
            return 1

        # 2) enabled + missing -> fail
        report_path.unlink(missing_ok=True)
        rc_missing, _, stderr_missing = run_check(
            report_path=report_path,
            inputs_path=inputs_path,
            json_out=json_out,
            enabled=True,
        )
        if rc_missing == 0:
            print("[fixed64-darwin-real-report-selftest] enabled missing case should fail", file=sys.stderr)
            return 1
        if "[fixed64-darwin-real-report] fail" not in stderr_missing:
            print("[fixed64-darwin-real-report-selftest] enabled missing fail marker missing", file=sys.stderr)
            return 1
        missing_doc = load_json(json_out)
        if not isinstance(missing_doc, dict) or str(missing_doc.get("status", "")) != "fail":
            print("[fixed64-darwin-real-report-selftest] enabled missing status mismatch", file=sys.stderr)
            return 1

        # 3) enabled + missing report + resolve candidate -> pass
        report_path.unlink(missing_ok=True)
        incoming_dir = base / "incoming"
        candidate_report = incoming_dir / "fixed64_cross_platform_probe_darwin.detjson"
        resolve_out = base / "fixed64_threeway_inputs.resolve.detjson"
        write_json(candidate_report, make_probe("Darwin"))
        rc_resolve_pass, stdout_resolve_pass, _ = run_check(
            report_path=report_path,
            inputs_path=inputs_path,
            json_out=json_out,
            enabled=True,
            max_age_minutes=360.0,
            resolve_threeway_inputs=True,
            resolve_inputs_json_out=resolve_out,
            resolve_inputs_strict_invalid=True,
            resolve_inputs_require_when_env="DDN_ENABLE_DARWIN_PROBE",
            resolve_input_candidates=[candidate_report],
        )
        if rc_resolve_pass != 0:
            print("[fixed64-darwin-real-report-selftest] resolve pass case failed", file=sys.stderr)
            return 1
        if "[fixed64-darwin-real-report] ok" not in stdout_resolve_pass:
            print("[fixed64-darwin-real-report-selftest] resolve pass marker missing", file=sys.stderr)
            return 1
        if not report_path.exists():
            print("[fixed64-darwin-real-report-selftest] resolve did not stage report", file=sys.stderr)
            return 1
        resolve_pass_doc = load_json(json_out)
        if not isinstance(resolve_pass_doc, dict) or str(resolve_pass_doc.get("status", "")) != "pass":
            print("[fixed64-darwin-real-report-selftest] resolve pass status mismatch", file=sys.stderr)
            return 1
        resolve_inputs_doc = resolve_pass_doc.get("resolve_inputs")
        if not isinstance(resolve_inputs_doc, dict) or not bool(resolve_inputs_doc.get("ok", False)):
            print("[fixed64-darwin-real-report-selftest] resolve_inputs should be ok", file=sys.stderr)
            return 1
        resolve_payload = resolve_inputs_doc.get("payload")
        if not isinstance(resolve_payload, dict):
            print("[fixed64-darwin-real-report-selftest] resolve_inputs payload missing", file=sys.stderr)
            return 1
        selected_source = str(resolve_payload.get("selected_source", ""))
        if selected_source != str(candidate_report):
            print("[fixed64-darwin-real-report-selftest] resolve selected_source mismatch", file=sys.stderr)
            return 1

        # 4) enabled + darwin report -> pass
        write_json(report_path, make_probe("Darwin"))
        rc_pass, stdout_pass, _ = run_check(
            report_path=report_path,
            inputs_path=inputs_path,
            json_out=json_out,
            enabled=True,
            max_age_minutes=360.0,
        )
        if rc_pass != 0:
            print("[fixed64-darwin-real-report-selftest] enabled pass case failed", file=sys.stderr)
            return 1
        if "[fixed64-darwin-real-report] ok" not in stdout_pass:
            print("[fixed64-darwin-real-report-selftest] enabled pass marker missing", file=sys.stderr)
            return 1
        pass_doc = load_json(json_out)
        if not isinstance(pass_doc, dict) or str(pass_doc.get("status", "")) != "pass":
            print("[fixed64-darwin-real-report-selftest] enabled pass status mismatch", file=sys.stderr)
            return 1

        # 5) enabled + non-darwin report -> fail
        write_json(report_path, make_probe("Linux"))
        rc_non_darwin, _, _ = run_check(
            report_path=report_path,
            inputs_path=inputs_path,
            json_out=json_out,
            enabled=True,
        )
        if rc_non_darwin == 0:
            print("[fixed64-darwin-real-report-selftest] non-darwin case should fail", file=sys.stderr)
            return 1

        # 6) enabled + synthetic darwin platform payload -> fail
        write_json(
            report_path,
            make_probe(
                "Darwin",
                release="selftest",
                version="selftest",
                machine="selftest",
            ),
        )
        rc_synthetic, _, _ = run_check(
            report_path=report_path,
            inputs_path=inputs_path,
            json_out=json_out,
            enabled=True,
        )
        if rc_synthetic == 0:
            print("[fixed64-darwin-real-report-selftest] synthetic platform case should fail", file=sys.stderr)
            return 1

        # 7) enabled + stale report -> fail
        stale_stamp = datetime.now(timezone.utc) - timedelta(days=2)
        write_json(report_path, make_probe("Darwin", generated_at=stale_stamp))
        rc_stale, _, _ = run_check(
            report_path=report_path,
            inputs_path=inputs_path,
            json_out=json_out,
            enabled=True,
            max_age_minutes=60.0,
        )
        if rc_stale == 0:
            print("[fixed64-darwin-real-report-selftest] stale case should fail", file=sys.stderr)
            return 1

    print("[fixed64-darwin-real-report-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
