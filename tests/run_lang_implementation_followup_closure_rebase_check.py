from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_IMPLEMENTATION_FOLLOWUP_CLOSURE_REBASE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_IMPLEMENTATION_FOLLOWUP_CLOSURE_REBASE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_IMPLEMENTATION_FOLLOWUP_CLOSURE_REBASE_20260606.md"
PACK = ROOT / "pack" / "lang_implementation_followup_closure_rebase_v1"
MANIFEST = PACK / "implementation_followup_closure_rebase.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_implementation_followup_closure_rebase_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_READINESS = ROOT / "pack" / "lang_implementation_readiness_rebase_v1" / "implementation_readiness_rebase.detjson"
SOURCE_PRIME = ROOT / "pack" / "lang_prime_parser_frontdoor_spike_v1" / "prime_parser_frontdoor_spike.detjson"
SOURCE_CONNECT = ROOT / "pack" / "lang_connect_seum_lowering_parser_spike_v1" / "connect_seum_lowering_parser_spike.detjson"
SOURCE_VERLET = ROOT / "pack" / "lang_velocity_verlet_fixed64_order_v1" / "velocity_verlet_fixed64_order.detjson"
SOURCE_DULTRA = ROOT / "pack" / "lang_dultra_recorded_replay_contract_v1" / "dultra_recorded_replay_contract.detjson"
SOURCE_OWNER = ROOT / "pack" / "lang_owner_inner_seum_parser_boundary_spike_v1" / "owner_inner_seum_parser_boundary_spike.detjson"
PREVIOUS_CHECKER = ROOT / "tests" / "run_lang_owner_inner_seum_parser_boundary_spike_check.py"

WORK_ITEM = "LANG_IMPLEMENTATION_FOLLOWUP_CLOSURE_REBASE_V1"
NEXT = "LANG_FLOW_HISTORY_ALIAS_MIGRATION_PLAN_V1"

CLOSED_ITEMS = [
    ("LANG_IMPLEMENTATION_READINESS_REBASE_V1", SOURCE_READINESS),
    ("LANG_PRIME_PARSER_FRONTDOOR_SPIKE_V1", SOURCE_PRIME),
    ("LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1", SOURCE_CONNECT),
    ("LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_V1", SOURCE_VERLET),
    ("LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_V1", SOURCE_DULTRA),
    ("LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1", SOURCE_OWNER),
]

NEXT_QUEUE = [
    ("LANG_IMPLEMENTATION_FOLLOWUP_CLOSURE_REBASE_V1", "closed"),
    ("LANG_FLOW_HISTORY_ALIAS_MIGRATION_PLAN_V1", "next"),
    ("LANG_TUCK_CONSTRAINT_SURFACE_SHAPE_PROPOSAL_V1", "planned"),
    ("LANG_VELOCITY_VERLET_RUNTIME_GATE_REBASE_V1", "planned"),
    ("LANG_DULTRA_REPLAY_ARTIFACT_IMPLEMENTATION_GATE_V1", "planned"),
    ("LANG_OWNER_INNER_SEUM_RUNTIME_SCOPE_REBASE_V1", "planned"),
]

RISK_NEXT = {
    "flow_type_collision": "LANG_FLOW_HISTORY_ALIAS_MIGRATION_PLAN_V1",
    "tuck_constraint_layer": "LANG_TUCK_CONSTRAINT_SURFACE_SHAPE_PROPOSAL_V1",
    "velocity_verlet_runtime": "LANG_VELOCITY_VERLET_RUNTIME_GATE_REBASE_V1",
    "dultra_replay_artifact_runtime": "LANG_DULTRA_REPLAY_ARTIFACT_IMPLEMENTATION_GATE_V1",
    "owner_inner_seum_runtime_scope": "LANG_OWNER_INNER_SEUM_RUNTIME_SCOPE_REBASE_V1",
}


def fail(message: str) -> None:
    print(f"lang_implementation_followup_closure_rebase_check: FAIL: {message}", file=sys.stderr)
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
        PREVIOUS_CHECKER,
        *[path for _, path in CLOSED_ITEMS],
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        "implementation-readiness follow-up queue",
        "LANG_FLOW_HISTORY_ALIAS_MIGRATION_PLAN_V1",
        "흐름",
        "언어 구현 준비 후속 계획: 6/6 = 100%",
        "언어 구현 후속 closure rebase: 1/1 = 100%",
        "다음 언어 구현 위험 제거 계획: 1/6 = 17%",
        "ROADMAP_V2 전체: queue-expanded 54/90 = 60%",
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, tokens + ["No SSOT edit by Codex"])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            WORK_ITEM,
            NEXT,
            "flow port terminology should keep 흐름",
            "No parser/frontdoor change",
            "No runtime or stdlib surface change",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.implementation_followup_closure_rebase.v1",
            "lang_implementation_followup_closure_rebase_v1",
            "언어 구현 준비 후속 계획: 6/6 = 100%",
            "언어 구현 후속 closure rebase: 1/1 = 100%",
            "다음 언어 구현 위험 제거 계획: 1/6 = 17%",
            "ROADMAP_V2 전체: queue-expanded 54/90 = 60%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_implementation_followup_closure_rebase_v1",
        "kind": "lang_implementation_followup_closure_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "closure_rebase_claim": True,
        "flow_runtime_rename_landed_claim": False,
        "tuck_parser_runtime_landed_claim": False,
        "velocity_verlet_runtime_landed_claim": False,
        "dultra_replay_runtime_landed_claim": False,
        "owner_inner_seum_runtime_landed_claim": False,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1",
        "proposal_doc": "docs/context/proposals/LANG_IMPLEMENTATION_FOLLOWUP_CLOSURE_REBASE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_IMPLEMENTATION_FOLLOWUP_CLOSURE_REBASE_20260606.md",
        "decision_manifest": "pack/lang_implementation_followup_closure_rebase_v1/implementation_followup_closure_rebase.detjson",
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
        "language_risk_removal_closed": 1,
        "language_risk_removal_total": 6,
        "language_risk_removal_percent": 17,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 54,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 60,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.implementation_followup_closure_rebase.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")

    expected_flags = {
        "closure_rebase_claim": True,
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "flow_runtime_rename_landed_claim": False,
        "tuck_parser_runtime_landed_claim": False,
        "velocity_verlet_runtime_landed_claim": False,
        "dultra_replay_runtime_landed_claim": False,
        "owner_inner_seum_runtime_landed_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")

    closed = manifest.get("closed_followup_items", [])
    if len(closed) != len(CLOSED_ITEMS):
        fail(f"closed item count mismatch: {len(closed)}")
    for index, ((item, path), row) in enumerate(zip(CLOSED_ITEMS, closed), start=1):
        if row.get("order") != index:
            fail(f"closed item order mismatch: {row!r}")
        if row.get("item") != item:
            fail(f"closed item mismatch: {row!r}")
        if row.get("path") != str(path.relative_to(ROOT)).replace("\\", "/"):
            fail(f"closed item path mismatch: {row!r}")
        if row.get("status") != "closed":
            fail(f"closed item status mismatch: {row!r}")
        require(path)

    capability_ids = {row.get("id") for row in manifest.get("closed_capabilities", [])}
    expected_capabilities = {
        "prime_parser_frontdoor",
        "connect_seum_rows",
        "velocity_verlet_fixed64_order",
        "dultra_recorded_replay_contract",
        "owner_inner_seum_parser_boundary",
    }
    if capability_ids != expected_capabilities:
        fail(f"capability ids mismatch: {capability_ids!r}")

    risks = {row.get("id"): row for row in manifest.get("remaining_risk_classifications", [])}
    if set(risks) != set(RISK_NEXT):
        fail(f"risk ids mismatch: {set(risks)!r}")
    for risk_id, next_item in RISK_NEXT.items():
        row = risks[risk_id]
        if row.get("next") != next_item:
            fail(f"risk {risk_id} next mismatch: {row!r}")
        if not row.get("classification") or not row.get("reason"):
            fail(f"risk {risk_id} missing classification/reason: {row!r}")

    queue = manifest.get("recommended_next_queue", [])
    if len(queue) != len(NEXT_QUEUE):
        fail(f"next queue count mismatch: {len(queue)}")
    for index, ((item, status), row) in enumerate(zip(NEXT_QUEUE, queue), start=1):
        if row.get("order") != index or row.get("item") != item or row.get("status") != status:
            fail(f"next queue row mismatch: {row!r}")

    required_blocked = {
        "docs_ssot_edit",
        "parser_frontdoor_change",
        "runtime_surface_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "product_ui_change",
        "flow_runtime_rename_landed",
        "tuck_parser_runtime_landed",
        "velocity_verlet_runtime_landed",
        "dultra_replay_runtime_landed",
        "owner_inner_seum_runtime_landed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")

    expected_plans = {
        "implementation_readiness_followup_plan": {"closed": 6, "total": 6, "percent": 100},
        "implementation_followup_closure_rebase_plan": {"closed": 1, "total": 1, "percent": 100},
        "language_risk_removal_plan": {"closed": 1, "total": 6, "percent": 17},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 54, "total": 90, "percent": 60},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"{key} mismatch: {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    readiness = load_json(SOURCE_READINESS)
    classifications = {row.get("id"): row for row in readiness.get("readiness_classifications", [])}
    if classifications.get("flow_type_collision", {}).get("next") != "LANG_FLOW_HISTORY_ALIAS_MIGRATION_PLAN_V1":
        fail("readiness rebase must route flow collision to flow history migration")

    prime = load_json(SOURCE_PRIME)
    if prime.get("next_item") != "LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1":
        fail(f"prime next mismatch: {prime.get('next_item')!r}")

    connect = load_json(SOURCE_CONNECT)
    if connect.get("next_item") != "LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_V1":
        fail(f"connect next mismatch: {connect.get('next_item')!r}")

    verlet = load_json(SOURCE_VERLET)
    if verlet.get("next_item") != "LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_V1":
        fail(f"velocity verlet next mismatch: {verlet.get('next_item')!r}")

    dultra = load_json(SOURCE_DULTRA)
    if dultra.get("next_item") != "LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1":
        fail(f"dultra next mismatch: {dultra.get('next_item')!r}")

    owner = load_json(SOURCE_OWNER)
    if owner.get("next_item") != WORK_ITEM:
        fail(f"owner next mismatch: {owner.get('next_item')!r}")
    if owner.get("implementation_readiness_followup_plan") != {"closed": 6, "total": 6, "percent": 100}:
        fail(f"owner followup progress mismatch: {owner.get('implementation_readiness_followup_plan')!r}")

    for item, path in CLOSED_ITEMS:
        require_contains(path, [item])


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_implementation_followup_closure_rebase_v1"], timeout=240)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_previous_checker() -> None:
    proc = run([sys.executable, str(PREVIOUS_CHECKER.relative_to(ROOT))], timeout=600)
    if proc.returncode != 0:
        fail(f"previous checker failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_pack_golden()
    check_previous_checker()
    require_docs_ssot_clean()
    print("lang_implementation_followup_closure_rebase_check: PASS")


if __name__ == "__main__":
    main()
