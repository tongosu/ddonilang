from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANGUAGE_DESIGN_PRIORITY_REBASE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANGUAGE_DESIGN_PRIORITY_REBASE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANGUAGE_DESIGN_PRIORITY_REBASE_20260606.md"
PACK = ROOT / "pack" / "language_design_priority_rebase_v1"
MANIFEST = PACK / "language_design_priority_rebase.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_language_design_priority_rebase_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
NEXT = "LANG_PRIME_DERIVATIVE_NOTATION_DECISION_V1"


def fail(message: str) -> None:
    print(f"language_design_priority_rebase_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")


def require(path: Path) -> None:
    if not path.exists():
        fail(f"missing required path: {path.relative_to(ROOT)}")


def require_contains(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing tokens: {missing}")


def run(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
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


def load_json(path: Path) -> dict:
    return json.loads(read(path))


def require_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed: {proc.stdout.strip()}")
    status_lines = [
        line for line in proc.stdout.splitlines()
        if line.strip() and not line.startswith("warning:")
    ]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def expected_queue_ids() -> list[str]:
    return [
        "LANGUAGE_DESIGN_PRIORITY_REBASE_V1",
        "LANG_PRIME_DERIVATIVE_NOTATION_DECISION_V1",
        "LANG_FLOW_TYPE_COLLISION_RENAME_V1",
        "LANG_SIM_CONSTRAINT_THIRD_LAYER_NAME_V1",
        "LANG_SEUM_VOL3_PRIME_EXAMPLE_PACK_V1",
        "LANG_OWNER_INNER_SEUM_STRUCTURE_CHECK_V1",
        "LANG_CONNECT_LOWERING_TO_SEUM_CHECK_V1",
        "LANG_DSTRICT_DULTRA_SOLVER_STRATEGY_PROPOSAL_V1",
    ]


def check_required_files() -> None:
    for path in [
        DOC,
        PROPOSAL,
        SSOT_NOTE,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        MANIFEST,
        CHECKER,
        DEV_SUMMARY,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "LANGUAGE_DESIGN_PRIORITY_REBASE_V1",
        "LANG_PRIME_DERIVATIVE_NOTATION_DECISION_V1",
        "위치'",
        "위치''",
        "이력",
        "턱",
        "잇기",
        "세움",
        "docs/ssot/**",
        "새 언어 설계 안정화 계획: 1/8 = 13%",
        "긴급 언어 결정 후보 추천: 3/3 = 100%",
        "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, tokens[:8])
    require_contains(
        SSOT_NOTE,
        [
            "Derivative notation",
            "Flow naming collision",
            "Third simulation constraint layer",
            "위치'",
            "이력",
            "턱",
            "Codex did not edit `docs/ssot/**`",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "LANGUAGE_DESIGN_PRIORITY_REBASE_V1",
            "language_design_priority_rebase_v1",
            "ddn.language.design_priority_rebase.v1",
            "새 언어 설계 안정화 계획: 1/8 = 13%",
            "긴급 언어 결정 후보 추천: 3/3 = 100%",
            "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "language_design_priority_rebase_v1",
        "kind": "language_design_priority_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "language_design_priority_rebase_claim": True,
        "prime_notation_landed_claim": False,
        "flow_type_rename_landed_claim": False,
        "constraint_layer_landed_claim": False,
        "connect_block_claim": False,
        "connect_lowering_landed_claim": False,
        "seum_vol3_example_landed_claim": False,
        "owner_inner_seum_landed_claim": False,
        "dstrict_verlet_landed_claim": False,
        "dultra_solver_strategy_landed_claim": False,
        "closed_by": "LANGUAGE_DESIGN_PRIORITY_REBASE_V1",
        "proposal_doc": "docs/context/proposals/LANGUAGE_DESIGN_PRIORITY_REBASE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANGUAGE_DESIGN_PRIORITY_REBASE_20260606.md",
        "priority_manifest": "pack/language_design_priority_rebase_v1/language_design_priority_rebase.detjson",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "studio_new_closed": 6,
        "studio_new_total": 8,
        "studio_new_percent": 75,
        "language_design_queue_closed": 1,
        "language_design_queue_total": 8,
        "language_design_queue_percent": 13,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 48,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 53,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.design_priority_rebase.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "LANGUAGE_DESIGN_PRIORITY_REBASE_V1":
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    for flag in [
        "runtime_claim",
        "product_code_change",
        "product_ui_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "parser_frontdoor_change",
        "stdlib_surface_change",
        "ssot_edit_claim",
    ]:
        if manifest.get(flag) is not False:
            fail(f"manifest {flag} expected false, got {manifest.get(flag)!r}")

    guardrail_ids = [row.get("id") for row in manifest.get("identity_guardrails", [])]
    expected_guardrails = [
        "deterministic_evidence_core",
        "operator_triad_fixed",
        "declarative_surfaces_first",
        "connect_is_seum_sugar",
    ]
    if guardrail_ids != expected_guardrails:
        fail(f"guardrail ids mismatch: {guardrail_ids!r}")

    recs = {row.get("id"): row for row in manifest.get("urgent_recommendations", [])}
    if recs.get("prime_derivative_notation", {}).get("preferred") != "위치'":
        fail("prime derivative preferred notation mismatch")
    if recs.get("prime_derivative_notation", {}).get("second_order") != "위치''":
        fail("second order prime notation mismatch")
    if recs.get("flow_type_collision", {}).get("rename_ring_buffer_to") != "이력":
        fail("flow type rename recommendation mismatch")
    if recs.get("sim_constraint_third_layer", {}).get("preferred") != "턱":
        fail("constraint layer recommendation mismatch")
    for row in recs.values():
        if row.get("ssot_landed") is not False or row.get("runtime_landed") is not False:
            fail(f"recommendation must not claim landed: {row!r}")

    queue_ids = [row.get("id") for row in manifest.get("implementation_queue", [])]
    if queue_ids != expected_queue_ids():
        fail(f"implementation queue mismatch: {queue_ids!r}")
    statuses = [row.get("status") for row in manifest.get("implementation_queue", [])]
    if statuses[0] != "closed" or statuses[1] != "next" or any(status != "planned" for status in statuses[2:]):
        fail(f"queue statuses mismatch: {statuses!r}")

    if manifest.get("queue_plan") != {"closed": 1, "total": 8, "percent": 13}:
        fail(f"queue plan mismatch: {manifest.get('queue_plan')!r}")
    if manifest.get("urgent_recommendation_plan") != {"closed": 3, "total": 3, "percent": 100}:
        fail(f"urgent recommendation plan mismatch: {manifest.get('urgent_recommendation_plan')!r}")
    if manifest.get("urgent_ssot_landed_plan") != {"closed": 0, "total": 3, "percent": 0}:
        fail(f"urgent SSOT plan mismatch: {manifest.get('urgent_ssot_landed_plan')!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")
    required_blocked = {
        "docs_ssot_edit",
        "parser_frontdoor_change",
        "runtime_surface_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "connect_block_addition",
        "prime_notation_runtime_landed",
        "flow_type_rename_runtime_landed",
        "constraint_layer_runtime_landed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "LANGUAGE_DESIGN_PRIORITY_REBASE_V1",
        "language design priority rebase sealed",
        "priority schema: ddn.language.design_priority_rebase.v1",
        "language queue: 1/8 = 13%",
        "urgent recommendations: 3/3 = 100%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/language_design_priority_rebase_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    proc = run(["python", "tests/run_pack_golden.py", "language_design_priority_rebase_v1"], timeout=180)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("language_design_priority_rebase_check: ok")


if __name__ == "__main__":
    main()
