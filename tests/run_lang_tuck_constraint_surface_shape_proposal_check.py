from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_TUCK_CONSTRAINT_SURFACE_SHAPE_PROPOSAL_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_TUCK_CONSTRAINT_SURFACE_SHAPE_PROPOSAL_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_TUCK_CONSTRAINT_SURFACE_SHAPE_PROPOSAL_20260606.md"
PACK = ROOT / "pack" / "lang_tuck_constraint_surface_shape_proposal_v1"
MANIFEST = PACK / "tuck_constraint_surface_shape_proposal.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_tuck_constraint_surface_shape_proposal_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_TUCK_NAME = ROOT / "pack" / "lang_sim_constraint_third_layer_name_v1" / "sim_constraint_third_layer_name.detjson"
SOURCE_FLOW_MIGRATION = ROOT / "pack" / "lang_flow_history_alias_migration_plan_v1" / "flow_history_alias_migration_plan.detjson"
PREVIOUS_CHECKER = ROOT / "tests" / "run_lang_flow_history_alias_migration_plan_check.py"

WORK_ITEM = "LANG_TUCK_CONSTRAINT_SURFACE_SHAPE_PROPOSAL_V1"
NEXT = "LANG_VELOCITY_VERLET_RUNTIME_GATE_REBASE_V1"
REQUIRED_FIELDS = ["id", "when", "effect", "priority", "determinism"]


def fail(message: str) -> None:
    print(f"lang_tuck_constraint_surface_shape_proposal_check: FAIL: {message}", file=sys.stderr)
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


def run(cmd: list[str], *, timeout: int = 240) -> subprocess.CompletedProcess[str]:
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
        SOURCE_TUCK_NAME,
        SOURCE_FLOW_MIGRATION,
        PREVIOUS_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        "턱-row",
        "id",
        "when",
        "effect",
        "priority",
        "determinism",
        "ceiling_limit",
        "bankruptcy_exit",
        "No `docs/ssot/**` edit",
        "No `턱 {}` block landed claim",
        "No `턱-row` parser landed claim",
        "다음 언어 구현 위험 제거 계획: 3/6 = 50%",
        "Tuck constraint surface shape proposal: 1/1 = 100%",
        "ROADMAP_V2 전체: queue-expanded 56/90 = 62%",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "턱-row", "id", "when", "effect", "priority", "determinism", "3/6 = 50%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "Required row fields: `id`, `when`, `effect`, `priority`, `determinism`",
            "No parser/frontdoor change",
            "No runtime or stdlib surface change",
            "No `턱 {}` block landed claim",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.tuck_constraint_surface_shape_proposal.v1",
            "lang_tuck_constraint_surface_shape_proposal_v1",
            "다음 언어 구현 위험 제거 계획: 3/6 = 50%",
            "Tuck constraint surface shape proposal: 1/1 = 100%",
            "ROADMAP_V2 전체: queue-expanded 56/90 = 62%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_tuck_constraint_surface_shape_proposal_v1",
        "kind": "lang_tuck_constraint_surface_shape_proposal",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "tuck_surface_shape_proposal_claim": True,
        "tuck_block_landed_claim": False,
        "tuck_row_parser_landed_claim": False,
        "constraint_runtime_landed_claim": False,
        "solver_internal_inequality_claim": False,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_FLOW_HISTORY_ALIAS_MIGRATION_PLAN_V1",
        "proposal_doc": "docs/context/proposals/LANG_TUCK_CONSTRAINT_SURFACE_SHAPE_PROPOSAL_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_TUCK_CONSTRAINT_SURFACE_SHAPE_PROPOSAL_20260606.md",
        "decision_manifest": "pack/lang_tuck_constraint_surface_shape_proposal_v1/tuck_constraint_surface_shape_proposal.detjson",
        "source_tuck_name": "pack/lang_sim_constraint_third_layer_name_v1/sim_constraint_third_layer_name.detjson",
        "source_flow_history_alias_migration": "pack/lang_flow_history_alias_migration_plan_v1/flow_history_alias_migration_plan.detjson",
        "selected_layer_name": "턱",
        "proposed_surface_family": "턱-row",
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
        "language_risk_removal_closed": 3,
        "language_risk_removal_total": 6,
        "language_risk_removal_percent": 50,
        "tuck_constraint_surface_shape_proposal_closed": 1,
        "tuck_constraint_surface_shape_proposal_total": 1,
        "tuck_constraint_surface_shape_proposal_percent": 100,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 56,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 62,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.tuck_constraint_surface_shape_proposal.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")

    expected_flags = {
        "tuck_surface_shape_proposal_claim": True,
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "tuck_block_landed_claim": False,
        "tuck_row_parser_landed_claim": False,
        "constraint_runtime_landed_claim": False,
        "solver_internal_inequality_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")

    shape = manifest.get("surface_shape", {})
    expected_shape = {
        "layer_name": "턱",
        "surface_family": "턱-row",
        "representation": "named_boundary_threshold_record_family",
        "block_landed": False,
        "parser_landed": False,
        "runtime_landed": False,
        "ssot_landed": False,
    }
    if shape != expected_shape:
        fail(f"surface shape mismatch: {shape!r}")

    fields = manifest.get("required_fields", [])
    if [row.get("field") for row in fields] != REQUIRED_FIELDS:
        fail(f"required fields mismatch: {fields!r}")
    for row in fields:
        if not row.get("role"):
            fail(f"required field missing role: {row!r}")

    examples = manifest.get("candidate_examples", [])
    example_ids = [row.get("id") for row in examples]
    if example_ids != ["ceiling_limit", "cartpole_out_of_range", "bankruptcy_exit", "policy_threshold_transition"]:
        fail(f"candidate example ids mismatch: {example_ids!r}")
    for row in examples:
        if row.get("runtime_landed") is not False:
            fail(f"candidate example must not be runtime landed: {row!r}")
        for key in ["when", "effect", "priority", "determinism"]:
            if not row.get(key):
                fail(f"candidate example missing {key}: {row!r}")

    gates = manifest.get("implementation_gates", [])
    expected_gates = [
        ("surface_shape_proposal", "closed_now"),
        ("ssot_acceptance", "blocked"),
        ("parser_frontdoor_spike", "planned_after_ssot"),
        ("deterministic_runtime_order", "planned_after_parser"),
    ]
    if len(gates) != len(expected_gates):
        fail(f"implementation gate count mismatch: {len(gates)}")
    for index, ((gate_id, status), row) in enumerate(zip(expected_gates, gates), start=1):
        if row.get("order") != index or row.get("id") != gate_id or row.get("status") != status:
            fail(f"implementation gate mismatch: {row!r}")
        if not row.get("required_evidence"):
            fail(f"implementation gate missing evidence: {row!r}")

    for row in manifest.get("product_anchor_rows", []):
        path = ROOT / row.get("path", "")
        require(path)
        require_contains(path, row.get("tokens", []))
        if row.get("changed_now") is not False:
            fail(f"product anchor changed_now must be false: {row!r}")

    required_blocked = {
        "docs_ssot_edit",
        "parser_frontdoor_change",
        "runtime_surface_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "product_ui_change",
        "tuck_block_landed",
        "tuck_row_parser_landed",
        "constraint_runtime_landed",
        "solver_internal_inequality",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")

    expected_plans = {
        "language_risk_removal_plan": {"closed": 3, "total": 6, "percent": 50},
        "tuck_constraint_surface_shape_proposal": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 56, "total": 90, "percent": 62},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"{key} mismatch: {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    tuck = load_json(SOURCE_TUCK_NAME)
    decision = tuck.get("naming_decision", {})
    if decision.get("selected_constraint_layer_name") != "턱":
        fail(f"source tuck name mismatch: {decision!r}")
    if decision.get("parser_landed") is not False or decision.get("runtime_landed") is not False:
        fail(f"source tuck name must not be parser/runtime landed: {decision!r}")
    if tuck.get("implementation_policy", {}).get("later_decision_needed") != "block_vs_clause_vs_existing_surface_row_family":
        fail(f"source tuck implementation policy mismatch: {tuck.get('implementation_policy')!r}")

    flow = load_json(SOURCE_FLOW_MIGRATION)
    if flow.get("next_item") != WORK_ITEM:
        fail(f"source flow migration next mismatch: {flow.get('next_item')!r}")
    if flow.get("language_risk_removal_plan") != {"closed": 2, "total": 6, "percent": 33}:
        fail(f"source flow risk progress mismatch: {flow.get('language_risk_removal_plan')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected_stdout = [
        WORK_ITEM,
        "tuck constraint surface shape proposal sealed",
        "schema: ddn.language.tuck_constraint_surface_shape_proposal.v1",
        "surface family: 턱-row",
        "required fields: id, when, effect, priority, determinism",
        "risk removal: 3/6 = 50%",
        "runtime landed: false",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/lang_tuck_constraint_surface_shape_proposal_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected_stdout:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_tuck_constraint_surface_shape_proposal_v1"], timeout=240)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_previous_checker() -> None:
    proc = run([sys.executable, str(PREVIOUS_CHECKER.relative_to(ROOT))], timeout=900)
    if proc.returncode != 0:
        fail(f"previous checker failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_golden()
    check_pack_golden()
    check_previous_checker()
    require_docs_ssot_clean()
    print("lang_tuck_constraint_surface_shape_proposal_check: PASS")


if __name__ == "__main__":
    main()
