from __future__ import annotations

from pathlib import Path
from typing import Iterable


# Canonical seamgrim step-contract set shared by sanity/sync/report-index/aggregate diagnostics.
SEAMGRIM_BLOCKER_STEP_CONTRACT_STEPS: tuple[str, ...] = (
    "seamgrim_product_blocker_bundle_check",
    "seamgrim_observe_output_contract_check",
    "seamgrim_runtime_view_source_strict_check",
    "seamgrim_view_only_state_hash_invariant_check",
    "seamgrim_run_legacy_autofix_check",
)

SEAMGRIM_BLOCKER_STEP_SCRIPT_PATHS: tuple[tuple[str, str], ...] = (
    ("seamgrim_product_blocker_bundle_check", "tests/run_seamgrim_product_blocker_bundle_check.py"),
    ("seamgrim_observe_output_contract_check", "tests/run_seamgrim_observe_output_contract_check.py"),
    ("seamgrim_runtime_view_source_strict_check", "tests/run_seamgrim_runtime_view_source_strict_check.py"),
    (
        "seamgrim_view_only_state_hash_invariant_check",
        "tests/run_seamgrim_view_only_state_hash_invariant_check.py",
    ),
    ("seamgrim_run_legacy_autofix_check", "tests/run_seamgrim_run_legacy_autofix_check.py"),
)
SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME: dict[str, str] = dict(SEAMGRIM_BLOCKER_STEP_SCRIPT_PATHS)

SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS: tuple[tuple[str, str], ...] = (
    ("ci_sanity_seamgrim_product_blocker_bundle_check_ok", "seamgrim_product_blocker_bundle_check"),
    ("ci_sanity_seamgrim_observe_output_contract_check_ok", "seamgrim_observe_output_contract_check"),
    ("ci_sanity_seamgrim_runtime_view_source_strict_check_ok", "seamgrim_runtime_view_source_strict_check"),
    (
        "ci_sanity_seamgrim_view_only_state_hash_invariant_check_ok",
        "seamgrim_view_only_state_hash_invariant_check",
    ),
    ("ci_sanity_seamgrim_run_legacy_autofix_check_ok", "seamgrim_run_legacy_autofix_check"),
)

SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_FIELDS: tuple[tuple[str, str], ...] = (
    ("seamgrim_product_blocker_bundle_check", "E_CI_SANITY_SEAMGRIM_PRODUCT_BLOCKER_BUNDLE_FAIL"),
    ("seamgrim_observe_output_contract_check", "E_CI_SANITY_SEAMGRIM_OBSERVE_OUTPUT_CONTRACT_FAIL"),
    ("seamgrim_runtime_view_source_strict_check", "E_CI_SANITY_SEAMGRIM_RUNTIME_VIEW_SOURCE_STRICT_FAIL"),
    (
        "seamgrim_view_only_state_hash_invariant_check",
        "E_CI_SANITY_SEAMGRIM_VIEW_ONLY_STATE_HASH_INVARIANT_FAIL",
    ),
    ("seamgrim_run_legacy_autofix_check", "E_CI_SANITY_SEAMGRIM_RUN_LEGACY_AUTOFIX_FAIL"),
)
SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_BY_STEP: dict[str, str] = dict(SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_FIELDS)

SEAMGRIM_PLATFORM_STEP_CONTRACT_STEPS: tuple[str, ...] = (
    "seamgrim_auth_save_surface_check",
    "seamgrim_object_revision_surface_check",
    "seamgrim_sharing_publishing_surface_check",
    "seamgrim_package_registry_surface_check",
    "seamgrim_source_management_surface_check",
    "seamgrim_publication_snapshot_surface_check",
    "seamgrim_review_workflow_surface_check",
    "seamgrim_platform_mock_interface_contract_check",
    "seamgrim_platform_mock_menu_mode_check",
    "seamgrim_platform_mock_adapter_roundtrip_check",
    "seamgrim_platform_mock_payload_snapshot_check",
    "seamgrim_platform_server_adapter_contract_check",
    "seamgrim_platform_route_precedence_check",
    "seamgrim_platform_server_action_rail_check",
)

SEAMGRIM_PLATFORM_STEP_SCRIPT_PATHS: tuple[tuple[str, str], ...] = (
    ("seamgrim_auth_save_surface_check", "tests/run_seamgrim_auth_save_surface_check.py"),
    ("seamgrim_object_revision_surface_check", "tests/run_seamgrim_object_revision_surface_check.py"),
    ("seamgrim_sharing_publishing_surface_check", "tests/run_seamgrim_sharing_publishing_surface_check.py"),
    ("seamgrim_package_registry_surface_check", "tests/run_seamgrim_package_registry_surface_check.py"),
    ("seamgrim_source_management_surface_check", "tests/run_seamgrim_source_management_surface_check.py"),
    (
        "seamgrim_publication_snapshot_surface_check",
        "tests/run_seamgrim_publication_snapshot_surface_check.py",
    ),
    ("seamgrim_review_workflow_surface_check", "tests/run_seamgrim_review_workflow_surface_check.py"),
    (
        "seamgrim_platform_mock_interface_contract_check",
        "tests/run_seamgrim_platform_mock_interface_contract_check.py",
    ),
    (
        "seamgrim_platform_mock_menu_mode_check",
        "tests/run_seamgrim_platform_mock_menu_mode_check.py",
    ),
    (
        "seamgrim_platform_mock_adapter_roundtrip_check",
        "tests/run_seamgrim_platform_mock_adapter_roundtrip_check.py",
    ),
    (
        "seamgrim_platform_mock_payload_snapshot_check",
        "tests/run_seamgrim_platform_mock_payload_snapshot_check.py",
    ),
    (
        "seamgrim_platform_server_adapter_contract_check",
        "tests/run_seamgrim_platform_server_adapter_contract_check.py",
    ),
    (
        "seamgrim_platform_route_precedence_check",
        "tests/run_seamgrim_platform_route_precedence_check.py",
    ),
    (
        "seamgrim_platform_server_action_rail_check",
        "tests/run_seamgrim_platform_server_action_rail_check.py",
    ),
)
SEAMGRIM_PLATFORM_STEP_SCRIPT_PATH_BY_NAME: dict[str, str] = dict(SEAMGRIM_PLATFORM_STEP_SCRIPT_PATHS)

SEAMGRIM_PLATFORM_SANITY_SUMMARY_STEP_FIELDS: tuple[tuple[str, str], ...] = (
    ("ci_sanity_seamgrim_auth_save_surface_check_ok", "seamgrim_auth_save_surface_check"),
    ("ci_sanity_seamgrim_object_revision_surface_check_ok", "seamgrim_object_revision_surface_check"),
    ("ci_sanity_seamgrim_sharing_publishing_surface_check_ok", "seamgrim_sharing_publishing_surface_check"),
    ("ci_sanity_seamgrim_package_registry_surface_check_ok", "seamgrim_package_registry_surface_check"),
    ("ci_sanity_seamgrim_source_management_surface_check_ok", "seamgrim_source_management_surface_check"),
    (
        "ci_sanity_seamgrim_publication_snapshot_surface_check_ok",
        "seamgrim_publication_snapshot_surface_check",
    ),
    ("ci_sanity_seamgrim_review_workflow_surface_check_ok", "seamgrim_review_workflow_surface_check"),
    (
        "ci_sanity_seamgrim_platform_mock_interface_contract_check_ok",
        "seamgrim_platform_mock_interface_contract_check",
    ),
    (
        "ci_sanity_seamgrim_platform_mock_menu_mode_check_ok",
        "seamgrim_platform_mock_menu_mode_check",
    ),
    (
        "ci_sanity_seamgrim_platform_mock_adapter_roundtrip_check_ok",
        "seamgrim_platform_mock_adapter_roundtrip_check",
    ),
    (
        "ci_sanity_seamgrim_platform_mock_payload_snapshot_check_ok",
        "seamgrim_platform_mock_payload_snapshot_check",
    ),
    (
        "ci_sanity_seamgrim_platform_server_adapter_contract_check_ok",
        "seamgrim_platform_server_adapter_contract_check",
    ),
    (
        "ci_sanity_seamgrim_platform_route_precedence_check_ok",
        "seamgrim_platform_route_precedence_check",
    ),
    (
        "ci_sanity_seamgrim_platform_server_action_rail_check_ok",
        "seamgrim_platform_server_action_rail_check",
    ),
)

SEAMGRIM_PLATFORM_SANITY_FAIL_CODE_FIELDS: tuple[tuple[str, str], ...] = (
    ("seamgrim_auth_save_surface_check", "E_CI_SANITY_SEAMGRIM_AUTH_SAVE_SURFACE_FAIL"),
    ("seamgrim_object_revision_surface_check", "E_CI_SANITY_SEAMGRIM_OBJECT_REVISION_SURFACE_FAIL"),
    ("seamgrim_sharing_publishing_surface_check", "E_CI_SANITY_SEAMGRIM_SHARING_PUBLISHING_SURFACE_FAIL"),
    ("seamgrim_package_registry_surface_check", "E_CI_SANITY_SEAMGRIM_PACKAGE_REGISTRY_SURFACE_FAIL"),
    ("seamgrim_source_management_surface_check", "E_CI_SANITY_SEAMGRIM_SOURCE_MANAGEMENT_SURFACE_FAIL"),
    (
        "seamgrim_publication_snapshot_surface_check",
        "E_CI_SANITY_SEAMGRIM_PUBLICATION_SNAPSHOT_SURFACE_FAIL",
    ),
    ("seamgrim_review_workflow_surface_check", "E_CI_SANITY_SEAMGRIM_REVIEW_WORKFLOW_SURFACE_FAIL"),
    (
        "seamgrim_platform_mock_interface_contract_check",
        "E_CI_SANITY_SEAMGRIM_PLATFORM_MOCK_INTERFACE_CONTRACT_FAIL",
    ),
    (
        "seamgrim_platform_mock_menu_mode_check",
        "E_CI_SANITY_SEAMGRIM_PLATFORM_MOCK_MENU_MODE_FAIL",
    ),
    (
        "seamgrim_platform_mock_adapter_roundtrip_check",
        "E_CI_SANITY_SEAMGRIM_PLATFORM_MOCK_ADAPTER_ROUNDTRIP_FAIL",
    ),
    (
        "seamgrim_platform_mock_payload_snapshot_check",
        "E_CI_SANITY_SEAMGRIM_PLATFORM_MOCK_PAYLOAD_SNAPSHOT_FAIL",
    ),
    (
        "seamgrim_platform_server_adapter_contract_check",
        "E_CI_SANITY_SEAMGRIM_PLATFORM_SERVER_ADAPTER_CONTRACT_FAIL",
    ),
    (
        "seamgrim_platform_route_precedence_check",
        "E_CI_SANITY_SEAMGRIM_PLATFORM_ROUTE_PRECEDENCE_FAIL",
    ),
    (
        "seamgrim_platform_server_action_rail_check",
        "E_CI_SANITY_SEAMGRIM_PLATFORM_SERVER_ACTION_RAIL_FAIL",
    ),
)
SEAMGRIM_PLATFORM_SANITY_FAIL_CODE_BY_STEP: dict[str, str] = dict(SEAMGRIM_PLATFORM_SANITY_FAIL_CODE_FIELDS)

SEAMGRIM_PLATFORM_SYNC_READINESS_VALIDATE_ONLY_CASES: tuple[tuple[str, str, str, str], ...] = (
    (
        "seamgrim_auth_save_surface_check",
        "auth_save_surface",
        "validate_only_missing_auth_save_surface_should_fail",
        "validate_missing_auth_save_surface_msg_should_mention_step",
    ),
    (
        "seamgrim_object_revision_surface_check",
        "object_revision_surface",
        "validate_only_missing_object_revision_surface_should_fail",
        "validate_missing_object_revision_surface_msg_should_mention_step",
    ),
    (
        "seamgrim_sharing_publishing_surface_check",
        "sharing_publishing_surface",
        "validate_only_missing_sharing_publishing_surface_should_fail",
        "validate_missing_sharing_publishing_surface_msg_should_mention_step",
    ),
    (
        "seamgrim_package_registry_surface_check",
        "package_registry_surface",
        "validate_only_missing_package_registry_surface_should_fail",
        "validate_missing_package_registry_surface_msg_should_mention_step",
    ),
    (
        "seamgrim_source_management_surface_check",
        "source_management_surface",
        "validate_only_missing_source_management_surface_should_fail",
        "validate_missing_source_management_surface_msg_should_mention_step",
    ),
    (
        "seamgrim_publication_snapshot_surface_check",
        "publication_snapshot_surface",
        "validate_only_missing_publication_snapshot_surface_should_fail",
        "validate_missing_publication_snapshot_surface_msg_should_mention_step",
    ),
    (
        "seamgrim_review_workflow_surface_check",
        "review_workflow_surface",
        "validate_only_missing_review_workflow_surface_should_fail",
        "validate_missing_review_workflow_surface_msg_should_mention_step",
    ),
    (
        "seamgrim_platform_mock_interface_contract_check",
        "platform_mock_interface_contract",
        "validate_only_missing_platform_mock_interface_contract_should_fail",
        "validate_missing_platform_mock_interface_contract_msg_should_mention_step",
    ),
    (
        "seamgrim_platform_mock_menu_mode_check",
        "platform_mock_menu_mode",
        "validate_only_missing_platform_mock_menu_mode_should_fail",
        "validate_missing_platform_mock_menu_mode_msg_should_mention_step",
    ),
    (
        "seamgrim_platform_mock_adapter_roundtrip_check",
        "platform_mock_adapter_roundtrip",
        "validate_only_missing_platform_mock_adapter_roundtrip_should_fail",
        "validate_missing_platform_mock_adapter_roundtrip_msg_should_mention_step",
    ),
    (
        "seamgrim_platform_mock_payload_snapshot_check",
        "platform_mock_payload_snapshot",
        "validate_only_missing_platform_mock_payload_snapshot_should_fail",
        "validate_missing_platform_mock_payload_snapshot_msg_should_mention_step",
    ),
    (
        "seamgrim_platform_server_adapter_contract_check",
        "platform_server_adapter_contract",
        "validate_only_missing_platform_server_adapter_contract_should_fail",
        "validate_missing_platform_server_adapter_contract_msg_should_mention_step",
    ),
    (
        "seamgrim_platform_route_precedence_check",
        "platform_route_precedence",
        "validate_only_missing_platform_route_precedence_should_fail",
        "validate_missing_platform_route_precedence_msg_should_mention_step",
    ),
    (
        "seamgrim_platform_server_action_rail_check",
        "platform_server_action_rail",
        "validate_only_missing_platform_server_action_rail_should_fail",
        "validate_missing_platform_server_action_rail_msg_should_mention_step",
    ),
)

SEAMGRIM_PLATFORM_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES: tuple[tuple[str, str, str, str], ...] = (
    (
        "seamgrim_auth_save_surface_check",
        "auth_save_surface",
        "validate_only_failed_auth_save_surface_should_fail",
        "validate_failed_auth_save_surface_msg_should_mention_step",
    ),
    (
        "seamgrim_object_revision_surface_check",
        "object_revision_surface",
        "validate_only_failed_object_revision_surface_should_fail",
        "validate_failed_object_revision_surface_msg_should_mention_step",
    ),
    (
        "seamgrim_sharing_publishing_surface_check",
        "sharing_publishing_surface",
        "validate_only_failed_sharing_publishing_surface_should_fail",
        "validate_failed_sharing_publishing_surface_msg_should_mention_step",
    ),
    (
        "seamgrim_package_registry_surface_check",
        "package_registry_surface",
        "validate_only_failed_package_registry_surface_should_fail",
        "validate_failed_package_registry_surface_msg_should_mention_step",
    ),
    (
        "seamgrim_source_management_surface_check",
        "source_management_surface",
        "validate_only_failed_source_management_surface_should_fail",
        "validate_failed_source_management_surface_msg_should_mention_step",
    ),
    (
        "seamgrim_publication_snapshot_surface_check",
        "publication_snapshot_surface",
        "validate_only_failed_publication_snapshot_surface_should_fail",
        "validate_failed_publication_snapshot_surface_msg_should_mention_step",
    ),
    (
        "seamgrim_review_workflow_surface_check",
        "review_workflow_surface",
        "validate_only_failed_review_workflow_surface_should_fail",
        "validate_failed_review_workflow_surface_msg_should_mention_step",
    ),
    (
        "seamgrim_platform_mock_interface_contract_check",
        "platform_mock_interface_contract",
        "validate_only_failed_platform_mock_interface_contract_should_fail",
        "validate_failed_platform_mock_interface_contract_msg_should_mention_step",
    ),
    (
        "seamgrim_platform_mock_menu_mode_check",
        "platform_mock_menu_mode",
        "validate_only_failed_platform_mock_menu_mode_should_fail",
        "validate_failed_platform_mock_menu_mode_msg_should_mention_step",
    ),
    (
        "seamgrim_platform_mock_adapter_roundtrip_check",
        "platform_mock_adapter_roundtrip",
        "validate_only_failed_platform_mock_adapter_roundtrip_should_fail",
        "validate_failed_platform_mock_adapter_roundtrip_msg_should_mention_step",
    ),
    (
        "seamgrim_platform_mock_payload_snapshot_check",
        "platform_mock_payload_snapshot",
        "validate_only_failed_platform_mock_payload_snapshot_should_fail",
        "validate_failed_platform_mock_payload_snapshot_msg_should_mention_step",
    ),
    (
        "seamgrim_platform_server_adapter_contract_check",
        "platform_server_adapter_contract",
        "validate_only_failed_platform_server_adapter_contract_should_fail",
        "validate_failed_platform_server_adapter_contract_msg_should_mention_step",
    ),
    (
        "seamgrim_platform_route_precedence_check",
        "platform_route_precedence",
        "validate_only_failed_platform_route_precedence_should_fail",
        "validate_failed_platform_route_precedence_msg_should_mention_step",
    ),
    (
        "seamgrim_platform_server_action_rail_check",
        "platform_server_action_rail",
        "validate_only_failed_platform_server_action_rail_should_fail",
        "validate_failed_platform_server_action_rail_msg_should_mention_step",
    ),
)

SEAMGRIM_PLATFORM_GATE_REPORT_INDEX_MISSING_STEP_CASES: tuple[tuple[str, str, str], ...] = (
    (
        "seamgrim_auth_save_surface_check",
        "seamgrim profile missing auth/save surface step case must fail",
        "seamgrim profile missing auth/save surface step code mismatch",
    ),
    (
        "seamgrim_object_revision_surface_check",
        "seamgrim profile missing object/revision surface step case must fail",
        "seamgrim profile missing object/revision surface step code mismatch",
    ),
    (
        "seamgrim_sharing_publishing_surface_check",
        "seamgrim profile missing sharing/publishing surface step case must fail",
        "seamgrim profile missing sharing/publishing surface step code mismatch",
    ),
    (
        "seamgrim_package_registry_surface_check",
        "seamgrim profile missing package/registry surface step case must fail",
        "seamgrim profile missing package/registry surface step code mismatch",
    ),
    (
        "seamgrim_source_management_surface_check",
        "seamgrim profile missing source management surface step case must fail",
        "seamgrim profile missing source management surface step code mismatch",
    ),
    (
        "seamgrim_publication_snapshot_surface_check",
        "seamgrim profile missing publication snapshot surface step case must fail",
        "seamgrim profile missing publication snapshot surface step code mismatch",
    ),
    (
        "seamgrim_review_workflow_surface_check",
        "seamgrim profile missing review workflow surface step case must fail",
        "seamgrim profile missing review workflow surface step code mismatch",
    ),
    (
        "seamgrim_platform_mock_interface_contract_check",
        "seamgrim profile missing platform mock interface contract step case must fail",
        "seamgrim profile missing platform mock interface contract step code mismatch",
    ),
    (
        "seamgrim_platform_mock_menu_mode_check",
        "seamgrim profile missing platform mock menu mode step case must fail",
        "seamgrim profile missing platform mock menu mode step code mismatch",
    ),
    (
        "seamgrim_platform_mock_adapter_roundtrip_check",
        "seamgrim profile missing platform mock adapter roundtrip step case must fail",
        "seamgrim profile missing platform mock adapter roundtrip step code mismatch",
    ),
    (
        "seamgrim_platform_mock_payload_snapshot_check",
        "seamgrim profile missing platform mock payload snapshot step case must fail",
        "seamgrim profile missing platform mock payload snapshot step code mismatch",
    ),
    (
        "seamgrim_platform_server_adapter_contract_check",
        "seamgrim profile missing platform server adapter contract step case must fail",
        "seamgrim profile missing platform server adapter contract step code mismatch",
    ),
    (
        "seamgrim_platform_route_precedence_check",
        "seamgrim profile missing platform route precedence step case must fail",
        "seamgrim profile missing platform route precedence step code mismatch",
    ),
    (
        "seamgrim_platform_server_action_rail_check",
        "seamgrim profile missing platform server action rail step case must fail",
        "seamgrim profile missing platform server action rail step code mismatch",
    ),
)

SEAMGRIM_PLATFORM_GATE_REPORT_INDEX_FAILED_STEP_CASES: tuple[tuple[str, str, str], ...] = (
    (
        "seamgrim_auth_save_surface_check",
        "seamgrim profile failed auth/save surface step case must fail",
        "seamgrim profile failed auth/save surface step code mismatch",
    ),
    (
        "seamgrim_object_revision_surface_check",
        "seamgrim profile failed object/revision surface step case must fail",
        "seamgrim profile failed object/revision surface step code mismatch",
    ),
    (
        "seamgrim_sharing_publishing_surface_check",
        "seamgrim profile failed sharing/publishing surface step case must fail",
        "seamgrim profile failed sharing/publishing surface step code mismatch",
    ),
    (
        "seamgrim_package_registry_surface_check",
        "seamgrim profile failed package/registry surface step case must fail",
        "seamgrim profile failed package/registry surface step code mismatch",
    ),
    (
        "seamgrim_source_management_surface_check",
        "seamgrim profile failed source management surface step case must fail",
        "seamgrim profile failed source management surface step code mismatch",
    ),
    (
        "seamgrim_publication_snapshot_surface_check",
        "seamgrim profile failed publication snapshot surface step case must fail",
        "seamgrim profile failed publication snapshot surface step code mismatch",
    ),
    (
        "seamgrim_review_workflow_surface_check",
        "seamgrim profile failed review workflow surface step case must fail",
        "seamgrim profile failed review workflow surface step code mismatch",
    ),
    (
        "seamgrim_platform_mock_interface_contract_check",
        "seamgrim profile failed platform mock interface contract step case must fail",
        "seamgrim profile failed platform mock interface contract step code mismatch",
    ),
    (
        "seamgrim_platform_mock_menu_mode_check",
        "seamgrim profile failed platform mock menu mode step case must fail",
        "seamgrim profile failed platform mock menu mode step code mismatch",
    ),
    (
        "seamgrim_platform_mock_adapter_roundtrip_check",
        "seamgrim profile failed platform mock adapter roundtrip step case must fail",
        "seamgrim profile failed platform mock adapter roundtrip step code mismatch",
    ),
    (
        "seamgrim_platform_mock_payload_snapshot_check",
        "seamgrim profile failed platform mock payload snapshot step case must fail",
        "seamgrim profile failed platform mock payload snapshot step code mismatch",
    ),
    (
        "seamgrim_platform_server_adapter_contract_check",
        "seamgrim profile failed platform server adapter contract step case must fail",
        "seamgrim profile failed platform server adapter contract step code mismatch",
    ),
    (
        "seamgrim_platform_route_precedence_check",
        "seamgrim profile failed platform route precedence step case must fail",
        "seamgrim profile failed platform route precedence step code mismatch",
    ),
    (
        "seamgrim_platform_server_action_rail_check",
        "seamgrim profile failed platform server action rail step case must fail",
        "seamgrim profile failed platform server action rail step code mismatch",
    ),
)

SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES: tuple[tuple[str, str, str, str], ...] = (
    (
        "seamgrim_product_blocker_bundle_check",
        "product_blocker_bundle",
        "validate_only_missing_product_blocker_bundle_should_fail",
        "validate_missing_product_blocker_bundle_msg_should_mention_step",
    ),
    (
        "seamgrim_observe_output_contract_check",
        "observe_output_contract",
        "validate_only_missing_observe_output_contract_should_fail",
        "validate_missing_observe_output_contract_msg_should_mention_step",
    ),
    (
        "seamgrim_runtime_view_source_strict_check",
        "runtime_view_source_strict",
        "validate_only_missing_runtime_view_source_strict_should_fail",
        "validate_missing_runtime_view_source_strict_msg_should_mention_step",
    ),
    (
        "seamgrim_view_only_state_hash_invariant_check",
        "view_only_state_hash_invariant",
        "validate_only_missing_view_only_state_hash_invariant_should_fail",
        "validate_missing_view_only_state_hash_invariant_msg_should_mention_step",
    ),
    (
        "seamgrim_run_legacy_autofix_check",
        "run_legacy_autofix",
        "validate_only_missing_run_legacy_autofix_should_fail",
        "validate_missing_run_legacy_autofix_msg_should_mention_step",
    ),
)

SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES: tuple[tuple[str, str, str, str], ...] = (
    (
        "seamgrim_product_blocker_bundle_check",
        "product_blocker_bundle",
        "validate_only_failed_product_blocker_bundle_should_fail",
        "validate_failed_product_blocker_bundle_msg_should_mention_step",
    ),
    (
        "seamgrim_observe_output_contract_check",
        "observe_output_contract",
        "validate_only_failed_observe_output_contract_should_fail",
        "validate_failed_observe_output_contract_msg_should_mention_step",
    ),
    (
        "seamgrim_runtime_view_source_strict_check",
        "runtime_view_source_strict",
        "validate_only_failed_runtime_view_source_strict_should_fail",
        "validate_failed_runtime_view_source_strict_msg_should_mention_step",
    ),
    (
        "seamgrim_view_only_state_hash_invariant_check",
        "view_only_state_hash_invariant",
        "validate_only_failed_view_only_state_hash_invariant_should_fail",
        "validate_failed_view_only_state_hash_invariant_msg_should_mention_step",
    ),
    (
        "seamgrim_run_legacy_autofix_check",
        "run_legacy_autofix",
        "validate_only_failed_run_legacy_autofix_should_fail",
        "validate_failed_run_legacy_autofix_msg_should_mention_step",
    ),
)

SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_MISSING_STEP_CASES: tuple[tuple[str, str, str], ...] = (
    (
        "seamgrim_product_blocker_bundle_check",
        "seamgrim profile missing product blocker step case must fail",
        "seamgrim profile missing product blocker step code mismatch",
    ),
    (
        "seamgrim_observe_output_contract_check",
        "seamgrim profile missing observe output contract step case must fail",
        "seamgrim profile missing observe output contract step code mismatch",
    ),
    (
        "seamgrim_runtime_view_source_strict_check",
        "seamgrim profile missing runtime view-source strict step case must fail",
        "seamgrim profile missing runtime view-source strict step code mismatch",
    ),
    (
        "seamgrim_view_only_state_hash_invariant_check",
        "seamgrim profile missing view-only state-hash invariant step case must fail",
        "seamgrim profile missing view-only state-hash invariant step code mismatch",
    ),
    (
        "seamgrim_run_legacy_autofix_check",
        "seamgrim profile missing run legacy autofix step case must fail",
        "seamgrim profile missing run legacy autofix step code mismatch",
    ),
)

SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_FAILED_STEP_CASES: tuple[tuple[str, str, str], ...] = (
    (
        "seamgrim_product_blocker_bundle_check",
        "seamgrim profile failed product blocker step case must fail",
        "seamgrim profile failed product blocker step code mismatch",
    ),
    (
        "seamgrim_observe_output_contract_check",
        "seamgrim profile failed observe output contract step case must fail",
        "seamgrim profile failed observe output contract step code mismatch",
    ),
    (
        "seamgrim_runtime_view_source_strict_check",
        "seamgrim profile failed runtime view-source strict step case must fail",
        "seamgrim profile failed runtime view-source strict step code mismatch",
    ),
    (
        "seamgrim_view_only_state_hash_invariant_check",
        "seamgrim profile failed view-only state-hash invariant step case must fail",
        "seamgrim profile failed view-only state-hash invariant step code mismatch",
    ),
    (
        "seamgrim_run_legacy_autofix_check",
        "seamgrim profile failed run legacy autofix step case must fail",
        "seamgrim profile failed run legacy autofix step code mismatch",
    ),
)

SEAMGRIM_FEATURED_SEED_STEP_CONTRACT_STEPS: tuple[str, ...] = (
    "seamgrim_ci_gate_featured_seed_catalog_step_check",
    "seamgrim_ci_gate_featured_seed_catalog_autogen_step_check",
)

SEAMGRIM_FEATURED_SEED_STEP_SCRIPT_PATHS: tuple[tuple[str, str], ...] = (
    (
        "seamgrim_ci_gate_featured_seed_catalog_step_check",
        "tests/run_seamgrim_ci_gate_featured_seed_catalog_step_check.py",
    ),
    (
        "seamgrim_ci_gate_featured_seed_catalog_autogen_step_check",
        "tests/run_seamgrim_ci_gate_featured_seed_catalog_autogen_step_check.py",
    ),
)
SEAMGRIM_FEATURED_SEED_STEP_SCRIPT_PATH_BY_NAME: dict[str, str] = dict(SEAMGRIM_FEATURED_SEED_STEP_SCRIPT_PATHS)

SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_CASES: tuple[tuple[str, str, str, str], ...] = (
    (
        "seamgrim_ci_gate_featured_seed_catalog_step_check",
        "featured_seed_catalog",
        "validate_only_missing_featured_seed_catalog_should_fail",
        "validate_missing_featured_seed_catalog_msg_should_mention_step",
    ),
    (
        "seamgrim_ci_gate_featured_seed_catalog_autogen_step_check",
        "featured_seed_catalog_autogen",
        "validate_only_missing_featured_seed_catalog_autogen_should_fail",
        "validate_missing_featured_seed_catalog_autogen_msg_should_mention_step",
    ),
)

SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES: tuple[tuple[str, str, str, str], ...] = (
    (
        "seamgrim_ci_gate_featured_seed_catalog_step_check",
        "featured_seed_catalog",
        "validate_only_failed_featured_seed_catalog_should_fail",
        "validate_failed_featured_seed_catalog_msg_should_mention_step",
    ),
    (
        "seamgrim_ci_gate_featured_seed_catalog_autogen_step_check",
        "featured_seed_catalog_autogen",
        "validate_only_failed_featured_seed_catalog_autogen_should_fail",
        "validate_failed_featured_seed_catalog_autogen_msg_should_mention_step",
    ),
)

SEAMGRIM_FEATURED_GATE_REPORT_INDEX_MISSING_STEP_CASES: tuple[tuple[str, str, str], ...] = (
    (
        "seamgrim_ci_gate_featured_seed_catalog_step_check",
        "seamgrim profile missing featured seed catalog step case must fail",
        "seamgrim profile missing featured seed catalog step code mismatch",
    ),
    (
        "seamgrim_ci_gate_featured_seed_catalog_autogen_step_check",
        "seamgrim profile missing featured seed catalog autogen step case must fail",
        "seamgrim profile missing featured seed catalog autogen step code mismatch",
    ),
)

SEAMGRIM_FEATURED_GATE_REPORT_INDEX_FAILED_STEP_CASES: tuple[tuple[str, str, str], ...] = (
    (
        "seamgrim_ci_gate_featured_seed_catalog_step_check",
        "seamgrim profile failed featured seed catalog step case must fail",
        "seamgrim profile failed featured seed catalog step code mismatch",
    ),
    (
        "seamgrim_ci_gate_featured_seed_catalog_autogen_step_check",
        "seamgrim profile failed featured seed catalog autogen step case must fail",
        "seamgrim profile failed featured seed catalog autogen step code mismatch",
    ),
)

SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS: tuple[str, ...] = (
    "seamgrim_ci_gate_seed_meta_step_check",
    "seamgrim_ci_gate_worker_env_step_check",
    *SEAMGRIM_FEATURED_SEED_STEP_CONTRACT_STEPS,
    "seamgrim_ci_gate_sam_seulgi_family_step_check",
    "seamgrim_ci_gate_runtime5_passthrough_check",
    "seamgrim_ci_gate_lesson_warning_step_check",
    "seamgrim_ci_gate_stateful_preview_step_check",
    *SEAMGRIM_BLOCKER_STEP_CONTRACT_STEPS,
    *SEAMGRIM_PLATFORM_STEP_CONTRACT_STEPS,
    "seamgrim_v2_task_batch_check",
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


def collect_blocker_contract_issues() -> tuple[str, ...]:
    issues: list[str] = []

    canonical_steps = tuple(SEAMGRIM_BLOCKER_STEP_CONTRACT_STEPS)
    canonical_set = set(canonical_steps)
    if len(canonical_steps) != len(canonical_set):
        issues.append("SEAMGRIM_BLOCKER_STEP_CONTRACT_STEPS contains duplicate step names")

    script_steps = [step_name for step_name, _script_path in SEAMGRIM_BLOCKER_STEP_SCRIPT_PATHS]
    summary_steps = [step_name for _summary_key, step_name in SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS]
    fail_code_steps = [step_name for step_name, _code in SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_FIELDS]
    sync_case_steps = [step_name for step_name, _slug, _fail_label, _msg_label in SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES]
    sync_failed_case_steps = [
        step_name
        for step_name, _slug, _fail_label, _msg_label in SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES
    ]
    report_case_steps = [step_name for step_name, _fail_message, _code_message in SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_MISSING_STEP_CASES]
    report_failed_case_steps = [
        step_name for step_name, _fail_message, _code_message in SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_FAILED_STEP_CASES
    ]

    def _check_step_set(label: str, steps: list[str], expected_set: set[str]) -> None:
        step_set = set(steps)
        if len(step_set) != len(steps):
            issues.append(f"{label} contains duplicate step names")
        if step_set != expected_set:
            missing = sorted(expected_set - step_set)
            extra = sorted(step_set - expected_set)
            if missing:
                issues.append(f"{label} missing steps: {', '.join(missing)}")
            if extra:
                issues.append(f"{label} has unknown steps: {', '.join(extra)}")

    _check_step_set("SEAMGRIM_BLOCKER_STEP_SCRIPT_PATHS", script_steps, canonical_set)
    _check_step_set("SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS", summary_steps, canonical_set)
    _check_step_set("SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_FIELDS", fail_code_steps, canonical_set)
    _check_step_set("SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES", sync_case_steps, canonical_set)
    _check_step_set(
        "SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES",
        sync_failed_case_steps,
        canonical_set,
    )
    _check_step_set("SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_MISSING_STEP_CASES", report_case_steps, canonical_set)
    _check_step_set(
        "SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_FAILED_STEP_CASES",
        report_failed_case_steps,
        canonical_set,
    )

    featured_steps = tuple(SEAMGRIM_FEATURED_SEED_STEP_CONTRACT_STEPS)
    featured_set = set(featured_steps)
    if len(featured_steps) != len(featured_set):
        issues.append("SEAMGRIM_FEATURED_SEED_STEP_CONTRACT_STEPS contains duplicate step names")

    featured_script_steps = [step_name for step_name, _script_path in SEAMGRIM_FEATURED_SEED_STEP_SCRIPT_PATHS]
    featured_sync_case_steps = [
        step_name
        for step_name, _slug, _fail_label, _msg_label in SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_CASES
    ]
    featured_sync_failed_case_steps = [
        step_name
        for step_name, _slug, _fail_label, _msg_label in SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES
    ]
    featured_report_case_steps = [
        step_name for step_name, _fail_message, _code_message in SEAMGRIM_FEATURED_GATE_REPORT_INDEX_MISSING_STEP_CASES
    ]
    featured_report_failed_case_steps = [
        step_name for step_name, _fail_message, _code_message in SEAMGRIM_FEATURED_GATE_REPORT_INDEX_FAILED_STEP_CASES
    ]
    _check_step_set("SEAMGRIM_FEATURED_SEED_STEP_SCRIPT_PATHS", featured_script_steps, featured_set)
    _check_step_set(
        "SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_CASES",
        featured_sync_case_steps,
        featured_set,
    )
    _check_step_set(
        "SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES",
        featured_sync_failed_case_steps,
        featured_set,
    )
    _check_step_set(
        "SEAMGRIM_FEATURED_GATE_REPORT_INDEX_MISSING_STEP_CASES",
        featured_report_case_steps,
        featured_set,
    )
    _check_step_set(
        "SEAMGRIM_FEATURED_GATE_REPORT_INDEX_FAILED_STEP_CASES",
        featured_report_failed_case_steps,
        featured_set,
    )

    repo_root = Path(__file__).resolve().parent.parent
    summary_keys = [summary_key for summary_key, _step_name in SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS]
    if len(summary_keys) != len(set(summary_keys)):
        issues.append("SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS contains duplicate summary keys")

    fail_codes = [code for _step_name, code in SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_FIELDS]
    if len(fail_codes) != len(set(fail_codes)):
        issues.append("SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_FIELDS contains duplicate fail codes")

    sync_case_slugs = [slug for _step_name, slug, _fail_label, _msg_label in SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES]
    if len(sync_case_slugs) != len(set(sync_case_slugs)):
        issues.append("SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES contains duplicate case slugs")
    sync_failed_case_slugs = [
        slug
        for _step_name, slug, _fail_label, _msg_label in SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES
    ]
    if len(sync_failed_case_slugs) != len(set(sync_failed_case_slugs)):
        issues.append("SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES contains duplicate case slugs")
    featured_sync_case_slugs = [
        slug for _step_name, slug, _fail_label, _msg_label in SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_CASES
    ]
    if len(featured_sync_case_slugs) != len(set(featured_sync_case_slugs)):
        issues.append("SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_CASES contains duplicate case slugs")
    featured_sync_failed_case_slugs = [
        slug
        for _step_name, slug, _fail_label, _msg_label in SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES
    ]
    if len(featured_sync_failed_case_slugs) != len(set(featured_sync_failed_case_slugs)):
        issues.append("SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES contains duplicate case slugs")

    for step_name, script_path in SEAMGRIM_BLOCKER_STEP_SCRIPT_PATHS:
        normalized = str(script_path).replace("\\", "/")
        if not normalized.startswith("tests/"):
            issues.append(f"{step_name} script path must stay under tests/: {script_path}")
        target = repo_root / script_path
        if not target.exists():
            issues.append(f"{step_name} script path does not exist: {script_path}")
    for step_name, script_path in SEAMGRIM_FEATURED_SEED_STEP_SCRIPT_PATHS:
        normalized = str(script_path).replace("\\", "/")
        if not normalized.startswith("tests/"):
            issues.append(f"{step_name} script path must stay under tests/: {script_path}")
        target = repo_root / script_path
        if not target.exists():
            issues.append(f"{step_name} script path does not exist: {script_path}")

    if set(SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME.keys()) != canonical_set:
        issues.append("SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME keys must match canonical blocker steps")
    if SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME != dict(SEAMGRIM_BLOCKER_STEP_SCRIPT_PATHS):
        issues.append("SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME must mirror SEAMGRIM_BLOCKER_STEP_SCRIPT_PATHS")

    if set(SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_BY_STEP.keys()) != canonical_set:
        issues.append("SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_BY_STEP keys must match canonical blocker steps")
    if SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_BY_STEP != dict(SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_FIELDS):
        issues.append("SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_BY_STEP must mirror SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_FIELDS")
    if set(SEAMGRIM_FEATURED_SEED_STEP_SCRIPT_PATH_BY_NAME.keys()) != featured_set:
        issues.append("SEAMGRIM_FEATURED_SEED_STEP_SCRIPT_PATH_BY_NAME keys must match featured seamgrim steps")
    if SEAMGRIM_FEATURED_SEED_STEP_SCRIPT_PATH_BY_NAME != dict(SEAMGRIM_FEATURED_SEED_STEP_SCRIPT_PATHS):
        issues.append("SEAMGRIM_FEATURED_SEED_STEP_SCRIPT_PATH_BY_NAME must mirror SEAMGRIM_FEATURED_SEED_STEP_SCRIPT_PATHS")
    required_steps = tuple(SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS)
    if len(required_steps) != len(set(required_steps)):
        issues.append("SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS contains duplicate step names")
    missing_required_featured_steps = sorted(featured_set - set(required_steps))
    if missing_required_featured_steps:
        issues.append(
            "SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS missing featured steps: "
            + ", ".join(missing_required_featured_steps)
        )

    return tuple(issues)


def collect_platform_contract_issues() -> tuple[str, ...]:
    issues: list[str] = []

    canonical_steps = tuple(SEAMGRIM_PLATFORM_STEP_CONTRACT_STEPS)
    canonical_set = set(canonical_steps)
    if len(canonical_steps) != len(canonical_set):
        issues.append("SEAMGRIM_PLATFORM_STEP_CONTRACT_STEPS contains duplicate step names")

    script_steps = [step_name for step_name, _script_path in SEAMGRIM_PLATFORM_STEP_SCRIPT_PATHS]
    summary_steps = [step_name for _summary_key, step_name in SEAMGRIM_PLATFORM_SANITY_SUMMARY_STEP_FIELDS]
    fail_code_steps = [step_name for step_name, _code in SEAMGRIM_PLATFORM_SANITY_FAIL_CODE_FIELDS]
    sync_case_steps = [
        step_name for step_name, _slug, _fail_label, _msg_label in SEAMGRIM_PLATFORM_SYNC_READINESS_VALIDATE_ONLY_CASES
    ]
    sync_failed_case_steps = [
        step_name
        for step_name, _slug, _fail_label, _msg_label in SEAMGRIM_PLATFORM_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES
    ]
    report_case_steps = [
        step_name for step_name, _fail_message, _code_message in SEAMGRIM_PLATFORM_GATE_REPORT_INDEX_MISSING_STEP_CASES
    ]
    report_failed_case_steps = [
        step_name for step_name, _fail_message, _code_message in SEAMGRIM_PLATFORM_GATE_REPORT_INDEX_FAILED_STEP_CASES
    ]

    def _check_step_set(label: str, steps: list[str], expected_set: set[str]) -> None:
        step_set = set(steps)
        if len(step_set) != len(steps):
            issues.append(f"{label} contains duplicate step names")
        if step_set != expected_set:
            missing = sorted(expected_set - step_set)
            extra = sorted(step_set - expected_set)
            if missing:
                issues.append(f"{label} missing steps: {', '.join(missing)}")
            if extra:
                issues.append(f"{label} has unknown steps: {', '.join(extra)}")

    _check_step_set("SEAMGRIM_PLATFORM_STEP_SCRIPT_PATHS", script_steps, canonical_set)
    _check_step_set("SEAMGRIM_PLATFORM_SANITY_SUMMARY_STEP_FIELDS", summary_steps, canonical_set)
    _check_step_set("SEAMGRIM_PLATFORM_SANITY_FAIL_CODE_FIELDS", fail_code_steps, canonical_set)
    _check_step_set("SEAMGRIM_PLATFORM_SYNC_READINESS_VALIDATE_ONLY_CASES", sync_case_steps, canonical_set)
    _check_step_set(
        "SEAMGRIM_PLATFORM_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES",
        sync_failed_case_steps,
        canonical_set,
    )
    _check_step_set("SEAMGRIM_PLATFORM_GATE_REPORT_INDEX_MISSING_STEP_CASES", report_case_steps, canonical_set)
    _check_step_set(
        "SEAMGRIM_PLATFORM_GATE_REPORT_INDEX_FAILED_STEP_CASES",
        report_failed_case_steps,
        canonical_set,
    )

    repo_root = Path(__file__).resolve().parent.parent
    for step_name, script_path in SEAMGRIM_PLATFORM_STEP_SCRIPT_PATHS:
        normalized = str(script_path).replace("\\", "/")
        if not normalized.startswith("tests/"):
            issues.append(f"{step_name} script path must stay under tests/: {script_path}")
        target = repo_root / script_path
        if not target.exists():
            issues.append(f"{step_name} script path does not exist: {script_path}")

    if set(SEAMGRIM_PLATFORM_STEP_SCRIPT_PATH_BY_NAME.keys()) != canonical_set:
        issues.append("SEAMGRIM_PLATFORM_STEP_SCRIPT_PATH_BY_NAME keys must match canonical platform steps")
    if SEAMGRIM_PLATFORM_STEP_SCRIPT_PATH_BY_NAME != dict(SEAMGRIM_PLATFORM_STEP_SCRIPT_PATHS):
        issues.append("SEAMGRIM_PLATFORM_STEP_SCRIPT_PATH_BY_NAME must mirror SEAMGRIM_PLATFORM_STEP_SCRIPT_PATHS")
    if set(SEAMGRIM_PLATFORM_SANITY_FAIL_CODE_BY_STEP.keys()) != canonical_set:
        issues.append("SEAMGRIM_PLATFORM_SANITY_FAIL_CODE_BY_STEP keys must match canonical platform steps")
    if SEAMGRIM_PLATFORM_SANITY_FAIL_CODE_BY_STEP != dict(SEAMGRIM_PLATFORM_SANITY_FAIL_CODE_FIELDS):
        issues.append("SEAMGRIM_PLATFORM_SANITY_FAIL_CODE_BY_STEP must mirror SEAMGRIM_PLATFORM_SANITY_FAIL_CODE_FIELDS")

    required_steps = tuple(SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS)
    missing_required_platform_steps = sorted(canonical_set - set(required_steps))
    if missing_required_platform_steps:
        issues.append(
            "SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS missing platform steps: "
            + ", ".join(missing_required_platform_steps)
        )

    return tuple(issues)
