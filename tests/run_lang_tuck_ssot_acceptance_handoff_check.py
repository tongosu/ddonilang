from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_TUCK_SSOT_ACCEPTANCE_HANDOFF_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_TUCK_SSOT_ACCEPTANCE_HANDOFF_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_TUCK_ACCEPTANCE_HANDOFF_20260606.md"
PACK = ROOT / "pack" / "lang_tuck_ssot_acceptance_handoff_v1"
MANIFEST = PACK / "tuck_ssot_acceptance_handoff.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_tuck_ssot_acceptance_handoff_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_TUCK_SHAPE = ROOT / "pack" / "lang_tuck_constraint_surface_shape_proposal_v1" / "tuck_constraint_surface_shape_proposal.detjson"
SOURCE_TUCK_NAME = ROOT / "pack" / "lang_sim_constraint_third_layer_name_v1" / "sim_constraint_third_layer_name.detjson"
SOURCE_VELOCITY = ROOT / "pack" / "lang_velocity_verlet_stdlib_surface_acceptance_v1" / "velocity_verlet_stdlib_surface_acceptance.detjson"
PREVIOUS_CHECKER = ROOT / "tests" / "run_lang_velocity_verlet_stdlib_surface_acceptance_check.py"
TUCK_SHAPE_CHECKER = ROOT / "tests" / "run_lang_tuck_constraint_surface_shape_proposal_check.py"

WORK_ITEM = "LANG_TUCK_SSOT_ACCEPTANCE_HANDOFF_V1"
NEXT = "LANG_PRODUCT_PATH_TRANSITION_CLOSURE_REBASE_V1"
REQUIRED_FIELDS = ["id", "when", "effect", "priority", "determinism"]
FALSE_FLAGS = [
    "runtime_claim",
    "product_code_change",
    "product_ui_change",
    "lesson_schema_change",
    "active_allowlist_mutation",
    "parser_frontdoor_change",
    "stdlib_surface_change",
    "ssot_edit_claim",
    "ssot_landed_claim",
    "tuck_block_landed_claim",
    "tuck_row_parser_landed_claim",
    "constraint_runtime_landed_claim",
    "solver_internal_inequality_claim",
]


def fail(message: str) -> None:
    print(f"lang_tuck_ssot_acceptance_handoff_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_TUCK_SHAPE,
        SOURCE_TUCK_NAME,
        SOURCE_VELOCITY,
        PREVIOUS_CHECKER,
        TUCK_SHAPE_CHECKER,
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
        "No `docs/ssot/**` edit",
        "No SSOT landed claim",
        "No parser/frontdoor change",
        "언어 제품 경로 구현 전환 계획: `7/7 = 100%`",
        "Tuck SSOT acceptance handoff: `1/1 = 100%`",
        "ROADMAP_V2 전체: `queue-expanded 66/90 = 73%`",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "SSOT-owner handoff", "7/7 = 100%", "66/90 = 73%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "Accept `턱`",
            "Minimum proposed row fields",
            "No SSOT landed claim by Codex",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.tuck_ssot_acceptance_handoff.v1",
            "lang_tuck_ssot_acceptance_handoff_v1",
            "언어 제품 경로 구현 전환 계획: 7/7 = 100%",
            "Tuck SSOT acceptance handoff: 1/1 = 100%",
            "ROADMAP_V2 전체: queue-expanded 66/90 = 73%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_tuck_ssot_acceptance_handoff_v1",
        "kind": "lang_tuck_ssot_acceptance_handoff",
        "tuck_ssot_acceptance_handoff_claim": True,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_VELOCITY_VERLET_STDLIB_SURFACE_ACCEPTANCE_V1",
        "proposal_doc": "docs/context/proposals/LANG_TUCK_SSOT_ACCEPTANCE_HANDOFF_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_TUCK_ACCEPTANCE_HANDOFF_20260606.md",
        "decision_manifest": "pack/lang_tuck_ssot_acceptance_handoff_v1/tuck_ssot_acceptance_handoff.detjson",
        "source_tuck_constraint_surface_shape_proposal": "pack/lang_tuck_constraint_surface_shape_proposal_v1/tuck_constraint_surface_shape_proposal.detjson",
        "source_tuck_name": "pack/lang_sim_constraint_third_layer_name_v1/sim_constraint_third_layer_name.detjson",
        "source_velocity_verlet_stdlib_surface_acceptance": "pack/lang_velocity_verlet_stdlib_surface_acceptance_v1/velocity_verlet_stdlib_surface_acceptance.detjson",
        "selected_layer_name": "턱",
        "proposed_surface_family": "턱-row",
        "language_product_path_transition_closed": 7,
        "language_product_path_transition_total": 7,
        "language_product_path_transition_percent": 100,
        "tuck_ssot_acceptance_handoff_closed": 1,
        "tuck_ssot_acceptance_handoff_total": 1,
        "tuck_ssot_acceptance_handoff_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 66,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 73,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    for key in FALSE_FLAGS:
        if contract.get(key) is not False:
            fail(f"contract {key} must be false, got {contract.get(key)!r}")
    for source_key in [
        "source_tuck_constraint_surface_shape_proposal",
        "source_tuck_name",
        "source_velocity_verlet_stdlib_surface_acceptance",
    ]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.tuck_ssot_acceptance_handoff.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    if manifest.get("tuck_ssot_acceptance_handoff_claim") is not True:
        fail("handoff claim must be true")
    for key in FALSE_FLAGS:
        if manifest.get(key) is not False:
            fail(f"manifest {key} must be false, got {manifest.get(key)!r}")

    packet = manifest.get("handoff_packet", {})
    expected_packet = {
        "selected_layer_name": "턱",
        "proposed_surface_family": "턱-row",
        "representation": "named_boundary_threshold_record_family",
        "required_fields": REQUIRED_FIELDS,
        "candidate_effects": ["clamp", "terminate", "exit", "transition"],
        "deterministic_priority": "domain_order_then_stable_id",
    }
    if packet != expected_packet:
        fail(f"handoff packet mismatch: {packet!r}")

    text_rows = manifest.get("recommended_ssot_acceptance_text", [])
    for token in ["`턱`", "row-family", "Parser and runtime behavior require separate product-path evidence"]:
        if not any(token in row for row in text_rows):
            fail(f"recommended acceptance text missing {token!r}")

    gates = manifest.get("post_acceptance_gates", [])
    expected_gates = [
        ("parser_frontdoor_spike", "planned_after_ssot_acceptance"),
        ("deterministic_runtime_order", "planned_after_parser"),
        ("replay_evidence", "planned_after_runtime_order"),
    ]
    if len(gates) != len(expected_gates):
        fail(f"post acceptance gate count mismatch: {len(gates)}")
    for index, ((gate_id, status), row) in enumerate(zip(expected_gates, gates), start=1):
        if row.get("order") != index or row.get("id") != gate_id or row.get("status") != status:
            fail(f"post acceptance gate mismatch: {row!r}")
        if not row.get("required_evidence"):
            fail(f"post acceptance gate missing evidence: {row!r}")

    for row in manifest.get("product_anchor_rows", []):
        path = ROOT / row.get("path", "")
        require(path)
        require_contains(path, row.get("tokens", []))
        if row.get("changed_now") is not False:
            fail(f"anchor changed_now must be false: {row!r}")

    required_blocked = {
        "docs_ssot_edit",
        "ssot_landed",
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
        "language_product_path_transition_plan": {"closed": 7, "total": 7, "percent": 100},
        "tuck_ssot_acceptance_handoff": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 66, "total": 90, "percent": 73},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    shape = load_json(SOURCE_TUCK_SHAPE)
    if shape.get("surface_shape", {}).get("layer_name") != "턱":
        fail(f"shape source layer mismatch: {shape.get('surface_shape')!r}")
    if shape.get("surface_shape", {}).get("surface_family") != "턱-row":
        fail(f"shape source family mismatch: {shape.get('surface_shape')!r}")
    if [row.get("field") for row in shape.get("required_fields", [])] != REQUIRED_FIELDS:
        fail(f"shape source required fields mismatch: {shape.get('required_fields')!r}")
    for key in [
        "ssot_edit_claim",
        "tuck_block_landed_claim",
        "tuck_row_parser_landed_claim",
        "constraint_runtime_landed_claim",
        "solver_internal_inequality_claim",
    ]:
        if shape.get(key) is not False:
            fail(f"shape source {key} must remain false")

    name = load_json(SOURCE_TUCK_NAME)
    decision = name.get("naming_decision", {})
    if decision.get("selected_constraint_layer_name") != "턱":
        fail(f"name source mismatch: {decision!r}")
    for key in ["ssot_landed", "parser_landed", "runtime_landed", "stdlib_landed"]:
        if decision.get(key) is not False:
            fail(f"name source {key} must remain false")

    velocity = load_json(SOURCE_VELOCITY)
    if velocity.get("next_item") != WORK_ITEM:
        fail(f"velocity source next expected {WORK_ITEM}, got {velocity.get('next_item')!r}")
    if velocity.get("language_product_path_transition_plan") != {"closed": 6, "total": 7, "percent": 86}:
        fail(f"velocity source transition progress mismatch: {velocity.get('language_product_path_transition_plan')!r}")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_tuck_ssot_acceptance_handoff_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    require_contains(
        PACK / "golden.jsonl",
        [
            WORK_ITEM,
            "schema: ddn.language.tuck_ssot_acceptance_handoff.v1",
            "selected layer: 턱",
            "surface family: 턱-row",
            "product transition: 7/7 = 100%; roadmap: 66/90 = 73%",
            "ssot landed: false",
            NEXT,
        ],
    )


def check_previous_checkers() -> None:
    for checker in [TUCK_SHAPE_CHECKER, PREVIOUS_CHECKER]:
        proc = run([sys.executable, str(checker.relative_to(ROOT))], timeout=1200)
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
    print("lang_tuck_ssot_acceptance_handoff_check: PASS")


if __name__ == "__main__":
    main()

