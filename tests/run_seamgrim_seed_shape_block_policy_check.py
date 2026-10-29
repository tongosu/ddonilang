#!/usr/bin/env python
from __future__ import annotations

import argparse
from pathlib import Path

from _seamgrim_visual_contract_lib import run_seed_shape_policy


def fail(detail: str) -> int:
    print(f"check=seed_shape_policy detail={detail}")
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Legacy seed shape policy wrapper")
    parser.add_argument(
        "--compat",
        action="store_true",
        help="suppress deprecation note for legacy wrapper usage",
    )
    args = parser.parse_args()

    if not args.compat:
        print(
            "[compat-note] run_seamgrim_seed_shape_block_policy_check.py is legacy; "
            "prefer `python tests/run_seamgrim_visual_contract_check.py --scope seed`."
        )

    root = Path(__file__).resolve().parent.parent
    detail, checked = run_seed_shape_policy(root)
    if detail:
        return fail(detail)

    print(f"seamgrim seed shape block policy check ok count={checked}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
