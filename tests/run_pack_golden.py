#!/usr/bin/env python
import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path
import re
from datetime import datetime, timezone
import time

DIALECT_SMOKE_PACK = "lang_dialect_smoke_v1"
DIALECT_BUILTIN_EQUIV_PACK = "dialect_builtin_equiv_v1"


def iter_packs(root: Path, names: list[str], use_all: bool) -> list[Path]:
    pack_root = root / "pack"
    if names:
        return [pack_root / name for name in names]
    if use_all:
        return sorted({p.parent for p in pack_root.rglob("golden.jsonl")})
    default_packs = [
        pack_root / "stdlib_text_basics",
        pack_root / "stdlib_charim_basics",
        pack_root / "stdlib_range_basics",
        pack_root / "stdlib_math_basics",
        pack_root / "stdlib_map_basics",
        pack_root / "edu_math_function_line",
        pack_root / "edu_math_parabola",
        pack_root / "edu_math_stats",
        pack_root / "edu_physics_motion",
        pack_root / "edu_physics_energy",
        pack_root / "edu_chem_reaction",
        pack_root / "edu_algo_loop",
        pack_root / "edu_algo_search",
        pack_root / "edu_data_struct",
        pack_root / "edu_ddn_mastercourse/L01",
        pack_root / "edu_ddn_mastercourse/L02",
        pack_root / "edu_ddn_mastercourse/L03",
        pack_root / "compound_update_basics",
        pack_root / "diag_contract_mode_v1",
        pack_root / "math_calculus_v1",
        pack_root / "tensor_stdlib_phase0",
        pack_root / "w25_all_block_sugar",
        pack_root / "input_key_alias_ko_v1",
        pack_root / "bogae_bg_key_v1",
        pack_root / "bogae_drawlist_listkey_v1",
        pack_root / "bdl2_subpixel_aa_v1",
        pack_root / "age1_charim_index",
        pack_root / "age2_kernel_smoke_v1",
        pack_root / "project_age_target_precedence_v1",
        pack_root / "lang_ddn_meta_header_basics",
        pack_root / DIALECT_SMOKE_PACK,
        pack_root / DIALECT_BUILTIN_EQUIV_PACK,
        pack_root / "lang_range_literal_v0",
        pack_root / "lang_regex_literal_v1",
        pack_root / "gogae8_w81_reward_v1",
        pack_root / "gogae8_w82_seulgi_train_v2",
        pack_root / "gogae8_w83_edu_accuracy_v1",
        pack_root / "gogae8_w84_swarm_deterministic_collision",
        pack_root / "gogae8_w85_ondevice_infer_v1",
        pack_root / "gogae8_w86_imitation_learning_v1",
        pack_root / "gogae8_w87_eval_suite_v1",
        pack_root / "gogae8_w88_bundle_hash_parity",
        pack_root / "gogae9_w91_malmoi_docset",
        pack_root / "geoul_min_schema_v0",
        pack_root / "seamgrim_smoke_golden_v1",
        pack_root / "seamgrim_curriculum_seed_smoke_v1",
        pack_root / "seamgrim_curriculum_rewrite_core_smoke_v1",
        pack_root / "seamgrim_curriculum_batch_smoke_v1",
        pack_root / "seamgrim_curriculum_rewrite_sample_smoke_v1",
        pack_root / "seamgrim_state_hash_view_boundary_smoke_v1",
        pack_root / "seamgrim_overlay_param_compare_v0",
        pack_root / "seamgrim_overlay_session_roundtrip_v0",
        pack_root / "bogae_api_catalog_v1_basic",
        pack_root / "bogae_api_catalog_v1_graph",
        pack_root / "bogae_api_catalog_v1_game_hud",
        pack_root / "bogae_api_catalog_v1_errors",
        pack_root / "dotbogi_ddn_interface_v1_smoke",
        pack_root / "dotbogi_ddn_interface_v1_event_roundtrip",
        pack_root / "dotbogi_ddn_interface_v1_write_forbidden",
        pack_root / "eco_diag_convergence_smoke",
        pack_root / "eco_macro_micro_runner_smoke",
        pack_root / "eco_network_flow_smoke",
        pack_root / "eco_abm_spatial_smoke",
        pack_root / "eco_stats_stdlib_smoke",
    ]
    negative_packs = sorted(
        {
            path.parent
            for path in pack_root.rglob("golden.jsonl")
            if path.parent.name.endswith("_negative")
        }
    )
    seen = set(default_packs)
    for pack in negative_packs:
        if pack not in seen:
            default_packs.append(pack)
            seen.add(pack)
    return default_packs


def load_cases(pack_dir: Path) -> list[dict]:
    golden_path = pack_dir / "golden.jsonl"
    lines = golden_path.read_text(encoding="utf-8").splitlines()
    cases = []
    for idx, line in enumerate(lines, 1):
        if not line.strip():
            continue
        try:
            data = json.loads(line)
        except json.JSONDecodeError as exc:
            raise ValueError(f"{golden_path} line {idx}: {exc}")
        has_core_expectation = (
            "stdout" in data
            or "stdout_path" in data
            or "bogae_hash" in data
            or "expected_error_code" in data
            or "expected_warning_code" in data
            or "smoke_golden" in data
        )
        has_graph_expectation = "fixture" in data and "expected_graph" in data
        has_dotbogi_expectation = "dotbogi_case" in data
        has_overlay_compare_expectation = "overlay_compare_case" in data
        has_overlay_session_expectation = "overlay_session_case" in data
        if (
            not has_core_expectation
            and not has_graph_expectation
            and not has_dotbogi_expectation
            and not has_overlay_compare_expectation
            and not has_overlay_session_expectation
        ):
            raise ValueError(
                f"{golden_path} line {idx}: missing stdout/stdout_path, bogae_hash, expected_error_code, expected_warning_code, smoke_golden, fixture/expected_graph, dotbogi_case, overlay_compare_case, or overlay_session_case"
            )
        if "stdout" in data and not isinstance(data["stdout"], list):
            raise ValueError(f"{golden_path} line {idx}: stdout must be a list")
        if "stdout_path" in data and not isinstance(data["stdout_path"], str):
            raise ValueError(f"{golden_path} line {idx}: stdout_path must be a string")
        if "bogae_hash" in data and not isinstance(data["bogae_hash"], str):
            raise ValueError(f"{golden_path} line {idx}: bogae_hash must be a string")
        if "stderr" in data and not isinstance(data["stderr"], list):
            raise ValueError(f"{golden_path} line {idx}: stderr must be a list")
        if "meta_out" in data and not isinstance(data["meta_out"], str):
            raise ValueError(f"{golden_path} line {idx}: meta_out must be a string")
        if "expected_meta" in data and not isinstance(data["expected_meta"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_meta must be a string")
        if "expected_error_code" in data and not isinstance(data["expected_error_code"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_error_code must be a string")
        if "expected_warning_code" in data and not isinstance(data["expected_warning_code"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_warning_code must be a string")
        if "smoke_golden" in data and not isinstance(data["smoke_golden"], str):
            raise ValueError(f"{golden_path} line {idx}: smoke_golden must be a string")
        if "fixture" in data and "expected_graph" not in data:
            raise ValueError(f"{golden_path} line {idx}: fixture requires expected_graph")
        if "fixture" in data and not isinstance(data["fixture"], str):
            raise ValueError(f"{golden_path} line {idx}: fixture must be a string")
        if "expected_graph" in data and not isinstance(data["expected_graph"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_graph must be a string")
        if "dotbogi_case" in data and not isinstance(data["dotbogi_case"], str):
            raise ValueError(f"{golden_path} line {idx}: dotbogi_case must be a string")
        if "overlay_compare_case" in data and not isinstance(data["overlay_compare_case"], str):
            raise ValueError(f"{golden_path} line {idx}: overlay_compare_case must be a string")
        if "overlay_session_case" in data and not isinstance(data["overlay_session_case"], str):
            raise ValueError(f"{golden_path} line {idx}: overlay_session_case must be a string")
        if "expected_dotbogi_output" in data and not isinstance(data["expected_dotbogi_output"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_dotbogi_output must be a string")
        if "expected_view_meta_hash" in data and not isinstance(data["expected_view_meta_hash"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_view_meta_hash must be a string")
        if "expected_after_state" in data and not isinstance(data["expected_after_state"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_after_state must be a string")
        if "expected_after_state_hash" in data and not isinstance(data["expected_after_state_hash"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_after_state_hash must be a string")
        if "expected_contract" in data and not isinstance(data["expected_contract"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_contract must be a string")
        if "expected_detmath_seal_hash" in data and not isinstance(data["expected_detmath_seal_hash"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_detmath_seal_hash must be a string")
        if "expected_nuri_lock_hash" in data and not isinstance(data["expected_nuri_lock_hash"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_nuri_lock_hash must be a string")
        if "exit_code" in data and not isinstance(data["exit_code"], int):
            raise ValueError(f"{golden_path} line {idx}: exit_code must be an int")
        validate_case_mode_exclusivity(golden_path, idx, data)
        validate_run_case_contract(golden_path, idx, data)
        validate_eco_case_contract(pack_dir, golden_path, idx, data)
        validate_dotbogi_case_contract(pack_dir, golden_path, idx, data)
        validate_overlay_compare_case_contract(pack_dir, golden_path, idx, data)
        validate_overlay_session_case_contract(pack_dir, golden_path, idx, data)
        cases.append(data)
    return cases


def validate_case_mode_exclusivity(golden_path: Path, line_no: int, data: dict) -> None:
    has_smoke = isinstance(data.get("smoke_golden"), str) and bool(str(data.get("smoke_golden")).strip())
    has_dotbogi = isinstance(data.get("dotbogi_case"), str) and bool(str(data.get("dotbogi_case")).strip())
    has_graph = ("fixture" in data) or ("expected_graph" in data)
    has_overlay_compare = isinstance(data.get("overlay_compare_case"), str) and bool(
        str(data.get("overlay_compare_case")).strip()
    )
    has_overlay_session = isinstance(data.get("overlay_session_case"), str) and bool(
        str(data.get("overlay_session_case")).strip()
    )

    mode_count = int(has_smoke) + int(has_dotbogi) + int(has_graph) + int(has_overlay_compare) + int(has_overlay_session)
    if mode_count > 1:
        raise ValueError(
            f"{golden_path} line {line_no}: smoke_golden, dotbogi_case, fixture/expected_graph, overlay_compare_case, overlay_session_case modes are mutually exclusive"
        )

    if has_smoke:
        for key in ("dotbogi_case", "fixture", "expected_graph", "cmd"):
            if key in data:
                raise ValueError(
                    f"{golden_path} line {line_no}: smoke_golden case must not define '{key}'"
                )

    if has_dotbogi:
        for key in ("smoke_golden", "fixture", "expected_graph", "cmd", "stdout", "stdout_path", "bogae_hash"):
            if key in data:
                raise ValueError(
                    f"{golden_path} line {line_no}: dotbogi_case must not define '{key}'"
                )

    if has_graph:
        if "fixture" not in data or "expected_graph" not in data:
            raise ValueError(
                f"{golden_path} line {line_no}: graph case requires both fixture and expected_graph"
            )
        for key in ("smoke_golden", "dotbogi_case", "overlay_compare_case", "overlay_session_case", "cmd", "stdout", "stdout_path", "bogae_hash"):
            if key in data:
                raise ValueError(
                    f"{golden_path} line {line_no}: graph case must not define '{key}'"
                )

    if has_overlay_compare:
        for key in ("smoke_golden", "dotbogi_case", "fixture", "expected_graph", "overlay_session_case", "cmd", "stdout", "stdout_path", "bogae_hash"):
            if key in data:
                raise ValueError(
                    f"{golden_path} line {line_no}: overlay_compare_case must not define '{key}'"
                )

    if has_overlay_session:
        for key in ("smoke_golden", "dotbogi_case", "fixture", "expected_graph", "overlay_compare_case", "cmd", "stdout", "stdout_path", "bogae_hash"):
            if key in data:
                raise ValueError(
                    f"{golden_path} line {line_no}: overlay_session_case must not define '{key}'"
                )


def validate_run_case_contract(golden_path: Path, line_no: int, data: dict) -> None:
    cmd = data.get("cmd")
    if not isinstance(cmd, list) or not cmd:
        return
    argv = [str(item) for item in cmd]
    if argv[0] != "run":
        return

    if len(argv) < 2 or argv[1].startswith("-"):
        raise ValueError(
            f"{golden_path} line {line_no}: run cmd requires input path as second argument"
        )

    if "stdout" in data and "stdout_path" in data:
        raise ValueError(
            f"{golden_path} line {line_no}: stdout and stdout_path cannot be used together"
        )

    if ("meta_out" in data) ^ ("expected_meta" in data):
        raise ValueError(
            f"{golden_path} line {line_no}: meta_out and expected_meta must be provided together"
        )

    exit_code = int(data.get("exit_code", 0))
    expected_error = data.get("expected_error_code")
    if expected_error is not None:
        if exit_code == 0:
            raise ValueError(
                f"{golden_path} line {line_no}: expected_error_code case must have non-zero exit_code"
            )
        if any(key in data for key in ("stdout", "stdout_path", "bogae_hash")):
            raise ValueError(
                f"{golden_path} line {line_no}: expected_error_code case must not set stdout/stdout_path/bogae_hash"
            )
    else:
        if exit_code != 0:
            raise ValueError(
                f"{golden_path} line {line_no}: non-zero exit_code requires expected_error_code"
            )

    if "expected_warning_code" in data and "expected_error_code" in data:
        raise ValueError(
            f"{golden_path} line {line_no}: expected_warning_code and expected_error_code are mutually exclusive"
        )


def validate_eco_case_contract(pack_dir: Path, golden_path: Path, line_no: int, data: dict) -> None:
    if not pack_dir.name.startswith("eco_"):
        return
    cmd = data.get("cmd")
    if not isinstance(cmd, list) or not cmd:
        return
    argv = [str(item) for item in cmd]
    if argv[0] != "eco":
        return
    if len(argv) < 2:
        raise ValueError(f"{golden_path} line {line_no}: eco cmd must include subcommand")
    subcommand = argv[1]
    if subcommand not in {"macro-micro", "network-flow", "abm-spatial"}:
        raise ValueError(
            f"{golden_path} line {line_no}: unsupported eco subcommand '{subcommand}'"
        )

    expected_error = data.get("expected_error_code")
    exit_code = int(data.get("exit_code", 0))
    has_stdout_expectation = ("stdout" in data) or ("stdout_path" in data)
    has_meta_expectation = ("meta_out" in data) or ("expected_meta" in data)

    if expected_error is not None:
        if exit_code == 0:
            raise ValueError(
                f"{golden_path} line {line_no}: expected_error_code case must have non-zero exit_code"
            )
        if has_stdout_expectation:
            raise ValueError(
                f"{golden_path} line {line_no}: expected_error_code case must not set stdout/stdout_path"
            )
        if ("meta_out" in data) ^ ("expected_meta" in data):
            raise ValueError(
                f"{golden_path} line {line_no}: expected_error_code case meta_out/expected_meta must be provided together"
            )
    else:
        if exit_code != 0:
            raise ValueError(
                f"{golden_path} line {line_no}: eco success case must use exit_code=0"
            )
        if not has_stdout_expectation:
            raise ValueError(
                f"{golden_path} line {line_no}: eco success case must set stdout or stdout_path"
            )
        if not has_meta_expectation:
            raise ValueError(
                f"{golden_path} line {line_no}: eco success case must set meta_out and expected_meta"
            )
        if ("meta_out" in data) ^ ("expected_meta" in data):
            raise ValueError(
                f"{golden_path} line {line_no}: meta_out and expected_meta must be provided together"
            )

    def has_flag(flag: str) -> bool:
        return any(arg == flag or arg.startswith(f"{flag}=") for arg in argv[2:])

    if subcommand == "macro-micro":
        if len(argv) < 3 or not argv[2].endswith(".json"):
            raise ValueError(
                f"{golden_path} line {line_no}: eco macro-micro expects runner.json argument"
            )
        if not has_flag("--out"):
            raise ValueError(
                f"{golden_path} line {line_no}: eco macro-micro case requires --out"
            )
    if subcommand in {"network-flow", "abm-spatial"}:
        for flag in ("--madi", "--seed", "--out"):
            if not has_flag(flag):
                raise ValueError(
                    f"{golden_path} line {line_no}: eco {subcommand} case requires {flag}"
                )
    if subcommand == "network-flow" and not has_flag("--threshold"):
        raise ValueError(
            f"{golden_path} line {line_no}: eco network-flow case requires --threshold"
        )


def validate_dotbogi_case_contract(pack_dir: Path, golden_path: Path, line_no: int, data: dict) -> None:
    if "dotbogi_case" not in data:
        return
    dotbogi_case = data.get("dotbogi_case")
    if not isinstance(dotbogi_case, str) or not dotbogi_case.strip():
        raise ValueError(f"{golden_path} line {line_no}: dotbogi_case must be non-empty string")

    expected_error = data.get("expected_error_code")
    expected_output = data.get("expected_dotbogi_output")
    expected_view_meta_hash = data.get("expected_view_meta_hash")
    expected_after_state = data.get("expected_after_state")
    expected_after_state_hash = data.get("expected_after_state_hash")
    exit_code = int(data.get("exit_code", 0))

    if expected_error is not None:
        if expected_output is not None:
            raise ValueError(
                f"{golden_path} line {line_no}: dotbogi expected_error_code case must not set expected_dotbogi_output"
            )
        if expected_view_meta_hash is not None:
            raise ValueError(
                f"{golden_path} line {line_no}: dotbogi expected_error_code case must not set expected_view_meta_hash"
            )
        if expected_after_state is not None or expected_after_state_hash is not None:
            raise ValueError(
                f"{golden_path} line {line_no}: dotbogi expected_error_code case must not set expected_after_state/expected_after_state_hash"
            )
        return

    if exit_code != 0:
        raise ValueError(
            f"{golden_path} line {line_no}: dotbogi success case must use exit_code=0"
        )
    if not isinstance(expected_output, str) or not expected_output.strip():
        raise ValueError(
            f"{golden_path} line {line_no}: dotbogi success case must set expected_dotbogi_output"
        )
    if expected_view_meta_hash is not None and expected_output is None:
        raise ValueError(
            f"{golden_path} line {line_no}: expected_view_meta_hash requires expected_dotbogi_output"
        )
    if expected_after_state is not None and expected_output is None:
        raise ValueError(
            f"{golden_path} line {line_no}: expected_after_state requires expected_dotbogi_output"
        )
    if expected_after_state_hash is not None and expected_after_state is None:
        raise ValueError(
            f"{golden_path} line {line_no}: expected_after_state_hash requires expected_after_state"
        )


def validate_overlay_compare_case_contract(pack_dir: Path, golden_path: Path, line_no: int, data: dict) -> None:
    if "overlay_compare_case" not in data:
        return
    rel = data.get("overlay_compare_case")
    if not isinstance(rel, str) or not rel.strip():
        raise ValueError(f"{golden_path} line {line_no}: overlay_compare_case must be non-empty string")
    if "exit_code" in data and int(data.get("exit_code", 0)) != 0:
        raise ValueError(f"{golden_path} line {line_no}: overlay_compare_case supports exit_code=0 only")


def validate_overlay_session_case_contract(pack_dir: Path, golden_path: Path, line_no: int, data: dict) -> None:
    if "overlay_session_case" not in data:
        return
    rel = data.get("overlay_session_case")
    if not isinstance(rel, str) or not rel.strip():
        raise ValueError(f"{golden_path} line {line_no}: overlay_session_case must be non-empty string")
    if "exit_code" in data and int(data.get("exit_code", 0)) != 0:
        raise ValueError(f"{golden_path} line {line_no}: overlay_session_case supports exit_code=0 only")
    case_path = (pack_dir / rel).resolve()
    if not case_path.exists():
        raise ValueError(f"{golden_path} line {line_no}: overlay_session_case file not found: {rel}")
    try:
        case_doc = json.loads(case_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError(f"{golden_path} line {line_no}: overlay_session_case JSON parse failed: {rel}: {exc}") from exc
    if not isinstance(case_doc, dict):
        raise ValueError(f"{golden_path} line {line_no}: overlay_session_case doc must be object: {rel}")
    if str(case_doc.get("schema", "")) != "ddn.seamgrim.overlay_session_case.v1":
        raise ValueError(
            f"{golden_path} line {line_no}: overlay_session_case schema must be ddn.seamgrim.overlay_session_case.v1: {rel}"
        )
    expect = case_doc.get("expect")
    if not isinstance(expect, dict):
        raise ValueError(f"{golden_path} line {line_no}: overlay_session_case expect must be object: {rel}")
    expected_view_combo = expect.get("view_combo")
    if expected_view_combo is not None:
        if not isinstance(expected_view_combo, dict):
            raise ValueError(f"{golden_path} line {line_no}: expect.view_combo must be object: {rel}")
        if "enabled" not in expected_view_combo:
            raise ValueError(f"{golden_path} line {line_no}: expect.view_combo.enabled is required: {rel}")
        if not isinstance(expected_view_combo.get("enabled"), bool):
            raise ValueError(f"{golden_path} line {line_no}: expect.view_combo.enabled must be bool: {rel}")
        layout = expected_view_combo.get("layout")
        if not isinstance(layout, str) or not layout:
            raise ValueError(f"{golden_path} line {line_no}: expect.view_combo.layout must be non-empty string: {rel}")
        if layout not in {"horizontal", "vertical", "overlay"}:
            raise ValueError(
                f"{golden_path} line {line_no}: expect.view_combo.layout must be one of horizontal|vertical|overlay: {rel}"
            )
        overlay_order = expected_view_combo.get("overlay_order")
        if not isinstance(overlay_order, str) or not overlay_order:
            raise ValueError(
                f"{golden_path} line {line_no}: expect.view_combo.overlay_order must be non-empty string: {rel}"
            )
        if overlay_order not in {"graph", "space2d"}:
            raise ValueError(
                f"{golden_path} line {line_no}: expect.view_combo.overlay_order must be one of graph|space2d: {rel}"
            )


def parse_age_target_rank(text: str | None) -> int:
    if not text:
        return 0
    match = re.match(r"^AGE(\d+)$", str(text).strip().upper())
    if not match:
        return 0
    try:
        return int(match.group(1))
    except ValueError:
        return 0


def load_root_run_policy(root: Path) -> dict:
    policy = {
        "age_target_rank": 0,
        "trace_tier": None,
        "det_tier": None,
        "detmath_seal_hash": None,
        "nuri_lock_hash": None,
    }
    project_path = root / "ddn.project.json"
    if not project_path.exists():
        return policy
    try:
        data = json.loads(project_path.read_text(encoding="utf-8"))
    except Exception:
        return policy
    policy["age_target_rank"] = parse_age_target_rank(data.get("age_target"))
    trace_tier = data.get("trace_tier")
    if isinstance(trace_tier, str) and trace_tier.strip():
        policy["trace_tier"] = trace_tier.strip()
    det_tier = data.get("det_tier")
    if isinstance(det_tier, str) and det_tier.strip():
        policy["det_tier"] = det_tier.strip()
    detmath_seal_hash = data.get("detmath_seal_hash")
    if isinstance(detmath_seal_hash, str) and detmath_seal_hash.strip():
        policy["detmath_seal_hash"] = detmath_seal_hash.strip()
    nuri_lock_hash = data.get("nuri_lock_hash")
    if isinstance(nuri_lock_hash, str) and nuri_lock_hash.strip():
        policy["nuri_lock_hash"] = nuri_lock_hash.strip()
    return policy


def normalize_stderr_line(line: str, root: Path) -> str:
    root_str = str(root)
    root_posix = root.as_posix()
    if root_str in line:
        line = line.replace(root_str, "")
    if root_posix in line:
        line = line.replace(root_posix, "")
    line = line.replace("\\", "/")
    if line.startswith("/"):
        line = line[1:]
    line = line.replace(" /", " ")
    return line


def split_expected(text: str) -> list[str]:
    if not text:
        return []
    parts = [part.strip() for part in text.split("|")]
    return [part for part in parts if part]


def matches_any_expected(line: str, expected: list[str]) -> bool:
    if not expected:
        return True
    return line in expected


def extract_bogae_hash(lines: list[str]) -> str | None:
    for line in lines:
        text = line.strip()
        if text.startswith("bogae_hash="):
            value = text[len("bogae_hash="):].split()[0]
            return value
    return None


def has_cli_option(args: list[str], name: str) -> bool:
    for arg in args:
        text = str(arg)
        if text == name or text.startswith(f"{name}="):
            return True
    return False


def should_apply_default_trace_tier(command_args: list[str], run_policy: dict) -> bool:
    if not command_args:
        return False
    if str(command_args[0]) != "run":
        return False
    if has_cli_option(command_args, "--trace-tier"):
        return False
    trace_tier = run_policy.get("trace_tier")
    if not trace_tier:
        return False
    return int(run_policy.get("age_target_rank", 0)) >= 2


def map_contract_from_policy(run_policy: dict) -> str:
    det = str(run_policy.get("det_tier") or "").strip().upper()
    if det == "D-STRICT":
        return "D-STRICT"
    if det:
        return "D-APPROX"
    return ""


def load_smoke_golden_doc(root: Path, pack_dir: Path, relative_path: str) -> tuple[Path, dict]:
    doc_path = (pack_dir / relative_path).resolve()
    if not doc_path.exists():
        raise ValueError(f"{pack_dir}: smoke_golden file not found: {relative_path}")
    try:
        doc = json.loads(doc_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError(f"{doc_path}: JSON parse failed: {exc}") from exc
    if str(doc.get("schema", "")) != "ddn.smoke.golden.v1":
        raise ValueError(f"{doc_path}: schema must be ddn.smoke.golden.v1")
    ddn_file = doc.get("ddn_file")
    if not isinstance(ddn_file, str) or not ddn_file.strip():
        raise ValueError(f"{doc_path}: ddn_file must be non-empty string")
    checkpoints = doc.get("checkpoints")
    if not isinstance(checkpoints, list) or not checkpoints:
        raise ValueError(f"{doc_path}: checkpoints must be non-empty list")
    for idx, cp in enumerate(checkpoints, 1):
        if not isinstance(cp, dict):
            raise ValueError(f"{doc_path}: checkpoints[{idx}] must be object")
        tick = cp.get("tick")
        state_hash = cp.get("state_hash")
        if not isinstance(tick, int) or tick < 0:
            raise ValueError(f"{doc_path}: checkpoints[{idx}].tick must be non-negative int")
        if not isinstance(state_hash, str) or not state_hash.strip():
            raise ValueError(f"{doc_path}: checkpoints[{idx}].state_hash must be non-empty string")
    max_ticks = doc.get("max_ticks")
    if max_ticks is not None and (not isinstance(max_ticks, int) or max_ticks < 0):
        raise ValueError(f"{doc_path}: max_ticks must be non-negative int")
    timeout_ms = doc.get("timeout_ms")
    if timeout_ms is not None and (not isinstance(timeout_ms, int) or timeout_ms <= 0):
        raise ValueError(f"{doc_path}: timeout_ms must be positive int")
    if not doc_path.is_relative_to(root):
        raise ValueError(f"{doc_path}: smoke_golden file must be under workspace root")
    return doc_path, doc


def extract_last_state_hash(lines: list[str]) -> str | None:
    for raw in reversed(lines):
        line = raw.strip()
        if not line.startswith("state_hash="):
            continue
        value = line.split("=", 1)[1].strip().split()[0]
        if value:
            return value
    return None


def run_smoke_golden_case(
    root: Path,
    manifest: Path,
    pack_dir: Path,
    case_idx: int,
    case: dict,
    run_policy: dict,
) -> tuple[bool, list[str], list[str], str, dict]:
    smoke_rel = str(case.get("smoke_golden", "")).strip()
    smoke_path, smoke_doc = load_smoke_golden_doc(root, pack_dir, smoke_rel)
    ddn_path = (pack_dir / smoke_doc["ddn_file"]).resolve()
    if not ddn_path.exists():
        raise ValueError(f"{smoke_path}: ddn_file not found: {smoke_doc['ddn_file']}")
    if not ddn_path.is_relative_to(root):
        raise ValueError(f"{smoke_path}: ddn_file must be under workspace root")

    expected_contract = smoke_doc.get("contract")
    expected_seal = smoke_doc.get("detmath_seal_hash")
    expected_lock = smoke_doc.get("nuri_lock_hash")
    project_contract = map_contract_from_policy(run_policy)
    project_seal = run_policy.get("detmath_seal_hash")
    project_lock = run_policy.get("nuri_lock_hash")
    if expected_contract is not None and expected_contract != project_contract:
        return (
            False,
            [f"contract={expected_contract}"],
            [f"contract={project_contract}"],
            "",
            {"stdout_lines": [], "stderr_lines": [], "exit_code": 0, "actual_meta": None},
        )
    if expected_seal is not None and expected_seal != project_seal:
        return (
            False,
            [f"detmath_seal_hash={expected_seal}"],
            [f"detmath_seal_hash={project_seal}"],
            "",
            {"stdout_lines": [], "stderr_lines": [], "exit_code": 0, "actual_meta": None},
        )
    if expected_lock is not None and expected_lock != project_lock:
        return (
            False,
            [f"nuri_lock_hash={expected_lock}"],
            [f"nuri_lock_hash={project_lock}"],
            "",
            {"stdout_lines": [], "stderr_lines": [], "exit_code": 0, "actual_meta": None},
        )

    max_ticks = smoke_doc.get("max_ticks")
    checkpoints = smoke_doc["checkpoints"]
    timeout_ms = int(smoke_doc.get("timeout_ms", 30000))
    extra_cli = case.get("cli", [])
    if extra_cli and not isinstance(extra_cli, list):
        raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: cli must be a list")

    got_lines: list[str] = []
    stderr_lines: list[str] = []
    rows_for_update: list[dict] = []
    mismatches: list[tuple[int, str, str | None]] = []
    for cp in checkpoints:
        tick = int(cp["tick"])
        if max_ticks is not None and tick > int(max_ticks):
            return (
                False,
                [f"tick<={max_ticks}"],
                [f"tick={tick}"],
                "",
                {"stdout_lines": [], "stderr_lines": [], "exit_code": 0, "actual_meta": None},
            )
        command_args = ["run", str(ddn_path), "--madi", str(tick)] + [str(arg) for arg in extra_cli]
        if should_apply_default_trace_tier(command_args, run_policy):
            command_args += ["--trace-tier", str(run_policy["trace_tier"])]
        cmd = [
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            str(manifest),
            "--",
        ] + command_args
        result = subprocess.run(
            cmd,
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=max(1, timeout_ms) / 1000.0,
        )
        stderr_case = [normalize_stderr_line(line, root) for line in result.stderr.splitlines() if line.strip()]
        if result.returncode != 0:
            return (
                False,
                [f"tick={tick} exit_code=0"],
                [f"tick={tick} exit_code={result.returncode}"],
                "\n".join(stderr_case),
                {"stdout_lines": [], "stderr_lines": stderr_case, "exit_code": result.returncode, "actual_meta": None},
            )
        state_hash = extract_last_state_hash(result.stdout.splitlines())
        expected_hash = str(cp["state_hash"])
        got_lines.append(f"tick={tick} state_hash={state_hash}")
        rows_for_update.append({"tick": tick, "state_hash": state_hash or ""})
        stderr_lines.extend(stderr_case)
        if state_hash != expected_hash:
            mismatches.append((tick, expected_hash, state_hash))

    artifacts = {
        "stdout_lines": got_lines,
        "stderr_lines": stderr_lines,
        "exit_code": 0,
        "actual_meta": None,
        "smoke_golden_path": str(smoke_path),
        "smoke_golden_doc": smoke_doc,
        "smoke_rows": rows_for_update,
    }
    if mismatches:
        expected_lines = [f"tick={tick} state_hash={expected_hash}" for tick, expected_hash, _ in mismatches]
        actual_lines = [f"tick={tick} state_hash={actual_hash}" for tick, _, actual_hash in mismatches]
        return False, expected_lines, actual_lines, "\n".join(stderr_lines), artifacts
    return True, got_lines, got_lines, "\n".join(stderr_lines), artifacts


def canonical_json_lines(data: object) -> list[str]:
    return [json.dumps(data, ensure_ascii=False, sort_keys=True, separators=(",", ":"))]


def canonical_json_text(data: object) -> str:
    return json.dumps(data, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def hash_canonical_json(data: object, token: str) -> str:
    value = str(token).strip()
    if ":" not in value:
        raise ValueError(f"hash token must have algorithm prefix: {token}")
    algo, _ = value.split(":", 1)
    payload = canonical_json_text(data).encode("utf-8")
    algo = algo.strip().lower()
    if algo == "sha256":
        return f"sha256:{hashlib.sha256(payload).hexdigest()}"
    if algo == "blake3":
        try:
            import blake3  # type: ignore
        except Exception as exc:
            raise ValueError(f"blake3 hash requested but blake3 module is unavailable: {exc}") from exc
        return f"blake3:{blake3.blake3(payload).hexdigest()}"
    raise ValueError(f"unsupported hash algorithm: {algo}")


def load_json_doc(path: Path) -> object:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError(f"{path}: JSON parse failed: {exc}") from exc


def run_dotbogi_case(
    root: Path,
    manifest: Path,
    pack_dir: Path,
    case_idx: int,
    case: dict,
) -> tuple[bool, list[str], list[str], str, dict]:
    rel = str(case.get("dotbogi_case", "")).strip()
    if not rel:
        raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: dotbogi_case must be non-empty string")
    doc_path = (pack_dir / rel).resolve()
    if not doc_path.exists():
        raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: dotbogi_case not found: {rel}")
    report_path = pack_dir / f".tmp_dotbogi_case_{case_idx}.report.detjson"
    cmd = [
        "cargo",
        "run",
        "-q",
        "--manifest-path",
        str(manifest),
        "--",
        "dotbogi",
        "case",
        str(doc_path),
        "--report-out",
        str(report_path),
    ]
    result = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    stderr_lines = [
        normalize_stderr_line(line, root)
        for line in result.stderr.splitlines()
        if line.strip()
    ]
    stdout_lines = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    expected_error_code = case.get("expected_error_code")
    pre_artifacts = {
        "stdout_lines": stdout_lines,
        "stderr_lines": stderr_lines,
        "exit_code": result.returncode,
        "actual_meta": None,
    }
    if isinstance(expected_error_code, str):
        if result.returncode == 0:
            report_path.unlink(missing_ok=True)
            return (
                False,
                [f"exit_code!=0 expected ({expected_error_code})"],
                [f"exit_code={result.returncode}"],
                "\n".join(stderr_lines),
                pre_artifacts,
            )
        combined = stderr_lines + stdout_lines
        report_path.unlink(missing_ok=True)
        if not any(expected_error_code in line for line in combined):
            return (
                False,
                [f"expected_error_code={expected_error_code}"],
                combined,
                "\n".join(stderr_lines),
                pre_artifacts,
            )
        return True, [f"expected_error_code={expected_error_code}"], combined, "\n".join(stderr_lines), pre_artifacts

    if result.returncode != 0:
        report_path.unlink(missing_ok=True)
        return (
            False,
            ["exit_code=0"],
            [f"exit_code={result.returncode}"],
            "\n".join(stderr_lines),
            pre_artifacts,
        )

    if not report_path.exists():
        return (
            False,
            ["dotbogi report file exists"],
            ["dotbogi report missing"],
            "\n".join(stderr_lines),
            pre_artifacts,
        )
    report = load_json_doc(report_path)
    report_path.unlink(missing_ok=True)
    if not isinstance(report, dict):
        raise ValueError(f"{doc_path}: report root must be object")
    output = report.get("output")
    if not isinstance(output, dict):
        raise ValueError(f"{doc_path}: report.output must be object")
    view_meta = output.get("view_meta")
    if not isinstance(view_meta, dict):
        raise ValueError(f"{doc_path}: report.output.view_meta must be object")

    output_artifacts: dict[str, object] = {
        "dotbogi_output_actual_json": output,
    }
    after_state_artifacts: dict[str, object] = {}

    expected_output_rel = case.get("expected_dotbogi_output")
    if isinstance(expected_output_rel, str) and expected_output_rel.strip():
        expected_output_path = (pack_dir / expected_output_rel).resolve()
        output_artifacts["dotbogi_output_expected_path"] = str(expected_output_path)
        if not expected_output_path.exists():
            raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: expected_dotbogi_output not found: {expected_output_rel}")
        expected_output = load_json_doc(expected_output_path)
        expected_lines = canonical_json_lines(expected_output)
        actual_lines = canonical_json_lines(output)
        if expected_lines != actual_lines:
            return (
                False,
                expected_lines,
                actual_lines,
                "\n".join(stderr_lines),
                {
                    "stdout_lines": actual_lines,
                    "stderr_lines": stderr_lines,
                    "exit_code": 0,
                    "actual_meta": None,
                    **output_artifacts,
                },
            )

    actual_view_meta_hash = report.get("view_meta_hash")
    if not isinstance(actual_view_meta_hash, str) or not actual_view_meta_hash.strip():
        actual_view_meta_hash = None
    expected_view_meta_hash = case.get("expected_view_meta_hash")
    if isinstance(expected_view_meta_hash, str) and expected_view_meta_hash.strip():
        if not (
            isinstance(actual_view_meta_hash, str)
            and expected_view_meta_hash.startswith("sha256:")
            and actual_view_meta_hash.startswith("sha256:")
        ):
            actual_view_meta_hash = hash_canonical_json(view_meta, expected_view_meta_hash)
        if actual_view_meta_hash != expected_view_meta_hash:
            return (
                False,
                [f"view_meta_hash={expected_view_meta_hash}"],
                [f"view_meta_hash={actual_view_meta_hash}"],
                "\n".join(stderr_lines),
                {
                    "stdout_lines": stdout_lines,
                    "stderr_lines": stderr_lines,
                    "exit_code": 0,
                    "actual_meta": None,
                    "dotbogi_view_meta_hash_actual": actual_view_meta_hash,
                    **output_artifacts,
                },
            )

    after_state = report.get("after_state")
    if isinstance(after_state, dict):
        after_state_artifacts["dotbogi_after_state_actual_json"] = after_state
    else:
        after_state = None
    expected_after_state_rel = case.get("expected_after_state")
    if isinstance(expected_after_state_rel, str) and expected_after_state_rel.strip():
        if after_state is None:
            return (
                False,
                ["after_state expected but roundtrip is missing"],
                ["after_state unavailable"],
                "\n".join(stderr_lines),
                {"stdout_lines": stdout_lines, "stderr_lines": stderr_lines, "exit_code": 0, "actual_meta": None},
            )
        expected_after_state_path = (pack_dir / expected_after_state_rel).resolve()
        after_state_artifacts["dotbogi_after_state_expected_path"] = str(expected_after_state_path)
        if not expected_after_state_path.exists():
            raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: expected_after_state not found: {expected_after_state_rel}")
        expected_after_state = load_json_doc(expected_after_state_path)
        expected_lines = canonical_json_lines(expected_after_state)
        actual_lines = canonical_json_lines(after_state)
        if expected_lines != actual_lines:
            return (
                False,
                expected_lines,
                actual_lines,
                "\n".join(stderr_lines),
                {
                    "stdout_lines": actual_lines,
                    "stderr_lines": stderr_lines,
                    "exit_code": 0,
                    "actual_meta": None,
                    **output_artifacts,
                    **after_state_artifacts,
                },
            )

    actual_after_state_hash = report.get("after_state_hash")
    if not isinstance(actual_after_state_hash, str) or not actual_after_state_hash.strip():
        actual_after_state_hash = None
    expected_after_state_hash = case.get("expected_after_state_hash")
    if isinstance(expected_after_state_hash, str) and expected_after_state_hash.strip():
        if after_state is None:
            return (
                False,
                ["after_state_hash expected but roundtrip is missing"],
                ["after_state unavailable"],
                "\n".join(stderr_lines),
                {"stdout_lines": stdout_lines, "stderr_lines": stderr_lines, "exit_code": 0, "actual_meta": None},
            )
        if not (
            isinstance(actual_after_state_hash, str)
            and expected_after_state_hash.startswith("sha256:")
            and actual_after_state_hash.startswith("sha256:")
        ):
            actual_after_state_hash = hash_canonical_json(after_state, expected_after_state_hash)
        if actual_after_state_hash != expected_after_state_hash:
            return (
                False,
                [f"after_state_hash={expected_after_state_hash}"],
                [f"after_state_hash={actual_after_state_hash}"],
                "\n".join(stderr_lines),
                {
                    "stdout_lines": stdout_lines,
                    "stderr_lines": stderr_lines,
                    "exit_code": 0,
                    "actual_meta": None,
                    "dotbogi_after_state_hash_actual": actual_after_state_hash,
                    **output_artifacts,
                    **after_state_artifacts,
                },
            )

    lines = stdout_lines if stdout_lines else [f"dotbogi_output={canonical_json_text(output)}"]
    return (
        True,
        lines,
        lines,
        "\n".join(stderr_lines),
        {
            "stdout_lines": lines,
            "stderr_lines": stderr_lines,
            "exit_code": 0,
            "actual_meta": None,
            "dotbogi_view_meta_hash_actual": actual_view_meta_hash,
            "dotbogi_after_state_hash_actual": actual_after_state_hash,
            **output_artifacts,
            **after_state_artifacts,
        },
    )


def run_overlay_compare_case(
    root: Path,
    pack_dir: Path,
    case_idx: int,
    case: dict,
) -> tuple[bool, list[str], list[str], str, dict]:
    rel = str(case.get("overlay_compare_case", "")).strip()
    if not rel:
        raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: overlay_compare_case must be non-empty string")
    case_path = (pack_dir / rel).resolve()
    if not case_path.exists():
        raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: overlay_compare_case not found: {rel}")

    runner = root / "tests" / "seamgrim_overlay_compare_pack_runner.mjs"
    report_path = pack_dir / f".tmp_overlay_compare_case_{case_idx}.report.detjson"
    report_path.unlink(missing_ok=True)
    cmd = [
        "node",
        "--no-warnings",
        str(runner),
        "--pack-root",
        str(pack_dir),
        "--case-file",
        rel,
        "--json-out",
        str(report_path),
        "--quiet",
    ]
    result = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    stderr_lines = [
        normalize_stderr_line(line, root)
        for line in result.stderr.splitlines()
        if line.strip()
    ]
    stdout_lines = [line.strip() for line in result.stdout.splitlines() if line.strip()]

    report_doc = None
    if report_path.exists():
        try:
            report_doc = json.loads(report_path.read_text(encoding="utf-8"))
        except Exception:
            report_doc = None
        report_path.unlink(missing_ok=True)

    if not isinstance(report_doc, dict):
        expected = ["overlay_compare runner report json"]
        got = [f"exit_code={result.returncode}"]
        return (
            False,
            expected,
            got,
            "\n".join(stderr_lines + stdout_lines),
            {
                "stdout_lines": stdout_lines,
                "stderr_lines": stderr_lines,
                "exit_code": result.returncode,
                "actual_meta": None,
            },
        )

    rows = report_doc.get("cases")
    row = rows[0] if isinstance(rows, list) and rows else {}
    actual = row.get("actual") if isinstance(row, dict) else {}
    expected_ok = bool(row.get("expected_ok", False)) if isinstance(row, dict) else False
    expected_code = str(row.get("expected_code", "") or "-") if isinstance(row, dict) else "-"
    reason_contains = str(row.get("reason_contains", "") or "") if isinstance(row, dict) else ""
    actual_ok = bool(actual.get("ok", False)) if isinstance(actual, dict) else False
    actual_code = str(actual.get("code", "") or "-") if isinstance(actual, dict) else "-"
    actual_reason = str(actual.get("reason", "") or "") if isinstance(actual, dict) else ""

    expected_lines = [
        f"overlay_ok={int(expected_ok)}",
        f"overlay_code={expected_code}",
    ]
    got_lines = [
        f"overlay_ok={int(actual_ok)}",
        f"overlay_code={actual_code}",
    ]
    if reason_contains:
        expected_lines.append(f"reason_contains={reason_contains}")
        got_lines.append(f"reason={actual_reason}")

    ok = result.returncode == 0 and bool(report_doc.get("ok", False)) and bool(row.get("ok", False))
    return (
        ok,
        expected_lines,
        got_lines,
        "\n".join(stderr_lines + stdout_lines),
        {
            "stdout_lines": got_lines,
            "stderr_lines": stderr_lines,
            "exit_code": result.returncode,
            "actual_meta": None,
            "overlay_compare_case_report": report_doc,
            "overlay_compare_case_path": str(case_path),
        },
    )


def run_overlay_session_case(
    root: Path,
    pack_dir: Path,
    case_idx: int,
    case: dict,
) -> tuple[bool, list[str], list[str], str, dict]:
    rel = str(case.get("overlay_session_case", "")).strip()
    if not rel:
        raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: overlay_session_case must be non-empty string")
    case_path = (pack_dir / rel).resolve()
    if not case_path.exists():
        raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: overlay_session_case not found: {rel}")

    runner = root / "tests" / "seamgrim_overlay_session_pack_runner.mjs"
    report_path = pack_dir / f".tmp_overlay_session_case_{case_idx}.report.detjson"
    report_path.unlink(missing_ok=True)
    cmd = [
        "node",
        "--no-warnings",
        str(runner),
        "--pack-root",
        str(pack_dir),
        "--case-file",
        rel,
        "--json-out",
        str(report_path),
        "--quiet",
    ]
    result = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    stderr_lines = [
        normalize_stderr_line(line, root)
        for line in result.stderr.splitlines()
        if line.strip()
    ]
    stdout_lines = [line.strip() for line in result.stdout.splitlines() if line.strip()]

    report_doc = None
    if report_path.exists():
        try:
            report_doc = json.loads(report_path.read_text(encoding="utf-8"))
        except Exception:
            report_doc = None
        report_path.unlink(missing_ok=True)

    if not isinstance(report_doc, dict):
        expected = ["overlay_session runner report json"]
        got = [f"exit_code={result.returncode}"]
        return (
            False,
            expected,
            got,
            "\n".join(stderr_lines + stdout_lines),
            {
                "stdout_lines": stdout_lines,
                "stderr_lines": stderr_lines,
                "exit_code": result.returncode,
                "actual_meta": None,
            },
        )

    rows = report_doc.get("cases")
    row = rows[0] if isinstance(rows, list) and rows else {}
    expected_obj = row.get("expected") if isinstance(row, dict) else {}
    actual_obj = row.get("actual") if isinstance(row, dict) else {}
    expected_enabled = int(bool(expected_obj.get("enabled", False))) if isinstance(expected_obj, dict) else 0
    expected_baseline = str(expected_obj.get("baselineId", "-") or "-") if isinstance(expected_obj, dict) else "-"
    expected_variant = str(expected_obj.get("variantId", "-") or "-") if isinstance(expected_obj, dict) else "-"
    expected_drop_code = str(expected_obj.get("dropCode", "-") or "-") if isinstance(expected_obj, dict) else "-"
    actual_enabled = int(bool(actual_obj.get("enabled", False))) if isinstance(actual_obj, dict) else 0
    actual_baseline = str(actual_obj.get("baselineId", "-") or "-") if isinstance(actual_obj, dict) else "-"
    actual_variant = str(actual_obj.get("variantId", "-") or "-") if isinstance(actual_obj, dict) else "-"
    actual_drop_code = str(actual_obj.get("dropCode", "-") or "-") if isinstance(actual_obj, dict) else "-"
    actual_reason = str(actual_obj.get("blockReason", "") or "") if isinstance(actual_obj, dict) else ""

    expected_lines = [
        f"enabled={expected_enabled}",
        f"baseline_id={expected_baseline}",
        f"variant_id={expected_variant}",
        f"drop_code={expected_drop_code}",
    ]
    got_lines = [
        f"enabled={actual_enabled}",
        f"baseline_id={actual_baseline}",
        f"variant_id={actual_variant}",
        f"drop_code={actual_drop_code}",
    ]
    if actual_reason:
        got_lines.append(f"block_reason={actual_reason}")

    ok = result.returncode == 0 and bool(report_doc.get("ok", False)) and bool(row.get("ok", False))
    return (
        ok,
        expected_lines,
        got_lines,
        "\n".join(stderr_lines + stdout_lines),
        {
            "stdout_lines": got_lines,
            "stderr_lines": stderr_lines,
            "exit_code": result.returncode,
            "actual_meta": None,
            "overlay_session_case_report": report_doc,
            "overlay_session_case_path": str(case_path),
        },
    )


def run_graph_autorender_case(
    root: Path,
    pack_dir: Path,
    case_idx: int,
    case: dict,
) -> tuple[bool, list[str], list[str], str, dict]:
    fixture_rel = case.get("fixture")
    expected_rel = case.get("expected_graph")
    prefer_patch = bool(case.get("prefer_patch", False))
    if not isinstance(fixture_rel, str) or not fixture_rel.strip():
        raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: fixture must be non-empty string")
    if not isinstance(expected_rel, str) or not expected_rel.strip():
        raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: expected_graph must be non-empty string")

    expected_path = (pack_dir / expected_rel).resolve()
    if not expected_path.exists():
        raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: expected_graph not found: {expected_rel}")
    expected_obj = json.loads(expected_path.read_text(encoding="utf-8"))

    runner = root / "tests" / "seamgrim_graph_autorender_runner.mjs"
    cmd = [
        "node",
        "--no-warnings",
        str(runner),
        str(pack_dir),
        fixture_rel,
        "true" if prefer_patch else "false",
    ]
    result = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    stderr_lines = [line for line in result.stderr.splitlines() if line.strip()]
    if result.returncode != 0:
        detail = stderr_lines or [line for line in result.stdout.splitlines() if line.strip()] or ["runner failed"]
        return (
            False,
            ["graph autorender runner exit_code=0"],
            [f"graph autorender runner exit_code={result.returncode}"],
            "\n".join(detail),
            {
                "stdout_lines": [line for line in result.stdout.splitlines() if line.strip()],
                "stderr_lines": stderr_lines,
                "exit_code": result.returncode,
                "actual_meta": None,
            },
        )

    actual_obj = json.loads(result.stdout)
    expected_lines = canonical_json_lines(expected_obj)
    actual_lines = canonical_json_lines(actual_obj)
    ok = expected_lines == actual_lines
    artifacts = {
        "stdout_lines": actual_lines,
        "stderr_lines": stderr_lines,
        "exit_code": 0,
        "actual_meta": None,
        "graph_expected_path": str(expected_path),
        "graph_actual_json": actual_obj,
    }
    return ok, expected_lines, actual_lines, "\n".join(stderr_lines), artifacts


def run_case(
    root: Path,
    manifest: Path,
    pack_dir: Path,
    case_idx: int,
    case: dict,
    run_policy: dict,
) -> tuple[bool, list[str], list[str], str, dict]:
    smoke_golden = case.get("smoke_golden")
    if isinstance(smoke_golden, str) and smoke_golden.strip():
        return run_smoke_golden_case(root, manifest, pack_dir, case_idx, case, run_policy)
    dotbogi_case = case.get("dotbogi_case")
    if isinstance(dotbogi_case, str) and dotbogi_case.strip():
        return run_dotbogi_case(root, manifest, pack_dir, case_idx, case)
    overlay_compare_case = case.get("overlay_compare_case")
    if isinstance(overlay_compare_case, str) and overlay_compare_case.strip():
        return run_overlay_compare_case(root, pack_dir, case_idx, case)
    overlay_session_case = case.get("overlay_session_case")
    if isinstance(overlay_session_case, str) and overlay_session_case.strip():
        return run_overlay_session_case(root, pack_dir, case_idx, case)
    if "fixture" in case and "expected_graph" in case:
        return run_graph_autorender_case(root, pack_dir, case_idx, case)

    input_text = case.get("input")
    input_path = case.get("input_path")
    extra_cli = case.get("cli", [])
    cmd_override = case.get("cmd")
    stdout_path = case.get("stdout_path")
    meta_out = case.get("meta_out")
    expected_meta_path = case.get("expected_meta")
    expected_exit = case.get("exit_code", 0)
    expected_stderr = case.get("stderr")
    expected_error_code = case.get("expected_error_code")
    expected_warning_code = case.get("expected_warning_code")
    expected_contract = case.get("expected_contract")
    expected_detmath_seal_hash = case.get("expected_detmath_seal_hash")
    expected_nuri_lock_hash = case.get("expected_nuri_lock_hash")
    temp_path = None

    if input_text is not None:
        input_text = input_text.replace("{{CWD}}", root.as_posix())
        input_text = input_text.replace("{{CWD_UPPER}}", root.as_posix().upper())
        temp_path = pack_dir / f".tmp_case_{case_idx}.ddn"
        temp_path.write_text(input_text, encoding="utf-8", newline="\n")
        run_path = temp_path
    elif input_path is not None:
        run_path = pack_dir / input_path
    else:
        run_path = pack_dir / "input.ddn"

    base_cmd = [
        "cargo",
        "run",
        "-q",
        "--manifest-path",
        str(manifest),
        "--",
    ]
    command_args: list[str]
    if cmd_override is not None:
        if not isinstance(cmd_override, list):
            raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: cmd must be a list")
        command_args = [str(arg) for arg in cmd_override]
        is_run_cmd = len(cmd_override) > 0 and cmd_override[0] == "run"
    else:
        command_args = ["run", str(run_path)]
        if extra_cli:
            if not isinstance(extra_cli, list):
                raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: cli must be a list")
            command_args += [str(arg) for arg in extra_cli]
        is_run_cmd = True
    if should_apply_default_trace_tier(command_args, run_policy):
        command_args += ["--trace-tier", str(run_policy["trace_tier"])]
    cmd = base_cmd + command_args
    if meta_out:
        meta_out_path = pack_dir / meta_out
        meta_out_path.unlink(missing_ok=True)
    result = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )

    if temp_path is not None:
        temp_path.unlink(missing_ok=True)

    stderr_lines = []
    for line in result.stderr.splitlines():
        if not line.strip():
            continue
        trimmed = line.lstrip()
        if line.startswith("warning:"):
            continue
        if trimmed.startswith(("-->", "|", "=", "...")) or trimmed.startswith(("note:", "help:")):
            continue
        if re.match(r"^\d+ \|", trimmed):
            continue
        stderr_lines.append(normalize_stderr_line(line, root))
    stdout_raw = result.stdout.splitlines()
    actual_meta: str | None = None
    if meta_out:
        meta_out_path = pack_dir / meta_out
        if meta_out_path.exists():
            actual_meta = meta_out_path.read_text(encoding="utf-8").strip()
            meta_out_path.unlink(missing_ok=True)
    pre_artifacts = {
        "stdout_lines": [line for line in stdout_raw if line.strip()],
        "stderr_lines": stderr_lines,
        "exit_code": result.returncode,
        "actual_meta": actual_meta,
    }

    if expected_error_code is not None:
        if expected_exit != 0:
            if result.returncode != expected_exit:
                return (
                    False,
                    [f"exit_code={expected_exit}"],
                    [f"exit_code={result.returncode}"],
                    "\n".join(stderr_lines),
                    pre_artifacts,
                )
        elif result.returncode == 0:
            return (
                False,
                [f"exit_code!=0 expected ({expected_error_code})"],
                [f"exit_code={result.returncode}"],
                "\n".join(stderr_lines),
                pre_artifacts,
            )
        combined = stderr_lines + [line for line in stdout_raw if line.strip()]
        if not any(expected_error_code in line for line in combined):
            return (
                False,
                [f"expected_error_code={expected_error_code}"],
                combined,
                "\n".join(stderr_lines),
                pre_artifacts,
            )
        if meta_out and expected_meta_path:
            expected_meta = (pack_dir / expected_meta_path).read_text(encoding="utf-8").strip()
            if actual_meta is None:
                return (
                    False,
                    ["meta_out missing"],
                    ["meta_out missing"],
                    "\n".join(stderr_lines),
                    pre_artifacts,
                )
            if actual_meta != expected_meta:
                return (
                    False,
                    expected_meta.splitlines(),
                    actual_meta.splitlines(),
                    "\n".join(stderr_lines),
                    pre_artifacts,
                )
        if (
            expected_contract is not None
            or expected_detmath_seal_hash is not None
            or expected_nuri_lock_hash is not None
        ):
            if actual_meta is None:
                return (
                    False,
                    ["meta_out missing for contract/seal/lock check"],
                    ["meta_out missing"],
                    "\n".join(stderr_lines),
                    pre_artifacts,
                )
            try:
                actual_meta_json = json.loads(actual_meta)
            except json.JSONDecodeError:
                return (
                    False,
                    ["meta_out must be valid JSON for contract/seal/lock check"],
                    [actual_meta],
                    "\n".join(stderr_lines),
                    pre_artifacts,
                )
            meta_scope = actual_meta_json.get("meta")
            if not isinstance(meta_scope, dict):
                meta_scope = {}

            def pick_meta(key: str):
                if key in actual_meta_json:
                    return actual_meta_json.get(key)
                return meta_scope.get(key)

            checks = [
                ("contract", expected_contract),
                ("detmath_seal_hash", expected_detmath_seal_hash),
                ("nuri_lock_hash", expected_nuri_lock_hash),
            ]
            for key, expected_value in checks:
                if expected_value is None:
                    continue
                actual_value = pick_meta(key)
                if actual_value != expected_value:
                    return (
                        False,
                        [f"{key}={expected_value}"],
                        [f"{key}={actual_value}"],
                        "\n".join(stderr_lines),
                        pre_artifacts,
                    )
        return True, [f"expected_error_code={expected_error_code}"], combined, "\n".join(stderr_lines), pre_artifacts

    if result.returncode != expected_exit:
        return (
            False,
            [f"exit_code={expected_exit}"],
            [f"exit_code={result.returncode}"],
            "\n".join(stderr_lines),
            pre_artifacts,
        )

    if stdout_path:
        stdout_lines = stdout_raw
    else:
        stdout_lines = [line for line in stdout_raw if line.strip()]
    if is_run_cmd:
        stdout_lines = [
            line
            for line in stdout_lines
            if not line.startswith("state_hash=") and not line.startswith("trace_hash=")
        ]
    if stdout_path:
        expected = (pack_dir / stdout_path).read_text(encoding="utf-8").splitlines()
    else:
        expected = case.get("stdout", [])
    expected_bogae_hash = case.get("bogae_hash")
    artifacts = {
        "stdout_lines": stdout_lines,
        "stderr_lines": stderr_lines,
        "exit_code": result.returncode,
        "actual_meta": actual_meta,
    }
    if expected_stderr is not None:
        if len(stderr_lines) != len(expected_stderr):
            return False, expected, stdout_lines, "\n".join(stderr_lines), artifacts
        for got, want in zip(stderr_lines, expected_stderr):
            if not matches_any_expected(got, split_expected(want)):
                return False, expected, stdout_lines, "\n".join(stderr_lines), artifacts
    if meta_out and expected_meta_path:
        expected_meta = (pack_dir / expected_meta_path).read_text(encoding="utf-8").strip()
        if actual_meta is None:
            return False, ["meta_out missing"], ["meta_out missing"], "\n".join(stderr_lines), artifacts
        if actual_meta != expected_meta:
            return False, expected_meta.splitlines(), actual_meta.splitlines(), "\n".join(stderr_lines), artifacts
    if (
        expected_contract is not None
        or expected_detmath_seal_hash is not None
        or expected_nuri_lock_hash is not None
    ):
        if actual_meta is None:
            return (
                False,
                ["meta_out missing for contract/seal/lock check"],
                ["meta_out missing"],
                "\n".join(stderr_lines),
                artifacts,
            )
        try:
            actual_meta_json = json.loads(actual_meta)
        except json.JSONDecodeError:
            return (
                False,
                ["meta_out must be valid JSON for contract/seal/lock check"],
                [actual_meta],
                "\n".join(stderr_lines),
                artifacts,
            )
        meta_scope = actual_meta_json.get("meta")
        if not isinstance(meta_scope, dict):
            meta_scope = {}

        def pick_meta(key: str):
            if key in actual_meta_json:
                return actual_meta_json.get(key)
            return meta_scope.get(key)

        checks = [
            ("contract", expected_contract),
            ("detmath_seal_hash", expected_detmath_seal_hash),
            ("nuri_lock_hash", expected_nuri_lock_hash),
        ]
        for key, expected_value in checks:
            if expected_value is None:
                continue
            actual_value = pick_meta(key)
            if actual_value != expected_value:
                return (
                    False,
                    [f"{key}={expected_value}"],
                    [f"{key}={actual_value}"],
                    "\n".join(stderr_lines),
                    artifacts,
                )
    if expected_bogae_hash is not None:
        got_bogae_hash = extract_bogae_hash(stdout_lines)
        if got_bogae_hash != expected_bogae_hash:
            return (
                False,
                [f"bogae_hash={expected_bogae_hash}"],
                [f"bogae_hash={got_bogae_hash or 'missing'}"],
                "\n".join(stderr_lines),
                artifacts,
            )
    if expected_warning_code is not None:
        combined = stderr_lines + [line for line in stdout_raw if line.strip()]
        if not any(expected_warning_code in line for line in combined):
            return (
                False,
                [f"expected_warning_code={expected_warning_code}"],
                combined,
                "\n".join(stderr_lines),
                artifacts,
            )
    if "stdout" in case or stdout_path:
        return stdout_lines == expected, expected, stdout_lines, "\n".join(stderr_lines), artifacts
    return True, expected, stdout_lines, "\n".join(stderr_lines), artifacts


def should_write_expected(path: Path, update: bool, record: bool) -> bool:
    if update:
        return True
    if record and not path.exists():
        return True
    return False


def clip_lines(lines: list[str], limit: int = 6) -> list[str]:
    if len(lines) <= limit:
        return lines
    return lines[:limit] + [f"... ({len(lines) - limit} more lines)"]


def format_failure_lines(failure: tuple, root: Path) -> list[str]:
    if len(failure) == 2:
        pack_dir, reason = failure
        pack_text = str(pack_dir)
        try:
            pack_text = str(Path(pack_dir).relative_to(root))
        except Exception:
            pass
        return [f"[FAIL] pack={pack_text} reason={reason}"]
    if len(failure) != 5:
        return [f"[FAIL] unexpected failure tuple={failure}"]
    pack_dir, idx, expected, got, stderr = failure
    pack_text = str(pack_dir)
    try:
        pack_text = str(Path(pack_dir).relative_to(root))
    except Exception:
        pass
    lines = [f"[FAIL] pack={pack_text} case={idx}"]
    expected_lines = clip_lines([str(item) for item in (expected or [])], limit=5)
    got_lines = clip_lines([str(item) for item in (got or [])], limit=5)
    stderr_lines = clip_lines([line for line in str(stderr).splitlines() if line.strip()], limit=5)
    if expected_lines:
        lines.append(f"  expected: {' | '.join(expected_lines)}")
    if got_lines:
        lines.append(f"  got: {' | '.join(got_lines)}")
    if stderr_lines:
        lines.append(f"  stderr: {' | '.join(stderr_lines)}")
    return lines


def write_case_updates(
    pack_dir: Path,
    case: dict,
    artifacts: dict,
    update: bool,
    record: bool,
) -> tuple[bool, list[str]]:
    issues: list[str] = []
    case_changed = False

    stdout_lines = list(artifacts.get("stdout_lines") or [])
    stderr_lines = list(artifacts.get("stderr_lines") or [])
    exit_code = int(artifacts.get("exit_code", 0))
    actual_meta = artifacts.get("actual_meta")

    if "stdout_path" in case and isinstance(case.get("stdout_path"), str):
        out_path = pack_dir / str(case["stdout_path"])
        if should_write_expected(out_path, update, record):
            out_path.parent.mkdir(parents=True, exist_ok=True)
            text = "\n".join(stdout_lines)
            if text:
                text += "\n"
            out_path.write_text(text, encoding="utf-8", newline="\n")
    elif "stdout" in case and update:
        case["stdout"] = stdout_lines
        case_changed = True

    if "stderr" in case and update:
        case["stderr"] = stderr_lines
        case_changed = True

    if update:
        case["exit_code"] = exit_code
        case_changed = True

    expected_meta_path = case.get("expected_meta")
    if isinstance(expected_meta_path, str) and expected_meta_path:
        meta_path = pack_dir / expected_meta_path
        if should_write_expected(meta_path, update, record):
            if actual_meta is None:
                issues.append(f"{pack_dir}: expected_meta 대상인데 meta_out 결과가 없습니다 ({expected_meta_path})")
            else:
                meta_path.parent.mkdir(parents=True, exist_ok=True)
                meta_path.write_text(f"{actual_meta}\n", encoding="utf-8", newline="\n")

    smoke_path_text = artifacts.get("smoke_golden_path")
    smoke_doc = artifacts.get("smoke_golden_doc")
    smoke_rows = artifacts.get("smoke_rows")
    if (
        isinstance(case.get("smoke_golden"), str)
        and isinstance(smoke_path_text, str)
        and isinstance(smoke_doc, dict)
        and isinstance(smoke_rows, list)
    ):
        if update:
            checkpoints = smoke_doc.get("checkpoints")
            if isinstance(checkpoints, list) and len(checkpoints) == len(smoke_rows):
                for cp, row in zip(checkpoints, smoke_rows):
                    if isinstance(cp, dict) and isinstance(row, dict) and "state_hash" in row:
                        cp["state_hash"] = str(row.get("state_hash", ""))
                smoke_path = Path(smoke_path_text)
                smoke_path.write_text(
                    json.dumps(smoke_doc, ensure_ascii=False, indent=2) + "\n",
                    encoding="utf-8",
                    newline="\n",
                )

    graph_expected_path = artifacts.get("graph_expected_path")
    graph_actual_json = artifacts.get("graph_actual_json")
    if isinstance(graph_expected_path, str) and graph_actual_json is not None:
        graph_path = Path(graph_expected_path)
        if should_write_expected(graph_path, update, record):
            graph_path.parent.mkdir(parents=True, exist_ok=True)
            graph_path.write_text(
                json.dumps(graph_actual_json, ensure_ascii=False, indent=2) + "\n",
                encoding="utf-8",
                newline="\n",
            )

    dotbogi_output_expected_path = artifacts.get("dotbogi_output_expected_path")
    dotbogi_output_actual_json = artifacts.get("dotbogi_output_actual_json")
    if isinstance(dotbogi_output_expected_path, str) and dotbogi_output_actual_json is not None:
        output_path = Path(dotbogi_output_expected_path)
        if should_write_expected(output_path, update, record):
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text(
                json.dumps(dotbogi_output_actual_json, ensure_ascii=False, indent=2) + "\n",
                encoding="utf-8",
                newline="\n",
            )

    dotbogi_after_state_expected_path = artifacts.get("dotbogi_after_state_expected_path")
    dotbogi_after_state_actual_json = artifacts.get("dotbogi_after_state_actual_json")
    if isinstance(dotbogi_after_state_expected_path, str) and dotbogi_after_state_actual_json is not None:
        after_state_path = Path(dotbogi_after_state_expected_path)
        if should_write_expected(after_state_path, update, record):
            after_state_path.parent.mkdir(parents=True, exist_ok=True)
            after_state_path.write_text(
                json.dumps(dotbogi_after_state_actual_json, ensure_ascii=False, indent=2) + "\n",
                encoding="utf-8",
                newline="\n",
            )

    dotbogi_view_meta_hash_actual = artifacts.get("dotbogi_view_meta_hash_actual")
    if (
        update
        and "expected_view_meta_hash" in case
        and isinstance(dotbogi_view_meta_hash_actual, str)
        and dotbogi_view_meta_hash_actual.strip()
    ):
        case["expected_view_meta_hash"] = dotbogi_view_meta_hash_actual
        case_changed = True

    dotbogi_after_state_hash_actual = artifacts.get("dotbogi_after_state_hash_actual")
    if (
        update
        and "expected_after_state_hash" in case
        and isinstance(dotbogi_after_state_hash_actual, str)
        and dotbogi_after_state_hash_actual.strip()
    ):
        case["expected_after_state_hash"] = dotbogi_after_state_hash_actual
        case_changed = True

    return case_changed, issues


def main() -> int:
    parser = argparse.ArgumentParser(description="Run pack golden cases using teul-cli")
    parser.add_argument("packs", nargs="*", help="pack names under ./pack")
    parser.add_argument("--all", action="store_true", help="scan all pack folders with golden.jsonl")
    parser.add_argument("--record", action="store_true", help="record missing expected artifacts")
    parser.add_argument("--update", action="store_true", help="update expected artifacts in-place")
    parser.add_argument(
        "--manifest-path",
        help="cargo manifest path for CLI runner (default: tools/teul-cli/Cargo.toml)",
    )
    parser.add_argument("--report-out", help="write JSON report to this path")
    args = parser.parse_args()
    if args.record and args.update:
        print("--record 와 --update 는 동시에 사용할 수 없습니다.", file=sys.stderr)
        return 2

    root = Path(__file__).resolve().parent.parent
    manifest = Path(args.manifest_path) if args.manifest_path else (root / "tools" / "teul-cli" / "Cargo.toml")
    if not manifest.is_absolute():
        manifest = (root / manifest).resolve()
    if not manifest.exists():
        print(f"manifest not found: {manifest}", file=sys.stderr)
        return 2
    packs = iter_packs(root, args.packs, args.all)
    run_policy = load_root_run_policy(root)

    failures = []
    updated_pack_files = 0
    started = time.perf_counter()
    report_packs: list[dict] = []
    for pack_dir in packs:
        pack_started = time.perf_counter()
        try:
            pack_name = pack_dir.relative_to(root / "pack").as_posix()
        except ValueError:
            pack_name = pack_dir.as_posix()
        pack_report = {
            "pack": pack_name,
            "path": str(pack_dir),
            "ok": True,
            "case_count": 0,
            "failed_case_count": 0,
            "cases": [],
            "errors": [],
        }
        if not pack_dir.exists():
            failures.append((pack_dir, "missing pack"))
            pack_report["ok"] = False
            pack_report["errors"].append("missing pack")
            pack_report["elapsed_ms"] = int((time.perf_counter() - pack_started) * 1000)
            report_packs.append(pack_report)
            continue
        cases = load_cases(pack_dir)
        pack_report["case_count"] = len(cases)
        case_file_changed = False
        for idx, case in enumerate(cases, 1):
            ok, expected, got, stderr, artifacts = run_case(
                root, manifest, pack_dir, idx, case, run_policy
            )
            stderr_lines = [line for line in str(stderr).splitlines() if line.strip()]
            case_report = {
                "index": idx,
                "ok": True,
                "checked_ok": bool(ok),
            }
            if args.update or args.record:
                changed, issues = write_case_updates(pack_dir, case, artifacts, args.update, args.record)
                if changed:
                    case_file_changed = True
                for issue in issues:
                    failures.append((pack_dir, idx, [issue], [], stderr))
                if issues:
                    case_report["ok"] = False
                    case_report["issues"] = issues
                    if stderr_lines:
                        case_report["stderr"] = stderr_lines
                    pack_report["ok"] = False
                    pack_report["failed_case_count"] += 1
            elif not ok:
                failures.append((pack_dir, idx, expected, got, stderr))
                case_report["ok"] = False
                case_report["expected"] = expected
                case_report["got"] = got
                if stderr_lines:
                    case_report["stderr"] = stderr_lines
                pack_report["ok"] = False
                pack_report["failed_case_count"] += 1
            pack_report["cases"].append(case_report)
        if (args.update or args.record) and case_file_changed:
            golden_path = pack_dir / "golden.jsonl"
            lines = [json.dumps(case, ensure_ascii=False) for case in cases]
            golden_path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
            updated_pack_files += 1
        pack_report["elapsed_ms"] = int((time.perf_counter() - pack_started) * 1000)
        report_packs.append(pack_report)

    if args.report_out:
        report_path = Path(args.report_out)
        report_path.parent.mkdir(parents=True, exist_ok=True)
        mode = "check"
        if args.update:
            mode = "update"
        elif args.record:
            mode = "record"
        report = {
            "schema": "ddn.pack.golden.report.v1",
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "mode": mode,
            "overall_ok": len(failures) == 0,
            "updated_pack_files": updated_pack_files,
            "failure_count": len(failures),
            "elapsed_ms": int((time.perf_counter() - started) * 1000),
            "packs": report_packs,
        }
        report_path.write_text(
            json.dumps(report, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
            newline="\n",
        )

    if failures:
        print("pack golden failed", file=sys.stderr)
        for failure in failures:
            for line in format_failure_lines(failure, root):
                print(line, file=sys.stderr)
        print(f"failure_count={len(failures)}", file=sys.stderr)
        if args.report_out:
            print(f"report_out={args.report_out}", file=sys.stderr)
        return 1

    if args.update:
        print(f"pack golden updated ({updated_pack_files} files)")
    elif args.record:
        print("pack golden recorded")
    else:
        print("pack golden ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
