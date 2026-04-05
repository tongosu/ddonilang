from __future__ import annotations

AGE3_COMPLETION_GATE_CRITERIA_NAMES: tuple[str, ...] = (
    "age3_ssot_walk_pack_contract_sync",
    "gogae7_pack_set_pass",
    "bogae_backend_profile_smoke_pass",
    "lang_teulcli_parser_parity_selftest_pass",
    "diag_fixit_selftest_pass",
    "lang_maegim_smoke_pack_pass",
    "lang_unit_temp_smoke_pack_pass",
    "gate0_contract_abort_state_check_pass",
    "block_editor_roundtrip_check_pass",
    "seamgrim_wasm_canon_contract_check_pass",
    "proof_runtime_minimum_check_pass",
    "seamgrim_wasm_web_smoke_contract_pass",
    "seamgrim_wasm_web_step_check_pass",
    "bogae_geoul_visibility_smoke_pass",
    "external_intent_boundary_pack_pass",
    "seulgi_v1_pack_pass",
    "external_intent_seulgi_walk_alignment_pass",
    "age3_doc_paths_exist",
)


def age3_completion_gate_criteria_summary_key(name: str) -> str:
    return f"ci_sanity_age3_completion_gate_criteria_{name}_ok"


def age3_completion_gate_criteria_sync_summary_key(name: str) -> str:
    return f"ci_sync_readiness_ci_sanity_age3_completion_gate_criteria_{name}_ok"


AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS: tuple[str, ...] = tuple(
    age3_completion_gate_criteria_summary_key(name)
    for name in AGE3_COMPLETION_GATE_CRITERIA_NAMES
)

AGE3_COMPLETION_GATE_CRITERIA_SYNC_SUMMARY_KEYS: tuple[str, ...] = tuple(
    age3_completion_gate_criteria_sync_summary_key(name)
    for name in AGE3_COMPLETION_GATE_CRITERIA_NAMES
)

AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS: tuple[tuple[str, str], ...] = tuple(
    zip(
        AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS,
        AGE3_COMPLETION_GATE_CRITERIA_SYNC_SUMMARY_KEYS,
    )
)
