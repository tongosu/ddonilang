#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1.md"
PREV = ROOT / "STUDIO_RC_CHECKER_COST_TRIM_V1.md"
PACK = ROOT / "pack" / "studio_browser_smoke_flake_audit_v1"
AUDIT = PACK / "browser_smoke_audit.detjson"
SMOKE_MATRIX = ROOT / "pack" / "studio_public_release_smoke_matrix_v1" / "smoke_matrix.detjson"


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
        AUDIT,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        SMOKE_MATRIX,
        ROOT / "tests" / "run_studio_public_release_smoke_matrix_check.py",
        ROOT / "pack" / "studio_rc_checker_cost_trim_v1" / "contract.detjson",
    ]
    if AUDIT.exists():
        audit = json.loads(AUDIT.read_text(encoding="utf-8"))
        for item in audit.get("browser_smokes", []):
            required.append(ROOT / item["checker"])
            required.append(ROOT / item["runner"])
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_BROWSER_FLAKE_AUDIT_MISSING", str(missing))
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
                "STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1",
                "six Studio browser smokes",
                "fail on browser console errors",
                "timeout=120",
                "STUDIO_DOC_INDEX_REFRESH_V1",
                "docs/ssot/**",
            ],
            "E_STUDIO_BROWSER_FLAKE_AUDIT_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_RC_CHECKER_COST_TRIM_V1",
                "STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1",
            ],
            "E_STUDIO_BROWSER_FLAKE_AUDIT_PREV",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_audit() -> int:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_browser_smoke_flake_audit_v1",
        "kind": "studio_browser_smoke_flake_audit",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1",
        "audit": "pack/studio_browser_smoke_flake_audit_v1/browser_smoke_audit.detjson",
        "based_on": "STUDIO_RC_CHECKER_COST_TRIM_V1",
        "smoke_matrix": "pack/studio_public_release_smoke_matrix_v1/smoke_matrix.detjson",
        "browser_smoke_count": 6,
        "requires_playwright_chromium_probe": True,
        "requires_browser_runner_timeout_seconds": 120,
        "release_execution_claim": False,
        "public_release_claim": False,
        "asset_generation_claim": False,
        "next_item": "STUDIO_DOC_INDEX_REFRESH_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_BROWSER_FLAKE_AUDIT_CONTRACT", f"{key}={contract.get(key)!r}")

    audit = json.loads(AUDIT.read_text(encoding="utf-8"))
    if audit.get("schema") != "ddn.studio.browser_smoke_flake_audit.v1":
        return fail("E_STUDIO_BROWSER_FLAKE_AUDIT_SCHEMA", repr(audit.get("schema")))
    if len(audit.get("browser_smokes", [])) != 6:
        return fail("E_STUDIO_BROWSER_FLAKE_AUDIT_COUNT", repr(audit.get("browser_smokes")))
    for flag in ("release_execution_claim", "public_release_claim", "asset_generation_claim"):
        if audit.get(flag) is not False:
            return fail("E_STUDIO_BROWSER_FLAKE_AUDIT_FLAG", f"{flag}={audit.get(flag)!r}")
    return 0


def check_matches_smoke_matrix() -> int:
    audit = json.loads(AUDIT.read_text(encoding="utf-8"))
    matrix = json.loads(SMOKE_MATRIX.read_text(encoding="utf-8"))
    audited = [(item["id"], item["checker"], item["runner"]) for item in audit["browser_smokes"]]
    expected = [(item["id"], item["checker"], item["runner"]) for item in matrix["browser_smokes"]]
    if audited != expected:
        return fail("E_STUDIO_BROWSER_FLAKE_AUDIT_MATRIX_MISMATCH", repr({"audit": audited, "matrix": expected}))
    return 0


def check_runner_policy() -> int:
    audit = json.loads(AUDIT.read_text(encoding="utf-8"))
    required_runner_tokens = [
        'import { chromium } from "playwright"',
        "chromium.launch({ headless: true })",
        "function createServer(root)",
        "UNSAFE_BROWSER_PORTS",
        "server.listen(0, \"127.0.0.1\"",
        "UNSAFE_BROWSER_PORTS.has(address.port)",
        'page.on("console"',
        "Failed to load resource",
        'page.on("pageerror"',
        'page.on("requestfailed"',
        'page.on("response"',
        "res.status() >= 400",
        "isAllowedFallback404",
        "await page.goto",
        "waitUntil: \"domcontentloaded\"",
        "await context.close()",
        "await browser.close()",
        "await closeServer(server)",
        "console.log(OK)",
    ]
    for item in audit["browser_smokes"]:
        runner = ROOT / item["runner"]
        text = read(runner)
        missing = [token for token in required_runner_tokens if token not in text]
        if item["ok_line"] not in text:
            missing.append(item["ok_line"])
        if missing:
            return fail("E_STUDIO_BROWSER_FLAKE_AUDIT_RUNNER_POLICY", f"{item['runner']} missing {missing}")
    return 0


def check_checker_policy() -> int:
    audit = json.loads(AUDIT.read_text(encoding="utf-8"))
    for item in audit["browser_smokes"]:
        checker = ROOT / item["checker"]
        text = read(checker)
        required = [
            "def check_playwright_available()",
            "chromium.launch({headless:true})",
            "def run_browser_smoke()",
            f'"{item["runner"]}"',
            "timeout=120",
            item["ok_line"],
            "git\", \"status\", \"--short\", \"--\", \"docs/ssot",
        ]
        missing = [token for token in required if token not in text]
        if missing:
            return fail("E_STUDIO_BROWSER_FLAKE_AUDIT_CHECKER_POLICY", f"{item['checker']} missing {missing}")
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1",
        "studio browser smoke flake audit sealed",
        "next: STUDIO_DOC_INDEX_REFRESH_V1",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_BROWSER_FLAKE_AUDIT_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_browser_smoke_flake_audit_v1"],
        ["python", "tests/run_studio_public_release_smoke_matrix_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd)
        if proc.returncode != 0:
            return fail("E_STUDIO_BROWSER_FLAKE_AUDIT_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1",
        "studio_browser_smoke_flake_audit_v1",
        "browser_smoke_audit.detjson",
        "run_studio_browser_smoke_flake_audit_check.py",
        "STUDIO_DOC_INDEX_REFRESH_V1",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_BROWSER_FLAKE_AUDIT_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_BROWSER_FLAKE_AUDIT_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_BROWSER_FLAKE_AUDIT_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_audit,
        check_matches_smoke_matrix,
        check_runner_policy,
        check_checker_policy,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-browser-smoke-flake-audit-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
