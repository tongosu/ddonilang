from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_DULTRA_REPLAY_ARTIFACT_WRITER_SEED_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_DULTRA_REPLAY_ARTIFACT_WRITER_SEED_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_DULTRA_REPLAY_ARTIFACT_WRITER_SEED_20260606.md"
PACK = ROOT / "pack" / "lang_dultra_replay_artifact_writer_seed_v1"
MANIFEST = PACK / "dultra_replay_artifact_writer_seed.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_dultra_replay_artifact_writer_seed_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_CONTRACT = ROOT / "pack" / "lang_dultra_recorded_replay_contract_v1" / "dultra_recorded_replay_contract.detjson"
SOURCE_GATE = ROOT / "pack" / "lang_dultra_replay_artifact_implementation_gate_v1" / "dultra_replay_artifact_implementation_gate.detjson"
SOURCE_HISTORY = ROOT / "pack" / "lang_history_alias_stdlib_bridge_v1" / "history_alias_stdlib_bridge.detjson"
HISTORY_CHECKER = ROOT / "tests" / "run_lang_history_alias_stdlib_bridge_check.py"
DULTRA_GATE_CHECKER = ROOT / "tests" / "run_lang_dultra_replay_artifact_implementation_gate_check.py"

WORK_ITEM = "LANG_DULTRA_REPLAY_ARTIFACT_WRITER_SEED_V1"
NEXT = "LANG_VELOCITY_VERLET_STDLIB_SURFACE_ACCEPTANCE_V1"
REQUIRED_SECTIONS = [
    "solver_identity",
    "initial_context",
    "input_sequence",
    "step_trace",
    "normalization_metadata",
    "failure_diag",
    "claim_boundary",
]
FALSE_CLAIMS = [
    "writer.runtime_landed",
    "writer.verifier_landed",
    "sections.claim_boundary.dstrict_truth_claim",
    "sections.claim_boundary.current_line_support_claim",
    "sections.claim_boundary.runtime_recorded_replay_implementation_landed",
]


def fail(message: str) -> None:
    print(f"lang_dultra_replay_artifact_writer_seed_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_CONTRACT,
        SOURCE_GATE,
        SOURCE_HISTORY,
        HISTORY_CHECKER,
        DULTRA_GATE_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        "build_dultra_replay_seed_artifact",
        "solver_identity",
        "claim_boundary",
        "언어 제품 경로 구현 전환 계획: 5/7 = 71%",
        "D-ULTRA replay artifact writer seed: 1/1 = 100%",
        "ROADMAP_V2 전체: queue-expanded 64/90 = 71%",
        "No `docs/ssot/**` edit",
        "No D-STRICT truth claim",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "Product Path", "5/7 = 71%", "64/90 = 71%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "build_dultra_replay_seed_artifact",
            "seven required D-ULTRA replay contract sections",
            "No runtime replay behavior change",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.dultra_replay_artifact_writer_seed.v1",
            "lang_dultra_replay_artifact_writer_seed_v1",
            "언어 제품 경로 구현 전환 계획: 5/7 = 71%",
            "D-ULTRA replay artifact writer seed: 1/1 = 100%",
            "ROADMAP_V2 전체: queue-expanded 64/90 = 71%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_dultra_replay_artifact_writer_seed_v1",
        "kind": "lang_dultra_replay_artifact_writer_seed",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "dultra_replay_artifact_writer_seed_claim": True,
        "dultra_replay_artifact_writer_seed_helper_landed_claim": True,
        "dultra_replay_artifact_writer_runtime_landed_claim": False,
        "dultra_replay_verifier_landed_claim": False,
        "dstrict_truth_claim": False,
        "dae_solver_current_line_claim": False,
        "adaptive_solver_current_line_claim": False,
        "external_solver_current_line_claim": False,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_HISTORY_ALIAS_STDLIB_BRIDGE_V1",
        "proposal_doc": "docs/context/proposals/LANG_DULTRA_REPLAY_ARTIFACT_WRITER_SEED_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_DULTRA_REPLAY_ARTIFACT_WRITER_SEED_20260606.md",
        "decision_manifest": "pack/lang_dultra_replay_artifact_writer_seed_v1/dultra_replay_artifact_writer_seed.detjson",
        "source_dultra_recorded_replay_contract": "pack/lang_dultra_recorded_replay_contract_v1/dultra_recorded_replay_contract.detjson",
        "source_dultra_replay_artifact_implementation_gate": "pack/lang_dultra_replay_artifact_implementation_gate_v1/dultra_replay_artifact_implementation_gate.detjson",
        "source_history_alias_stdlib_bridge": "pack/lang_history_alias_stdlib_bridge_v1/history_alias_stdlib_bridge.detjson",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "language_design_queue_closed": 8,
        "language_design_queue_total": 8,
        "language_design_queue_percent": 100,
        "implementation_readiness_rebase_closed": 1,
        "implementation_readiness_rebase_total": 1,
        "implementation_readiness_rebase_percent": 100,
        "implementation_readiness_followup_closed": 6,
        "implementation_readiness_followup_total": 6,
        "implementation_readiness_followup_percent": 100,
        "implementation_followup_closure_rebase_closed": 1,
        "implementation_followup_closure_rebase_total": 1,
        "implementation_followup_closure_rebase_percent": 100,
        "language_risk_removal_closed": 6,
        "language_risk_removal_total": 6,
        "language_risk_removal_percent": 100,
        "language_risk_removal_closure_rebase_closed": 1,
        "language_risk_removal_closure_rebase_total": 1,
        "language_risk_removal_closure_rebase_percent": 100,
        "language_product_path_transition_closed": 5,
        "language_product_path_transition_total": 7,
        "language_product_path_transition_percent": 71,
        "dultra_replay_artifact_writer_seed_closed": 1,
        "dultra_replay_artifact_writer_seed_total": 1,
        "dultra_replay_artifact_writer_seed_percent": 100,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 64,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 71,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    for source_key in [
        "source_dultra_recorded_replay_contract",
        "source_dultra_replay_artifact_implementation_gate",
        "source_history_alias_stdlib_bridge",
    ]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.dultra_replay_artifact_writer_seed.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")

    expected_flags = {
        "dultra_replay_artifact_writer_seed_claim": True,
        "dultra_replay_artifact_writer_seed_helper_landed_claim": True,
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "dultra_replay_artifact_writer_runtime_landed_claim": False,
        "dultra_replay_verifier_landed_claim": False,
        "dstrict_truth_claim": False,
        "dae_solver_current_line_claim": False,
        "adaptive_solver_current_line_claim": False,
        "external_solver_current_line_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")

    helper = manifest.get("product_helper", {})
    expected_helper = {
        "api": "teul_cli::cli::dultra_replay::build_dultra_replay_seed_artifact",
        "path": "tools/teul-cli/src/cli/dultra_replay.rs",
        "module_export": "tools/teul-cli/src/cli/mod.rs",
        "artifact_schema": "ddn.dultra_replay.detjson.v1",
        "product_path": True,
        "runtime_landed": False,
        "verifier_landed": False,
    }
    for key, value in expected_helper.items():
        if helper.get(key) != value:
            fail(f"helper {key} expected {value!r}, got {helper.get(key)!r}")
    if manifest.get("required_sections") != REQUIRED_SECTIONS:
        fail(f"required sections mismatch: {manifest.get('required_sections')!r}")
    if manifest.get("false_claim_fields") != FALSE_CLAIMS:
        fail(f"false claim fields mismatch: {manifest.get('false_claim_fields')!r}")

    required_not_landed = {
        "dultra_replay.trace.jsonl",
        "dultra_replay.failure.detjson",
        "dultra_replay.claim_boundary.detjson",
        "reader_verifier_product_path",
    }
    if set(manifest.get("recommended_outputs_not_landed", [])) != required_not_landed:
        fail(f"not landed outputs mismatch: {manifest.get('recommended_outputs_not_landed')!r}")

    for row in manifest.get("product_anchor_rows", []):
        path = ROOT / row.get("path", "")
        require(path)
        require_contains(path, row.get("tokens", []))

    required_blocked = {
        "docs_ssot_edit",
        "parser_frontdoor_change",
        "runtime_replay_behavior_change",
        "verifier_behavior_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "product_ui_change",
        "dultra_replay_artifact_writer_runtime_landed",
        "dultra_replay_verifier_landed",
        "dstrict_truth_claim",
        "dae_solver_current_line",
        "adaptive_solver_current_line",
        "external_solver_current_line",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")

    expected_plans = {
        "language_product_path_transition_plan": {"closed": 5, "total": 7, "percent": 71},
        "dultra_replay_artifact_writer_seed": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 64, "total": 90, "percent": 71},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    contract = load_json(SOURCE_CONTRACT)
    for section in REQUIRED_SECTIONS:
        if section not in [row.get("id") for row in contract.get("required_contract_sections", [])]:
            fail(f"source contract missing section {section}")
    if contract.get("recorded_replay_runtime_landed_claim") is not False:
        fail("source contract runtime replay claim must remain false")

    gate = load_json(SOURCE_GATE)
    if gate.get("dultra_replay_artifact_writer_landed_claim") is not False:
        fail("source artifact gate writer landed claim must remain false")
    if gate.get("dultra_replay_verifier_landed_claim") is not False:
        fail("source artifact gate verifier landed claim must remain false")

    history = load_json(SOURCE_HISTORY)
    if history.get("next_item") != WORK_ITEM:
        fail(f"history bridge next item expected {WORK_ITEM}, got {history.get('next_item')!r}")
    if history.get("language_product_path_transition_plan") != {"closed": 4, "total": 7, "percent": 57}:
        fail(f"history bridge transition progress mismatch: {history.get('language_product_path_transition_plan')!r}")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_dultra_replay_artifact_writer_seed_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    expected = [
        "LANG_DULTRA_REPLAY_ARTIFACT_WRITER_SEED_V1",
        "dultra replay artifact writer seed sealed",
        "schema: ddn.language.dultra_replay_artifact_writer_seed.v1",
        "required sections: 7",
        "product transition: 5/7 = 71%",
        "runtime landed: false",
        "next: LANG_VELOCITY_VERLET_STDLIB_SURFACE_ACCEPTANCE_V1",
    ]
    require_contains(PACK / "golden.jsonl", expected)


def check_product_test() -> None:
    proc = run(
        [
            "cargo",
            "test",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "dultra_replay_seed_artifact_has_required_sections_and_false_claims",
            "--quiet",
        ],
        timeout=300,
    )
    if proc.returncode != 0:
        fail(f"cargo D-ULTRA replay seed test failed:\n{proc.stdout}")


def check_previous_checkers() -> None:
    for checker in [HISTORY_CHECKER, DULTRA_GATE_CHECKER]:
        proc = run([sys.executable, str(checker.relative_to(ROOT))], timeout=1200)
        if proc.returncode != 0:
            fail(f"{checker.relative_to(ROOT)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_golden()
    check_product_test()
    check_previous_checkers()
    require_docs_ssot_clean()
    print("lang_dultra_replay_artifact_writer_seed_check: PASS")


if __name__ == "__main__":
    main()
