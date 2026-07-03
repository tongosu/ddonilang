from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_SSOT_LANDING_COORDINATION_REBASE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_SSOT_LANDING_COORDINATION_REBASE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_LANDING_COORDINATION_REBASE_20260606.md"
PACK = ROOT / "pack" / "lang_ssot_landing_coordination_rebase_v1"
MANIFEST = PACK / "ssot_landing_coordination_rebase.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_ssot_landing_coordination_rebase_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_CLOSURE = ROOT / "pack" / "lang_product_path_transition_closure_rebase_v1" / "product_path_transition_closure_rebase.detjson"
SOURCE_PRIME = ROOT / "pack" / "lang_prime_derivative_notation_decision_v1" / "prime_derivative_notation_decision.detjson"
SOURCE_FLOW = ROOT / "pack" / "lang_flow_type_collision_rename_v1" / "flow_type_collision_rename.detjson"
SOURCE_TUCK = ROOT / "pack" / "lang_tuck_ssot_acceptance_handoff_v1" / "tuck_ssot_acceptance_handoff.detjson"
PREVIOUS_CHECKER = ROOT / "tests" / "run_lang_product_path_transition_closure_rebase_check.py"

WORK_ITEM = "LANG_SSOT_LANDING_COORDINATION_REBASE_V1"
NEXT = "LANG_PRIME_SSOT_ACCEPTANCE_REQUEST_V1"
TARGETS = [
    ("prime_derivative_notation", SOURCE_PRIME, "LANG_PRIME_SSOT_ACCEPTANCE_REQUEST_V1"),
    ("flow_history_naming_split", SOURCE_FLOW, "LANG_FLOW_HISTORY_SSOT_ACCEPTANCE_REQUEST_V1"),
    ("tuck_constraint_layer", SOURCE_TUCK, "LANG_TUCK_SSOT_ACCEPTANCE_REQUEST_V1"),
]
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
    "backward_compat_break_claim",
]


def fail(message: str) -> None:
    print(f"lang_ssot_landing_coordination_rebase_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_FLOW,
        SOURCE_TUCK,
        PREVIOUS_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        "prime_derivative_notation",
        "flow_history_naming_split",
        "tuck_constraint_layer",
        "No `docs/ssot/**` edit",
        "No SSOT landed claim",
        "SSOT landing coordination rebase: `1/1 = 100%`",
        "긴급 언어 결정 SSOT 반영: `0/3 = 0%`",
        "ROADMAP_V2 전체: `queue-expanded 68/90 = 76%`",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "0/3 = 0%", "1/1 = 100%", "68/90 = 76%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "urgent SSOT landing queue remains `0/3 = 0%`",
            "Prime derivative notation",
            "Flow/history naming split",
            "Tuck constraint layer",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.ssot_landing_coordination_rebase.v1",
            "lang_ssot_landing_coordination_rebase_v1",
            "SSOT landing coordination rebase: 1/1 = 100%",
            "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
            "ROADMAP_V2 전체: queue-expanded 68/90 = 76%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_ssot_landing_coordination_rebase_v1",
        "kind": "lang_ssot_landing_coordination_rebase",
        "ssot_landing_coordination_rebase_claim": True,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_PRODUCT_PATH_TRANSITION_CLOSURE_REBASE_V1",
        "proposal_doc": "docs/context/proposals/LANG_SSOT_LANDING_COORDINATION_REBASE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_LANDING_COORDINATION_REBASE_20260606.md",
        "decision_manifest": "pack/lang_ssot_landing_coordination_rebase_v1/ssot_landing_coordination_rebase.detjson",
        "source_product_path_transition_closure": "pack/lang_product_path_transition_closure_rebase_v1/product_path_transition_closure_rebase.detjson",
        "source_prime_derivative_notation_decision": "pack/lang_prime_derivative_notation_decision_v1/prime_derivative_notation_decision.detjson",
        "source_flow_type_collision_rename": "pack/lang_flow_type_collision_rename_v1/flow_type_collision_rename.detjson",
        "source_tuck_ssot_acceptance_handoff": "pack/lang_tuck_ssot_acceptance_handoff_v1/tuck_ssot_acceptance_handoff.detjson",
        "ssot_landing_coordination_rebase_closed": 1,
        "ssot_landing_coordination_rebase_total": 1,
        "ssot_landing_coordination_rebase_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 68,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 76,
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
        "source_product_path_transition_closure",
        "source_prime_derivative_notation_decision",
        "source_flow_type_collision_rename",
        "source_tuck_ssot_acceptance_handoff",
    ]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.ssot_landing_coordination_rebase.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    if manifest.get("ssot_landing_coordination_rebase_claim") is not True:
        fail("coordination claim must be true")
    for key in FALSE_FLAGS:
        if manifest.get(key) is not False:
            fail(f"manifest {key} must be false, got {manifest.get(key)!r}")

    targets = manifest.get("coordination_targets", [])
    if [(row.get("id"), ROOT / row.get("source", ""), row.get("next_item")) for row in targets] != TARGETS:
        fail(f"coordination targets mismatch: {targets!r}")
    for index, row in enumerate(targets, start=1):
        if row.get("order") != index:
            fail(f"target order mismatch: {row!r}")
        if row.get("ssot_landed") is not False:
            fail(f"target ssot_landed must be false: {row!r}")
        if row.get("parser_landed") is not False or row.get("runtime_landed") is not False:
            fail(f"target parser/runtime landed must be false: {row!r}")
        for key in ["decision", "recommended_acceptance_text"]:
            if not row.get(key):
                fail(f"target missing {key}: {row!r}")

    policy = manifest.get("coordination_policy", {})
    expected_policy = {
        "codex_may_edit_docs_ssot": False,
        "ssot_landing_counter_changes_now": False,
        "acceptance_requires_user_or_ssot_maintainer": True,
        "acceptance_requests_are_docs_context_only": True,
    }
    if policy != expected_policy:
        fail(f"coordination policy mismatch: {policy!r}")

    required_blocked = {
        "docs_ssot_edit",
        "ssot_landed",
        "parser_frontdoor_change",
        "runtime_surface_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "product_ui_change",
        "backward_compat_break",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")

    expected_plans = {
        "ssot_landing_coordination_rebase": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 68, "total": 90, "percent": 76},
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
    if closure.get("urgent_ssot_landed_plan") != {"closed": 0, "total": 3, "percent": 0}:
        fail(f"closure urgent ssot progress mismatch: {closure.get('urgent_ssot_landed_plan')!r}")

    prime = load_json(SOURCE_PRIME)
    if [row.get("surface") for row in prime.get("selected_notations", [])] != ["위치'", "위치''"]:
        fail(f"prime selected notations mismatch: {prime.get('selected_notations')!r}")
    for row in prime.get("selected_notations", []):
        if row.get("ssot_landed") is not False:
            fail(f"prime notation ssot_landed must be false: {row!r}")

    flow = load_json(SOURCE_FLOW)
    naming = flow.get("naming_decision", {})
    if naming.get("kept_connector_term") != "흐름" or naming.get("selected_value_type_name") != "이력":
        fail(f"flow naming decision mismatch: {naming!r}")
    for key in ["ssot_landed", "parser_landed", "runtime_landed"]:
        if naming.get(key) is not False:
            fail(f"flow naming {key} must be false")

    tuck = load_json(SOURCE_TUCK)
    packet = tuck.get("handoff_packet", {})
    if packet.get("selected_layer_name") != "턱" or packet.get("proposed_surface_family") != "턱-row":
        fail(f"tuck handoff packet mismatch: {packet!r}")
    if tuck.get("ssot_landed_claim") is not False:
        fail("tuck handoff ssot landed claim must be false")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_ssot_landing_coordination_rebase_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    require_contains(
        PACK / "golden.jsonl",
        [
            WORK_ITEM,
            "schema: ddn.language.ssot_landing_coordination_rebase.v1",
            "coordination targets: 3",
            "urgent ssot landed: 0/3 = 0%",
            "coordination rebase: 1/1 = 100%",
            "roadmap: 68/90 = 76%",
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
    print("lang_ssot_landing_coordination_rebase_check: PASS")


if __name__ == "__main__":
    main()

