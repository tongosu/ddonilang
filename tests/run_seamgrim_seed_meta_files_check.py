#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from _seamgrim_visual_contract_lib import run_seed_meta_files_policy


def fail(detail: str) -> int:
    print(f"check=seed_meta_files detail={detail}")
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    detail, checked = run_seed_meta_files_policy(root)
    if detail:
        return fail(detail)
    print(f"seamgrim seed meta files check ok count={checked}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
