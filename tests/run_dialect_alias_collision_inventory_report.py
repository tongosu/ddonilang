#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path

from _dialect_alias_collision_inventory import build_inventory_report


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--out",
        default="build/tmp/dialect_alias_collision_inventory_report.detjson",
        help="report output path",
    )
    args = parser.parse_args()

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    payload = build_inventory_report()
    out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"[dialect-alias-collision-inventory-report] wrote={out_path}")
    print(f"ko_collision_count={payload['ko_collision_count']}")
    print(f"non_ko_scope_count={payload['non_ko_scope_count']}")
    print(f"non_ko_collision_count={payload['non_ko_collision_count']}")
    print(f"known_inventory_match={str(payload['known_inventory_match']).lower()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
