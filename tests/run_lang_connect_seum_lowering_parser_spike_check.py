from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_20260606.md"
PACK = ROOT / "pack" / "lang_connect_seum_lowering_parser_spike_v1"
MANIFEST = PACK / "connect_seum_lowering_parser_spike.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_connect_seum_lowering_parser_spike_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
SOURCE_REBASE = ROOT / "pack" / "lang_implementation_readiness_rebase_v1" / "implementation_readiness_rebase.detjson"
SOURCE_CONNECT = ROOT / "pack" / "lang_connect_lowering_to_seum_check_v1" / "connect_lowering_to_seum_check.detjson"
SOURCE_PRIME = ROOT / "pack" / "lang_prime_parser_frontdoor_spike_v1" / "prime_parser_frontdoor_spike.detjson"
NEXT = "LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_V1"

PRODUCT_FILES = [
    ROOT / "lang" / "src" / "frontdoor.rs",
    ROOT / "lang" / "src" / "lib.rs",
]

SEUM_ROWS = [
    "전지.양극.전압 =:= 전구.왼핀.전압.",
    "전지.양극.전류 + 전구.왼핀.전류 =:= 0.",
]


def fail(message: str) -> None:
    print(f"lang_connect_seum_lowering_parser_spike_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_CONNECT,
        SOURCE_PRIME,
        ROOT / "tests" / "run_lang_prime_parser_frontdoor_spike_check.py",
    ] + PRODUCT_FILES:
        require(path)


def check_docs() -> None:
    tokens = [
        "LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1",
        "connect_endpoint_relation_seum_rows",
        "전지.양극과 전구.왼핀을 (전압은 같게, 전류는 흐르게) 잇기.",
        *SEUM_ROWS,
        "잇기 {}",
        "No `세움 {}` equation solver landed claim",
        "언어 구현 준비 후속 계획: 3/6 = 50%",
        "Connect seum lowering parser spike: 1/1 = 100%",
        "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
        "ROADMAP_V2 전체: queue-expanded 50/90 = 56%",
        NEXT,
    ]
    require_contains(DOC, tokens + ["docs/ssot/**"])
    require_contains(PROPOSAL, tokens[:7] + ["No SSOT edit by Codex"])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "connect_endpoint_relation_seum_rows",
            *SEUM_ROWS,
            "No `잇기 {}` block",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1",
            "ddn.language.connect_seum_lowering_parser_spike.v1",
            "lang_connect_seum_lowering_parser_spike_v1",
            "언어 구현 준비 후속 계획: 3/6 = 50%",
            "ROADMAP_V2 전체: queue-expanded 50/90 = 56%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_connect_seum_lowering_parser_spike_v1",
        "kind": "lang_connect_seum_lowering_parser_spike",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": True,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "connect_seum_lowering_parser_spike_claim": True,
        "connect_seum_rows_api_landed_claim": True,
        "connect_block_landed_claim": False,
        "seum_equation_solver_landed_claim": False,
        "connect_runtime_change_claim": False,
        "closed_by": "LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1",
        "based_on": "LANG_PRIME_PARSER_FRONTDOOR_SPIKE_V1",
        "proposal_doc": "docs/context/proposals/LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_20260606.md",
        "decision_manifest": "pack/lang_connect_seum_lowering_parser_spike_v1/connect_seum_lowering_parser_spike.detjson",
        "source_readiness_rebase": "pack/lang_implementation_readiness_rebase_v1/implementation_readiness_rebase.detjson",
        "source_connect_design": "pack/lang_connect_lowering_to_seum_check_v1/connect_lowering_to_seum_check.detjson",
        "source_prime_parser_spike": "pack/lang_prime_parser_frontdoor_spike_v1/prime_parser_frontdoor_spike.detjson",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "language_design_queue_closed": 8,
        "language_design_queue_total": 8,
        "language_design_queue_percent": 100,
        "implementation_readiness_rebase_closed": 1,
        "implementation_readiness_rebase_total": 1,
        "implementation_readiness_rebase_percent": 100,
        "implementation_readiness_followup_closed": 3,
        "implementation_readiness_followup_total": 6,
        "implementation_readiness_followup_percent": 50,
        "connect_seum_lowering_parser_spike_closed": 1,
        "connect_seum_lowering_parser_spike_total": 1,
        "connect_seum_lowering_parser_spike_percent": 100,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 50,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 56,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.connect_seum_lowering_parser_spike.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1":
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
        "connect_seum_rows_api_landed_claim": True,
        "connect_block_landed_claim": False,
        "seum_equation_solver_landed_claim": False,
        "connect_runtime_change_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("landed_api") != "ddonirang_lang::connect_endpoint_relation_seum_rows":
        fail(f"landed API mismatch: {manifest.get('landed_api')!r}")
    if manifest.get("seum_rows") != SEUM_ROWS:
        fail(f"seum rows mismatch: {manifest.get('seum_rows')!r}")
    accepted = [row.get("kind") for row in manifest.get("accepted_surfaces", [])]
    if accepted != ["unassigned_sentence_connect", "assigned_sentence_connect"]:
        fail(f"accepted surface kinds mismatch: {accepted!r}")
    rejected = [row.get("surface") for row in manifest.get("rejected_surfaces", [])]
    if rejected != ["잇기 { 전압은 같게. }.", "전지.양극과 전구.왼핀을 (재화가 돈에 실리게) 잇기."]:
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
        "connect_block_landed",
        "seum_equation_solver_landed",
        "derivative_semantics_landed",
        "runtime_integrator_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "product_ui_change",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")
    expected_plans = {
        "implementation_readiness_followup_plan": {"closed": 3, "total": 6, "percent": 50},
        "connect_seum_lowering_parser_spike_plan": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 50, "total": 90, "percent": 56},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"{key} mismatch: {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    rebase = load_json(SOURCE_REBASE)
    classifications = {row.get("id"): row for row in rebase.get("readiness_classifications", [])}
    connect = classifications.get("connect_lowering_to_seum")
    if connect is None or connect.get("next") != "LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1":
        fail(f"readiness connect classification mismatch: {connect!r}")
    prime = load_json(SOURCE_PRIME)
    if prime.get("next_item") != "LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1":
        fail(f"prime parser spike next mismatch: {prime.get('next_item')!r}")
    source_connect = load_json(SOURCE_CONNECT)
    if source_connect.get("surface_seed", {}).get("connect_is_new_block") is not False:
        fail("source connect design must reject connect block")
    lowered = [row.get("lowered_row") for row in source_connect.get("lowering_rows", [])]
    if lowered != SEUM_ROWS:
        fail(f"source connect lowered rows mismatch: {lowered!r}")


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_connect_seum_lowering_parser_spike_v1"], timeout=240)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_rust_tests() -> None:
    commands = [
        ["cargo", "test", "-p", "ddonirang-lang", "connect_endpoint_", "--", "--nocapture"],
        ["cargo", "test", "-p", "ddonirang-lang", "connect_block_surface_has_no_seum_rows", "--", "--nocapture"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=300)
        if proc.returncode != 0:
            fail(f"rust test failed ({' '.join(cmd)}):\n{proc.stdout}")


def check_previous_checker() -> None:
    proc = run([sys.executable, "tests/run_lang_prime_parser_frontdoor_spike_check.py"], timeout=300)
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
    print("lang_connect_seum_lowering_parser_spike_check: PASS")


if __name__ == "__main__":
    main()
