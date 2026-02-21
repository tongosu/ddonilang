#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCHEMA = "ddn.fixed64.cross_platform_threeway_gate.v1"
PROBE_SCHEMA = "ddn.fixed64.cross_platform_probe.v1"
VECTOR_SCHEMA = "ddn.fixed64.determinism_vector.v1"
SAMPLE_BLAKE3 = "3b4f486e7a86e1ba6b45a8fa89ee9998f2a05f503ae1e9c2ba1722726307e7ed"
SAMPLE_RAW = [6442450944, 2147483648, -2147483648, 2147483648, 8589934592, 2147483648, 0]


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


def make_probe(system: str) -> dict:
    return {
        "schema": PROBE_SCHEMA,
        "generated_at_utc": "2026-02-21T00:00:00Z",
        "ok": True,
        "errors": [],
        "platform": {
            "system": system,
            "release": "selftest",
            "version": "selftest",
            "machine": "x86_64",
            "python": "3.13.0",
        },
        "cmd": ["cargo", "run", "-q", "-p", "ddonirang-core", "--example", "fixed64_determinism_vector"],
        "returncode": 0,
        "probe": {
            "schema": VECTOR_SCHEMA,
            "status": "pass",
            "blake3": SAMPLE_BLAKE3,
            "raw_i64": SAMPLE_RAW,
            "expected_raw_i64": SAMPLE_RAW,
        },
        "compare": {},
        "stdout": [],
        "stderr": [],
    }


def run_gate(
    py: str,
    report_out: Path,
    windows_report: Path,
    linux_report: Path,
    darwin_report: Path,
    require_darwin: bool,
) -> int:
    cmd = [
        py,
        "tests/run_fixed64_cross_platform_threeway_gate.py",
        "--report-out",
        str(report_out),
        "--windows-report",
        str(windows_report),
        "--linux-report",
        str(linux_report),
        "--darwin-report",
        str(darwin_report),
    ]
    if require_darwin:
        cmd.append("--require-darwin")
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return int(proc.returncode)


def main() -> int:
    py = sys.executable
    out_dir = ROOT / "build" / "reports" / "fixed64_threeway_gate_selftest"
    out_dir.mkdir(parents=True, exist_ok=True)
    windows_report = out_dir / "probe_windows.detjson"
    linux_report = out_dir / "probe_linux.detjson"
    darwin_report = out_dir / "probe_darwin.detjson"
    gate_report = out_dir / "gate.detjson"

    # Selftest는 재실행 시에도 동일하게 동작해야 하므로 stale 결과를 제거한다.
    darwin_report.unlink(missing_ok=True)
    gate_report.unlink(missing_ok=True)

    write_json(windows_report, make_probe("Windows"))
    write_json(linux_report, make_probe("Linux"))

    # 1) darwin missing + no require => pending (rc=0)
    rc_pending = run_gate(
        py,
        gate_report,
        windows_report,
        linux_report,
        darwin_report,
        require_darwin=False,
    )
    if rc_pending != 0:
        print("[fixed64-3way-selftest] pending case failed", file=sys.stderr)
        return rc_pending
    pending_doc = load_json(gate_report)
    if not isinstance(pending_doc, dict) or str(pending_doc.get("schema", "")) != SCHEMA:
        print("[fixed64-3way-selftest] pending report invalid", file=sys.stderr)
        return 1
    if str(pending_doc.get("status", "")) != "pending_darwin":
        print(
            f"[fixed64-3way-selftest] pending status mismatch status={pending_doc.get('status')!r}",
            file=sys.stderr,
        )
        return 1

    # 2) darwin missing + require => fail (rc!=0)
    rc_require_fail = run_gate(
        py,
        gate_report,
        windows_report,
        linux_report,
        darwin_report,
        require_darwin=True,
    )
    if rc_require_fail == 0:
        print("[fixed64-3way-selftest] require-darwin missing case should fail", file=sys.stderr)
        return 1
    require_fail_doc = load_json(gate_report)
    if not isinstance(require_fail_doc, dict) or bool(require_fail_doc.get("ok", True)):
        print("[fixed64-3way-selftest] require-darwin fail report invalid", file=sys.stderr)
        return 1

    # 3) darwin present + require => pass_3way (rc=0)
    write_json(darwin_report, make_probe("Darwin"))
    rc_pass = run_gate(
        py,
        gate_report,
        windows_report,
        linux_report,
        darwin_report,
        require_darwin=True,
    )
    if rc_pass != 0:
        print("[fixed64-3way-selftest] pass_3way case failed", file=sys.stderr)
        return rc_pass
    pass_doc = load_json(gate_report)
    if not isinstance(pass_doc, dict):
        print("[fixed64-3way-selftest] pass_3way report invalid", file=sys.stderr)
        return 1
    if str(pass_doc.get("status", "")) != "pass_3way" or not bool(pass_doc.get("ok", False)):
        print("[fixed64-3way-selftest] pass_3way status mismatch", file=sys.stderr)
        return 1

    print("[fixed64-3way-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
