#!/usr/bin/env python
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
PROBE_SCHEMA = "ddn.fixed64.cross_platform_probe.v1"
VECTOR_SCHEMA = "ddn.fixed64.determinism_vector.v1"


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


def make_probe(system: str, ok: bool = True) -> dict:
    return {
        "schema": PROBE_SCHEMA,
        "generated_at_utc": "2026-02-21T00:00:00Z",
        "ok": ok,
        "errors": [],
        "platform": {
            "system": system,
            "release": "selftest",
            "version": "selftest",
            "machine": "x86_64",
            "python": "3.13.0",
        },
        "probe": {
            "schema": VECTOR_SCHEMA,
            "status": "pass",
            "blake3": "3b4f486e7a86e1ba6b45a8fa89ee9998f2a05f503ae1e9c2ba1722726307e7ed",
            "raw_i64": [1, 2, 3],
            "expected_raw_i64": [1, 2, 3],
        },
    }


def run_script(report_dir: Path, env_path: str, json_out: Path, strict_invalid: bool = False) -> int:
    cmd = [
        sys.executable,
        "tools/scripts/resolve_fixed64_threeway_inputs.py",
        "--report-dir",
        str(report_dir),
        "--darwin-report-env",
        "DDN_DARWIN_PROBE_REPORT",
        "--json-out",
        str(json_out),
    ]
    if strict_invalid:
        cmd.append("--strict-invalid")
    env = os.environ.copy()
    env["DDN_DARWIN_PROBE_REPORT"] = env_path
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return int(proc.returncode)


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="fixed64_threeway_inputs_selftest_") as tmp:
        base = Path(tmp)
        report_dir = base / "reports"
        report_dir.mkdir(parents=True, exist_ok=True)
        json_out = report_dir / "inputs.detjson"

        # 1) missing case
        rc_missing = run_script(report_dir, str(base / "missing.detjson"), json_out, strict_invalid=True)
        if rc_missing != 0:
            print("[fixed64-3way-inputs-selftest] missing case should pass", file=sys.stderr)
            return 1
        missing_doc = load_json(json_out)
        if not isinstance(missing_doc, dict) or str(missing_doc.get("status", "")) != "missing":
            print("[fixed64-3way-inputs-selftest] missing status mismatch", file=sys.stderr)
            return 1

        # 2) valid darwin report case
        darwin_src = base / "darwin_probe.detjson"
        write_json(darwin_src, make_probe("Darwin"))
        rc_staged = run_script(report_dir, str(darwin_src), json_out, strict_invalid=True)
        if rc_staged != 0:
            print("[fixed64-3way-inputs-selftest] staged case failed", file=sys.stderr)
            return 1
        staged_doc = load_json(json_out)
        staged_target = report_dir / "fixed64_cross_platform_probe_darwin.detjson"
        if not isinstance(staged_doc, dict) or str(staged_doc.get("status", "")) != "staged":
            print("[fixed64-3way-inputs-selftest] staged status mismatch", file=sys.stderr)
            return 1
        if not staged_target.exists():
            print("[fixed64-3way-inputs-selftest] staged target missing", file=sys.stderr)
            return 1

        # 3) invalid report case (strict)
        invalid_src = base / "invalid_probe.detjson"
        write_json(invalid_src, make_probe("Linux"))
        rc_invalid = run_script(report_dir, str(invalid_src), json_out, strict_invalid=True)
        if rc_invalid == 0:
            print("[fixed64-3way-inputs-selftest] strict invalid should fail", file=sys.stderr)
            return 1
        invalid_doc = load_json(json_out)
        if not isinstance(invalid_doc, dict) or str(invalid_doc.get("status", "")) != "invalid":
            print("[fixed64-3way-inputs-selftest] invalid status mismatch", file=sys.stderr)
            return 1

    print("[fixed64-3way-inputs-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
