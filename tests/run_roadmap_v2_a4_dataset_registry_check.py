#!/usr/bin/env python3
"""Validate A4_NURIGYM_DATASET_REGISTRY_V1."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "아-4_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "nuri_gym_dataset_registry_v1"
CONTRACT = PACK / "contract.detjson"
REGISTRY = PACK / "registry.detjson"

EXPECTED_HASHES = {
    "w53_dataset_export": {
        "dataset_hash": "sha256:5458d126393789f68fc92822de7d0ce788b22db2e62363f59b392d66c67a7b99",
        "source_audit_hash": "blake3:2a7e18dcce782f96e3a409f816f2ef75499993511ea6cfee954f123e6949ba40",
    },
    "w55_smart_errand_dataset": {
        "dataset_hash": "sha256:1c9159dfbdd505c74023b5ebd042235ec88f1b1c7bd9b6647500581552358075",
        "source_audit_hash": "blake3:6c2833ef911bf9cbb9e15a252ecd572ddaa4ce3f1387e3eb0e4555a07966bdaf",
    },
}


def fail(message: str) -> None:
    print(f"[roadmap-v2-a4-dataset-registry] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    try:
        payload = json.loads(read(path))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be JSON object")
    return payload


def load_first_jsonl(path: Path) -> dict:
    for line in read(path).splitlines():
        if line.strip():
            payload = json.loads(line)
            if not isinstance(payload, dict):
                fail(f"{path.relative_to(ROOT)} first JSONL record must be object")
            return payload
    fail(f"{path.relative_to(ROOT)} is empty")


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def require_tokens(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing {missing}")


def run(args: list[str], *, timeout: float | None = None) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )
    if proc.returncode != 0:
        print(proc.stdout, end="")
        fail(f"command failed: {' '.join(args)}")
    return proc


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return f"sha256:{digest.hexdigest()}"


def count_matrix_statuses() -> tuple[int, int, int]:
    rows = []
    for line in read(MATRIX).splitlines():
        if not line.startswith("| ") or "마루" not in line or line.startswith("| 마루"):
            continue
        cols = [col.strip() for col in line.strip().strip("|").split("|")]
        if len(cols) == 5 and cols[0] and cols[0][0] in "012345" and "마루" in cols[0]:
            rows.append(cols)
    return (
        len(rows),
        sum(1 for row in rows if row[-1] == "닫힘-동작"),
        sum(1 for row in rows if row[-1] == "닫힘-문서"),
    )


def check_docs() -> None:
    for path in [
        MATRIX,
        TRACKER,
        MANIFEST,
        REPORT,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        REGISTRY,
        ROOT / "pack" / "gogae5_w53_dataset_export" / "golden.jsonl",
        ROOT / "pack" / "gogae5_w55_smart_errand_integration" / "golden.jsonl",
        ROOT / "tests" / "run_dataset_export_v0_provenance_check.py",
    ]:
        require_file(path)
    require_tokens(MATRIX, ["| 4마루 나눔마루 | dataset registry | episode detjsonl, registry | dataset publish pack | 닫힘-동작 |"])
    require_tokens(TRACKER, ["| 16.75 | `아-4` | dataset registry | 닫힘-동작 |", "아-4_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `아-4` | `nuri_gym_dataset_registry_v1`; `gogae5_w53_dataset_export`; `gogae5_w55_smart_errand_integration` |"])
    shared = [
        "A4_NURIGYM_DATASET_REGISTRY_V1",
        "A4 NuriGym dataset registry 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 76/90 = 84%",
        "ROADMAP_V2 docs-closed: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 78/90 = 87%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_A3_NURIGYM_PYTHON_WEB_PARITY_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 A4 NuriGym dataset registry closure", "76/90 = 84%", "78/90 = 87%"])
    total, behavior, docs = count_matrix_statuses()
    if total != 90 or behavior < 76 or docs < 5:
        fail(f"matrix counts mismatch: rows={total} behavior={behavior} docs={docs}")


def check_contract() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "nuri_gym_dataset_registry_v1",
        "kind": "roadmap_v2_a4_nurigym_dataset_registry",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "A4_NURIGYM_DATASET_REGISTRY_V1",
        "roadmap_coordinate": "아-4",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 76,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 84,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 78,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 87,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "registry_manifest": "pack/nuri_gym_dataset_registry_v1/registry.detjson",
        "runner": "tests/run_dataset_export_v0_provenance_check.py",
        "next_item": "ROADMAP_V2_A3_NURIGYM_PYTHON_WEB_PARITY_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    source_packs = contract.get("source_packs")
    if source_packs != ["gogae5_w53_dataset_export", "gogae5_w55_smart_errand_integration"]:
        fail(f"source_packs mismatch: {source_packs!r}")
    for key, value in contract.get("false_claims", {}).items():
        if value is not False:
            fail(f"contract false claim {key}={value!r}")


def check_registry_payload() -> None:
    registry = read_json(REGISTRY)
    if registry.get("schema") != "ddn.nurigym.dataset_registry.v1":
        fail("registry schema mismatch")
    if registry.get("coordinate") != "아-4":
        fail("registry coordinate mismatch")
    if registry.get("status") != "local_registry_ready":
        fail("registry status mismatch")
    if registry.get("publish_scope") != "local_pack_registry_only":
        fail("registry publish scope mismatch")
    rows = registry.get("rows")
    if not isinstance(rows, list):
        fail("registry rows must be list")
    by_id = {row.get("id"): row for row in rows if isinstance(row, dict)}
    if set(by_id) != set(EXPECTED_HASHES):
        fail(f"registry rows mismatch: {sorted(by_id)!r}")
    for row_id, expected in EXPECTED_HASHES.items():
        row = by_id[row_id]
        if row.get("dataset_hash") != expected["dataset_hash"]:
            fail(f"{row_id} dataset_hash mismatch")
        if row.get("source_audit_hash") != expected["source_audit_hash"]:
            fail(f"{row_id} source_audit_hash mismatch")
        if row.get("ready") is not True:
            fail(f"{row_id} ready must be true")
    progress = registry.get("progress", {})
    expected_progress = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 76,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 84,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 78,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 87,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
    }
    for key, value in expected_progress.items():
        if progress.get(key) != value:
            fail(f"registry progress {key}={progress.get(key)!r}, expected {value!r}")
    for key, value in registry.get("false_claims", {}).items():
        if value is not False:
            fail(f"registry false claim {key}={value!r}")


def check_dataset_outputs() -> None:
    registry = read_json(REGISTRY)
    rows = {row["id"]: row for row in registry["rows"]}
    for row_id, row in rows.items():
        dataset_path = ROOT / row["dataset_path"]
        manifest_path = ROOT / row["geoul_path"] / "manifest.detjson"
        require_file(dataset_path)
        require_file(manifest_path)
        actual_hash = sha256_file(dataset_path)
        if actual_hash != row["dataset_hash"]:
            fail(f"{row_id} file sha256 {actual_hash}, expected {row['dataset_hash']}")
        manifest = read_json(manifest_path)
        header = load_first_jsonl(dataset_path)
        if header.get("schema") != "nurigym.dataset.v1":
            fail(f"{row_id} dataset schema mismatch")
        if header.get("source_hash") != manifest.get("audit_hash"):
            fail(f"{row_id} header source_hash mismatch")
        if header.get("source_hash") != row["source_audit_hash"]:
            fail(f"{row_id} registry source_audit_hash mismatch")
        provenance = header.get("source_provenance", {})
        if provenance.get("schema") != "nurigym.source_provenance.v1":
            fail(f"{row_id} provenance schema mismatch")
        for key in ["audit_hash", "entry_file", "entry_hash", "age_target_source", "age_target_value"]:
            if provenance.get(key) != manifest.get(key):
                fail(f"{row_id} provenance {key} mismatch")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 77/90",
        '"roadmap_v2_matrix_behavior_closed": 77',
        '"public_registry_publish_claim": true',
        '"network_sync_claim": true',
        '"cloud_storage_claim": true',
        '"account_permission_claim": true',
        '"python_web_parity_claim": true',
        '"training_workflow_claim": true',
        '"nurigym_runtime_change": true',
        '"runtime_claim": true',
        '"product_code_change": true',
        '"product_ui_change": true',
    ]
    for path in [REPORT, PACK / "README.md", CONTRACT, REGISTRY]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "nuri_gym_dataset_registry_v1"], timeout=120)
    run([sys.executable, "tests/run_dataset_export_v0_provenance_check.py"], timeout=600)
    check_dataset_outputs()
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_contract()
    check_registry_payload()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-a4-dataset-registry] OK")


if __name__ == "__main__":
    main()
