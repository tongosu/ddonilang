from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_20260606.md"
PACK = ROOT / "pack" / "lang_owner_inner_seum_parser_boundary_spike_v1"
MANIFEST = PACK / "owner_inner_seum_parser_boundary_spike.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_owner_inner_seum_parser_boundary_spike_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
SOURCE_OWNER = ROOT / "pack" / "lang_owner_inner_seum_structure_check_v1" / "owner_inner_seum_structure_check.detjson"
SOURCE_REBASE = ROOT / "pack" / "lang_implementation_readiness_rebase_v1" / "implementation_readiness_rebase.detjson"
SOURCE_DULTRA = ROOT / "pack" / "lang_dultra_recorded_replay_contract_v1" / "dultra_recorded_replay_contract.detjson"
SOURCE_PRIME = ROOT / "pack" / "lang_prime_parser_frontdoor_spike_v1" / "prime_parser_frontdoor_spike.detjson"
NEXT = "LANG_IMPLEMENTATION_FOLLOWUP_CLOSURE_REBASE_V1"
CANON_ROW = "세움{\n    위치' =:= 속도.\n}"


def fail(message: str) -> None:
    print(f"lang_owner_inner_seum_parser_boundary_spike_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_OWNER,
        SOURCE_REBASE,
        SOURCE_DULTRA,
        SOURCE_PRIME,
        ROOT / "tests" / "run_lang_dultra_recorded_replay_contract_pack_check.py",
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1",
        "성질",
        "owner_inner_seum_canon_rows",
        "세움",
        "받으면",
        "No equation solver landed claim",
        "언어 구현 준비 후속 계획: 6/6 = 100%",
        "Owner inner seum parser boundary spike: 1/1 = 100%",
        "ROADMAP_V2 전체: queue-expanded 53/90 = 59%",
        NEXT,
    ]
    require_contains(DOC, tokens + ["docs/ssot/**", "No derivative semantics landed claim"])
    require_contains(PROPOSAL, tokens[:9] + ["No SSOT edit by Codex"])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "성질",
            "owner_inner_seum_canon_rows",
            "No equation solver landed claim",
            "No derivative semantics landed claim",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1",
            "ddn.language.owner_inner_seum_parser_boundary_spike.v1",
            "lang_owner_inner_seum_parser_boundary_spike_v1",
            "언어 구현 준비 후속 계획: 6/6 = 100%",
            "ROADMAP_V2 전체: queue-expanded 53/90 = 59%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_owner_inner_seum_parser_boundary_spike_v1",
        "kind": "lang_owner_inner_seum_parser_boundary_spike",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": True,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "owner_inner_seum_parser_boundary_landed_claim": True,
        "owner_inner_seum_runtime_landed_claim": False,
        "seongjil_owner_state_alias_landed_claim": True,
        "seongjil_global_block_landed_claim": False,
        "seum_equation_solver_landed_claim": False,
        "derivative_semantics_landed_claim": False,
        "closed_by": "LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1",
        "based_on": "LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_V1",
        "proposal_doc": "docs/context/proposals/LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_20260606.md",
        "decision_manifest": "pack/lang_owner_inner_seum_parser_boundary_spike_v1/owner_inner_seum_parser_boundary_spike.detjson",
        "implementation_readiness_followup_closed": 6,
        "implementation_readiness_followup_total": 6,
        "implementation_readiness_followup_percent": 100,
        "owner_inner_seum_parser_boundary_closed": 1,
        "owner_inner_seum_parser_boundary_total": 1,
        "owner_inner_seum_parser_boundary_percent": 100,
        "roadmap_v2_queue_expanded_closed": 53,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 59,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.owner_inner_seum_parser_boundary_spike.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1":
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    expected_flags = {
        "owner_inner_seum_parser_boundary_landed_claim": True,
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": True,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "owner_inner_seum_runtime_landed_claim": False,
        "seongjil_owner_state_alias_landed_claim": True,
        "seongjil_global_block_landed_claim": False,
        "seum_equation_solver_landed_claim": False,
        "derivative_semantics_landed_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")
    boundary = manifest.get("parser_boundary", {})
    expected_boundary = {
        "owner_seed_kind": "임자",
        "state_block_alias": "성질",
        "state_block_alias_scope": "임자_body_only",
        "state_block_canonical_path": "DeclBlock::Gureut",
        "relation_block": "세움",
        "relation_block_ast_path": "ExprKind::Assertion",
        "event_reaction_block": "받으면",
        "event_reaction_relation_row": False,
        "owner_inner_rows_api": "ddonirang_lang::owner_inner_seum_canon_rows",
    }
    for key, value in expected_boundary.items():
        if boundary.get(key) != value:
            fail(f"boundary {key} expected {value!r}, got {boundary.get(key)!r}")
    if manifest.get("owner_inner_seum_canon_rows") != [CANON_ROW]:
        fail(f"canon rows mismatch: {manifest.get('owner_inner_seum_canon_rows')!r}")
    expected_product_files = [
        "lang/src/parser.rs",
        "lang/src/frontdoor.rs",
        "lang/src/lib.rs",
        "tools/teul-cli/src/lang/lexer.rs",
        "tools/teul-cli/src/lang/parser.rs",
        "tools/teul-cli/src/cli/frontdoor_parse.rs",
    ]
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
        "runtime_surface_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "product_ui_change",
        "owner_inner_seum_runtime_landed",
        "seum_equation_solver_landed",
        "derivative_semantics_landed",
        "seongjil_global_block_landed",
        "receive_block_relation_row_landed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")
    expected_plans = {
        "implementation_readiness_followup_plan": {"closed": 6, "total": 6, "percent": 100},
        "owner_inner_seum_parser_boundary_plan": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 53, "total": 90, "percent": 59},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"{key} mismatch: {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    owner = load_json(SOURCE_OWNER)
    if owner.get("surface_seed", {}).get("owner") != "공:임자":
        fail("owner structure source mismatch")
    if owner.get("surface_seed", {}).get("parser_landed") is not False:
        fail("prior owner structure must have been non-parser-landed")
    rebase = load_json(SOURCE_REBASE)
    classifications = {row.get("id"): row for row in rebase.get("readiness_classifications", [])}
    owner_class = classifications.get("owner_inner_seum")
    if owner_class is None or owner_class.get("next") != "LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1":
        fail(f"readiness owner classification mismatch: {owner_class!r}")
    dultra = load_json(SOURCE_DULTRA)
    if dultra.get("next_item") != "LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1":
        fail(f"dultra next mismatch: {dultra.get('next_item')!r}")
    prime = load_json(SOURCE_PRIME)
    if prime.get("prime_parser_frontdoor_spike_claim") is not True:
        fail("prime parser frontdoor spike must be landed before owner-inner parser boundary")
    require_contains(SOURCE_OWNER, ["공:임자", "성질", "세움", "위치' =:= 속도.", "받으면"])
    require_contains(SOURCE_REBASE, ["owner_inner_seum", "parser_boundary_spike_ready_after_prime_scope"])
    require_contains(SOURCE_DULTRA, ["LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1"])
    require_contains(SOURCE_PRIME, ["prime_parser_frontdoor_spike_claim"])


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_owner_inner_seum_parser_boundary_spike_v1"], timeout=240)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_rust_tests() -> None:
    commands = [
        ["cargo", "test", "-p", "ddonirang-lang", "imja_owner_inner_seum", "--", "--nocapture"],
        ["cargo", "test", "--manifest-path", "tools/teul-cli/Cargo.toml", "parse_imja_owner_inner_seongjil_and_seum_boundary", "--", "--nocapture"],
        ["cargo", "test", "--manifest-path", "tools/teul-cli/Cargo.toml", "parse_runtime_accepts_imja_owner_inner_seongjil_and_seum_boundary", "--", "--nocapture"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=420)
        if proc.returncode != 0:
            fail(f"rust test failed ({' '.join(cmd)}):\n{proc.stdout}")


def check_previous_checker() -> None:
    proc = run([sys.executable, "tests/run_lang_dultra_recorded_replay_contract_pack_check.py"], timeout=420)
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
    print("lang_owner_inner_seum_parser_boundary_spike_check: PASS")


if __name__ == "__main__":
    main()
