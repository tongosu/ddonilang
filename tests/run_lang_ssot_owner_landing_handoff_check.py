from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_SSOT_OWNER_LANDING_HANDOFF_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_SSOT_OWNER_LANDING_HANDOFF_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_OWNER_LANDING_HANDOFF_20260608.md"
PACK = ROOT / "pack" / "lang_ssot_owner_landing_handoff_v1"
MANIFEST = PACK / "ssot_owner_landing_handoff.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_ssot_owner_landing_handoff_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_GATE = ROOT / "pack" / "lang_post_ssot_landing_product_gate_rebase_v1" / "post_ssot_landing_product_gate_rebase.detjson"
SOURCE_WAIT = ROOT / "pack" / "lang_ssot_landing_wait_state_rebase_v1" / "ssot_landing_wait_state_rebase.detjson"
GATE_CHECKER = ROOT / "tests" / "run_lang_post_ssot_landing_product_gate_rebase_check.py"

WORK_ITEM = "LANG_SSOT_OWNER_LANDING_HANDOFF_V1"
NEXT = "LANG_SSOT_OWNER_LANDING_AUDIT_REBASE_V1"
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
    print(f"lang_ssot_owner_landing_handoff_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_GATE,
        SOURCE_WAIT,
        GATE_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    require_contains(
        DOC,
        [
            WORK_ITEM,
            "Owner Landing Rows",
            "After Owner Landing",
            "SSOT owner landing rows prepared: `3/3 = 100%`",
            "긴급 언어 결정 SSOT 반영: `0/3 = 0%`",
            "ROADMAP_V2 전체: `queue-expanded 75/90 = 83%`",
            NEXT,
        ],
    )
    require_contains(PROPOSAL, [WORK_ITEM, "3/3 = 100%", "0/3 = 0%", "75/90 = 83%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "SSOT owner landing rows prepared is `3/3 = 100%`",
            "Urgent language decision SSOT landing remains `0/3 = 0%`",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.ssot_owner_landing_handoff.v1",
            "lang_ssot_owner_landing_handoff_v1",
            "SSOT owner landing handoff: 1/1 = 100%",
            "SSOT owner landing rows prepared: 3/3 = 100%",
            "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
            "ROADMAP_V2 전체: queue-expanded 75/90 = 83%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_ssot_owner_landing_handoff_v1",
        "kind": "lang_ssot_owner_landing_handoff",
        "ssot_owner_landing_handoff_claim": True,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_POST_SSOT_LANDING_PRODUCT_GATE_REBASE_V1",
        "proposal_doc": "docs/context/proposals/LANG_SSOT_OWNER_LANDING_HANDOFF_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_OWNER_LANDING_HANDOFF_20260608.md",
        "decision_manifest": "pack/lang_ssot_owner_landing_handoff_v1/ssot_owner_landing_handoff.detjson",
        "source_post_ssot_landing_product_gate_rebase": "pack/lang_post_ssot_landing_product_gate_rebase_v1/post_ssot_landing_product_gate_rebase.detjson",
        "source_ssot_landing_wait_state_rebase": "pack/lang_ssot_landing_wait_state_rebase_v1/ssot_landing_wait_state_rebase.detjson",
        "ssot_owner_landing_handoff_closed": 1,
        "ssot_owner_landing_handoff_total": 1,
        "ssot_owner_landing_handoff_percent": 100,
        "ssot_owner_landing_rows_prepared_closed": 3,
        "ssot_owner_landing_rows_prepared_total": 3,
        "ssot_owner_landing_rows_prepared_percent": 100,
        "post_ssot_product_gates_open_closed": 0,
        "post_ssot_product_gates_open_total": 3,
        "post_ssot_product_gates_open_percent": 0,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 75,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 83,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    for key in FALSE_FLAGS:
        if contract.get(key) is not False:
            fail(f"contract {key} must be false, got {contract.get(key)!r}")
    for source_key in ["source_post_ssot_landing_product_gate_rebase", "source_ssot_landing_wait_state_rebase"]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.ssot_owner_landing_handoff.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    if manifest.get("ssot_owner_landing_handoff_claim") is not True:
        fail("owner handoff claim must be true")
    for key in FALSE_FLAGS:
        if manifest.get(key) is not False:
            fail(f"manifest {key} must be false, got {manifest.get(key)!r}")

    policy = manifest.get("handoff_policy", {})
    expected_policy = {
        "id": "urgent_language_ssot_owner_landing_handoff",
        "codex_ssot_edit_allowed": False,
        "owner_action_required": True,
        "post_owner_audit_required": True,
        "product_followup_gate_before_owner_landing": "closed",
    }
    if policy != expected_policy:
        fail(f"handoff policy mismatch: {policy!r}")

    rows = manifest.get("owner_landing_rows", [])
    if [row.get("target_id") for row in rows] != TARGETS:
        fail(f"owner landing targets mismatch: {rows!r}")
    for index, row in enumerate(rows, start=1):
        if row.get("order") != index:
            fail(f"owner row order mismatch: {row!r}")
        if row.get("codex_claim") != "none" or row.get("ssot_landed_by_codex") is not False:
            fail(f"owner row Codex claim must stay empty: {row!r}")
        require(ROOT / row["source_request"])

    expected_audit = [
        "verify_owner_changed_docs_ssot",
        "verify_three_ssot_landed_sources",
        "recompute_post_ssot_product_gates",
        "keep_product_implementation_separate",
    ]
    if manifest.get("post_owner_audit_requirements") != expected_audit:
        fail(f"post owner audit requirements mismatch: {manifest.get('post_owner_audit_requirements')!r}")

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
        "ssot_owner_landing_handoff": {"closed": 1, "total": 1, "percent": 100},
        "ssot_owner_landing_rows_prepared": {"closed": 3, "total": 3, "percent": 100},
        "post_ssot_landing_product_gate_rebase": {"closed": 1, "total": 1, "percent": 100},
        "post_ssot_product_gates_open": {"closed": 0, "total": 3, "percent": 0},
        "urgent_recommendations_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 75, "total": 90, "percent": 83},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    gate = load_json(SOURCE_GATE)
    if gate.get("work_item") != "LANG_POST_SSOT_LANDING_PRODUCT_GATE_REBASE_V1":
        fail(f"gate source work item mismatch: {gate.get('work_item')!r}")
    if gate.get("next_item") != WORK_ITEM:
        fail(f"gate source next expected {WORK_ITEM}, got {gate.get('next_item')!r}")
    if gate.get("post_ssot_product_gates_open") != {"closed": 0, "total": 3, "percent": 0}:
        fail(f"gate source product gates progress mismatch: {gate.get('post_ssot_product_gates_open')!r}")
    if [row.get("target_id") for row in gate.get("product_gate_rows", [])] != TARGETS:
        fail(f"gate source targets mismatch: {gate.get('product_gate_rows')!r}")
    for row in gate.get("product_gate_rows", []):
        if row.get("status") != "blocked" or row.get("product_followup_allowed") is not False:
            fail(f"gate source row must remain blocked: {row!r}")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_ssot_owner_landing_handoff_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    require_contains(
        PACK / "golden.jsonl",
        [
            WORK_ITEM,
            "schema: ddn.language.ssot_owner_landing_handoff.v1",
            "owner landing rows prepared: 3/3 = 100%",
            "urgent ssot landed: 0/3 = 0%",
            "post ssot product gates open: 0/3 = 0%",
            "roadmap: 75/90 = 83%",
            NEXT,
        ],
    )


def check_previous_checker() -> None:
    proc = run([sys.executable, str(GATE_CHECKER.relative_to(ROOT))], timeout=1200)
    if proc.returncode != 0:
        fail(f"{GATE_CHECKER.relative_to(ROOT)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_golden()
    check_previous_checker()
    require_docs_ssot_clean()
    print("lang_ssot_owner_landing_handoff_check: PASS")


if __name__ == "__main__":
    main()

