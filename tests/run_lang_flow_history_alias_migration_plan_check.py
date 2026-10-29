from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_FLOW_HISTORY_ALIAS_MIGRATION_PLAN_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_FLOW_HISTORY_ALIAS_MIGRATION_PLAN_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_FLOW_HISTORY_ALIAS_MIGRATION_PLAN_20260606.md"
PACK = ROOT / "pack" / "lang_flow_history_alias_migration_plan_v1"
MANIFEST = PACK / "flow_history_alias_migration_plan.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_flow_history_alias_migration_plan_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_FLOW_RENAME = ROOT / "pack" / "lang_flow_type_collision_rename_v1" / "flow_type_collision_rename.detjson"
SOURCE_CLOSURE = ROOT / "pack" / "lang_implementation_followup_closure_rebase_v1" / "implementation_followup_closure_rebase.detjson"
PREVIOUS_CHECKER = ROOT / "tests" / "run_lang_implementation_followup_closure_rebase_check.py"
FLOW_RENAME_CHECKER = ROOT / "tests" / "run_lang_flow_type_collision_rename_check.py"

WORK_ITEM = "LANG_FLOW_HISTORY_ALIAS_MIGRATION_PLAN_V1"
NEXT = "LANG_TUCK_CONSTRAINT_SURFACE_SHAPE_PROPOSAL_V1"

ALIAS_ROWS = [
    ("이력.만들기", "흐름.만들기", ["흐름만들기"]),
    ("이력.밀어넣기", "흐름.밀어넣기", ["흐름넣기", "흐름추가"]),
    ("이력.차림", "흐름.차림", ["흐름값들"]),
    ("이력.최근값", "흐름.최근값", ["흐름최근"]),
    ("이력.길이", "흐름.길이", ["흐름길이"]),
    ("이력.용량", "흐름.용량", ["흐름용량"]),
    ("이력.비우기", "흐름.비우기", ["흐름비우기"]),
    ("이력.잘라보기", "흐름.잘라보기", ["흐름잘라보기", "흐름최근N"]),
]

GATES = [
    ("alias_plan", "closed_now"),
    ("stdlib_alias_addition", "planned"),
    ("view_meta_bridge", "planned"),
    ("resource_schema_bridge", "planned"),
    ("docs_and_examples_bridge", "planned"),
    ("removal_gate", "blocked"),
]


def fail(message: str) -> None:
    print(f"lang_flow_history_alias_migration_plan_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_FLOW_RENAME,
        SOURCE_CLOSURE,
        PREVIOUS_CHECKER,
        FLOW_RENAME_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        "흐름",
        "이력",
        "흐름.만들기",
        "ddn.stream.v1",
        "가격흐름_길이",
        "No `docs/ssot/**` edit",
        "No `이력.*` alias landed claim",
        "No `흐름.*` removal claim",
        "다음 언어 구현 위험 제거 계획: 2/6 = 33%",
        "Flow history alias migration plan: 1/1 = 100%",
        "ROADMAP_V2 전체: queue-expanded 55/90 = 61%",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "이력.*", "흐름.*", "No `docs/ssot/**` edit", "2/6 = 33%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "Keep `흐름` for connector/port flow",
            "Use `이력` as the future canonical value history/ring-buffer family",
            "No parser/frontdoor change",
            "No runtime or stdlib surface change",
            "No `이력.*` alias landed claim",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.flow_history_alias_migration_plan.v1",
            "lang_flow_history_alias_migration_plan_v1",
            "다음 언어 구현 위험 제거 계획: 2/6 = 33%",
            "Flow history alias migration plan: 1/1 = 100%",
            "ROADMAP_V2 전체: queue-expanded 55/90 = 61%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_flow_history_alias_migration_plan_v1",
        "kind": "lang_flow_history_alias_migration_plan",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "flow_history_alias_migration_plan_claim": True,
        "history_alias_landed_claim": False,
        "flow_alias_removed_claim": False,
        "ddn_history_schema_landed_claim": False,
        "sidecar_rename_landed_claim": False,
        "backward_compat_break_claim": False,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_IMPLEMENTATION_FOLLOWUP_CLOSURE_REBASE_V1",
        "proposal_doc": "docs/context/proposals/LANG_FLOW_HISTORY_ALIAS_MIGRATION_PLAN_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_FLOW_HISTORY_ALIAS_MIGRATION_PLAN_20260606.md",
        "decision_manifest": "pack/lang_flow_history_alias_migration_plan_v1/flow_history_alias_migration_plan.detjson",
        "source_flow_type_collision_rename": "pack/lang_flow_type_collision_rename_v1/flow_type_collision_rename.detjson",
        "source_implementation_followup_closure_rebase": "pack/lang_implementation_followup_closure_rebase_v1/implementation_followup_closure_rebase.detjson",
        "selected_value_family": "이력",
        "kept_connector_term": "흐름",
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
        "language_risk_removal_closed": 2,
        "language_risk_removal_total": 6,
        "language_risk_removal_percent": 33,
        "flow_history_alias_migration_plan_closed": 1,
        "flow_history_alias_migration_plan_total": 1,
        "flow_history_alias_migration_plan_percent": 100,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 55,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 61,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.flow_history_alias_migration_plan.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")

    expected_flags = {
        "flow_history_alias_migration_plan_claim": True,
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "history_alias_landed_claim": False,
        "flow_alias_removed_claim": False,
        "ddn_history_schema_landed_claim": False,
        "sidecar_rename_landed_claim": False,
        "backward_compat_break_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")

    policy = manifest.get("naming_policy", {})
    expected_policy = {
        "keep_flow_for": "port_connector_flow",
        "kept_connector_term": "흐름",
        "future_value_family": "이력",
        "preserve_current_flow_value_family_until_removal_gate": True,
        "ssot_landed": False,
        "runtime_landed": False,
        "stdlib_landed": False,
        "parser_landed": False,
    }
    if policy != expected_policy:
        fail(f"naming policy mismatch: {policy!r}")

    rows = manifest.get("alias_migration_family", [])
    if len(rows) != len(ALIAS_ROWS):
        fail(f"alias row count mismatch: {len(rows)}")
    for row, (canonical, compat, legacy_aliases) in zip(rows, ALIAS_ROWS):
        if row.get("canonical") != canonical:
            fail(f"canonical mismatch: {row!r}")
        if row.get("compat") != compat:
            fail(f"compat mismatch: {row!r}")
        if row.get("legacy_aliases") != legacy_aliases:
            fail(f"legacy aliases mismatch: {row!r}")
        if row.get("action") != "add_alias_before_removal":
            fail(f"alias action mismatch: {row!r}")
        if row.get("landed_now") is not False:
            fail(f"alias row must not be landed now: {row!r}")

    gates = manifest.get("migration_gates", [])
    if len(gates) != len(GATES):
        fail(f"migration gate count mismatch: {len(gates)}")
    for index, ((gate_id, status), row) in enumerate(zip(GATES, gates), start=1):
        if row.get("order") != index or row.get("id") != gate_id or row.get("status") != status:
            fail(f"migration gate mismatch: {row!r}")
        if not row.get("required_evidence"):
            fail(f"migration gate missing evidence: {row!r}")

    required_surfaces = {
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
    }
    if set(manifest.get("compatibility_surfaces_to_preserve", [])) != required_surfaces:
        fail(f"compatibility surfaces mismatch: {manifest.get('compatibility_surfaces_to_preserve')!r}")

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
        "history_alias_landed",
        "flow_alias_removed",
        "ddn_history_schema_landed",
        "sidecar_rename_landed",
        "backward_compat_break",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")

    expected_plans = {
        "language_risk_removal_plan": {"closed": 2, "total": 6, "percent": 33},
        "flow_history_alias_migration_plan": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 55, "total": 90, "percent": 61},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"{key} mismatch: {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    flow = load_json(SOURCE_FLOW_RENAME)
    if flow.get("naming_decision", {}).get("selected_value_type_name") != "이력":
        fail("source flow rename must select 이력")
    if flow.get("naming_decision", {}).get("kept_connector_term") != "흐름":
        fail("source flow rename must keep 흐름")
    if flow.get("flow_type_rename_runtime_landed_claim") is not False:
        fail("source flow rename must not claim runtime landing")
    if flow.get("stream_alias_removed_claim") is not False:
        fail("source flow rename must not claim alias removal")

    closure = load_json(SOURCE_CLOSURE)
    risks = {row.get("id"): row for row in closure.get("remaining_risk_classifications", [])}
    flow_risk = risks.get("flow_type_collision")
    if flow_risk is None:
        fail("closure rebase missing flow_type_collision risk")
    if flow_risk.get("next") != WORK_ITEM:
        fail(f"closure flow next mismatch: {flow_risk!r}")
    if closure.get("language_risk_removal_plan") != {"closed": 1, "total": 6, "percent": 17}:
        fail(f"source closure risk progress mismatch: {closure.get('language_risk_removal_plan')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected_stdout = [
        WORK_ITEM,
        "flow history alias migration plan sealed",
        "schema: ddn.language.flow_history_alias_migration_plan.v1",
        "keep connector term: 흐름",
        "future value family: 이력",
        "risk removal: 2/6 = 33%",
        "runtime landed: false",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/lang_flow_history_alias_migration_plan_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected_stdout:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_flow_history_alias_migration_plan_v1"], timeout=240)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_previous_checkers() -> None:
    for checker in [PREVIOUS_CHECKER, FLOW_RENAME_CHECKER]:
        proc = run([sys.executable, str(checker.relative_to(ROOT))], timeout=900)
        if proc.returncode != 0:
            fail(f"previous checker failed ({checker.relative_to(ROOT)}):\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_golden()
    check_pack_golden()
    check_previous_checkers()
    require_docs_ssot_clean()
    print("lang_flow_history_alias_migration_plan_check: PASS")


if __name__ == "__main__":
    main()
