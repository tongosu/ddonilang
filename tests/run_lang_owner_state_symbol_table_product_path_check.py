from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_OWNER_STATE_SYMBOL_TABLE_PRODUCT_PATH_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_OWNER_STATE_SYMBOL_TABLE_PRODUCT_PATH_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_OWNER_STATE_SYMBOL_TABLE_PRODUCT_PATH_20260606.md"
PACK = ROOT / "pack" / "lang_owner_state_symbol_table_product_path_v1"
MANIFEST = PACK / "owner_state_symbol_table_product_path.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_owner_state_symbol_table_product_path_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_PRIME_GATE = ROOT / "pack" / "lang_prime_derivative_runtime_semantics_gate_v1" / "prime_derivative_runtime_semantics_gate.detjson"
SOURCE_OWNER_SCOPE = ROOT / "pack" / "lang_owner_inner_seum_runtime_scope_rebase_v1" / "owner_inner_seum_runtime_scope_rebase.detjson"
PRIME_GATE_CHECKER = ROOT / "tests" / "run_lang_prime_derivative_runtime_semantics_gate_check.py"

WORK_ITEM = "LANG_OWNER_STATE_SYMBOL_TABLE_PRODUCT_PATH_V1"
NEXT = "LANG_HISTORY_ALIAS_STDLIB_BRIDGE_V1"
SAMPLE_ROWS = [
    "owner=공;symbol=위치;type=수;kind=state;initializer=0",
    "owner=공;symbol=속도;type=수;kind=state;initializer=0",
    "owner=공;symbol=이름;type=글;kind=constant;initializer=\"공\"",
]


def fail(message: str) -> None:
    print(f"lang_owner_state_symbol_table_product_path_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_PRIME_GATE,
        SOURCE_OWNER_SCOPE,
        PRIME_GATE_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        "owner_state_symbol_table_rows",
        "owner-local state symbol table",
        "언어 제품 경로 구현 전환 계획: 3/7 = 43%",
        "Owner state symbol table product path: 1/1 = 100%",
        "ROADMAP_V2 전체: queue-expanded 62/90 = 69%",
        "No `docs/ssot/**` edit",
        "No runtime owner state symbol table landed claim",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "Product Path", "3/7 = 43%", "62/90 = 69%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "owner_state_symbol_table_rows(source)",
            "AST currently does not preserve",
            "No runtime or stdlib surface change",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.owner_state_symbol_table_product_path.v1",
            "lang_owner_state_symbol_table_product_path_v1",
            "언어 제품 경로 구현 전환 계획: 3/7 = 43%",
            "Owner state symbol table product path: 1/1 = 100%",
            "ROADMAP_V2 전체: queue-expanded 62/90 = 69%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_owner_state_symbol_table_product_path_v1",
        "kind": "lang_owner_state_symbol_table_product_path",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "owner_state_symbol_table_product_path_claim": True,
        "owner_state_symbol_table_helper_landed_claim": True,
        "runtime_owner_state_symbol_table_landed_claim": False,
        "derivative_semantics_landed_claim": False,
        "derivative_runtime_product_path_landed_claim": False,
        "seum_equation_solver_landed_claim": False,
        "owner_inner_seum_runtime_landed_claim": False,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_PRIME_DERIVATIVE_RUNTIME_SEMANTICS_GATE_V1",
        "proposal_doc": "docs/context/proposals/LANG_OWNER_STATE_SYMBOL_TABLE_PRODUCT_PATH_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_OWNER_STATE_SYMBOL_TABLE_PRODUCT_PATH_20260606.md",
        "decision_manifest": "pack/lang_owner_state_symbol_table_product_path_v1/owner_state_symbol_table_product_path.detjson",
        "source_prime_derivative_runtime_semantics_gate": "pack/lang_prime_derivative_runtime_semantics_gate_v1/prime_derivative_runtime_semantics_gate.detjson",
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
        "language_product_path_transition_closed": 3,
        "language_product_path_transition_total": 7,
        "language_product_path_transition_percent": 43,
        "owner_state_symbol_table_product_path_closed": 1,
        "owner_state_symbol_table_product_path_total": 1,
        "owner_state_symbol_table_product_path_percent": 100,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 62,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 69,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    for source_key in ["source_prime_derivative_runtime_semantics_gate", "source_owner_inner_seum_runtime_scope"]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.owner_state_symbol_table_product_path.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")

    expected_flags = {
        "owner_state_symbol_table_product_path_claim": True,
        "owner_state_symbol_table_helper_landed_claim": True,
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "runtime_owner_state_symbol_table_landed_claim": False,
        "derivative_semantics_landed_claim": False,
        "derivative_runtime_product_path_landed_claim": False,
        "seum_equation_solver_landed_claim": False,
        "owner_inner_seum_runtime_landed_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")

    helper = manifest.get("product_helper", {})
    expected_helper = {
        "api": "ddonirang_lang::owner_state_symbol_table_rows",
        "path": "lang/src/frontdoor.rs",
        "export_path": "lang/src/lib.rs",
        "product_path": True,
        "runtime_landed": False,
        "parser_grammar_changed": False,
    }
    for key, value in expected_helper.items():
        if helper.get(key) != value:
            fail(f"helper {key} expected {value!r}, got {helper.get(key)!r}")

    row_contract = manifest.get("row_contract", {})
    expected_row_contract = {
        "format": "owner=<owner>;symbol=<name>;type=<type>;kind=<state|constant>;initializer=<expr|<unset>>",
        "ordering": "source_order_within_owner_body",
        "owner_scope": "top_level_seed_kind_named_imja_only",
        "event_body_policy": "receive_bodies_excluded",
        "surface_keyword_preserved": False,
    }
    for key, value in expected_row_contract.items():
        if row_contract.get(key) != value:
            fail(f"row contract {key} expected {value!r}, got {row_contract.get(key)!r}")

    if manifest.get("sample_rows") != SAMPLE_ROWS:
        fail(f"sample rows mismatch: {manifest.get('sample_rows')!r}")

    for row in manifest.get("product_anchor_rows", []):
        path = ROOT / row.get("path", "")
        require(path)
        require_contains(path, row.get("tokens", []))

    required_blocked = {
        "docs_ssot_edit",
        "parser_frontdoor_grammar_change",
        "runtime_surface_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "product_ui_change",
        "runtime_owner_state_symbol_table_landed",
        "derivative_semantics_landed",
        "derivative_runtime_product_path_landed",
        "seum_equation_solver_landed",
        "owner_inner_seum_runtime_landed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")

    expected_plans = {
        "language_product_path_transition_plan": {"closed": 3, "total": 7, "percent": 43},
        "owner_state_symbol_table_product_path": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 62, "total": 90, "percent": 69},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    prime = load_json(SOURCE_PRIME_GATE)
    if prime.get("next_item") != WORK_ITEM:
        fail(f"prime gate next item expected {WORK_ITEM}, got {prime.get('next_item')!r}")
    if prime.get("language_product_path_transition_plan") != {"closed": 2, "total": 7, "percent": 29}:
        fail(f"prime gate product transition progress mismatch: {prime.get('language_product_path_transition_plan')!r}")
    if prime.get("derivative_semantics_landed_claim") is not False:
        fail("prime gate derivative semantics landed claim must remain false")

    owner = load_json(SOURCE_OWNER_SCOPE)
    if owner.get("owner_inner_seum_runtime_landed_claim") is not False:
        fail("owner scope runtime landed claim must remain false")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_owner_state_symbol_table_product_path_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    expected = [
        "LANG_OWNER_STATE_SYMBOL_TABLE_PRODUCT_PATH_V1",
        "owner state symbol table product path sealed",
        "schema: ddn.language.owner_state_symbol_table_product_path.v1",
        "product helper: owner_state_symbol_table_rows",
        "product transition: 3/7 = 43%",
        "runtime landed: false",
        "next: LANG_HISTORY_ALIAS_STDLIB_BRIDGE_V1",
    ]
    require_contains(PACK / "golden.jsonl", expected)


def check_product_test() -> None:
    proc = run(
        [
            "cargo",
            "test",
            "-p",
            "ddonirang-lang",
            "imja_owner_state_symbol_table_rows_stay_owner_scoped",
            "--quiet",
        ],
        timeout=300,
    )
    if proc.returncode != 0:
        fail(f"cargo product helper test failed:\n{proc.stdout}")


def check_previous_checker() -> None:
    proc = run([sys.executable, str(PRIME_GATE_CHECKER.relative_to(ROOT))], timeout=900)
    if proc.returncode != 0:
        fail(f"{PRIME_GATE_CHECKER.relative_to(ROOT)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_golden()
    check_product_test()
    check_previous_checker()
    require_docs_ssot_clean()
    print("lang_owner_state_symbol_table_product_path_check: PASS")


if __name__ == "__main__":
    main()
