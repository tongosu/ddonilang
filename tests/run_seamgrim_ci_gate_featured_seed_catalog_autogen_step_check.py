#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


REQUIRED_TOKENS = [
    '"featured_seed_catalog_autogen"',
    "tests/run_seamgrim_featured_seed_catalog_autogen_check.py",
]


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    target = root / "tests" / "run_seamgrim_ci_gate.py"
    if not target.exists():
        print(f"missing target: {target}")
        return 1
    text = target.read_text(encoding="utf-8")

    missing = [token for token in REQUIRED_TOKENS if token not in text]
    if missing:
        print("seamgrim ci gate featured seed catalog autogen step check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1

    print("seamgrim ci gate featured seed catalog autogen step check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
