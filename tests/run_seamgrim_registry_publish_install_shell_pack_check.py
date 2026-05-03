#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_registry_publish_install_shell_v1"
CHECKS = [
    "tests/run_seamgrim_package_registry_surface_check.py",
    "tests/run_seamgrim_sharing_publishing_surface_check.py",
    "tests/run_seamgrim_publication_snapshot_surface_check.py",
    "tests/run_seamgrim_platform_mock_interface_contract_check.py",
]


def fail(detail: str) -> int:
    print(f"check=seamgrim_registry_publish_install_shell_pack detail={detail}")
    return 1


def main() -> int:
    required = [PACK / "README.md", PACK / "contract.detjson", PACK / "input.ddn", PACK / "golden.jsonl"]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.seamgrim_registry_publish_install_shell.pack.contract.v1":
        return fail("schema")
    for check in CHECKS:
        proc = subprocess.run(
            [sys.executable, check],
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=300,
        )
        if proc.returncode != 0:
            detail = proc.stderr.strip() or proc.stdout.strip() or f"returncode={proc.returncode}"
            return fail(f"{check}:{detail}")
    print("seamgrim registry/publish/install shell pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

