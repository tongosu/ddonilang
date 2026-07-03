from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_POST_SSOT_LANDING_PRODUCT_GATE_REBASE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_POST_SSOT_LANDING_PRODUCT_GATE_REBASE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_POST_LANDING_PRODUCT_GATE_REBASE_20260607.md"
PACK = ROOT / "pack" / "lang_post_ssot_landing_product_gate_rebase_v1"
MANIFEST = PACK / "post_ssot_landing_product_gate_rebase.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_post_ssot_landing_product_gate_rebase_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_WAIT = ROOT / "pack" / "lang_ssot_landing_wait_state_rebase_v1" / "ssot_landing_wait_state_rebase.detjson"
SOURCE_CLOSURE = ROOT / "pack" / "lang_ssot_acceptance_request_closure_rebase_v1" / "ssot_acceptance_request_closure_rebase.detjson"
WAIT_CHECKER = ROOT / "tests" / "run_lang_ssot_landing_wait_state_rebase_check.py"

WORK_ITEM = "LANG_POST_SSOT_LANDING_PRODUCT_GATE_REBASE_V1"
NEXT = "LANG_SSOT_OWNER_LANDING_HANDOFF_V1"
TARGETS = ["prime_derivative_notation", "flow_history_naming_split", "tuck_constraint_layer"]
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
]


def fail(message: str) -> None:
    print(f"lang_post_ssot_landing_product_gate_rebase_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_WAIT,
        SOURCE_CLOSURE,
        WAIT_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    require_contains(
        DOC,
        [
            WORK_ITEM,
            "The gate remains closed",
            "Post-SSOT product gates open: `0/3 = 0%`",
            "No `docs/ssot/**` edit",
            "No post-SSOT product gate open claim",
            "ROADMAP_V2 전체: `queue-expanded 74/90 = 82%`",
            NEXT,
        ],
    )
    require_contains(PROPOSAL, [WORK_ITEM, "0/3 = 0%", "74/90 = 82%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "product gate: blocked",
            "Post-SSOT product gates open remains `0/3 = 0%`",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.post_ssot_landing_product_gate_rebase.v1",
            "lang_post_ssot_landing_product_gate_rebase_v1",
            "Post-SSOT product gate rebase: 1/1 = 100%",
            "Post-SSOT product gates open: 0/3 = 0%",
            "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
            "ROADMAP_V2 전체: queue-expanded 74/90 = 82%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_post_ssot_landing_product_gate_rebase_v1",
        "kind": "lang_post_ssot_landing_product_gate_rebase",
        "post_ssot_landing_product_gate_rebase_claim": True,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_SSOT_LANDING_WAIT_STATE_REBASE_V1",
        "proposal_doc": "docs/context/proposals/LANG_POST_SSOT_LANDING_PRODUCT_GATE_REBASE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_POST_LANDING_PRODUCT_GATE_REBASE_20260607.md",
        "decision_manifest": "pack/lang_post_ssot_landing_product_gate_rebase_v1/post_ssot_landing_product_gate_rebase.detjson",
        "source_ssot_landing_wait_state_rebase": "pack/lang_ssot_landing_wait_state_rebase_v1/ssot_landing_wait_state_rebase.detjson",
        "source_ssot_acceptance_request_closure_rebase": "pack/lang_ssot_acceptance_request_closure_rebase_v1/ssot_acceptance_request_closure_rebase.detjson",
        "post_ssot_landing_product_gate_rebase_closed": 1,
        "post_ssot_landing_product_gate_rebase_total": 1,
        "post_ssot_landing_product_gate_rebase_percent": 100,
        "post_ssot_product_gates_open_closed": 0,
        "post_ssot_product_gates_open_total": 3,
        "post_ssot_product_gates_open_percent": 0,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 74,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 82,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    for key in FALSE_FLAGS:
        if contract.get(key) is not False:
            fail(f"contract {key} must be false, got {contract.get(key)!r}")
    for source_key in ["source_ssot_landing_wait_state_rebase", "source_ssot_acceptance_request_closure_rebase"]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.post_ssot_landing_product_gate_rebase.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    if manifest.get("post_ssot_landing_product_gate_rebase_claim") is not True:
        fail("product gate rebase claim must be true")
    for key in FALSE_FLAGS:
        if manifest.get(key) is not False:
            fail(f"manifest {key} must be false, got {manifest.get(key)!r}")

    gate_policy = manifest.get("gate_policy", {})
    expected_policy = {
        "id": "post_ssot_language_product_gate",
        "status": "closed",
        "open_condition": "ssot_landed_or_explicit_user_override",
        "reason": "urgent SSOT landing remains 0/3",
        "codex_product_followup_allowed": False,
    }
    if gate_policy != expected_policy:
        fail(f"gate policy mismatch: {gate_policy!r}")

    rows = manifest.get("product_gate_rows", [])
    if [row.get("target_id") for row in rows] != TARGETS:
        fail(f"gate row targets mismatch: {rows!r}")
    for index, row in enumerate(rows, start=1):
        if row.get("order") != index or row.get("status") != "blocked":
            fail(f"gate row malformed: {row!r}")
        if row.get("reason") != "SSOT landed false":
            fail(f"gate row reason mismatch: {row!r}")
        if row.get("ssot_landed") is not False or row.get("product_followup_allowed") is not False:
            fail(f"gate row landed/followup flags must be false: {row!r}")

    required_blocked = {
        "docs_ssot_edit",
        "ssot_landed",
        "post_ssot_product_gate_open",
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
        "ssot_acceptance_request_queue": {"closed": 3, "total": 3, "percent": 100},
        "ssot_landing_wait_state_rebase": {"closed": 1, "total": 1, "percent": 100},
        "post_ssot_landing_product_gate_rebase": {"closed": 1, "total": 1, "percent": 100},
        "post_ssot_product_gates_open": {"closed": 0, "total": 3, "percent": 0},
        "urgent_recommendations_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 74, "total": 90, "percent": 82},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    wait = load_json(SOURCE_WAIT)
    if wait.get("work_item") != "LANG_SSOT_LANDING_WAIT_STATE_REBASE_V1":
        fail(f"wait source work item mismatch: {wait.get('work_item')!r}")
    if wait.get("next_item") != WORK_ITEM:
        fail(f"wait source next expected {WORK_ITEM}, got {wait.get('next_item')!r}")
    if wait.get("wait_state", {}).get("product_followup_gate") != "closed_until_ssot_landed_or_explicit_user_override":
        fail(f"wait source gate mismatch: {wait.get('wait_state')!r}")
    if wait.get("urgent_ssot_landed_plan") != {"closed": 0, "total": 3, "percent": 0}:
        fail(f"wait source urgent ssot landed mismatch: {wait.get('urgent_ssot_landed_plan')!r}")
    pending = wait.get("pending_ssot_landing_items", [])
    if [row.get("target_id") for row in pending] != TARGETS:
        fail(f"wait source pending targets mismatch: {pending!r}")
    for row in pending:
        if row.get("ssot_landed") is not False or row.get("product_followup_allowed") is not False:
            fail(f"wait source pending row must remain blocked: {row!r}")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_post_ssot_landing_product_gate_rebase_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    require_contains(
        PACK / "golden.jsonl",
        [
            WORK_ITEM,
            "schema: ddn.language.post_ssot_landing_product_gate_rebase.v1",
            "product gate rows: 3",
            "post ssot product gates open: 0/3 = 0%",
            "urgent ssot landed: 0/3 = 0%",
            "roadmap: 74/90 = 82%",
            NEXT,
        ],
    )


def check_previous_checker() -> None:
    proc = run([sys.executable, str(WAIT_CHECKER.relative_to(ROOT))], timeout=1200)
    if proc.returncode != 0:
        fail(f"{WAIT_CHECKER.relative_to(ROOT)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_golden()
    check_previous_checker()
    require_docs_ssot_clean()
    print("lang_post_ssot_landing_product_gate_rebase_check: PASS")


if __name__ == "__main__":
    main()

