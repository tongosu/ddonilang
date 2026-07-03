from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_TUCK_SSOT_ACCEPTANCE_REQUEST_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_TUCK_SSOT_ACCEPTANCE_REQUEST_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_TUCK_ACCEPTANCE_REQUEST_20260606.md"
PACK = ROOT / "pack" / "lang_tuck_ssot_acceptance_request_v1"
MANIFEST = PACK / "tuck_ssot_acceptance_request.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_tuck_ssot_acceptance_request_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_PREVIOUS = ROOT / "pack" / "lang_flow_history_ssot_acceptance_request_v1" / "flow_history_ssot_acceptance_request.detjson"
SOURCE_COORDINATION = ROOT / "pack" / "lang_ssot_landing_coordination_rebase_v1" / "ssot_landing_coordination_rebase.detjson"
SOURCE_TUCK_NAME = ROOT / "pack" / "lang_sim_constraint_third_layer_name_v1" / "sim_constraint_third_layer_name.detjson"
SOURCE_TUCK_SHAPE = ROOT / "pack" / "lang_tuck_constraint_surface_shape_proposal_v1" / "tuck_constraint_surface_shape_proposal.detjson"
SOURCE_HANDOFF = ROOT / "pack" / "lang_tuck_ssot_acceptance_handoff_v1" / "tuck_ssot_acceptance_handoff.detjson"
PREVIOUS_CHECKER = ROOT / "tests" / "run_lang_flow_history_ssot_acceptance_request_check.py"

WORK_ITEM = "LANG_TUCK_SSOT_ACCEPTANCE_REQUEST_V1"
NEXT = "LANG_SSOT_ACCEPTANCE_REQUEST_CLOSURE_REBASE_V1"
REQUIRED_FIELDS = ["id", "when", "effect", "priority", "determinism"]
CANDIDATE_EFFECTS = ["clamp", "terminate", "exit", "transition"]
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
    "tuck_ssot_landed_claim",
    "tuck_block_landed_claim",
    "tuck_row_parser_landed_claim",
    "constraint_runtime_landed_claim",
    "solver_internal_inequality_claim",
]


def fail(message: str) -> None:
    print(f"lang_tuck_ssot_acceptance_request_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_PREVIOUS,
        SOURCE_COORDINATION,
        SOURCE_TUCK_NAME,
        SOURCE_TUCK_SHAPE,
        SOURCE_HANDOFF,
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
        "No `docs/ssot/**` edit",
        "No SSOT landed claim",
        "No `턱-row` parser landed claim",
        "SSOT acceptance request queue: `3/3 = 100%`",
        "Tuck SSOT acceptance request: `1/1 = 100%`",
        "긴급 언어 결정 SSOT 반영: `0/3 = 0%`",
        "ROADMAP_V2 전체: `queue-expanded 71/90 = 79%`",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "3/3 = 100%", "0/3 = 0%", "71/90 = 79%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "`턱`: third simulation-constraint layer",
            "`턱-row`: proposed initial row-family surface",
            "No `턱-row` parser landed claim",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.tuck_ssot_acceptance_request.v1",
            "lang_tuck_ssot_acceptance_request_v1",
            "SSOT acceptance request queue: 3/3 = 100%",
            "Tuck SSOT acceptance request: 1/1 = 100%",
            "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
            "ROADMAP_V2 전체: queue-expanded 71/90 = 79%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_tuck_ssot_acceptance_request_v1",
        "kind": "lang_tuck_ssot_acceptance_request",
        "tuck_ssot_acceptance_request_claim": True,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_FLOW_HISTORY_SSOT_ACCEPTANCE_REQUEST_V1",
        "proposal_doc": "docs/context/proposals/LANG_TUCK_SSOT_ACCEPTANCE_REQUEST_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_TUCK_ACCEPTANCE_REQUEST_20260606.md",
        "decision_manifest": "pack/lang_tuck_ssot_acceptance_request_v1/tuck_ssot_acceptance_request.detjson",
        "source_flow_history_ssot_acceptance_request": "pack/lang_flow_history_ssot_acceptance_request_v1/flow_history_ssot_acceptance_request.detjson",
        "source_ssot_landing_coordination": "pack/lang_ssot_landing_coordination_rebase_v1/ssot_landing_coordination_rebase.detjson",
        "source_tuck_name": "pack/lang_sim_constraint_third_layer_name_v1/sim_constraint_third_layer_name.detjson",
        "source_tuck_constraint_surface_shape_proposal": "pack/lang_tuck_constraint_surface_shape_proposal_v1/tuck_constraint_surface_shape_proposal.detjson",
        "source_tuck_ssot_acceptance_handoff": "pack/lang_tuck_ssot_acceptance_handoff_v1/tuck_ssot_acceptance_handoff.detjson",
        "ssot_acceptance_request_queue_closed": 3,
        "ssot_acceptance_request_queue_total": 3,
        "ssot_acceptance_request_queue_percent": 100,
        "tuck_ssot_acceptance_request_closed": 1,
        "tuck_ssot_acceptance_request_total": 1,
        "tuck_ssot_acceptance_request_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 71,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 79,
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
        "source_flow_history_ssot_acceptance_request",
        "source_ssot_landing_coordination",
        "source_tuck_name",
        "source_tuck_constraint_surface_shape_proposal",
        "source_tuck_ssot_acceptance_handoff",
    ]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.tuck_ssot_acceptance_request.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    if manifest.get("tuck_ssot_acceptance_request_claim") is not True:
        fail("request claim must be true")
    for key in FALSE_FLAGS:
        if manifest.get(key) is not False:
            fail(f"manifest {key} must be false, got {manifest.get(key)!r}")

    request = manifest.get("request", {})
    expected_request_keys = {
        "target_id": "tuck_constraint_layer",
        "selected_layer_name": "턱",
        "proposed_surface_family": "턱-row",
        "representation": "named_boundary_threshold_record_family",
    }
    for key, value in expected_request_keys.items():
        if request.get(key) != value:
            fail(f"request {key} expected {value!r}, got {request.get(key)!r}")
    if request.get("required_fields") != REQUIRED_FIELDS:
        fail(f"required fields mismatch: {request.get('required_fields')!r}")
    if request.get("candidate_effects") != CANDIDATE_EFFECTS:
        fail(f"candidate effects mismatch: {request.get('candidate_effects')!r}")
    if request.get("post_acceptance_gates") != ["parser_frontdoor_spike", "deterministic_runtime_order", "replay_evidence"]:
        fail(f"post acceptance gates mismatch: {request.get('post_acceptance_gates')!r}")
    for token in ["`턱` is the third", "`턱-row`", "separate product-path evidence"]:
        if not any(token in row for row in request.get("recommended_acceptance_text", [])):
            fail(f"recommended text missing {token!r}")

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
        "ssot_acceptance_request_queue": {"closed": 3, "total": 3, "percent": 100},
        "tuck_ssot_acceptance_request": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 71, "total": 90, "percent": 79},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    previous = load_json(SOURCE_PREVIOUS)
    if previous.get("next_item") != WORK_ITEM:
        fail(f"previous request next expected {WORK_ITEM}, got {previous.get('next_item')!r}")
    if previous.get("ssot_acceptance_request_queue") != {"closed": 2, "total": 3, "percent": 67}:
        fail(f"previous request queue mismatch: {previous.get('ssot_acceptance_request_queue')!r}")

    coordination = load_json(SOURCE_COORDINATION)
    target = coordination.get("coordination_targets", [])[2]
    if target.get("id") != "tuck_constraint_layer" or target.get("next_item") != WORK_ITEM:
        fail(f"coordination tuck target mismatch: {target!r}")
    if target.get("ssot_landed") is not False:
        fail("coordination tuck target ssot_landed must be false")

    name = load_json(SOURCE_TUCK_NAME)
    decision = name.get("naming_decision", {})
    if decision.get("selected_constraint_layer_name") != "턱":
        fail(f"tuck name decision mismatch: {decision!r}")
    for key in ["ssot_landed", "parser_landed", "runtime_landed", "stdlib_landed"]:
        if decision.get(key) is not False:
            fail(f"tuck name {key} must be false")

    shape = load_json(SOURCE_TUCK_SHAPE)
    surface = shape.get("surface_shape", {})
    if surface.get("surface_family") != "턱-row":
        fail(f"tuck surface family mismatch: {surface!r}")
    if [row.get("field") for row in shape.get("required_fields", [])] != REQUIRED_FIELDS:
        fail(f"shape required fields mismatch: {shape.get('required_fields')!r}")
    for key in ["tuck_block_landed_claim", "tuck_row_parser_landed_claim", "constraint_runtime_landed_claim"]:
        if shape.get(key) is not False:
            fail(f"shape {key} must be false")

    handoff = load_json(SOURCE_HANDOFF)
    packet = handoff.get("handoff_packet", {})
    if packet.get("selected_layer_name") != "턱" or packet.get("proposed_surface_family") != "턱-row":
        fail(f"handoff packet mismatch: {packet!r}")
    if handoff.get("ssot_landed_claim") is not False:
        fail("handoff ssot landed claim must remain false")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_tuck_ssot_acceptance_request_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    require_contains(
        PACK / "golden.jsonl",
        [
            WORK_ITEM,
            "schema: ddn.language.tuck_ssot_acceptance_request.v1",
            "surface: 턱 / 턱-row",
            "ssot request queue: 3/3 = 100%",
            "urgent ssot landed: 0/3 = 0%",
            "roadmap: 71/90 = 79%",
            NEXT,
        ],
    )


def check_previous_checker() -> None:
    proc = run([sys.executable, str(PREVIOUS_CHECKER.relative_to(ROOT))], timeout=1200)
    if proc.returncode != 0:
        fail(f"{PREVIOUS_CHECKER.relative_to(ROOT)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_golden()
    check_previous_checker()
    require_docs_ssot_clean()
    print("lang_tuck_ssot_acceptance_request_check: PASS")


if __name__ == "__main__":
    main()

