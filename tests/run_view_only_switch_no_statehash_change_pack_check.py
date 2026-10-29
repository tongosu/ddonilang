#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "view_only_switch_no_statehash_change_v1"


def fail(detail: str) -> int:
    print(f"check=view_only_switch_no_statehash_change_pack detail={detail}")
    return 1


def main() -> int:
    required = [PACK / "README.md", PACK / "contract.detjson", PACK / "input.ddn", PACK / "golden.jsonl"]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.seamgrim_view_only_state_hash_invariant.pack.contract.v1":
        return fail("schema")
    proc = subprocess.run(
        [sys.executable, "tests/run_seamgrim_view_only_state_hash_invariant_check.py"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=300,
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or f"returncode={proc.returncode}"
        return fail(detail)
    print("view_only switch/state_hash invariant pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

