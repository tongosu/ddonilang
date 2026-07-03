from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_OWNER_INNER_SEUM_STRUCTURE_CHECK_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_OWNER_INNER_SEUM_STRUCTURE_CHECK_V1.md"
MANUSCRIPT_SEED = ROOT / "docs" / "context" / "manuscripts" / "ddonirang_series" / "03_실행과_시뮬레이션" / "OWNER_INNER_SEUM_STRUCTURE_SEED_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_OWNER_INNER_SEUM_STRUCTURE_CHECK_20260606.md"
PACK = ROOT / "pack" / "lang_owner_inner_seum_structure_check_v1"
MANIFEST = PACK / "owner_inner_seum_structure_check.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_owner_inner_seum_structure_check.py"
SOURCE_SEUM = ROOT / "pack" / "lang_seum_vol3_prime_example_pack_v1" / "seum_vol3_prime_example_pack.detjson"
SOURCE_PROPOSAL = ROOT / "docs" / "context" / "proposals" / "PROPOSAL_SEUM_VOL3_RELATION_SURFACE_V1_20260524.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
NEXT = "LANG_CONNECT_LOWERING_TO_SEUM_CHECK_V1"


def fail(message: str) -> None:
    print(f"lang_owner_inner_seum_structure_check: FAIL: {message}", file=sys.stderr)
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
        "공:임자",
        "성질",
        "세움",
        "위치: 수 <- 0.",
        "속도: 수 <- 0.",
        "위치' =:= 속도.",
        "힘가해짐을 받으면",
        "속도 <- 속도 + 힘.",
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
        SOURCE_SEUM,
        SOURCE_PROPOSAL,
        ROOT / "tests" / "run_lang_seum_vol3_prime_example_pack_check.py",
        DEV_SUMMARY,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "LANG_OWNER_INNER_SEUM_STRUCTURE_CHECK_V1",
        *surface_tokens(),
        "docs/ssot/**",
        "새 언어 설계 안정화 계획: 6/8 = 75%",
        "긴급 언어 결정 evidence closure: 3/3 = 100%",
        "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, surface_tokens() + [NEXT])
    require_contains(MANUSCRIPT_SEED, surface_tokens() + ["parser/runtime landed claim", "3권"])
    require_contains(
        SSOT_NOTE,
        surface_tokens()
        + [
            "No parser/runtime landed claim",
            "No owner-inner `세움 {}` runtime landed claim",
            "Codex did not edit `docs/ssot/**`",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "LANG_OWNER_INNER_SEUM_STRUCTURE_CHECK_V1",
            "lang_owner_inner_seum_structure_check_v1",
            "ddn.language.owner_inner_seum_structure_check.v1",
            "새 언어 설계 안정화 계획: 6/8 = 75%",
            "긴급 언어 결정 evidence closure: 3/3 = 100%",
            "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_owner_inner_seum_structure_check_v1",
        "kind": "lang_owner_inner_seum_structure_check",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "owner_inner_seum_structure_check_claim": True,
        "owner_inner_seum_parser_landed_claim": False,
        "owner_inner_seum_runtime_landed_claim": False,
        "prime_notation_parser_landed_claim": False,
        "seongjil_block_landed_claim": False,
        "owner_surface": "공:임자",
        "state_group_surface": "성질",
        "relation_surface": "세움",
        "event_surface": "받으면",
        "selected_relation_row": "위치' =:= 속도.",
        "selected_event_row": "속도 <- 속도 + 힘.",
        "closed_by": "LANG_OWNER_INNER_SEUM_STRUCTURE_CHECK_V1",
        "based_on": "LANG_SEUM_VOL3_PRIME_EXAMPLE_PACK_V1",
        "proposal_doc": "docs/context/proposals/LANG_OWNER_INNER_SEUM_STRUCTURE_CHECK_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_OWNER_INNER_SEUM_STRUCTURE_CHECK_20260606.md",
        "manuscript_seed_doc": "docs/context/manuscripts/ddonirang_series/03_실행과_시뮬레이션/OWNER_INNER_SEUM_STRUCTURE_SEED_V1.md",
        "decision_manifest": "pack/lang_owner_inner_seum_structure_check_v1/owner_inner_seum_structure_check.detjson",
        "source_seum_prime_examples": "pack/lang_seum_vol3_prime_example_pack_v1/seum_vol3_prime_example_pack.detjson",
        "source_seum_relation_proposal": "docs/context/proposals/PROPOSAL_SEUM_VOL3_RELATION_SURFACE_V1_20260524.md",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "language_design_queue_closed": 6,
        "language_design_queue_total": 8,
        "language_design_queue_percent": 75,
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
    if manifest.get("schema") != "ddn.language.owner_inner_seum_structure_check.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "LANG_OWNER_INNER_SEUM_STRUCTURE_CHECK_V1":
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
        "owner_inner_seum_parser_landed_claim",
        "owner_inner_seum_runtime_landed_claim",
        "prime_notation_parser_landed_claim",
        "seongjil_block_landed_claim",
    ]:
        if manifest.get(flag) is not False:
            fail(f"manifest {flag} expected false, got {manifest.get(flag)!r}")
    expected_seed = {
        "owner": "공:임자",
        "state_group": "성질",
        "relation_block": "세움",
        "event_reaction": "받으면",
        "parser_landed": False,
        "runtime_landed": False,
        "publication_ready": False,
    }
    if manifest.get("surface_seed") != expected_seed:
        fail(f"surface seed mismatch: {manifest.get('surface_seed')!r}")
    row_surfaces = [row.get("surface") for row in manifest.get("structure_rows", [])]
    if row_surfaces != ["위치: 수 <- 0.", "속도: 수 <- 0.", "위치' =:= 속도.", "속도 <- 속도 + 힘."]:
        fail(f"structure row surfaces mismatch: {row_surfaces!r}")
    if manifest.get("queue_plan") != {"closed": 6, "total": 8, "percent": 75}:
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
        "owner_inner_seum_parser_landed",
        "owner_inner_seum_runtime_landed",
        "prime_notation_parser_landed",
        "seongjil_block_landed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")


def check_source_alignment() -> None:
    seum = load_json(SOURCE_SEUM)
    if seum.get("schema") != "ddn.language.seum_vol3_prime_example_pack.v1":
        fail(f"source seum schema mismatch: {seum.get('schema')!r}")
    if seum.get("next_item") != "LANG_OWNER_INNER_SEUM_STRUCTURE_CHECK_V1":
        fail(f"source seum next item mismatch: {seum.get('next_item')!r}")
    if seum.get("queue_plan") != {"closed": 5, "total": 8, "percent": 63}:
        fail(f"source seum queue mismatch: {seum.get('queue_plan')!r}")
    require_contains(SOURCE_PROPOSAL, ["임자 안에서도", "세움 {}", "힘 =:= 질량 * 가속도."])


def check_surface_seed_not_executed() -> None:
    require_contains(PACK / "example_surface.ddn", surface_tokens())
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
        "LANG_OWNER_INNER_SEUM_STRUCTURE_CHECK_V1",
        "owner inner seum structure check sealed",
        "owner seum schema: ddn.language.owner_inner_seum_structure_check.v1",
        "surface: 공:임자 / 성질 / 세움 / 받으면",
        "language queue: 6/8 = 75%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/lang_owner_inner_seum_structure_check_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "lang_owner_inner_seum_structure_check_v1"],
        ["python", "tests/run_lang_seum_vol3_prime_example_pack_check.py"],
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
    print("lang_owner_inner_seum_structure_check: ok")


if __name__ == "__main__":
    main()
