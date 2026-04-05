#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
import zipfile
from datetime import datetime, timezone
from pathlib import Path


SCHEMA = "ddn.fixed64.threeway_inputs.v1"
PROBE_SCHEMA = "ddn.fixed64.cross_platform_probe.v1"
VECTOR_SCHEMA = "ddn.fixed64.determinism_vector.v1"
DARWIN_ARTIFACT_SCHEMA = "ddn.fixed64.darwin_probe_artifact.v1"
DARWIN_REPORT_NAME = "fixed64_cross_platform_probe_darwin.detjson"
SYNTHETIC_PLATFORM_TOKENS = ("selftest", "sample", "dummy", "placeholder")


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


def is_valid_darwin_probe_doc(doc: dict | None) -> tuple[bool, str]:
    if not isinstance(doc, dict):
        return False, "invalid json"
    if str(doc.get("schema", "")) != PROBE_SCHEMA:
        return False, "schema mismatch"
    if not bool(doc.get("ok", False)):
        return False, "probe ok=false"
    platform_doc = doc.get("platform")
    if not isinstance(platform_doc, dict):
        return False, "platform missing"
    system = str(platform_doc.get("system", "")).strip().lower()
    if system != "darwin":
        return False, f"platform.system={system or '-'}"
    release = str(platform_doc.get("release", "")).strip().lower()
    version = str(platform_doc.get("version", "")).strip().lower()
    machine = str(platform_doc.get("machine", "")).strip().lower()
    if not release or any(token in release for token in SYNTHETIC_PLATFORM_TOKENS):
        return False, "platform.release looks synthetic"
    if not version or any(token in version for token in SYNTHETIC_PLATFORM_TOKENS):
        return False, "platform.version looks synthetic"
    if not machine or any(token in machine for token in SYNTHETIC_PLATFORM_TOKENS):
        return False, "platform.machine looks synthetic"

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


def is_valid_darwin_probe(path: Path) -> tuple[bool, str]:
    return is_valid_darwin_probe_doc(load_json(path))


def stage_darwin_probe_from_zip(zip_path: Path, target: Path) -> tuple[bool, str, str]:
    try:
        with zipfile.ZipFile(zip_path, "r") as zf:
            entries = [
                name
                for name in zf.namelist()
                if name and not name.endswith("/") and Path(name).name == DARWIN_REPORT_NAME
            ]
            if not entries:
                return False, "zip missing darwin report", ""
            entries.sort(key=lambda item: (len(Path(item).parts), len(item), item))
            entry = entries[0]
            try:
                raw = zf.read(entry)
            except Exception:
                return False, "zip read failed", ""
    except Exception:
        return False, "zip open failed", ""

    try:
        text = raw.decode("utf-8")
    except Exception:
        return False, "zip entry utf-8 decode failed", ""
    try:
        payload = json.loads(text)
    except Exception:
        return False, "zip entry invalid json", ""

    ok, reason = is_valid_darwin_probe_doc(payload if isinstance(payload, dict) else None)
    if not ok:
        return False, f"zip entry invalid ({reason})", ""

    target.parent.mkdir(parents=True, exist_ok=True)
    if not text.endswith("\n"):
        text += "\n"
    target.write_text(text, encoding="utf-8")
    return True, "-", f"{zip_path}!{entry}"


def stage_darwin_probe_from_artifact_summary(summary_path: Path, target: Path) -> tuple[bool, str, str]:
    doc = load_json(summary_path)
    if not isinstance(doc, dict):
        return False, "summary invalid json", ""
    if str(doc.get("schema", "")) != DARWIN_ARTIFACT_SCHEMA:
        return False, "not_darwin_artifact_summary", ""
    status = str(doc.get("status", "")).strip().lower()
    if status != "staged":
        return False, f"artifact summary status={status or '-'}", ""
    errors: list[str] = []

    probe_raw = str(doc.get("probe_report", "")).strip()
    if probe_raw:
        probe_path = Path(probe_raw)
        if not probe_path.is_absolute():
            probe_path = (summary_path.parent / probe_path).resolve()
        else:
            probe_path = probe_path.resolve()
        if not probe_path.exists():
            errors.append(f"artifact probe report missing: {probe_path}")
        else:
            ok, reason = is_valid_darwin_probe(probe_path)
            if not ok:
                errors.append(f"artifact probe invalid ({reason})")
            else:
                if probe_path != target:
                    target.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(probe_path, target)
                return True, "-", f"{summary_path} -> {probe_path}"
    else:
        errors.append("artifact summary probe_report missing")

    zip_candidates: list[Path] = []
    zip_doc = doc.get("zip")
    if isinstance(zip_doc, dict):
        zip_raw = str(zip_doc.get("path", "")).strip()
        if zip_raw:
            zip_path = Path(zip_raw)
            if not zip_path.is_absolute():
                zip_path = (summary_path.parent / zip_path).resolve()
            else:
                zip_path = zip_path.resolve()
            zip_candidates.append(zip_path)
    default_zip = summary_path.parent / "fixed64_darwin_probe_artifact.zip"
    if default_zip.exists():
        zip_candidates.append(default_zip.resolve())

    seen_zip: set[str] = set()
    for zip_path in zip_candidates:
        key = str(zip_path).lower()
        if key in seen_zip:
            continue
        seen_zip.add(key)
        if not zip_path.exists():
            errors.append(f"artifact zip missing: {zip_path}")
            continue
        zipped_ok, zipped_reason, zipped_source = stage_darwin_probe_from_zip(zip_path, target)
        if not zipped_ok:
            errors.append(f"artifact zip invalid ({zipped_reason})")
            continue
        return True, "-", f"{summary_path} -> {zipped_source}"

    return False, "; ".join(errors) if errors else "artifact summary unusable", ""


def build_candidates(report_dir: Path, env_path: str, extras: list[str]) -> list[Path]:
    out: list[Path] = []
    explicit_rows: list[str] = []
    if env_path.strip():
        explicit_rows.append(env_path.strip())
    explicit_rows.extend([item.strip() for item in extras if item.strip()])

    implicit_rows: list[str] = [
        str(report_dir / DARWIN_REPORT_NAME),
        str(report_dir / "fixed64_darwin_probe_artifact.detjson"),
        str(report_dir / "fixed64_darwin_probe_artifact.zip"),
        str(report_dir / "darwin" / DARWIN_REPORT_NAME),
        str(report_dir / "darwin_probe" / DARWIN_REPORT_NAME),
        str(report_dir / "darwin_probe_archive"),
        f"build/reports/{DARWIN_REPORT_NAME}",
        "build/reports/fixed64_darwin_probe_artifact.detjson",
        "build/reports/fixed64_darwin_probe_artifact.zip",
        f"build/reports/darwin/{DARWIN_REPORT_NAME}",
        f"build/reports/darwin_probe/{DARWIN_REPORT_NAME}",
        "build/reports/darwin_probe_archive",
        f"darwin_probe/{DARWIN_REPORT_NAME}",
    ]
    # When explicit candidates are provided, do not silently fall back to
    # ambient default locations. This keeps resolution deterministic.
    raw_rows: list[str] = explicit_rows if explicit_rows else implicit_rows
    seen: set[str] = set()
    for raw in raw_rows:
        path = Path(raw).resolve()
        key = str(path).lower()
        if key in seen:
            continue
        seen.add(key)
        out.append(path)
    return out


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def default_report_dir() -> str:
    preferred = Path("I:/home/urihanl/ddn/codex/build/reports")
    if os.name == "nt":
        try:
            preferred.mkdir(parents=True, exist_ok=True)
        except OSError:
            pass
        return str(preferred)
    return "build/reports"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="fixed64 3-way 입력(darwin probe report) 후보를 찾아 보고서 디렉터리로 스테이징한다."
    )
    parser.add_argument("--report-dir", default=default_report_dir(), help="보고서 디렉터리")
    parser.add_argument(
        "--darwin-report-env",
        default="DDN_DARWIN_PROBE_REPORT",
        help="darwin report 경로를 담은 환경변수 이름",
    )
    parser.add_argument(
        "--candidate",
        action="append",
        default=[],
        help="추가 후보 경로(여러 번 지정 가능)",
    )
    parser.add_argument(
        "--json-out",
        default="",
        help="결과 detjson 경로(선택)",
    )
    parser.add_argument(
        "--strict-invalid",
        action="store_true",
        help="후보 파일이 있는데 형식이 잘못된 경우 실패",
    )
    parser.add_argument(
        "--require-when-env",
        default="",
        help="해당 환경변수 값이 1/true/on/yes면 staged 결과를 강제(require)",
    )
    args = parser.parse_args()

    report_dir = Path(args.report_dir).resolve()
    report_dir.mkdir(parents=True, exist_ok=True)
    target = (report_dir / DARWIN_REPORT_NAME).resolve()
    env_value = str(os.environ.get(args.darwin_report_env, "")).strip()
    candidates = build_candidates(report_dir, env_value, args.candidate)

    selected_source = ""
    selected_reason = "missing"
    staged = False
    invalid_hits: list[str] = []

    for candidate in candidates:
        if not candidate.exists():
            continue
        if candidate.is_dir():
            archive_rows = sorted(
                candidate.glob("fixed64_cross_platform_probe_darwin*.detjson"),
                key=lambda item: (item.name, item.stat().st_mtime),
                reverse=True,
            )
            if not archive_rows:
                continue
            picked = False
            for archive_item in archive_rows:
                ok, reason = is_valid_darwin_probe(archive_item)
                if not ok:
                    invalid_hits.append(f"{archive_item} ({reason})")
                    continue
                selected_source = str(archive_item)
                selected_reason = "found"
                if archive_item != target:
                    target.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(archive_item, target)
                staged = target.exists()
                picked = staged
                if picked:
                    break
            if picked:
                break
            continue
        if candidate.suffix.lower() == ".zip":
            zipped_ok, zipped_reason, zipped_source = stage_darwin_probe_from_zip(candidate, target)
            if not zipped_ok:
                invalid_hits.append(f"{candidate} ({zipped_reason})")
                continue
            selected_source = zipped_source
            selected_reason = "found"
            staged = target.exists()
            break
        summary_ok, summary_reason, summary_source = stage_darwin_probe_from_artifact_summary(candidate, target)
        if summary_ok:
            selected_source = summary_source
            selected_reason = "found"
            staged = target.exists()
            break
        if summary_reason not in {"not_darwin_artifact_summary"}:
            invalid_hits.append(f"{candidate} ({summary_reason})")
            continue
        ok, reason = is_valid_darwin_probe(candidate)
        if not ok:
            invalid_hits.append(f"{candidate} ({reason})")
            continue
        selected_source = str(candidate)
        selected_reason = "found"
        if candidate != target:
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(candidate, target)
        staged = target.exists()
        break

    payload = {
        "schema": SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": True,
        "target_report": str(target),
        "selected_source": selected_source,
        "status": "staged" if staged else "missing",
        "reason": selected_reason,
        "env_key": args.darwin_report_env,
        "env_value": env_value,
        "invalid_hits": invalid_hits,
        "candidates": [str(path) for path in candidates],
    }

    if invalid_hits and args.strict_invalid and not staged:
        payload["ok"] = False
        payload["status"] = "invalid"
        payload["reason"] = "invalid candidate report"

    require_env_key = args.require_when_env.strip()
    require_enabled = False
    if require_env_key:
        raw = str(os.environ.get(require_env_key, "")).strip().lower()
        require_enabled = raw in {"1", "true", "on", "yes"}
        payload["require_env_key"] = require_env_key
        payload["require_env_enabled"] = require_enabled
        if require_enabled and str(payload.get("status", "")) != "staged":
            payload["ok"] = False
            payload["status"] = "missing_required"
            payload["reason"] = f"required by env: {require_env_key}"

    if args.json_out.strip():
        write_json(Path(args.json_out).resolve(), payload)

    print(
        "[fixed64-3way-inputs] "
        f"status={payload['status']} source={selected_source or '-'} target={target}"
    )
    if invalid_hits:
        print(f"[fixed64-3way-inputs] invalid_candidates={len(invalid_hits)}")
        for row in invalid_hits[:5]:
            print(f" - {row}")

    return 0 if bool(payload.get("ok", False)) else 1


if __name__ == "__main__":
    raise SystemExit(main())
