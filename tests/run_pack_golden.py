#!/usr/bin/env python
from __future__ import annotations

import json
import os
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

PROGRESS_ENV_KEY = "DDN_RUN_PACK_GOLDEN_PROGRESS_JSON"


def write_json_atomic(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = path.with_name(f"{path.name}.{time.time_ns()}.tmp")
    tmp_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for attempt in range(10):
        try:
            tmp_path.replace(path)
            return
        except PermissionError:
            if attempt >= 9:
                raise
            time.sleep(0.01 * (attempt + 1))


def _parse_bootstrap_cli(argv: list[str]) -> tuple[str, str]:
    current_pack = "-"
    if argv and not str(argv[0]).startswith("-"):
        current_pack = str(argv[0]).strip() or "-"
    return current_pack, "-"


def _write_bootstrap_progress(stage: str, last_completed_stage: str = "-") -> None:
    path_text = str(os.environ.get(PROGRESS_ENV_KEY, "")).strip()
    if not path_text:
        return
    current_pack, current_case = _parse_bootstrap_cli(sys.argv[1:])
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.pack.golden.progress.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": "running",
        "current_stage": stage,
        "last_completed_stage": last_completed_stage,
        "current_pack": current_pack,
        "last_completed_pack": "-",
        "current_case": current_case,
        "last_completed_case": "-",
        "total_elapsed_ms": 0,
    }
    write_json_atomic(out, payload)


def main() -> int:
    _write_bootstrap_progress("bootstrap_imports")
    from _run_pack_golden_impl import main as impl_main

    _write_bootstrap_progress("bootstrap_delegate_call_impl", "bootstrap_imports")
    return int(impl_main())


if __name__ == "__main__":
    raise SystemExit(main())
