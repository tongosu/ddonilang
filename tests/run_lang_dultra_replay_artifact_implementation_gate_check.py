from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_DULTRA_REPLAY_ARTIFACT_IMPLEMENTATION_GATE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_DULTRA_REPLAY_ARTIFACT_IMPLEMENTATION_GATE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_DULTRA_REPLAY_ARTIFACT_IMPLEMENTATION_GATE_20260606.md"
PACK = ROOT / "pack" / "lang_dultra_replay_artifact_implementation_gate_v1"
MANIFEST = PACK / "dultra_replay_artifact_implementation_gate.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_dultra_replay_artifact_implementation_gate_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_CONTRACT = ROOT / "pack" / "lang_dultra_recorded_replay_contract_v1" / "dultra_recorded_replay_contract.detjson"
SOURCE_VELOCITY = ROOT / "pack" / "lang_velocity_verlet_runtime_gate_rebase_v1" / "velocity_verlet_runtime_gate_rebase.detjson"
PREVIOUS_CHECKER = ROOT / "tests" / "run_lang_velocity_verlet_runtime_gate_rebase_check.py"
CONTRACT_CHECKER = ROOT / "tests" / "run_lang_dultra_recorded_replay_contract_pack_check.py"

WORK_ITEM = "LANG_DULTRA_REPLAY_ARTIFACT_IMPLEMENTATION_GATE_V1"
NEXT = "LANG_OWNER_INNER_SEUM_RUNTIME_SCOPE_REBASE_V1"
SECTIONS = [
    "solver_identity",
    "initial_context",
    "input_sequence",
    "step_trace",
    "normalization_metadata",
    "failure_diag",
    "claim_boundary",
]
GATES = [
    ("contract_pack", "closed"),
    ("artifact_schema_shape", "closed_now"),
    ("writer_product_path", "planned_after_schema"),
    ("reader_verifier_product_path", "planned_after_writer"),
    ("failure_taxonomy", "planned_after_verifier"),
    ("claim_boundary_check", "planned_after_verifier"),
]


def fail(message: str) -> None:
    print(f"lang_dultra_replay_artifact_implementation_gate_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_CONTRACT,
        SOURCE_VELOCITY,
        PREVIOUS_CHECKER,
        CONTRACT_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        "solver_identity",
        "initial_context",
        "input_sequence",
        "step_trace",
        "normalization_metadata",
        "failure_diag",
        "claim_boundary",
        "dultra_replay.detjson",
        "No `docs/ssot/**` edit",
        "No D-ULTRA replay artifact writer landed claim",
        "No D-ULTRA replay verifier landed claim",
        "다음 언어 구현 위험 제거 계획: 5/6 = 83%",
        "D-ULTRA replay artifact implementation gate: 1/1 = 100%",
        "ROADMAP_V2 전체: queue-expanded 58/90 = 64%",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "Required Gates", "Required Sections", "5/6 = 83%", "58/90 = 64%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "seven sections",
            "writer and reader/verifier only in product paths",
            "No runtime or stdlib surface change",
            "No D-ULTRA replay artifact writer landed claim",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.dultra_replay_artifact_implementation_gate.v1",
            "lang_dultra_replay_artifact_implementation_gate_v1",
            "다음 언어 구현 위험 제거 계획: 5/6 = 83%",
            "D-ULTRA replay artifact implementation gate: 1/1 = 100%",
            "ROADMAP_V2 전체: queue-expanded 58/90 = 64%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_dultra_replay_artifact_implementation_gate_v1",
        "kind": "lang_dultra_replay_artifact_implementation_gate",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "dultra_replay_artifact_implementation_gate_claim": True,
        "dultra_replay_artifact_writer_landed_claim": False,
        "dultra_replay_verifier_landed_claim": False,
        "dstrict_truth_claim": False,
        "dae_solver_current_line_claim": False,
        "adaptive_solver_current_line_claim": False,
        "external_solver_current_line_claim": False,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_VELOCITY_VERLET_RUNTIME_GATE_REBASE_V1",
        "proposal_doc": "docs/context/proposals/LANG_DULTRA_REPLAY_ARTIFACT_IMPLEMENTATION_GATE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_DULTRA_REPLAY_ARTIFACT_IMPLEMENTATION_GATE_20260606.md",
        "decision_manifest": "pack/lang_dultra_replay_artifact_implementation_gate_v1/dultra_replay_artifact_implementation_gate.detjson",
        "source_dultra_recorded_replay_contract": "pack/lang_dultra_recorded_replay_contract_v1/dultra_recorded_replay_contract.detjson",
        "source_velocity_verlet_runtime_gate": "pack/lang_velocity_verlet_runtime_gate_rebase_v1/velocity_verlet_runtime_gate_rebase.detjson",
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
        "language_risk_removal_closed": 5,
        "language_risk_removal_total": 6,
        "language_risk_removal_percent": 83,
        "dultra_replay_artifact_implementation_gate_closed": 1,
        "dultra_replay_artifact_implementation_gate_total": 1,
        "dultra_replay_artifact_implementation_gate_percent": 100,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 58,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 64,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.dultra_replay_artifact_implementation_gate.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")

    expected_flags = {
        "dultra_replay_artifact_implementation_gate_claim": True,
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "dultra_replay_artifact_writer_landed_claim": False,
        "dultra_replay_verifier_landed_claim": False,
        "dstrict_truth_claim": False,
        "dae_solver_current_line_claim": False,
        "adaptive_solver_current_line_claim": False,
        "external_solver_current_line_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")

    if manifest.get("required_sections") != SECTIONS:
        fail(f"required sections mismatch: {manifest.get('required_sections')!r}")

    artifacts = manifest.get("recommended_artifact_outputs", [])
    if [row.get("path") for row in artifacts] != [
        "dultra_replay.detjson",
        "dultra_replay.trace.jsonl",
        "dultra_replay.failure.detjson",
        "dultra_replay.claim_boundary.detjson",
    ]:
        fail(f"artifact outputs mismatch: {artifacts!r}")
    for row in artifacts:
        if not row.get("role") or row.get("landed_now") is not False:
            fail(f"artifact output must be non-landed and have role: {row!r}")

    gates = manifest.get("implementation_gates", [])
    if len(gates) != len(GATES):
        fail(f"implementation gate count mismatch: {len(gates)}")
    for index, ((gate_id, status), row) in enumerate(zip(GATES, gates), start=1):
        if row.get("order") != index or row.get("id") != gate_id or row.get("status") != status:
            fail(f"implementation gate mismatch: {row!r}")
        if not row.get("required_evidence"):
            fail(f"implementation gate missing evidence: {row!r}")

    expected_failures = {
        "missing_artifact",
        "tampered_artifact_hash",
        "parse_failure",
        "schema_mismatch",
        "low_replay_confidence",
        "claim_boundary_violation",
    }
    if set(manifest.get("failure_taxonomy", [])) != expected_failures:
        fail(f"failure taxonomy mismatch: {manifest.get('failure_taxonomy')!r}")

    for row in manifest.get("product_anchor_rows", []):
        path = ROOT / row.get("path", "")
        require(path)
        require_contains(path, row.get("tokens", []))
        if row.get("changed_now") is not False:
            fail(f"product anchor changed_now must be false: {row!r}")

    required_blocked = {
        "docs_ssot_edit",
        "parser_frontdoor_change",
        "runtime_surface_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "product_ui_change",
        "dultra_replay_artifact_writer_landed",
        "dultra_replay_verifier_landed",
        "dstrict_truth_claim",
        "dae_solver_current_line",
        "adaptive_solver_current_line",
        "external_solver_current_line",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")

    expected_plans = {
        "language_risk_removal_plan": {"closed": 5, "total": 6, "percent": 83},
        "dultra_replay_artifact_implementation_gate": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 58, "total": 90, "percent": 64},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"{key} mismatch: {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    contract = load_json(SOURCE_CONTRACT)
    if [row.get("id") for row in contract.get("required_contract_sections", [])] != SECTIONS:
        fail("source contract sections mismatch")
    if contract.get("recorded_replay_runtime_landed_claim") is not False:
        fail("source contract must not claim runtime replay")
    if contract.get("dstrict_truth_claim") is not False:
        fail("source contract must not claim D-STRICT truth")

    velocity = load_json(SOURCE_VELOCITY)
    if velocity.get("next_item") != WORK_ITEM:
        fail(f"velocity gate next mismatch: {velocity.get('next_item')!r}")
    if velocity.get("dultra_replay_runtime_landed_claim") is not False:
        fail("velocity gate must not claim D-ULTRA replay runtime")
    if velocity.get("language_risk_removal_plan") != {"closed": 4, "total": 6, "percent": 67}:
        fail(f"velocity gate risk progress mismatch: {velocity.get('language_risk_removal_plan')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected_stdout = [
        WORK_ITEM,
        "dultra replay artifact implementation gate sealed",
        "schema: ddn.language.dultra_replay_artifact_implementation_gate.v1",
        "required sections: 7",
        "required artifact gates: 6",
        "risk removal: 5/6 = 83%",
        "runtime landed: false",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/lang_dultra_replay_artifact_implementation_gate_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected_stdout:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_dultra_replay_artifact_implementation_gate_v1"], timeout=240)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_previous_checkers() -> None:
    for checker in [PREVIOUS_CHECKER, CONTRACT_CHECKER]:
        proc = run([sys.executable, str(checker.relative_to(ROOT))], timeout=900)
        if proc.returncode != 0:
            fail(f"previous checker failed ({checker.relative_to(ROOT)}):\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_golden()
    check_pack_golden()
    check_previous_checkers()
    require_docs_ssot_clean()
    print("lang_dultra_replay_artifact_implementation_gate_check: PASS")


if __name__ == "__main__":
    main()
