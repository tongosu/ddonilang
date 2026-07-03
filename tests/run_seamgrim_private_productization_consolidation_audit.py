from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "SEAMGRIM_PRIVATE_PRODUCTIZATION_CONSOLIDATION_AUDIT_V1.md"
REPORT = ROOT / "docs" / "studio" / "PRIVATE_PRODUCTIZATION_CONSOLIDATION_AUDIT_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
ROADMAP = ROOT / "docs" / "context" / "queue" / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "seamgrim_private_productization_consolidation_audit_v1"
CONTRACT = PACK / "contract.detjson"
AUDIT = PACK / "audit.detjson"


def fail(message: str) -> None:
    print(f"seamgrim_private_productization_consolidation_audit: FAIL: {message}", file=sys.stderr)
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
    audit = load_json(AUDIT)
    required = [
        DOC,
        REPORT,
        INDEX,
        ROADMAP,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        AUDIT,
    ]
    required.extend(ROOT / vector["source"] for vector in audit["audit_vectors"])
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        fail(f"missing required paths: {missing}")


def check_docs() -> None:
    common = [
        "SEAMGRIM_PRIVATE_PRODUCTIZATION_CONSOLIDATION_AUDIT_V1",
        "STUDIO_RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1",
        "9/18 = 50%",
        "90/90 = 100%",
        "0/90 = 0%",
        "0/1 = 0% approval-gated",
        "docs/ssot/**",
    ]
    require_contains(DOC, common + ["consolidated audit vectors: 4/4 = 100%", "numeric micro-slice containment: 3/3 = 100%"])
    require_contains(REPORT, common + ["Status: docs-closed consolidation audit", "Audit Vectors"])
    require_contains(ROADMAP, ["SEAMGRIM_PRIVATE_PRODUCTIZATION_CONSOLIDATION_AUDIT_V1", "STUDIO_RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1"])
    require_contains(
        INDEX,
        [
            "SEAMGRIM_PRIVATE_PRODUCTIZATION_CONSOLIDATION_AUDIT_V1",
            "SEAMGRIM_PRIVATE_PRODUCTIZATION_CONSOLIDATION_AUDIT_V1.md",
            "pack/seamgrim_private_productization_consolidation_audit_v1",
            "tests/run_seamgrim_private_productization_consolidation_audit.py",
            "docs/studio/PRIVATE_PRODUCTIZATION_CONSOLIDATION_AUDIT_V1.md",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "SEAMGRIM_PRIVATE_PRODUCTIZATION_CONSOLIDATION_AUDIT_V1",
            "seamgrim_private_productization_consolidation_audit_v1",
            "private productization consolidation audit 5/5 = 100%",
            "consolidated audit vectors: 4/4 = 100%",
            "numeric micro-slice containment: 3/3 = 100%",
            "STUDIO_RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1",
            "Studio-local 초장기 계획: 9/18 = 50%",
        ],
    )
    require_contains(PROJECT_STATUS, ["SEAMGRIM_PRIVATE_PRODUCTIZATION_CONSOLIDATION_AUDIT_V1", "consolidation audit `5/5 = 100%`"])
    require_contains(CHANGELOG, ["Seamgrim private productization consolidation audit", "consolidation audit is `5/5 = 100%`"])


def check_contract_and_audit() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_private_productization_consolidation_audit_v1",
        "kind": "seamgrim_private_productization_consolidation_audit",
        "closed_by": "SEAMGRIM_PRIVATE_PRODUCTIZATION_CONSOLIDATION_AUDIT_V1",
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
        "private_productization_chain_closed": 8,
        "private_productization_chain_total": 8,
        "export_summary_evidence_closed": 6,
        "export_summary_evidence_total": 6,
        "numeric_micro_slice_containment_closed": 3,
        "numeric_micro_slice_containment_total": 3,
        "audit_vectors_closed": 4,
        "audit_vectors_total": 4,
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
        "next_item": "STUDIO_RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    audit = load_json(AUDIT)
    if audit.get("schema") != "ddn.seamgrim.private_productization_consolidation_audit.v1":
        fail(f"audit schema mismatch: {audit.get('schema')!r}")
    if audit.get("next_safe_action") != "STUDIO_RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1":
        fail(f"next safe action mismatch: {audit.get('next_safe_action')!r}")
    if len(audit.get("audit_vectors", [])) != 4:
        fail(f"audit vector count mismatch: {audit.get('audit_vectors')!r}")
    authority = audit.get("current_authority", {})
    for key, value in {
        "studio_local_super_long": "9/18 = 50%",
        "roadmap_v2_behavior_closed": "90/90 = 100%",
        "roadmap_v2_docs_closed": "0/90 = 0%",
        "public_release_execution": "0/1 = 0% approval-gated",
    }.items():
        if authority.get(key) != value:
            fail(f"authority {key} expected {value!r}, got {authority.get(key)!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "SEAMGRIM_PRIVATE_PRODUCTIZATION_CONSOLIDATION_AUDIT_V1",
        "private productization consolidation audit sealed",
        "audit vectors: 4/4 = 100%",
        "numeric micro-slice containment: 3/3 = 100%",
        "next safe continuation: STUDIO_RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1",
        "public release execution: approval-gated",
        "studio local progress: 9/18 = 50%",
        "roadmap v2 behavior: 90/90 = 100%",
    ]
    if payload.get("cmd") != ["run", "pack/seamgrim_private_productization_consolidation_audit_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "seamgrim_private_productization_consolidation_audit_v1"],
        ["python", "tests/run_seamgrim_private_productization_next_queue_recheck.py"],
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
    print("seamgrim_private_productization_consolidation_audit: ok")


if __name__ == "__main__":
    main()
