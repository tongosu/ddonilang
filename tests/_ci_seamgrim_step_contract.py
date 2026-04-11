from __future__ import annotations

from typing import Iterable


# Canonical seamgrim step-contract set shared by sanity/sync/report-index/aggregate diagnostics.
SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS: tuple[str, ...] = (
    "seamgrim_ci_gate_seed_meta_step_check",
    "seamgrim_ci_gate_sam_seulgi_family_step_check",
    "seamgrim_ci_gate_runtime5_passthrough_check",
    "seamgrim_ci_gate_lesson_warning_step_check",
    "seamgrim_ci_gate_stateful_preview_step_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
    "seamgrim_wasm_cli_diag_parity_check",
)


def merge_step_names(base: Iterable[str], required: Iterable[str]) -> tuple[str, ...]:
    out: list[str] = []
    seen: set[str] = set()
    for raw in base:
        step = str(raw).strip()
        if not step or step in seen:
            continue
        out.append(step)
        seen.add(step)
    for raw in required:
        step = str(raw).strip()
        if not step or step in seen:
            continue
        out.append(step)
        seen.add(step)
    return tuple(out)
