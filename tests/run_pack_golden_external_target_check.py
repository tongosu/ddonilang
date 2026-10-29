#!/usr/bin/env python3
from __future__ import annotations

import os
from pathlib import Path
from unittest.mock import patch

from _run_pack_golden_impl import (
    PACK_GOLDEN_TARGET_ENV_KEY,
    build_pack_golden_subprocess_env,
)


ROOT = Path(__file__).resolve().parents[1]


def is_relative_to(path: Path, parent: Path) -> bool:
    try:
        path.relative_to(parent)
        return True
    except ValueError:
        return False


def main() -> int:
    clean_env = {
        key: value
        for key, value in os.environ.items()
        if key not in {"CARGO_TARGET_DIR", PACK_GOLDEN_TARGET_ENV_KEY, "RUST_MIN_STACK"}
    }
    with patch.dict(os.environ, clean_env, clear=True):
        resolved = Path(build_pack_golden_subprocess_env()["CARGO_TARGET_DIR"]).resolve()
        assert not is_relative_to(resolved, ROOT / "pack"), resolved
        assert resolved.name == "cargo-target-pack-golden", resolved

    custom = ROOT.parent / "custom-pack-golden-target"
    with patch.dict(
        os.environ,
        {**clean_env, PACK_GOLDEN_TARGET_ENV_KEY: str(custom)},
        clear=True,
    ):
        resolved = Path(build_pack_golden_subprocess_env()["CARGO_TARGET_DIR"]).resolve()
        assert resolved == custom.resolve(), resolved

    existing = ROOT.parent / "existing-cargo-target"
    with patch.dict(
        os.environ,
        {
            **clean_env,
            "CARGO_TARGET_DIR": str(existing),
            PACK_GOLDEN_TARGET_ENV_KEY: str(custom),
            "RUST_MIN_STACK": "123456",
        },
        clear=True,
    ):
        env = build_pack_golden_subprocess_env()
        assert Path(env["CARGO_TARGET_DIR"]).resolve() == existing.resolve()
        assert env["RUST_MIN_STACK"] == "123456"

    print("pack_golden_external_target_check: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
