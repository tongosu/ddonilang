#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import subprocess
import sys
import zipfile
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCHEMA = "ddn.fixed64.darwin_probe_artifact.v1"
PROBE_SCHEMA = "ddn.fixed64.cross_platform_probe.v1"
VECTOR_SCHEMA = "ddn.fixed64.determinism_vector.v1"
REPORT_NAME = "fixed64_cross_platform_probe_darwin.detjson"
SYNTHETIC_PLATFORM_TOKENS = ("selftest", "sample", "dummy", "placeholder")


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


def list_of_strings(value: object) -> list[str] | None:
    if not isinstance(value, list):
        return None
    out: list[str] = []
    for item in value:
        if not isinstance(item, str):
            return None
        text = item.strip()
        if not text:
            return None
        out.append(text)
    return out


def default_report_dir() -> str:
    preferred = Path("I:/home/urihanl/ddn/codex/build/reports")
    if os.name == "nt":
        try:
            preferred.mkdir(parents=True, exist_ok=True)
        except OSError:
            pass
        return str(preferred)
    return "build/reports"


def validate_probe_doc(doc: dict | None) -> tuple[bool, str]:
    if not isinstance(doc, dict):
        return False, "probe report invalid"
    if str(doc.get("schema", "")) != PROBE_SCHEMA:
        return False, "probe schema mismatch"
    if not bool(doc.get("ok", False)):
        return False, "probe ok=false"
    platform_doc = doc.get("platform")
    system = str(platform_doc.get("system", "")).strip().lower() if isinstance(platform_doc, dict) else ""
    if system != "darwin":
        return False, f"probe platform mismatch: {system or '-'}"
    if isinstance(platform_doc, dict):
        release = str(platform_doc.get("release", "")).strip().lower()
        version = str(platform_doc.get("version", "")).strip().lower()
        machine = str(platform_doc.get("machine", "")).strip().lower()
        if any(token in release for token in SYNTHETIC_PLATFORM_TOKENS):
            return False, "probe platform.release looks synthetic"
        if any(token in version for token in SYNTHETIC_PLATFORM_TOKENS):
            return False, "probe platform.version looks synthetic"
        if any(token in machine for token in SYNTHETIC_PLATFORM_TOKENS):
            return False, "probe platform.machine looks synthetic"

    probe_doc = doc.get("probe")
    if not isinstance(probe_doc, dict):
        return False, "probe missing"
    if str(probe_doc.get("schema", "")).strip() != VECTOR_SCHEMA:
        return False, "probe schema mismatch"
    if str(probe_doc.get("status", "")).strip() != "pass":
        return False, "probe status mismatch"
    if not str(probe_doc.get("blake3", "")).strip():
        return False, "probe blake3 missing"
    raw_i64 = probe_doc.get("raw_i64")
    expected_raw_i64 = probe_doc.get("expected_raw_i64")
    if not isinstance(raw_i64, list) or not raw_i64:
        return False, "probe raw_i64 missing"
    if not isinstance(expected_raw_i64, list) or not expected_raw_i64:
        return False, "probe expected_raw_i64 missing"
    if raw_i64 != expected_raw_i64:
        return False, "probe raw_i64 != expected_raw_i64"

    cmd_rows = list_of_strings(doc.get("cmd"))
    if cmd_rows is None or not cmd_rows:
        return False, "cmd missing or invalid"
    if "fixed64_determinism_vector" not in " ".join(cmd_rows):
        return False, "cmd missing fixed64_determinism_vector"

    try:
        returncode = int(doc.get("returncode", -1))
    except Exception:
        returncode = -1
    if returncode != 0:
        return False, f"returncode mismatch: {returncode}"

    errors_rows = doc.get("errors")
    if not isinstance(errors_rows, list):
        return False, "errors missing or invalid"
    if len(errors_rows) != 0:
        return False, "errors must be empty on pass"

    stdout_rows = list_of_strings(doc.get("stdout"))
    if stdout_rows is None or not stdout_rows:
        return False, "stdout missing or invalid"
    stdout_joined = "\n".join(stdout_rows)
    required_stdout_tokens = (
        f"schema={VECTOR_SCHEMA}",
        "status=pass",
        "blake3=",
        "raw_i64=",
        "expected_raw_i64=",
    )
    for token in required_stdout_tokens:
        if token not in stdout_joined:
            return False, f"stdout missing token: {token}"

    stderr_rows = doc.get("stderr")
    if not isinstance(stderr_rows, list):
        return False, "stderr missing or invalid"
    return True, "-"


def archive_probe_report(
    probe_path: Path,
    archive_dir: Path,
    keep: int,
) -> tuple[bool, str, str, list[str], int]:
    archive_dir.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    archive_name = f"fixed64_cross_platform_probe_darwin.{stamp}.detjson"
    archive_path = archive_dir / archive_name
    if archive_path.exists():
        suffix = datetime.now(timezone.utc).strftime("%f")
        archive_path = archive_dir / f"fixed64_cross_platform_probe_darwin.{stamp}.{suffix}.detjson"
    try:
        shutil.copy2(probe_path, archive_path)
    except Exception as exc:
        return False, f"archive copy failed: {exc}", "", [], 0

    candidates = sorted(
        archive_dir.glob("fixed64_cross_platform_probe_darwin*.detjson"),
        key=lambda item: (item.name, item.stat().st_mtime),
    )
    pruned: list[str] = []
    if keep > 0 and len(candidates) > keep:
        for stale in candidates[: len(candidates) - keep]:
            try:
                stale.unlink()
                pruned.append(str(stale))
            except Exception as exc:
                return False, f"archive prune failed: {exc}", str(archive_path), pruned, len(candidates)
        candidates = sorted(
            archive_dir.glob("fixed64_cross_platform_probe_darwin*.detjson"),
            key=lambda item: (item.name, item.stat().st_mtime),
        )
    return True, "-", str(archive_path), pruned, int(len(candidates))


def stage_probe_zip(probe_path: Path, zip_path: Path) -> tuple[bool, str]:
    try:
        zip_path.parent.mkdir(parents=True, exist_ok=True)
        with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
            zf.write(probe_path, arcname=REPORT_NAME)
    except Exception as exc:
        return False, f"zip write failed: {exc}"
    return True, "-"


def main() -> int:
    parser = argparse.ArgumentParser(description="macOS에서 fixed64 darwin probe 아티팩트를 생성한다.")
    parser.add_argument("--python", default=sys.executable, help="python executable")
    parser.add_argument("--report-dir", default=default_report_dir(), help="report root directory")
    parser.add_argument(
        "--out-dir",
        default="",
        help="darwin report output directory (default: <report-dir>/darwin_probe)",
    )
    parser.add_argument(
        "--json-out",
        default="",
        help="summary detjson path (default: <report-dir>/fixed64_darwin_probe_artifact.detjson)",
    )
    parser.add_argument(
        "--probe-report-in",
        default="",
        help="use existing darwin probe report instead of executing probe command",
    )
    parser.add_argument(
        "--archive-dir",
        default="",
        help="archive directory (default: <report-dir>/darwin_probe_archive)",
    )
    parser.add_argument(
        "--archive-keep",
        type=int,
        default=30,
        help="max archive file count to keep (0 means no pruning)",
    )
    parser.add_argument(
        "--skip-archive",
        action="store_true",
        help="skip archive copy/prune stage",
    )
    parser.add_argument(
        "--zip-out",
        default="",
        help="zip artifact output path (default: <report-dir>/fixed64_darwin_probe_artifact.zip)",
    )
    parser.add_argument(
        "--skip-zip",
        action="store_true",
        help="skip zip artifact stage",
    )
    parser.add_argument("--require-darwin", action="store_true", help="darwin host가 아니면 실패")
    args = parser.parse_args()
    if args.archive_keep < 0:
        print("[fixed64-darwin-probe] --archive-keep must be >= 0", file=sys.stderr)
        return 1

    report_dir = Path(args.report_dir).resolve()
    report_dir.mkdir(parents=True, exist_ok=True)
    out_dir = Path(args.out_dir).resolve() if args.out_dir.strip() else report_dir / "darwin_probe"
    out_dir.mkdir(parents=True, exist_ok=True)
    archive_dir = Path(args.archive_dir).resolve() if args.archive_dir.strip() else report_dir / "darwin_probe_archive"
    zip_out = (
        Path(args.zip_out).resolve()
        if args.zip_out.strip()
        else report_dir / "fixed64_darwin_probe_artifact.zip"
    )
    summary_path = (
        Path(args.json_out).resolve()
        if args.json_out.strip()
        else report_dir / "fixed64_darwin_probe_artifact.detjson"
    )
    probe_path = out_dir / REPORT_NAME

    host = platform.system().strip().lower()
    payload: dict[str, object] = {
        "schema": SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": False,
        "status": "fail",
        "reason": "-",
        "host_system": host,
        "probe_report": str(probe_path),
        "summary_report": str(summary_path),
        "probe_report_in": "",
        "source": "probe_command",
        "cmd": [],
        "returncode": -1,
        "archive": {
            "enabled": not bool(args.skip_archive),
            "dir": str(archive_dir),
            "keep": int(args.archive_keep),
            "archived_report": "",
            "pruned_reports": [],
            "kept_count": 0,
            "status": "disabled" if args.skip_archive else "pending",
            "reason": "disabled" if args.skip_archive else "-",
        },
        "zip": {
            "enabled": not bool(args.skip_zip),
            "path": str(zip_out),
            "status": "disabled" if args.skip_zip else "pending",
            "reason": "disabled" if args.skip_zip else "-",
        },
    }

    probe_report_in = Path(args.probe_report_in).resolve() if args.probe_report_in.strip() else None
    if probe_report_in is not None:
        payload["source"] = "probe_report_in"
        payload["probe_report_in"] = str(probe_report_in)
        if not probe_report_in.exists():
            payload["reason"] = "probe_report_in does not exist"
            write_json(summary_path, payload)
            print("[fixed64-darwin-probe] probe_report_in missing", file=sys.stderr)
            return 1
        probe_doc = load_json(probe_report_in)
        probe_ok, probe_reason = validate_probe_doc(probe_doc)
        if not probe_ok:
            payload["reason"] = probe_reason
            write_json(summary_path, payload)
            print(f"[fixed64-darwin-probe] {probe_reason}", file=sys.stderr)
            return 1
        try:
            shutil.copy2(probe_report_in, probe_path)
        except Exception as exc:
            payload["reason"] = f"probe_report_in copy failed: {exc}"
            write_json(summary_path, payload)
            print("[fixed64-darwin-probe] probe_report_in copy failed", file=sys.stderr)
            return 1
        payload["returncode"] = 0
    else:
        if host != "darwin":
            if args.require_darwin:
                payload["reason"] = "host is not darwin"
                payload["status"] = "fail_non_darwin"
                zip_payload = payload.get("zip")
                if isinstance(zip_payload, dict) and str(zip_payload.get("status", "")).strip() == "pending":
                    zip_payload["status"] = "skipped_non_darwin"
                    zip_payload["reason"] = "host is not darwin"
                write_json(summary_path, payload)
                print("[fixed64-darwin-probe] non-darwin host", file=sys.stderr)
                return 1
            payload["ok"] = True
            payload["status"] = "skip_non_darwin"
            payload["reason"] = "host is not darwin"
            zip_payload = payload.get("zip")
            if isinstance(zip_payload, dict) and str(zip_payload.get("status", "")).strip() == "pending":
                zip_payload["status"] = "skipped_non_darwin"
                zip_payload["reason"] = "host is not darwin"
            write_json(summary_path, payload)
            print(f"[fixed64-darwin-probe] skip host={host} summary={summary_path}")
            return 0

        cmd = [
            args.python,
            "tests/run_fixed64_cross_platform_probe.py",
            "--report-out",
            str(probe_path),
        ]
        payload["cmd"] = cmd
        proc = subprocess.run(
            cmd,
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        payload["returncode"] = int(proc.returncode)
        payload["stdout"] = (proc.stdout or "").strip().splitlines()
        payload["stderr"] = (proc.stderr or "").strip().splitlines()
        if proc.returncode != 0:
            payload["reason"] = "probe command failed"
            write_json(summary_path, payload)
            print("[fixed64-darwin-probe] probe command failed", file=sys.stderr)
            return int(proc.returncode)

    probe_doc = load_json(probe_path)
    probe_ok, probe_reason = validate_probe_doc(probe_doc)
    if not probe_ok:
        payload["reason"] = probe_reason
        write_json(summary_path, payload)
        print(f"[fixed64-darwin-probe] {probe_reason}", file=sys.stderr)
        return 1

    if not args.skip_archive:
        archive_ok, archive_reason, archived_report, pruned_reports, kept_count = archive_probe_report(
            probe_path,
            archive_dir,
            int(args.archive_keep),
        )
        archive_payload = payload.get("archive")
        if isinstance(archive_payload, dict):
            archive_payload["archived_report"] = archived_report
            archive_payload["pruned_reports"] = pruned_reports
            archive_payload["kept_count"] = kept_count
            archive_payload["status"] = "staged" if archive_ok else "fail"
            archive_payload["reason"] = archive_reason
        if not archive_ok:
            payload["reason"] = "archive stage failed"
            write_json(summary_path, payload)
            print("[fixed64-darwin-probe] archive stage failed", file=sys.stderr)
            return 1

    if not args.skip_zip:
        zip_ok, zip_reason = stage_probe_zip(probe_path, zip_out)
        zip_payload = payload.get("zip")
        if isinstance(zip_payload, dict):
            zip_payload["status"] = "staged" if zip_ok else "fail"
            zip_payload["reason"] = zip_reason
        if not zip_ok:
            payload["reason"] = "zip stage failed"
            write_json(summary_path, payload)
            print("[fixed64-darwin-probe] zip stage failed", file=sys.stderr)
            return 1

    payload["ok"] = True
    payload["status"] = "staged"
    payload["reason"] = "-"
    write_json(summary_path, payload)
    print(f"[fixed64-darwin-probe] ok report={probe_path} summary={summary_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
