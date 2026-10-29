#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "STUDIO_RC_CHECKER_COST_TRIM_V1.md"
PREV = ROOT / "docs" / "context" / "queue" / "STUDIO_POST_RELEASE_GATE_MAINTENANCE_QUEUE_V1.md"
PACK = ROOT / "pack" / "studio_rc_checker_cost_trim_v1"
COST_TRIM = PACK / "cost_trim.detjson"
EXECUTION_GATE_CHECKER = ROOT / "tests" / "run_studio_public_release_execution_gate_check.py"
EXECUTION_GATE = ROOT / "pack" / "studio_public_release_execution_gate_v1" / "execution_gate.detjson"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(cmd: list[str], *, timeout: int = 420) -> subprocess.CompletedProcess[str]:
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
        DOC,
        PREV,
        PACK / "README.md",
        PACK / "contract.detjson",
        COST_TRIM,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        EXECUTION_GATE_CHECKER,
        EXECUTION_GATE,
        ROOT / "tests" / "run_studio_public_release_smoke_matrix_check.py",
        ROOT / "tests" / "run_studio_public_release_asset_plan_check.py",
        ROOT / "tests" / "run_studio_release_candidate_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_RC_COST_TRIM_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_docs() -> int:
    checks = [
        (
            DOC,
            [
                "STUDIO_RC_CHECKER_COST_TRIM_V1",
                "smoke matrix checker once",
                "asset plan checker already runs the release candidate checker",
                "Do not create release archives",
                "docs/ssot/**",
                "STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1",
            ],
            "E_STUDIO_RC_COST_TRIM_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_POST_RELEASE_GATE_MAINTENANCE_QUEUE_V1",
                "STUDIO_RC_CHECKER_COST_TRIM_V1",
            ],
            "E_STUDIO_RC_COST_TRIM_PREV",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_cost_trim() -> int:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_rc_checker_cost_trim_v1",
        "kind": "studio_rc_checker_cost_trim",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_RC_CHECKER_COST_TRIM_V1",
        "cost_trim": "pack/studio_rc_checker_cost_trim_v1/cost_trim.detjson",
        "trimmed_checker": "tests/run_studio_public_release_execution_gate_check.py",
        "aggregate_gate": "tests/run_studio_public_release_smoke_matrix_check.py",
        "direct_duplicate_asset_plan_call": False,
        "direct_duplicate_release_candidate_call": False,
        "preserves_execution_gate_contract": True,
        "release_execution_claim": False,
        "public_release_claim": False,
        "asset_generation_claim": False,
        "next_item": "STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_RC_COST_TRIM_CONTRACT", f"{key}={contract.get(key)!r}")

    cost_trim = json.loads(COST_TRIM.read_text(encoding="utf-8"))
    if cost_trim.get("schema") != "ddn.studio.rc_checker_cost_trim.v1":
        return fail("E_STUDIO_RC_COST_TRIM_SCHEMA", repr(cost_trim.get("schema")))
    after = cost_trim.get("after")
    if not isinstance(after, dict) or after.get("execution_gate_direct_preflight_calls") != [
        "tests/run_studio_public_release_smoke_matrix_check.py"
    ]:
        return fail("E_STUDIO_RC_COST_TRIM_AFTER", repr(after))
    if cost_trim.get("preserved_contract_preflight_ids") != [
        "smoke_matrix",
        "asset_plan",
        "release_candidate",
        "docs_ssot_clean",
    ]:
        return fail("E_STUDIO_RC_COST_TRIM_PREFLIGHT_IDS", repr(cost_trim.get("preserved_contract_preflight_ids")))
    return 0


def check_execution_gate_checker_trimmed() -> int:
    text = read(EXECUTION_GATE_CHECKER)
    start = text.find("def run_required_gates()")
    end = text.find("def check_dev_summary()", start)
    if start == -1 or end == -1:
        return fail("E_STUDIO_RC_COST_TRIM_FUNCTION_BODY", "run_required_gates body not found")
    body = text[start:end]
    if "tests/run_studio_public_release_smoke_matrix_check.py" not in body:
        return fail("E_STUDIO_RC_COST_TRIM_MISSING_SMOKE_GATE", body)
    forbidden = [
        "tests/run_studio_public_release_asset_plan_check.py",
        "tests/run_studio_release_candidate_check.py",
    ]
    found = [token for token in forbidden if token in body]
    if found:
        return fail("E_STUDIO_RC_COST_TRIM_DUPLICATE_GATE", repr(found))

    gate = json.loads(EXECUTION_GATE.read_text(encoding="utf-8"))
    ids = [item.get("id") for item in gate.get("preflight_gates", [])]
    if ids != ["smoke_matrix", "asset_plan", "release_candidate", "docs_ssot_clean"]:
        return fail("E_STUDIO_RC_COST_TRIM_GATE_CONTRACT", repr(ids))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_RC_CHECKER_COST_TRIM_V1",
        "studio rc checker cost trim sealed",
        "next: STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_RC_COST_TRIM_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_rc_checker_cost_trim_v1"],
        ["python", "tests/run_studio_public_release_execution_gate_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd)
        if proc.returncode != 0:
            return fail("E_STUDIO_RC_COST_TRIM_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_RC_CHECKER_COST_TRIM_V1",
        "studio_rc_checker_cost_trim_v1",
        "run_studio_rc_checker_cost_trim_check.py",
        "run_studio_public_release_execution_gate_check.py",
        "STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_RC_COST_TRIM_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_RC_COST_TRIM_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_RC_COST_TRIM_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_cost_trim,
        check_execution_gate_checker_trimmed,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-rc-checker-cost-trim-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
