from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_PRIME_SSOT_ACCEPTANCE_REQUEST_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_PRIME_SSOT_ACCEPTANCE_REQUEST_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_PRIME_ACCEPTANCE_REQUEST_20260606.md"
PACK = ROOT / "pack" / "lang_prime_ssot_acceptance_request_v1"
MANIFEST = PACK / "prime_ssot_acceptance_request.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_prime_ssot_acceptance_request_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_COORDINATION = ROOT / "pack" / "lang_ssot_landing_coordination_rebase_v1" / "ssot_landing_coordination_rebase.detjson"
SOURCE_DECISION = ROOT / "pack" / "lang_prime_derivative_notation_decision_v1" / "prime_derivative_notation_decision.detjson"
SOURCE_PARSER = ROOT / "pack" / "lang_prime_parser_frontdoor_spike_v1" / "prime_parser_frontdoor_spike.detjson"
SOURCE_GATE = ROOT / "pack" / "lang_prime_derivative_runtime_semantics_gate_v1" / "prime_derivative_runtime_semantics_gate.detjson"
PREVIOUS_CHECKER = ROOT / "tests" / "run_lang_ssot_landing_coordination_rebase_check.py"

WORK_ITEM = "LANG_PRIME_SSOT_ACCEPTANCE_REQUEST_V1"
NEXT = "LANG_FLOW_HISTORY_SSOT_ACCEPTANCE_REQUEST_V1"
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
    "prime_notation_ssot_landed_claim",
    "prime_derivative_semantics_landed_claim",
    "seum_equation_solver_landed_claim",
    "owner_inner_seum_runtime_landed_claim",
]


def fail(message: str) -> None:
    print(f"lang_prime_ssot_acceptance_request_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_COORDINATION,
        SOURCE_DECISION,
        SOURCE_PARSER,
        SOURCE_GATE,
        PREVIOUS_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        "위치'",
        "위치''",
        "Recommended SSOT Text",
        "No `docs/ssot/**` edit",
        "No SSOT landed claim",
        "SSOT acceptance request queue: `1/3 = 33%`",
        "Prime SSOT acceptance request: `1/1 = 100%`",
        "긴급 언어 결정 SSOT 반영: `0/3 = 0%`",
        "ROADMAP_V2 전체: `queue-expanded 69/90 = 77%`",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "1/3 = 33%", "0/3 = 0%", "69/90 = 77%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "`이름'`: first derivative",
            "`이름''`: second derivative",
            "No SSOT landed claim by Codex",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.prime_ssot_acceptance_request.v1",
            "lang_prime_ssot_acceptance_request_v1",
            "SSOT acceptance request queue: 1/3 = 33%",
            "Prime SSOT acceptance request: 1/1 = 100%",
            "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
            "ROADMAP_V2 전체: queue-expanded 69/90 = 77%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_prime_ssot_acceptance_request_v1",
        "kind": "lang_prime_ssot_acceptance_request",
        "prime_ssot_acceptance_request_claim": True,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_SSOT_LANDING_COORDINATION_REBASE_V1",
        "proposal_doc": "docs/context/proposals/LANG_PRIME_SSOT_ACCEPTANCE_REQUEST_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_PRIME_ACCEPTANCE_REQUEST_20260606.md",
        "decision_manifest": "pack/lang_prime_ssot_acceptance_request_v1/prime_ssot_acceptance_request.detjson",
        "source_ssot_landing_coordination": "pack/lang_ssot_landing_coordination_rebase_v1/ssot_landing_coordination_rebase.detjson",
        "source_prime_derivative_notation_decision": "pack/lang_prime_derivative_notation_decision_v1/prime_derivative_notation_decision.detjson",
        "source_prime_parser_frontdoor_spike": "pack/lang_prime_parser_frontdoor_spike_v1/prime_parser_frontdoor_spike.detjson",
        "source_prime_derivative_runtime_semantics_gate": "pack/lang_prime_derivative_runtime_semantics_gate_v1/prime_derivative_runtime_semantics_gate.detjson",
        "ssot_acceptance_request_queue_closed": 1,
        "ssot_acceptance_request_queue_total": 3,
        "ssot_acceptance_request_queue_percent": 33,
        "prime_ssot_acceptance_request_closed": 1,
        "prime_ssot_acceptance_request_total": 1,
        "prime_ssot_acceptance_request_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 69,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 77,
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
        "source_ssot_landing_coordination",
        "source_prime_derivative_notation_decision",
        "source_prime_parser_frontdoor_spike",
        "source_prime_derivative_runtime_semantics_gate",
    ]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.prime_ssot_acceptance_request.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    if manifest.get("prime_ssot_acceptance_request_claim") is not True:
        fail("request claim must be true")
    for key in FALSE_FLAGS:
        if manifest.get(key) is not False:
            fail(f"manifest {key} must be false, got {manifest.get(key)!r}")

    request = manifest.get("request", {})
    surfaces = request.get("surfaces", [])
    if surfaces != [
        {"surface": "위치'", "derivative_order": 1},
        {"surface": "위치''", "derivative_order": 2},
    ]:
        fail(f"request surfaces mismatch: {surfaces!r}")
    limits = request.get("current_line_limits", {})
    expected_limits = {
        "max_prime_suffixes": 2,
        "runtime_semantics_landed": False,
        "equation_solver_landed": False,
        "owner_inner_seum_runtime_landed": False,
    }
    if limits != expected_limits:
        fail(f"limits mismatch: {limits!r}")
    for token in ["`이름'`", "`이름''`", "runtime derivative semantics require separate runtime evidence"]:
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
        "runtime_surface_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "product_ui_change",
        "prime_notation_ssot_landed",
        "prime_derivative_semantics_landed",
        "seum_equation_solver_landed",
        "owner_inner_seum_runtime_landed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")

    expected_plans = {
        "ssot_acceptance_request_queue": {"closed": 1, "total": 3, "percent": 33},
        "prime_ssot_acceptance_request": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 69, "total": 90, "percent": 77},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    coordination = load_json(SOURCE_COORDINATION)
    target = coordination.get("coordination_targets", [])[0]
    if target.get("id") != "prime_derivative_notation" or target.get("next_item") != WORK_ITEM:
        fail(f"coordination first target mismatch: {target!r}")
    if target.get("ssot_landed") is not False:
        fail("coordination target ssot_landed must be false")

    decision = load_json(SOURCE_DECISION)
    if [row.get("surface") for row in decision.get("selected_notations", [])] != ["위치'", "위치''"]:
        fail(f"decision selected surfaces mismatch: {decision.get('selected_notations')!r}")
    for row in decision.get("selected_notations", []):
        if row.get("ssot_landed") is not False or row.get("runtime_landed") is not False:
            fail(f"decision row landed flags must be false: {row!r}")

    parser = load_json(SOURCE_PARSER)
    if parser.get("prime_identifier_parser_acceptance_landed_claim") is not True:
        fail("parser frontdoor evidence should remain landed")
    if parser.get("prime_derivative_semantics_landed_claim") is not False:
        fail("parser source derivative semantics must remain false")

    gate = load_json(SOURCE_GATE)
    if gate.get("derivative_semantics_landed_claim") is not False:
        fail("runtime semantics gate landed claim must remain false")
    if gate.get("next_item") != "LANG_OWNER_STATE_SYMBOL_TABLE_PRODUCT_PATH_V1":
        fail(f"runtime gate next mismatch: {gate.get('next_item')!r}")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_prime_ssot_acceptance_request_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    require_contains(
        PACK / "golden.jsonl",
        [
            WORK_ITEM,
            "schema: ddn.language.prime_ssot_acceptance_request.v1",
            "surfaces: 위치', 위치''",
            "ssot request queue: 1/3 = 33%",
            "urgent ssot landed: 0/3 = 0%",
            "roadmap: 69/90 = 77%",
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
    print("lang_prime_ssot_acceptance_request_check: PASS")


if __name__ == "__main__":
    main()

