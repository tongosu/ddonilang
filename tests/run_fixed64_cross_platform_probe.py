#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import platform
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VECTOR_SCHEMA = "ddn.fixed64.determinism_vector.v1"
REPORT_SCHEMA = "ddn.fixed64.cross_platform_probe.v1"


def resolve_build_root() -> Path:
    system = platform.system().lower()
    if system not in {"windows", "win32"}:
        local = ROOT / "build"
        local.mkdir(parents=True, exist_ok=True)
        return local
    preferred = Path("I:/home/urihanl/ddn/codex/build")
    try:
        preferred.mkdir(parents=True, exist_ok=True)
        return preferred
    except OSError as exc:
        raise RuntimeError(f"build root 경로를 만들 수 없습니다: {preferred}") from exc


def parse_kv_lines(text: str) -> dict[str, str]:
    out: dict[str, str] = {}
    for raw in text.splitlines():
        line = raw.strip()
        if not line or "=" not in line:
            continue
        key, value = line.split("=", 1)
        out[key.strip()] = value.strip()
    return out


def parse_csv_i64(text: str) -> list[int]:
    values: list[int] = []
    for token in text.split(","):
        item = token.strip()
        if not item:
            continue
        values.append(int(item, 10))
    return values


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def resolve_linux_target_dir() -> Path:
    home = Path.home()
    return (home / ".cache" / "ddn-cargo-target" / "fixed64-probe").resolve()


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Fixed64 raw_i64 결정성 벡터를 실행/기록하고 OS 간 결과를 비교한다."
    )
    parser.add_argument("--cargo", default="cargo", help="cargo 실행 파일 경로")
    parser.add_argument(
        "--report-out",
        default="",
        help="detjson 출력 경로(기본: build/reports/fixed64_cross_platform_probe.detjson)",
    )
    parser.add_argument(
        "--compare-report",
        default="",
        help="비교할 이전 probe detjson 경로(선택)",
    )
    args = parser.parse_args()

    build_root = resolve_build_root()
    report_out = (
        Path(args.report_out).resolve()
        if args.report_out.strip()
        else build_root / "reports" / "fixed64_cross_platform_probe.detjson"
    )
    report_out.parent.mkdir(parents=True, exist_ok=True)

    cmd = [args.cargo, "run", "-q", "-p", "ddonirang-core", "--example", "fixed64_determinism_vector"]
    env = os.environ.copy()
    if platform.system().lower() == "linux":
        raw_target = env.get("CARGO_TARGET_DIR", "").strip()
        if not raw_target or ":" in raw_target:
            target_dir = resolve_linux_target_dir()
            target_dir.mkdir(parents=True, exist_ok=True)
            env["CARGO_TARGET_DIR"] = str(target_dir)
    try:
        proc = subprocess.run(
            cmd,
            cwd=ROOT,
            env=env,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
    except OSError as exc:
        print(f"[fixed64-probe] 실행 실패: {exc}", file=sys.stderr)
        return 1

    rows = parse_kv_lines(proc.stdout or "")
    ok = proc.returncode == 0
    errors: list[str] = []

    schema = rows.get("schema", "")
    status = rows.get("status", "")
    blake3_hex = rows.get("blake3", "")
    raw_csv = rows.get("raw_i64", "")
    expected_csv = rows.get("expected_raw_i64", "")

    if schema != VECTOR_SCHEMA:
        ok = False
        errors.append(f"schema mismatch: {schema}")
    if status != "pass":
        ok = False
        errors.append(f"status mismatch: {status}")
    if not blake3_hex:
        ok = False
        errors.append("missing blake3")

    try:
        raw_values = parse_csv_i64(raw_csv)
    except Exception:
        raw_values = []
        ok = False
        errors.append("raw_i64 parse failed")
    try:
        expected_values = parse_csv_i64(expected_csv)
    except Exception:
        expected_values = []
        ok = False
        errors.append("expected_raw_i64 parse failed")

    if raw_values and expected_values and raw_values != expected_values:
        ok = False
        errors.append("raw_i64 != expected_raw_i64")

    compare_payload: dict[str, object] = {}
    if args.compare_report.strip():
        compare_path = Path(args.compare_report).resolve()
        compare_doc = load_json(compare_path)
        if not compare_doc:
            ok = False
            errors.append(f"compare report load failed: {compare_path}")
            compare_payload = {"path": str(compare_path), "ok": False}
        else:
            old_probe = compare_doc.get("probe")
            old_blake3 = ""
            old_raw: list[int] = []
            if isinstance(old_probe, dict):
                old_blake3 = str(old_probe.get("blake3", ""))
                old_raw_val = old_probe.get("raw_i64")
                if isinstance(old_raw_val, list):
                    try:
                        old_raw = [int(item) for item in old_raw_val]
                    except Exception:
                        old_raw = []
            same_digest = bool(old_blake3) and old_blake3 == blake3_hex
            same_raw = bool(old_raw) and old_raw == raw_values
            compare_ok = same_digest and same_raw
            if not compare_ok:
                ok = False
                errors.append("cross-platform compare mismatch")
            compare_payload = {
                "path": str(compare_path),
                "ok": compare_ok,
                "same_blake3": same_digest,
                "same_raw_i64": same_raw,
                "other_blake3": old_blake3,
            }

    report = {
        "schema": REPORT_SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": ok,
        "errors": errors,
        "platform": {
            "system": platform.system(),
            "release": platform.release(),
            "version": platform.version(),
            "machine": platform.machine(),
            "python": platform.python_version(),
        },
        "cmd": cmd,
        "returncode": int(proc.returncode),
        "probe": {
            "schema": schema,
            "status": status,
            "blake3": blake3_hex,
            "raw_i64": raw_values,
            "expected_raw_i64": expected_values,
        },
        "compare": compare_payload,
        "stdout": (proc.stdout or "").strip().splitlines(),
        "stderr": (proc.stderr or "").strip().splitlines(),
    }
    report_out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    if ok:
        print(
            f"[fixed64-probe] ok blake3={blake3_hex} count={len(raw_values)} "
            f"report={report_out}"
        )
        return 0

    print(f"[fixed64-probe] failed report={report_out}", file=sys.stderr)
    for item in errors:
        print(f" - {item}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
