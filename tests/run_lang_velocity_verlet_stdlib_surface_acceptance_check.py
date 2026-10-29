from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_VELOCITY_VERLET_STDLIB_SURFACE_ACCEPTANCE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_VELOCITY_VERLET_STDLIB_SURFACE_ACCEPTANCE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_VELOCITY_VERLET_STDLIB_SURFACE_ACCEPTANCE_20260606.md"
PACK = ROOT / "pack" / "lang_velocity_verlet_stdlib_surface_acceptance_v1"
MANIFEST = PACK / "velocity_verlet_stdlib_surface_acceptance.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_velocity_verlet_stdlib_surface_acceptance_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
STDLIB = ROOT / "lang" / "src" / "stdlib.rs"

SOURCE_GATE = ROOT / "pack" / "lang_velocity_verlet_runtime_gate_rebase_v1" / "velocity_verlet_runtime_gate_rebase.detjson"
SOURCE_ORDER = ROOT / "pack" / "lang_velocity_verlet_fixed64_order_v1" / "velocity_verlet_fixed64_order.detjson"
SOURCE_DULTRA = ROOT / "pack" / "lang_dultra_replay_artifact_writer_seed_v1" / "dultra_replay_artifact_writer_seed.detjson"
DULTRA_CHECKER = ROOT / "tests" / "run_lang_dultra_replay_artifact_writer_seed_check.py"
GATE_CHECKER = ROOT / "tests" / "run_lang_velocity_verlet_runtime_gate_rebase_check.py"

WORK_ITEM = "LANG_VELOCITY_VERLET_STDLIB_SURFACE_ACCEPTANCE_V1"
NEXT = "LANG_TUCK_SSOT_ACCEPTANCE_HANDOFF_V1"
CANONICAL = "적분.속도베를레"
ALIAS = "적분.속도_베를레"
PARAMS = ["위치", "속도", "가속도", "다음가속도", "dt"]
RET = "(다음위치, 다음속도)"


def fail(message: str) -> None:
    print(f"lang_velocity_verlet_stdlib_surface_acceptance_check: FAIL: {message}", file=sys.stderr)
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
        STDLIB,
        SOURCE_GATE,
        SOURCE_ORDER,
        SOURCE_DULTRA,
        DULTRA_CHECKER,
        GATE_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        WORK_ITEM,
        CANONICAL,
        ALIAS,
        RET,
        "runtime_claim`: false",
        "velocity_verlet_stdlib_surface_landed_claim`: true",
        "velocity_verlet_runtime_landed_claim`: false",
        "언어 제품 경로 구현 전환 계획: `6/7 = 86%`",
        "Velocity Verlet stdlib surface acceptance: `1/1 = 100%`",
        "ROADMAP_V2 전체: `queue-expanded 65/90 = 72%`",
        NEXT,
    ]
    require_contains(DOC, tokens)
    require_contains(PROPOSAL, [WORK_ITEM, CANONICAL, ALIAS, "6/7 = 86%", "65/90 = 72%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            CANONICAL,
            ALIAS,
            "not runtime landed behavior",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.velocity_verlet_stdlib_surface_acceptance.v1",
            "lang_velocity_verlet_stdlib_surface_acceptance_v1",
            "언어 제품 경로 구현 전환 계획: 6/7 = 86%",
            "Velocity Verlet stdlib surface acceptance: 1/1 = 100%",
            "ROADMAP_V2 전체: queue-expanded 65/90 = 72%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_stdlib_surface() -> None:
    require_contains(
        STDLIB,
        [
            CANONICAL,
            ALIAS,
            "canonicalize_stdlib_alias",
            "velocity_verlet_stdlib_surface_acceptance_v1_signature_is_present",
            "다음가속도",
            RET,
        ],
    )
    text = read(STDLIB)
    if f'"{ALIAS}" => "{CANONICAL}"' not in text:
        fail("stdlib alias canonicalization missing")


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_velocity_verlet_stdlib_surface_acceptance_v1",
        "kind": "lang_velocity_verlet_stdlib_surface_acceptance",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": True,
        "ssot_edit_claim": False,
        "velocity_verlet_stdlib_surface_landed_claim": True,
        "velocity_verlet_runtime_landed_claim": False,
        "velocity_verlet_cli_wasm_parity_claim": False,
        "dae_solver_current_line_claim": False,
        "adaptive_solver_current_line_claim": False,
        "external_solver_current_line_claim": False,
        "dultra_replay_runtime_landed_claim": False,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_DULTRA_REPLAY_ARTIFACT_WRITER_SEED_V1",
        "proposal_doc": "docs/context/proposals/LANG_VELOCITY_VERLET_STDLIB_SURFACE_ACCEPTANCE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_VELOCITY_VERLET_STDLIB_SURFACE_ACCEPTANCE_20260606.md",
        "decision_manifest": "pack/lang_velocity_verlet_stdlib_surface_acceptance_v1/velocity_verlet_stdlib_surface_acceptance.detjson",
        "source_velocity_verlet_runtime_gate_rebase": "pack/lang_velocity_verlet_runtime_gate_rebase_v1/velocity_verlet_runtime_gate_rebase.detjson",
        "source_velocity_verlet_fixed64_order": "pack/lang_velocity_verlet_fixed64_order_v1/velocity_verlet_fixed64_order.detjson",
        "source_dultra_replay_artifact_writer_seed": "pack/lang_dultra_replay_artifact_writer_seed_v1/dultra_replay_artifact_writer_seed.detjson",
        "language_product_path_transition_closed": 6,
        "language_product_path_transition_total": 7,
        "language_product_path_transition_percent": 86,
        "velocity_verlet_stdlib_surface_acceptance_closed": 1,
        "velocity_verlet_stdlib_surface_acceptance_total": 1,
        "velocity_verlet_stdlib_surface_acceptance_percent": 100,
        "urgent_evidence_closed": 3,
        "urgent_evidence_total": 3,
        "urgent_evidence_percent": 100,
        "urgent_ssot_landed_closed": 0,
        "urgent_ssot_landed_total": 3,
        "urgent_ssot_landed_percent": 0,
        "roadmap_v2_queue_expanded_closed": 65,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 72,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    for source_key in [
        "source_velocity_verlet_runtime_gate_rebase",
        "source_velocity_verlet_fixed64_order",
        "source_dultra_replay_artifact_writer_seed",
    ]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.velocity_verlet_stdlib_surface_acceptance.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")

    expected_flags = {
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "stdlib_surface_change": True,
        "ssot_edit_claim": False,
        "velocity_verlet_stdlib_surface_landed_claim": True,
        "velocity_verlet_runtime_landed_claim": False,
        "velocity_verlet_cli_wasm_parity_claim": False,
        "dae_solver_current_line_claim": False,
        "adaptive_solver_current_line_claim": False,
        "external_solver_current_line_claim": False,
        "dultra_replay_runtime_landed_claim": False,
    }
    for key, value in expected_flags.items():
        if manifest.get(key) != value:
            fail(f"manifest {key} expected {value!r}, got {manifest.get(key)!r}")

    surfaces = manifest.get("accepted_surfaces", [])
    expected_surfaces = {
        CANONICAL: {"status": "accepted", "runtime_landed": False, "stdlib_landed_now": True},
        ALIAS: {"status": "accepted_alias", "runtime_landed": False, "stdlib_landed_now": True},
    }
    for surface, expected in expected_surfaces.items():
        row = next((item for item in surfaces if item.get("surface") == surface), None)
        if row is None:
            fail(f"missing accepted surface: {surface}")
        for key, value in expected.items():
            if row.get(key) != value:
                fail(f"surface {surface} {key} expected {value!r}, got {row.get(key)!r}")
    if next(row for row in surfaces if row["surface"] == ALIAS).get("alias_for") != CANONICAL:
        fail("alias surface does not point at canonical surface")

    signature = manifest.get("signature", {})
    if signature.get("canonical") != CANONICAL or signature.get("alias") != ALIAS:
        fail(f"signature surface mismatch: {signature!r}")
    if signature.get("params") != PARAMS:
        fail(f"signature params mismatch: {signature.get('params')!r}")
    if signature.get("ret") != RET:
        fail(f"signature ret mismatch: {signature.get('ret')!r}")

    if [row.get("id") for row in manifest.get("required_operation_order", [])] != [
        "dt_half",
        "a0",
        "v_half",
        "x1",
        "a1",
        "v1",
        "energy_indicator",
    ]:
        fail("required operation order mismatch")

    runtime_boundary = manifest.get("runtime_boundary", {})
    for key in ["teul_cli_runtime_arm_landed", "tool_runtime_arm_landed", "cli_wasm_parity_landed"]:
        if runtime_boundary.get(key) is not False:
            fail(f"runtime boundary {key} must be false")

    for row in manifest.get("product_anchor_rows", []):
        path = ROOT / row.get("path", "")
        require(path)
        require_contains(path, row.get("tokens", []))

    required_blocked = {
        "docs_ssot_edit",
        "parser_frontdoor_change",
        "runtime_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
        "product_ui_change",
        "velocity_verlet_runtime_landed",
        "velocity_verlet_cli_wasm_parity_landed",
        "dae_solver_current_line",
        "adaptive_solver_current_line",
        "external_solver_current_line",
        "dultra_replay_runtime_landed",
    }
    if set(manifest.get("blocked_claims", [])) != required_blocked:
        fail(f"blocked claims mismatch: {manifest.get('blocked_claims')!r}")

    expected_plans = {
        "language_product_path_transition_plan": {"closed": 6, "total": 7, "percent": 86},
        "velocity_verlet_stdlib_surface_acceptance": {"closed": 1, "total": 1, "percent": 100},
        "urgent_evidence_plan": {"closed": 3, "total": 3, "percent": 100},
        "urgent_ssot_landed_plan": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 65, "total": 90, "percent": 72},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    gate = load_json(SOURCE_GATE)
    candidate_surfaces = {row.get("surface") for row in gate.get("candidate_public_surfaces", [])}
    if {CANONICAL, ALIAS} - candidate_surfaces:
        fail(f"source gate missing candidate surfaces: {candidate_surfaces!r}")
    if gate.get("velocity_verlet_runtime_landed_claim") is not False:
        fail("source gate runtime claim must remain false")

    order = load_json(SOURCE_ORDER)
    if order.get("recommended_dstrict_candidate") != "속도 베를레":
        fail(f"order pack candidate mismatch: {order.get('recommended_dstrict_candidate')!r}")
    if [row.get("id") for row in order.get("operation_order", [])] != [
        "dt_half",
        "a0",
        "v_half",
        "x1",
        "a1",
        "v1",
        "energy_indicator",
    ]:
        fail("order pack operation order mismatch")

    dultra = load_json(SOURCE_DULTRA)
    if dultra.get("next_item") != WORK_ITEM:
        fail(f"D-ULTRA writer seed next item expected {WORK_ITEM}, got {dultra.get('next_item')!r}")
    if dultra.get("language_product_path_transition_plan") != {"closed": 5, "total": 7, "percent": 71}:
        fail(f"D-ULTRA writer seed transition progress mismatch: {dultra.get('language_product_path_transition_plan')!r}")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_velocity_verlet_stdlib_surface_acceptance_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    expected = [
        WORK_ITEM,
        CANONICAL,
        f"{ALIAS} -> {CANONICAL}",
        "params: 위치, 속도, 가속도, 다음가속도, dt",
        "product transition: 6/7 = 86%",
        "runtime landed: false",
        NEXT,
    ]
    require_contains(PACK / "golden.jsonl", expected)


def check_product_tests() -> None:
    commands = [
        ["cargo", "test", "-p", "ddonirang-lang", "velocity_verlet_stdlib_surface_acceptance_v1_signature_is_present", "--quiet"],
        ["cargo", "test", "-p", "ddonirang-lang", "canonicalize_stdlib_aliases_map_to_single_canonical_names", "--quiet"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=300)
        if proc.returncode != 0:
            fail(f"product test failed ({' '.join(cmd)}):\n{proc.stdout}")


def check_previous_checkers() -> None:
    for checker in [DULTRA_CHECKER, GATE_CHECKER]:
        proc = run([sys.executable, str(checker.relative_to(ROOT))], timeout=1200)
        if proc.returncode != 0:
            fail(f"{checker.relative_to(ROOT)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_stdlib_surface()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_golden()
    check_product_tests()
    check_previous_checkers()
    require_docs_ssot_clean()
    print("lang_velocity_verlet_stdlib_surface_acceptance_check: PASS")


if __name__ == "__main__":
    main()

