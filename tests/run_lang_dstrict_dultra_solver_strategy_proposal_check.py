from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_DSTRICT_DULTRA_SOLVER_STRATEGY_PROPOSAL_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_DSTRICT_DULTRA_SOLVER_STRATEGY_PROPOSAL_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_DSTRICT_DULTRA_SOLVER_STRATEGY_PROPOSAL_20260606.md"
PACK = ROOT / "pack" / "lang_dstrict_dultra_solver_strategy_proposal_v1"
MANIFEST = PACK / "dstrict_dultra_solver_strategy_proposal.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_dstrict_dultra_solver_strategy_proposal_check.py"
SOURCE_CONNECT = ROOT / "pack" / "lang_connect_lowering_to_seum_check_v1" / "connect_lowering_to_seum_check.detjson"
SOURCE_SOLVER = ROOT / "docs" / "context" / "proposals" / "PROPOSAL_DETERMINISTIC_SOLVER_METHODS_V1_20260524.md"
SOURCE_DULTRA = ROOT / "docs" / "context" / "proposals" / "PROPOSAL_DULTRA_SOLVER_STRATEGY_RECORDED_CONTRACT_V1_20260524.md"
SOURCE_L1 = ROOT / "pack" / "stdlib_l1_integrators_v1"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
NEXT = "LANG_IMPLEMENTATION_READINESS_REBASE_V1"


def fail(message: str) -> None:
    print(f"lang_dstrict_dultra_solver_strategy_proposal_check: FAIL: {message}", file=sys.stderr)
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


def required_tokens() -> list[str]:
    return [
        "LANG_DSTRICT_DULTRA_SOLVER_STRATEGY_PROPOSAL_V1",
        "속도 베를레",
        "D-STRICT",
        "D-ULTRA",
        "Fixed64",
        "recorded replay",
        "DAE",
        "adaptive",
        "external",
        "적분.오일러",
        "적분.반암시적오일러",
        NEXT,
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
        SOURCE_CONNECT,
        SOURCE_SOLVER,
        SOURCE_DULTRA,
        SOURCE_L1 / "README.md",
        ROOT / "tests" / "run_lang_connect_lowering_to_seum_check.py",
        DEV_SUMMARY,
    ]:
        require(path)


def check_docs() -> None:
    require_contains(DOC, required_tokens() + ["docs/ssot/**", "새 언어 설계 안정화 계획: 8/8 = 100%", "긴급 언어 결정 SSOT 반영: 0/3 = 0%"])
    require_contains(PROPOSAL, required_tokens())
    require_contains(
        SSOT_NOTE,
        [
            "속도 베를레",
            "D-ULTRA",
            "recorded replay",
            "No parser/runtime landed claim",
            "No DAE/adaptive/external solver current-line claim",
            "Codex did not edit `docs/ssot/**`",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "LANG_DSTRICT_DULTRA_SOLVER_STRATEGY_PROPOSAL_V1",
            "lang_dstrict_dultra_solver_strategy_proposal_v1",
            "ddn.language.dstrict_dultra_solver_strategy_proposal.v1",
            "새 언어 설계 안정화 계획: 8/8 = 100%",
            "긴급 언어 결정 evidence closure: 3/3 = 100%",
            "긴급 언어 결정 SSOT 반영: 0/3 = 0%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_dstrict_dultra_solver_strategy_proposal_v1",
        "kind": "lang_dstrict_dultra_solver_strategy_proposal",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "solver_strategy_proposal_claim": True,
        "velocity_verlet_runtime_landed_claim": False,
        "dultra_solver_runtime_landed_claim": False,
        "dae_solver_current_line_claim": False,
        "recorded_replay_contract_landed_claim": False,
        "recommended_dstrict_candidate": "속도 베를레",
        "current_dstrict_baseline": ["적분.오일러", "적분.반암시적오일러"],
        "closed_by": "LANG_DSTRICT_DULTRA_SOLVER_STRATEGY_PROPOSAL_V1",
        "based_on": "LANG_CONNECT_LOWERING_TO_SEUM_CHECK_V1",
        "proposal_doc": "docs/context/proposals/LANG_DSTRICT_DULTRA_SOLVER_STRATEGY_PROPOSAL_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_DSTRICT_DULTRA_SOLVER_STRATEGY_PROPOSAL_20260606.md",
        "decision_manifest": "pack/lang_dstrict_dultra_solver_strategy_proposal_v1/dstrict_dultra_solver_strategy_proposal.detjson",
        "source_connect_lowering": "pack/lang_connect_lowering_to_seum_check_v1/connect_lowering_to_seum_check.detjson",
        "source_deterministic_solver_methods": "docs/context/proposals/PROPOSAL_DETERMINISTIC_SOLVER_METHODS_V1_20260524.md",
        "source_dultra_recorded_contract": "docs/context/proposals/PROPOSAL_DULTRA_SOLVER_STRATEGY_RECORDED_CONTRACT_V1_20260524.md",
        "source_l1_integrators": "pack/stdlib_l1_integrators_v1",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "language_design_queue_closed": 8,
        "language_design_queue_total": 8,
        "language_design_queue_percent": 100,
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
    if manifest.get("schema") != "ddn.language.dstrict_dultra_solver_strategy_proposal.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "LANG_DSTRICT_DULTRA_SOLVER_STRATEGY_PROPOSAL_V1":
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
        "velocity_verlet_runtime_landed_claim",
        "dultra_solver_runtime_landed_claim",
        "dae_solver_current_line_claim",
        "recorded_replay_contract_landed_claim",
    ]:
        if manifest.get(flag) is not False:
            fail(f"manifest {flag} expected false, got {manifest.get(flag)!r}")
    dstrict = manifest.get("dstrict_policy", {})
    if dstrict.get("recommended_next_candidate") != "속도 베를레":
        fail(f"D-STRICT candidate mismatch: {dstrict!r}")
    if dstrict.get("current_baseline") != ["적분.오일러", "적분.반암시적오일러"]:
        fail(f"D-STRICT baseline mismatch: {dstrict!r}")
    if dstrict.get("candidate_landed") is not False or dstrict.get("fixed64_required") is not True:
        fail(f"D-STRICT landed/fixed64 flags mismatch: {dstrict!r}")
    required_conditions = [
        "Fixed64 operation order",
        "position_velocity_acceleration_update_order",
        "a_t_and_a_t_plus_dt_evaluation_timing",
        "rounding_points",
        "golden_hash_and_energy_smoke_pack",
    ]
    if manifest.get("velocity_verlet_landing_conditions") != required_conditions:
        fail(f"velocity verlet conditions mismatch: {manifest.get('velocity_verlet_landing_conditions')!r}")
    dultra = manifest.get("dultra_policy", {})
    if dultra != {
        "includes_solver_strategy_relaxation": True,
        "replay_contract_required": True,
        "must_not_claim_dstrict_truth": True,
        "runtime_landed": False,
    }:
        fail(f"D-ULTRA policy mismatch: {dultra!r}")
    if manifest.get("queue_plan") != {"closed": 8, "total": 8, "percent": 100}:
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
        "velocity_verlet_runtime_landed",
        "dultra_solver_runtime_landed",
        "dae_solver_current_line",
        "recorded_replay_contract_landed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")


def check_source_alignment() -> None:
    connect = load_json(SOURCE_CONNECT)
    if connect.get("schema") != "ddn.language.connect_lowering_to_seum_check.v1":
        fail(f"source connect schema mismatch: {connect.get('schema')!r}")
    if connect.get("next_item") != "LANG_DSTRICT_DULTRA_SOLVER_STRATEGY_PROPOSAL_V1":
        fail(f"source connect next item mismatch: {connect.get('next_item')!r}")
    if connect.get("queue_plan") != {"closed": 7, "total": 8, "percent": 88}:
        fail(f"source connect queue mismatch: {connect.get('queue_plan')!r}")
    require_contains(SOURCE_SOLVER, ["속도 베를레", "D-STRICT 후보/권장", "Fixed64 연산 순서", "golden hash / energy smoke pack"])
    require_contains(SOURCE_DULTRA, ["D-ULTRA", "DAE solver", "adaptive solver", "recorded replay contract", "solver id / version / configuration"])
    require_contains(SOURCE_L1 / "README.md", ["적분.오일러", "적분.반암시적오일러"])
    require_contains(SOURCE_L1 / "input.ddn", ["적분.오일러", "적분.반암시적오일러"])


def check_evidence_rows() -> None:
    manifest = load_json(MANIFEST)
    for row in manifest.get("evidence_rows", []):
        path = ROOT / row["path"]
        require(path)
        require_contains(path, list(row["tokens"]))


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "LANG_DSTRICT_DULTRA_SOLVER_STRATEGY_PROPOSAL_V1",
        "dstrict dultra solver strategy proposal sealed",
        "solver strategy schema: ddn.language.dstrict_dultra_solver_strategy_proposal.v1",
        "recommended D-STRICT candidate: 속도 베를레",
        "language queue: 8/8 = 100%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/lang_dstrict_dultra_solver_strategy_proposal_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "lang_dstrict_dultra_solver_strategy_proposal_v1"],
        ["python", "tests/run_lang_connect_lowering_to_seum_check.py"],
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
    check_evidence_rows()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("lang_dstrict_dultra_solver_strategy_proposal_check: ok")


if __name__ == "__main__":
    main()
