#!/usr/bin/env python3
"""Validate A0_NURIGYM_SCHEMA_SKELETON_MATRIX_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "아-0_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_a0_nurigym_schema_skeleton_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"
SPEC_PACK = ROOT / "pack" / "gogae5_w47_nurigym_observation_spec"
EXPECTED_OBS_HASH = "sha256:cb4733790acfd96ca558d51324e5e3703637f71e031e9d2b945957479ce1211b"
EXPECTED_ACTION_HASH = "sha256:f429769842eb3ea2034d40727bcea85bcca272298e5a08fbea5332efade14985"


def fail(message: str) -> None:
    print(f"[roadmap-v2-a0-nurigym-schema-skeleton] FAIL: {message}", file=sys.stderr)
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


def section(path: Path, heading: str) -> str:
    text = read(path)
    start = text.find(heading)
    if start < 0:
        fail(f"{path.relative_to(ROOT)} missing section {heading}")
    next_heading = text.find("\n#### ", start + 1)
    if next_heading < 0:
        return text[start:]
    return text[start:next_heading]


def check_docs() -> None:
    for path in [
        MATRIX,
        GUIDE,
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
        RECONCILIATION,
        SPEC_PACK / "README.md",
        SPEC_PACK / "input.ddn",
        SPEC_PACK / "golden.jsonl",
        ROOT / "tests" / "run_nuri_gym_contract_check.py",
    ]:
        require_file(path)
    require_tokens(MATRIX, ["| 0마루 씨앗마루 | Gym schema skeleton | reset/step/obs/reward/info | nurigym schema docs | 닫힘-동작 |"])
    a0_section = section(GUIDE, "#### 아-0 — Gym schema skeleton")
    if "| 현재 상태 | 닫힘-동작 |" not in a0_section:
        fail("GUIDE 아-0 status is not 닫힘-동작")
    da0_section = section(GUIDE, "#### 다-0 — math/proof 범위 확정")
    if "| 현재 상태 |" not in da0_section:
        fail("GUIDE 다-0 status row missing")
    require_tokens(TRACKER, ["| 16.45 | `아-0` | Gym schema skeleton | 닫힘-동작 |", "아-0_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `아-0` | `gogae5_w47_nurigym_observation_spec`; `nuri_gym_canon_contract_v1`; `nuri_gym_gridworld_v1`; `roadmap_v2_a0_nurigym_schema_skeleton_v1` |"])
    shared = [
        "A0_NURIGYM_SCHEMA_SKELETON_MATRIX_RECONCILIATION_V1",
        "A0 NuriGym schema skeleton 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 79/90 = 88%",
        "ROADMAP_V2 docs-closed: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 81/90 = 90%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "SA0_BOGAE_SCHEMA_BOUNDARY_RECONCILIATION_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 A0 NuriGym schema skeleton matrix reconciliation", "79/90 = 88%", "81/90 = 90%"])
    total, behavior, docs = count_matrix_statuses()
    if total != 90 or behavior < 79 or docs < 5:
        fail(f"matrix counts mismatch: rows={total} behavior={behavior} docs={docs}")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "work_unit_closed": 5,
        "work_unit_total": 5,
        "work_unit_percent": 100,
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 79,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 88,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 81,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 90,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
    }
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}, expected {value!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_a0_nurigym_schema_skeleton_v1",
        "kind": "roadmap_v2_a0_nurigym_schema_skeleton",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "nurigym_runtime_change": False,
        "reset_step_first_run_claim": False,
        "representative_environment_golden_claim": False,
        "python_web_parity_claim": False,
        "dataset_registry_claim": False,
        "training_workflow_claim": False,
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "roadmap_coordinate": "아-0",
        "next_item": "SA0_BOGAE_SCHEMA_BOUNDARY_RECONCILIATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    check_payload(CONTRACT)
    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("status") != "behavior_closed":
        fail("reconciliation status mismatch")
    if reconciliation.get("matrix_status_record", {}).get("new_status") != "닫힘-동작":
        fail("matrix status record mismatch")
    hashes = reconciliation.get("spec_hashes", {})
    if hashes.get("obs_spec_hash") != EXPECTED_OBS_HASH:
        fail("obs spec hash mismatch")
    if hashes.get("action_spec_hash") != EXPECTED_ACTION_HASH:
        fail("action spec hash mismatch")
    for key, value in reconciliation.get("false_claims", {}).items():
        if value is not False:
            fail(f"false claim {key}={value!r}")
    check_payload(RECONCILIATION)


def check_spec_golden() -> None:
    lines = [json.loads(line) for line in read(SPEC_PACK / "golden.jsonl").splitlines() if line.strip()]
    if len(lines) != 1:
        fail("unexpected gogae5_w47 golden row count")
    stdout = lines[0].get("stdout", [])
    if f"obs_spec_hash={EXPECTED_OBS_HASH}" not in stdout:
        fail("gogae5_w47 missing expected obs spec hash")
    if f"action_spec_hash={EXPECTED_ACTION_HASH}" not in stdout:
        fail("gogae5_w47 missing expected action spec hash")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 80/90",
        '"roadmap_v2_matrix_behavior_closed": 80',
        '"nurigym_runtime_change": true',
        '"reset_step_first_run_claim": true',
        '"representative_environment_golden_claim": true',
        '"python_web_parity_claim": true',
        '"dataset_registry_claim": true',
        '"training_workflow_claim": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
    ]
    for path in [REPORT, PACK / "README.md", CONTRACT, RECONCILIATION]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_a0_nurigym_schema_skeleton_v1"], timeout=120)
    run([sys.executable, "tests/run_pack_golden.py", "gogae5_w47_nurigym_observation_spec"], timeout=120)
    run([sys.executable, "tests/run_nuri_gym_contract_check.py"], timeout=600)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_contracts()
    check_spec_golden()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-a0-nurigym-schema-skeleton] OK")


if __name__ == "__main__":
    main()
