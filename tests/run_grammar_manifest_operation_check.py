#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path


EXPECTED_SCHEMA = "ddn.grammar_manifest_operation_check.v1"


def run_cmd(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )


def fail(detail: str) -> int:
    print(f"check=grammar_manifest_operation_runner detail={detail}")
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run grammar manifest operational checker contract"
    )
    parser.add_argument("--repo-root", default=".", help="repository root path")
    args = parser.parse_args()

    root = Path(args.repo_root).resolve()
    tool = root / "tools" / "scripts" / "check_grammar_manifest_operation.py"
    if not tool.exists():
        return fail(f"tool_missing:{tool}")

    with tempfile.TemporaryDirectory(prefix="grammar_manifest_operation_check_") as temp_dir:
        temp_root = Path(temp_dir)
        report_path = temp_root / "report.detjson"
        table_path = temp_root / "table.md"

        default_cmd = [
            sys.executable,
            str(tool),
            "--repo-root",
            str(root),
            "--report",
            str(report_path),
            "--table-out",
            str(table_path),
        ]
        default_proc = run_cmd(default_cmd, cwd=root)
        if default_proc.returncode != 0:
            detail = default_proc.stderr.strip() or default_proc.stdout.strip() or "default_failed"
            return fail(f"default_failed:{detail}")

        if not report_path.exists():
            return fail("report_missing")
        if not table_path.exists():
            return fail("table_missing")

        report = json.loads(report_path.read_text(encoding="utf-8"))
        if report.get("schema") != EXPECTED_SCHEMA:
            return fail(f"schema_mismatch:{report.get('schema')}")
        if "runtime_status_allowed" not in report:
            return fail("runtime_status_allowed_missing")
        allowed = set(report.get("runtime_status_allowed", []))
        if allowed != {"implemented", "partial", "designed", "reserved"}:
            return fail(f"runtime_status_allowed_mismatch:{sorted(allowed)}")

        table_text = table_path.read_text(encoding="utf-8")
        if "| runtime_status |" not in table_text:
            return fail("table_contract_missing_runtime_status_column")

        # strict negative case: invalid runtime_status + dead pack ref
        fixture_root = temp_root / "fixture"
        (fixture_root / "pack" / "dummy_ok").mkdir(parents=True, exist_ok=True)
        (fixture_root / "pack" / "dummy_ok" / "README.md").write_text(
            "# dummy_ok\n\nevidence_tier: golden_closed\n",
            encoding="utf-8",
        )
        bad_manifest = fixture_root / "manifest.detjson"
        bad_schema = fixture_root / "schema.json"
        bad_schema.write_text(
            json.dumps({"title": "noop schema"}, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )
        bad_manifest.write_text(
            json.dumps(
                {
                    "schema": "ddn.grammar_capability_manifest.v0.1.json",
                    "version": 1,
                    "items": [
                        {
                            "id": "lang.bad.sample",
                            "canon_name": "샘플",
                            "category": "stmt",
                            "input_aliases": [],
                            "compat_only_surfaces": [],
                            "deprecated_input_surfaces": [],
                            "forbidden_surfaces": [],
                            "reserved_surfaces": [],
                            "age_scope": ["AGE0"],
                            "allowed_parents": ["top_level"],
                            "check_phase": "canon",
                            "state_hash_effect": "excluded",
                            "runtime_status": "unknown_status",
                            "pack_refs": ["missing_pack_ref"],
                            "notes": "fixture",
                        }
                    ],
                },
                ensure_ascii=False,
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )

        strict_cmd = [
            sys.executable,
            str(tool),
            "--repo-root",
            str(fixture_root),
            "--manifest-path",
            str(bad_manifest),
            "--schema-path",
            str(bad_schema),
            "--pack-root",
            str(fixture_root / "pack"),
            "--strict",
        ]
        strict_proc = run_cmd(strict_cmd, cwd=root)
        if strict_proc.returncode == 0:
            return fail("strict_negative_must_fail")
        strict_log = f"{strict_proc.stdout}\n{strict_proc.stderr}"
        if "strict_failed" not in strict_log:
            return fail(f"strict_failed_marker_missing:{strict_log}")

    print("check=grammar_manifest_operation_runner detail=ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
