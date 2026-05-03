#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "lang_settings_header_closure_v1"


def fail(detail: str) -> int:
    print(f"check=lang_settings_header_closure_pack detail={detail}")
    return 1


def run_cmd(cmd: list[str]) -> tuple[int, str]:
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=600,
    )
    detail = proc.stderr.strip() or proc.stdout.strip() or f"returncode={proc.returncode}"
    return proc.returncode, detail


def main() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "legacy_boim.ddn",
        PACK / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.lang_settings_header_closure.pack.contract.v1":
        return fail("schema")

    commands = [
        ["cargo", "test", "-p", "ddonirang-lang", "validate_no_legacy_header_accepts_canonical_surface", "--", "--nocapture"],
        ["cargo", "test", "-p", "ddonirang-lang", "validate_no_legacy_boim_surface_accepts_boim_block", "--", "--nocapture"],
        ["cargo", "test", "--manifest-path", "tools/teul-cli/Cargo.toml", "canon_accepts_boim_surface_on_frontdoor", "--", "--nocapture"],
    ]
    for cmd in commands:
        rc, detail = run_cmd(cmd)
        if rc != 0:
            return fail(f"{' '.join(cmd)}:{detail}")

    print("lang settings header closure pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
