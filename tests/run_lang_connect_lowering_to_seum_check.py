from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_CONNECT_LOWERING_TO_SEUM_CHECK_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_CONNECT_LOWERING_TO_SEUM_CHECK_V1.md"
MANUSCRIPT_SEED = ROOT / "docs" / "context" / "manuscripts" / "ddonirang_series" / "03_실행과_시뮬레이션" / "CONNECT_LOWERING_TO_SEUM_SEED_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_CONNECT_LOWERING_TO_SEUM_CHECK_20260606.md"
PACK = ROOT / "pack" / "lang_connect_lowering_to_seum_check_v1"
MANIFEST = PACK / "connect_lowering_to_seum_check.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_connect_lowering_to_seum_check.py"
SOURCE_REBASE = ROOT / "pack" / "language_design_priority_rebase_v1" / "language_design_priority_rebase.detjson"
SOURCE_OWNER = ROOT / "pack" / "lang_owner_inner_seum_structure_check_v1" / "owner_inner_seum_structure_check.detjson"
SOURCE_CONNECT = ROOT / "docs" / "context" / "proposals" / "PROPOSAL_CONNECT_KOREAN_INNER_SENTENCE_SURFACE_V1_20260524.md"
SOURCE_FLOW = ROOT / "docs" / "context" / "proposals" / "PROPOSAL_CONNECT_FLOW_TERM_CONFLICT_HEULEUGE_V1_20260524.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
NEXT = "LANG_DSTRICT_DULTRA_SOLVER_STRATEGY_PROPOSAL_V1"


def fail(message: str) -> None:
    print(f"lang_connect_lowering_to_seum_check: FAIL: {message}", file=sys.stderr)
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


def surface_tokens() -> list[str]:
    return [
        "전지.양극과 전구.왼핀을 (전압은 같게, 전류는 흐르게) 잇기.",
        "세움",
        "전지.양극.전압 =:= 전구.왼핀.전압.",
        "전지.양극.전류 + 전구.왼핀.전류 =:= 0.",
        "잇기 {}",
        "실리게",
        "흐름포트",
    ]


def check_required_files() -> None:
    for path in [
        DOC,
        PROPOSAL,
        MANUSCRIPT_SEED,
        SSOT_NOTE,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        PACK / "example_surface.ddn",
        CONTRACT,
        MANIFEST,
        CHECKER,
        SOURCE_REBASE,
        SOURCE_OWNER,
        SOURCE_CONNECT,
        SOURCE_FLOW,
        ROOT / "tests" / "run_lang_owner_inner_seum_structure_check.py",
        DEV_SUMMARY,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "LANG_CONNECT_LOWERING_TO_SEUM_CHECK_V1",
        *surface_tokens(),
        "docs/ssot/**",
        "새 언어 설계 안정화 계획: 7/8 = 88%",
        "긴급 언어 결정 evidence closure: 3/3 = 100%",
        "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, surface_tokens() + [NEXT])
    require_contains(MANUSCRIPT_SEED, surface_tokens()[:5] + ["parser/runtime landed claim", "3권"])
    require_contains(
        SSOT_NOTE,
        surface_tokens()[:5]
        + [
            "No parser/runtime landed claim",
            "No `잇기 {}` block landed claim",
            "Codex did not edit `docs/ssot/**`",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "LANG_CONNECT_LOWERING_TO_SEUM_CHECK_V1",
            "lang_connect_lowering_to_seum_check_v1",
            "ddn.language.connect_lowering_to_seum_check.v1",
            "새 언어 설계 안정화 계획: 7/8 = 88%",
            "긴급 언어 결정 evidence closure: 3/3 = 100%",
            "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_connect_lowering_to_seum_check_v1",
        "kind": "lang_connect_lowering_to_seum_check",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "connect_lowering_to_seum_check_claim": True,
        "connect_lowering_parser_landed_claim": False,
        "connect_lowering_runtime_landed_claim": False,
        "connect_block_landed_claim": False,
        "flow_sign_convention_closed_claim": False,
        "source_connect_surface": "전지.양극과 전구.왼핀을 (전압은 같게, 전류는 흐르게) 잇기.",
        "selected_equality_row": "전지.양극.전압 =:= 전구.왼핀.전압.",
        "selected_flow_row": "전지.양극.전류 + 전구.왼핀.전류 =:= 0.",
        "closed_by": "LANG_CONNECT_LOWERING_TO_SEUM_CHECK_V1",
        "based_on": "LANG_OWNER_INNER_SEUM_STRUCTURE_CHECK_V1",
        "proposal_doc": "docs/context/proposals/LANG_CONNECT_LOWERING_TO_SEUM_CHECK_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_CONNECT_LOWERING_TO_SEUM_CHECK_20260606.md",
        "manuscript_seed_doc": "docs/context/manuscripts/ddonirang_series/03_실행과_시뮬레이션/CONNECT_LOWERING_TO_SEUM_SEED_V1.md",
        "decision_manifest": "pack/lang_connect_lowering_to_seum_check_v1/connect_lowering_to_seum_check.detjson",
        "source_priority_rebase": "pack/language_design_priority_rebase_v1/language_design_priority_rebase.detjson",
        "source_owner_inner_seum": "pack/lang_owner_inner_seum_structure_check_v1/owner_inner_seum_structure_check.detjson",
        "source_connect_proposal": "docs/context/proposals/PROPOSAL_CONNECT_KOREAN_INNER_SENTENCE_SURFACE_V1_20260524.md",
        "source_flow_conflict_proposal": "docs/context/proposals/PROPOSAL_CONNECT_FLOW_TERM_CONFLICT_HEULEUGE_V1_20260524.md",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "language_design_queue_closed": 7,
        "language_design_queue_total": 8,
        "language_design_queue_percent": 88,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
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
    if manifest.get("schema") != "ddn.language.connect_lowering_to_seum_check.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "LANG_CONNECT_LOWERING_TO_SEUM_CHECK_V1":
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
        "connect_lowering_parser_landed_claim",
        "connect_lowering_runtime_landed_claim",
        "connect_block_landed_claim",
        "flow_sign_convention_closed_claim",
    ]:
        if manifest.get(flag) is not False:
            fail(f"manifest {flag} expected false, got {manifest.get(flag)!r}")
    expected_seed = {
        "source_surface": "전지.양극과 전구.왼핀을 (전압은 같게, 전류는 흐르게) 잇기.",
        "target_surface": "세움",
        "connect_is_new_block": False,
        "parser_landed": False,
        "runtime_landed": False,
        "publication_ready": False,
    }
    if manifest.get("surface_seed") != expected_seed:
        fail(f"surface seed mismatch: {manifest.get('surface_seed')!r}")
    lowered = [row.get("lowered_row") for row in manifest.get("lowering_rows", [])]
    if lowered != ["전지.양극.전압 =:= 전구.왼핀.전압.", "전지.양극.전류 + 전구.왼핀.전류 =:= 0."]:
        fail(f"lowering rows mismatch: {lowered!r}")
    rejected = [row.get("surface") for row in manifest.get("non_core_surfaces", [])]
    if rejected != ["잇기 {}", "실리게", "흐름포트"]:
        fail(f"non-core surfaces mismatch: {rejected!r}")
    if manifest.get("queue_plan") != {"closed": 7, "total": 8, "percent": 88}:
        fail(f"queue plan mismatch: {manifest.get('queue_plan')!r}")
    if manifest.get("urgent_evidence_plan") != {"closed": 3, "total": 3, "percent": 100}:
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
        "connect_lowering_parser_landed",
        "connect_lowering_runtime_landed",
        "connect_block_landed",
        "flow_sign_convention_closed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")


def check_source_alignment() -> None:
    rebase = load_json(SOURCE_REBASE)
    guardrail = next((row for row in rebase.get("identity_guardrails", []) if row.get("id") == "connect_is_seum_sugar"), None)
    if not guardrail or guardrail.get("keep") is not True:
        fail(f"source rebase connect guardrail mismatch: {guardrail!r}")
    if guardrail.get("items") != ["잇기"]:
        fail(f"source rebase connect items mismatch: {guardrail!r}")

    owner = load_json(SOURCE_OWNER)
    if owner.get("schema") != "ddn.language.owner_inner_seum_structure_check.v1":
        fail(f"source owner schema mismatch: {owner.get('schema')!r}")
    if owner.get("next_item") != "LANG_CONNECT_LOWERING_TO_SEUM_CHECK_V1":
        fail(f"source owner next item mismatch: {owner.get('next_item')!r}")
    if owner.get("queue_plan") != {"closed": 6, "total": 8, "percent": 75}:
        fail(f"source owner queue mismatch: {owner.get('queue_plan')!r}")

    require_contains(SOURCE_CONNECT, ["전지.양극과 전구.왼핀을", "전압은 같게", "전류는 흐르게", "전지.양극.전압 =:= 전구.왼핀.전압.", "잇기 {}"])
    require_contains(SOURCE_FLOW, ["Y은/는 흐르게", "전지.양극.전류 + 전구.왼핀.전류 =:= 0.", "흐름포트", "실리게"])


def check_surface_seed_not_executed() -> None:
    require_contains(PACK / "example_surface.ddn", surface_tokens()[:4])
    golden = read(PACK / "golden.jsonl")
    if "example_surface.ddn" in golden:
        fail("example_surface.ddn must remain a non-executed surface seed")


def check_evidence_rows() -> None:
    manifest = load_json(MANIFEST)
    for row in manifest.get("evidence_rows", []):
        path = ROOT / row["path"]
        require(path)
        require_contains(path, list(row["tokens"]))


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "LANG_CONNECT_LOWERING_TO_SEUM_CHECK_V1",
        "connect lowering to seum check sealed",
        "connect lowering schema: ddn.language.connect_lowering_to_seum_check.v1",
        "surface: 잇기 -> 세움 equations",
        "language queue: 7/8 = 88%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/lang_connect_lowering_to_seum_check_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "lang_connect_lowering_to_seum_check_v1"],
        ["python", "tests/run_lang_owner_inner_seum_structure_check.py"],
    ]:
        proc = run(cmd, timeout=300)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_surface_seed_not_executed()
    check_evidence_rows()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("lang_connect_lowering_to_seum_check: ok")


if __name__ == "__main__":
    main()
