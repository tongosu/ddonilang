#!/usr/bin/env python
from __future__ import annotations

import json
import platform
import subprocess
import sys
import tempfile
import zipfile
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCHEMA = "ddn.fixed64.darwin_probe_artifact.v1"
PROBE_SCHEMA = "ddn.fixed64.cross_platform_probe.v1"
VECTOR_SCHEMA = "ddn.fixed64.determinism_vector.v1"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def make_probe(*, synthetic: bool = False) -> dict:
    release = "selftest" if synthetic else "23.6.0"
    version = "selftest" if synthetic else "Darwin Kernel Version 23.6.0"
    machine = "x86_64"
    return {
        "schema": PROBE_SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": True,
        "errors": [],
        "platform": {
            "system": "Darwin",
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


def run_case(
    summary: Path,
    report_dir: Path,
    require_darwin: bool,
    *,
    probe_report_in: Path | None = None,
    archive_keep: int | None = None,
) -> int:
    cmd = [
        sys.executable,
        "tests/run_fixed64_darwin_probe_artifact.py",
        "--report-dir",
        str(report_dir),
        "--json-out",
        str(summary),
    ]
    if probe_report_in is not None:
        cmd.extend(["--probe-report-in", str(probe_report_in)])
    if archive_keep is not None:
        cmd.extend(["--archive-keep", str(int(archive_keep))])
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
    host = platform.system().strip().lower()
    with tempfile.TemporaryDirectory(prefix="fixed64_darwin_probe_artifact_selftest_") as tmp:
        root = Path(tmp)
        report_dir = root / "reports"
        summary = report_dir / "darwin_probe_summary.detjson"

        rc_normal = run_case(summary, report_dir, require_darwin=False)
        if rc_normal != 0:
            print("[fixed64-darwin-probe-selftest] normal case failed", file=sys.stderr)
            return 1
        normal_doc = load_json(summary)
        if not isinstance(normal_doc, dict) or str(normal_doc.get("schema", "")) != SCHEMA:
            print("[fixed64-darwin-probe-selftest] normal summary invalid", file=sys.stderr)
            return 1
        normal_status = str(normal_doc.get("status", ""))
        if host == "darwin":
            if normal_status != "staged":
                print("[fixed64-darwin-probe-selftest] expected staged on darwin", file=sys.stderr)
                return 1
        else:
            if normal_status != "skip_non_darwin":
                print("[fixed64-darwin-probe-selftest] expected skip_non_darwin on non-darwin", file=sys.stderr)
                return 1

        rc_require = run_case(summary, report_dir, require_darwin=True)
        require_doc = load_json(summary)
        if not isinstance(require_doc, dict):
            print("[fixed64-darwin-probe-selftest] require summary invalid", file=sys.stderr)
            return 1
        require_status = str(require_doc.get("status", ""))
        if host == "darwin":
            if rc_require != 0 or require_status != "staged":
                print("[fixed64-darwin-probe-selftest] require case mismatch on darwin", file=sys.stderr)
                return 1
        else:
            if rc_require == 0 or require_status != "fail_non_darwin":
                print("[fixed64-darwin-probe-selftest] require case mismatch on non-darwin", file=sys.stderr)
                return 1

        # probe-report-in + archive keep regression (works on all hosts)
        probe_src_1 = root / "probe_input_1.detjson"
        probe_src_2 = root / "probe_input_2.detjson"
        write_json(probe_src_1, make_probe())
        write_json(probe_src_2, make_probe())
        rc_input_1 = run_case(summary, report_dir, require_darwin=False, probe_report_in=probe_src_1, archive_keep=1)
        if rc_input_1 != 0:
            print("[fixed64-darwin-probe-selftest] probe-report-in case#1 failed", file=sys.stderr)
            return 1
        input_doc_1 = load_json(summary)
        if not isinstance(input_doc_1, dict) or str(input_doc_1.get("status", "")) != "staged":
            print("[fixed64-darwin-probe-selftest] probe-report-in case#1 status mismatch", file=sys.stderr)
            return 1
        zip_doc_1 = input_doc_1.get("zip")
        if not isinstance(zip_doc_1, dict) or str(zip_doc_1.get("status", "")) != "staged":
            print("[fixed64-darwin-probe-selftest] probe-report-in case#1 zip status mismatch", file=sys.stderr)
            return 1
        zip_artifact = report_dir / "fixed64_darwin_probe_artifact.zip"
        if not zip_artifact.exists():
            print("[fixed64-darwin-probe-selftest] probe-report-in case#1 zip artifact missing", file=sys.stderr)
            return 1
        try:
            with zipfile.ZipFile(zip_artifact, "r") as zf:
                names = set(zf.namelist())
                if "fixed64_cross_platform_probe_darwin.detjson" not in names:
                    print("[fixed64-darwin-probe-selftest] zip entry missing", file=sys.stderr)
                    return 1
                probe_in_zip = json.loads(zf.read("fixed64_cross_platform_probe_darwin.detjson").decode("utf-8"))
        except Exception:
            print("[fixed64-darwin-probe-selftest] zip artifact read failed", file=sys.stderr)
            return 1
        if not isinstance(probe_in_zip, dict) or str(probe_in_zip.get("schema", "")) != PROBE_SCHEMA:
            print("[fixed64-darwin-probe-selftest] zip entry schema mismatch", file=sys.stderr)
            return 1
        archive_doc_1 = input_doc_1.get("archive")
        if not isinstance(archive_doc_1, dict) or str(archive_doc_1.get("status", "")) != "staged":
            print("[fixed64-darwin-probe-selftest] probe-report-in case#1 archive mismatch", file=sys.stderr)
            return 1
        rc_input_2 = run_case(summary, report_dir, require_darwin=False, probe_report_in=probe_src_2, archive_keep=1)
        if rc_input_2 != 0:
            print("[fixed64-darwin-probe-selftest] probe-report-in case#2 failed", file=sys.stderr)
            return 1
        archive_dir = report_dir / "darwin_probe_archive"
        archive_files = sorted(archive_dir.glob("fixed64_cross_platform_probe_darwin*.detjson"))
        if len(archive_files) != 1:
            print("[fixed64-darwin-probe-selftest] archive keep/prune mismatch", file=sys.stderr)
            return 1

        # synthetic platform values must be rejected.
        probe_src_bad = root / "probe_input_synthetic.detjson"
        write_json(probe_src_bad, make_probe(synthetic=True))
        rc_input_bad = run_case(summary, report_dir, require_darwin=False, probe_report_in=probe_src_bad)
        if rc_input_bad == 0:
            print("[fixed64-darwin-probe-selftest] synthetic probe should fail", file=sys.stderr)
            return 1
        bad_doc = load_json(summary)
        if not isinstance(bad_doc, dict):
            print("[fixed64-darwin-probe-selftest] synthetic summary invalid", file=sys.stderr)
            return 1
        bad_reason = str(bad_doc.get("reason", ""))
        if "looks synthetic" not in bad_reason:
            print("[fixed64-darwin-probe-selftest] synthetic fail reason mismatch", file=sys.stderr)
            return 1

    print("[fixed64-darwin-probe-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
