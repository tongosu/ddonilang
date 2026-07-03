from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_PRODUCT_PATH_TRANSITION_CLOSURE_REBASE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_PRODUCT_PATH_TRANSITION_CLOSURE_REBASE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_PRODUCT_PATH_TRANSITION_CLOSURE_REBASE_20260606.md"
PACK = ROOT / "pack" / "lang_product_path_transition_closure_rebase_v1"
MANIFEST = PACK / "product_path_transition_closure_rebase.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_product_path_transition_closure_rebase_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_RISK_CLOSURE = ROOT / "pack" / "lang_language_risk_removal_closure_rebase_v1" / "language_risk_removal_closure_rebase.detjson"
SOURCE_PRIME = ROOT / "pack" / "lang_prime_derivative_runtime_semantics_gate_v1" / "prime_derivative_runtime_semantics_gate.detjson"
SOURCE_OWNER = ROOT / "pack" / "lang_owner_state_symbol_table_product_path_v1" / "owner_state_symbol_table_product_path.detjson"
SOURCE_HISTORY = ROOT / "pack" / "lang_history_alias_stdlib_bridge_v1" / "history_alias_stdlib_bridge.detjson"
SOURCE_DULTRA = ROOT / "pack" / "lang_dultra_replay_artifact_writer_seed_v1" / "dultra_replay_artifact_writer_seed.detjson"
SOURCE_VERLET = ROOT / "pack" / "lang_velocity_verlet_stdlib_surface_acceptance_v1" / "velocity_verlet_stdlib_surface_acceptance.detjson"
SOURCE_TUCK = ROOT / "pack" / "lang_tuck_ssot_acceptance_handoff_v1" / "tuck_ssot_acceptance_handoff.detjson"

TUCK_CHECKER = ROOT / "tests" / "run_lang_tuck_ssot_acceptance_handoff_check.py"
VERLET_CHECKER = ROOT / "tests" / "run_lang_velocity_verlet_stdlib_surface_acceptance_check.py"

WORK_ITEM = "LANG_PRODUCT_PATH_TRANSITION_CLOSURE_REBASE_V1"
NEXT = "LANG_SSOT_LANDING_COORDINATION_REBASE_V1"
CLOSED_ITEMS = [
    ("LANG_LANGUAGE_RISK_REMOVAL_CLOSURE_REBASE_V1", SOURCE_RISK_CLOSURE, {"closed": 1, "total": 7, "percent": 14}, "LANG_PRIME_DERIVATIVE_RUNTIME_SEMANTICS_GATE_V1"),
    ("LANG_PRIME_DERIVATIVE_RUNTIME_SEMANTICS_GATE_V1", SOURCE_PRIME, {"closed": 2, "total": 7, "percent": 29}, "LANG_OWNER_STATE_SYMBOL_TABLE_PRODUCT_PATH_V1"),
    ("LANG_OWNER_STATE_SYMBOL_TABLE_PRODUCT_PATH_V1", SOURCE_OWNER, {"closed": 3, "total": 7, "percent": 43}, "LANG_HISTORY_ALIAS_STDLIB_BRIDGE_V1"),
    ("LANG_HISTORY_ALIAS_STDLIB_BRIDGE_V1", SOURCE_HISTORY, {"closed": 4, "total": 7, "percent": 57}, "LANG_DULTRA_REPLAY_ARTIFACT_WRITER_SEED_V1"),
    ("LANG_DULTRA_REPLAY_ARTIFACT_WRITER_SEED_V1", SOURCE_DULTRA, {"closed": 5, "total": 7, "percent": 71}, "LANG_VELOCITY_VERLET_STDLIB_SURFACE_ACCEPTANCE_V1"),
    ("LANG_VELOCITY_VERLET_STDLIB_SURFACE_ACCEPTANCE_V1", SOURCE_VERLET, {"closed": 6, "total": 7, "percent": 86}, "LANG_TUCK_SSOT_ACCEPTANCE_HANDOFF_V1"),
    ("LANG_TUCK_SSOT_ACCEPTANCE_HANDOFF_V1", SOURCE_TUCK, {"closed": 7, "total": 7, "percent": 100}, WORK_ITEM),
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
    "derivative_semantics_landed_claim",
    "owner_inner_seum_runtime_landed_claim",
    "velocity_verlet_runtime_landed_claim",
    "tuck_row_parser_landed_claim",
    "constraint_runtime_landed_claim",
]


def fail(message: str) -> None:
    print(f"lang_product_path_transition_closure_rebase_check: FAIL: {message}", file=sys.stderr)
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
        TUCK_CHECKER,
        VERLET_CHECKER,
        *[path for _, path, _, _ in CLOSED_ITEMS],
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        "Closed Product-Path Transition Items",
        "LANG_TUCK_SSOT_ACCEPTANCE_HANDOFF_V1",
        "No `docs/ssot/**` edit",
        "No SSOT landed claim",
        "언어 제품 경로 구현 전환 계획: `7/7 = 100%`",
        "언어 제품 경로 전환 closure rebase: `1/1 = 100%`",
        "긴급 언어 결정 SSOT 반영: `0/3 = 0%`",
        "ROADMAP_V2 전체: `queue-expanded 67/90 = 74%`",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "Closed Inputs", "Remaining Boundaries", "7/7 = 100%", "67/90 = 74%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "language product-path transition queue is closed",
            "Urgent language decision SSOT landing remains `0/3 = 0%`",
            "No SSOT landed claim by Codex",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.product_path_transition_closure_rebase.v1",
            "lang_product_path_transition_closure_rebase_v1",
            "언어 제품 경로 구현 전환 계획: 7/7 = 100%",
            "언어 제품 경로 전환 closure rebase: 1/1 = 100%",
            "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
            "ROADMAP_V2 전체: queue-expanded 67/90 = 74%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_product_path_transition_closure_rebase_v1",
        "kind": "lang_product_path_transition_closure_rebase",
        "product_path_transition_closure_rebase_claim": True,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_TUCK_SSOT_ACCEPTANCE_HANDOFF_V1",
        "proposal_doc": "docs/context/proposals/LANG_PRODUCT_PATH_TRANSITION_CLOSURE_REBASE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_PRODUCT_PATH_TRANSITION_CLOSURE_REBASE_20260606.md",
        "decision_manifest": "pack/lang_product_path_transition_closure_rebase_v1/product_path_transition_closure_rebase.detjson",
        "language_product_path_transition_closed": 7,
        "language_product_path_transition_total": 7,
        "language_product_path_transition_percent": 100,
        "language_product_path_transition_closure_rebase_closed": 1,
        "language_product_path_transition_closure_rebase_total": 1,
        "language_product_path_transition_closure_rebase_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 67,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 74,
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
        "source_language_risk_removal_closure",
        "source_prime_derivative_runtime_semantics_gate",
        "source_owner_state_symbol_table_product_path",
        "source_history_alias_stdlib_bridge",
        "source_dultra_replay_artifact_writer_seed",
        "source_velocity_verlet_stdlib_surface_acceptance",
        "source_tuck_ssot_acceptance_handoff",
    ]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.product_path_transition_closure_rebase.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    if manifest.get("product_path_transition_closure_rebase_claim") is not True:
        fail("closure claim must be true")
    for key in FALSE_FLAGS:
        if manifest.get(key) is not False:
            fail(f"manifest {key} must be false, got {manifest.get(key)!r}")

    closed = manifest.get("closed_product_path_transition_items", [])
    expected_items = [(item, str(path.relative_to(ROOT)).replace("\\", "/")) for item, path, _, _ in CLOSED_ITEMS]
    actual_items = [(row.get("item"), row.get("path")) for row in closed]
    if actual_items != expected_items:
        fail(f"closed transition items mismatch: {actual_items!r}")
    for index, row in enumerate(closed, start=1):
        if row.get("order") != index or row.get("status") != "closed":
            fail(f"closed transition row malformed: {row!r}")
        require(ROOT / row["path"])

    summary_ids = [row.get("id") for row in manifest.get("closed_result_summary", [])]
    expected_summary_ids = [
        "prime_derivative_gate",
        "owner_state_symbol_table",
        "history_alias_bridge",
        "dultra_replay_artifact_writer_seed",
        "velocity_verlet_stdlib_surface",
        "tuck_ssot_acceptance_handoff",
    ]
    if summary_ids != expected_summary_ids:
        fail(f"summary ids mismatch: {summary_ids!r}")
    for row in manifest.get("closed_result_summary", []):
        if row.get("runtime_landed") is not False:
            fail(f"summary runtime_landed must be false: {row!r}")

    next_queue = manifest.get("next_queue", {})
    expected_next = {
        "id": "ssot_landing_coordination",
        "next_item": NEXT,
        "reason": "urgent language decision evidence is closed but SSOT landing remains 0/3",
        "codex_ssot_edit_allowed": False,
        "initial_progress": {"closed": 0, "total": 3, "percent": 0},
    }
    if next_queue != expected_next:
        fail(f"next queue mismatch: {next_queue!r}")

    required_blocked = {
        "docs_ssot_edit",
        "ssot_landed",
        "parser_frontdoor_change",
        "runtime_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "product_ui_change",
        "derivative_semantics_landed",
        "owner_inner_seum_runtime_landed",
        "velocity_verlet_runtime_landed",
        "tuck_row_parser_landed",
        "constraint_runtime_landed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")

    expected_plans = {
        "language_product_path_transition_plan": {"closed": 7, "total": 7, "percent": 100},
        "language_product_path_transition_closure_rebase": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 67, "total": 90, "percent": 74},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    for item, path, progress, next_item in CLOSED_ITEMS:
        data = load_json(path)
        if data.get("work_item") != item:
            fail(f"{path.relative_to(ROOT)} work item expected {item}, got {data.get('work_item')!r}")
        if data.get("language_product_path_transition_plan") != progress:
            fail(f"{path.relative_to(ROOT)} transition progress mismatch: {data.get('language_product_path_transition_plan')!r}")
        if data.get("next_item") != next_item:
            fail(f"{path.relative_to(ROOT)} next item expected {next_item}, got {data.get('next_item')!r}")

    false_source_checks = [
        (SOURCE_PRIME, ["derivative_semantics_landed_claim", "derivative_runtime_product_path_landed_claim"]),
        (SOURCE_OWNER, ["runtime_owner_state_symbol_table_landed_claim", "owner_inner_seum_runtime_landed_claim"]),
        (SOURCE_DULTRA, ["dultra_replay_artifact_writer_runtime_landed_claim", "dultra_replay_verifier_landed_claim"]),
        (SOURCE_VERLET, ["velocity_verlet_runtime_landed_claim", "velocity_verlet_cli_wasm_parity_claim"]),
        (SOURCE_TUCK, ["ssot_landed_claim", "tuck_row_parser_landed_claim", "constraint_runtime_landed_claim"]),
    ]
    for path, keys in false_source_checks:
        data = load_json(path)
        for key in keys:
            if data.get(key) is not False:
                fail(f"{path.relative_to(ROOT)} {key} must remain false")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_product_path_transition_closure_rebase_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    require_contains(
        PACK / "golden.jsonl",
        [
            WORK_ITEM,
            "schema: ddn.language.product_path_transition_closure_rebase.v1",
            "closed product transition items: 7",
            "product transition: 7/7 = 100%",
            "urgent ssot landed: 0/3 = 0%",
            "roadmap: 67/90 = 74%",
            NEXT,
        ],
    )


def check_previous_checkers() -> None:
    for checker in [TUCK_CHECKER, VERLET_CHECKER]:
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
    print("lang_product_path_transition_closure_rebase_check: PASS")


if __name__ == "__main__":
    main()

