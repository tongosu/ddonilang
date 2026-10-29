#!/usr/bin/env python
from __future__ import annotations

import os
import runpy
import sys
from pathlib import Path


LEGACY_ENV = "DDN_W107_GOLDEN_INDEX_SELFTEST_PROGRESS_JSON"
NEW_ENV = "DDN_W107_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON"


def main() -> int:
    # Backward-compat shim: old selftest entrypoint forwards to selfcheck implementation.
    old_value = str(os.environ.get(LEGACY_ENV, "")).strip()
    new_value = str(os.environ.get(NEW_ENV, "")).strip()
    if old_value and not new_value:
        os.environ[NEW_ENV] = old_value
    if new_value and not old_value:
        os.environ[LEGACY_ENV] = new_value

    target = (
        Path(__file__).resolve().parents[1]
        / "tools"
        / "teul-cli"
        / "tests"
        / "run_w107_golden_index_selfcheck.py"
    )
    runpy.run_path(str(target), run_name="__main__")
    return 0


if __name__ == "__main__":
    sys.exit(main())
