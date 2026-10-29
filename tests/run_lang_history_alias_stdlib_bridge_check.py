from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_HISTORY_ALIAS_STDLIB_BRIDGE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_HISTORY_ALIAS_STDLIB_BRIDGE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_HISTORY_ALIAS_STDLIB_BRIDGE_20260606.md"
PACK = ROOT / "pack" / "lang_history_alias_stdlib_bridge_v1"
MANIFEST = PACK / "history_alias_stdlib_bridge.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_history_alias_stdlib_bridge_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_FLOW = ROOT / "pack" / "lang_flow_history_alias_migration_plan_v1" / "flow_history_alias_migration_plan.detjson"
SOURCE_OWNER_STATE = ROOT / "pack" / "lang_owner_state_symbol_table_product_path_v1" / "owner_state_symbol_table_product_path.detjson"
OWNER_STATE_CHECKER = ROOT / "tests" / "run_lang_owner_state_symbol_table_product_path_check.py"

WORK_ITEM = "LANG_HISTORY_ALIAS_STDLIB_BRIDGE_V1"
NEXT = "LANG_DULTRA_REPLAY_ARTIFACT_WRITER_SEED_V1"
HISTORY_BRIDGE = [
    ("이력.만들기", "흐름.만들기"),
    ("이력.밀어넣기", "흐름.밀어넣기"),
    ("이력.차림", "흐름.차림"),
    ("이력.최근값", "흐름.최근값"),
    ("이력.길이", "흐름.길이"),
    ("이력.용량", "흐름.용량"),
    ("이력.비우기", "흐름.비우기"),
    ("이력.잘라보기", "흐름.잘라보기"),
]
PRESERVED = {
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
}


def fail(message: str) -> None:
    print(f"lang_history_alias_stdlib_bridge_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_FLOW,
        SOURCE_OWNER_STATE,
        OWNER_STATE_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        "이력.만들기",
        "흐름.만들기",
        "Compatibility preserved",
        "언어 제품 경로 구현 전환 계획: 4/7 = 57%",
        "History alias stdlib bridge: 1/1 = 100%",
        "ROADMAP_V2 전체: queue-expanded 63/90 = 70%",
        "No `docs/ssot/**` edit",
        "No `흐름.*` removal",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "Product Path", "4/7 = 57%", "63/90 = 70%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "`이력.*` signatures exist",
            "`흐름.*` and legacy `흐름...` aliases are preserved",
            "No runtime behavior change",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.history_alias_stdlib_bridge.v1",
            "lang_history_alias_stdlib_bridge_v1",
            "언어 제품 경로 구현 전환 계획: 4/7 = 57%",
            "History alias stdlib bridge: 1/1 = 100%",
            "ROADMAP_V2 전체: queue-expanded 63/90 = 70%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_history_alias_stdlib_bridge_v1",
        "kind": "lang_history_alias_stdlib_bridge",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": True,
        "ssot_edit_claim": False,
        "history_alias_stdlib_bridge_claim": True,
        "history_stdlib_alias_landed_claim": True,
        "history_runtime_alias_landed_claim": False,
        "flow_alias_removed_claim": False,
        "ddn_history_schema_landed_claim": False,
        "sidecar_rename_landed_claim": False,
        "backward_compat_break_claim": False,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_OWNER_STATE_SYMBOL_TABLE_PRODUCT_PATH_V1",
        "proposal_doc": "docs/context/proposals/LANG_HISTORY_ALIAS_STDLIB_BRIDGE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_HISTORY_ALIAS_STDLIB_BRIDGE_20260606.md",
        "decision_manifest": "pack/lang_history_alias_stdlib_bridge_v1/history_alias_stdlib_bridge.detjson",
        "source_flow_history_alias_migration_plan": "pack/lang_flow_history_alias_migration_plan_v1/flow_history_alias_migration_plan.detjson",
        "source_owner_state_symbol_table_product_path": "pack/lang_owner_state_symbol_table_product_path_v1/owner_state_symbol_table_product_path.detjson",
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
        "language_product_path_transition_closed": 4,
        "language_product_path_transition_total": 7,
        "language_product_path_transition_percent": 57,
        "history_alias_stdlib_bridge_closed": 1,
        "history_alias_stdlib_bridge_total": 1,
        "history_alias_stdlib_bridge_percent": 100,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 63,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 70,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    for source_key in ["source_flow_history_alias_migration_plan", "source_owner_state_symbol_table_product_path"]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.history_alias_stdlib_bridge.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")

    expected_flags = {
        "history_alias_stdlib_bridge_claim": True,
        "history_stdlib_alias_landed_claim": True,
        "history_runtime_alias_landed_claim": False,
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": True,
        "ssot_edit_claim": False,
        "flow_alias_removed_claim": False,
        "ddn_history_schema_landed_claim": False,
        "sidecar_rename_landed_claim": False,
        "backward_compat_break_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")

    bridge = [
        (row.get("history"), row.get("runtime_compat"), row.get("landed_now"))
        for row in manifest.get("history_alias_bridge", [])
    ]
    if bridge != [(history, compat, True) for history, compat in HISTORY_BRIDGE]:
        fail(f"history alias bridge mismatch: {bridge!r}")
    if set(manifest.get("compatibility_surfaces_preserved", [])) != PRESERVED:
        fail(f"compatibility surfaces mismatch: {manifest.get('compatibility_surfaces_preserved')!r}")

    for row in manifest.get("product_anchor_rows", []):
        path = ROOT / row.get("path", "")
        require(path)
        require_contains(path, row.get("tokens", []))

    required_blocked = {
        "docs_ssot_edit",
        "parser_frontdoor_change",
        "runtime_behavior_change",
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
        "language_product_path_transition_plan": {"closed": 4, "total": 7, "percent": 57},
        "history_alias_stdlib_bridge": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 63, "total": 90, "percent": 70},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    flow = load_json(SOURCE_FLOW)
    if flow.get("history_alias_landed_claim") is not False:
        fail("source flow migration plan must still predate alias landing")
    aliases = [
        (row.get("canonical"), row.get("compat"), row.get("landed_now"))
        for row in flow.get("alias_migration_family", [])
    ]
    if aliases != [(history, compat, False) for history, compat in HISTORY_BRIDGE]:
        fail(f"source flow alias family mismatch: {aliases!r}")

    owner = load_json(SOURCE_OWNER_STATE)
    if owner.get("next_item") != WORK_ITEM:
        fail(f"owner state source next item expected {WORK_ITEM}, got {owner.get('next_item')!r}")
    if owner.get("language_product_path_transition_plan") != {"closed": 3, "total": 7, "percent": 43}:
        fail(f"owner state source product transition progress mismatch: {owner.get('language_product_path_transition_plan')!r}")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_history_alias_stdlib_bridge_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    expected = [
        "LANG_HISTORY_ALIAS_STDLIB_BRIDGE_V1",
        "history alias stdlib bridge sealed",
        "schema: ddn.language.history_alias_stdlib_bridge.v1",
        "history aliases: 8",
        "product transition: 4/7 = 57%",
        "runtime landed: false",
        "next: LANG_DULTRA_REPLAY_ARTIFACT_WRITER_SEED_V1",
    ]
    require_contains(PACK / "golden.jsonl", expected)


def check_product_tests() -> None:
    for test_name in [
        "stdlib_includes_list_and_string",
        "canonicalize_stdlib_aliases_map_to_single_canonical_names",
    ]:
        proc = run(["cargo", "test", "-p", "ddonirang-lang", test_name, "--quiet"], timeout=300)
        if proc.returncode != 0:
            fail(f"cargo product stdlib test {test_name} failed:\n{proc.stdout}")


def check_previous_checker() -> None:
    proc = run([sys.executable, str(OWNER_STATE_CHECKER.relative_to(ROOT))], timeout=1200)
    if proc.returncode != 0:
        fail(f"{OWNER_STATE_CHECKER.relative_to(ROOT)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_golden()
    check_product_tests()
    check_previous_checker()
    require_docs_ssot_clean()
    print("lang_history_alias_stdlib_bridge_check: PASS")


if __name__ == "__main__":
    main()
