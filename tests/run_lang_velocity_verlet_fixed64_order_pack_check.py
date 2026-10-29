from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_20260606.md"
PACK = ROOT / "pack" / "lang_velocity_verlet_fixed64_order_v1"
MANIFEST = PACK / "velocity_verlet_fixed64_order.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_velocity_verlet_fixed64_order_pack_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
SOURCE_SOLVER = ROOT / "pack" / "lang_dstrict_dultra_solver_strategy_proposal_v1" / "dstrict_dultra_solver_strategy_proposal.detjson"
SOURCE_CONNECT = ROOT / "pack" / "lang_connect_seum_lowering_parser_spike_v1" / "connect_seum_lowering_parser_spike.detjson"
SOURCE_METHODS = ROOT / "docs" / "context" / "proposals" / "PROPOSAL_DETERMINISTIC_SOLVER_METHODS_V1_20260524.md"
SOURCE_STDLIB = ROOT / "pack" / "stdlib_l1_integrators_v1" / "README.md"
SOURCE_CORE_FIXED64 = ROOT / "core" / "src" / "fixed64.rs"
SOURCE_TOOL_FIXED64 = ROOT / "tools" / "teul-cli" / "src" / "core" / "fixed64.rs"
NEXT = "LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_V1"

SCALE = 1 << 32
INT64_MIN = -(1 << 63)
INT64_MAX = (1 << 63) - 1


def fail(message: str) -> None:
    print(f"lang_velocity_verlet_fixed64_order_pack_check: FAIL: {message}", file=sys.stderr)
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


def sat64(value: int) -> int:
    return max(INT64_MIN, min(INT64_MAX, value))


def fixed_mul(lhs: int, rhs: int) -> int:
    return sat64((lhs * rhs) >> 32)


def fixed_add(lhs: int, rhs: int) -> int:
    return sat64(lhs + rhs)


def fixed_trace() -> dict[str, int]:
    x0 = SCALE
    v0 = 0
    dt = SCALE // 4
    dt_half = SCALE // 8
    a0 = -x0
    v_half = fixed_add(v0, fixed_mul(a0, dt_half))
    x1 = fixed_add(x0, fixed_mul(v_half, dt))
    a1 = -x1
    v1 = fixed_add(v_half, fixed_mul(a1, dt_half))
    energy0 = fixed_add(fixed_mul(x0, x0), fixed_mul(v0, v0))
    energy1 = fixed_add(fixed_mul(x1, x1), fixed_mul(v1, v1))
    explicit_v1 = fixed_add(v0, fixed_mul(a0, dt))
    explicit_x1 = fixed_add(x0, fixed_mul(v0, dt))
    explicit_energy1 = fixed_add(fixed_mul(explicit_x1, explicit_x1), fixed_mul(explicit_v1, explicit_v1))
    semi_v1 = explicit_v1
    semi_x1 = fixed_add(x0, fixed_mul(semi_v1, dt))
    semi_energy1 = fixed_add(fixed_mul(semi_x1, semi_x1), fixed_mul(semi_v1, semi_v1))
    return {
        "x0": x0,
        "v0": v0,
        "dt": dt,
        "dt_half": dt_half,
        "a0": a0,
        "v_half": v_half,
        "x1": x1,
        "a1": a1,
        "v1": v1,
        "energy0": energy0,
        "energy1": energy1,
        "energy_delta": energy1 - energy0,
        "explicit_euler_energy1": explicit_energy1,
        "semi_implicit_euler_energy1": semi_energy1,
    }


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
        SOURCE_CONNECT,
        SOURCE_METHODS,
        SOURCE_STDLIB,
        SOURCE_CORE_FIXED64,
        SOURCE_TOOL_FIXED64,
        ROOT / "tests" / "run_lang_connect_seum_lowering_parser_spike_check.py",
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_V1",
        "속도 베를레",
        "Q32.32",
        "v_half = v0 + a0 * dt_half",
        "x1 = x0 + v_half * dt",
        "v1 = v_half + a1 * dt_half",
        "energy1 = 0.9990386962890625",
        "No `속도 베를레` runtime landed claim",
        "언어 구현 준비 후속 계획: 4/6 = 67%",
        "Velocity Verlet Fixed64 order pack: 1/1 = 100%",
        "ROADMAP_V2 전체: queue-expanded 51/90 = 57%",
        NEXT,
    ]
    require_contains(DOC, tokens + ["docs/ssot/**", "No parser/frontdoor grammar change"])
    require_contains(PROPOSAL, tokens[:8] + ["No SSOT edit by Codex"])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "속도 베를레",
            "Q32.32 Fixed64",
            "energy1 = 0.9990386962890625",
            "No `적분.속도베를레` stdlib landed claim",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_V1",
            "ddn.language.velocity_verlet_fixed64_order.v1",
            "lang_velocity_verlet_fixed64_order_v1",
            "언어 구현 준비 후속 계획: 4/6 = 67%",
            "ROADMAP_V2 전체: queue-expanded 51/90 = 57%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_velocity_verlet_fixed64_order_v1",
        "kind": "lang_velocity_verlet_fixed64_order",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": False,
        "ssot_edit_claim": False,
        "velocity_verlet_fixed64_order_pack_claim": True,
        "velocity_verlet_runtime_landed_claim": False,
        "velocity_verlet_stdlib_landed_claim": False,
        "dae_solver_current_line_claim": False,
        "adaptive_solver_current_line_claim": False,
        "recorded_replay_contract_landed_claim": False,
        "closed_by": "LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_V1",
        "based_on": "LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1",
        "proposal_doc": "docs/context/proposals/LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_20260606.md",
        "decision_manifest": "pack/lang_velocity_verlet_fixed64_order_v1/velocity_verlet_fixed64_order.detjson",
        "source_dstrict_dultra_solver_strategy": "pack/lang_dstrict_dultra_solver_strategy_proposal_v1/dstrict_dultra_solver_strategy_proposal.detjson",
        "source_connect_seum_lowering_parser_spike": "pack/lang_connect_seum_lowering_parser_spike_v1/connect_seum_lowering_parser_spike.detjson",
        "recommended_dstrict_candidate": "속도 베를레",
        "current_dstrict_baseline": ["적분.오일러", "적분.반암시적오일러"],
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "language_design_queue_closed": 8,
        "language_design_queue_total": 8,
        "language_design_queue_percent": 100,
        "implementation_readiness_rebase_closed": 1,
        "implementation_readiness_rebase_total": 1,
        "implementation_readiness_rebase_percent": 100,
        "implementation_readiness_followup_closed": 4,
        "implementation_readiness_followup_total": 6,
        "implementation_readiness_followup_percent": 67,
        "velocity_verlet_fixed64_order_closed": 1,
        "velocity_verlet_fixed64_order_total": 1,
        "velocity_verlet_fixed64_order_percent": 100,
        "urgent_recommendations_closed": 3,
        "urgent_recommendations_total": 3,
        "urgent_recommendations_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 51,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 57,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.velocity_verlet_fixed64_order.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_V1":
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    expected_flags = {
        "velocity_verlet_fixed64_order_pack_claim": True,
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
        "recorded_replay_contract_landed_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("recommended_dstrict_candidate") != "속도 베를레":
        fail(f"candidate mismatch: {manifest.get('recommended_dstrict_candidate')!r}")
    if manifest.get("current_dstrict_baseline") != ["적분.오일러", "적분.반암시적오일러"]:
        fail(f"baseline mismatch: {manifest.get('current_dstrict_baseline')!r}")
    policy = manifest.get("fixed64_policy", {})
    expected_policy = {
        "format": "Q32.32",
        "scale_bits": 32,
        "scale_raw": SCALE,
        "multiply_intermediate": "i128",
        "multiply_rounding": "shift_right_32_once_per_multiplication",
        "addition": "saturating_integer_add",
        "subtraction": "saturating_integer_sub",
        "division": "checked_or_exact_raw_division_for_seed_dt_half",
        "float_runtime_claim": False,
    }
    for key, value in expected_policy.items():
        if policy.get(key) != value:
            fail(f"fixed64 policy {key} expected {value!r}, got {policy.get(key)!r}")
    expected_order = [
        "dt_half = dt / 2",
        "a0 = accel(x0)",
        "v_half = v0 + a0 * dt_half",
        "x1 = x0 + v_half * dt",
        "a1 = accel(x1)",
        "v1 = v_half + a1 * dt_half",
        "energy_indicator = x * x + v * v",
    ]
    formulas = [row.get("formula") for row in manifest.get("operation_order", [])]
    if formulas != expected_order:
        fail(f"operation order mismatch: {formulas!r}")
    trace = fixed_trace()
    sample = manifest.get("sample", {})
    for key in ["x0", "v0", "dt", "dt_half", "a0", "v_half", "x1", "a1", "v1", "energy0", "energy1", "energy_delta"]:
        row = sample.get(key, {})
        if row.get("raw") != trace[key]:
            fail(f"sample {key} raw expected {trace[key]}, got {row.get('raw')!r}")
    expected_decimals = {
        "x0": "1",
        "v0": "0",
        "dt": "0.25",
        "dt_half": "0.125",
        "a0": "-1",
        "v_half": "-0.125",
        "x1": "0.96875",
        "a1": "-0.96875",
        "v1": "-0.24609375",
        "energy0": "1",
        "energy1": "0.9990386962890625",
        "energy_delta": "-0.0009613037109375",
    }
    for key, value in expected_decimals.items():
        if sample.get(key, {}).get("decimal") != value:
            fail(f"sample {key} decimal expected {value!r}, got {sample.get(key, {}).get('decimal')!r}")
    comparison = manifest.get("comparison_smoke", {})
    expected_comparison = {
        "explicit_euler_energy1": trace["explicit_euler_energy1"],
        "semi_implicit_euler_energy1": trace["semi_implicit_euler_energy1"],
        "velocity_verlet_energy1": trace["energy1"],
    }
    for key, raw in expected_comparison.items():
        if comparison.get(key, {}).get("raw") != raw:
            fail(f"comparison {key} raw expected {raw}, got {comparison.get(key, {}).get('raw')!r}")
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
        "velocity_verlet_runtime_landed",
        "velocity_verlet_stdlib_landed",
        "recorded_replay_contract_landed",
        "dae_solver_current_line",
        "adaptive_solver_current_line",
        "external_solver_current_line",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")
    expected_plans = {
        "implementation_readiness_followup_plan": {"closed": 4, "total": 6, "percent": 67},
        "velocity_verlet_fixed64_order_plan": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 51, "total": 90, "percent": 57},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"{key} mismatch: {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    solver = load_json(SOURCE_SOLVER)
    policy = solver.get("dstrict_policy", {})
    if policy.get("recommended_next_candidate") != "속도 베를레":
        fail(f"solver candidate mismatch: {policy!r}")
    if policy.get("candidate_landed") is not False:
        fail(f"solver candidate must not be landed: {policy!r}")
    if policy.get("fixed64_required") is not True:
        fail(f"solver candidate must require Fixed64: {policy!r}")
    required_conditions = [
        "Fixed64 operation order",
        "position_velocity_acceleration_update_order",
        "a_t_and_a_t_plus_dt_evaluation_timing",
        "rounding_points",
        "golden_hash_and_energy_smoke_pack",
    ]
    if solver.get("velocity_verlet_landing_conditions") != required_conditions:
        fail(f"solver landing conditions mismatch: {solver.get('velocity_verlet_landing_conditions')!r}")
    connect = load_json(SOURCE_CONNECT)
    if connect.get("next_item") != "LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_V1":
        fail(f"connect spike next mismatch: {connect.get('next_item')!r}")
    require_contains(SOURCE_METHODS, ["속도 베를레", "D-STRICT 후보/권장", "Fixed64 연산 순서", "golden hash / energy smoke pack"])
    require_contains(SOURCE_STDLIB, ["적분.오일러", "적분.반암시적오일러"])
    require_contains(SOURCE_CORE_FIXED64, ["Fixed64", "saturating_mul", "i128", ">> 32"])
    require_contains(SOURCE_TOOL_FIXED64, ["Fixed64", "SCALE_BITS", "saturating_mul", "checked_div"])


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_velocity_verlet_fixed64_order_v1"], timeout=240)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_previous_checker() -> None:
    proc = run([sys.executable, "tests/run_lang_connect_seum_lowering_parser_spike_check.py"], timeout=420)
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
    print("lang_velocity_verlet_fixed64_order_pack_check: PASS")


if __name__ == "__main__":
    main()
