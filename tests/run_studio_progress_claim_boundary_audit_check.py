from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_PROGRESS_CLAIM_BOUNDARY_AUDIT_V1.md"
REPORT = ROOT / "docs" / "studio" / "PROGRESS_CLAIM_BOUNDARY_AUDIT_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "studio_progress_claim_boundary_audit_v1"
CONTRACT = PACK / "contract.detjson"
AUDIT = PACK / "audit.detjson"


def fail(message: str) -> None:
    print(f"studio_progress_claim_boundary_audit_check: FAIL: {message}", file=sys.stderr)
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


def require_files() -> None:
    audit = load_json(AUDIT)
    required = [
        DOC,
        REPORT,
        INDEX,
        DEV_SUMMARY,
        PACK / "README.md",
        CONTRACT,
        AUDIT,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
    ]
    required.extend(ROOT / path for path in audit["stale_progress_paths"])
    required.extend(ROOT / path for path in audit.get("repaired_progress_paths", []))
    required.extend(ROOT / path for path in audit["allowed_progress_exception_paths"])
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        fail(f"missing required paths: {missing}")


def require_contains(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing tokens: {missing}")


def count_stale_matches(audit: dict) -> tuple[int, int, list[str]]:
    patterns = audit["stale_progress_patterns"]
    paths = audit["stale_progress_paths"]
    file_count = 0
    match_count = 0
    no_match: list[str] = []
    for item in paths:
        text = read(ROOT / item)
        matches = sum(text.count(pattern) for pattern in patterns)
        if matches:
            file_count += 1
            match_count += matches
        else:
            no_match.append(item)
    return file_count, match_count, no_match


def check_docs() -> None:
    common = [
        "STUDIO_PROGRESS_CLAIM_BOUNDARY_AUDIT_V1",
        "9/18 = 50%",
        "90/90 = 100%",
        "12",
        "0",
        "12/12 = 100%",
        "docs/ssot/**",
    ]
    require_contains(DOC, common + ["닫힘-문서", "stale progress repair: 12/12 files normalized = 100%"])
    require_contains(REPORT, common + ["docs-closed audit", "RUNTIME_5MIN_BASELINE_REPAIR_V1.md"])
    require_contains(
        INDEX,
        [
            "STUDIO_PROGRESS_CLAIM_BOUNDARY_AUDIT_V1",
            "pack/studio_progress_claim_boundary_audit_v1",
            "tests/run_studio_progress_claim_boundary_audit_check.py",
            "docs/studio/PROGRESS_CLAIM_BOUNDARY_AUDIT_V1.md",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_PROGRESS_CLAIM_BOUNDARY_AUDIT_V1",
            "studio_progress_claim_boundary_audit_v1",
            "stale progress claim boundary audit 4/4 = 100%",
            "stale progress repair: 12/12 = 100%",
            "Studio-local 초장기 계획: 9/18 = 50%",
        ],
    )


def check_contract_and_audit() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_progress_claim_boundary_audit_v1",
        "kind": "studio_progress_claim_boundary_audit",
        "closed_by": "STUDIO_PROGRESS_CLAIM_BOUNDARY_AUDIT_V1",
        "closure_tier": "닫힘-문서",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "behavior_closed_claim": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "benchmark_execution_claim": False,
        "lts_certification_claim": False,
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
        "stale_progress_file_count": 0,
        "stale_progress_match_count": 0,
        "stale_progress_repair_closed": 12,
        "stale_progress_repair_total": 12,
        "requires_docs_ssot_clean": True,
        "next_item": "STUDIO_PROGRESS_CLAIM_BOUNDARY_REPAIR_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    audit = load_json(AUDIT)
    if audit.get("schema") != "ddn.studio.progress_claim_boundary_audit.v1":
        fail(f"audit schema mismatch: {audit.get('schema')!r}")
    if audit.get("official_progress", {}).get("studio_local_super_long") != "9/18 = 50%":
        fail(f"audit official progress mismatch: {audit.get('official_progress')!r}")
    if audit.get("original_stale_progress_file_count") != 12:
        fail(f"audit original stale count mismatch: {audit.get('original_stale_progress_file_count')!r}")
    if len(audit.get("stale_progress_paths", [])) != 0:
        fail("audit stale path count mismatch")
    if len(audit.get("repaired_progress_paths", [])) != 12:
        fail("audit repaired path count mismatch")
    file_count, match_count, no_match = count_stale_matches(audit)
    if no_match:
        fail(f"audit stale paths without matches: {no_match}")
    if file_count != audit.get("stale_progress_file_count"):
        fail(f"stale file count expected {audit.get('stale_progress_file_count')}, got {file_count}")
    if match_count != audit.get("stale_progress_match_count"):
        fail(f"stale match count expected {audit.get('stale_progress_match_count')}, got {match_count}")
    if audit.get("repair_progress") != {"closed": 12, "total": 12, "percent": 100}:
        fail(f"repair progress mismatch: {audit.get('repair_progress')!r}")
    repaired_audit = {**audit, "stale_progress_paths": audit.get("repaired_progress_paths", [])}
    repaired_file_count, repaired_match_count, _ = count_stale_matches(repaired_audit)
    if repaired_file_count or repaired_match_count:
        fail(
            "repaired stale paths still contain stale progress patterns: "
            f"files={repaired_file_count}, matches={repaired_match_count}"
        )
    exception = ROOT / audit["allowed_progress_exception_paths"][0]
    require_contains(exception, ["runtime 5-minute smoke: 18/18 = 100%"])


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_PROGRESS_CLAIM_BOUNDARY_AUDIT_V1",
        "studio progress claim boundary audit sealed",
        "stale progress remaining files: 0/12",
        "stale progress remaining matches: 0",
        "repair progress: 12/12 = 100%",
        "official studio local progress: 9/18 = 50%",
        "roadmap v2 behavior: 90/90 = 100%",
        "next: progress claim boundary repair",
    ]
    if payload.get("cmd") != ["run", "pack/studio_progress_claim_boundary_audit_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_progress_claim_boundary_audit_v1"],
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
    check_contract_and_audit()
    check_golden()
    run_required_gates()
    check_docs_ssot_clean()
    print("studio_progress_claim_boundary_audit_check: ok")


if __name__ == "__main__":
    main()
