#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_approved_workbench_promotion_packet_v1"
REPORT = ROOT / "docs" / "reports" / "impl" / "SEAMGRIM_APPROVED_WORKBENCH_PROMOTION_PACKET_V1_20260713.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def run(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def require_files() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        REPORT,
        ROOT / "tests" / "run_seamgrim_approved_workbench_shell_check.py",
        ROOT / "tests" / "run_seamgrim_approved_workbench_runtime_lineage_check.py",
        ROOT / "tests" / "run_seamgrim_approved_workbench_semantic_verticals_check.py",
        ROOT / "tests" / "run_seamgrim_approved_workbench_quality_gate_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_PROMOTION_MISSING", str(missing))
    return 0


def require_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_approved_workbench_promotion_packet_v1",
        "default_shell_switch_claim": False,
        "behavior_closed_claim": False,
        "docs_closed_claim": True,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
        "docs_ssot_change": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_PROMOTION_CONTRACT", f"{key}={payload.get(key)!r}")
    deps = payload.get("depends_on_packs")
    if not isinstance(deps, list) or len(deps) != 4:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_PROMOTION_DEPS", repr(deps))
    for pack_name in deps:
        if not (ROOT / "pack" / str(pack_name) / "contract.detjson").exists():
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_PROMOTION_DEP_MISSING", str(pack_name))
    return 0


def require_report_tokens() -> int:
    text = REPORT.read_text(encoding="utf-8")
    tokens = [
        "Product path: `solutions/seamgrim_ui_mvp/ui/workbench/index.html`",
        "Old To New Mapping",
        "Compatibility And Redirect Candidates",
        "닫힘-동작",
        "닫힘-문서",
        "Open P2/P3/P4",
        "default shell",
        "state_hash",
        "docs/ssot",
        "not used as evidence",
    ]
    for token in tokens:
        if token not in text:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_PROMOTION_REPORT_TOKEN", token)
    return 0


def run_pack_golden() -> int:
    proc = run(["python", "tests/run_pack_golden.py", "seamgrim_approved_workbench_promotion_packet_v1"], timeout=120)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_PROMOTION_PACK", proc.stdout.strip())
    return 0


def require_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_PROMOTION_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_PROMOTION_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (
        require_files,
        require_contract,
        require_report_tokens,
        run_pack_golden,
        require_docs_ssot_clean,
    ):
        rc = check()
        if rc:
            return rc
    print("[seamgrim-approved-workbench-promotion-packet-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
