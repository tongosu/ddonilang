#!/usr/bin/env python3
"""Validate CHA0_RPG_SEED_REASSESSMENT_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "CHA0_RPG_SEED_REASSESSMENT_V1.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "차-0_REASSESSMENT_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_cha0_rpg_seed_reassessment_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"

SOURCE_PACKS = [
    "roadmap_v2_cha0_rpg_seed_rebase_v1",
    "malhim_rpg_1_v1",
    "malhim_rpg_2_v1",
    "malhim_rpg_3_v1",
    "malhim_rpg_4_v1",
    "malhim_rpg_5_v1",
]
SOURCE_CHECKERS = [
    "tests/run_roadmap_v2_cha1_rpg_phrase_action_smoke_check.py",
    "tests/run_roadmap_v2_cha2_rpg_story_pack_closure_check.py",
    "tests/run_roadmap_v2_cha3_rpg_box_authoring_ui_check.py",
    "tests/run_roadmap_v2_cha4_rpg_story_package_check.py",
    "tests/run_roadmap_v2_cha5_rpg_engine_adapter_lts_check.py",
]


def fail(message: str) -> None:
    print(f"[roadmap-v2-cha0-rpg-seed-reassessment] FAIL: {message}", file=sys.stderr)
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
        sys.stdout.buffer.write(proc.stdout.encode("utf-8", "replace"))
        fail(f"command failed: {' '.join(args)}")
    return proc


def section(path: Path, heading: str) -> str:
    text = read(path)
    start = text.find(heading)
    if start < 0:
        fail(f"{path.relative_to(ROOT)} missing section {heading}")
    next_heading = text.find("\n#### ", start + 1)
    if next_heading < 0:
        return text[start:]
    return text[start:next_heading]


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
        DOC,
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
        ROOT / "CHA0_RPG_SEED_REBASE_V1.md",
    ]:
        require_file(path)
    for pack in SOURCE_PACKS:
        require_file(ROOT / "pack" / pack / "golden.jsonl")
    for checker in SOURCE_CHECKERS:
        require_file(ROOT / checker)

    require_tokens(MATRIX, ["| 0마루 씨앗마루 | action/story 설계 | action_method, phrase binding | proposal/schema | 닫힘-동작 |"])
    cha0_section = section(GUIDE, "#### 차-0 — action/story 설계")
    if "| 현재 상태 | 닫힘-동작 |" not in cha0_section:
        fail("GUIDE 차-0 status is not 닫힘-동작")
    require_tokens(TRACKER, ["| 27 | `차-0` | 말힘누리/RPG seed reassessment | 닫힘-동작 |", "차-0_REASSESSMENT_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `차-0` | `roadmap_v2_cha0_rpg_seed_rebase_v1`; `roadmap_v2_cha0_rpg_seed_reassessment_v1`; downstream `malhim_rpg_1_v1`~`malhim_rpg_5_v1` |"])

    shared = [
        "CHA0_RPG_SEED_REASSESSMENT_V1",
        "CHA0 RPG seed reassessment 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 85/90 = 94%",
        "ROADMAP_V2 docs-closed: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 87/90 = 97%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "GA5_GRAMMAR_LTS_BEHAVIOR_RECHECK_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 CHA0 RPG seed reassessment", "85/90 = 94%", "87/90 = 97%"])

    total, behavior, docs = count_matrix_statuses()
    if total != 90 or behavior < 85 or docs != 5:
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
        "roadmap_v2_matrix_behavior_closed": 85,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 94,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 87,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 97,
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
        "pack": "roadmap_v2_cha0_rpg_seed_reassessment_v1",
        "kind": "roadmap_v2_cha0_rpg_seed_reassessment",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "new_rpg_behavior": False,
        "seed_reassessment_only": True,
        "new_parser_runtime_semantics": False,
        "registry_publish_claim": False,
        "native_runtime_execution_claim": False,
        "godot_project_build_claim": False,
        "public_upload_claim": False,
        "closed_by": "CHA0_RPG_SEED_REASSESSMENT_V1",
        "roadmap_coordinate": "차-0",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "cha0_previous_matrix_status": "씨앗",
        "cha0_matrix_status": "닫힘-동작",
        "next_item": "GA5_GRAMMAR_LTS_BEHAVIOR_RECHECK_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    if contract.get("source_evidence") != SOURCE_PACKS:
        fail(f"contract source_evidence={contract.get('source_evidence')!r}")
    check_payload(CONTRACT)

    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("status") != "behavior_closed":
        fail("reconciliation status mismatch")
    status_record = reconciliation.get("matrix_status_record", {})
    if status_record.get("old_status") != "씨앗" or status_record.get("new_status") != "닫힘-동작":
        fail(f"matrix status record mismatch: {status_record!r}")
    evidence = reconciliation.get("evidence", {})
    if evidence.get("seed_boundary", {}).get("pack") != "roadmap_v2_cha0_rpg_seed_rebase_v1":
        fail("missing seed boundary evidence")
    packs = evidence.get("downstream_product_path", {}).get("packs")
    if packs != SOURCE_PACKS[1:]:
        fail(f"downstream packs mismatch: {packs!r}")
    for key, value in reconciliation.get("false_claims", {}).items():
        if value is not False:
            fail(f"false claim {key}={value!r}")
    check_payload(RECONCILIATION)


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 86/90",
        '"roadmap_v2_matrix_behavior_closed": 86',
        '"new_rpg_behavior": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
        '"new_parser_runtime_semantics": true',
        '"registry_publish_claim": true',
        '"native_runtime_execution_claim": true',
        '"godot_project_build_claim": true',
        '"public_upload_claim": true',
    ]
    for path in [DOC, REPORT, PACK / "README.md", CONTRACT, RECONCILIATION]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_cha0_rpg_seed_reassessment_v1"], timeout=120)
    run([sys.executable, "tests/run_roadmap_v2_cha1_rpg_phrase_action_smoke_check.py"], timeout=900)
    run([sys.executable, "tests/run_roadmap_v2_cha2_rpg_story_pack_closure_check.py"], timeout=900)
    run([sys.executable, "tests/run_roadmap_v2_cha3_rpg_box_authoring_ui_check.py"], timeout=900)
    run([sys.executable, "tests/run_roadmap_v2_cha4_rpg_story_package_check.py"], timeout=900)
    run([sys.executable, "tests/run_roadmap_v2_cha5_rpg_engine_adapter_lts_check.py"], timeout=900)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-cha0-rpg-seed-reassessment] OK")


if __name__ == "__main__":
    main()
