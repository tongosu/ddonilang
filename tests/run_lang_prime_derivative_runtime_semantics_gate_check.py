from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_PRIME_DERIVATIVE_RUNTIME_SEMANTICS_GATE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_PRIME_DERIVATIVE_RUNTIME_SEMANTICS_GATE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_PRIME_DERIVATIVE_RUNTIME_SEMANTICS_GATE_20260606.md"
PACK = ROOT / "pack" / "lang_prime_derivative_runtime_semantics_gate_v1"
MANIFEST = PACK / "prime_derivative_runtime_semantics_gate.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_prime_derivative_runtime_semantics_gate_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_CLOSURE = ROOT / "pack" / "lang_language_risk_removal_closure_rebase_v1" / "language_risk_removal_closure_rebase.detjson"
SOURCE_PRIME = ROOT / "pack" / "lang_prime_parser_frontdoor_spike_v1" / "prime_parser_frontdoor_spike.detjson"
SOURCE_OWNER = ROOT / "pack" / "lang_owner_inner_seum_runtime_scope_rebase_v1" / "owner_inner_seum_runtime_scope_rebase.detjson"
CLOSURE_CHECKER = ROOT / "tests" / "run_lang_language_risk_removal_closure_rebase_check.py"
PRIME_CHECKER = ROOT / "tests" / "run_lang_prime_parser_frontdoor_spike_check.py"

WORK_ITEM = "LANG_PRIME_DERIVATIVE_RUNTIME_SEMANTICS_GATE_V1"
NEXT = "LANG_OWNER_STATE_SYMBOL_TABLE_PRODUCT_PATH_V1"
GATES = [
    ("parser_frontdoor_acceptance", "closed"),
    ("runtime_slot_classification", "planned"),
    ("tick_interval_policy", "planned"),
    ("state_history_snapshot", "planned_after_tick_policy"),
    ("relation_solver_boundary", "planned_after_owner_scope"),
    ("integrator_order_boundary", "planned_after_solver_boundary"),
    ("diagnostic_contract", "planned_after_runtime_shape"),
]
TRANSITION_QUEUE = [
    ("LANG_LANGUAGE_RISK_REMOVAL_CLOSURE_REBASE_V1", "closed"),
    ("LANG_PRIME_DERIVATIVE_RUNTIME_SEMANTICS_GATE_V1", "closed"),
    ("LANG_OWNER_STATE_SYMBOL_TABLE_PRODUCT_PATH_V1", "next"),
    ("LANG_HISTORY_ALIAS_STDLIB_BRIDGE_V1", "planned"),
    ("LANG_DULTRA_REPLAY_ARTIFACT_WRITER_SEED_V1", "planned"),
    ("LANG_VELOCITY_VERLET_STDLIB_SURFACE_ACCEPTANCE_V1", "planned"),
    ("LANG_TUCK_SSOT_ACCEPTANCE_HANDOFF_V1", "planned"),
]


def fail(message: str) -> None:
    print(f"lang_prime_derivative_runtime_semantics_gate_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_CLOSURE,
        SOURCE_PRIME,
        SOURCE_OWNER,
        CLOSURE_CHECKER,
        PRIME_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        "Runtime derivative semantics",
        "runtime_slot_classification",
        "tick_interval_policy",
        "state_history_snapshot",
        "relation_solver_boundary",
        "integrator_order_boundary",
        "diagnostic_contract",
        "언어 제품 경로 구현 전환 계획: 2/7 = 29%",
        "Prime derivative runtime semantics gate: 1/1 = 100%",
        "ROADMAP_V2 전체: queue-expanded 61/90 = 68%",
        "No `docs/ssot/**` edit",
        "No derivative semantics landed claim",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "Required Gate Decisions", "2/7 = 29%", "61/90 = 68%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "`base'` is a first-order derivative slot",
            "`base''` is a second-order derivative slot",
            "`base'''` remains invalid",
            "No derivative semantics landed claim",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.prime_derivative_runtime_semantics_gate.v1",
            "lang_prime_derivative_runtime_semantics_gate_v1",
            "언어 제품 경로 구현 전환 계획: 2/7 = 29%",
            "Prime derivative runtime semantics gate: 1/1 = 100%",
            "ROADMAP_V2 전체: queue-expanded 61/90 = 68%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_prime_derivative_runtime_semantics_gate_v1",
        "kind": "lang_prime_derivative_runtime_semantics_gate",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "prime_derivative_runtime_semantics_gate_claim": True,
        "derivative_semantics_landed_claim": False,
        "derivative_runtime_product_path_landed_claim": False,
        "seum_equation_solver_landed_claim": False,
        "owner_inner_seum_runtime_landed_claim": False,
        "velocity_verlet_runtime_landed_claim": False,
        "velocity_verlet_stdlib_landed_claim": False,
        "tuck_parser_runtime_landed_claim": False,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_LANGUAGE_RISK_REMOVAL_CLOSURE_REBASE_V1",
        "proposal_doc": "docs/context/proposals/LANG_PRIME_DERIVATIVE_RUNTIME_SEMANTICS_GATE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_PRIME_DERIVATIVE_RUNTIME_SEMANTICS_GATE_20260606.md",
        "decision_manifest": "pack/lang_prime_derivative_runtime_semantics_gate_v1/prime_derivative_runtime_semantics_gate.detjson",
        "source_language_risk_removal_closure": "pack/lang_language_risk_removal_closure_rebase_v1/language_risk_removal_closure_rebase.detjson",
        "source_prime_parser_frontdoor_spike": "pack/lang_prime_parser_frontdoor_spike_v1/prime_parser_frontdoor_spike.detjson",
        "source_owner_inner_seum_runtime_scope": "pack/lang_owner_inner_seum_runtime_scope_rebase_v1/owner_inner_seum_runtime_scope_rebase.detjson",
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
        "language_product_path_transition_closed": 2,
        "language_product_path_transition_total": 7,
        "language_product_path_transition_percent": 29,
        "prime_derivative_runtime_semantics_gate_closed": 1,
        "prime_derivative_runtime_semantics_gate_total": 1,
        "prime_derivative_runtime_semantics_gate_percent": 100,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 61,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 68,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    for source_key in [
        "source_language_risk_removal_closure",
        "source_prime_parser_frontdoor_spike",
        "source_owner_inner_seum_runtime_scope",
    ]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.prime_derivative_runtime_semantics_gate.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")

    expected_flags = {
        "prime_derivative_runtime_semantics_gate_claim": True,
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "derivative_semantics_landed_claim": False,
        "derivative_runtime_product_path_landed_claim": False,
        "seum_equation_solver_landed_claim": False,
        "owner_inner_seum_runtime_landed_claim": False,
        "velocity_verlet_runtime_landed_claim": False,
        "velocity_verlet_stdlib_landed_claim": False,
        "tuck_parser_runtime_landed_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")

    slots = manifest.get("runtime_derivative_slot_contract", [])
    slot_pairs = [(row.get("surface"), row.get("order"), row.get("ordinary_identifier"), row.get("runtime_landed")) for row in slots]
    expected_slots = [("base'", 1, False, False), ("base''", 2, False, False)]
    if slot_pairs != expected_slots:
        fail(f"slot contract mismatch: {slot_pairs!r}")

    gates = [(row.get("id"), row.get("status")) for row in manifest.get("semantics_gates", [])]
    if gates != GATES:
        fail(f"semantics gate mismatch: {gates!r}")
    gate_orders = [row.get("order") for row in manifest.get("semantics_gates", [])]
    if gate_orders != [1, 2, 3, 4, 5, 6, 7]:
        fail(f"semantics gate order mismatch: {gate_orders!r}")

    allowed = set(manifest.get("allowed_after_gate", []))
    if allowed != {
        "prime runtime semantics gate is specified",
        "base prime surfaces are derivative slots by contract",
        "executable derivative semantics are still unlanded",
    }:
        fail(f"allowed claims mismatch: {allowed!r}")
    forbidden = set(manifest.get("forbidden_after_gate", []))
    if forbidden != {
        "derivative_semantics_landed",
        "derivative_runtime_product_path_landed",
        "세움_equation_solver_landed",
        "owner_inner_seum_runtime_landed",
        "velocity_verlet_runtime_landed",
        "tuck_runtime_landed",
    }:
        fail(f"forbidden claims mismatch: {forbidden!r}")

    for row in manifest.get("product_anchor_rows", []):
        path = ROOT / row.get("path", "")
        require(path)
        require_contains(path, row.get("tokens", []))
        if row.get("changed_now") is not False:
            fail(f"product anchor row must not be changed_now: {row!r}")

    required_blocked = {
        "docs_ssot_edit",
        "parser_frontdoor_change",
        "runtime_surface_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "product_ui_change",
        "derivative_semantics_landed",
        "derivative_runtime_product_path_landed",
        "seum_equation_solver_landed",
        "owner_inner_seum_runtime_landed",
        "velocity_verlet_runtime_landed",
        "velocity_verlet_stdlib_landed",
        "tuck_parser_runtime_landed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")

    expected_plans = {
        "language_product_path_transition_plan": {"closed": 2, "total": 7, "percent": 29},
        "prime_derivative_runtime_semantics_gate": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 61, "total": 90, "percent": 68},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    closure = load_json(SOURCE_CLOSURE)
    if closure.get("next_item") != WORK_ITEM:
        fail(f"closure next item expected {WORK_ITEM}, got {closure.get('next_item')!r}")
    if closure.get("language_product_path_transition_plan") != {"closed": 1, "total": 7, "percent": 14}:
        fail(f"closure product transition progress mismatch: {closure.get('language_product_path_transition_plan')!r}")
    if closure.get("derivative_semantics_landed_claim") is not False:
        fail("closure derivative semantics landed claim must remain false")

    prime = load_json(SOURCE_PRIME)
    expected_prime = {
        "prime_identifier_parser_acceptance_landed_claim": True,
        "prime_derivative_semantics_landed_claim": False,
        "seum_equation_solver_landed_claim": False,
        "owner_inner_seum_landed_claim": False,
    }
    for key, value in expected_prime.items():
        if prime.get(key) != value:
            fail(f"prime source {key} expected {value!r}, got {prime.get(key)!r}")
    accepted = prime.get("accepted_surfaces", [])
    if [(row.get("surface"), row.get("derivative_order"), row.get("semantics_landed")) for row in accepted] != [
        ("위치'", 1, False),
        ("위치''", 2, False),
    ]:
        fail(f"accepted prime surfaces mismatch: {accepted!r}")

    owner = load_json(SOURCE_OWNER)
    if owner.get("next_item") != "LANG_LANGUAGE_RISK_REMOVAL_CLOSURE_REBASE_V1":
        fail(f"owner source next item mismatch: {owner.get('next_item')!r}")
    if owner.get("derivative_semantics_landed_claim") is not False:
        fail("owner source derivative semantics landed claim must remain false")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_prime_derivative_runtime_semantics_gate_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    expected = [
        "LANG_PRIME_DERIVATIVE_RUNTIME_SEMANTICS_GATE_V1",
        "prime derivative runtime semantics gate sealed",
        "schema: ddn.language.prime_derivative_runtime_semantics_gate.v1",
        "required semantics gates: 7",
        "product transition: 2/7 = 29%",
        "runtime landed: false",
        "next: LANG_OWNER_STATE_SYMBOL_TABLE_PRODUCT_PATH_V1",
    ]
    require_contains(PACK / "golden.jsonl", expected)


def check_previous_checkers() -> None:
    for checker in [CLOSURE_CHECKER, PRIME_CHECKER]:
        proc = run([sys.executable, str(checker.relative_to(ROOT))], timeout=700)
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
    print("lang_prime_derivative_runtime_semantics_gate_check: PASS")


if __name__ == "__main__":
    main()
