#!/usr/bin/env python
from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPORTS = [
    ROOT / "docs" / "reports" / "audit" / "SETTINGS_GEOUL_RUN_AGE_GATE_RECHECK_V1.md",
    ROOT / "docs" / "reports" / "audit" / "NURIGYM_CONSUMER_EVIDENCE_STATUS_V1.md",
]
INDEX = ROOT / "docs" / "reports" / "audit" / "INDEX.md"


def fail(message: str) -> int:
    print(f"dayrun_g7_reports_check: FAIL: {message}", file=sys.stderr)
    return 1


def main() -> int:
    missing = [path.relative_to(ROOT).as_posix() for path in REPORTS if not path.exists()]
    if missing:
        return fail(f"missing reports: {missing}")
    index = INDEX.read_text(encoding="utf-8")
    for report in REPORTS:
        if report.name not in index:
            return fail(f"index missing {report.name}")
        text = report.read_text(encoding="utf-8")
        for token in ["Drift / 현황", "닫힘-문서", ":"]:
            if token not in text:
                return fail(f"{report.name} missing token {token!r}")
        if "docs/ssot/ssot/" not in text and "docs/context/" not in text:
            return fail(f"{report.name} lacks file:line evidence")
    settings = REPORTS[0].read_text(encoding="utf-8")
    if "설정.거울" not in settings or "설정.실행" not in settings or "lang_settings_header_closure_g7_probe.log" not in settings:
        return fail("settings report lacks required scope/probe evidence")
    nurigym = REPORTS[1].read_text(encoding="utf-8")
    for token in ["wasmtime-py", "PyO3", "nurigym_python_web_parity_g7_probe.log", "nuri_gym_contract_g7_probe.log"]:
        if token not in nurigym:
            return fail(f"nurigym report missing {token}")
    print("dayrun_g7_reports_check: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
