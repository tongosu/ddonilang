from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_OWNER_INNER_SEUM_RUNTIME_SCOPE_REBASE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_OWNER_INNER_SEUM_RUNTIME_SCOPE_REBASE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_OWNER_INNER_SEUM_RUNTIME_SCOPE_REBASE_20260606.md"
PACK = ROOT / "pack" / "lang_owner_inner_seum_runtime_scope_rebase_v1"
MANIFEST = PACK / "owner_inner_seum_runtime_scope_rebase.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_owner_inner_seum_runtime_scope_rebase_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_OWNER = ROOT / "pack" / "lang_owner_inner_seum_parser_boundary_spike_v1" / "owner_inner_seum_parser_boundary_spike.detjson"
SOURCE_DULTRA_GATE = ROOT / "pack" / "lang_dultra_replay_artifact_implementation_gate_v1" / "dultra_replay_artifact_implementation_gate.detjson"
OWNER_PARSER_CHECKER = ROOT / "tests" / "run_lang_owner_inner_seum_parser_boundary_spike_check.py"
DULTRA_GATE_CHECKER = ROOT / "tests" / "run_lang_dultra_replay_artifact_implementation_gate_check.py"

WORK_ITEM = "LANG_OWNER_INNER_SEUM_RUNTIME_SCOPE_REBASE_V1"
NEXT = "LANG_LANGUAGE_RISK_REMOVAL_CLOSURE_REBASE_V1"
GATES = [
    ("parser_boundary", "closed"),
    ("owner_state_symbol_table", "planned"),
    ("prime_derivative_semantics", "blocked"),
    ("equation_solver_binding", "planned_after_derivative"),
    ("runtime_tick_order", "planned_after_solver"),
    ("event_reaction_isolation", "planned_after_solver"),
]
LANDED_SCOPE = [
    "seongjil_owner_state_alias",
    "owner_local_seum_parser_boundary",
    "owner_inner_seum_canon_rows",
    "receive_block_is_event_reaction",
]


def fail(message: str) -> None:
    print(f"lang_owner_inner_seum_runtime_scope_rebase_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_OWNER,
        SOURCE_DULTRA_GATE,
        OWNER_PARSER_CHECKER,
        DULTRA_GATE_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        "owner_state_symbol_table",
        "prime_derivative_semantics",
        "equation_solver_binding",
        "runtime_tick_order",
        "event_reaction_isolation",
        "No `docs/ssot/**` edit",
        "No owner-inner `세움` runtime landed claim",
        "No `세움` equation solver landed claim",
        "다음 언어 구현 위험 제거 계획: 6/6 = 100%",
        "Owner inner seum runtime scope rebase: 1/1 = 100%",
        "ROADMAP_V2 전체: queue-expanded 59/90 = 66%",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "Runtime Gates", "6/6 = 100%", "59/90 = 66%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "parser/frontdoor boundary evidence has landed",
            "No owner-inner `세움` runtime landed claim",
            "No owner-inner `세움` runtime landed claim",
            "No derivative semantics landed claim",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.owner_inner_seum_runtime_scope_rebase.v1",
            "lang_owner_inner_seum_runtime_scope_rebase_v1",
            "다음 언어 구현 위험 제거 계획: 6/6 = 100%",
            "Owner inner seum runtime scope rebase: 1/1 = 100%",
            "ROADMAP_V2 전체: queue-expanded 59/90 = 66%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_owner_inner_seum_runtime_scope_rebase_v1",
        "kind": "lang_owner_inner_seum_runtime_scope_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "owner_inner_seum_runtime_scope_rebase_claim": True,
        "owner_inner_seum_runtime_landed_claim": False,
        "seum_equation_solver_landed_claim": False,
        "derivative_semantics_landed_claim": False,
        "receive_block_relation_row_landed_claim": False,
        "seongjil_global_block_landed_claim": False,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_DULTRA_REPLAY_ARTIFACT_IMPLEMENTATION_GATE_V1",
        "proposal_doc": "docs/context/proposals/LANG_OWNER_INNER_SEUM_RUNTIME_SCOPE_REBASE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_OWNER_INNER_SEUM_RUNTIME_SCOPE_REBASE_20260606.md",
        "decision_manifest": "pack/lang_owner_inner_seum_runtime_scope_rebase_v1/owner_inner_seum_runtime_scope_rebase.detjson",
        "source_owner_inner_seum_parser_boundary": "pack/lang_owner_inner_seum_parser_boundary_spike_v1/owner_inner_seum_parser_boundary_spike.detjson",
        "source_dultra_replay_artifact_gate": "pack/lang_dultra_replay_artifact_implementation_gate_v1/dultra_replay_artifact_implementation_gate.detjson",
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
        "owner_inner_seum_runtime_scope_rebase_closed": 1,
        "owner_inner_seum_runtime_scope_rebase_total": 1,
        "owner_inner_seum_runtime_scope_rebase_percent": 100,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 59,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 66,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.owner_inner_seum_runtime_scope_rebase.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")

    expected_flags = {
        "owner_inner_seum_runtime_scope_rebase_claim": True,
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "owner_inner_seum_runtime_landed_claim": False,
        "seum_equation_solver_landed_claim": False,
        "derivative_semantics_landed_claim": False,
        "receive_block_relation_row_landed_claim": False,
        "seongjil_global_block_landed_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")

    scope_ids = [row.get("id") for row in manifest.get("current_landed_scope", [])]
    if scope_ids != LANDED_SCOPE:
        fail(f"landed scope mismatch: {scope_ids!r}")
    for row in manifest.get("current_landed_scope", []):
        if row.get("runtime_landed") is not False:
            fail(f"current landed scope must keep runtime_landed false: {row!r}")

    gates = [(row.get("id"), row.get("status")) for row in manifest.get("runtime_gates", [])]
    if gates != GATES:
        fail(f"runtime gate mismatch: {gates!r}")
    gate_orders = [row.get("order") for row in manifest.get("runtime_gates", [])]
    if gate_orders != [1, 2, 3, 4, 5, 6]:
        fail(f"runtime gate order mismatch: {gate_orders!r}")

    required_future = {
        "owner fields are resolved deterministically by owner scope before global lookup",
        "derivative variables are not treated as ordinary unrelated identifiers at runtime",
        "solver input rows are emitted in stable order",
        "event handlers do not mutate relation rows during relation collection",
        "runtime output records solved, skipped, or rejected relation status",
    }
    if set(manifest.get("future_runtime_evidence_requirements", [])) != required_future:
        fail("future runtime evidence requirements mismatch")

    for row in manifest.get("product_anchor_rows", []):
        path = ROOT / row.get("path", "")
        require(path)
        require_contains(path, row.get("tokens", []))
        if row.get("changed_now") is not False:
            fail(f"product anchor rows must not be changed_now: {row!r}")

    required_blocked = {
        "docs_ssot_edit",
        "parser_frontdoor_change",
        "runtime_surface_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "product_ui_change",
        "owner_inner_seum_runtime_landed",
        "seum_equation_solver_landed",
        "derivative_semantics_landed",
        "receive_block_relation_row_landed",
        "seongjil_global_block_landed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")

    expected_plans = {
        "language_risk_removal_plan": {"closed": 6, "total": 6, "percent": 100},
        "owner_inner_seum_runtime_scope_rebase": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 59, "total": 90, "percent": 66},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    owner = load_json(SOURCE_OWNER)
    expected_owner = {
        "owner_inner_seum_parser_boundary_landed_claim": True,
        "owner_inner_seum_runtime_landed_claim": False,
        "seum_equation_solver_landed_claim": False,
        "derivative_semantics_landed_claim": False,
    }
    for key, value in expected_owner.items():
        if owner.get(key) != value:
            fail(f"owner parser source {key} expected {value!r}, got {owner.get(key)!r}")

    gate = load_json(SOURCE_DULTRA_GATE)
    if gate.get("next_item") != WORK_ITEM:
        fail(f"D-ULTRA gate source next item expected {WORK_ITEM}, got {gate.get('next_item')!r}")
    if gate.get("language_risk_removal_plan") != {"closed": 5, "total": 6, "percent": 83}:
        fail(f"D-ULTRA gate source risk progress mismatch: {gate.get('language_risk_removal_plan')!r}")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_owner_inner_seum_runtime_scope_rebase_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    expected = [
        "LANG_OWNER_INNER_SEUM_RUNTIME_SCOPE_REBASE_V1",
        "owner inner seum runtime scope rebase sealed",
        "schema: ddn.language.owner_inner_seum_runtime_scope_rebase.v1",
        "required runtime gates: 6",
        "risk removal: 6/6 = 100%",
        "runtime landed: false",
        "next: LANG_LANGUAGE_RISK_REMOVAL_CLOSURE_REBASE_V1",
    ]
    require_contains(PACK / "golden.jsonl", expected)


def check_previous_checkers() -> None:
    for checker in [DULTRA_GATE_CHECKER, OWNER_PARSER_CHECKER]:
        proc = run([sys.executable, str(checker.relative_to(ROOT))], timeout=420)
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
    print("lang_owner_inner_seum_runtime_scope_rebase_check: PASS")


if __name__ == "__main__":
    main()
