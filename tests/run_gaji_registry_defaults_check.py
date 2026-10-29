#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


REQUIRED_TOKEN_MAP = {
    "tools/teul-cli/src/main.rs": [
        "fn gaji_install_main_cli_defaults_follow_dr169()",
        "fn gaji_update_main_cli_defaults_follow_dr169()",
        "fn gaji_vendor_main_cli_defaults_follow_dr169()",
        "strict-registry default must be false",
        "frozen-lockfile default must be false",
        "deny-yanked-locked default must be false in non-strict mode",
    ],
}


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    missing: list[str] = []

    for rel_path, tokens in REQUIRED_TOKEN_MAP.items():
        target = root / rel_path
        if not target.exists():
            print(f"missing target: {target}")
            return 1
        text = target.read_text(encoding="utf-8")
        for token in tokens:
            if token not in text:
                missing.append(f"{rel_path}::{token}")

    if missing:
        print("gaji registry defaults check failed:")
        for token in missing[:16]:
            print(f" - missing token: {token}")
        return 1

    print("gaji registry defaults check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
