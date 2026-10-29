from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_IMPLEMENTATION_READINESS_REBASE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_IMPLEMENTATION_READINESS_REBASE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_IMPLEMENTATION_READINESS_REBASE_20260606.md"
PACK = ROOT / "pack" / "lang_implementation_readiness_rebase_v1"
MANIFEST = PACK / "implementation_readiness_rebase.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_implementation_readiness_rebase_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PRIOR_CHECKER = ROOT / "tests" / "run_lang_dstrict_dultra_solver_strategy_proposal_check.py"
PRIOR_MANIFEST = ROOT / "pack" / "lang_dstrict_dultra_solver_strategy_proposal_v1" / "dstrict_dultra_solver_strategy_proposal.detjson"
NEXT = "LANG_PRIME_PARSER_FRONTDOOR_SPIKE_V1"

SOURCE_PATHS = [
    ROOT / "pack" / "language_design_priority_rebase_v1" / "language_design_priority_rebase.detjson",
    ROOT / "pack" / "lang_prime_derivative_notation_decision_v1" / "prime_derivative_notation_decision.detjson",
    ROOT / "pack" / "lang_flow_type_collision_rename_v1" / "flow_type_collision_rename.detjson",
    ROOT / "pack" / "lang_sim_constraint_third_layer_name_v1" / "sim_constraint_third_layer_name.detjson",
    ROOT / "pack" / "lang_seum_vol3_prime_example_pack_v1" / "seum_vol3_prime_example_pack.detjson",
    ROOT / "pack" / "lang_owner_inner_seum_structure_check_v1" / "owner_inner_seum_structure_check.detjson",
    ROOT / "pack" / "lang_connect_lowering_to_seum_check_v1" / "connect_lowering_to_seum_check.detjson",
    PRIOR_MANIFEST,
]


def fail(message: str) -> None:
    print(f"lang_implementation_readiness_rebase_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")


def load_json(path: Path) -> dict:
    return json.loads(read(path))


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
        PRIOR_CHECKER,
    ] + SOURCE_PATHS:
        require(path)


def check_docs() -> None:
    tokens = [
        "LANG_IMPLEMENTATION_READINESS_REBASE_V1",
        "LANG_PRIME_PARSER_FRONTDOOR_SPIKE_V1",
        "LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1",
        "LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_V1",
        "LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_V1",
        "LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1",
        "위치'",
        "이력",
        "턱",
        "잇기",
        "속도 베를레",
        "새 언어 설계 안정화 계획: 8/8 = 100%",
        "언어 구현 준비 후속 계획: 1/6 = 17%",
        "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
        "ROADMAP_V2 전체: queue-expanded 48/90 = 53%",
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, tokens[:11] + ["product frontdoor", "Test-only Python/JS lowering"])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "Prime derivative notation",
            "No parser/frontdoor landed claim",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "LANG_IMPLEMENTATION_READINESS_REBASE_V1",
            "ddn.language.implementation_readiness_rebase.v1",
            "lang_implementation_readiness_rebase_v1",
            "언어 구현 준비 후속 계획: 1/6 = 17%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_implementation_readiness_rebase_v1",
        "kind": "lang_implementation_readiness_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "implementation_readiness_rebase_claim": True,
        "prime_parser_landed_claim": False,
        "flow_history_runtime_rename_landed_claim": False,
        "tuck_constraint_parser_landed_claim": False,
        "connect_lowering_parser_landed_claim": False,
        "velocity_verlet_runtime_landed_claim": False,
        "dultra_recorded_replay_landed_claim": False,
        "closed_by": "LANG_IMPLEMENTATION_READINESS_REBASE_V1",
        "based_on": "LANG_DSTRICT_DULTRA_SOLVER_STRATEGY_PROPOSAL_V1",
        "proposal_doc": "docs/context/proposals/LANG_IMPLEMENTATION_READINESS_REBASE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_IMPLEMENTATION_READINESS_REBASE_20260606.md",
        "decision_manifest": "pack/lang_implementation_readiness_rebase_v1/implementation_readiness_rebase.detjson",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "language_design_queue_closed": 8,
        "language_design_queue_total": 8,
        "language_design_queue_percent": 100,
        "implementation_readiness_rebase_closed": 1,
        "implementation_readiness_rebase_total": 1,
        "implementation_readiness_rebase_percent": 100,
        "implementation_readiness_followup_closed": 1,
        "implementation_readiness_followup_total": 6,
        "implementation_readiness_followup_percent": 17,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
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
    if manifest.get("schema") != "ddn.language.implementation_readiness_rebase.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "LANG_IMPLEMENTATION_READINESS_REBASE_V1":
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
        "prime_parser_landed_claim",
        "flow_history_runtime_rename_landed_claim",
        "tuck_constraint_parser_landed_claim",
        "connect_lowering_parser_landed_claim",
        "velocity_verlet_runtime_landed_claim",
        "dultra_recorded_replay_landed_claim",
    ]:
        if manifest.get(flag) is not False:
            fail(f"manifest {flag} expected false, got {manifest.get(flag)!r}")
    if manifest.get("implementation_readiness_rebase_claim") is not True:
        fail("implementation readiness claim must be true")
    source_items = manifest.get("source_items", [])
    if len(source_items) != 8:
        fail(f"expected 8 source items, got {len(source_items)}")
    source_paths = {item.get("path") for item in source_items}
    expected_source_paths = {str(path.relative_to(ROOT)).replace("\\", "/") for path in SOURCE_PATHS}
    if source_paths != expected_source_paths:
        fail(f"source paths mismatch: {source_paths!r}")
    for item in source_items:
        if item.get("status") != "closed" or item.get("runtime_landed") is not False or item.get("ssot_landed") is not False:
            fail(f"source item not closed/no-claim: {item!r}")
    classifications = {
        row.get("id"): row for row in manifest.get("readiness_classifications", [])
    }
    expected_classifications = {
        "prime_derivative_notation": ("parser_spike_ready_after_ssot_or_explicit_user_approval", NEXT),
        "flow_type_collision": ("migration_plan_required_before_runtime_rename", "LANG_FLOW_HISTORY_ALIAS_MIGRATION_PLAN_V1"),
        "sim_constraint_third_layer": ("ssot_acceptance_required_before_parser_shape", "LANG_TUCK_CONSTRAINT_SURFACE_SHAPE_PROPOSAL_V1"),
        "seum_vol3_prime_examples": ("depends_on_prime_parser_support", NEXT),
        "owner_inner_seum": ("parser_boundary_spike_ready_after_prime_scope", "LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1"),
        "connect_lowering_to_seum": ("parser_lowering_spike_ready_without_new_block", "LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1"),
        "dstrict_dultra_solver_strategy": ("pack_spec_ready_before_runtime", "LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_V1"),
    }
    if set(classifications) != set(expected_classifications):
        fail(f"classification ids mismatch: {set(classifications)!r}")
    for key, (classification, next_item) in expected_classifications.items():
        row = classifications[key]
        if row.get("classification") != classification or row.get("next") != next_item:
            fail(f"classification mismatch for {key}: {row!r}")
    queue = manifest.get("recommended_next_queue", [])
    expected_queue = [
        ("LANG_IMPLEMENTATION_READINESS_REBASE_V1", "closed"),
        (NEXT, "next"),
        ("LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1", "planned"),
        ("LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_V1", "planned"),
        ("LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_V1", "planned"),
        ("LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1", "planned"),
    ]
    if [(row.get("item"), row.get("status")) for row in queue] != expected_queue:
        fail(f"queue mismatch: {queue!r}")
    gate = manifest.get("implementation_gate_policy", {})
    if gate != {
        "product_path_required": True,
        "test_only_lowering_disallowed": True,
        "checker_required": True,
        "d_pack_required": True,
        "docs_ssot_edit_disallowed": True,
    }:
        fail(f"gate policy mismatch: {gate!r}")
    required_blocked = {
        "docs_ssot_edit",
        "parser_frontdoor_change",
        "runtime_surface_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "ssot_landed_claim",
        "runtime_landed_claim",
        "compat_break_without_migration",
        "connect_block_landed",
        "velocity_verlet_runtime_landed",
        "dultra_recorded_replay_landed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")
    expected_plans = {
        "language_design_queue_plan": {"closed": 8, "total": 8, "percent": 100},
        "implementation_readiness_rebase_plan": {"closed": 1, "total": 1, "percent": 100},
        "implementation_readiness_followup_plan": {"closed": 1, "total": 6, "percent": 17},
        "urgent_recommendations_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 48, "total": 90, "percent": 53},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"{key} mismatch: {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    for path in SOURCE_PATHS:
        load_json(path)
    prior = load_json(PRIOR_MANIFEST)
    if prior.get("next_item") != "LANG_IMPLEMENTATION_READINESS_REBASE_V1":
        fail(f"prior next_item mismatch: {prior.get('next_item')!r}")
    if prior.get("queue_plan") != {"closed": 8, "total": 8, "percent": 100}:
        fail(f"prior queue plan mismatch: {prior.get('queue_plan')!r}")


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_implementation_readiness_rebase_v1"], timeout=180)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_previous_checker() -> None:
    proc = run([sys.executable, "tests/run_lang_dstrict_dultra_solver_strategy_proposal_check.py"], timeout=180)
    if proc.returncode != 0:
        fail(f"prior checker failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_pack_golden()
    check_previous_checker()
    require_docs_ssot_clean()
    print("lang_implementation_readiness_rebase_check: PASS")


if __name__ == "__main__":
    main()
