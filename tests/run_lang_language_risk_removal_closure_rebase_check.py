from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_LANGUAGE_RISK_REMOVAL_CLOSURE_REBASE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_LANGUAGE_RISK_REMOVAL_CLOSURE_REBASE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_LANGUAGE_RISK_REMOVAL_CLOSURE_REBASE_20260606.md"
PACK = ROOT / "pack" / "lang_language_risk_removal_closure_rebase_v1"
MANIFEST = PACK / "language_risk_removal_closure_rebase.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_language_risk_removal_closure_rebase_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_IMPLEMENTATION_CLOSURE = ROOT / "pack" / "lang_implementation_followup_closure_rebase_v1" / "implementation_followup_closure_rebase.detjson"
SOURCE_FLOW = ROOT / "pack" / "lang_flow_history_alias_migration_plan_v1" / "flow_history_alias_migration_plan.detjson"
SOURCE_TUCK = ROOT / "pack" / "lang_tuck_constraint_surface_shape_proposal_v1" / "tuck_constraint_surface_shape_proposal.detjson"
SOURCE_VERLET = ROOT / "pack" / "lang_velocity_verlet_runtime_gate_rebase_v1" / "velocity_verlet_runtime_gate_rebase.detjson"
SOURCE_DULTRA = ROOT / "pack" / "lang_dultra_replay_artifact_implementation_gate_v1" / "dultra_replay_artifact_implementation_gate.detjson"
SOURCE_OWNER = ROOT / "pack" / "lang_owner_inner_seum_runtime_scope_rebase_v1" / "owner_inner_seum_runtime_scope_rebase.detjson"
OWNER_SCOPE_CHECKER = ROOT / "tests" / "run_lang_owner_inner_seum_runtime_scope_rebase_check.py"
DULTRA_GATE_CHECKER = ROOT / "tests" / "run_lang_dultra_replay_artifact_implementation_gate_check.py"

WORK_ITEM = "LANG_LANGUAGE_RISK_REMOVAL_CLOSURE_REBASE_V1"
NEXT = "LANG_PRIME_DERIVATIVE_RUNTIME_SEMANTICS_GATE_V1"
CLOSED_ITEMS = [
    ("LANG_IMPLEMENTATION_FOLLOWUP_CLOSURE_REBASE_V1", SOURCE_IMPLEMENTATION_CLOSURE),
    ("LANG_FLOW_HISTORY_ALIAS_MIGRATION_PLAN_V1", SOURCE_FLOW),
    ("LANG_TUCK_CONSTRAINT_SURFACE_SHAPE_PROPOSAL_V1", SOURCE_TUCK),
    ("LANG_VELOCITY_VERLET_RUNTIME_GATE_REBASE_V1", SOURCE_VERLET),
    ("LANG_DULTRA_REPLAY_ARTIFACT_IMPLEMENTATION_GATE_V1", SOURCE_DULTRA),
    ("LANG_OWNER_INNER_SEUM_RUNTIME_SCOPE_REBASE_V1", SOURCE_OWNER),
]
TRANSITION_QUEUE = [
    ("LANG_LANGUAGE_RISK_REMOVAL_CLOSURE_REBASE_V1", "closed"),
    ("LANG_PRIME_DERIVATIVE_RUNTIME_SEMANTICS_GATE_V1", "next"),
    ("LANG_OWNER_STATE_SYMBOL_TABLE_PRODUCT_PATH_V1", "planned"),
    ("LANG_HISTORY_ALIAS_STDLIB_BRIDGE_V1", "planned"),
    ("LANG_DULTRA_REPLAY_ARTIFACT_WRITER_SEED_V1", "planned"),
    ("LANG_VELOCITY_VERLET_STDLIB_SURFACE_ACCEPTANCE_V1", "planned"),
    ("LANG_TUCK_SSOT_ACCEPTANCE_HANDOFF_V1", "planned"),
]


def fail(message: str) -> None:
    print(f"lang_language_risk_removal_closure_rebase_check: FAIL: {message}", file=sys.stderr)
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


def run(cmd: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
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
        OWNER_SCOPE_CHECKER,
        DULTRA_GATE_CHECKER,
        *[path for _, path in CLOSED_ITEMS],
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        "language implementation risk-removal queue",
        "LANG_PRIME_DERIVATIVE_RUNTIME_SEMANTICS_GATE_V1",
        "언어 구현 위험 제거 closure rebase: 1/1 = 100%",
        "언어 제품 경로 구현 전환 계획: 1/7 = 14%",
        "ROADMAP_V2 전체: queue-expanded 60/90 = 67%",
        "No `docs/ssot/**` edit",
        "No derivative semantics landed claim",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "Closed Inputs", "Transition Queue", "1/7 = 14%", "60/90 = 67%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "6/6 language implementation risk-removal queue",
            "flow/history naming migration plan exists",
            "Define `위치'` and `위치''` runtime derivative semantics",
            "No derivative semantics landed claim",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.language_risk_removal_closure_rebase.v1",
            "lang_language_risk_removal_closure_rebase_v1",
            "다음 언어 구현 위험 제거 계획: 6/6 = 100%",
            "언어 구현 위험 제거 closure rebase: 1/1 = 100%",
            "언어 제품 경로 구현 전환 계획: 1/7 = 14%",
            "ROADMAP_V2 전체: queue-expanded 60/90 = 67%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_language_risk_removal_closure_rebase_v1",
        "kind": "lang_language_risk_removal_closure_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "language_risk_removal_closure_rebase_claim": True,
        "history_alias_landed_claim": False,
        "tuck_parser_runtime_landed_claim": False,
        "velocity_verlet_runtime_landed_claim": False,
        "velocity_verlet_stdlib_landed_claim": False,
        "dultra_replay_artifact_writer_landed_claim": False,
        "dultra_replay_verifier_landed_claim": False,
        "owner_inner_seum_runtime_landed_claim": False,
        "derivative_semantics_landed_claim": False,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_OWNER_INNER_SEUM_RUNTIME_SCOPE_REBASE_V1",
        "proposal_doc": "docs/context/proposals/LANG_LANGUAGE_RISK_REMOVAL_CLOSURE_REBASE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_LANGUAGE_RISK_REMOVAL_CLOSURE_REBASE_20260606.md",
        "decision_manifest": "pack/lang_language_risk_removal_closure_rebase_v1/language_risk_removal_closure_rebase.detjson",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "language_design_queue_closed": 8,
        "language_design_queue_total": 8,
        "language_design_queue_percent": 100,
        "implementation_readiness_rebase_closed": 1,
        "implementation_readiness_rebase_total": 1,
        "implementation_readiness_rebase_percent": 100,
        "implementation_readiness_followup_closed": 6,
        "implementation_readiness_followup_total": 6,
        "implementation_readiness_followup_percent": 100,
        "implementation_followup_closure_rebase_closed": 1,
        "implementation_followup_closure_rebase_total": 1,
        "implementation_followup_closure_rebase_percent": 100,
        "language_risk_removal_closed": 6,
        "language_risk_removal_total": 6,
        "language_risk_removal_percent": 100,
        "language_risk_removal_closure_rebase_closed": 1,
        "language_risk_removal_closure_rebase_total": 1,
        "language_risk_removal_closure_rebase_percent": 100,
        "language_product_path_transition_closed": 1,
        "language_product_path_transition_total": 7,
        "language_product_path_transition_percent": 14,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 60,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 67,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    for source_key in [
        "source_implementation_followup_closure_rebase",
        "source_flow_history_alias_migration_plan",
        "source_tuck_constraint_surface_shape_proposal",
        "source_velocity_verlet_runtime_gate_rebase",
        "source_dultra_replay_artifact_implementation_gate",
        "source_owner_inner_seum_runtime_scope_rebase",
    ]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.language_risk_removal_closure_rebase.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")

    expected_flags = {
        "language_risk_removal_closure_rebase_claim": True,
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "history_alias_landed_claim": False,
        "tuck_parser_runtime_landed_claim": False,
        "velocity_verlet_runtime_landed_claim": False,
        "velocity_verlet_stdlib_landed_claim": False,
        "dultra_replay_artifact_writer_landed_claim": False,
        "dultra_replay_verifier_landed_claim": False,
        "owner_inner_seum_runtime_landed_claim": False,
        "derivative_semantics_landed_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")

    closed = manifest.get("closed_risk_removal_items", [])
    closed_pairs = [(row.get("item"), ROOT / row.get("path", "")) for row in closed]
    if closed_pairs != CLOSED_ITEMS:
        fail(f"closed risk removal items mismatch: {closed_pairs!r}")
    for index, row in enumerate(closed, start=1):
        if row.get("order") != index or row.get("status") != "closed":
            fail(f"closed risk row malformed: {row!r}")
        require(ROOT / row["path"])

    summary_ids = [row.get("id") for row in manifest.get("closed_risk_summary", [])]
    expected_summary_ids = [
        "flow_history_naming",
        "tuck_constraint_layer",
        "velocity_verlet_runtime_gate",
        "dultra_replay_artifact_gate",
        "owner_inner_seum_runtime_scope",
    ]
    if summary_ids != expected_summary_ids:
        fail(f"closed risk summary mismatch: {summary_ids!r}")
    for row in manifest.get("closed_risk_summary", []):
        if row.get("runtime_landed") is not False:
            fail(f"closed risk summary must keep runtime_landed false: {row!r}")

    transition = [
        (row.get("item"), row.get("status"))
        for row in manifest.get("recommended_product_path_transition_queue", [])
    ]
    if transition != TRANSITION_QUEUE:
        fail(f"transition queue mismatch: {transition!r}")

    required_rationale = {
        "prime parser/frontdoor support already exists",
        "runtime derivative semantics block owner-inner 세움 execution",
        "runtime derivative semantics block deterministic integrator and 턱 ordering",
        "next unit must use product path or explicit product-path gate, not test-only lowering",
    }
    if set(manifest.get("next_item_rationale", [])) != required_rationale:
        fail("next item rationale mismatch")

    required_blocked = {
        "docs_ssot_edit",
        "parser_frontdoor_change",
        "runtime_surface_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "product_ui_change",
        "history_alias_landed",
        "tuck_parser_runtime_landed",
        "velocity_verlet_runtime_landed",
        "velocity_verlet_stdlib_landed",
        "dultra_replay_artifact_writer_landed",
        "dultra_replay_verifier_landed",
        "owner_inner_seum_runtime_landed",
        "derivative_semantics_landed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")

    expected_plans = {
        "language_risk_removal_plan": {"closed": 6, "total": 6, "percent": 100},
        "language_risk_removal_closure_rebase": {"closed": 1, "total": 1, "percent": 100},
        "language_product_path_transition_plan": {"closed": 1, "total": 7, "percent": 14},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 60, "total": 90, "percent": 67},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    expected_progress = [
        (SOURCE_IMPLEMENTATION_CLOSURE, {"closed": 1, "total": 6, "percent": 17}, "LANG_FLOW_HISTORY_ALIAS_MIGRATION_PLAN_V1"),
        (SOURCE_FLOW, {"closed": 2, "total": 6, "percent": 33}, "LANG_TUCK_CONSTRAINT_SURFACE_SHAPE_PROPOSAL_V1"),
        (SOURCE_TUCK, {"closed": 3, "total": 6, "percent": 50}, "LANG_VELOCITY_VERLET_RUNTIME_GATE_REBASE_V1"),
        (SOURCE_VERLET, {"closed": 4, "total": 6, "percent": 67}, "LANG_DULTRA_REPLAY_ARTIFACT_IMPLEMENTATION_GATE_V1"),
        (SOURCE_DULTRA, {"closed": 5, "total": 6, "percent": 83}, "LANG_OWNER_INNER_SEUM_RUNTIME_SCOPE_REBASE_V1"),
        (SOURCE_OWNER, {"closed": 6, "total": 6, "percent": 100}, "LANG_LANGUAGE_RISK_REMOVAL_CLOSURE_REBASE_V1"),
    ]
    for path, progress, next_item in expected_progress:
        data = load_json(path)
        if data.get("language_risk_removal_plan") != progress:
            fail(f"{path.relative_to(ROOT)} risk progress mismatch: {data.get('language_risk_removal_plan')!r}")
        if data.get("next_item") != next_item:
            fail(f"{path.relative_to(ROOT)} next item expected {next_item}, got {data.get('next_item')!r}")

    source_owner = load_json(SOURCE_OWNER)
    for key in [
        "owner_inner_seum_runtime_landed_claim",
        "seum_equation_solver_landed_claim",
        "derivative_semantics_landed_claim",
    ]:
        if source_owner.get(key) is not False:
            fail(f"owner runtime source {key} must remain false")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_language_risk_removal_closure_rebase_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    expected = [
        "LANG_LANGUAGE_RISK_REMOVAL_CLOSURE_REBASE_V1",
        "language risk removal closure rebase sealed",
        "schema: ddn.language.language_risk_removal_closure_rebase.v1",
        "closed risk removal items: 6",
        "product transition: 1/7 = 14%",
        "roadmap: 60/90 = 67%",
        "next: LANG_PRIME_DERIVATIVE_RUNTIME_SEMANTICS_GATE_V1",
    ]
    require_contains(PACK / "golden.jsonl", expected)


def check_previous_checkers() -> None:
    for checker in [OWNER_SCOPE_CHECKER, DULTRA_GATE_CHECKER]:
        proc = run([sys.executable, str(checker.relative_to(ROOT))], timeout=600)
        if proc.returncode != 0:
            fail(f"{checker.relative_to(ROOT)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_golden()
    check_previous_checkers()
    require_docs_ssot_clean()
    print("lang_language_risk_removal_closure_rebase_check: PASS")


if __name__ == "__main__":
    main()
