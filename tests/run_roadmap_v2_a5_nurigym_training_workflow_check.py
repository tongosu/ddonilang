#!/usr/bin/env python3
"""Validate A5_NURIGYM_TRAINING_WORKFLOW_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "아-5_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "nurigym_training_workflow_v1"
CONTRACT = PACK / "contract.detjson"
WORKFLOW = PACK / "workflow.detjson"
V25 = ROOT / "pack" / "v25_first_ai_production_path_v1"


def fail(message: str) -> None:
    print(f"[roadmap-v2-a5-nurigym-training-workflow] FAIL: {message}", file=sys.stderr)
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
        WORKFLOW,
        V25 / "golden.jsonl",
        V25 / "train_config.json",
        V25 / "expected" / "dataset_hash.txt",
        ROOT / "tests" / "run_train_eval_provenance_check.py",
        ROOT / "tests" / "run_v25_first_ai_production_path_provenance_check.py",
        ROOT / "tests" / "run_roadmap_v2_a3_nurigym_python_web_parity_check.py",
        ROOT / "tests" / "run_roadmap_v2_a4_dataset_registry_check.py",
    ]:
        require_file(path)
    require_tokens(MATRIX, ["| 5마루 단단마루 | training workflow | curriculum RL, benchmark | training workflow suite | 닫힘-동작 |"])
    require_tokens(TRACKER, ["| 16.85 | `아-5` | training workflow | 닫힘-동작 |", "아-5_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `아-5` | `nurigym_training_workflow_v1`; `v25_first_ai_production_path_v1`; `gogae8_w82_seulgi_train_v2`; `gogae8_w87_eval_suite_v1`; `model_artifact_closure_v1`; `nuri_gym_dataset_registry_v1`; `nurigym_python_web_parity_v1` |"])
    shared = [
        "A5_NURIGYM_TRAINING_WORKFLOW_V1",
        "A5 NuriGym training workflow 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 78/90 = 87%",
        "ROADMAP_V2 docs-closed: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 80/90 = 89%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_GLOBAL_REMAINING_MATRIX_REAUDIT_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 A5 NuriGym training workflow closure", "78/90 = 87%", "80/90 = 89%"])
    total, behavior, docs = count_matrix_statuses()
    if total != 90 or behavior < 78 or docs < 5:
        fail(f"matrix counts mismatch: rows={total} behavior={behavior} docs={docs}")


def check_contract_and_workflow() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "nurigym_training_workflow_v1",
        "kind": "roadmap_v2_a5_nurigym_training_workflow",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "A5_NURIGYM_TRAINING_WORKFLOW_V1",
        "roadmap_coordinate": "아-5",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 78,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 87,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 80,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 89,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "workflow_manifest": "pack/nurigym_training_workflow_v1/workflow.detjson",
        "next_item": "ROADMAP_V2_GLOBAL_REMAINING_MATRIX_REAUDIT_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    for key, value in contract.get("false_claims", {}).items():
        if value is not False:
            fail(f"contract false claim {key}={value!r}")

    workflow = read_json(WORKFLOW)
    if workflow.get("schema") != "ddn.nurigym.training_workflow.v1":
        fail("workflow schema mismatch")
    if workflow.get("coordinate") != "아-5":
        fail("workflow coordinate mismatch")
    if workflow.get("status") != "bounded_training_workflow_ready":
        fail("workflow status mismatch")
    ids = [row.get("id") for row in workflow.get("workflow_rows", [])]
    if ids != ["dataset_registry", "python_web_parity", "train_artifact", "eval_artifact", "runtime_infer"]:
        fail(f"workflow rows mismatch: {ids!r}")
    expected_hashes = {
        "dataset_hash": "sha256:34289c2282de6f9b8f4869a20fa48dff68ff4f3b49f2d0340178c321b2aad313",
        "train_config_hash": "sha256:f1fa1631454ff53d02298dbd6520517f6413efe5822da99bb4581d4f266c01ca",
        "train_report_hash": "sha256:699d0d818e88825e03f171bb9a20b915198bdee263ea41eea48e04c147eefb94",
        "weights_hash": "sha256:78b34a7b30d2edc742e93a731c8b42cf84ac5065620fb24197aeb43cb4a2541a",
        "model_hash": "blake3:f69dbfcb8ca34df7d92fc0569aa08cbb5ef402adf640089e1ea31d6d235313ba",
    }
    text = json.dumps(workflow, ensure_ascii=False, sort_keys=True)
    for value in expected_hashes.values():
        if value not in text:
            fail(f"workflow missing hash {value}")
    for key, value in workflow.get("false_claims", {}).items():
        if value is not False:
            fail(f"workflow false claim {key}={value!r}")


def check_v25_outputs() -> None:
    dataset_hash = (V25 / "expected" / "dataset_hash.txt").read_text(encoding="utf-8").strip()
    if dataset_hash != "sha256:34289c2282de6f9b8f4869a20fa48dff68ff4f3b49f2d0340178c321b2aad313":
        fail(f"v25 dataset hash mismatch: {dataset_hash}")
    train_config = read_json(V25 / "train_config.json")
    if train_config.get("dataset_hash") != dataset_hash:
        fail("v25 train_config dataset hash mismatch")
    train_report = read_json(V25 / "out" / "train" / "train.report.detjson")
    if train_report.get("pass") is not True or train_report.get("score") != 115:
        fail(f"v25 train report mismatch: {train_report!r}")
    if train_report.get("source_provenance", {}).get("dataset_hash") != dataset_hash:
        fail("v25 train report dataset provenance mismatch")
    infer = read_json(V25 / "out" / "infer" / "infer.output.detjson")
    if infer.get("model_hash") != "blake3:f69dbfcb8ca34df7d92fc0569aa08cbb5ef402adf640089e1ea31d6d235313ba":
        fail("v25 infer model hash mismatch")
    if infer.get("output") != [17352]:
        fail(f"v25 infer output mismatch: {infer.get('output')!r}")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 79/90",
        '"roadmap_v2_matrix_behavior_closed": 79',
        '"online_rl_training_claim": true',
        '"broad_curriculum_rl_claim": true',
        '"optimizer_autodiff_claim": true',
        '"public_model_registry_publish_claim": true',
        '"cloud_execution_claim": true',
        '"nurigym_runtime_change": true',
        '"runtime_claim": true',
        '"product_code_change": true',
        '"product_ui_change": true',
    ]
    for path in [REPORT, PACK / "README.md", CONTRACT, WORKFLOW]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "nurigym_training_workflow_v1", "v25_first_ai_production_path_v1"], timeout=240)
    run([sys.executable, "tests/run_train_eval_provenance_check.py"], timeout=600)
    run([sys.executable, "tests/run_v25_first_ai_production_path_provenance_check.py"], timeout=900)
    check_v25_outputs()
    run([sys.executable, "tests/run_roadmap_v2_a3_nurigym_python_web_parity_check.py"], timeout=600)
    run([sys.executable, "tests/run_roadmap_v2_a4_dataset_registry_check.py"], timeout=900)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_contract_and_workflow()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-a5-nurigym-training-workflow] OK")


if __name__ == "__main__":
    main()
