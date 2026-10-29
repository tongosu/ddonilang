#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "docs" / "context" / "reports" / "GAJI_METADATA_3_APPLIED_STATUS_V1_20260709.md"
INDEX = ROOT / "docs" / "context" / "reports" / "INDEX.md"

EXPECTED = {
    "gaji/std_grid": {
        "path": ROOT / "gaji" / "std_grid" / "gaji.toml",
        "name": "표준 격자 가지",
        "version": "0.1.0",
        "ssot_requires": ">=20.1.22",
        "det_tier": "D-STRICT",
        "openness": "closed",
        "description": "정수 격자 world/state helper",
    },
    "gaji/std_input_map": {
        "path": ROOT / "gaji" / "std_input_map" / "gaji.toml",
        "name": "표준 입력사상 가지",
        "version": "0.1.0",
        "ssot_requires": ">=20.1.22",
        "det_tier": "D-STRICT",
        "openness": "closed",
        "description": "InputSnapshot 기반 입력 해석 helper",
    },
    "gaji/std_physics_1d": {
        "path": ROOT / "gaji" / "std_physics_1d" / "gaji.toml",
        "name": "표준 1D 물리 가지",
        "version": "0.1.0",
        "ssot_requires": ">=20.2.26",
        "det_tier": "D-STRICT",
        "openness": "closed",
        "description": "1차원 운동/충돌 helper",
    },
}


def fail(message: str) -> int:
    print(f"gaji_metadata_3_applied_status_check: FAIL: {message}", file=sys.stderr)
    return 1


def parse_toml_subset(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except FileNotFoundError as exc:
        raise ValueError(f"missing metadata file: {path}") from exc
    for line in lines:
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if "=" not in stripped:
            raise ValueError(f"{path}: unsupported line: {line}")
        key, raw_value = stripped.split("=", 1)
        key = key.strip()
        raw_value = raw_value.strip()
        if not (raw_value.startswith('"') and raw_value.endswith('"')):
            raise ValueError(f"{path}: value must be quoted for {key}")
        values[key] = raw_value[1:-1]
    return values


def git_output(args: list[str]) -> str:
    proc = subprocess.run(
        ["git", *args],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=30,
    )
    if proc.returncode != 0:
        raise ValueError(proc.stderr.strip() or proc.stdout.strip())
    return proc.stdout


def assert_tracked(paths: list[Path]) -> None:
    rels = [str(path.relative_to(ROOT)).replace("\\", "/") for path in paths]
    output = git_output(["ls-files", *rels])
    tracked = {line.strip().replace("\\", "/") for line in output.splitlines() if line.strip()}
    missing = sorted(set(rels) - tracked)
    if missing:
        raise ValueError(f"metadata files are not tracked: {missing}")


def main() -> int:
    try:
        rows: dict[str, Any] = {}
        paths = [item["path"] for item in EXPECTED.values()]
        assert_tracked(paths)

        for package_id, expected in EXPECTED.items():
            path = expected["path"]
            actual = parse_toml_subset(path)
            expected_values = {key: value for key, value in expected.items() if key != "path"}
            expected_values["id"] = package_id
            if actual != expected_values:
                raise ValueError(
                    f"{package_id}: metadata mismatch\nexpected={json.dumps(expected_values, ensure_ascii=False, sort_keys=True)}\nactual={json.dumps(actual, ensure_ascii=False, sort_keys=True)}"
                )
            rows[package_id] = {
                "path": str(path.relative_to(ROOT)).replace("\\", "/"),
                "metadata": actual,
            }

        log = git_output(
            [
                "log",
                "--oneline",
                "--",
                "gaji/std_grid/gaji.toml",
                "gaji/std_input_map/gaji.toml",
                "gaji/std_physics_1d/gaji.toml",
            ]
        )
        if "9d94496 [LHF-E] apply approved gaji metadata" not in log:
            raise ValueError("expected applying commit not found in gaji metadata history")

        report_text = REPORT.read_text(encoding="utf-8")
        required_report_tokens = [
            "9d94496 [LHF-E] apply approved gaji metadata",
            "docs/context/reports/GAJI_REGISTRY_CLOSURE_V1.md:73",
            "docs/context/reports/GAJI_REGISTRY_CLOSURE_V1.md:94",
            "docs/context/proposals/PROPOSAL_GAJI_MISSING_METADATA_V1_20260706.md:92",
            "gaji/std_grid/gaji.toml:1",
            "gaji/std_input_map/gaji.toml:1",
            "gaji/std_physics_1d/gaji.toml:1",
        ]
        for token in required_report_tokens:
            if token not in report_text:
                raise ValueError(f"report missing token: {token}")

        index_text = INDEX.read_text(encoding="utf-8")
        if "GAJI_METADATA_3_APPLIED_STATUS_V1_20260709.md" not in index_text:
            raise ValueError("docs/context/reports/INDEX.md missing report entry")

        docs_ssot_status = git_output(["status", "--short", "--", "docs/ssot"]).strip()
        if docs_ssot_status:
            raise ValueError(f"docs/ssot changed: {docs_ssot_status}")
    except ValueError as exc:
        return fail(str(exc))

    print("gaji_metadata_3_applied_status_check: PASS")
    print("packages=3")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
