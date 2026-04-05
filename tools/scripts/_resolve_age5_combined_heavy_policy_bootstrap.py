#!/usr/bin/env python
from __future__ import annotations

import json
import os
import sys
import time
from pathlib import Path


PROGRESS_ENV_KEY = "DDN_AGE5_COMBINED_HEAVY_POLICY_PROGRESS_JSON"
SCHEMA = "ddn.age5_combined_heavy_policy_progress.v1"


def _parse_bootstrap_cli(argv: list[str]) -> tuple[str, str]:
    provider = "-"
    output_format = "text"
    for idx, token in enumerate(argv):
        if token == "--provider" and idx + 1 < len(argv):
            provider = str(argv[idx + 1]).strip() or "-"
        elif token == "--format" and idx + 1 < len(argv):
            output_format = str(argv[idx + 1]).strip() or "text"
    return provider, output_format


def _write_progress(*, provider: str, output_format: str, stage: str, elapsed_ms: int) -> None:
    path_text = str(os.environ.get(PROGRESS_ENV_KEY, "")).strip()
    if not path_text:
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": SCHEMA,
        "provider": provider,
        "format": output_format,
        "status": "running",
        "current_stage": stage,
        "last_completed_stage": "-",
        "total_elapsed_ms": int(elapsed_ms),
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    started = time.perf_counter()
    provider, output_format = _parse_bootstrap_cli(sys.argv[1:])
    _write_progress(
        provider=provider,
        output_format=output_format,
        stage="bootstrap_delegate_import_impl",
        elapsed_ms=0,
    )
    from _resolve_age5_combined_heavy_policy_impl import main as impl_main

    _write_progress(
        provider=provider,
        output_format=output_format,
        stage="bootstrap_delegate_call_impl",
        elapsed_ms=int(round((time.perf_counter() - started) * 1000.0)),
    )
    return int(impl_main())


if __name__ == "__main__":
    raise SystemExit(main())
