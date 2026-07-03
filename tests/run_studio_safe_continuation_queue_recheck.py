from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_SAFE_CONTINUATION_QUEUE_RECHECK_V1.md"
REPORT = ROOT / "docs" / "studio" / "SAFE_CONTINUATION_QUEUE_RECHECK_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "studio_safe_continuation_queue_recheck_v1"
CONTRACT = PACK / "contract.detjson"
QUEUE = PACK / "queue.detjson"


def fail(message: str) -> None:
    print(f"studio_safe_continuation_queue_recheck: FAIL: {message}", file=sys.stderr)
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
        QUEUE,
        ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        fail(f"missing required paths: {missing}")


def check_docs() -> None:
    common = [
        "STUDIO_SAFE_CONTINUATION_QUEUE_RECHECK_V1",
        "SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_V1",
        "9/18 = 50%",
        "90/90 = 100%",
        "0/90 = 0%",
        "0/1 = 0% approval-gated",
        "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다",
        "docs/ssot/**",
    ]
    require_contains(DOC, common + ["safe continuation candidate selected: 1/1 = 100%"])
    require_contains(REPORT, common + ["Status: docs-closed queue recheck", "Candidate does not exist yet"])
    require_contains(
        INDEX,
        [
            "STUDIO_SAFE_CONTINUATION_QUEUE_RECHECK_V1",
            "STUDIO_SAFE_CONTINUATION_QUEUE_RECHECK_V1.md",
            "pack/studio_safe_continuation_queue_recheck_v1",
            "tests/run_studio_safe_continuation_queue_recheck.py",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_SAFE_CONTINUATION_QUEUE_RECHECK_V1",
            "studio_safe_continuation_queue_recheck_v1",
            "safe continuation queue recheck 5/5 = 100%",
            "safe continuation candidate selected: 1/1 = 100%",
            "SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_V1",
            "Studio-local 초장기 계획: 9/18 = 50%",
        ],
    )
    require_contains(PROJECT_STATUS, ["STUDIO_SAFE_CONTINUATION_QUEUE_RECHECK_V1", "safe continuation queue recheck `5/5 = 100%`"])
    require_contains(CHANGELOG, ["Studio safe continuation queue recheck", "safe continuation queue recheck is `5/5 = 100%`"])


def check_contract_and_queue() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_safe_continuation_queue_recheck_v1",
        "kind": "studio_safe_continuation_queue_recheck",
        "closed_by": "STUDIO_SAFE_CONTINUATION_QUEUE_RECHECK_V1",
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
        "official_studio_local_closed": 9,
        "official_studio_local_total": 18,
        "official_studio_local_percent": 50,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
        "roadmap_v2_docs_closed": 0,
        "public_release_execution_closed": 0,
        "public_release_execution_total": 1,
        "safe_candidate_selected": "SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_V1",
        "safe_candidate_exists": False,
        "requires_behavior_verification_if_implemented": True,
        "requires_docs_ssot_clean": True,
        "next_item": "SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    queue = load_json(QUEUE)
    if queue.get("schema") != "ddn.studio.safe_continuation_queue_recheck.v1":
        fail(f"queue schema mismatch: {queue.get('schema')!r}")
    candidate = queue.get("candidate", {})
    expected_candidate = {
        "id": "SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_V1",
        "kind": "private_productization",
        "source": "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
        "root_doc_exists": False,
        "pack_exists": False,
        "checker_exists": False,
        "release_execution_required": False,
        "requires_behavior_verification_if_implemented": True,
    }
    for key, value in expected_candidate.items():
        if candidate.get(key) != value:
            fail(f"candidate {key} expected {value!r}, got {candidate.get(key)!r}")
    for path in [
        ROOT / "SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_V1.md",
        ROOT / "pack" / "seamgrim_run_history_export_summary_v1",
        ROOT / "tests" / "run_seamgrim_run_history_export_summary_check.py",
    ]:
        if path.exists():
            fail(f"candidate unexpectedly exists before implementation: {path.relative_to(ROOT)}")
    authority = queue.get("current_authority", {})
    for key, value in {
        "studio_local_super_long": "9/18 = 50%",
        "roadmap_v2_behavior_closed": "90/90 = 100%",
        "roadmap_v2_docs_closed": "0/90 = 0%",
        "public_release_execution": "0/1 = 0% approval-gated",
    }.items():
        if authority.get(key) != value:
            fail(f"authority {key} expected {value!r}, got {authority.get(key)!r}")
    require_contains(ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md", ["SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_V1"])


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected_stdout = [
        "STUDIO_SAFE_CONTINUATION_QUEUE_RECHECK_V1",
        "safe continuation queue recheck sealed",
        "release execution selected: false",
        "public release execution: approval-gated",
        "safe candidate: SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_V1",
        "candidate exists: false",
        "studio local progress: 9/18 = 50%",
        "roadmap v2 behavior: 90/90 = 100%",
    ]
    if payload.get("cmd") != ["run", "pack/studio_safe_continuation_queue_recheck_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected_stdout:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_safe_continuation_queue_recheck_v1"],
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
    check_contract_and_queue()
    check_golden()
    run_required_gates()
    check_docs_ssot_clean()
    print("studio_safe_continuation_queue_recheck: ok")


if __name__ == "__main__":
    main()
