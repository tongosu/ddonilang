#!/usr/bin/env python
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
PROBE_SCHEMA = "ddn.fixed64.cross_platform_probe.v1"
VECTOR_SCHEMA = "ddn.fixed64.determinism_vector.v1"
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


def make_probe(system: str, ok: bool = True, *, synthetic: bool = False) -> dict:
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
    if synthetic:
        release = "selftest"
        version = "selftest"
        machine = "selftest"

    return {
        "schema": PROBE_SCHEMA,
        "generated_at_utc": "2026-02-21T00:00:00Z",
        "ok": ok,
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
            "blake3": "3b4f486e7a86e1ba6b45a8fa89ee9998f2a05f503ae1e9c2ba1722726307e7ed",
            "raw_i64": [1, 2, 3],
            "expected_raw_i64": [1, 2, 3],
        },
        "cmd": ["cargo", "run", "-q", "-p", "ddonirang-core", "--example", "fixed64_determinism_vector"],
        "returncode": 0,
        "stdout": [
            f"schema={VECTOR_SCHEMA}",
            "status=pass",
            "blake3=3b4f486e7a86e1ba6b45a8fa89ee9998f2a05f503ae1e9c2ba1722726307e7ed",
            "raw_i64=1,2,3",
            "expected_raw_i64=1,2,3",
        ],
        "stderr": [],
    }


def make_artifact_summary(
    probe_report: str = "",
    status: str = "staged",
    *,
    zip_path: str = "",
) -> dict:
    payload = {
        "schema": DARWIN_ARTIFACT_SCHEMA,
        "generated_at_utc": "2026-02-21T00:00:00Z",
        "ok": status == "staged",
        "status": status,
        "reason": "-",
        "probe_report": probe_report,
        "summary_report": "fixed64_darwin_probe_artifact.detjson",
    }
    if zip_path.strip():
        payload["zip"] = {
            "enabled": True,
            "path": zip_path,
            "status": "staged" if status == "staged" else "pending",
            "reason": "-",
        }
    return payload


def run_script(
    report_dir: Path,
    env_path: str,
    json_out: Path,
    strict_invalid: bool = False,
    require_when_env: bool = False,
    enable_darwin_mode: bool = False,
) -> int:
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
    if require_when_env:
        cmd.extend(["--require-when-env", "DDN_ENABLE_DARWIN_PROBE"])
    env = os.environ.copy()
    env["DDN_DARWIN_PROBE_REPORT"] = env_path
    if enable_darwin_mode:
        env["DDN_ENABLE_DARWIN_PROBE"] = "1"
    else:
        env["DDN_ENABLE_DARWIN_PROBE"] = "0"
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

        # 4) synthetic darwin report case (strict)
        synthetic_src = base / "synthetic_probe.detjson"
        write_json(synthetic_src, make_probe("Darwin", synthetic=True))
        rc_synthetic = run_script(report_dir, str(synthetic_src), json_out, strict_invalid=True)
        if rc_synthetic == 0:
            print("[fixed64-3way-inputs-selftest] synthetic probe should fail in strict mode", file=sys.stderr)
            return 1
        synthetic_doc = load_json(json_out)
        if not isinstance(synthetic_doc, dict) or str(synthetic_doc.get("status", "")) != "invalid":
            print("[fixed64-3way-inputs-selftest] synthetic invalid status mismatch", file=sys.stderr)
            return 1

        # 5) require-when-env + missing => fail
        staged_target.unlink(missing_ok=True)
        rc_required_missing = run_script(
            report_dir,
            str(base / "missing_required.detjson"),
            json_out,
            strict_invalid=True,
            require_when_env=True,
            enable_darwin_mode=True,
        )
        if rc_required_missing == 0:
            print("[fixed64-3way-inputs-selftest] required missing should fail", file=sys.stderr)
            return 1
        required_doc = load_json(json_out)
        if not isinstance(required_doc, dict) or str(required_doc.get("status", "")) != "missing_required":
            print("[fixed64-3way-inputs-selftest] required missing status mismatch", file=sys.stderr)
            return 1

        # 6) archive directory candidate case (pick latest valid report)
        archive_dir = base / "darwin_probe_archive"
        archive_old = archive_dir / "fixed64_cross_platform_probe_darwin.20260101T010101Z.detjson"
        archive_new = archive_dir / "fixed64_cross_platform_probe_darwin.20260102T020202Z.detjson"
        write_json(archive_old, make_probe("Darwin"))
        write_json(archive_new, make_probe("Darwin"))
        staged_target.unlink(missing_ok=True)
        rc_archive = run_script(report_dir, str(archive_dir), json_out, strict_invalid=True)
        if rc_archive != 0:
            print("[fixed64-3way-inputs-selftest] archive dir case failed", file=sys.stderr)
            return 1
        archive_doc = load_json(json_out)
        if not isinstance(archive_doc, dict) or str(archive_doc.get("status", "")) != "staged":
            print("[fixed64-3way-inputs-selftest] archive dir status mismatch", file=sys.stderr)
            return 1
        selected_source = str(archive_doc.get("selected_source", ""))
        if str(archive_new) != selected_source:
            print("[fixed64-3way-inputs-selftest] archive dir selected_source mismatch", file=sys.stderr)
            return 1
        if not staged_target.exists():
            print("[fixed64-3way-inputs-selftest] archive dir staged target missing", file=sys.stderr)
            return 1

        # 7) zip artifact case
        zip_src = base / "darwin_probe_artifact.zip"
        zip_entry = Path("artifact") / "nested" / "fixed64_cross_platform_probe_darwin.detjson"
        with zipfile.ZipFile(zip_src, "w", compression=zipfile.ZIP_DEFLATED) as zf:
            zf.writestr(str(zip_entry).replace("\\", "/"), json.dumps(make_probe("Darwin"), ensure_ascii=False))
        rc_zip = run_script(report_dir, str(zip_src), json_out, strict_invalid=True)
        if rc_zip != 0:
            print("[fixed64-3way-inputs-selftest] zip staged case failed", file=sys.stderr)
            return 1
        zip_doc = load_json(json_out)
        if not isinstance(zip_doc, dict) or str(zip_doc.get("status", "")) != "staged":
            print("[fixed64-3way-inputs-selftest] zip staged status mismatch", file=sys.stderr)
            return 1
        selected_source = str(zip_doc.get("selected_source", ""))
        if ".zip!" not in selected_source:
            print("[fixed64-3way-inputs-selftest] zip selected_source marker missing", file=sys.stderr)
            return 1
        if not staged_target.exists():
            print("[fixed64-3way-inputs-selftest] zip staged target missing", file=sys.stderr)
            return 1

        # 8) darwin artifact summary case (relative probe_report path)
        summary_probe_dir = base / "artifact_custom_out"
        summary_probe = summary_probe_dir / "fixed64_cross_platform_probe_darwin.detjson"
        write_json(summary_probe, make_probe("Darwin"))
        summary_src = base / "fixed64_darwin_probe_artifact.detjson"
        relative_probe = summary_probe.relative_to(summary_src.parent)
        write_json(
            summary_src,
            make_artifact_summary(str(relative_probe).replace("\\", "/"), status="staged"),
        )
        staged_target.unlink(missing_ok=True)
        rc_summary = run_script(report_dir, str(summary_src), json_out, strict_invalid=True)
        if rc_summary != 0:
            print("[fixed64-3way-inputs-selftest] summary staged case failed", file=sys.stderr)
            return 1
        summary_doc = load_json(json_out)
        if not isinstance(summary_doc, dict) or str(summary_doc.get("status", "")) != "staged":
            print("[fixed64-3way-inputs-selftest] summary staged status mismatch", file=sys.stderr)
            return 1
        summary_source = str(summary_doc.get("selected_source", ""))
        if "fixed64_darwin_probe_artifact.detjson" not in summary_source:
            print("[fixed64-3way-inputs-selftest] summary selected_source marker missing", file=sys.stderr)
            return 1
        if not staged_target.exists():
            print("[fixed64-3way-inputs-selftest] summary staged target missing", file=sys.stderr)
            return 1

        # 9) summary fallback case (probe_report missing, zip.path provided)
        summary_zip = base / "artifact_summary_only.zip"
        zip_entry = "fixed64_cross_platform_probe_darwin.detjson"
        with zipfile.ZipFile(summary_zip, "w", compression=zipfile.ZIP_DEFLATED) as zf:
            zf.writestr(zip_entry, json.dumps(make_probe("Darwin"), ensure_ascii=False))
        summary_zip_src = base / "fixed64_darwin_probe_artifact_zip_only.detjson"
        relative_zip = summary_zip.relative_to(summary_zip_src.parent)
        write_json(
            summary_zip_src,
            make_artifact_summary(
                probe_report="missing_probe.detjson",
                status="staged",
                zip_path=str(relative_zip).replace("\\", "/"),
            ),
        )
        staged_target.unlink(missing_ok=True)
        rc_summary_zip = run_script(report_dir, str(summary_zip_src), json_out, strict_invalid=True)
        if rc_summary_zip != 0:
            print("[fixed64-3way-inputs-selftest] summary zip fallback case failed", file=sys.stderr)
            return 1
        summary_zip_doc = load_json(json_out)
        if not isinstance(summary_zip_doc, dict) or str(summary_zip_doc.get("status", "")) != "staged":
            print("[fixed64-3way-inputs-selftest] summary zip fallback status mismatch", file=sys.stderr)
            return 1
        summary_zip_source = str(summary_zip_doc.get("selected_source", ""))
        if ".zip!" not in summary_zip_source:
            print("[fixed64-3way-inputs-selftest] summary zip fallback selected_source missing zip marker", file=sys.stderr)
            return 1
        if not staged_target.exists():
            print("[fixed64-3way-inputs-selftest] summary zip fallback staged target missing", file=sys.stderr)
            return 1

        # 10) implicit default candidate should include fixed64_darwin_probe_artifact.zip
        default_zip = report_dir / "fixed64_darwin_probe_artifact.zip"
        with zipfile.ZipFile(default_zip, "w", compression=zipfile.ZIP_DEFLATED) as zf:
            zf.writestr("nested/fixed64_cross_platform_probe_darwin.detjson", json.dumps(make_probe("Darwin"), ensure_ascii=False))
        staged_target.unlink(missing_ok=True)
        rc_default_zip = run_script(report_dir, "", json_out, strict_invalid=True)
        if rc_default_zip != 0:
            print("[fixed64-3way-inputs-selftest] implicit default zip candidate case failed", file=sys.stderr)
            return 1
        default_zip_doc = load_json(json_out)
        if not isinstance(default_zip_doc, dict) or str(default_zip_doc.get("status", "")) != "staged":
            print("[fixed64-3way-inputs-selftest] implicit default zip candidate status mismatch", file=sys.stderr)
            return 1
        default_zip_source = str(default_zip_doc.get("selected_source", ""))
        if ".zip!" not in default_zip_source:
            print("[fixed64-3way-inputs-selftest] implicit default zip candidate selected_source missing zip marker", file=sys.stderr)
            return 1
        if not staged_target.exists():
            print("[fixed64-3way-inputs-selftest] implicit default zip candidate staged target missing", file=sys.stderr)
            return 1

        # 11) strict-invalid + invalid 선행 + valid 후행 explicit candidates => staged pass
        invalid_then_valid_out = report_dir / "inputs_invalid_then_valid.detjson"
        invalid_then_valid_cmd = [
            sys.executable,
            "tools/scripts/resolve_fixed64_threeway_inputs.py",
            "--report-dir",
            str(report_dir),
            "--json-out",
            str(invalid_then_valid_out),
            "--strict-invalid",
        ]
        invalid_candidate = base / "invalid_then_valid.synthetic.detjson"
        valid_candidate = base / "invalid_then_valid.valid.detjson"
        write_json(invalid_candidate, make_probe("Darwin", synthetic=True))
        write_json(valid_candidate, make_probe("Darwin"))
        invalid_then_valid_cmd.extend(["--candidate", str(invalid_candidate)])
        invalid_then_valid_cmd.extend(["--candidate", str(valid_candidate)])
        proc_invalid_then_valid = subprocess.run(
            invalid_then_valid_cmd,
            cwd=ROOT,
            env={**os.environ, "DDN_ENABLE_DARWIN_PROBE": "1", "DDN_DARWIN_PROBE_REPORT": ""},
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if int(proc_invalid_then_valid.returncode) != 0:
            print("[fixed64-3way-inputs-selftest] invalid+valid strict case should pass", file=sys.stderr)
            return 1
        invalid_then_valid_doc = load_json(invalid_then_valid_out)
        if not isinstance(invalid_then_valid_doc, dict):
            print("[fixed64-3way-inputs-selftest] invalid+valid strict report invalid", file=sys.stderr)
            return 1
        if str(invalid_then_valid_doc.get("status", "")) != "staged" or not bool(invalid_then_valid_doc.get("ok", False)):
            print("[fixed64-3way-inputs-selftest] invalid+valid strict status mismatch", file=sys.stderr)
            return 1
        invalid_hits = invalid_then_valid_doc.get("invalid_hits")
        if not isinstance(invalid_hits, list) or len(invalid_hits) == 0:
            print("[fixed64-3way-inputs-selftest] invalid+valid strict invalid_hits missing", file=sys.stderr)
            return 1
        selected_source = str(invalid_then_valid_doc.get("selected_source", ""))
        if selected_source != str(valid_candidate):
            print("[fixed64-3way-inputs-selftest] invalid+valid strict selected_source mismatch", file=sys.stderr)
            return 1

    print("[fixed64-3way-inputs-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
