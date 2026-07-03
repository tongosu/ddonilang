from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_SEUM_VOL3_PRIME_EXAMPLE_PACK_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_SEUM_VOL3_PRIME_EXAMPLE_PACK_V1.md"
MANUSCRIPT_SEED = ROOT / "docs" / "context" / "manuscripts" / "ddonirang_series" / "03_실행과_시뮬레이션" / "SEUM_PRIME_EXAMPLE_SEED_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_SEUM_VOL3_PRIME_EXAMPLE_PACK_20260606.md"
PACK = ROOT / "pack" / "lang_seum_vol3_prime_example_pack_v1"
MANIFEST = PACK / "seum_vol3_prime_example_pack.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_seum_vol3_prime_example_pack_check.py"
SOURCE_PRIME = ROOT / "pack" / "lang_prime_derivative_notation_decision_v1" / "prime_derivative_notation_decision.detjson"
SOURCE_CONSTRAINT = ROOT / "pack" / "lang_sim_constraint_third_layer_name_v1" / "sim_constraint_third_layer_name.detjson"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
NEXT = "LANG_OWNER_INNER_SEUM_STRUCTURE_CHECK_V1"


def fail(message: str) -> None:
    print(f"lang_seum_vol3_prime_example_pack_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_PRIME,
        SOURCE_CONSTRAINT,
        ROOT / "tests" / "run_lang_sim_constraint_third_layer_name_check.py",
        DEV_SUMMARY,
    ]:
        require(path)


def required_example_tokens() -> list[str]:
    return [
        "세움",
        "위치' =:= 속도.",
        "속도' =:= 가속도.",
        "위치'' =:= 가속도.",
        "위치 <- 위치 + 속도.",
    ]


def check_docs() -> None:
    tokens = [
        "LANG_SEUM_VOL3_PRIME_EXAMPLE_PACK_V1",
        *required_example_tokens(),
        "docs/ssot/**",
        "새 언어 설계 안정화 계획: 5/8 = 63%",
        "긴급 언어 결정 evidence closure: 3/3 = 100%",
        "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, tokens[:6])
    require_contains(MANUSCRIPT_SEED, required_example_tokens() + ["parser/runtime landed claim", "3권"])
    require_contains(
        SSOT_NOTE,
        [
            "세움",
            "위치' =:= 속도.",
            "속도' =:= 가속도.",
            "위치'' =:= 가속도.",
            "No parser/runtime landed claim",
            "Codex did not edit `docs/ssot/**`",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "LANG_SEUM_VOL3_PRIME_EXAMPLE_PACK_V1",
            "lang_seum_vol3_prime_example_pack_v1",
            "ddn.language.seum_vol3_prime_example_pack.v1",
            "새 언어 설계 안정화 계획: 5/8 = 63%",
            "긴급 언어 결정 evidence closure: 3/3 = 100%",
            "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
            "docs/ssot/** 변경 없음",
        ],
    )


def expected_rows() -> list[dict[str, object]]:
    return [
        {
            "id": "position_prime_velocity",
            "row": "위치' =:= 속도.",
            "reading": "위치의 한 마디 변화는 속도다",
            "derivative_order": 1,
            "preferred_for_vol3_intro": True,
            "parser_landed": False,
            "runtime_landed": False,
        },
        {
            "id": "velocity_prime_acceleration",
            "row": "속도' =:= 가속도.",
            "reading": "속도의 한 마디 변화는 가속도다",
            "derivative_order": 1,
            "preferred_for_vol3_intro": True,
            "parser_landed": False,
            "runtime_landed": False,
        },
        {
            "id": "position_double_prime_acceleration",
            "row": "위치'' =:= 가속도.",
            "reading": "위치의 두 번째 변화는 가속도다",
            "derivative_order": 2,
            "preferred_for_vol3_intro": True,
            "parser_landed": False,
            "runtime_landed": False,
        },
    ]


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_seum_vol3_prime_example_pack_v1",
        "kind": "lang_seum_vol3_prime_example_pack",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "seum_vol3_prime_example_pack_claim": True,
        "prime_notation_parser_landed_claim": False,
        "prime_notation_runtime_landed_claim": False,
        "vol3_publication_landed_claim": False,
        "selected_example_rows": ["위치' =:= 속도.", "속도' =:= 가속도.", "위치'' =:= 가속도."],
        "contrast_row": "위치 <- 위치 + 속도.",
        "closed_by": "LANG_SEUM_VOL3_PRIME_EXAMPLE_PACK_V1",
        "based_on": "LANG_SIM_CONSTRAINT_THIRD_LAYER_NAME_V1",
        "proposal_doc": "docs/context/proposals/LANG_SEUM_VOL3_PRIME_EXAMPLE_PACK_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_SEUM_VOL3_PRIME_EXAMPLE_PACK_20260606.md",
        "manuscript_seed_doc": "docs/context/manuscripts/ddonirang_series/03_실행과_시뮬레이션/SEUM_PRIME_EXAMPLE_SEED_V1.md",
        "decision_manifest": "pack/lang_seum_vol3_prime_example_pack_v1/seum_vol3_prime_example_pack.detjson",
        "source_prime_decision": "pack/lang_prime_derivative_notation_decision_v1/prime_derivative_notation_decision.detjson",
        "source_constraint_name": "pack/lang_sim_constraint_third_layer_name_v1/sim_constraint_third_layer_name.detjson",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "language_design_queue_closed": 5,
        "language_design_queue_total": 8,
        "language_design_queue_percent": 63,
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
    if manifest.get("schema") != "ddn.language.seum_vol3_prime_example_pack.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "LANG_SEUM_VOL3_PRIME_EXAMPLE_PACK_V1":
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
        "prime_notation_parser_landed_claim",
        "prime_notation_runtime_landed_claim",
        "vol3_publication_landed_claim",
        "connect_block_claim",
    ]:
        if manifest.get(flag) is not False:
            fail(f"manifest {flag} expected false, got {manifest.get(flag)!r}")
    expected_pack = {
        "target_volume": "3권_실행과_시뮬레이션",
        "surface": "세움",
        "teaching_goal": "model_first_simulation_intro",
        "publication_ready": False,
        "parser_landed": False,
        "runtime_landed": False,
    }
    if manifest.get("example_pack") != expected_pack:
        fail(f"example pack mismatch: {manifest.get('example_pack')!r}")
    if manifest.get("example_rows") != expected_rows():
        fail(f"example rows mismatch: {manifest.get('example_rows')!r}")
    contrast = manifest.get("contrast_rows", [])
    if contrast != [
        {
            "id": "procedural_position_update",
            "row": "위치 <- 위치 + 속도.",
            "classification": "procedural_update_contrast",
            "preferred_for_vol3_intro": False,
        }
    ]:
        fail(f"contrast rows mismatch: {contrast!r}")
    if manifest.get("example_block") != [
        "세움 {",
        "  위치' =:= 속도.",
        "  속도' =:= 가속도.",
        "  위치'' =:= 가속도.",
        "}.",
    ]:
        fail(f"example block mismatch: {manifest.get('example_block')!r}")
    if manifest.get("queue_plan") != {"closed": 5, "total": 8, "percent": 63}:
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
        "prime_notation_parser_landed",
        "prime_notation_runtime_landed",
        "vol3_publication_landed",
        "connect_block_addition",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")


def check_source_alignment() -> None:
    prime = load_json(SOURCE_PRIME)
    selected = prime.get("selected_notations", [])
    first = next((row for row in selected if row.get("id") == "first_time_derivative"), None)
    second = next((row for row in selected if row.get("id") == "second_time_derivative"), None)
    if not first or first.get("example_row") != "위치' =:= 속도.":
        fail(f"source prime first notation mismatch: {first!r}")
    if not second or second.get("example_row") != "위치'' =:= 가속도.":
        fail(f"source prime second notation mismatch: {second!r}")
    for row in selected:
        if row.get("parser_landed") is not False or row.get("runtime_landed") is not False:
            fail(f"source prime row must not be landed: {row!r}")

    constraint = load_json(SOURCE_CONSTRAINT)
    if constraint.get("schema") != "ddn.language.sim_constraint_third_layer_name.v1":
        fail(f"source constraint schema mismatch: {constraint.get('schema')!r}")
    if constraint.get("next_item") != "LANG_SEUM_VOL3_PRIME_EXAMPLE_PACK_V1":
        fail(f"source constraint next item mismatch: {constraint.get('next_item')!r}")
    if constraint.get("queue_plan") != {"closed": 4, "total": 8, "percent": 50}:
        fail(f"source constraint queue mismatch: {constraint.get('queue_plan')!r}")


def check_surface_seed_not_executed() -> None:
    require_contains(PACK / "example_surface.ddn", required_example_tokens())
    golden = read(PACK / "golden.jsonl")
    if "example_surface.ddn" in golden:
        fail("example_surface.ddn must remain a non-executed surface seed")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "LANG_SEUM_VOL3_PRIME_EXAMPLE_PACK_V1",
        "seum vol3 prime example pack sealed",
        "seum example schema: ddn.language.seum_vol3_prime_example_pack.v1",
        "example rows: 위치' =:= 속도 / 속도' =:= 가속도 / 위치'' =:= 가속도",
        "language queue: 5/8 = 63%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/lang_seum_vol3_prime_example_pack_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "lang_seum_vol3_prime_example_pack_v1"],
        ["python", "tests/run_lang_sim_constraint_third_layer_name_check.py"],
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
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("lang_seum_vol3_prime_example_pack_check: ok")


if __name__ == "__main__":
    main()
