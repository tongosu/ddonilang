#!/usr/bin/env python
from __future__ import annotations

import argparse
from pathlib import Path

from _seamgrim_visual_contract_lib import run_rewrite_core_shape_policy, run_seed_shape_policy


def fail(detail: str) -> int:
    print(f"check=visual_contract detail={detail}")
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Run seamgrim visual contract checks")
    parser.add_argument(
        "--scope",
        choices=("all", "rewrite", "seed"),
        default="all",
        help="contract scope to run (default: all)",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    rewrite_detail = None
    rewrite_count = 0
    seed_detail = None
    seed_count = 0

    if args.scope in ("all", "rewrite"):
        rewrite_detail, rewrite_count = run_rewrite_core_shape_policy(root)
    if args.scope in ("all", "seed"):
        seed_detail, seed_count = run_seed_shape_policy(root)

    errors: list[str] = []
    if args.scope in ("all", "rewrite") and rewrite_detail:
        errors.append(f"rewrite:{rewrite_detail}")
    if args.scope in ("all", "seed") and seed_detail:
        errors.append(f"seed:{seed_detail}")
    if errors:
        return fail(";".join(errors))

    total = rewrite_count + seed_count
    print(
        f"seamgrim visual contract check ok total={total} rewrite_count={rewrite_count} seed_count={seed_count}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
