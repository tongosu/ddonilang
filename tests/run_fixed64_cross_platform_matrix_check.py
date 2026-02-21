#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
PROBE_SCHEMA = "ddn.fixed64.cross_platform_probe.v1"
VECTOR_SCHEMA = "ddn.fixed64.determinism_vector.v1"


def load_json(path: Path) -> dict:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError(f"not object: {path}")
    return payload


def normalize_system(value: str) -> str:
    return value.strip().lower()


def parse_require_systems(value: str) -> set[str]:
    out: set[str] = set()
    for token in value.split(","):
        item = normalize_system(token)
        if item:
            out.add(item)
    return out


def gather_report_paths(args_reports: list[str]) -> list[Path]:
    if args_reports:
        rows = [Path(item).resolve() for item in args_reports]
        return rows
    report_dir = ROOT / "build" / "reports"
    rows = sorted(report_dir.glob("fixed64_cross_platform_probe*.detjson"))
    return [path.resolve() for path in rows if path.is_file()]


def main() -> int:
    parser = argparse.ArgumentParser(
        description="fixed64 cross-platform probe 보고서를 비교해 raw_i64 결정성 일치를 확인한다."
    )
    parser.add_argument(
        "--report",
        action="append",
        default=[],
        help="비교할 probe detjson 경로 (여러 번 지정 가능)",
    )
    parser.add_argument(
        "--require-systems",
        default="",
        help="필수 시스템 목록(소문자, 콤마 구분). 예: windows,linux,darwin",
    )
    args = parser.parse_args()

    paths = gather_report_paths(args.report)
    if not paths:
        print("[fixed64-matrix] no report files", file=sys.stderr)
        return 1

    required_systems = parse_require_systems(args.require_systems)
    rows: list[tuple[Path, str, str, tuple[int, ...]]] = []
    errors: list[str] = []
    seen_systems: set[str] = set()

    for path in paths:
        if not path.exists():
            errors.append(f"missing report: {path}")
            continue
        try:
            doc = load_json(path)
        except Exception as exc:
            errors.append(f"load failed: {path} ({exc})")
            continue

        if str(doc.get("schema", "")) != PROBE_SCHEMA:
            errors.append(f"schema mismatch: {path}")
            continue
        if not bool(doc.get("ok", False)):
            errors.append(f"probe not ok: {path}")
            continue

        platform_doc = doc.get("platform")
        probe_doc = doc.get("probe")
        if not isinstance(platform_doc, dict) or not isinstance(probe_doc, dict):
            errors.append(f"missing platform/probe object: {path}")
            continue

        system = normalize_system(str(platform_doc.get("system", "")))
        if not system:
            errors.append(f"missing platform.system: {path}")
            continue

        if str(probe_doc.get("schema", "")) != VECTOR_SCHEMA:
            errors.append(f"vector schema mismatch: {path}")
            continue
        if str(probe_doc.get("status", "")) != "pass":
            errors.append(f"vector status mismatch: {path}")
            continue

        blake3_hex = str(probe_doc.get("blake3", "")).strip()
        if not blake3_hex:
            errors.append(f"missing probe.blake3: {path}")
            continue

        raw = probe_doc.get("raw_i64")
        if not isinstance(raw, list) or not raw:
            errors.append(f"missing probe.raw_i64: {path}")
            continue
        try:
            raw_tuple = tuple(int(item) for item in raw)
        except Exception:
            errors.append(f"probe.raw_i64 parse failed: {path}")
            continue

        rows.append((path, system, blake3_hex, raw_tuple))
        seen_systems.add(system)

    if required_systems and not required_systems.issubset(seen_systems):
        missing = sorted(required_systems - seen_systems)
        errors.append(f"required systems missing: {','.join(missing)}")

    if rows:
        base_path, _, base_blake3, base_raw = rows[0]
        for path, _, blake3_hex, raw_tuple in rows[1:]:
            if blake3_hex != base_blake3:
                errors.append(
                    f"blake3 mismatch: base={base_path.name}({base_blake3}) vs {path.name}({blake3_hex})"
                )
            if raw_tuple != base_raw:
                errors.append(
                    f"raw_i64 mismatch: base={base_path.name} vs {path.name}"
                )

    if errors:
        print("[fixed64-matrix] failed", file=sys.stderr)
        for item in errors[:20]:
            print(f" - {item}", file=sys.stderr)
        return 1

    print(
        "[fixed64-matrix] ok "
        f"reports={len(rows)} systems={','.join(sorted(seen_systems))} "
        f"blake3={rows[0][2]}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
