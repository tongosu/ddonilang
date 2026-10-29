#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


REQUIRED_TOKENS = [
    "--runtime-5min-skip-ui-common",
    "if args.runtime_5min_skip_ui_common:",
    'runtime_5min_cmd.append("--skip-ui-common")',
    '"runtime_5min_skip_ui_common": bool(args.runtime_5min_skip_ui_common)',
    "--runtime-5min-skip-showcase-check",
    "if args.runtime_5min_skip_showcase_check:",
    'runtime_5min_cmd.append("--skip-showcase-check")',
    "--runtime-5min-showcase-smoke",
    "if args.runtime_5min_showcase_smoke:",
    '"--showcase-smoke",',
    "--runtime-5min-showcase-smoke-madi-pendulum",
    "--runtime-5min-showcase-smoke-madi-tetris",
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
        print("seamgrim ci gate runtime5 passthrough check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1

    print("seamgrim ci gate runtime5 passthrough check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
