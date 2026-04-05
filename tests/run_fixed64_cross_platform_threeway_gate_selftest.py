#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import zipfile
from datetime import datetime, timedelta, timezone
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


def iso_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def make_probe(
    system: str,
    *,
    generated_at_utc: str | None = None,
    blake3_hex: str = SAMPLE_BLAKE3,
    raw_i64: list[int] | None = None,
) -> dict:
    normalized = system.strip().lower()
    if normalized == "darwin":
        release = "23.6.0"
        version = "Darwin Kernel Version 23.6.0"
        machine = "arm64"
    elif normalized == "windows":
        release = "10.0.22631"
        version = "Windows 11 Pro"
        machine = "AMD64"
    else:
        release = "6.8.0"
        version = "Linux 6.8.0-52-generic"
        machine = "x86_64"

    raw_values = list(raw_i64) if isinstance(raw_i64, list) else list(SAMPLE_RAW)
    return {
        "schema": PROBE_SCHEMA,
        "generated_at_utc": generated_at_utc or iso_now(),
        "ok": True,
        "errors": [],
        "platform": {
            "system": system,
            "release": release,
            "version": version,
            "machine": machine,
            "python": "3.13.0",
        },
        "cmd": ["cargo", "run", "-q", "-p", "ddonirang-core", "--example", "fixed64_determinism_vector"],
        "returncode": 0,
        "probe": {
            "schema": VECTOR_SCHEMA,
            "status": "pass",
            "blake3": blake3_hex,
            "raw_i64": raw_values,
            "expected_raw_i64": raw_values,
        },
        "compare": {},
        "stdout": [
            f"schema={VECTOR_SCHEMA}",
            "status=pass",
            f"blake3={blake3_hex}",
            "raw_i64=" + ",".join(str(v) for v in raw_values),
            "expected_raw_i64=" + ",".join(str(v) for v in raw_values),
        ],
        "stderr": [],
    }


def run_gate(
    py: str,
    report_out: Path,
    windows_report: Path,
    linux_report: Path,
    darwin_report: Path,
    require_darwin: bool,
    max_report_age_minutes: float | None = None,
    resolve_threeway_inputs: bool = False,
    resolve_inputs_strict_invalid: bool = False,
    resolve_input_candidates: list[Path] | None = None,
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
    if max_report_age_minutes is not None:
        cmd.extend(["--max-report-age-minutes", str(max_report_age_minutes)])
    if resolve_threeway_inputs:
        cmd.append("--resolve-threeway-inputs")
    if resolve_inputs_strict_invalid:
        cmd.append("--resolve-inputs-strict-invalid")
    for candidate in list(resolve_input_candidates or []):
        cmd.extend(["--resolve-input-candidate", str(candidate)])
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        env={key: value for key, value in os.environ.items() if key != "DDN_DARWIN_PROBE_REPORT"},
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return int(proc.returncode)


def main() -> int:
    parser = argparse.ArgumentParser(description="Selftest for fixed64 cross-platform threeway gate")
    parser.add_argument(
        "--out-dir",
        default=str(ROOT / "build" / "reports" / "fixed64_threeway_gate_selftest"),
        help="directory for selftest artifacts (default: build/reports/fixed64_threeway_gate_selftest)",
    )
    args = parser.parse_args()

    py = sys.executable
    out_dir = Path(str(args.out_dir)).expanduser()
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

    # 2) darwin missing + windows/linux mismatch => fail (rc!=0)
    mismatch_linux_report = out_dir / "probe_linux_mismatch.detjson"
    mismatch_raw = list(SAMPLE_RAW)
    mismatch_raw[0] = mismatch_raw[0] + 1
    write_json(
        mismatch_linux_report,
        make_probe(
            "Linux",
            blake3_hex="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            raw_i64=mismatch_raw,
        ),
    )
    rc_mismatch = run_gate(
        py,
        gate_report,
        windows_report,
        mismatch_linux_report,
        darwin_report,
        require_darwin=False,
    )
    if rc_mismatch == 0:
        print("[fixed64-3way-selftest] pending path should fail on windows/linux mismatch", file=sys.stderr)
        return 1
    mismatch_doc = load_json(gate_report)
    if not isinstance(mismatch_doc, dict):
        print("[fixed64-3way-selftest] mismatch report invalid", file=sys.stderr)
        return 1
    if str(mismatch_doc.get("reason", "")) != "matrix check failed":
        print("[fixed64-3way-selftest] mismatch reason mismatch", file=sys.stderr)
        return 1

    # 3) darwin missing + require => fail (rc!=0)
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

    # 4) darwin present + require => pass_3way (rc=0)
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

    # 5) stale windows report + freshness option => fail (rc!=0)
    stale_windows_report = out_dir / "probe_windows_stale.detjson"
    fresh_linux_report = out_dir / "probe_linux_fresh.detjson"
    stale_time = (datetime.now(timezone.utc) - timedelta(days=3)).isoformat()
    write_json(stale_windows_report, make_probe("Windows", generated_at_utc=stale_time))
    write_json(fresh_linux_report, make_probe("Linux"))
    darwin_report.unlink(missing_ok=True)
    rc_stale = run_gate(
        py,
        gate_report,
        stale_windows_report,
        fresh_linux_report,
        darwin_report,
        require_darwin=False,
        max_report_age_minutes=60.0,
    )
    if rc_stale == 0:
        print("[fixed64-3way-selftest] stale freshness case should fail", file=sys.stderr)
        return 1
    stale_doc = load_json(gate_report)
    if not isinstance(stale_doc, dict):
        print("[fixed64-3way-selftest] stale freshness report invalid", file=sys.stderr)
        return 1
    if str(stale_doc.get("reason", "")) != "report freshness check failed":
        print("[fixed64-3way-selftest] stale freshness reason mismatch", file=sys.stderr)
        return 1

    # 6) resolve-threeway-inputs + artifact summary candidate => pass_3way (rc=0)
    darwin_report.unlink(missing_ok=True)
    darwin_probe_dir = out_dir / "darwin_probe_payload"
    darwin_probe = darwin_probe_dir / "fixed64_cross_platform_probe_darwin.detjson"
    write_json(darwin_probe, make_probe("Darwin"))
    artifact_summary = out_dir / "fixed64_darwin_probe_artifact.detjson"
    relative_probe = darwin_probe.relative_to(artifact_summary.parent)
    write_json(
        artifact_summary,
        {
            "schema": "ddn.fixed64.darwin_probe_artifact.v1",
            "generated_at_utc": iso_now(),
            "ok": True,
            "status": "staged",
            "reason": "-",
            "probe_report": str(relative_probe).replace("\\", "/"),
            "summary_report": str(artifact_summary),
        },
    )
    rc_resolve_pass = run_gate(
        py,
        gate_report,
        windows_report,
        linux_report,
        darwin_report,
        require_darwin=True,
        resolve_threeway_inputs=True,
        resolve_inputs_strict_invalid=True,
        resolve_input_candidates=[artifact_summary],
    )
    if rc_resolve_pass != 0:
        print("[fixed64-3way-selftest] resolve-inputs artifact summary case failed", file=sys.stderr)
        return rc_resolve_pass
    resolve_doc = load_json(gate_report)
    if not isinstance(resolve_doc, dict):
        print("[fixed64-3way-selftest] resolve-inputs report invalid", file=sys.stderr)
        return 1
    if str(resolve_doc.get("status", "")) != "pass_3way" or not bool(resolve_doc.get("ok", False)):
        print("[fixed64-3way-selftest] resolve-inputs pass_3way status mismatch", file=sys.stderr)
        return 1
    resolve_payload = resolve_doc.get("resolve_inputs")
    if not isinstance(resolve_payload, dict) or not bool(resolve_payload.get("ok", False)):
        print("[fixed64-3way-selftest] resolve_inputs payload missing/failed", file=sys.stderr)
        return 1
    resolve_inner_payload = resolve_payload.get("payload")
    if not isinstance(resolve_inner_payload, dict):
        print("[fixed64-3way-selftest] resolve_inputs inner payload missing", file=sys.stderr)
        return 1
    selected_source = str(resolve_inner_payload.get("selected_source", ""))
    if not selected_source.startswith(f"{artifact_summary} -> ") or str(darwin_probe) not in selected_source:
        print("[fixed64-3way-selftest] resolve selected_source mismatch", file=sys.stderr)
        return 1

    # 7) resolve-threeway-inputs + non-staged resolve result => fail (rc!=0)
    artifact_summary.unlink(missing_ok=True)
    darwin_report_missing = out_dir / "resolve_missing_case" / "probe_darwin.detjson"
    darwin_report_missing.unlink(missing_ok=True)
    missing_summary = out_dir / "fixed64_darwin_probe_artifact_missing.detjson"
    write_json(
        missing_summary,
        {
            "schema": "ddn.fixed64.darwin_probe_artifact.v1",
            "generated_at_utc": iso_now(),
            "ok": True,
            "status": "missing",
            "reason": "selftest",
            "probe_report": "",
            "summary_report": str(missing_summary),
        },
    )
    rc_resolve_missing = run_gate(
        py,
        gate_report,
        windows_report,
        linux_report,
        darwin_report_missing,
        require_darwin=False,
        resolve_threeway_inputs=True,
        resolve_inputs_strict_invalid=False,
        resolve_input_candidates=[missing_summary],
    )
    if rc_resolve_missing == 0:
        print("[fixed64-3way-selftest] resolve non-staged case should fail", file=sys.stderr)
        return 1
    resolve_missing_doc = load_json(gate_report)
    if not isinstance(resolve_missing_doc, dict):
        print("[fixed64-3way-selftest] resolve non-staged report invalid", file=sys.stderr)
        return 1
    if str(resolve_missing_doc.get("reason", "")) != "resolve threeway inputs failed":
        print("[fixed64-3way-selftest] resolve non-staged reason mismatch", file=sys.stderr)
        return 1
    resolve_missing_payload = resolve_missing_doc.get("resolve_inputs")
    if not isinstance(resolve_missing_payload, dict):
        print("[fixed64-3way-selftest] resolve non-staged payload missing", file=sys.stderr)
        return 1
    if "resolve status is not staged" not in str(resolve_missing_payload.get("reason", "")):
        print("[fixed64-3way-selftest] resolve non-staged payload reason mismatch", file=sys.stderr)
        return 1

    # 8) resolve-threeway-inputs + artifact summary(zip.path fallback) => pass_3way (rc=0)
    darwin_report.unlink(missing_ok=True)
    zip_payload = out_dir / "artifact_zip_payload" / "fixed64_darwin_probe_artifact.zip"
    zip_payload.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(zip_payload, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        zf.writestr(
            "nested/fixed64_cross_platform_probe_darwin.detjson",
            json.dumps(make_probe("Darwin"), ensure_ascii=False),
        )
    summary_zip_only = out_dir / "fixed64_darwin_probe_artifact_zip_only.detjson"
    relative_zip = zip_payload.relative_to(summary_zip_only.parent)
    write_json(
        summary_zip_only,
        {
            "schema": "ddn.fixed64.darwin_probe_artifact.v1",
            "generated_at_utc": iso_now(),
            "ok": True,
            "status": "staged",
            "reason": "-",
            "probe_report": "missing_probe.detjson",
            "summary_report": str(summary_zip_only),
            "zip": {
                "enabled": True,
                "path": str(relative_zip).replace("\\", "/"),
                "status": "staged",
                "reason": "-",
            },
        },
    )
    rc_resolve_zip = run_gate(
        py,
        gate_report,
        windows_report,
        linux_report,
        darwin_report,
        require_darwin=True,
        resolve_threeway_inputs=True,
        resolve_inputs_strict_invalid=True,
        resolve_input_candidates=[summary_zip_only],
    )
    if rc_resolve_zip != 0:
        print("[fixed64-3way-selftest] resolve-inputs summary zip fallback case failed", file=sys.stderr)
        return rc_resolve_zip
    resolve_zip_doc = load_json(gate_report)
    if not isinstance(resolve_zip_doc, dict):
        print("[fixed64-3way-selftest] resolve-inputs summary zip fallback report invalid", file=sys.stderr)
        return 1
    if str(resolve_zip_doc.get("status", "")) != "pass_3way" or not bool(resolve_zip_doc.get("ok", False)):
        print("[fixed64-3way-selftest] resolve-inputs summary zip fallback status mismatch", file=sys.stderr)
        return 1
    resolve_zip_payload = resolve_zip_doc.get("resolve_inputs")
    if not isinstance(resolve_zip_payload, dict) or not bool(resolve_zip_payload.get("ok", False)):
        print("[fixed64-3way-selftest] resolve-inputs summary zip fallback payload missing/failed", file=sys.stderr)
        return 1
    resolve_zip_inner_payload = resolve_zip_payload.get("payload")
    if not isinstance(resolve_zip_inner_payload, dict):
        print("[fixed64-3way-selftest] resolve-inputs summary zip fallback inner payload missing", file=sys.stderr)
        return 1
    selected_source_zip = str(resolve_zip_inner_payload.get("selected_source", ""))
    if ".zip!" not in selected_source_zip:
        print("[fixed64-3way-selftest] resolve-inputs summary zip fallback selected_source mismatch", file=sys.stderr)
        return 1

    # 9) resolve-threeway-inputs + strict-invalid + invalid 선행 후보 + valid 후행 후보 => pass_3way (rc=0)
    darwin_report.unlink(missing_ok=True)
    invalid_probe = out_dir / "probe_darwin_invalid.detjson"
    invalid_doc = make_probe("Darwin")
    if isinstance(invalid_doc.get("platform"), dict):
        invalid_doc["platform"]["release"] = "selftest"
    write_json(invalid_probe, invalid_doc)

    valid_probe = out_dir / "probe_darwin_valid.detjson"
    write_json(valid_probe, make_probe("Darwin"))
    rc_resolve_strict_with_valid = run_gate(
        py,
        gate_report,
        windows_report,
        linux_report,
        darwin_report,
        require_darwin=True,
        resolve_threeway_inputs=True,
        resolve_inputs_strict_invalid=True,
        resolve_input_candidates=[invalid_probe, valid_probe],
    )
    if rc_resolve_strict_with_valid != 0:
        print(
            "[fixed64-3way-selftest] resolve strict-invalid should pass when a valid candidate is staged",
            file=sys.stderr,
        )
        return rc_resolve_strict_with_valid
    resolve_strict_doc = load_json(gate_report)
    if not isinstance(resolve_strict_doc, dict):
        print("[fixed64-3way-selftest] resolve strict-invalid report invalid", file=sys.stderr)
        return 1
    if str(resolve_strict_doc.get("status", "")) != "pass_3way" or not bool(resolve_strict_doc.get("ok", False)):
        print("[fixed64-3way-selftest] resolve strict-invalid status mismatch", file=sys.stderr)
        return 1
    resolve_strict_payload = resolve_strict_doc.get("resolve_inputs")
    if not isinstance(resolve_strict_payload, dict) or not bool(resolve_strict_payload.get("ok", False)):
        print("[fixed64-3way-selftest] resolve strict-invalid payload missing/failed", file=sys.stderr)
        return 1
    resolve_strict_inner = resolve_strict_payload.get("payload")
    if not isinstance(resolve_strict_inner, dict):
        print("[fixed64-3way-selftest] resolve strict-invalid inner payload missing", file=sys.stderr)
        return 1
    if str(resolve_strict_inner.get("status", "")) != "staged" or not bool(resolve_strict_inner.get("ok", False)):
        print("[fixed64-3way-selftest] resolve strict-invalid inner status mismatch", file=sys.stderr)
        return 1
    invalid_hits = resolve_strict_inner.get("invalid_hits")
    if not isinstance(invalid_hits, list) or len(invalid_hits) == 0:
        print("[fixed64-3way-selftest] resolve strict-invalid invalid_hits missing", file=sys.stderr)
        return 1
    strict_selected_source = str(resolve_strict_inner.get("selected_source", ""))
    if strict_selected_source != str(valid_probe):
        print("[fixed64-3way-selftest] resolve strict-invalid selected_source mismatch", file=sys.stderr)
        return 1

    print("[fixed64-3way-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
