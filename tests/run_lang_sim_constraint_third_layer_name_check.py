from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_SIM_CONSTRAINT_THIRD_LAYER_NAME_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_SIM_CONSTRAINT_THIRD_LAYER_NAME_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_SIM_CONSTRAINT_THIRD_LAYER_NAME_20260606.md"
PACK = ROOT / "pack" / "lang_sim_constraint_third_layer_name_v1"
MANIFEST = PACK / "sim_constraint_third_layer_name.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_sim_constraint_third_layer_name_check.py"
SOURCE_REBASE = ROOT / "pack" / "language_design_priority_rebase_v1" / "language_design_priority_rebase.detjson"
SOURCE_FLOW = ROOT / "pack" / "lang_flow_type_collision_rename_v1" / "flow_type_collision_rename.detjson"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
NEXT = "LANG_SEUM_VOL3_PRIME_EXAMPLE_PACK_V1"


def fail(message: str) -> None:
    print(f"lang_sim_constraint_third_layer_name_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_REBASE,
        SOURCE_FLOW,
        ROOT / "tests" / "run_lang_flow_type_collision_rename_check.py",
        DEV_SUMMARY,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "LANG_SIM_CONSTRAINT_THIRD_LAYER_NAME_V1",
        "턱",
        "닿음",
        "막음",
        "문턱",
        "천장",
        "파산",
        "docs/ssot/**",
        "새 언어 설계 안정화 계획: 4/8 = 50%",
        "긴급 언어 결정 evidence closure: 3/3 = 100%",
        "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, tokens[:7])
    require_contains(
        SSOT_NOTE,
        [
            "Adopt `턱`",
            "ceiling limit",
            "bankruptcy exit",
            "No parser/runtime landed claim",
            "Codex did not edit `docs/ssot/**`",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "LANG_SIM_CONSTRAINT_THIRD_LAYER_NAME_V1",
            "lang_sim_constraint_third_layer_name_v1",
            "ddn.language.sim_constraint_third_layer_name.v1",
            "새 언어 설계 안정화 계획: 4/8 = 50%",
            "긴급 언어 결정 evidence closure: 3/3 = 100%",
            "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_sim_constraint_third_layer_name_v1",
        "kind": "lang_sim_constraint_third_layer_name",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "sim_constraint_third_layer_name_decision_claim": True,
        "constraint_layer_runtime_landed_claim": False,
        "constraint_layer_parser_landed_claim": False,
        "constraint_block_landed_claim": False,
        "selected_constraint_layer_name": "턱",
        "closed_by": "LANG_SIM_CONSTRAINT_THIRD_LAYER_NAME_V1",
        "based_on": "LANG_FLOW_TYPE_COLLISION_RENAME_V1",
        "proposal_doc": "docs/context/proposals/LANG_SIM_CONSTRAINT_THIRD_LAYER_NAME_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_SIM_CONSTRAINT_THIRD_LAYER_NAME_20260606.md",
        "decision_manifest": "pack/lang_sim_constraint_third_layer_name_v1/sim_constraint_third_layer_name.detjson",
        "source_priority_rebase": "pack/language_design_priority_rebase_v1/language_design_priority_rebase.detjson",
        "source_flow_decision": "pack/lang_flow_type_collision_rename_v1/flow_type_collision_rename.detjson",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "language_design_queue_closed": 4,
        "language_design_queue_total": 8,
        "language_design_queue_percent": 50,
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


def expected_evidence_rows() -> list[dict[str, object]]:
    return [
        {
            "id": "priority_rebase_recommendation",
            "path": "pack/language_design_priority_rebase_v1/language_design_priority_rebase.detjson",
            "tokens": ["sim_constraint_third_layer", "턱", "ceiling_limit", "bankruptcy_exit"],
            "classification": "source_recommendation",
            "decision_only": True,
            "parser_landed": False,
            "runtime_landed": False,
            "ssot_landed": False,
        },
        {
            "id": "cartpole_limit_termination",
            "path": "core/src/nurigym/cartpole.rs",
            "tokens": ["position_limit", "angle_limit"],
            "classification": "runtime_limit_example_anchor",
            "decision_only": True,
            "parser_landed": False,
            "runtime_landed": False,
            "ssot_landed": False,
        },
        {
            "id": "pendulum_angle_limit",
            "path": "core/src/nurigym/pendulum.rs",
            "tokens": ["angle_limit"],
            "classification": "runtime_limit_example_anchor",
            "decision_only": True,
            "parser_landed": False,
            "runtime_landed": False,
            "ssot_landed": False,
        },
        {
            "id": "ddn_runtime_inequality_constraint",
            "path": "tool/src/ddn_runtime.rs",
            "tokens": ["apply_linear_inequality_constraint", "제약개수"],
            "classification": "existing_math_constraint_anchor",
            "decision_only": True,
            "parser_landed": False,
            "runtime_landed": False,
            "ssot_landed": False,
        },
        {
            "id": "econ_bankruptcy_anchor",
            "path": "docs/context/proposals/PROPOSAL_ECON_AGENT_STANDARD_V0_20260328.md",
            "tokens": ["파산", "파산상태"],
            "classification": "economic_exit_threshold_anchor",
            "decision_only": True,
            "parser_landed": False,
            "runtime_landed": False,
            "ssot_landed": False,
        },
        {
            "id": "execution_axis_term_collision",
            "path": "docs/context/proposals/proposal_execution_axes_v_3_20260404.md",
            "tokens": ["막음모음", "닿음끝"],
            "classification": "rejected_alternative_collision_anchor",
            "decision_only": True,
            "parser_landed": False,
            "runtime_landed": False,
            "ssot_landed": False,
        },
    ]


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.sim_constraint_third_layer_name.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "LANG_SIM_CONSTRAINT_THIRD_LAYER_NAME_V1":
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
        "constraint_layer_runtime_landed_claim",
        "constraint_layer_parser_landed_claim",
        "constraint_block_landed_claim",
    ]:
        if manifest.get(flag) is not False:
            fail(f"manifest {flag} expected false, got {manifest.get(flag)!r}")
    expected_decision = {
        "selected_constraint_layer_name": "턱",
        "semantic_role": "simulation_boundary_threshold_or_exit_constraint",
        "decision_landed": True,
        "ssot_landed": False,
        "parser_landed": False,
        "runtime_landed": False,
        "stdlib_landed": False,
    }
    if manifest.get("naming_decision") != expected_decision:
        fail(f"naming decision mismatch: {manifest.get('naming_decision')!r}")
    expected_layers = [
        {"layer": "세움", "role": "equation_or_declarative_relation"},
        {"layer": "받으면_or_상태머신", "role": "ordinary_event_or_state_transition"},
        {"layer": "턱", "role": "boundary_threshold_or_exit_intervention"},
    ]
    if manifest.get("layer_boundary") != expected_layers:
        fail(f"layer boundary mismatch: {manifest.get('layer_boundary')!r}")
    example_ids = [row.get("id") for row in manifest.get("example_roles", [])]
    if example_ids != [
        "ceiling_limit",
        "out_of_range_termination",
        "bankruptcy_exit",
        "policy_threshold_transition",
    ]:
        fail(f"example role ids mismatch: {example_ids!r}")
    if manifest.get("evidence_rows") != expected_evidence_rows():
        fail(f"evidence rows mismatch: {manifest.get('evidence_rows')!r}")
    rejected = [row.get("surface") for row in manifest.get("rejected_alternatives", [])]
    if rejected != ["닿음", "막음", "문턱"]:
        fail(f"rejected alternatives mismatch: {rejected!r}")
    expected_policy = {
        "add_new_block_now": False,
        "parser_change_now": False,
        "runtime_change_now": False,
        "ssot_update_required_later": True,
        "later_decision_needed": "block_vs_clause_vs_existing_surface_row_family",
    }
    if manifest.get("implementation_policy") != expected_policy:
        fail(f"implementation policy mismatch: {manifest.get('implementation_policy')!r}")
    if manifest.get("queue_plan") != {"closed": 4, "total": 8, "percent": 50}:
        fail(f"queue plan mismatch: {manifest.get('queue_plan')!r}")
    if manifest.get("urgent_evidence_plan") != {"closed": 3, "total": 3, "percent": 100}:
        fail(f"urgent evidence plan mismatch: {manifest.get('urgent_evidence_plan')!r}")
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
        "constraint_layer_runtime_landed",
        "constraint_layer_parser_landed",
        "constraint_block_landed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")


def check_evidence_anchors() -> None:
    manifest = load_json(MANIFEST)
    for row in manifest.get("evidence_rows", []):
        path = ROOT / row["path"]
        require(path)
        require_contains(path, list(row["tokens"]))


def check_source_alignment() -> None:
    rebase = load_json(SOURCE_REBASE)
    sim = None
    for row in rebase.get("urgent_recommendations", []):
        if row.get("id") == "sim_constraint_third_layer":
            sim = row
            break
    if not sim:
        fail("source rebase missing sim_constraint_third_layer recommendation")
    if sim.get("preferred") != "턱":
        fail(f"source sim preferred mismatch: {sim!r}")
    if sim.get("examples") != ["ceiling_limit", "bankruptcy_exit"]:
        fail(f"source sim examples mismatch: {sim!r}")
    if sim.get("next_item") != "LANG_SIM_CONSTRAINT_THIRD_LAYER_NAME_V1":
        fail(f"source sim next item mismatch: {sim!r}")
    if sim.get("ssot_landed") is not False or sim.get("runtime_landed") is not False:
        fail(f"source sim recommendation must not be landed: {sim!r}")

    flow = load_json(SOURCE_FLOW)
    if flow.get("schema") != "ddn.language.flow_type_collision_rename.v1":
        fail(f"source flow schema mismatch: {flow.get('schema')!r}")
    if flow.get("next_item") != "LANG_SIM_CONSTRAINT_THIRD_LAYER_NAME_V1":
        fail(f"source flow next item mismatch: {flow.get('next_item')!r}")
    if flow.get("queue_plan") != {"closed": 3, "total": 8, "percent": 38}:
        fail(f"source flow queue mismatch: {flow.get('queue_plan')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "LANG_SIM_CONSTRAINT_THIRD_LAYER_NAME_V1",
        "sim constraint third layer name sealed",
        "constraint layer schema: ddn.language.sim_constraint_third_layer_name.v1",
        "selected constraint layer name: 턱",
        "language queue: 4/8 = 50%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/lang_sim_constraint_third_layer_name_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "lang_sim_constraint_third_layer_name_v1"],
        ["python", "tests/run_lang_flow_type_collision_rename_check.py"],
    ]:
        proc = run(cmd, timeout=300)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_evidence_anchors()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("lang_sim_constraint_third_layer_name_check: ok")


if __name__ == "__main__":
    main()
