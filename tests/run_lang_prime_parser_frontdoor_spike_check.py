from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_PRIME_PARSER_FRONTDOOR_SPIKE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_PRIME_PARSER_FRONTDOOR_SPIKE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_PRIME_PARSER_FRONTDOOR_SPIKE_20260606.md"
PACK = ROOT / "pack" / "lang_prime_parser_frontdoor_spike_v1"
MANIFEST = PACK / "prime_parser_frontdoor_spike.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_prime_parser_frontdoor_spike_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
SOURCE_REBASE = ROOT / "pack" / "lang_implementation_readiness_rebase_v1" / "implementation_readiness_rebase.detjson"
SOURCE_PRIME = ROOT / "pack" / "lang_prime_derivative_notation_decision_v1" / "prime_derivative_notation_decision.detjson"
NEXT = "LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1"

PRODUCT_FILES = [
    ROOT / "lang" / "src" / "lexer.rs",
    ROOT / "lang" / "src" / "parser.rs",
    ROOT / "lang" / "src" / "lib.rs",
    ROOT / "tools" / "teul-cli" / "src" / "lang" / "lexer.rs",
    ROOT / "tools" / "teul-cli" / "src" / "cli" / "frontdoor_parse.rs",
]


def fail(message: str) -> None:
    print(f"lang_prime_parser_frontdoor_spike_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_REBASE,
        SOURCE_PRIME,
        ROOT / "tests" / "run_lang_implementation_readiness_rebase_check.py",
    ] + PRODUCT_FILES:
        require(path)


def check_docs() -> None:
    tokens = [
        "LANG_PRIME_PARSER_FRONTDOOR_SPIKE_V1",
        "위치'",
        "위치''",
        "위치'''",
        "lang/src/lexer.rs",
        "tools/teul-cli/src/lang/lexer.rs",
        "parser/frontdoor",
        "No derivative math semantics",
        "언어 구현 준비 후속 계획: 2/6 = 33%",
        "Prime parser/frontdoor spike: 1/1 = 100%",
        "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
        "ROADMAP_V2 전체: queue-expanded 49/90 = 54%",
        NEXT,
    ]
    require_contains(DOC, tokens + ["docs/ssot/**"])
    require_contains(PROPOSAL, tokens[:8] + ["No SSOT edit by Codex"])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "`lang` lexer/parser/normalizer accepts",
            "`teul-cli` runtime frontdoor lexer accepts",
            "No derivative semantics",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "LANG_PRIME_PARSER_FRONTDOOR_SPIKE_V1",
            "ddn.language.prime_parser_frontdoor_spike.v1",
            "lang_prime_parser_frontdoor_spike_v1",
            "언어 구현 준비 후속 계획: 2/6 = 33%",
            "ROADMAP_V2 전체: queue-expanded 49/90 = 54%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_prime_parser_frontdoor_spike_v1",
        "kind": "lang_prime_parser_frontdoor_spike",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": True,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "prime_parser_frontdoor_spike_claim": True,
        "prime_identifier_parser_acceptance_landed_claim": True,
        "prime_derivative_semantics_landed_claim": False,
        "seum_equation_solver_landed_claim": False,
        "owner_inner_seum_landed_claim": False,
        "closed_by": "LANG_PRIME_PARSER_FRONTDOOR_SPIKE_V1",
        "based_on": "LANG_IMPLEMENTATION_READINESS_REBASE_V1",
        "proposal_doc": "docs/context/proposals/LANG_PRIME_PARSER_FRONTDOOR_SPIKE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_PRIME_PARSER_FRONTDOOR_SPIKE_20260606.md",
        "decision_manifest": "pack/lang_prime_parser_frontdoor_spike_v1/prime_parser_frontdoor_spike.detjson",
        "source_readiness_rebase": "pack/lang_implementation_readiness_rebase_v1/implementation_readiness_rebase.detjson",
        "source_prime_decision": "pack/lang_prime_derivative_notation_decision_v1/prime_derivative_notation_decision.detjson",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "language_design_queue_closed": 8,
        "language_design_queue_total": 8,
        "language_design_queue_percent": 100,
        "implementation_readiness_rebase_closed": 1,
        "implementation_readiness_rebase_total": 1,
        "implementation_readiness_rebase_percent": 100,
        "implementation_readiness_followup_closed": 2,
        "implementation_readiness_followup_total": 6,
        "implementation_readiness_followup_percent": 33,
        "prime_parser_frontdoor_spike_closed": 1,
        "prime_parser_frontdoor_spike_total": 1,
        "prime_parser_frontdoor_spike_percent": 100,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 49,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 54,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.prime_parser_frontdoor_spike.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "LANG_PRIME_PARSER_FRONTDOOR_SPIKE_V1":
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    expected_flags = {
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": True,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "prime_identifier_parser_acceptance_landed_claim": True,
        "prime_derivative_semantics_landed_claim": False,
        "seum_equation_solver_landed_claim": False,
        "owner_inner_seum_landed_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")
    accepted = manifest.get("accepted_surfaces", [])
    if [(row.get("surface"), row.get("derivative_order")) for row in accepted] != [("위치'", 1), ("위치''", 2)]:
        fail(f"accepted surfaces mismatch: {accepted!r}")
    for row in accepted:
        for key in ["accepted_by_lang_lexer", "accepted_by_lang_parser", "accepted_by_teul_cli_frontdoor"]:
            if row.get(key) is not True:
                fail(f"accepted row {key} must be true: {row!r}")
        if row.get("semantics_landed") is not False:
            fail(f"semantics must not be landed: {row!r}")
    rejected = [row.get("surface") for row in manifest.get("rejected_surfaces", [])]
    if rejected != ["위치'''", "'위치", "위'치"]:
        fail(f"rejected surfaces mismatch: {rejected!r}")
    expected_product_files = [str(path.relative_to(ROOT)).replace("\\", "/") for path in PRODUCT_FILES]
    if manifest.get("product_files") != expected_product_files:
        fail(f"product files mismatch: {manifest.get('product_files')!r}")
    for row in manifest.get("evidence_rows", []):
        path = ROOT / row.get("path", "")
        require(path)
        require_contains(path, row.get("tokens", []))
        if row.get("product_path") is not True:
            fail(f"evidence row must be product path: {row!r}")
    required_blocked = {
        "docs_ssot_edit",
        "derivative_semantics_landed",
        "seum_equation_solver_landed",
        "owner_inner_seum_landed",
        "runtime_integrator_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "product_ui_change",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")
    expected_plans = {
        "implementation_readiness_followup_plan": {"closed": 2, "total": 6, "percent": 33},
        "prime_parser_frontdoor_spike_plan": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 49, "total": 90, "percent": 54},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"{key} mismatch: {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    rebase = load_json(SOURCE_REBASE)
    if rebase.get("next_item") != "LANG_PRIME_PARSER_FRONTDOOR_SPIKE_V1":
        fail(f"readiness rebase next mismatch: {rebase.get('next_item')!r}")
    if rebase.get("implementation_readiness_followup_plan") != {"closed": 1, "total": 6, "percent": 17}:
        fail(f"readiness rebase followup plan mismatch: {rebase.get('implementation_readiness_followup_plan')!r}")
    prime = load_json(SOURCE_PRIME)
    selected = prime.get("selected_notations", [])
    if [(row.get("surface"), row.get("derivative_order")) for row in selected] != [("위치'", 1), ("위치''", 2)]:
        fail(f"source prime selected notation mismatch: {selected!r}")


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_prime_parser_frontdoor_spike_v1"], timeout=240)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_rust_tests() -> None:
    commands = [
        ["cargo", "test", "-p", "ddonirang-lang", "prime_derivative", "--", "--nocapture"],
        ["cargo", "test", "--manifest-path", "tools/teul-cli/Cargo.toml", "prime_derivative", "--", "--nocapture"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=300)
        if proc.returncode != 0:
            fail(f"rust test failed ({' '.join(cmd)}):\n{proc.stdout}")


def check_previous_checker() -> None:
    proc = run([sys.executable, "tests/run_lang_implementation_readiness_rebase_check.py"], timeout=240)
    if proc.returncode != 0:
        fail(f"previous checker failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_rust_tests()
    check_pack_golden()
    check_previous_checker()
    require_docs_ssot_clean()
    print("lang_prime_parser_frontdoor_spike_check: PASS")


if __name__ == "__main__":
    main()
