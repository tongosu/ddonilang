from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_FLOW_TYPE_COLLISION_RENAME_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_FLOW_TYPE_COLLISION_RENAME_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_FLOW_TYPE_COLLISION_RENAME_20260606.md"
PACK = ROOT / "pack" / "lang_flow_type_collision_rename_v1"
MANIFEST = PACK / "flow_type_collision_rename.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_flow_type_collision_rename_check.py"
SOURCE_REBASE = ROOT / "pack" / "language_design_priority_rebase_v1" / "language_design_priority_rebase.detjson"
SOURCE_PRIME = ROOT / "pack" / "lang_prime_derivative_notation_decision_v1" / "prime_derivative_notation_decision.detjson"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
NEXT = "LANG_SIM_CONSTRAINT_THIRD_LAYER_NAME_V1"


def fail(message: str) -> None:
    print(f"lang_flow_type_collision_rename_check: FAIL: {message}", file=sys.stderr)
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
        SSOT_NOTE,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        MANIFEST,
        CHECKER,
        SOURCE_REBASE,
        SOURCE_PRIME,
        ROOT / "tests" / "run_lang_prime_derivative_notation_decision_check.py",
        DEV_SUMMARY,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "LANG_FLOW_TYPE_COLLISION_RENAME_V1",
        "흐름",
        "이력",
        "흐름창",
        "묶음흐름",
        "값흐름",
        "docs/ssot/**",
        "새 언어 설계 안정화 계획: 3/8 = 38%",
        "긴급 언어 결정 evidence closure: 2/3 = 67%",
        "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, tokens[:6])
    require_contains(
        SSOT_NOTE,
        [
            "Adopt a naming split",
            "흐름",
            "이력",
            "No parser/runtime landed claim",
            "No stdlib rename landed claim",
            "Codex did not edit `docs/ssot/**`",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "LANG_FLOW_TYPE_COLLISION_RENAME_V1",
            "lang_flow_type_collision_rename_v1",
            "ddn.language.flow_type_collision_rename.v1",
            "새 언어 설계 안정화 계획: 3/8 = 38%",
            "긴급 언어 결정 evidence closure: 2/3 = 67%",
            "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_flow_type_collision_rename_v1",
        "kind": "lang_flow_type_collision_rename",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "flow_type_collision_rename_decision_claim": True,
        "flow_type_rename_runtime_landed_claim": False,
        "stream_alias_removed_claim": False,
        "backward_compat_break_claim": False,
        "selected_value_type_name": "이력",
        "kept_connector_term": "흐름",
        "closed_by": "LANG_FLOW_TYPE_COLLISION_RENAME_V1",
        "based_on": "LANG_PRIME_DERIVATIVE_NOTATION_DECISION_V1",
        "proposal_doc": "docs/context/proposals/LANG_FLOW_TYPE_COLLISION_RENAME_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_FLOW_TYPE_COLLISION_RENAME_20260606.md",
        "decision_manifest": "pack/lang_flow_type_collision_rename_v1/flow_type_collision_rename.detjson",
        "source_priority_rebase": "pack/language_design_priority_rebase_v1/language_design_priority_rebase.detjson",
        "source_prime_decision": "pack/lang_prime_derivative_notation_decision_v1/prime_derivative_notation_decision.detjson",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "language_design_queue_closed": 3,
        "language_design_queue_total": 8,
        "language_design_queue_percent": 38,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 2,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 67,
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


def expected_collision_rows() -> list[dict[str, object]]:
    return [
        {
            "id": "core_stream_schema_collision",
            "path": "core/src/platform.rs",
            "tokens": ["ddn.stream.v1", "흐름"],
            "classification": "value_stream_runtime_resource",
            "rename_decision_only": True,
            "parser_landed": False,
            "runtime_landed": False,
            "ssot_landed": False,
        },
        {
            "id": "wasm_stream_sidecar_collision",
            "path": "tool/src/wasm_api.rs",
            "tokens": ["collect_streams", "가격흐름_길이"],
            "classification": "wasm_stream_sidecar_metadata",
            "rename_decision_only": True,
            "parser_landed": False,
            "runtime_landed": False,
            "ssot_landed": False,
        },
        {
            "id": "stdlib_stream_alias_collision",
            "path": "lang/src/stdlib.rs",
            "tokens": ["흐름.만들기", "흐름.밀어넣기", "흐름.차림"],
            "classification": "stdlib_value_stream_surface",
            "rename_decision_only": True,
            "parser_landed": False,
            "runtime_landed": False,
            "ssot_landed": False,
        },
        {
            "id": "parser_pipe_flow_term_collision",
            "path": "lang/src/parser.rs",
            "tokens": ["흐름값"],
            "classification": "parser_diagnostic_or_pipe_flow_surface",
            "rename_decision_only": True,
            "parser_landed": False,
            "runtime_landed": False,
            "ssot_landed": False,
        },
        {
            "id": "normalizer_seed_flow_collision",
            "path": "lang/src/normalizer.rs",
            "tokens": ["흐름씨", "흐름값"],
            "classification": "seed_or_functional_flow_surface",
            "rename_decision_only": True,
            "parser_landed": False,
            "runtime_landed": False,
            "ssot_landed": False,
        },
        {
            "id": "old_stream_proposal_collision",
            "path": "docs/context/proposals/PROPOSAL_STREAM_STATEFUL_RINGBUFFER_V1_20260216.md",
            "tokens": ["흐름(N)", "Stream"],
            "classification": "old_ring_buffer_value_type_proposal",
            "rename_decision_only": True,
            "parser_landed": False,
            "runtime_landed": False,
            "ssot_landed": False,
        },
    ]


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.flow_type_collision_rename.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "LANG_FLOW_TYPE_COLLISION_RENAME_V1":
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
        "flow_type_rename_runtime_landed_claim",
        "stream_alias_removed_claim",
        "backward_compat_break_claim",
    ]:
        if manifest.get(flag) is not False:
            fail(f"manifest {flag} expected false, got {manifest.get(flag)!r}")
    decision = manifest.get("naming_decision", {})
    expected_decision = {
        "keep_flow_for": "port_connector_flow",
        "kept_connector_term": "흐름",
        "rename_ring_buffer_value_type_to": "이력",
        "selected_value_type_name": "이력",
        "decision_landed": True,
        "ssot_landed": False,
        "parser_landed": False,
        "runtime_landed": False,
        "stdlib_landed": False,
    }
    if decision != expected_decision:
        fail(f"naming decision mismatch: {decision!r}")
    expected_family = [
        "이력.만들기",
        "이력.밀어넣기",
        "이력.차림",
        "이력.최근값",
        "이력.길이",
        "이력.용량",
        "이력.비우기",
        "이력.잘라보기",
    ]
    if manifest.get("future_canonical_family") != expected_family:
        fail(f"future canonical family mismatch: {manifest.get('future_canonical_family')!r}")
    if manifest.get("collision_evidence_rows") != expected_collision_rows():
        fail(f"collision evidence rows mismatch: {manifest.get('collision_evidence_rows')!r}")
    rejected = [row.get("surface") for row in manifest.get("rejected_alternatives", [])]
    if rejected != ["흐름창", "묶음흐름", "값흐름"]:
        fail(f"rejected alternatives mismatch: {rejected!r}")
    policy = manifest.get("migration_policy", {})
    expected_policy = {
        "preserve_connector_flow_term": True,
        "selected_value_type_name": "이력",
        "compatibility_aliases_required_before_removal": True,
        "remove_old_aliases_now": False,
        "runtime_rename_now": False,
        "ssot_update_required_later": True,
    }
    if policy != expected_policy:
        fail(f"migration policy mismatch: {policy!r}")
    if manifest.get("queue_plan") != {"closed": 3, "total": 8, "percent": 38}:
        fail(f"queue plan mismatch: {manifest.get('queue_plan')!r}")
    if manifest.get("urgent_evidence_plan") != {"closed": 2, "total": 3, "percent": 67}:
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
        "flow_type_rename_runtime_landed",
        "stream_alias_removed",
        "backward_compat_break",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")


def check_source_alignment() -> None:
    rebase = load_json(SOURCE_REBASE)
    if rebase.get("schema") != "ddn.language.design_priority_rebase.v1":
        fail(f"source rebase schema mismatch: {rebase.get('schema')!r}")
    flow = None
    for row in rebase.get("urgent_recommendations", []):
        if row.get("id") == "flow_type_collision":
            flow = row
            break
    if not flow:
        fail("source rebase missing flow_type_collision recommendation")
    if flow.get("keep_flow_for") != "port_connector_flow" or flow.get("rename_ring_buffer_to") != "이력":
        fail(f"source flow recommendation mismatch: {flow!r}")
    if flow.get("next_item") != "LANG_FLOW_TYPE_COLLISION_RENAME_V1":
        fail(f"source flow next item mismatch: {flow!r}")
    if flow.get("ssot_landed") is not False or flow.get("runtime_landed") is not False:
        fail(f"source flow recommendation must not be landed: {flow!r}")

    prime = load_json(SOURCE_PRIME)
    if prime.get("schema") != "ddn.language.prime_derivative_notation_decision.v1":
        fail(f"source prime schema mismatch: {prime.get('schema')!r}")
    if prime.get("next_item") != "LANG_FLOW_TYPE_COLLISION_RENAME_V1":
        fail(f"source prime next item mismatch: {prime.get('next_item')!r}")
    if prime.get("queue_plan") != {"closed": 2, "total": 8, "percent": 25}:
        fail(f"source prime queue mismatch: {prime.get('queue_plan')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "LANG_FLOW_TYPE_COLLISION_RENAME_V1",
        "flow type collision rename sealed",
        "flow rename schema: ddn.language.flow_type_collision_rename.v1",
        "selected value type name: 이력",
        "language queue: 3/8 = 38%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/lang_flow_type_collision_rename_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "lang_flow_type_collision_rename_v1"],
        ["python", "tests/run_lang_prime_derivative_notation_decision_check.py"],
    ]:
        proc = run(cmd, timeout=240)
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
    print("lang_flow_type_collision_rename_check: ok")


if __name__ == "__main__":
    main()
