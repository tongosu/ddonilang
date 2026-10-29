from __future__ import annotations

import json
from datetime import datetime, timezone

from _ci_profile_matrix_full_real_smoke_contract import (
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_CORE_LANG,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_FULL,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_SEAMGRIM,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_CORE_LANG,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_FULL,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_SEAMGRIM,
    PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT,
)


PROFILE_MATRIX_SELFTEST_PROFILES = ("core_lang", "full", "seamgrim")
PROFILE_MATRIX_SELFTEST_LIGHTWEIGHT_FULL_REAL_ENV_KEY = "DDN_CI_PROFILE_MATRIX_SELFTEST_FULL_REAL"
PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT = PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT
PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC = {
    "core_lang": float(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_CORE_LANG),
    "full": float(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_FULL),
    "seamgrim": float(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_SEAMGRIM),
}
PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS = {
    "core_lang": PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_CORE_LANG,
    "full": PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_FULL,
    "seamgrim": PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_SEAMGRIM,
}
PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC_JSON = json.dumps(
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC,
    ensure_ascii=False,
    sort_keys=True,
    separators=(",", ":"),
)
PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS_JSON = json.dumps(
    PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS,
    ensure_ascii=False,
    sort_keys=True,
    separators=(",", ":"),
)
PROFILE_MATRIX_SUMMARY_VALUE_KEYS = (
    "ci_sanity_pack_golden_lang_consistency_ok",
    "ci_sanity_pack_golden_metadata_ok",
    "ci_sanity_pack_golden_graph_export_ok",
    "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok",
    "ci_sanity_canon_ast_dpack_ok",
    "ci_sanity_seamgrim_numeric_factor_policy_ok",
    "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok",
)
PROFILE_MATRIX_SELFTEST_SCHEMA = "ddn.ci.profile_matrix_gate_selftest.v1"
PROFILE_MATRIX_SNAPSHOT_FIELD_SPECS = (
    ("status", "text", "status", "unknown"),
    ("ok", "bool_text", "ok", "0"),
    ("total_elapsed_ms", "int_text", "total_elapsed_ms", "-"),
    ("selected_real_profiles", "names_text", "selected_real_profiles", "-"),
    ("skipped_real_profiles", "names_text", "skipped_real_profiles", "-"),
    ("step_timeout_defaults_text", "text", "step_timeout_defaults_text", PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT),
    ("step_timeout_defaults_sec_json", "json_dict_sec", "step_timeout_defaults_sec", PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC_JSON),
    ("step_timeout_env_keys_json", "json_dict_text", "step_timeout_env_keys", PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS_JSON),
    ("core_lang_elapsed_ms", "elapsed_text", "core_lang", "-"),
    ("full_elapsed_ms", "elapsed_text", "full", "-"),
    ("seamgrim_elapsed_ms", "elapsed_text", "seamgrim", "-"),
)
PROFILE_MATRIX_BRIEF_KEYS = (
    "profile_matrix_total_elapsed_ms",
    "profile_matrix_selected_real_profiles",
    "profile_matrix_core_lang_elapsed_ms",
    "profile_matrix_full_elapsed_ms",
    "profile_matrix_seamgrim_elapsed_ms",
)
PROFILE_MATRIX_TRIAGE_REQUIRED_BASE_KEYS = (
    "report_path",
    "status",
    "ok",
    "total_elapsed_ms",
    "selected_real_profiles",
    "skipped_real_profiles",
    "step_timeout_defaults_text",
    "step_timeout_defaults_sec",
    "step_timeout_env_keys",
    "core_lang_elapsed_ms",
    "full_elapsed_ms",
    "seamgrim_elapsed_ms",
    "aggregate_summary_sanity_ok",
    "aggregate_summary_sanity_checked_profiles",
    "aggregate_summary_sanity_failed_profiles",
    "aggregate_summary_sanity_skipped_profiles",
)
PROFILE_MATRIX_TRIAGE_REQUIRED_KEYS = PROFILE_MATRIX_TRIAGE_REQUIRED_BASE_KEYS + tuple(
    f"{profile_name}_aggregate_summary_status"
    for profile_name in PROFILE_MATRIX_SELFTEST_PROFILES
) + tuple(
    f"{profile_name}_aggregate_summary_ok"
    for profile_name in PROFILE_MATRIX_SELFTEST_PROFILES
) + tuple(
    f"{profile_name}_aggregate_summary_values"
    for profile_name in PROFILE_MATRIX_SELFTEST_PROFILES
)
AGE4_PROOF_SUMMARY_PAIRS = {
    "age4_proof_ok": "1",
    "age4_proof_failed_criteria": "0",
    "age4_proof_summary_hash": "sha256:fake-age4-proof-summary",
}


def parse_profile_matrix_selftest_real_profiles(raw: str) -> tuple[list[str], list[str]]:
    seen: set[str] = set()
    ordered: list[str] = []
    invalid: list[str] = []
    for token in str(raw).split(","):
        name = token.strip()
        if not name:
            continue
        if name not in PROFILE_MATRIX_SELFTEST_PROFILES:
            if name not in invalid:
                invalid.append(name)
            continue
        if name in seen:
            continue
        seen.add(name)
        ordered.append(name)
    return ordered, invalid


def expected_profile_matrix_summary_values(profile_name: str) -> dict[str, str]:
    enabled = profile_name in {"core_lang", "full"}
    value = "1" if enabled else "0"
    sync_graph_value = "1" if enabled else "0"
    numeric_ok_value = "na" if profile_name == "core_lang" else "1"
    return {
        "ci_sanity_pack_golden_lang_consistency_ok": value,
        "ci_sanity_pack_golden_metadata_ok": value,
        "ci_sanity_pack_golden_graph_export_ok": value,
        "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok": sync_graph_value,
        "ci_sanity_canon_ast_dpack_ok": value,
        "ci_sanity_seamgrim_numeric_factor_policy_ok": numeric_ok_value,
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok": numeric_ok_value,
    }


def format_profile_matrix_summary_values(values: dict[str, str]) -> str:
    return "/".join(str(values.get(key, "")).strip() or "-" for key in PROFILE_MATRIX_SUMMARY_VALUE_KEYS)


def join_profile_matrix_names(raw: object) -> str:
    if not isinstance(raw, list):
        return "-"
    values = [str(item).strip() for item in raw if str(item).strip()]
    return ",".join(values) if values else "-"


def normalize_timeout_defaults_sec(raw: object) -> dict[str, float]:
    result: dict[str, float] = {}
    source = raw if isinstance(raw, dict) else {}
    for profile_name in PROFILE_MATRIX_SELFTEST_PROFILES:
        fallback = float(PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC[profile_name])
        value = source.get(profile_name, fallback) if isinstance(source, dict) else fallback
        try:
            result[profile_name] = float(value)
        except Exception:
            result[profile_name] = fallback
    return result


def normalize_timeout_env_keys(raw: object) -> dict[str, str]:
    result: dict[str, str] = {}
    source = raw if isinstance(raw, dict) else {}
    for profile_name in PROFILE_MATRIX_SELFTEST_PROFILES:
        fallback = str(PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS[profile_name]).strip()
        value = source.get(profile_name, fallback) if isinstance(source, dict) else fallback
        text = str(value).strip()
        result[profile_name] = text or fallback
    return result


def parse_json_object_text(raw: object) -> dict[str, object] | None:
    text = str(raw).strip()
    if not text:
        return None
    try:
        payload = json.loads(text)
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def build_profile_matrix_snapshot_from_doc(doc: dict, *, report_path: str = "") -> dict[str, str] | None:
    if not isinstance(doc, dict):
        return None
    if str(doc.get("schema", "")).strip() != PROFILE_MATRIX_SELFTEST_SCHEMA:
        return None

    real_profiles = doc.get("real_profiles")

    def read_elapsed(profile_name: str) -> str:
        if not isinstance(real_profiles, dict):
            return "-"
        row = real_profiles.get(profile_name)
        if not isinstance(row, dict):
            return "-"
        elapsed_raw = row.get("total_elapsed_ms")
        if elapsed_raw is None:
            return "-"
        try:
            return str(max(0, int(elapsed_raw)))
        except Exception:
            return "-"

    def aggregate_summary_row(profile_name: str) -> dict | None:
        block = doc.get("aggregate_summary_sanity_by_profile")
        if not isinstance(block, dict):
            return None
        row = block.get(profile_name)
        return row if isinstance(row, dict) else None

    snapshot: dict[str, str] = {"path": str(report_path).strip()}
    for output_key, field_kind, source_key, default_value in PROFILE_MATRIX_SNAPSHOT_FIELD_SPECS:
        if field_kind == "text":
            snapshot[output_key] = str(doc.get(source_key, "")).strip() or str(default_value)
            continue
        if field_kind == "json_dict_sec":
            source = doc.get(source_key)
            normalized = normalize_timeout_defaults_sec(source if isinstance(source, dict) else {})
            snapshot[output_key] = json.dumps(
                normalized,
                ensure_ascii=False,
                sort_keys=True,
                separators=(",", ":"),
            )
            continue
        if field_kind == "json_dict_text":
            source = doc.get(source_key)
            normalized = normalize_timeout_env_keys(source if isinstance(source, dict) else {})
            snapshot[output_key] = json.dumps(
                normalized,
                ensure_ascii=False,
                sort_keys=True,
                separators=(",", ":"),
            )
            continue
        if field_kind == "bool_text":
            snapshot[output_key] = "1" if bool(doc.get(source_key, False)) else "0"
            continue
        if field_kind == "names_text":
            snapshot[output_key] = join_profile_matrix_names(doc.get(source_key))
            continue
        if field_kind == "elapsed_text":
            snapshot[output_key] = read_elapsed(source_key)
            continue
        if field_kind == "int_text":
            raw = doc.get(source_key)
            try:
                snapshot[output_key] = str(max(0, int(raw)))
            except Exception:
                snapshot[output_key] = str(default_value)
            continue
        snapshot[output_key] = str(default_value)
    snapshot["aggregate_summary_sanity_ok"] = "1" if bool(doc.get("aggregate_summary_sanity_ok", False)) else "0"
    for list_key in (
        "aggregate_summary_sanity_checked_profiles",
        "aggregate_summary_sanity_failed_profiles",
        "aggregate_summary_sanity_skipped_profiles",
    ):
        snapshot[list_key] = join_profile_matrix_names(doc.get(list_key))
    for profile_name in PROFILE_MATRIX_SELFTEST_PROFILES:
        row = aggregate_summary_row(profile_name)
        snapshot[f"{profile_name}_aggregate_summary_status"] = (
            str(row.get("status", "")).strip() if isinstance(row, dict) else "-"
        ) or "-"
        snapshot[f"{profile_name}_aggregate_summary_ok"] = (
            "1" if isinstance(row, dict) and bool(row.get("ok", False)) else "0"
        )
        values = row.get("values") if isinstance(row, dict) else None
        snapshot[f"{profile_name}_aggregate_summary_values"] = (
            format_profile_matrix_summary_values(values) if isinstance(values, dict) else "-"
        )
    return snapshot


def build_profile_matrix_triage_payload_from_snapshot(snapshot: dict[str, str]) -> dict[str, object]:
    timeout_defaults_raw = parse_json_object_text(snapshot.get("step_timeout_defaults_sec_json", ""))
    timeout_env_keys_raw = parse_json_object_text(snapshot.get("step_timeout_env_keys_json", ""))
    timeout_defaults_sec = normalize_timeout_defaults_sec(timeout_defaults_raw if isinstance(timeout_defaults_raw, dict) else {})
    timeout_env_keys = normalize_timeout_env_keys(timeout_env_keys_raw if isinstance(timeout_env_keys_raw, dict) else {})
    payload: dict[str, object] = {
        "report_path": snapshot["path"],
        "status": snapshot["status"],
        "ok": bool(snapshot["ok"] == "1"),
        "total_elapsed_ms": None if snapshot["total_elapsed_ms"] == "-" else int(snapshot["total_elapsed_ms"]),
        "selected_real_profiles": []
        if snapshot["selected_real_profiles"] == "-"
        else snapshot["selected_real_profiles"].split(","),
        "skipped_real_profiles": []
        if snapshot["skipped_real_profiles"] == "-"
        else snapshot["skipped_real_profiles"].split(","),
        "step_timeout_defaults_text": str(snapshot.get("step_timeout_defaults_text", "")).strip()
        or PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
        "step_timeout_defaults_sec": timeout_defaults_sec,
        "step_timeout_env_keys": timeout_env_keys,
        "core_lang_elapsed_ms": None
        if snapshot["core_lang_elapsed_ms"] == "-"
        else int(snapshot["core_lang_elapsed_ms"]),
        "full_elapsed_ms": None if snapshot["full_elapsed_ms"] == "-" else int(snapshot["full_elapsed_ms"]),
        "seamgrim_elapsed_ms": None
        if snapshot["seamgrim_elapsed_ms"] == "-"
        else int(snapshot["seamgrim_elapsed_ms"]),
        "aggregate_summary_sanity_ok": bool(snapshot["aggregate_summary_sanity_ok"] == "1"),
        "aggregate_summary_sanity_checked_profiles": []
        if snapshot["aggregate_summary_sanity_checked_profiles"] == "-"
        else snapshot["aggregate_summary_sanity_checked_profiles"].split(","),
        "aggregate_summary_sanity_failed_profiles": []
        if snapshot["aggregate_summary_sanity_failed_profiles"] == "-"
        else snapshot["aggregate_summary_sanity_failed_profiles"].split(","),
        "aggregate_summary_sanity_skipped_profiles": []
        if snapshot["aggregate_summary_sanity_skipped_profiles"] == "-"
        else snapshot["aggregate_summary_sanity_skipped_profiles"].split(","),
    }
    for profile_name in PROFILE_MATRIX_SELFTEST_PROFILES:
        payload[f"{profile_name}_aggregate_summary_status"] = snapshot[f"{profile_name}_aggregate_summary_status"]
        payload[f"{profile_name}_aggregate_summary_ok"] = bool(
            snapshot[f"{profile_name}_aggregate_summary_ok"] == "1"
        )
        payload[f"{profile_name}_aggregate_summary_values"] = snapshot[
            f"{profile_name}_aggregate_summary_values"
        ]
    return payload


def profile_matrix_triage_missing_keys(payload: dict[str, object]) -> list[str]:
    return [key for key in PROFILE_MATRIX_TRIAGE_REQUIRED_KEYS if key not in payload]


def build_profile_matrix_brief_payload_from_snapshot(snapshot: dict[str, str]) -> dict[str, str]:
    return {
        "profile_matrix_total_elapsed_ms": snapshot["total_elapsed_ms"],
        "profile_matrix_selected_real_profiles": snapshot["selected_real_profiles"],
        "profile_matrix_core_lang_elapsed_ms": snapshot["core_lang_elapsed_ms"],
        "profile_matrix_full_elapsed_ms": snapshot["full_elapsed_ms"],
        "profile_matrix_seamgrim_elapsed_ms": snapshot["seamgrim_elapsed_ms"],
    }


def expected_profile_matrix_aggregate_summary_contract(profile_name: str) -> dict[str, object]:
    if profile_name not in PROFILE_MATRIX_SELFTEST_PROFILES:
        raise ValueError(f"unsupported profile: {profile_name}")
    values = expected_profile_matrix_summary_values(profile_name)
    post_markers = build_lightweight_profile_gate_contract(profile_name, quick=False).get("post_markers", [])
    gate_marker_expected = any(
        str(marker).strip() == f"[ci-profile-{profile_name}] aggregate summary sanity markers ok"
        for marker in post_markers
    )
    return {
        "expected_present": True,
        "status": "pass",
        "ok": True,
        "expected_profile": profile_name,
        "expected_sync_profile": profile_name,
        "values": values,
        "values_text": format_profile_matrix_summary_values(values),
        "gate_marker_expected": gate_marker_expected,
    }


def build_lightweight_profile_gate_contract(profile_name: str, *, quick: bool) -> dict[str, object]:
    if profile_name not in PROFILE_MATRIX_SELFTEST_PROFILES:
        raise ValueError(f"unsupported profile: {profile_name}")

    if profile_name == "core_lang":
        contract = {
            "profile_status_marker": "ci_profile_core_lang_status=pass",
            "base_markers": [
                "ci_profile_core_lang_status=pass",
                "contract tier unsupported check ok",
                "contract tier age3 min enforcement check ok",
                "map access contract check ok",
                "gaji registry strict/audit check ok",
                "[stdlib-catalog-check] ok",
                "[tensor-v0-cli-check] ok",
                "[fixed64-darwin-schedule-policy] ok",
                "[fixed64-darwin-real-report] skip enabled=0 report=fake summary=fake",
                "seamgrim featured seed catalog autogen check ok",
                "[ci-profile-core-lang] runtime5 summary is emitted but not required for core_lang contract",
            ]
            ,
            "quick_skip_marker": "[ci-profile-core-lang] aggregate gate skipped by --quick",
            "summary_pairs": {
                "ci_sanity_gate_profile": "core_lang",
                "ci_sync_readiness_sanity_profile": "core_lang",
                "ci_sanity_pack_golden_lang_consistency_ok": "1",
                "ci_sanity_pack_golden_metadata_ok": "1",
                "ci_sanity_pack_golden_graph_export_ok": "1",
                "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok": "1",
                "ci_sanity_canon_ast_dpack_ok": "1",
                "ci_sanity_seamgrim_numeric_factor_policy_ok": "na",
                "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok": "na",
                **AGE4_PROOF_SUMMARY_PAIRS,
            },
            "post_markers": [
                "[ci-profile-core-lang] aggregate age4 proof ok=1 failed=0 hash=sha256:fake-age4-proof-summary",
                "[ci-gate-report-index-check] ok index=fake://core_lang",
            ],
        }
        return contract

    if profile_name == "full":
        contract = {
            "profile_status_marker": "ci_profile_full_status=pass",
            "base_markers": [
                "ci_profile_full_status=pass",
                "contract tier unsupported check ok",
                "contract tier age3 min enforcement check ok",
                "map access contract check ok",
                "gaji registry strict/audit check ok",
                "[stdlib-catalog-check] ok",
                "[tensor-v0-cli-check] ok",
                "[fixed64-darwin-schedule-policy] ok",
                "[fixed64-darwin-real-report] skip enabled=0 report=fake summary=fake",
                "seamgrim featured seed catalog autogen check ok",
                "seamgrim ci gate featured seed catalog step check ok",
                "seamgrim ci gate featured seed catalog autogen step check ok",
                "seamgrim ci gate lesson warning step check ok",
                "seamgrim ci gate stateful preview step check ok",
            ],
            "quick_skip_marker": "[ci-profile-full] aggregate gate skipped by --quick",
            "summary_pairs": {
                "ci_sanity_gate_profile": "full",
                "ci_sync_readiness_sanity_profile": "full",
                "ci_sanity_pack_golden_lang_consistency_ok": "1",
                "ci_sanity_pack_golden_metadata_ok": "1",
                "ci_sanity_pack_golden_graph_export_ok": "1",
                "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok": "1",
                "ci_sanity_canon_ast_dpack_ok": "1",
                "ci_sanity_seamgrim_numeric_factor_policy_ok": "1",
                "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok": "1",
                **AGE4_PROOF_SUMMARY_PAIRS,
            },
            "post_markers": [
                "[ci-profile-full] aggregate age4 proof ok=1 failed=0 hash=sha256:fake-age4-proof-summary",
                "[ci-gate-report-index-check] ok index=fake://full",
                "[ci-gate-summary-report-check] ok status=pass summary=fake index=fake",
                "[ci-profile-full] aggregate summary sanity markers ok",
                "[ci-emit-artifacts-check] ok index=fake://full status=pass require_brief=1 require_triage=1",
            ],
        }
        return contract

    return {
        "profile_status_marker": "ci_profile_seamgrim_status=pass",
        "base_markers": [
            "ci_profile_seamgrim_status=pass",
            "[fixed64-darwin-schedule-policy] ok",
            "[fixed64-darwin-real-report] skip enabled=0 report=fake summary=fake",
            "seamgrim ci gate seed meta step check ok",
            "seamgrim ci gate runtime5 passthrough check ok",
            "seamgrim featured seed catalog autogen check ok",
            "seamgrim ci gate featured seed catalog step check ok",
            "seamgrim ci gate featured seed catalog autogen step check ok",
            "seamgrim ci gate lesson warning step check ok",
            "seamgrim ci gate stateful preview step check ok",
            "seamgrim interface boundary contract check ok",
            "overlay compare diag parity check ok",
            "[seamgrim-wasm-cli-diag-parity] ok",
        ],
        "quick_skip_marker": "[ci-profile-seamgrim] aggregate gate skipped by --quick",
        "summary_pairs": {
            "ci_sanity_gate_profile": "seamgrim",
            "ci_sync_readiness_sanity_profile": "seamgrim",
            "ci_sanity_pack_golden_lang_consistency_ok": "0",
            "ci_sanity_pack_golden_metadata_ok": "0",
            "ci_sanity_pack_golden_graph_export_ok": "0",
            "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok": "0",
            "ci_sanity_canon_ast_dpack_ok": "0",
            "ci_sanity_seamgrim_numeric_factor_policy_ok": "1",
            "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok": "1",
            **AGE4_PROOF_SUMMARY_PAIRS,
        },
        "post_markers": [
            "[ci-profile-seamgrim] aggregate age4 proof ok=1 failed=0 hash=sha256:fake-age4-proof-summary",
            "[ci-gate-report-index-check] ok index=fake://seamgrim",
            "[ci-gate-summary-report-check] ok status=pass summary=fake index=fake",
            "[ci-profile-seamgrim] aggregate summary sanity markers ok",
            "[ci-emit-artifacts-check] ok index=fake://seamgrim status=pass require_brief=1 require_triage=1",
        ],
    }


def build_lightweight_profile_gate_lines(profile_name: str, *, quick: bool) -> list[str]:
    contract = build_lightweight_profile_gate_contract(profile_name, quick=quick)
    lines = list(contract.get("base_markers", []))
    if quick:
        quick_skip_marker = str(contract.get("quick_skip_marker", "")).strip()
        if quick_skip_marker:
            lines.append(quick_skip_marker)
        return lines
    summary_pairs = contract.get("summary_pairs", {})
    if isinstance(summary_pairs, dict):
        for key, value in summary_pairs.items():
            lines.append(f"[ci-gate-summary] {key}={value}")
    lines.extend(str(item) for item in contract.get("post_markers", []))
    return lines


def build_profile_matrix_selftest_fixture(
    selected_real_profiles: list[str],
    *,
    quick: bool,
    dry: bool,
) -> dict[str, object]:
    skipped_real_profiles = [name for name in PROFILE_MATRIX_SELFTEST_PROFILES if name not in selected_real_profiles]
    return {
        "schema": "ddn.ci.profile_matrix_gate_selftest.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": "pass",
        "ok": True,
        "selected_real_profiles": selected_real_profiles,
        "skipped_real_profiles": skipped_real_profiles,
        "step_timeout_defaults_text": PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
        "step_timeout_defaults_sec": dict(PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC),
        "step_timeout_env_keys": dict(PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS),
        "quick": bool(quick),
        "dry": bool(dry),
        "total_elapsed_ms": 0,
        "aggregate_summary_sanity_ok": True,
        "aggregate_summary_sanity_checked_profiles": [],
        "aggregate_summary_sanity_failed_profiles": [],
        "aggregate_summary_sanity_skipped_profiles": list(selected_real_profiles),
        "aggregate_summary_sanity_by_profile": {
            name: {
                "expected_present": expected_profile_matrix_aggregate_summary_contract(name)["expected_present"],
                "present": False,
                "status": "skipped",
                "reason": "not_selected",
                "expected_profile": expected_profile_matrix_aggregate_summary_contract(name)["expected_profile"],
                "expected_sync_profile": expected_profile_matrix_aggregate_summary_contract(name)[
                    "expected_sync_profile"
                ],
                "profile": "",
                "sync_profile": "",
                "expected_values": dict(expected_profile_matrix_aggregate_summary_contract(name)["values"]),
                "values": {
                    "ci_sanity_pack_golden_lang_consistency_ok": "",
                    "ci_sanity_pack_golden_metadata_ok": "",
                    "ci_sanity_pack_golden_graph_export_ok": "",
                    "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok": "",
                    "ci_sanity_canon_ast_dpack_ok": "",
                    "ci_sanity_seamgrim_numeric_factor_policy_ok": "",
                    "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok": "",
                },
                "missing_keys": [],
                "mismatched_keys": [],
                "profile_ok": True,
                "sync_profile_ok": True,
                "values_ok": True,
                "gate_marker_expected": bool(
                    expected_profile_matrix_aggregate_summary_contract(name)["gate_marker_expected"]
                ),
                "gate_marker_present": False,
                "gate_marker_ok": True,
                "ok": True,
            }
            for name in PROFILE_MATRIX_SELFTEST_PROFILES
        },
        "real_profiles": {
            name: {
                "selected": name in selected_real_profiles,
                "skipped": name not in selected_real_profiles,
                "status": "skipped" if name not in selected_real_profiles else "pending",
                "ok": bool(name not in selected_real_profiles),
                "total_elapsed_ms": None,
                "step_elapsed_ms": None,
            }
            for name in PROFILE_MATRIX_SELFTEST_PROFILES
        },
    }


def validate_profile_matrix_aggregate_summary(
    summary: object,
    *,
    profile: str,
    expected_values: dict[str, str],
    expected_present: bool,
    expected_gate_marker: bool,
) -> str | None:
    if not isinstance(summary, dict):
        return "aggregate_summary_sanity_missing"
    if bool(summary.get("expected_present", False)) is not bool(expected_present):
        return "aggregate_summary_expected_present_mismatch"
    if not expected_present:
        if str(summary.get("status", "")) != "skipped":
            return "aggregate_summary_status_mismatch"
        if not bool(summary.get("ok", False)):
            return "aggregate_summary_ok_mismatch"
        return None
    if not bool(summary.get("present", False)):
        return "aggregate_summary_present_mismatch"
    if str(summary.get("status", "")) != "pass":
        return "aggregate_summary_status_mismatch"
    if not bool(summary.get("ok", False)):
        return "aggregate_summary_ok_mismatch"
    if str(summary.get("profile", "")) != profile:
        return "aggregate_summary_profile_mismatch"
    if str(summary.get("sync_profile", "")) != profile:
        return "aggregate_summary_sync_profile_mismatch"
    if not bool(summary.get("profile_ok", False)):
        return "aggregate_summary_profile_ok_mismatch"
    if not bool(summary.get("sync_profile_ok", False)):
        return "aggregate_summary_sync_profile_ok_mismatch"
    if not bool(summary.get("values_ok", False)):
        return "aggregate_summary_values_ok_mismatch"
    if list(summary.get("missing_keys", [])) != []:
        return "aggregate_summary_missing_keys_mismatch"
    if list(summary.get("mismatched_keys", [])) != []:
        return "aggregate_summary_mismatched_keys_mismatch"
    if bool(summary.get("gate_marker_expected", False)) is not bool(expected_gate_marker):
        return "aggregate_summary_gate_marker_expected_mismatch"
    if not bool(summary.get("gate_marker_ok", False)):
        return "aggregate_summary_gate_marker_ok_mismatch"
    if expected_gate_marker and not bool(summary.get("gate_marker_present", False)):
        return "aggregate_summary_gate_marker_present_mismatch"
    values = summary.get("values", {})
    if not isinstance(values, dict):
        return "aggregate_summary_values_missing"
    for key, expected in expected_values.items():
        if str(values.get(key, "")) != str(expected):
            return f"aggregate_summary_{key}_mismatch"
    return None
