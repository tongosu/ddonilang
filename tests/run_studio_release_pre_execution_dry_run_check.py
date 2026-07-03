#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "STUDIO_RELEASE_PRE_EXECUTION_DRY_RUN_V1.md"
PREV = ROOT / "docs" / "context" / "queue" / "STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
REPORT = ROOT / "docs" / "studio" / "RELEASE_PRE_EXECUTION_DRY_RUN_V1.md"
PACK = ROOT / "pack" / "studio_release_pre_execution_dry_run_v1"
DRY_RUN = PACK / "dry_run.detjson"
READINESS = ROOT / "pack" / "studio_release_approval_readiness_recheck_v1" / "readiness.detjson"
ASSET_PLAN = ROOT / "pack" / "studio_public_release_asset_plan_v1" / "release_assets.detjson"
REQUIRED_APPROVAL = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다"
BLOCKED = [
    "github_release_create",
    "public_upload",
    "registry_publish",
    "cloud_sync",
    "account_setup",
    "artifact_signing",
    "publication_archive_generation",
    "checksum_manifest_generation_for_publication",
]


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


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def require_files() -> int:
    required = [
        DOC,
        PREV,
        INDEX,
        REPORT,
        PACK / "README.md",
        PACK / "contract.detjson",
        DRY_RUN,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        READINESS,
        ASSET_PLAN,
        ROOT / "tests" / "run_studio_release_approval_readiness_recheck.py",
    ]
    if DRY_RUN.exists():
        for source in load_json(DRY_RUN).get("evidence_inputs", []):
            required.append(ROOT / source)
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_RELEASE_DRY_RUN_MISSING", str(missing))
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
                "STUDIO_RELEASE_PRE_EXECUTION_DRY_RUN_V1",
                "does not execute the release",
                "STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1",
                "STUDIO_RELEASE_DRY_RUN_TEXT_SUMMARY_V1",
                "docs/ssot/**",
            ],
            "E_STUDIO_RELEASE_DRY_RUN_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1",
                "STUDIO_RELEASE_PRE_EXECUTION_DRY_RUN_V1",
            ],
            "E_STUDIO_RELEASE_DRY_RUN_PREV",
        ),
        (
            INDEX,
            [
                "STUDIO_RELEASE_PRE_EXECUTION_DRY_RUN_V1",
                "pack/studio_release_pre_execution_dry_run_v1",
                "tests/run_studio_release_pre_execution_dry_run_check.py",
                "docs/studio/RELEASE_PRE_EXECUTION_DRY_RUN_V1.md",
            ],
            "E_STUDIO_RELEASE_DRY_RUN_INDEX",
        ),
        (
            REPORT,
            [
                "Studio Release Pre-Execution Dry Run V1",
                "Status: approval-safe dry run",
                REQUIRED_APPROVAL,
                "These are planned paths only",
                "Blocked In Dry Run",
                "STUDIO_RELEASE_DRY_RUN_TEXT_SUMMARY_V1",
            ],
            "E_STUDIO_RELEASE_DRY_RUN_REPORT",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_dry_run() -> int:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_release_pre_execution_dry_run_v1",
        "kind": "studio_release_pre_execution_dry_run",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_RELEASE_PRE_EXECUTION_DRY_RUN_V1",
        "dry_run": "pack/studio_release_pre_execution_dry_run_v1/dry_run.detjson",
        "report": "docs/studio/RELEASE_PRE_EXECUTION_DRY_RUN_V1.md",
        "based_on": "STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1",
        "required_approval_phrase": REQUIRED_APPROVAL,
        "dry_run_only": True,
        "release_execution_claim": False,
        "public_release_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "asset_generation_claim": False,
        "next_item": "STUDIO_RELEASE_DRY_RUN_TEXT_SUMMARY_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_RELEASE_DRY_RUN_CONTRACT", f"{key}={contract.get(key)!r}")

    dry_run = load_json(DRY_RUN)
    if dry_run.get("schema") != "ddn.studio.release_pre_execution_dry_run.v1":
        return fail("E_STUDIO_RELEASE_DRY_RUN_SCHEMA", repr(dry_run.get("schema")))
    if dry_run.get("required_approval_phrase") != REQUIRED_APPROVAL:
        return fail("E_STUDIO_RELEASE_DRY_RUN_APPROVAL", repr(dry_run.get("required_approval_phrase")))
    if dry_run.get("blocked_in_dry_run") != BLOCKED:
        return fail("E_STUDIO_RELEASE_DRY_RUN_BLOCKED", repr(dry_run.get("blocked_in_dry_run")))
    for flag in ("release_execution_claim", "public_release_claim", "github_release_claim", "public_upload_claim", "asset_generation_claim"):
        if dry_run.get(flag) is not False:
            return fail("E_STUDIO_RELEASE_DRY_RUN_FLAG", f"{flag}={dry_run.get(flag)!r}")
    return 0


def check_against_sources() -> int:
    dry_run = load_json(DRY_RUN)
    readiness = load_json(READINESS)
    asset_plan = load_json(ASSET_PLAN)
    if dry_run.get("required_approval_phrase") != readiness.get("required_approval_phrase"):
        return fail("E_STUDIO_RELEASE_DRY_RUN_READINESS_APPROVAL", repr(readiness.get("required_approval_phrase")))
    if dry_run.get("preflight_commands") != [
        "python tests/run_studio_public_release_smoke_matrix_check.py",
        "python tests/run_studio_public_release_asset_plan_check.py",
        "python tests/run_studio_release_candidate_check.py",
        "git status --short -- docs/ssot",
    ]:
        return fail("E_STUDIO_RELEASE_DRY_RUN_PREFLIGHTS", repr(dry_run.get("preflight_commands")))
    planned = [(item["id"], item["planned_path"], item["generated_now"]) for item in dry_run["planned_assets"]]
    assets = [(item["id"], item["planned_path"], item["generated_now"]) for item in asset_plan["assets"]]
    if planned != assets:
        return fail("E_STUDIO_RELEASE_DRY_RUN_ASSETS", repr({"dry_run": planned, "asset_plan": assets}))
    if any(item["generated_now"] is not False for item in dry_run["planned_assets"]):
        return fail("E_STUDIO_RELEASE_DRY_RUN_GENERATED", repr(dry_run["planned_assets"]))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_RELEASE_PRE_EXECUTION_DRY_RUN_V1",
        "studio release pre-execution dry run sealed",
        "next: STUDIO_RELEASE_DRY_RUN_TEXT_SUMMARY_V1",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_RELEASE_DRY_RUN_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_release_pre_execution_dry_run_v1"],
        ["python", "tests/run_studio_release_approval_readiness_recheck.py"],
    ]
    for cmd in commands:
        proc = run(cmd)
        if proc.returncode != 0:
            return fail("E_STUDIO_RELEASE_DRY_RUN_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_RELEASE_PRE_EXECUTION_DRY_RUN_V1",
        "studio_release_pre_execution_dry_run_v1",
        "dry_run.detjson",
        "run_studio_release_pre_execution_dry_run_check.py",
        "STUDIO_RELEASE_DRY_RUN_TEXT_SUMMARY_V1",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_RELEASE_DRY_RUN_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_RELEASE_DRY_RUN_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_RELEASE_DRY_RUN_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_dry_run,
        check_against_sources,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-release-pre-execution-dry-run-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
