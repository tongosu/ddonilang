from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_20260606.md"
PACK = ROOT / "pack" / "lang_dultra_recorded_replay_contract_v1"
MANIFEST = PACK / "dultra_recorded_replay_contract.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_dultra_recorded_replay_contract_pack_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
SOURCE_SOLVER = ROOT / "pack" / "lang_dstrict_dultra_solver_strategy_proposal_v1" / "dstrict_dultra_solver_strategy_proposal.detjson"
SOURCE_VELOCITY = ROOT / "pack" / "lang_velocity_verlet_fixed64_order_v1" / "velocity_verlet_fixed64_order.detjson"
SOURCE_DULTRA = ROOT / "docs" / "context" / "proposals" / "PROPOSAL_DULTRA_SOLVER_STRATEGY_RECORDED_CONTRACT_V1_20260524.md"
SOURCE_METHODS = ROOT / "docs" / "context" / "proposals" / "PROPOSAL_DETERMINISTIC_SOLVER_METHODS_V1_20260524.md"
SOURCE_AI_DET = ROOT / "pack" / "ai_det_tier_capability_matrix_v1" / "README.md"
NEXT = "LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1"

REQUIRED_SECTIONS = [
    "solver_identity",
    "initial_context",
    "input_sequence",
    "step_trace",
    "normalization_metadata",
    "failure_diag",
    "claim_boundary",
]

SOURCE_RECORDED_FIELDS = [
    "solver id / version / configuration",
    "initial state hash and input sequence",
    "adaptive step choices or solver trace summary",
    "external result normalization metadata",
    "failure/diag code",
]


def fail(message: str) -> None:
    print(f"lang_dultra_recorded_replay_contract_pack_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_SOLVER,
        SOURCE_VELOCITY,
        SOURCE_DULTRA,
        SOURCE_METHODS,
        SOURCE_AI_DET,
        ROOT / "tests" / "run_lang_velocity_verlet_fixed64_order_pack_check.py",
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_V1",
        "D-ULTRA",
        "solver_identity",
        "initial_context",
        "input_sequence",
        "step_trace",
        "normalization_metadata",
        "failure_diag",
        "claim_boundary",
        "No D-ULTRA solver runtime landed claim",
        "언어 구현 준비 후속 계획: 5/6 = 83%",
        "D-ULTRA recorded replay contract pack: 1/1 = 100%",
        "ROADMAP_V2 전체: queue-expanded 52/90 = 58%",
        NEXT,
    ]
    require_contains(DOC, tokens + ["docs/ssot/**", "No D-ULTRA result as D-STRICT truth claim"])
    require_contains(PROPOSAL, tokens[:13] + ["No SSOT edit by Codex"])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            *SOURCE_RECORDED_FIELDS,
            "No D-ULTRA result as D-STRICT bit-perfect truth claim",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_V1",
            "ddn.language.dultra_recorded_replay_contract.v1",
            "lang_dultra_recorded_replay_contract_v1",
            "언어 구현 준비 후속 계획: 5/6 = 83%",
            "ROADMAP_V2 전체: queue-expanded 52/90 = 58%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_dultra_recorded_replay_contract_v1",
        "kind": "lang_dultra_recorded_replay_contract",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "dultra_recorded_replay_contract_pack_claim": True,
        "dultra_solver_runtime_landed_claim": False,
        "recorded_replay_runtime_landed_claim": False,
        "dstrict_truth_claim": False,
        "dae_solver_current_line_claim": False,
        "adaptive_solver_current_line_claim": False,
        "external_solver_current_line_claim": False,
        "closed_by": "LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_V1",
        "based_on": "LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_V1",
        "proposal_doc": "docs/context/proposals/LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_20260606.md",
        "decision_manifest": "pack/lang_dultra_recorded_replay_contract_v1/dultra_recorded_replay_contract.detjson",
        "source_dstrict_dultra_solver_strategy": "pack/lang_dstrict_dultra_solver_strategy_proposal_v1/dstrict_dultra_solver_strategy_proposal.detjson",
        "source_velocity_verlet_fixed64_order": "pack/lang_velocity_verlet_fixed64_order_v1/velocity_verlet_fixed64_order.detjson",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "language_design_queue_closed": 8,
        "language_design_queue_total": 8,
        "language_design_queue_percent": 100,
        "implementation_readiness_rebase_closed": 1,
        "implementation_readiness_rebase_total": 1,
        "implementation_readiness_rebase_percent": 100,
        "implementation_readiness_followup_closed": 5,
        "implementation_readiness_followup_total": 6,
        "implementation_readiness_followup_percent": 83,
        "dultra_recorded_replay_contract_closed": 1,
        "dultra_recorded_replay_contract_total": 1,
        "dultra_recorded_replay_contract_percent": 100,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 52,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 58,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.dultra_recorded_replay_contract.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_V1":
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    expected_flags = {
        "dultra_recorded_replay_contract_pack_claim": True,
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "dultra_solver_runtime_landed_claim": False,
        "recorded_replay_runtime_landed_claim": False,
        "dstrict_truth_claim": False,
        "dae_solver_current_line_claim": False,
        "adaptive_solver_current_line_claim": False,
        "external_solver_current_line_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")
    policy = manifest.get("dultra_policy", {})
    expected_policy = {
        "includes_solver_strategy_relaxation": True,
        "performance_expressivity_first": True,
        "recorded_replay_contract_required": True,
        "must_not_claim_dstrict_truth": True,
        "runtime_landed": False,
    }
    for key, value in expected_policy.items():
        if policy.get(key) != value:
            fail(f"dultra policy {key} expected {value!r}, got {policy.get(key)!r}")
    expected_strategies = [
        "DAE solver",
        "adaptive solver",
        "external numerical solver",
        "SIMD / Float32 backend",
        "engine-assisted preview solver",
    ]
    if manifest.get("candidate_solver_strategies") != expected_strategies:
        fail(f"candidate strategies mismatch: {manifest.get('candidate_solver_strategies')!r}")
    sections = manifest.get("required_contract_sections", [])
    section_ids = [row.get("id") for row in sections]
    if section_ids != REQUIRED_SECTIONS:
        fail(f"section ids mismatch: {section_ids!r}")
    for row in sections:
        if not row.get("required_fields"):
            fail(f"section has no fields: {row!r}")
        if not row.get("reason"):
            fail(f"section has no reason: {row!r}")
    stub = manifest.get("sample_recorded_replay_stub", {})
    for section in REQUIRED_SECTIONS:
        if section not in stub:
            fail(f"stub missing section: {section}")
    boundary = stub.get("claim_boundary", {})
    if boundary.get("dstrict_truth_claim") is not False:
        fail(f"stub must reject D-STRICT truth claim: {boundary!r}")
    if boundary.get("current_line_support_claim") is not False:
        fail(f"stub must reject current-line claim: {boundary!r}")
    allowed = set(manifest.get("allowed_claims", []))
    if "recorded_trace_can_be_replayed_or_audited_under_recorded_configuration" not in allowed:
        fail(f"allowed claims missing recorded trace replay/audit: {allowed!r}")
    required_forbidden = {
        "D_ULTRA_result_is_D_STRICT_bit_perfect_truth",
        "external_DAE_solver_is_current_line_core_solver",
        "adaptive_solver_replay_without_trace",
        "runtime_recorded_replay_implementation_landed",
        "stdlib_surface_landed",
        "parser_frontdoor_landed",
        "docs_ssot_edited_by_codex",
    }
    if set(manifest.get("forbidden_claims", [])) != required_forbidden:
        fail(f"forbidden claims mismatch: {manifest.get('forbidden_claims')!r}")
    for row in manifest.get("evidence_rows", []):
        path = ROOT / row.get("path", "")
        require(path)
        require_contains(path, row.get("tokens", []))
    required_blocked = {
        "docs_ssot_edit",
        "parser_frontdoor_change",
        "runtime_surface_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "product_ui_change",
        "dultra_solver_runtime_landed",
        "recorded_replay_runtime_landed",
        "dstrict_truth_claim",
        "dae_solver_current_line",
        "adaptive_solver_current_line",
        "external_solver_current_line",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")
    expected_plans = {
        "implementation_readiness_followup_plan": {"closed": 5, "total": 6, "percent": 83},
        "dultra_recorded_replay_contract_plan": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 52, "total": 90, "percent": 58},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"{key} mismatch: {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    solver = load_json(SOURCE_SOLVER)
    dultra = solver.get("dultra_policy", {})
    if dultra.get("includes_solver_strategy_relaxation") is not True:
        fail(f"source D-ULTRA policy mismatch: {dultra!r}")
    if dultra.get("replay_contract_required") is not True:
        fail(f"source D-ULTRA replay requirement mismatch: {dultra!r}")
    if dultra.get("must_not_claim_dstrict_truth") is not True:
        fail(f"source D-ULTRA truth boundary mismatch: {dultra!r}")
    if dultra.get("runtime_landed") is not False:
        fail(f"source D-ULTRA runtime must not be landed: {dultra!r}")
    if solver.get("recorded_replay_contract_fields") != SOURCE_RECORDED_FIELDS:
        fail(f"source recorded fields mismatch: {solver.get('recorded_replay_contract_fields')!r}")
    velocity = load_json(SOURCE_VELOCITY)
    if velocity.get("next_item") != "LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_V1":
        fail(f"velocity pack next mismatch: {velocity.get('next_item')!r}")
    if velocity.get("recorded_replay_contract_landed_claim") is not False:
        fail("velocity prior item must not claim recorded replay landed")
    require_contains(SOURCE_DULTRA, ["D-ULTRA", *SOURCE_RECORDED_FIELDS, "trace 없이 replay 가능하다고 주장"])
    require_contains(SOURCE_METHODS, ["RK45/adaptive", "D-FAST/D-ULTRA", "D-STRICT claim 금지", "recorded replay contract 필요"])
    require_contains(SOURCE_AI_DET, ["D-ULTRA", "auto-open license"])


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_dultra_recorded_replay_contract_v1"], timeout=240)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_previous_checker() -> None:
    proc = run([sys.executable, "tests/run_lang_velocity_verlet_fixed64_order_pack_check.py"], timeout=420)
    if proc.returncode != 0:
        fail(f"previous checker failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_pack_golden()
    check_previous_checker()
    require_docs_ssot_clean()
    print("lang_dultra_recorded_replay_contract_pack_check: PASS")


if __name__ == "__main__":
    main()
