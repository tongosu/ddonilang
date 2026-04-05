#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path

from _dialect_alias_collision_inventory import KNOWN_NON_KO_COLLISIONS, build_inventory_report


ROOT = Path(__file__).resolve().parent.parent
README_PATH = ROOT / "tests" / "dialect_alias_collision_contract" / "README.md"

README_SNIPPETS = (
    "## Stable Report Surface",
    "`ddn.dialect_alias_collision_inventory.v1`",
    "`ko_collision_count`",
    "`non_ko_scope_count`",
    "`non_ko_collision_count`",
    "`known_inventory_match`",
    "`python tests/run_dialect_alias_collision_inventory_report_selftest.py`",
    "`build/tmp/dialect_alias_collision_inventory_report.detjson`",
)


def fail(message: str) -> int:
    print(f"[dialect-alias-collision-inventory-report-selftest] fail: {message}")
    return 1


def ensure_snippets() -> None:
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            raise ValueError(f"missing snippet in {README_PATH}: {snippet}")


def main() -> int:
    try:
        ensure_snippets()
        payload = build_inventory_report()
        if payload["ko_collision_count"] != 0:
            raise ValueError(f"unexpected ko collision count: {payload['ko_collision_count']}")
        if payload["known_inventory_match"] is not True:
            raise ValueError("known non-ko collision inventory mismatch")
        if payload["non_ko_scope_count"] != len(KNOWN_NON_KO_COLLISIONS):
            raise ValueError(f"unexpected non-ko scope count: {payload['non_ko_scope_count']}")
        expected_collision_count = sum(len(mapping) for mapping in KNOWN_NON_KO_COLLISIONS.values())
        if payload["non_ko_collision_count"] != expected_collision_count:
            raise ValueError(
                f"unexpected non-ko collision count: {payload['non_ko_collision_count']} != {expected_collision_count}"
            )
        with tempfile.TemporaryDirectory(prefix="dialect_alias_collision_inventory_report_") as tmp:
            out_path = Path(tmp) / "dialect_alias_collision_inventory_report.detjson"
            proc = subprocess.run(
                [
                    "python",
                    "tests/run_dialect_alias_collision_inventory_report.py",
                    "--out",
                    str(out_path),
                ],
                cwd=ROOT,
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace",
            )
            if proc.returncode != 0:
                raise ValueError(proc.stderr.strip() or proc.stdout.strip() or "runner failed")
            loaded = json.loads(out_path.read_text(encoding="utf-8"))
            if loaded != payload:
                raise ValueError("inventory report roundtrip mismatch")
    except ValueError as exc:
        return fail(str(exc))

    print(
        "[dialect-alias-collision-inventory-report-selftest] ok "
        f"non_ko_scopes={len(KNOWN_NON_KO_COLLISIONS)} "
        f"non_ko_collisions={sum(len(mapping) for mapping in KNOWN_NON_KO_COLLISIONS.values())}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
