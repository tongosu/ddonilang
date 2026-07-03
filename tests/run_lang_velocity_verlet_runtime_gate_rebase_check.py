from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_VELOCITY_VERLET_RUNTIME_GATE_REBASE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_VELOCITY_VERLET_RUNTIME_GATE_REBASE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_VELOCITY_VERLET_RUNTIME_GATE_REBASE_20260606.md"
PACK = ROOT / "pack" / "lang_velocity_verlet_runtime_gate_rebase_v1"
MANIFEST = PACK / "velocity_verlet_runtime_gate_rebase.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_velocity_verlet_runtime_gate_rebase_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_ORDER = ROOT / "pack" / "lang_velocity_verlet_fixed64_order_v1" / "velocity_verlet_fixed64_order.detjson"
SOURCE_TUCK = ROOT / "pack" / "lang_tuck_constraint_surface_shape_proposal_v1" / "tuck_constraint_surface_shape_proposal.detjson"
PREVIOUS_CHECKER = ROOT / "tests" / "run_lang_tuck_constraint_surface_shape_proposal_check.py"
ORDER_CHECKER = ROOT / "tests" / "run_lang_velocity_verlet_fixed64_order_pack_check.py"

WORK_ITEM = "LANG_VELOCITY_VERLET_RUNTIME_GATE_REBASE_V1"
NEXT = "LANG_DULTRA_REPLAY_ARTIFACT_IMPLEMENTATION_GATE_V1"

GATES = [
    ("fixed64_order_pack", "closed"),
    ("stdlib_surface_acceptance", "blocked"),
    ("runtime_product_path", "planned_after_surface"),
    ("golden_raw_trace", "planned_after_runtime"),
    ("cli_wasm_parity", "planned_after_runtime"),
    ("compat_boundary", "planned_after_runtime"),
]

OP_ORDER_IDS = ["dt_half", "a0", "v_half", "x1", "a1", "v1", "energy_indicator"]


def fail(message: str) -> None:
    print(f"lang_velocity_verlet_runtime_gate_rebase_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_ORDER,
        SOURCE_TUCK,
        PREVIOUS_CHECKER,
        ORDER_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        "속도 베를레",
        "적분.속도베를레",
        "fixed64_order_pack",
        "stdlib_surface_acceptance",
        "runtime_product_path",
        "golden_raw_trace",
        "cli_wasm_parity",
        "compat_boundary",
        "No `docs/ssot/**` edit",
        "No `속도 베를레` runtime landed claim",
        "No `적분.속도베를레` stdlib landed claim",
        "다음 언어 구현 위험 제거 계획: 4/6 = 67%",
        "Velocity Verlet runtime gate rebase: 1/1 = 100%",
        "ROADMAP_V2 전체: queue-expanded 57/90 = 63%",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, "속도 베를레", "Required Gates", "4/6 = 67%", "57/90 = 63%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "runtime gate rebase",
            "Fixed64 operation order",
            "No runtime or stdlib surface change",
            "No `속도 베를레` runtime landed claim",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.velocity_verlet_runtime_gate_rebase.v1",
            "lang_velocity_verlet_runtime_gate_rebase_v1",
            "다음 언어 구현 위험 제거 계획: 4/6 = 67%",
            "Velocity Verlet runtime gate rebase: 1/1 = 100%",
            "ROADMAP_V2 전체: queue-expanded 57/90 = 63%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_velocity_verlet_runtime_gate_rebase_v1",
        "kind": "lang_velocity_verlet_runtime_gate_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "velocity_verlet_runtime_gate_rebase_claim": True,
        "velocity_verlet_runtime_landed_claim": False,
        "velocity_verlet_stdlib_landed_claim": False,
        "dae_solver_current_line_claim": False,
        "adaptive_solver_current_line_claim": False,
        "external_solver_current_line_claim": False,
        "dultra_replay_runtime_landed_claim": False,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_TUCK_CONSTRAINT_SURFACE_SHAPE_PROPOSAL_V1",
        "proposal_doc": "docs/context/proposals/LANG_VELOCITY_VERLET_RUNTIME_GATE_REBASE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_VELOCITY_VERLET_RUNTIME_GATE_REBASE_20260606.md",
        "decision_manifest": "pack/lang_velocity_verlet_runtime_gate_rebase_v1/velocity_verlet_runtime_gate_rebase.detjson",
        "source_velocity_verlet_fixed64_order": "pack/lang_velocity_verlet_fixed64_order_v1/velocity_verlet_fixed64_order.detjson",
        "source_tuck_constraint_surface_shape": "pack/lang_tuck_constraint_surface_shape_proposal_v1/tuck_constraint_surface_shape_proposal.detjson",
        "recommended_dstrict_candidate": "속도 베를레",
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
        "language_risk_removal_closed": 4,
        "language_risk_removal_total": 6,
        "language_risk_removal_percent": 67,
        "velocity_verlet_runtime_gate_rebase_closed": 1,
        "velocity_verlet_runtime_gate_rebase_total": 1,
        "velocity_verlet_runtime_gate_rebase_percent": 100,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 57,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 63,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.velocity_verlet_runtime_gate_rebase.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")

    expected_flags = {
        "velocity_verlet_runtime_gate_rebase_claim": True,
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "velocity_verlet_runtime_landed_claim": False,
        "velocity_verlet_stdlib_landed_claim": False,
        "dae_solver_current_line_claim": False,
        "adaptive_solver_current_line_claim": False,
        "external_solver_current_line_claim": False,
        "dultra_replay_runtime_landed_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")

    surfaces = manifest.get("candidate_public_surfaces", [])
    if [row.get("surface") for row in surfaces] != ["적분.속도베를레", "적분.속도_베를레"]:
        fail(f"candidate surfaces mismatch: {surfaces!r}")
    for row in surfaces:
        if row.get("status") != "candidate" or row.get("stdlib_landed") is not False:
            fail(f"candidate surface must stay non-landed: {row!r}")

    gates = manifest.get("runtime_gates", [])
    if len(gates) != len(GATES):
        fail(f"runtime gate count mismatch: {len(gates)}")
    for index, ((gate_id, status), row) in enumerate(zip(GATES, gates), start=1):
        if row.get("order") != index or row.get("id") != gate_id or row.get("status") != status:
            fail(f"runtime gate mismatch: {row!r}")
        if not row.get("required_evidence"):
            fail(f"runtime gate missing evidence: {row!r}")

    order_rows = manifest.get("required_operation_order", [])
    if [row.get("id") for row in order_rows] != OP_ORDER_IDS:
        fail(f"operation order mismatch: {order_rows!r}")
    for index, row in enumerate(order_rows, start=1):
        if row.get("step") != index or not row.get("formula"):
            fail(f"bad operation order row: {row!r}")

    expected_trace = {
        "sample_id": "harmonic_oscillator_one_tick",
        "x0_raw": 4294967296,
        "v0_raw": 0,
        "dt_raw": 1073741824,
        "dt_half_raw": 536870912,
        "v_half_raw": -536870912,
        "x1_raw": 4160749568,
        "a1_raw": -4160749568,
        "v1_raw": -1056964608,
        "energy1_raw": 4290838528,
    }
    if manifest.get("seed_trace_requirements") != expected_trace:
        fail(f"seed trace mismatch: {manifest.get('seed_trace_requirements')!r}")

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
        "velocity_verlet_runtime_landed",
        "velocity_verlet_stdlib_landed",
        "dae_solver_current_line",
        "adaptive_solver_current_line",
        "external_solver_current_line",
        "dultra_replay_runtime_landed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")

    expected_plans = {
        "language_risk_removal_plan": {"closed": 4, "total": 6, "percent": 67},
        "velocity_verlet_runtime_gate_rebase": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 57, "total": 90, "percent": 63},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"{key} mismatch: {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    order = load_json(SOURCE_ORDER)
    if order.get("recommended_dstrict_candidate") != "속도 베를레":
        fail(f"source order candidate mismatch: {order.get('recommended_dstrict_candidate')!r}")
    if order.get("velocity_verlet_runtime_landed_claim") is not False:
        fail("source order pack must not claim runtime landing")
    if order.get("velocity_verlet_stdlib_landed_claim") is not False:
        fail("source order pack must not claim stdlib landing")
    if [row.get("id") for row in order.get("operation_order", [])] != OP_ORDER_IDS:
        fail(f"source operation order mismatch: {order.get('operation_order')!r}")
    sample = order.get("sample", {})
    if sample.get("energy1", {}).get("raw") != 4290838528:
        fail(f"source sample energy mismatch: {sample!r}")

    tuck = load_json(SOURCE_TUCK)
    if tuck.get("next_item") != WORK_ITEM:
        fail(f"source tuck next mismatch: {tuck.get('next_item')!r}")
    if tuck.get("language_risk_removal_plan") != {"closed": 3, "total": 6, "percent": 50}:
        fail(f"source tuck risk progress mismatch: {tuck.get('language_risk_removal_plan')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected_stdout = [
        WORK_ITEM,
        "velocity verlet runtime gate rebase sealed",
        "schema: ddn.language.velocity_verlet_runtime_gate_rebase.v1",
        "candidate: 속도 베를레",
        "required runtime gates: 6",
        "risk removal: 4/6 = 67%",
        "runtime landed: false",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/lang_velocity_verlet_runtime_gate_rebase_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected_stdout:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_velocity_verlet_runtime_gate_rebase_v1"], timeout=240)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_previous_checkers() -> None:
    for checker in [PREVIOUS_CHECKER, ORDER_CHECKER]:
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
    print("lang_velocity_verlet_runtime_gate_rebase_check: PASS")


if __name__ == "__main__":
    main()
