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
    cases = contract.get("cases")
    if not isinstance(cases, list):
        return fail("contract_cases_type")
    if "c02_boim_surface_accepted" not in cases or "c02_legacy_boim_forbidden" in cases:
        return fail("contract_cases")

    golden_rows = [
        json.loads(line)
        for line in (PACK / "golden.jsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    by_id = {row.get("id"): row for row in golden_rows}
    boim_row = by_id.get("c02_boim_surface_accepted")
    if not boim_row:
        return fail("golden_c02_missing")
    if boim_row.get("expected_error_code"):
        return fail("golden_c02_must_not_expect_error")
    if boim_row.get("exit_code") != 0:
        return fail("golden_c02_exit_code")
    if boim_row.get("stdout") != ["보임 {", "  x: 1.", "}."]:
        return fail("golden_c02_stdout")

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
