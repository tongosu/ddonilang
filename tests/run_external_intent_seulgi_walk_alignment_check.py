#!/usr/bin/env python
from __future__ import annotations

import argparse
import sys
from pathlib import Path


PACK_FILE_REQUIREMENTS: dict[str, tuple[str, ...]] = {
    "pack/external_intent_boundary_v1": (
        "README.md",
        "intent.md",
        "boundary_contract.detjson",
        "golden.jsonl",
    ),
    "pack/seulgi_v1": (
        "README.md",
        "intent.md",
        "contract.detjson",
        "golden.jsonl",
    ),
    "pack/sam_inputsnapshot_contract_v1": (
        "README.md",
        "intent.md",
        "contract.detjson",
        "golden.jsonl",
    ),
    "pack/sam_ai_ordering_v1": (
        "README.md",
        "intent.md",
        "contract.detjson",
        "golden.jsonl",
    ),
    "pack/seulgi_gatekeeper_v1": (
        "README.md",
        "intent.md",
        "contract.detjson",
        "golden.jsonl",
    ),
}

WALK_FILE_REQUIREMENTS: tuple[str, ...] = (
    "docs/ssot/walks/gogae5/w45_seulgi_intent/SPEC.md",
    "docs/ssot/walks/gogae5/w45_seulgi_intent/IMPL_GUIDE.md",
    "docs/ssot/walks/gogae5/w45_seulgi_intent/GOLDEN_TESTS.md",
    "docs/ssot/walks/gogae5/w49_latency_madi/SPEC.md",
    "docs/ssot/walks/gogae5/w49_latency_madi/IMPL_GUIDE.md",
    "docs/ssot/walks/gogae5/w49_latency_madi/GOLDEN_TESTS.md",
    "docs/ssot/walks/gogae5/w50_gatekeeper/SPEC.md",
    "docs/ssot/walks/gogae5/w50_gatekeeper/IMPL_GUIDE.md",
    "docs/ssot/walks/gogae5/w50_gatekeeper/GOLDEN_TESTS.md",
    "docs/ssot/walks/gogae5/w51_llm_bridge/SPEC.md",
    "docs/ssot/walks/gogae5/w51_llm_bridge/IMPL_GUIDE.md",
    "docs/ssot/walks/gogae5/w51_llm_bridge/GOLDEN_TESTS.md",
)

PROPOSAL_REQUIREMENTS: dict[str, tuple[str, ...]] = {
    "docs/context/proposals/PROPOSAL_EXTERNAL_INTENT_BOUNDARY_V1_20260323.md": (
        "external_intent_boundary_v1",
        "W45/W49/W50/W51",
    ),
    "docs/context/proposals/PROPOSAL_SEULGI_V1_20260323.md": (
        "pack/seulgi_v1",
        "W45/W49/W50/W51",
    ),
}

AGE3_GATE_REQUIRED_TOKENS: tuple[str, ...] = (
    "tests/run_external_intent_boundary_pack_check.py",
    "tests/run_seulgi_v1_pack_check.py",
    "tests/run_sam_inputsnapshot_contract_pack_check.py",
    "tests/run_sam_ai_ordering_pack_check.py",
    "tests/run_seulgi_gatekeeper_pack_check.py",
    "tests/run_external_intent_seulgi_walk_alignment_check.py",
    "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py",
    "external_intent_boundary_pack_pass",
    "seulgi_v1_pack_pass",
    "sam_inputsnapshot_contract_pack_pass",
    "sam_ai_ordering_pack_pass",
    "seulgi_gatekeeper_pack_pass",
    "external_intent_seulgi_walk_alignment_pass",
    "seamgrim_wasm_web_step_check_pass",
    "seamgrim_wasm_web_step_check_report_path",
)

AGE3_GATE_SELFTEST_REQUIRED_TOKENS: tuple[str, ...] = (
    "external_intent_boundary_pack_pass",
    "seulgi_v1_pack_pass",
    "sam_inputsnapshot_contract_pack_pass",
    "sam_ai_ordering_pack_pass",
    "seulgi_gatekeeper_pack_pass",
    "external_intent_seulgi_walk_alignment_pass",
    "seamgrim_wasm_web_step_check_pass",
    "seamgrim_wasm_web_step_check_report_path",
    "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1",
)

CI_SANITY_REQUIRED_TOKENS: tuple[str, ...] = (
    "sam_seulgi_family_contract_selftest",
    "tests/run_sam_seulgi_family_contract_selftest.py",
    "DDN_SAM_SEULGI_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON",
    "sam_seulgi_family_contract_selftest_current_probe=",
    "sam_seulgi_family_contract_selftest_last_completed_probe=",
    "sam_seulgi_family_contract_selftest_completed_checks=",
    "sam_seulgi_family_contract_selftest_total_checks=",
    "sam_seulgi_family_contract_selftest_checks_text=",
)
SEAMGRIM_CI_GATE_REQUIRED_TOKENS: tuple[str, ...] = (
    '"sam_seulgi_family_contract_selftest"',
    "tests/run_sam_seulgi_family_contract_selftest.py",
)
SEAMGRIM_PROFILE_REQUIRED_STEPS: tuple[str, ...] = (
    "sam_seulgi_family_contract_selftest",
    "external_intent_seulgi_walk_alignment_check_selftest",
)


def extract_set_block(text: str, anchor: str) -> str:
    anchor_index = text.find(anchor)
    if anchor_index < 0:
        raise ValueError(f"anchor missing: {anchor}")
    brace_start = text.find("{", anchor_index)
    if brace_start < 0:
        raise ValueError(f"set opening brace missing after anchor: {anchor}")
    depth = 0
    for index in range(brace_start, len(text)):
        char = text[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return text[brace_start : index + 1]
    raise ValueError(f"set closing brace missing after anchor: {anchor}")


def fail(code: str, msg: str) -> int:
    print(f"[external-intent-seulgi-walk-alignment] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError as exc:
        raise ValueError(f"cannot read {path}: {exc}")


def verify_path_exists(root: Path, rel_path: str) -> bool:
    return (root / rel_path).exists()


def main() -> int:
    parser = argparse.ArgumentParser(description="Check external-intent/seulgi walk alignment contracts")
    parser.add_argument(
        "--repo-root",
        default=".",
        help="repository root path",
    )
    args = parser.parse_args()

    root = Path(args.repo_root).resolve()

    missing_pack_files: list[str] = []
    for pack_root, files in PACK_FILE_REQUIREMENTS.items():
        for name in files:
            rel_path = f"{pack_root}/{name}"
            if not verify_path_exists(root, rel_path):
                missing_pack_files.append(rel_path)
    if missing_pack_files:
        return fail("E_ALIGNMENT_PACK_FILES_MISSING", ",".join(missing_pack_files))

    missing_walk_files = [path for path in WALK_FILE_REQUIREMENTS if not verify_path_exists(root, path)]
    if missing_walk_files:
        return fail("E_ALIGNMENT_WALK_FILES_MISSING", ",".join(missing_walk_files))

    for proposal_path, tokens in PROPOSAL_REQUIREMENTS.items():
        full_path = root / proposal_path
        if not full_path.exists():
            return fail("E_ALIGNMENT_PROPOSAL_MISSING", proposal_path)
        try:
            text = load_text(full_path)
        except ValueError as exc:
            return fail("E_ALIGNMENT_PROPOSAL_READ", str(exc))
        for token in tokens:
            if token not in text:
                return fail("E_ALIGNMENT_PROPOSAL_TOKEN", f"{proposal_path}:{token}")

    age3_gate_path = root / "tests/run_age3_completion_gate.py"
    try:
        age3_gate_text = load_text(age3_gate_path)
    except ValueError as exc:
        return fail("E_ALIGNMENT_AGE3_GATE_READ", str(exc))
    for token in AGE3_GATE_REQUIRED_TOKENS:
        if token not in age3_gate_text:
            return fail("E_ALIGNMENT_AGE3_GATE_TOKEN", token)

    age3_gate_selftest_path = root / "tests/run_age3_completion_gate_selftest.py"
    try:
        age3_gate_selftest_text = load_text(age3_gate_selftest_path)
    except ValueError as exc:
        return fail("E_ALIGNMENT_AGE3_GATE_SELFTEST_READ", str(exc))
    for token in AGE3_GATE_SELFTEST_REQUIRED_TOKENS:
        if token not in age3_gate_selftest_text:
            return fail("E_ALIGNMENT_AGE3_GATE_SELFTEST_TOKEN", token)

    ci_sanity_path = root / "tests/run_ci_sanity_gate.py"
    try:
        ci_sanity_text = load_text(ci_sanity_path)
    except ValueError as exc:
        return fail("E_ALIGNMENT_CI_SANITY_READ", str(exc))
    for token in CI_SANITY_REQUIRED_TOKENS:
        if token not in ci_sanity_text:
            return fail("E_ALIGNMENT_CI_SANITY_TOKEN", token)
    seamgrim_ci_gate_path = root / "tests/run_seamgrim_ci_gate.py"
    try:
        seamgrim_ci_gate_text = load_text(seamgrim_ci_gate_path)
    except ValueError as exc:
        return fail("E_ALIGNMENT_SEAMGRIM_CI_GATE_READ", str(exc))
    for token in SEAMGRIM_CI_GATE_REQUIRED_TOKENS:
        if token not in seamgrim_ci_gate_text:
            return fail("E_ALIGNMENT_SEAMGRIM_CI_GATE_TOKEN", token)
    try:
        seamgrim_profile_block = extract_set_block(ci_sanity_text, "SEAMGRIM_PROFILE_STEPS = {")
    except ValueError as exc:
        return fail("E_ALIGNMENT_CI_SANITY_SEAMGRIM_PROFILE_READ", str(exc))
    for token in SEAMGRIM_PROFILE_REQUIRED_STEPS:
        if token not in seamgrim_profile_block:
            return fail("E_ALIGNMENT_CI_SANITY_SEAMGRIM_PROFILE_TOKEN", token)

    print("[external-intent-seulgi-walk-alignment] ok")
    print(f"pack_roots={len(PACK_FILE_REQUIREMENTS)}")
    print(f"walk_files={len(WALK_FILE_REQUIREMENTS)}")
    print(f"proposal_docs={len(PROPOSAL_REQUIREMENTS)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
