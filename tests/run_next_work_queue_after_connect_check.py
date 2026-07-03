#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def require_files() -> int:
    required = [
        ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_CHECK_RUNNER_V1V.md",
        ROOT / "pack" / "connect_flow_v1v_closure_v1" / "contract.detjson",
        ROOT / "AGE5_CI_FRONTIER_RECHECK_V1.md",
        ROOT / "ROADMAP_V2_FOLLOWON_REBASE_V1.md",
        ROOT / "ROADMAP_V2_DA1_MATH_REBASE_V1.md",
        ROOT / "MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1.md",
        ROOT / "ROADMAP_V2_DA1_MATH_CLOSURE_V1.md",
        ROOT / "MATH_VECTOR_MINIMUM_FIRST_RUN_V1.md",
        ROOT / "pack" / "math_vector_minimum_first_run_v1" / "golden.jsonl",
        ROOT / "ROADMAP_V2_DA1_FINAL_CLOSURE_V1.md",
        ROOT / "ROADMAP_V2_POST_PRIORITY_REBASE_V1.md",
        ROOT / "ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1.md",
        ROOT / "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1.md",
        ROOT / "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1.md",
        ROOT / "ROADMAP_V2_LA2_FINAL_CLOSURE_V1.md",
        ROOT / "ROADMAP_V2_A1_NURIGYM_REBASE_V1.md",
        ROOT / "NURIGYM_DATASET_HASH_EXPECTED_REFRESH_V1.md",
        ROOT / "ROADMAP_V2_A1_NURIGYM_FINAL_CLOSURE_V1.md",
        ROOT / "ROADMAP_V2_A2_NURIGYM_REPRESENTATIVE_ENV_REBASE_V1.md",
        ROOT / "NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1.md",
        ROOT / "ROADMAP_V2_A2_NURIGYM_BANDIT_OR_FINAL_RECHECK_V1.md",
        ROOT / "NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1.md",
        ROOT / "NURIGYM_BANDIT_MINIMUM_PACK_V1.md",
        ROOT / "ROADMAP_V2_A2_NURIGYM_FINAL_CLOSURE_RECHECK_V1.md",
        ROOT / "ROOT_LOW_RISK_RETIRE_DELETE_PREFLIGHT_V1.md",
        ROOT / "tests" / "run_root_low_risk_retire_delete_preflight_check.py",
        ROOT / "ROOT_LOW_RISK_RETIRE_DELETE_DRY_RUN_PLAN_V1.md",
        ROOT / "tests" / "run_root_low_risk_retire_delete_dry_run_plan_check.py",
        ROOT / "ROOT_LOW_RISK_RETIRE_DELETE_APPROVAL_GATE_BLOCKED_V1.md",
        ROOT / "tests" / "run_root_low_risk_retire_delete_approval_gate_blocked_check.py",
        ROOT / "ROOT_LOW_RISK_RETIRE_DELETE_V1.md",
        ROOT / "tests" / "run_root_low_risk_retire_delete_check.py",
        ROOT / "docs" / "status" / "roadmap_v2" / "아-1_REPORT_20260604.md",
        ROOT / "docs" / "status" / "roadmap_v2" / "아-2_REBASE_20260604.md",
        ROOT / "docs" / "status" / "roadmap_v2" / "아-2_BANDIT_OR_FINAL_RECHECK_20260604.md",
        ROOT / "tests" / "run_roadmap_v2_a2_nurigym_rebase_check.py",
        ROOT / "tests" / "run_nurigym_contract_gridworld_expected_refresh_check.py",
        ROOT / "tests" / "run_roadmap_v2_a2_bandit_or_final_recheck.py",
        ROOT / "tests" / "run_nurigym_cartpole_pendulum_expected_refresh_check.py",
        ROOT / "tests" / "run_nurigym_bandit_minimum_pack_check.py",
        ROOT / "pack" / "gogae5_w47_nurigym_observation_spec" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_gridmaze_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_cartpole_shared_sync_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_canon_contract_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_gridworld_v1" / "golden.jsonl",
        ROOT / "pack" / "nuri_gym_bandit_v1" / "golden.jsonl",
        ROOT / "pack" / "seamgrim_malblock_roundtrip_subset_v1" / "expected" / "malblock_roundtrip_subset.detjson",
        ROOT / "NEXT_WORK_QUEUE_V3.md",
        ROOT / "NEXT_DEV_ROADMAP_AFTER_VIEWER_JS_DOM_V2.md",
        QUEUE,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_NEXT_WORK_QUEUE_AFTER_CONNECT_MISSING", str(missing))
    return 0


def check_queue_text() -> int:
    text = QUEUE.read_text(encoding="utf-8")
    required_tokens = [
        "CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_CHECK_RUNNER_V1V",
        "connect_flow_v1v_closure_v1",
        "std grid-game browser/live/rules minimum",
        "SEUM_ASSERTION_BLOCK_V1B",
        "connect endpoint work is closed",
        "NEXT_WORK_QUEUE_V3",
        "remains historical",
        "ROADMAP_V2_SA1_REBASE_V1",
        "ROADMAP_V2_NA2_UNIT_RANDOM_EVENT_REBASE_V1",
        "ROADMAP_V2_NA2_EVENT_MINIMUM_CLOSURE_REBASE_V1",
        "STD_EVENT_MINIMUM_CLOSURE_V1",
        "AGE5_CI_FRONTIER_RECHECK_V1",
        "profile-matrix full-real smoke policy selftests",
        "not a current blocker",
        "ROADMAP_V2_FOLLOWON_REBASE_V1",
        "do not start `거-1+`",
        "public registry final",
        "ROADMAP_V2_DA1_MATH_REBASE_V1",
        "math_calculus_v1",
        "MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1",
        "computed derivative/integral output",
        "ROADMAP_V2_DA1_MATH_CLOSURE_V1",
        "vector-specific first-run evidence",
        "MATH_VECTOR_MINIMUM_FIRST_RUN_V1",
        "closed by `MATH_VECTOR_MINIMUM_FIRST_RUN_V1.md`",
        "ROADMAP_V2_DA1_FINAL_CLOSURE_V1",
        "closed by `ROADMAP_V2_DA1_FINAL_CLOSURE_V1.md`",
        "ROADMAP_V2_POST_PRIORITY_REBASE_V1",
        "closed by `ROADMAP_V2_POST_PRIORITY_REBASE_V1.md`",
        "ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1",
        "closed by `ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1.md`",
        "LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1",
        "closed by `LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1.md`",
        "pack/lang_core_2_v1",
        "ROADMAP_V2_GA2_FINAL_CLOSURE_V1",
        "closed by `ROADMAP_V2_GA2_FINAL_CLOSURE_V1.md`",
        "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1",
        "closed by `ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1.md`",
        "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1",
        "closed by `BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1.md`",
        "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1",
        "closed by `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1.md`",
        "pack/seamgrim_malblock_roundtrip_subset_v1",
        "canonical equality",
        "raw/opaque blocks",
        "ROADMAP_V2_LA2_FINAL_CLOSURE_V1",
        "closed by `ROADMAP_V2_LA2_FINAL_CLOSURE_V1.md`",
        "ROADMAP_V2_A1_NURIGYM_REBASE_V1",
        "closed by `ROADMAP_V2_A1_NURIGYM_REBASE_V1.md`",
        "NURIGYM_DATASET_HASH_EXPECTED_REFRESH_V1",
        "closed by `NURIGYM_DATASET_HASH_EXPECTED_REFRESH_V1.md`",
        "nuri_gym_gridmaze_v1",
        "nuri_gym_cartpole_shared_sync_v1",
        "ROADMAP_V2_A1_NURIGYM_FINAL_CLOSURE_V1",
        "closed by `ROADMAP_V2_A1_NURIGYM_FINAL_CLOSURE_V1.md`",
        "ROADMAP_V2 `아-1` is closed",
        "ROADMAP_V2_A2_NURIGYM_REPRESENTATIVE_ENV_REBASE_V1",
        "closed by `ROADMAP_V2_A2_NURIGYM_REPRESENTATIVE_ENV_REBASE_V1.md`",
        "ROADMAP_V2 `아-2` remains open",
        "nuri_gym_canon_contract_v1",
        "nuri_gym_gridworld_v1",
        "NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1",
        "closed by `NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1.md`",
        "tests/run_nuri_gym_contract_check.py",
        "ROADMAP_V2_A2_NURIGYM_BANDIT_OR_FINAL_RECHECK_V1",
        "closed by `ROADMAP_V2_A2_NURIGYM_BANDIT_OR_FINAL_RECHECK_V1.md`",
        "NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1",
        "closed by `NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1.md`",
        "nuri_gym_cartpole_v1",
        "nuri_gym_pendulum_v1",
        "NURIGYM_BANDIT_MINIMUM_PACK_V1",
        "closed by `NURIGYM_BANDIT_MINIMUM_PACK_V1.md`",
        "nuri_gym_bandit_v1",
        "ROADMAP_V2_A2_NURIGYM_FINAL_CLOSURE_RECHECK_V1",
        "closed by `ROADMAP_V2_A2_NURIGYM_FINAL_CLOSURE_RECHECK_V1.md`",
        "ROADMAP_V2 `아-2` is closed",
        "ROOT_LOW_RISK_RETIRE_DELETE_PREFLIGHT_V1",
        "closed by `ROOT_LOW_RISK_RETIRE_DELETE_PREFLIGHT_V1.md`",
        "without deleting, moving, retiring, or path-rewriting any file",
        "ROOT_LOW_RISK_RETIRE_DELETE_DRY_RUN_PLAN_V1",
        "closed by `ROOT_LOW_RISK_RETIRE_DELETE_DRY_RUN_PLAN_V1.md`",
        "file-level deletion candidate inventory",
        "approval-gated PowerShell command shape",
        "ROOT_LOW_RISK_RETIRE_DELETE_APPROVAL_GATE_BLOCKED_V1",
        "closed by `ROOT_LOW_RISK_RETIRE_DELETE_APPROVAL_GATE_BLOCKED_V1.md`",
        "blocked_waiting_for_explicit_delete_approval",
        "repeated next-development requests do not override",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "closed by `ROOT_LOW_RISK_RETIRE_DELETE_V1.md`",
        "deleted only the 9 file-level candidates",
        "preserved mirror/pack counterparts",
        "No automatic next development item is selected",
        "가-2",
        "대표 문법 pack",
        "라-2",
        "DDN -> 말블록 subset roundtrip",
        "superseded",
        "docs/ssot/**",
    ]
    missing = [token for token in required_tokens if token not in text]
    if missing:
        return fail("E_NEXT_WORK_QUEUE_AFTER_CONNECT_TOKENS", str(missing))

    forbidden_phrases = [
        "connect_flow_v1w_closure_v1",
        "pack/connect_flow_v1w",
        "tests/run_connect_flow_v1w",
    ]
    present = [phrase for phrase in forbidden_phrases if phrase in text]
    if present:
        return fail("E_NEXT_WORK_QUEUE_AFTER_CONNECT_V1W_IMPL", str(present))

    if "No `connect_flow_v1w_*` implementation" not in text:
        return fail("E_NEXT_WORK_QUEUE_AFTER_CONNECT_V1W_POLICY", "missing connect v1w defer policy")
    if "ROADMAP_V2_SA1_IMPLEMENTATION_V1` is superseded" not in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_SA1_REBASE",
            "missing SA1 superseded decision",
        )
    if "unit and random axes are already closed" not in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_NA2_REBASE",
            "missing NA2 unit/random/event rebase decision",
        )
    if "full actor-event semantics remain deferred" not in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_EVENT_REBASE",
            "missing event minimum closure decision",
        )
    if "closed by `AGE5_CI_FRONTIER_RECHECK_V1.md`" not in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_AGE5_RECHECK",
            "missing AGE5 CI frontier recheck closure decision",
        )
    if "1. `AGE5_CI_FRONTIER_RECHECK_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_AGE5_OPEN",
            "AGE5 recheck is still listed as the next open queue item",
        )
    if "closed by `ROADMAP_V2_FOLLOWON_REBASE_V1.md`" not in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_FOLLOWON_REBASE",
            "missing ROADMAP V2 follow-on rebase closure decision",
        )
    if "1. `ROADMAP_V2_FOLLOWON_REBASE_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_FOLLOWON_OPEN",
            "ROADMAP V2 follow-on rebase is still listed as the next open item",
        )
    if "closed by `ROADMAP_V2_DA1_MATH_REBASE_V1.md`" not in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_DA1_REBASE",
            "missing ROADMAP V2 DA1 math rebase closure decision",
        )
    if "1. `ROADMAP_V2_DA1_MATH_REBASE_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_DA1_OPEN",
            "ROADMAP V2 DA1 math rebase is still listed as the next open item",
        )
    if "closed by `MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1.md`" not in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_MATH_CALCULUS_REFRESH",
            "missing math calculus computed output refresh closure decision",
        )
    if "1. `MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_MATH_CALCULUS_OPEN",
            "math calculus refresh is still listed as the next open item",
        )
    if "closed by `ROADMAP_V2_DA1_MATH_CLOSURE_V1.md`" not in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_DA1_CLOSURE",
            "missing ROADMAP V2 DA1 math closure decision",
        )
    if "1. `ROADMAP_V2_DA1_MATH_CLOSURE_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_DA1_CLOSURE_OPEN",
            "DA1 math closure is still listed as the next open item",
        )
    if "closed by `MATH_VECTOR_MINIMUM_FIRST_RUN_V1.md`" not in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_VECTOR_CLOSED",
            "missing math vector minimum first-run closure decision",
        )
    if "1. `MATH_VECTOR_MINIMUM_FIRST_RUN_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_VECTOR_OPEN",
            "MATH_VECTOR_MINIMUM_FIRST_RUN_V1 is still listed as the next open item",
        )
    if "closed by `ROADMAP_V2_DA1_FINAL_CLOSURE_V1.md`" not in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_DA1_FINAL_CLOSED",
            "missing ROADMAP V2 DA1 final closure decision",
        )
    if "1. `ROADMAP_V2_DA1_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_DA1_FINAL_OPEN",
            "ROADMAP_V2_DA1_FINAL_CLOSURE_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_POST_PRIORITY_REBASE_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_POST_PRIORITY_OPEN",
            "ROADMAP_V2_POST_PRIORITY_REBASE_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_GA2_OPEN",
            "ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1 is still listed as the next open item",
        )
    if "1. `LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_LANG_CORE_2_OPEN",
            "LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_GA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_GA2_FINAL_OPEN",
            "ROADMAP_V2_GA2_FINAL_CLOSURE_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_LA2_REBASE_OPEN",
            "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1 is still listed as the next open item",
        )
    if "1. `BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_BLOCK_EDITOR_REFRESH_OPEN",
            "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1 is still listed as the next open item",
        )
    if "1. `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_MALBLOCK_ROUNDTRIP_NEXT",
            "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_LA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_LA2_FINAL_NEXT",
            "ROADMAP_V2_LA2_FINAL_CLOSURE_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_A1_NURIGYM_REBASE_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_A1_REBASE_OPEN",
            "ROADMAP_V2_A1_NURIGYM_REBASE_V1 is still listed as the next open item",
        )
    if "1. `NURIGYM_DATASET_HASH_EXPECTED_REFRESH_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_NURIGYM_HASH_OPEN",
            "NURIGYM_DATASET_HASH_EXPECTED_REFRESH_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_A2_NURIGYM_REPRESENTATIVE_ENV_REBASE_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_A2_REBASE_OPEN",
            "ROADMAP_V2_A2_NURIGYM_REPRESENTATIVE_ENV_REBASE_V1 is still listed as the next open item",
        )
    if "1. `NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_NURIGYM_CONTRACT_REFRESH_OPEN",
            "NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_A2_NURIGYM_BANDIT_OR_FINAL_RECHECK_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_A2_BANDIT_RECHECK_OPEN",
            "ROADMAP_V2_A2_NURIGYM_BANDIT_OR_FINAL_RECHECK_V1 is still listed as the next open item",
        )
    if "1. `NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_NURIGYM_CARTPOLE_PENDULUM_OPEN",
            "NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1 is still listed as the next open item",
        )
    if "1. `NURIGYM_BANDIT_MINIMUM_PACK_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_NURIGYM_BANDIT_OPEN",
            "NURIGYM_BANDIT_MINIMUM_PACK_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_A2_NURIGYM_FINAL_CLOSURE_RECHECK_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_A2_FINAL_OPEN",
            "ROADMAP_V2_A2_NURIGYM_FINAL_CLOSURE_RECHECK_V1 is still listed as the next open item",
        )
    if "1. `ROOT_LOW_RISK_RETIRE_DELETE_V1`" in text:
        return fail(
            "E_NEXT_WORK_QUEUE_AFTER_CONNECT_ROOT_DELETE_OPEN",
            "ROOT_LOW_RISK_RETIRE_DELETE_V1 is still listed as the next open item",
        )
    if "No automatic next development item is selected" not in text:
        return fail("E_NEXT_WORK_QUEUE_AFTER_CONNECT_NO_NEXT", "missing no-next queue seal")
    return 0


def check_historical_queue_unchanged_policy() -> int:
    text = QUEUE.read_text(encoding="utf-8")
    if "without editing that historical document" not in text:
        return fail("E_NEXT_WORK_QUEUE_AFTER_CONNECT_HISTORICAL", "NEXT_WORK_QUEUE_V3 policy missing")
    return 0


def check_docs_ssot_clean() -> int:
    result = subprocess.run(
        ["git", "status", "--short", "--", "docs/ssot"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0:
        return fail("E_NEXT_WORK_QUEUE_AFTER_CONNECT_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_NEXT_WORK_QUEUE_AFTER_CONNECT_SSOT_DIRTY", result.stdout.strip())
    return 0


def check_reference_docs() -> int:
    dev_summary = (ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md").read_text(
        encoding="utf-8"
    )
    project_status = (ROOT / "docs" / "status" / "PROJECT_STATUS.md").read_text(encoding="utf-8")
    for token in [
        "CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_CHECK_RUNNER_V1V",
        "AGE5_CI_FRONTIER_RECHECK_V1",
        "ROADMAP_V2_FOLLOWON_REBASE_V1",
        "ROADMAP_V2_DA1_MATH_REBASE_V1",
        "MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1",
        "ROADMAP_V2_DA1_MATH_CLOSURE_V1",
        "MATH_VECTOR_MINIMUM_FIRST_RUN_V1",
        "ROADMAP_V2_DA1_FINAL_CLOSURE_V1",
        "ROADMAP_V2_POST_PRIORITY_REBASE_V1",
        "ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1",
        "LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1",
        "ROADMAP_V2_GA2_FINAL_CLOSURE_V1",
        "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1",
        "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1",
        "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1",
        "ROADMAP_V2_LA2_FINAL_CLOSURE_V1",
        "ROADMAP_V2_A1_NURIGYM_REBASE_V1",
        "NURIGYM_DATASET_HASH_EXPECTED_REFRESH_V1",
        "ROADMAP_V2_A1_NURIGYM_FINAL_CLOSURE_V1",
        "ROADMAP_V2_A2_NURIGYM_REPRESENTATIVE_ENV_REBASE_V1",
        "NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1",
        "ROADMAP_V2_A2_NURIGYM_BANDIT_OR_FINAL_RECHECK_V1",
        "NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1",
        "NURIGYM_BANDIT_MINIMUM_PACK_V1",
        "ROADMAP_V2_A2_NURIGYM_FINAL_CLOSURE_RECHECK_V1",
        "ROOT_LOW_RISK_RETIRE_DELETE_PREFLIGHT_V1",
        "ROOT_LOW_RISK_RETIRE_DELETE_DRY_RUN_PLAN_V1",
        "ROOT_LOW_RISK_RETIRE_DELETE_APPROVAL_GATE_BLOCKED_V1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "No automatic next development item is selected",
        "사-1",
        "ROADMAP_V2",
    ]:
        if token not in dev_summary and token not in project_status:
            return fail("E_NEXT_WORK_QUEUE_AFTER_CONNECT_REFERENCE_TOKEN", token)
    return 0


def main() -> int:
    checks = (
        require_files,
        check_queue_text,
        check_historical_queue_unchanged_policy,
        check_reference_docs,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[next-work-queue-after-connect-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
