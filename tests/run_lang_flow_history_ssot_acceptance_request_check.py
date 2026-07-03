from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_FLOW_HISTORY_SSOT_ACCEPTANCE_REQUEST_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_FLOW_HISTORY_SSOT_ACCEPTANCE_REQUEST_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_FLOW_HISTORY_ACCEPTANCE_REQUEST_20260606.md"
PACK = ROOT / "pack" / "lang_flow_history_ssot_acceptance_request_v1"
MANIFEST = PACK / "flow_history_ssot_acceptance_request.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_flow_history_ssot_acceptance_request_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_PRIME_REQUEST = ROOT / "pack" / "lang_prime_ssot_acceptance_request_v1" / "prime_ssot_acceptance_request.detjson"
SOURCE_COORDINATION = ROOT / "pack" / "lang_ssot_landing_coordination_rebase_v1" / "ssot_landing_coordination_rebase.detjson"
SOURCE_FLOW_DECISION = ROOT / "pack" / "lang_flow_type_collision_rename_v1" / "flow_type_collision_rename.detjson"
SOURCE_MIGRATION = ROOT / "pack" / "lang_flow_history_alias_migration_plan_v1" / "flow_history_alias_migration_plan.detjson"
SOURCE_BRIDGE = ROOT / "pack" / "lang_history_alias_stdlib_bridge_v1" / "history_alias_stdlib_bridge.detjson"
PREVIOUS_CHECKER = ROOT / "tests" / "run_lang_prime_ssot_acceptance_request_check.py"

WORK_ITEM = "LANG_FLOW_HISTORY_SSOT_ACCEPTANCE_REQUEST_V1"
NEXT = "LANG_TUCK_SSOT_ACCEPTANCE_REQUEST_V1"
PREFERRED_SURFACES = [
    "이력.만들기",
    "이력.밀어넣기",
    "이력.차림",
    "이력.최근값",
    "이력.길이",
    "이력.용량",
    "이력.비우기",
    "이력.잘라보기",
]
COMPAT_SURFACES = [
    "흐름.*",
    "흐름만들기",
    "흐름넣기",
    "흐름추가",
    "흐름값들",
    "흐름최근",
    "흐름길이",
    "흐름용량",
    "흐름비우기",
    "흐름잘라보기",
    "흐름최근N",
    "ddn.stream.v1",
    "stream sidecar suffixes",
    "가격흐름_길이",
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
    "flow_history_ssot_landed_claim",
    "flow_alias_removed_claim",
    "ddn_history_schema_landed_claim",
    "sidecar_rename_landed_claim",
    "backward_compat_break_claim",
]


def fail(message: str) -> None:
    print(f"lang_flow_history_ssot_acceptance_request_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_PRIME_REQUEST,
        SOURCE_COORDINATION,
        SOURCE_FLOW_DECISION,
        SOURCE_MIGRATION,
        SOURCE_BRIDGE,
        PREVIOUS_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        "흐름",
        "이력",
        "Recommended SSOT Text",
        "No `docs/ssot/**` edit",
        "No SSOT landed claim",
        "No `흐름.*` removal",
        "SSOT acceptance request queue: `2/3 = 67%`",
        "Flow/History SSOT acceptance request: `1/1 = 100%`",
        "긴급 언어 결정 SSOT 반영: `0/3 = 0%`",
        "ROADMAP_V2 전체: `queue-expanded 70/90 = 78%`",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "2/3 = 67%", "0/3 = 0%", "70/90 = 78%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "`흐름`: connector/port flow term",
            "`이력`: value-history/ring-buffer family",
            "No `흐름.*` removal",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.flow_history_ssot_acceptance_request.v1",
            "lang_flow_history_ssot_acceptance_request_v1",
            "SSOT acceptance request queue: 2/3 = 67%",
            "Flow/History SSOT acceptance request: 1/1 = 100%",
            "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
            "ROADMAP_V2 전체: queue-expanded 70/90 = 78%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_flow_history_ssot_acceptance_request_v1",
        "kind": "lang_flow_history_ssot_acceptance_request",
        "flow_history_ssot_acceptance_request_claim": True,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_PRIME_SSOT_ACCEPTANCE_REQUEST_V1",
        "proposal_doc": "docs/context/proposals/LANG_FLOW_HISTORY_SSOT_ACCEPTANCE_REQUEST_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_FLOW_HISTORY_ACCEPTANCE_REQUEST_20260606.md",
        "decision_manifest": "pack/lang_flow_history_ssot_acceptance_request_v1/flow_history_ssot_acceptance_request.detjson",
        "source_prime_ssot_acceptance_request": "pack/lang_prime_ssot_acceptance_request_v1/prime_ssot_acceptance_request.detjson",
        "source_ssot_landing_coordination": "pack/lang_ssot_landing_coordination_rebase_v1/ssot_landing_coordination_rebase.detjson",
        "source_flow_type_collision_rename": "pack/lang_flow_type_collision_rename_v1/flow_type_collision_rename.detjson",
        "source_flow_history_alias_migration_plan": "pack/lang_flow_history_alias_migration_plan_v1/flow_history_alias_migration_plan.detjson",
        "source_history_alias_stdlib_bridge": "pack/lang_history_alias_stdlib_bridge_v1/history_alias_stdlib_bridge.detjson",
        "ssot_acceptance_request_queue_closed": 2,
        "ssot_acceptance_request_queue_total": 3,
        "ssot_acceptance_request_queue_percent": 67,
        "flow_history_ssot_acceptance_request_closed": 1,
        "flow_history_ssot_acceptance_request_total": 1,
        "flow_history_ssot_acceptance_request_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 70,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 78,
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
        "source_prime_ssot_acceptance_request",
        "source_ssot_landing_coordination",
        "source_flow_type_collision_rename",
        "source_flow_history_alias_migration_plan",
        "source_history_alias_stdlib_bridge",
    ]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.flow_history_ssot_acceptance_request.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    if manifest.get("flow_history_ssot_acceptance_request_claim") is not True:
        fail("request claim must be true")
    for key in FALSE_FLAGS:
        if manifest.get(key) is not False:
            fail(f"manifest {key} must be false, got {manifest.get(key)!r}")

    request = manifest.get("request", {})
    if request.get("target_id") != "flow_history_naming_split":
        fail(f"request target mismatch: {request.get('target_id')!r}")
    if request.get("kept_connector_term") != "흐름":
        fail(f"kept connector term mismatch: {request.get('kept_connector_term')!r}")
    if request.get("selected_value_history_family") != "이력":
        fail(f"value history family mismatch: {request.get('selected_value_history_family')!r}")
    if request.get("preferred_surfaces") != PREFERRED_SURFACES:
        fail(f"preferred surfaces mismatch: {request.get('preferred_surfaces')!r}")
    if request.get("compatibility_surfaces_to_preserve") != COMPAT_SURFACES:
        fail(f"compatibility surfaces mismatch: {request.get('compatibility_surfaces_to_preserve')!r}")
    for token in ["`흐름` remains", "`이력.*`", "separate compatibility and migration gate"]:
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
        "flow_alias_removed",
        "ddn_history_schema_landed",
        "sidecar_rename_landed",
        "backward_compat_break",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")

    expected_plans = {
        "ssot_acceptance_request_queue": {"closed": 2, "total": 3, "percent": 67},
        "flow_history_ssot_acceptance_request": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 70, "total": 90, "percent": 78},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    previous = load_json(SOURCE_PRIME_REQUEST)
    if previous.get("next_item") != WORK_ITEM:
        fail(f"previous request next expected {WORK_ITEM}, got {previous.get('next_item')!r}")
    if previous.get("ssot_acceptance_request_queue") != {"closed": 1, "total": 3, "percent": 33}:
        fail(f"previous request queue mismatch: {previous.get('ssot_acceptance_request_queue')!r}")

    coordination = load_json(SOURCE_COORDINATION)
    target = coordination.get("coordination_targets", [])[1]
    if target.get("id") != "flow_history_naming_split" or target.get("next_item") != WORK_ITEM:
        fail(f"coordination flow target mismatch: {target!r}")
    if target.get("ssot_landed") is not False:
        fail("coordination flow target ssot_landed must be false")

    decision = load_json(SOURCE_FLOW_DECISION)
    naming = decision.get("naming_decision", {})
    if naming.get("kept_connector_term") != "흐름" or naming.get("selected_value_type_name") != "이력":
        fail(f"flow decision mismatch: {naming!r}")
    for key in ["ssot_landed", "parser_landed", "runtime_landed", "stdlib_landed"]:
        if naming.get(key) is not False:
            fail(f"flow decision {key} must be false")

    migration = load_json(SOURCE_MIGRATION)
    policy = migration.get("naming_policy", {})
    if policy.get("preserve_current_flow_value_family_until_removal_gate") is not True:
        fail(f"migration policy must preserve current flow family: {policy!r}")
    if migration.get("compatibility_surfaces_to_preserve") != COMPAT_SURFACES:
        fail("migration compatibility surfaces mismatch")

    bridge = load_json(SOURCE_BRIDGE)
    if len(bridge.get("history_alias_bridge", [])) != 8:
        fail("history alias bridge should contain 8 aliases")
    for key in ["flow_alias_removed_claim", "ddn_history_schema_landed_claim", "sidecar_rename_landed_claim", "backward_compat_break_claim"]:
        if bridge.get(key) is not False:
            fail(f"bridge {key} must remain false")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_flow_history_ssot_acceptance_request_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    require_contains(
        PACK / "golden.jsonl",
        [
            WORK_ITEM,
            "schema: ddn.language.flow_history_ssot_acceptance_request.v1",
            "split: 흐름 connector flow; 이력 value history",
            "ssot request queue: 2/3 = 67%",
            "urgent ssot landed: 0/3 = 0%",
            "roadmap: 70/90 = 78%",
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
    print("lang_flow_history_ssot_acceptance_request_check: PASS")


if __name__ == "__main__":
    main()

