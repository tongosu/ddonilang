from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_HISTORICAL_PROGRESS_LEDGER_RECONCILIATION_V1.md"
REPORT = ROOT / "docs" / "studio" / "HISTORICAL_PROGRESS_LEDGER_RECONCILIATION_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "studio_historical_progress_ledger_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
LEDGER = PACK / "ledger.detjson"


def fail(message: str) -> None:
    print(f"studio_historical_progress_ledger_reconciliation_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")


def load_json(path: Path) -> dict:
    return json.loads(read(path))


def run(cmd: list[str], *, timeout: int = 120) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
        check=False,
    )


def require_contains(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing tokens: {missing}")


def require_files() -> None:
    ledger = load_json(LEDGER)
    required = [
        DOC,
        REPORT,
        INDEX,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        LEDGER,
    ]
    required.extend(ROOT / path for path in ledger["historical_ledger_paths"])
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        fail(f"missing required paths: {missing}")


def check_docs() -> None:
    common = [
        "STUDIO_HISTORICAL_PROGRESS_LEDGER_RECONCILIATION_V1",
        "9/18 = 50%",
        "90/90 = 100%",
        "0/90 = 0%",
        "12/12 = 100%",
        "0/1 = 0% approval-gated",
        "STUDIO_SAFE_CONTINUATION_QUEUE_RECHECK_V1",
        "docs/ssot/**",
    ]
    require_contains(DOC, common + ["historical ledger paths inventoried: 9/9 = 100%"])
    require_contains(REPORT, common + ["Status: docs-closed reconciliation", "Historical-only ledgers"])
    require_contains(
        INDEX,
        [
            "STUDIO_HISTORICAL_PROGRESS_LEDGER_RECONCILIATION_V1",
            "STUDIO_HISTORICAL_PROGRESS_LEDGER_RECONCILIATION_V1.md",
            "pack/studio_historical_progress_ledger_reconciliation_v1",
            "tests/run_studio_historical_progress_ledger_reconciliation_check.py",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_HISTORICAL_PROGRESS_LEDGER_RECONCILIATION_V1",
            "studio_historical_progress_ledger_reconciliation_v1",
            "historical progress ledger reconciliation 5/5 = 100%",
            "historical ledger paths: 9/9 = 100%",
            "STUDIO_SAFE_CONTINUATION_QUEUE_RECHECK_V1",
            "Studio-local 초장기 계획: 9/18 = 50%",
        ],
    )
    require_contains(PROJECT_STATUS, ["STUDIO_HISTORICAL_PROGRESS_LEDGER_RECONCILIATION_V1", "historical progress ledger reconciliation `5/5 = 100%`"])
    require_contains(CHANGELOG, ["Studio historical progress ledger reconciliation", "historical progress ledger reconciliation is `5/5 = 100%`"])


def check_contract_and_ledger() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_historical_progress_ledger_reconciliation_v1",
        "kind": "studio_historical_progress_ledger_reconciliation",
        "closed_by": "STUDIO_HISTORICAL_PROGRESS_LEDGER_RECONCILIATION_V1",
        "closure_tier": "닫힘-문서",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "behavior_closed_claim": False,
        "goal_completion_claim": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "parser_frontdoor_change": False,
        "stdlib_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "historical_ledger_path_count": 9,
        "official_studio_local_closed": 9,
        "official_studio_local_total": 18,
        "official_studio_local_percent": 50,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
        "roadmap_v2_docs_closed": 0,
        "public_release_execution_closed": 0,
        "public_release_execution_total": 1,
        "requires_docs_ssot_clean": True,
        "next_item": "STUDIO_SAFE_CONTINUATION_QUEUE_RECHECK_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    ledger = load_json(LEDGER)
    if ledger.get("schema") != "ddn.studio.historical_progress_ledger_reconciliation.v1":
        fail(f"ledger schema mismatch: {ledger.get('schema')!r}")
    if ledger.get("classification") != "historical_reference_not_current_authority":
        fail(f"ledger classification mismatch: {ledger.get('classification')!r}")
    if ledger.get("next_safe_action") != "STUDIO_SAFE_CONTINUATION_QUEUE_RECHECK_V1":
        fail(f"next safe action mismatch: {ledger.get('next_safe_action')!r}")
    paths = ledger.get("historical_ledger_paths", [])
    if len(paths) != ledger.get("historical_path_count") or len(paths) != 9:
        fail(f"historical path count mismatch: {len(paths)} vs {ledger.get('historical_path_count')}")
    tokens = ledger.get("historical_progress_tokens", [])
    for item in paths:
        text = read(ROOT / item)
        if not any(token in text for token in tokens):
            fail(f"historical path lacks historical tokens: {item}")
    authority = ledger.get("current_authority", {})
    for key, value in {
        "studio_local_super_long": "9/18 = 50%",
        "roadmap_v2_behavior_closed": "90/90 = 100%",
        "roadmap_v2_docs_closed": "0/90 = 0%",
        "stale_progress_repair": "12/12 = 100%",
        "public_release_execution": "0/1 = 0% approval-gated",
    }.items():
        if authority.get(key) != value:
            fail(f"authority {key} expected {value!r}, got {authority.get(key)!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected_stdout = [
        "STUDIO_HISTORICAL_PROGRESS_LEDGER_RECONCILIATION_V1",
        "historical progress ledger reconciliation sealed",
        "historical ledger paths: 9/9 = 100%",
        "current authority preserved: 5/5 = 100%",
        "studio local progress: 9/18 = 50%",
        "roadmap v2 behavior: 90/90 = 100%",
        "public release execution: approval-gated",
        "next: STUDIO_SAFE_CONTINUATION_QUEUE_RECHECK_V1",
    ]
    if payload.get("cmd") != ["run", "pack/studio_historical_progress_ledger_reconciliation_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected_stdout:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_historical_progress_ledger_reconciliation_v1"],
    ]:
        proc = run(cmd, timeout=180)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def check_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed: {proc.stdout.strip()}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    require_files()
    check_docs()
    check_contract_and_ledger()
    check_golden()
    run_required_gates()
    check_docs_ssot_clean()
    print("studio_historical_progress_ledger_reconciliation_check: ok")


if __name__ == "__main__":
    main()
