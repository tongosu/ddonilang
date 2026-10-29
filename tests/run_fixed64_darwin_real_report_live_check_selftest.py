#!/usr/bin/env python
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import zipfile
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
PROBE_SCHEMA = "ddn.fixed64.cross_platform_probe.v1"
VECTOR_SCHEMA = "ddn.fixed64.determinism_vector.v1"
LIVE_SCHEMA = "ddn.fixed64.darwin_real_report_live_check.v1"
DARWIN_ARTIFACT_SCHEMA = "ddn.fixed64.darwin_probe_artifact.v1"


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


def make_probe(system: str, *, synthetic: bool = False) -> dict:
    raw_values = [1, 2, 3]
    blake3_hex = "3b4f486e7a86e1ba6b45a8fa89ee9998f2a05f503ae1e9c2ba1722726307e7ed"
    release = "23.6.0"
    version = "Darwin Kernel Version 23.6.0"
    machine = "arm64" if system.lower() == "darwin" else "x86_64"
    if synthetic:
        release = "selftest"
        version = "selftest"
        machine = "selftest"
    return {
        "schema": PROBE_SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
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


def make_artifact_summary(probe_report: str, *, zip_path: str = "") -> dict:
    payload = {
        "schema": DARWIN_ARTIFACT_SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": True,
        "status": "staged",
        "reason": "-",
        "probe_report": probe_report,
        "summary_report": "fixed64_darwin_probe_artifact.detjson",
    }
    if zip_path.strip():
        payload["zip"] = {
            "enabled": True,
            "path": zip_path,
            "status": "staged",
            "reason": "-",
        }
    return payload


def run_live_check(
    *,
    enabled: bool,
    darwin_report: Path,
    windows_report: Path,
    linux_report: Path,
    inputs_report: Path,
    resolve_out: Path,
    json_out: Path,
    candidate: Path | None = None,
    extra_candidates: list[Path] | None = None,
) -> tuple[int, str, str]:
    cmd = [
        sys.executable,
        "tests/run_fixed64_darwin_real_report_live_check.py",
        "--darwin-report",
        str(darwin_report),
        "--windows-report",
        str(windows_report),
        "--linux-report",
        str(linux_report),
        "--inputs-report",
        str(inputs_report),
        "--resolve-inputs-json-out",
        str(resolve_out),
        "--json-out",
        str(json_out),
        "--max-age-minutes",
        "360",
    ]
    if candidate is not None:
        cmd.extend(["--resolve-input-candidate", str(candidate)])
    for item in list(extra_candidates or []):
        cmd.extend(["--resolve-input-candidate", str(item)])

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
    with tempfile.TemporaryDirectory(prefix="fixed64_darwin_live_check_selftest_") as tmp:
        base = Path(tmp)
        reports = base / "reports"
        darwin_report = reports / "fixed64_cross_platform_probe_darwin.detjson"
        windows_report = reports / "fixed64_cross_platform_probe_windows.detjson"
        linux_report = reports / "fixed64_cross_platform_probe_linux.detjson"
        inputs_report = reports / "fixed64_threeway_inputs.live.detjson"
        resolve_out = reports / "fixed64_threeway_inputs.resolve.detjson"
        json_out = reports / "fixed64_darwin_real_report_live_check.detjson"
        candidate = base / "incoming" / "fixed64_cross_platform_probe_darwin.detjson"

        # 1) disabled => skip
        rc_skip, stdout_skip, _ = run_live_check(
            enabled=False,
            darwin_report=darwin_report,
            windows_report=windows_report,
            linux_report=linux_report,
            inputs_report=inputs_report,
            resolve_out=resolve_out,
            json_out=json_out,
        )
        if rc_skip != 0:
            print("[fixed64-darwin-live-check-selftest] disabled case should pass", file=sys.stderr)
            return 1
        if "[fixed64-darwin-live-check] skip" not in stdout_skip:
            print("[fixed64-darwin-live-check-selftest] disabled skip marker missing", file=sys.stderr)
            return 1
        doc_skip = load_json(json_out)
        if not isinstance(doc_skip, dict) or str(doc_skip.get("schema", "")) != LIVE_SCHEMA:
            print("[fixed64-darwin-live-check-selftest] disabled report schema mismatch", file=sys.stderr)
            return 1
        if str(doc_skip.get("status", "")) != "skip_disabled":
            print("[fixed64-darwin-live-check-selftest] disabled status mismatch", file=sys.stderr)
            return 1

        # 2) enabled + missing candidate => fail
        rc_missing, _, stderr_missing = run_live_check(
            enabled=True,
            darwin_report=darwin_report,
            windows_report=windows_report,
            linux_report=linux_report,
            inputs_report=inputs_report,
            resolve_out=resolve_out,
            json_out=json_out,
        )
        if rc_missing == 0:
            print("[fixed64-darwin-live-check-selftest] missing candidate case should fail", file=sys.stderr)
            return 1
        if "[fixed64-darwin-live-check] fail_readiness" not in stderr_missing:
            print("[fixed64-darwin-live-check-selftest] missing candidate fail marker mismatch", file=sys.stderr)
            return 1

        # 3) enabled + synthetic candidate => fail (resolve should reject)
        synthetic_candidate = base / "incoming_synthetic" / "fixed64_cross_platform_probe_darwin.detjson"
        write_json(synthetic_candidate, make_probe("Darwin", synthetic=True))
        darwin_report.unlink(missing_ok=True)
        rc_synthetic, _, stderr_synthetic = run_live_check(
            enabled=True,
            darwin_report=darwin_report,
            windows_report=windows_report,
            linux_report=linux_report,
            inputs_report=inputs_report,
            resolve_out=resolve_out,
            json_out=json_out,
            candidate=synthetic_candidate,
        )
        if rc_synthetic == 0:
            print("[fixed64-darwin-live-check-selftest] synthetic candidate case should fail", file=sys.stderr)
            return 1
        if "[fixed64-darwin-live-check] fail_readiness" not in stderr_synthetic:
            print("[fixed64-darwin-live-check-selftest] synthetic candidate fail marker mismatch", file=sys.stderr)
            return 1
        doc_synthetic = load_json(json_out)
        if not isinstance(doc_synthetic, dict) or str(doc_synthetic.get("status", "")) != "fail_readiness":
            print("[fixed64-darwin-live-check-selftest] synthetic candidate status mismatch", file=sys.stderr)
            return 1
        invalid_hits = doc_synthetic.get("resolve_invalid_hits")
        if not isinstance(invalid_hits, list) or not invalid_hits:
            print("[fixed64-darwin-live-check-selftest] synthetic candidate invalid_hits missing", file=sys.stderr)
            return 1

        # 4) enabled + default summary candidate + windows/linux ready => pass_3way
        write_json(windows_report, make_probe("Windows"))
        write_json(linux_report, make_probe("Linux"))
        darwin_report.unlink(missing_ok=True)
        default_summary_probe_dir = reports / "default_summary_payload"
        default_summary_probe = default_summary_probe_dir / "fixed64_cross_platform_probe_darwin.detjson"
        write_json(default_summary_probe, make_probe("Darwin"))
        default_summary = reports / "fixed64_darwin_probe_artifact.detjson"
        write_json(
            default_summary,
            make_artifact_summary(str(default_summary_probe.relative_to(default_summary.parent)).replace("\\", "/")),
        )
        rc_default_summary, stdout_default_summary, _ = run_live_check(
            enabled=True,
            darwin_report=darwin_report,
            windows_report=windows_report,
            linux_report=linux_report,
            inputs_report=inputs_report,
            resolve_out=resolve_out,
            json_out=json_out,
        )
        if rc_default_summary != 0:
            print("[fixed64-darwin-live-check-selftest] default summary candidate case should pass", file=sys.stderr)
            return 1
        if "[fixed64-darwin-live-check] ok status=pass_3way" not in stdout_default_summary:
            print("[fixed64-darwin-live-check-selftest] default summary marker mismatch", file=sys.stderr)
            return 1
        doc_default_summary = load_json(json_out)
        if not isinstance(doc_default_summary, dict) or str(doc_default_summary.get("status", "")) != "pass_3way":
            print("[fixed64-darwin-live-check-selftest] default summary status mismatch", file=sys.stderr)
            return 1
        if not darwin_report.exists():
            print("[fixed64-darwin-live-check-selftest] default summary did not stage darwin report", file=sys.stderr)
            return 1
        resolved_source_default = str(doc_default_summary.get("resolved_source", ""))
        if "fixed64_darwin_probe_artifact.detjson" not in resolved_source_default:
            print("[fixed64-darwin-live-check-selftest] default summary resolved_source mismatch", file=sys.stderr)
            return 1

        # 5) enabled + default summary(zip.path fallback) + windows/linux ready => pass_3way
        write_json(windows_report, make_probe("Windows"))
        write_json(linux_report, make_probe("Linux"))
        darwin_report.unlink(missing_ok=True)
        zip_payload = reports / "default_summary_zip_payload" / "fixed64_darwin_probe_artifact.zip"
        zip_payload.parent.mkdir(parents=True, exist_ok=True)
        with zipfile.ZipFile(zip_payload, "w", compression=zipfile.ZIP_DEFLATED) as zf:
            zf.writestr(
                "nested/fixed64_cross_platform_probe_darwin.detjson",
                json.dumps(make_probe("Darwin"), ensure_ascii=False),
            )
        write_json(
            default_summary,
            make_artifact_summary(
                "missing_probe.detjson",
                zip_path=str(zip_payload.relative_to(default_summary.parent)).replace("\\", "/"),
            ),
        )
        rc_default_summary_zip, stdout_default_summary_zip, _ = run_live_check(
            enabled=True,
            darwin_report=darwin_report,
            windows_report=windows_report,
            linux_report=linux_report,
            inputs_report=inputs_report,
            resolve_out=resolve_out,
            json_out=json_out,
        )
        if rc_default_summary_zip != 0:
            print("[fixed64-darwin-live-check-selftest] default summary zip fallback case should pass", file=sys.stderr)
            return 1
        if "[fixed64-darwin-live-check] ok status=pass_3way" not in stdout_default_summary_zip:
            print("[fixed64-darwin-live-check-selftest] default summary zip fallback marker mismatch", file=sys.stderr)
            return 1
        doc_default_summary_zip = load_json(json_out)
        if not isinstance(doc_default_summary_zip, dict) or str(doc_default_summary_zip.get("status", "")) != "pass_3way":
            print("[fixed64-darwin-live-check-selftest] default summary zip fallback status mismatch", file=sys.stderr)
            return 1
        resolved_source_default_zip = str(doc_default_summary_zip.get("resolved_source", ""))
        if ".zip!" not in resolved_source_default_zip:
            print("[fixed64-darwin-live-check-selftest] default summary zip fallback resolved_source mismatch", file=sys.stderr)
            return 1

        # 6) enabled + valid candidate + windows/linux ready => pass_3way
        write_json(windows_report, make_probe("Windows"))
        write_json(linux_report, make_probe("Linux"))
        write_json(candidate, make_probe("Darwin"))
        darwin_report.unlink(missing_ok=True)
        rc_pass, stdout_pass, _ = run_live_check(
            enabled=True,
            darwin_report=darwin_report,
            windows_report=windows_report,
            linux_report=linux_report,
            inputs_report=inputs_report,
            resolve_out=resolve_out,
            json_out=json_out,
            candidate=candidate,
        )
        if rc_pass != 0:
            print("[fixed64-darwin-live-check-selftest] pass_3way case should pass", file=sys.stderr)
            return 1
        if "[fixed64-darwin-live-check] ok status=pass_3way" not in stdout_pass:
            print("[fixed64-darwin-live-check-selftest] pass_3way marker mismatch", file=sys.stderr)
            return 1
        doc_pass = load_json(json_out)
        if not isinstance(doc_pass, dict) or str(doc_pass.get("status", "")) != "pass_3way":
            print("[fixed64-darwin-live-check-selftest] pass_3way status mismatch", file=sys.stderr)
            return 1
        if not darwin_report.exists():
            print("[fixed64-darwin-live-check-selftest] darwin report was not staged", file=sys.stderr)
            return 1
        resolved_source_pass = str(doc_pass.get("resolved_source", ""))
        if str(candidate) not in resolved_source_pass:
            print("[fixed64-darwin-live-check-selftest] explicit candidate resolved_source mismatch", file=sys.stderr)
            return 1

        # 7) enabled + strict resolve invalid 선행/valid 후행 후보 => pass_3way 유지
        write_json(windows_report, make_probe("Windows"))
        write_json(linux_report, make_probe("Linux"))
        darwin_report.unlink(missing_ok=True)
        invalid_candidate = base / "incoming_invalid" / "fixed64_cross_platform_probe_darwin.detjson"
        valid_candidate = base / "incoming_valid" / "fixed64_cross_platform_probe_darwin.detjson"
        write_json(invalid_candidate, make_probe("Darwin", synthetic=True))
        write_json(valid_candidate, make_probe("Darwin"))
        rc_mixed, stdout_mixed, _ = run_live_check(
            enabled=True,
            darwin_report=darwin_report,
            windows_report=windows_report,
            linux_report=linux_report,
            inputs_report=inputs_report,
            resolve_out=resolve_out,
            json_out=json_out,
            candidate=invalid_candidate,
            extra_candidates=[valid_candidate],
        )
        if rc_mixed != 0:
            print("[fixed64-darwin-live-check-selftest] mixed invalid/valid candidate case should pass", file=sys.stderr)
            return 1
        if "[fixed64-darwin-live-check] ok status=pass_3way" not in stdout_mixed:
            print("[fixed64-darwin-live-check-selftest] mixed invalid/valid marker mismatch", file=sys.stderr)
            return 1
        doc_mixed = load_json(json_out)
        if not isinstance(doc_mixed, dict) or str(doc_mixed.get("status", "")) != "pass_3way":
            print("[fixed64-darwin-live-check-selftest] mixed invalid/valid status mismatch", file=sys.stderr)
            return 1
        mixed_invalid_hits = doc_mixed.get("resolve_invalid_hits")
        if not isinstance(mixed_invalid_hits, list) or len(mixed_invalid_hits) == 0:
            print("[fixed64-darwin-live-check-selftest] mixed invalid/valid invalid_hits missing", file=sys.stderr)
            return 1
        resolved_source_mixed = str(doc_mixed.get("resolved_source", ""))
        if str(valid_candidate) not in resolved_source_mixed:
            print("[fixed64-darwin-live-check-selftest] mixed invalid/valid resolved_source mismatch", file=sys.stderr)
            return 1

    print("[fixed64-darwin-live-check-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
