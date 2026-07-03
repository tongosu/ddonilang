from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_SSOT_LANDING_WAIT_STATE_REBASE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_SSOT_LANDING_WAIT_STATE_REBASE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_LANDING_WAIT_STATE_REBASE_20260606.md"
PACK = ROOT / "pack" / "lang_ssot_landing_wait_state_rebase_v1"
MANIFEST = PACK / "ssot_landing_wait_state_rebase.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_ssot_landing_wait_state_rebase_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_CLOSURE = ROOT / "pack" / "lang_ssot_acceptance_request_closure_rebase_v1" / "ssot_acceptance_request_closure_rebase.detjson"
SOURCE_PRIME = ROOT / "pack" / "lang_prime_ssot_acceptance_request_v1" / "prime_ssot_acceptance_request.detjson"
SOURCE_FLOW = ROOT / "pack" / "lang_flow_history_ssot_acceptance_request_v1" / "flow_history_ssot_acceptance_request.detjson"
SOURCE_TUCK = ROOT / "pack" / "lang_tuck_ssot_acceptance_request_v1" / "tuck_ssot_acceptance_request.detjson"
CLOSURE_CHECKER = ROOT / "tests" / "run_lang_ssot_acceptance_request_closure_rebase_check.py"

WORK_ITEM = "LANG_SSOT_LANDING_WAIT_STATE_REBASE_V1"
NEXT = "LANG_POST_SSOT_LANDING_PRODUCT_GATE_REBASE_V1"
SOURCE_ITEMS = [
    ("LANG_PRIME_SSOT_ACCEPTANCE_REQUEST_V1", "prime_derivative_notation", SOURCE_PRIME),
    ("LANG_FLOW_HISTORY_SSOT_ACCEPTANCE_REQUEST_V1", "flow_history_naming_split", SOURCE_FLOW),
    ("LANG_TUCK_SSOT_ACCEPTANCE_REQUEST_V1", "tuck_constraint_layer", SOURCE_TUCK),
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
    "post_ssot_product_gate_open_claim",
    "prime_notation_ssot_landed_claim",
    "prime_derivative_semantics_landed_claim",
    "flow_history_ssot_landed_claim",
    "ddn_history_schema_landed_claim",
    "tuck_ssot_landed_claim",
    "tuck_row_parser_landed_claim",
    "constraint_runtime_landed_claim",
]


def fail(message: str) -> None:
    print(f"lang_ssot_landing_wait_state_rebase_check: FAIL: {message}", file=sys.stderr)
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
    env = os.environ.copy()
    env.setdefault("CARGO_TARGET_DIR", str(ROOT / "build" / "cargo-target-checks"))
    return subprocess.run(
        cmd,
        cwd=ROOT,
        env=env,
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
        CLOSURE_CHECKER,
        *[path for _, _, path in SOURCE_ITEMS],
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        "Wait-State Policy",
        "SSOT landing remains `0/3 = 0%`",
        "No `docs/ssot/**` edit",
        "No SSOT landed claim",
        "SSOT acceptance request queue: `3/3 = 100%`",
        "SSOT landing wait-state rebase: `1/1 = 100%`",
        "긴급 언어 결정 SSOT 반영: `0/3 = 0%`",
        "ROADMAP_V2 전체: `queue-expanded 73/90 = 81%`",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "3/3 = 100%", "1/1 = 100%", "0/3 = 0%", "73/90 = 81%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "SSOT landed false",
            "Urgent language decision SSOT landing remains `0/3 = 0%`",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.ssot_landing_wait_state_rebase.v1",
            "lang_ssot_landing_wait_state_rebase_v1",
            "SSOT acceptance request queue: 3/3 = 100%",
            "SSOT landing wait-state rebase: 1/1 = 100%",
            "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
            "ROADMAP_V2 전체: queue-expanded 73/90 = 81%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_ssot_landing_wait_state_rebase_v1",
        "kind": "lang_ssot_landing_wait_state_rebase",
        "ssot_landing_wait_state_rebase_claim": True,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_SSOT_ACCEPTANCE_REQUEST_CLOSURE_REBASE_V1",
        "proposal_doc": "docs/context/proposals/LANG_SSOT_LANDING_WAIT_STATE_REBASE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_LANDING_WAIT_STATE_REBASE_20260606.md",
        "decision_manifest": "pack/lang_ssot_landing_wait_state_rebase_v1/ssot_landing_wait_state_rebase.detjson",
        "source_ssot_acceptance_request_closure_rebase": "pack/lang_ssot_acceptance_request_closure_rebase_v1/ssot_acceptance_request_closure_rebase.detjson",
        "source_prime_ssot_acceptance_request": "pack/lang_prime_ssot_acceptance_request_v1/prime_ssot_acceptance_request.detjson",
        "source_flow_history_ssot_acceptance_request": "pack/lang_flow_history_ssot_acceptance_request_v1/flow_history_ssot_acceptance_request.detjson",
        "source_tuck_ssot_acceptance_request": "pack/lang_tuck_ssot_acceptance_request_v1/tuck_ssot_acceptance_request.detjson",
        "ssot_acceptance_request_queue_closed": 3,
        "ssot_acceptance_request_queue_total": 3,
        "ssot_acceptance_request_queue_percent": 100,
        "ssot_acceptance_request_closure_rebase_closed": 1,
        "ssot_acceptance_request_closure_rebase_total": 1,
        "ssot_acceptance_request_closure_rebase_percent": 100,
        "ssot_landing_wait_state_rebase_closed": 1,
        "ssot_landing_wait_state_rebase_total": 1,
        "ssot_landing_wait_state_rebase_percent": 100,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 73,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 81,
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
        "source_ssot_acceptance_request_closure_rebase",
        "source_prime_ssot_acceptance_request",
        "source_flow_history_ssot_acceptance_request",
        "source_tuck_ssot_acceptance_request",
    ]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.ssot_landing_wait_state_rebase.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    if manifest.get("ssot_landing_wait_state_rebase_claim") is not True:
        fail("wait-state claim must be true")
    for key in FALSE_FLAGS:
        if manifest.get(key) is not False:
            fail(f"manifest {key} must be false, got {manifest.get(key)!r}")

    wait_state = manifest.get("wait_state", {})
    expected_wait = {
        "id": "urgent_language_ssot_landing_wait_state",
        "reason": "acceptance requests are closed but SSOT landing remains outside the Codex write boundary",
        "codex_ssot_edit_allowed": False,
        "ssot_landing_progress": {"closed": 0, "total": 3, "percent": 0},
        "product_followup_gate": "closed_until_ssot_landed_or_explicit_user_override",
    }
    if wait_state != expected_wait:
        fail(f"wait state mismatch: {wait_state!r}")

    pending = manifest.get("pending_ssot_landing_items", [])
    expected_pending = [
        (item, target, str(path.relative_to(ROOT)).replace("\\", "/"))
        for item, target, path in SOURCE_ITEMS
    ]
    actual_pending = [(row.get("item"), row.get("target_id"), row.get("request_path")) for row in pending]
    if actual_pending != expected_pending:
        fail(f"pending item mismatch: {actual_pending!r}")
    for index, row in enumerate(pending, start=1):
        if row.get("order") != index:
            fail(f"pending row order mismatch: {row!r}")
        if row.get("request_status") != "closed":
            fail(f"pending row request_status must be closed: {row!r}")
        if row.get("ssot_landed") is not False or row.get("product_followup_allowed") is not False:
            fail(f"pending row landed/followup flags must be false: {row!r}")
        require(ROOT / row["request_path"])

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
        "post_ssot_product_gate_open",
        "prime_notation_ssot_landed",
        "prime_derivative_semantics_landed",
        "flow_history_ssot_landed",
        "ddn_history_schema_landed",
        "tuck_ssot_landed",
        "tuck_row_parser_landed",
        "constraint_runtime_landed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")

    expected_plans = {
        "ssot_acceptance_request_queue": {"closed": 3, "total": 3, "percent": 100},
        "ssot_acceptance_request_closure_rebase": {"closed": 1, "total": 1, "percent": 100},
        "ssot_landing_wait_state_rebase": {"closed": 1, "total": 1, "percent": 100},
        "urgent_recommendations_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 73, "total": 90, "percent": 81},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    closure = load_json(SOURCE_CLOSURE)
    if closure.get("work_item") != "LANG_SSOT_ACCEPTANCE_REQUEST_CLOSURE_REBASE_V1":
        fail(f"closure work item mismatch: {closure.get('work_item')!r}")
    if closure.get("next_item") != WORK_ITEM:
        fail(f"closure next item expected {WORK_ITEM}, got {closure.get('next_item')!r}")
    if closure.get("urgent_ssot_landed_plan") != {"closed": 0, "total": 3, "percent": 0}:
        fail(f"closure urgent ssot landed progress mismatch: {closure.get('urgent_ssot_landed_plan')!r}")
    if closure.get("next_queue", {}).get("initial_progress") != {"closed": 0, "total": 3, "percent": 0}:
        fail(f"closure next queue progress mismatch: {closure.get('next_queue')!r}")

    for item, target, path in SOURCE_ITEMS:
        data = load_json(path)
        if data.get("work_item") != item:
            fail(f"{path.relative_to(ROOT)} work item expected {item}, got {data.get('work_item')!r}")
        if data.get("request", {}).get("target_id") != target:
            fail(f"{path.relative_to(ROOT)} target_id expected {target}, got {data.get('request', {}).get('target_id')!r}")
        if data.get("ssot_landed_claim") is not False:
            fail(f"{path.relative_to(ROOT)} ssot_landed_claim must remain false")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_ssot_landing_wait_state_rebase_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    require_contains(
        PACK / "golden.jsonl",
        [
            WORK_ITEM,
            "schema: ddn.language.ssot_landing_wait_state_rebase.v1",
            "closed acceptance requests: 3",
            "ssot request queue: 3/3 = 100%",
            "wait-state rebase: 1/1 = 100%",
            "urgent ssot landed: 0/3 = 0%",
            "roadmap: 73/90 = 81%",
            NEXT,
        ],
    )


def check_previous_checker() -> None:
    # The closure manifest is already checked above. Keep this boundary focused on
    # deterministic pack output instead of recursively replaying the full older
    # Rust-heavy checker chain on every wait-state run.
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_ssot_acceptance_request_closure_rebase_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"closure pack golden failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_golden()
    check_previous_checker()
    require_docs_ssot_clean()
    print("lang_ssot_landing_wait_state_rebase_check: PASS")


if __name__ == "__main__":
    main()
