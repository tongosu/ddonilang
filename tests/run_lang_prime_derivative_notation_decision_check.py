from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_PRIME_DERIVATIVE_NOTATION_DECISION_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_PRIME_DERIVATIVE_NOTATION_DECISION_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_PRIME_DERIVATIVE_NOTATION_20260606.md"
PACK = ROOT / "pack" / "lang_prime_derivative_notation_decision_v1"
MANIFEST = PACK / "prime_derivative_notation_decision.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_prime_derivative_notation_decision_check.py"
SOURCE_REBASE = ROOT / "pack" / "language_design_priority_rebase_v1" / "language_design_priority_rebase.detjson"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
NEXT = "LANG_FLOW_TYPE_COLLISION_RENAME_V1"


def fail(message: str) -> None:
    print(f"lang_prime_derivative_notation_decision_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")


def require(path: Path) -> None:
    if not path.exists():
        fail(f"missing required path: {path.relative_to(ROOT)}")


def require_contains(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing tokens: {missing}")


def run(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
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


def load_json(path: Path) -> dict:
    return json.loads(read(path))


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


def expected_notations() -> list[dict[str, object]]:
    return [
        {
            "id": "first_time_derivative",
            "base_identifier": "위치",
            "surface": "위치'",
            "derivative_order": 1,
            "recommended": True,
            "ssot_landed": False,
            "parser_landed": False,
            "runtime_landed": False,
            "example_row": "위치' =:= 속도.",
        },
        {
            "id": "second_time_derivative",
            "base_identifier": "위치",
            "surface": "위치''",
            "derivative_order": 2,
            "recommended": True,
            "ssot_landed": False,
            "parser_landed": False,
            "runtime_landed": False,
            "example_row": "위치'' =:= 가속도.",
        },
    ]


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
        SOURCE_REBASE,
        ROOT / "tests" / "run_language_design_priority_rebase_check.py",
        DEV_SUMMARY,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "LANG_PRIME_DERIVATIVE_NOTATION_DECISION_V1",
        "위치'",
        "위치''",
        "위치' =:= 속도.",
        "위치'' =:= 가속도.",
        "위치의변화",
        "변화(위치)",
        "d위치/dt",
        "docs/ssot/**",
        "새 언어 설계 안정화 계획: 2/8 = 25%",
        "긴급 언어 결정 evidence closure: 1/3 = 33%",
        "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, tokens[:8])
    require_contains(
        SSOT_NOTE,
        [
            "Adopt prime derivative notation",
            "위치'",
            "위치''",
            "No parser/runtime landed claim",
            "Codex did not edit `docs/ssot/**`",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "LANG_PRIME_DERIVATIVE_NOTATION_DECISION_V1",
            "lang_prime_derivative_notation_decision_v1",
            "ddn.language.prime_derivative_notation_decision.v1",
            "새 언어 설계 안정화 계획: 2/8 = 25%",
            "긴급 언어 결정 evidence closure: 1/3 = 33%",
            "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_prime_derivative_notation_decision_v1",
        "kind": "lang_prime_derivative_notation_decision",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "prime_derivative_notation_decision_claim": True,
        "prime_notation_landed_claim": False,
        "prime_notation_parser_claim": False,
        "prime_notation_runtime_claim": False,
        "connect_block_claim": False,
        "selected_first_derivative": "위치'",
        "selected_second_derivative": "위치''",
        "closed_by": "LANG_PRIME_DERIVATIVE_NOTATION_DECISION_V1",
        "based_on": "LANGUAGE_DESIGN_PRIORITY_REBASE_V1",
        "proposal_doc": "docs/context/proposals/LANG_PRIME_DERIVATIVE_NOTATION_DECISION_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_PRIME_DERIVATIVE_NOTATION_20260606.md",
        "decision_manifest": "pack/lang_prime_derivative_notation_decision_v1/prime_derivative_notation_decision.detjson",
        "source_priority_rebase": "pack/language_design_priority_rebase_v1/language_design_priority_rebase.detjson",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "language_design_queue_closed": 2,
        "language_design_queue_total": 8,
        "language_design_queue_percent": 25,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 1,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 33,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 48,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 53,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.prime_derivative_notation_decision.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "LANG_PRIME_DERIVATIVE_NOTATION_DECISION_V1":
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    for flag in [
        "runtime_claim",
        "product_code_change",
        "product_ui_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "parser_frontdoor_change",
        "stdlib_surface_change",
        "ssot_edit_claim",
        "prime_notation_landed_claim",
        "prime_notation_parser_claim",
        "prime_notation_runtime_claim",
        "connect_block_claim",
    ]:
        if manifest.get(flag) is not False:
            fail(f"manifest {flag} expected false, got {manifest.get(flag)!r}")
    if manifest.get("selected_notations") != expected_notations():
        fail(f"selected notations mismatch: {manifest.get('selected_notations')!r}")
    rejected = [row.get("surface") for row in manifest.get("rejected_alternatives", [])]
    if rejected != ["위치의변화", "변화(위치)", "d위치/dt"]:
        fail(f"rejected alternatives mismatch: {rejected!r}")
    if manifest.get("example_block") != ["세움 {", "  위치' =:= 속도.", "  위치'' =:= 가속도.", "}."]:
        fail(f"example block mismatch: {manifest.get('example_block')!r}")
    if manifest.get("queue_plan") != {"closed": 2, "total": 8, "percent": 25}:
        fail(f"queue plan mismatch: {manifest.get('queue_plan')!r}")
    if manifest.get("urgent_evidence_plan") != {"closed": 1, "total": 3, "percent": 33}:
        fail(f"urgent evidence plan mismatch: {manifest.get('urgent_evidence_plan')!r}")
    if manifest.get("urgent_ssot_landed_plan") != {"closed": 0, "total": 3, "percent": 0}:
        fail(f"urgent SSOT plan mismatch: {manifest.get('urgent_ssot_landed_plan')!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")
    required_blocked = {
        "docs_ssot_edit",
        "parser_frontdoor_change",
        "runtime_surface_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "prime_notation_runtime_landed",
        "prime_notation_parser_landed",
        "connect_block_addition",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")


def check_source_alignment() -> None:
    source = load_json(SOURCE_REBASE)
    if source.get("schema") != "ddn.language.design_priority_rebase.v1":
        fail(f"source schema mismatch: {source.get('schema')!r}")
    if source.get("next_item") != "LANG_PRIME_DERIVATIVE_NOTATION_DECISION_V1":
        fail(f"source next item mismatch: {source.get('next_item')!r}")
    prime = None
    for row in source.get("urgent_recommendations", []):
        if row.get("id") == "prime_derivative_notation":
            prime = row
            break
    if not prime:
        fail("source missing prime_derivative_notation recommendation")
    if prime.get("preferred") != "위치'" or prime.get("second_order") != "위치''":
        fail(f"source prime notation mismatch: {prime!r}")
    if prime.get("ssot_landed") is not False or prime.get("runtime_landed") is not False:
        fail(f"source prime recommendation must not be landed: {prime!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "LANG_PRIME_DERIVATIVE_NOTATION_DECISION_V1",
        "prime derivative notation decision sealed",
        "prime notation schema: ddn.language.prime_derivative_notation_decision.v1",
        "selected notation: 위치' / 위치''",
        "language queue: 2/8 = 25%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/lang_prime_derivative_notation_decision_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "lang_prime_derivative_notation_decision_v1"],
        ["python", "tests/run_language_design_priority_rebase_check.py"],
    ]:
        proc = run(cmd, timeout=180)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("lang_prime_derivative_notation_decision_check: ok")


if __name__ == "__main__":
    main()
