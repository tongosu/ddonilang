#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path


SCHEMA = "ddn.fixed64.threeway_inputs.v1"
PROBE_SCHEMA = "ddn.fixed64.cross_platform_probe.v1"
DARWIN_REPORT_NAME = "fixed64_cross_platform_probe_darwin.detjson"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def is_valid_darwin_probe(path: Path) -> tuple[bool, str]:
    doc = load_json(path)
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
    return True, "-"


def build_candidates(report_dir: Path, env_path: str, extras: list[str]) -> list[Path]:
    out: list[Path] = []
    raw_rows: list[str] = []
    if env_path.strip():
        raw_rows.append(env_path.strip())
    raw_rows.extend(
        [
            str(report_dir / DARWIN_REPORT_NAME),
            str(report_dir / "darwin" / DARWIN_REPORT_NAME),
            str(report_dir / "darwin_probe" / DARWIN_REPORT_NAME),
            f"build/reports/{DARWIN_REPORT_NAME}",
            f"build/reports/darwin/{DARWIN_REPORT_NAME}",
            f"build/reports/darwin_probe/{DARWIN_REPORT_NAME}",
            f"darwin_probe/{DARWIN_REPORT_NAME}",
        ]
    )
    raw_rows.extend([item.strip() for item in extras if item.strip()])
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


def main() -> int:
    parser = argparse.ArgumentParser(
        description="fixed64 3-way 입력(darwin probe report) 후보를 찾아 build/reports로 스테이징한다."
    )
    parser.add_argument("--report-dir", default="build/reports", help="보고서 디렉터리")
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

    if invalid_hits and args.strict_invalid:
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
