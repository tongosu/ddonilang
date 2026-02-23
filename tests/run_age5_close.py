#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
from datetime import datetime, timezone
from pathlib import Path

AGE4_S2_TASK_PATH = Path("docs/context/codex_tasks/TASK_SEAMGRIM_AGE4_S2_PRIMITIVE_RUNTIME_UI_SLOTS_V1.md")
S5_BASELINE_TASK_PATH = Path("docs/context/codex_tasks/S5_OVERLAY_BASELINE_VARIANT.md")
S5_DETAILED_TASK_PATH = Path("docs/context/codex_tasks/S5_OVERLAY_2LAYER_DETAILED.md")
AGE5_SLOT_UI_PATH = Path("solutions/seamgrim_ui_mvp/ui/index.html")
AGE5_APP_UI_PATH = Path("solutions/seamgrim_ui_mvp/ui/app.js")
OVERLAY_SESSION_CONTRACT_PATH = Path("solutions/seamgrim_ui_mvp/ui/overlay_session_contract.js")
SLOT_LABELS = [
    "A 실시간 입력",
    "B 꾸러미 브라우저",
    "C 3D 렌더",
]
PACK_HINT = "pack/seamgrim_overlay_param_compare_v0"
PACK_GOLDEN_PATH = Path("pack/seamgrim_overlay_param_compare_v0/golden.jsonl")
PACK_MIN_CASE_COUNT = 76
S6_SESSION_PACK_HINT = "pack/seamgrim_overlay_session_roundtrip_v0"
S6_SESSION_PACK_GOLDEN_PATH = Path("pack/seamgrim_overlay_session_roundtrip_v0/golden.jsonl")
S6_SESSION_PACK_MIN_CASE_COUNT = 8
S5_BASELINE_DOD_TOKENS = [
    "- [x] 진자 L 비교 가능.",
    "- [x] 축 불일치 차단.",
]
S5_DETAILED_DOD_TOKENS = [
    "- [x] 진자 L=1.0 vs L=2.0 오버레이 가능",
    "- [x] 축 메타 불일치 시 차단",
    "- [x] session 저장/로드 시 variant params/visible/order 보존",
]
S6_SESSION_CONTRACT_APP_TOKENS = [
    "./overlay_session_contract.js",
    "buildOverlaySessionRunsPayload(",
    "buildOverlayCompareSessionPayload(",
    "resolveOverlayCompareFromSession(",
]
S6_SESSION_CONTRACT_MODULE_TOKENS = [
    "export function buildOverlaySessionRunsPayload",
    "export function buildOverlayCompareSessionPayload",
    "export function resolveOverlayCompareFromSession",
]
S6_VIEW_COMBO_CONTRACT_APP_TOKENS = [
    "buildSessionViewComboPayload(",
    "resolveSessionViewComboFromPayload(",
    "view_combo: buildSessionViewComboPayload({",
]
S6_VIEW_COMBO_CONTRACT_MODULE_TOKENS = [
    "export function resolveSessionViewComboFromPayload",
    "export function buildSessionViewComboPayload",
]
S6_SESSION_PACK_CASE_FILES = [
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c01_role_priority_restore_ok/case.detjson"),
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c02_compare_id_fallback_when_role_missing/case.detjson"),
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c03_drop_variant_on_axis_mismatch/case.detjson"),
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c04_disable_on_baseline_missing/case.detjson"),
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c05_ui_layout_restore_run_tools/case.detjson"),
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c06_ui_layout_invalid_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c07_ui_layout_view_combo_cross_restore/case.detjson"),
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c08_ui_layout_view_combo_cross_fallback/case.detjson"),
]
PACK_CASE_FILES = [
    Path("pack/seamgrim_overlay_param_compare_v0/c01_pendulum_L_compare/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c02_axis_mismatch_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c03_series_missing_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c04_series_mismatch_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c05_graph_kind_mismatch_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c06_y_unit_mismatch_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c07_graph_missing_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c08_y_kind_mismatch_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c09_series_missing_baseline_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c10_x_kind_mismatch_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c11_graph_kind_normalized_equal_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c12_series_id_normalized_equal_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c13_x_unit_normalized_equal_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c14_y_kind_normalized_equal_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c15_x_kind_normalized_equal_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c16_y_unit_normalized_equal_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c17_graph_kind_meta_kind_fallback_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c18_graph_kind_schema_fallback_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c19_x_kind_axis_kind_fallback_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c20_x_unit_axis_unit_fallback_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c21_x_kind_empty_fallback_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c22_x_unit_empty_fallback_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c23_graph_missing_priority_over_series_missing/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c24_series_missing_with_meta_kind_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c25_graph_missing_over_axis_mismatch_xkind/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c26_graph_missing_over_axis_mismatch_xunit/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c27_graph_missing_over_series_mismatch_candidate_baseline/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c28_graph_missing_over_series_mismatch_candidate_variant/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c29_graph_missing_over_axis_mismatch_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c30_graph_missing_over_axis_mismatch_reverse_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c31_axis_mismatch_order_graphkind_before_xkind/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c32_axis_mismatch_order_xkind_before_xunit/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c33_axis_mismatch_order_xunit_before_ykind/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c34_axis_mismatch_order_ykind_before_yunit/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c35_series_missing_over_mismatch_candidate_baseline/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c36_series_missing_over_mismatch_candidate_variant/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c37_graph_missing_over_graphkind_mismatch_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c38_graph_missing_over_graphkind_mismatch_reverse_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c39_graphkind_mismatch_over_series_missing_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c40_graphkind_mismatch_over_series_missing_reverse_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c41_xkind_mismatch_over_series_missing_with_axis_kind_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c42_xunit_mismatch_over_series_missing_reverse_with_axis_unit_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c43_ykind_mismatch_over_series_missing_with_y_kind_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c44_yunit_mismatch_over_series_missing_reverse_with_y_unit_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c45_graph_missing_over_ykind_mismatch_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c46_graph_missing_over_yunit_mismatch_reverse_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c47_graph_missing_over_series_mismatch_with_ykind_candidate/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c48_graph_missing_over_series_mismatch_reverse_with_yunit_candidate/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c49_series_missing_over_ykind_mismatch_candidate_baseline/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c50_series_missing_over_yunit_mismatch_candidate_variant/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c51_graphkind_mismatch_over_series_missing_with_ykind_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c52_graphkind_mismatch_over_series_missing_reverse_with_yunit_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c53_xkind_mismatch_over_series_missing_with_ykind_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c54_xunit_mismatch_over_series_missing_reverse_with_yunit_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c55_graph_missing_over_ykind_mismatch_with_series_candidate/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c56_graph_missing_over_yunit_mismatch_reverse_with_series_candidate/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c57_ykind_mismatch_over_series_mismatch_with_y_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c58_yunit_mismatch_over_series_mismatch_reverse_with_y_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c59_graphkind_mismatch_over_series_mismatch_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c60_xkind_mismatch_over_series_mismatch_reverse_with_axis_kind_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c61_graphkind_mismatch_over_series_mismatch_reverse_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c62_xunit_mismatch_over_series_mismatch_with_axis_unit_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c63_xunit_mismatch_over_series_mismatch_reverse_with_axis_unit_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c64_ykind_mismatch_over_series_mismatch_reverse_with_y_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c65_yunit_mismatch_over_series_mismatch_with_y_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c66_xkind_mismatch_over_series_mismatch_with_axis_kind_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c67_xunit_mismatch_over_yunit_and_series_mismatch_with_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c68_xunit_mismatch_over_yunit_and_series_mismatch_reverse_with_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c69_graphkind_mismatch_over_all_axis_and_series_mismatch/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c70_xkind_mismatch_over_remaining_axis_and_series_mismatch/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c71_xunit_mismatch_over_ykind_yunit_and_series_mismatch/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c72_ykind_mismatch_over_yunit_and_series_mismatch/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c73_yunit_mismatch_over_series_mismatch_with_full_fallback_chain/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c74_yunit_mismatch_over_series_mismatch_reverse_with_full_fallback_chain/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c75_graph_missing_over_series_missing_with_full_fallback_chain/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c76_graph_missing_over_series_missing_reverse_with_full_fallback_chain/case.detjson"),
]


def clip(text: str, limit: int = 140) -> str:
    normalized = " ".join(text.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def sample_items(items: list[str], limit: int = 2) -> str:
    if not items:
        return "-"
    return ",".join(items[:limit])


def full_items(items: list[str]) -> str:
    if not items:
        return "-"
    return ",".join(items)


def sample_window(items: list[str], index: int, radius: int = 1) -> str:
    if not items:
        return "-"
    if index < 0:
        return sample_items(items, limit=3)
    start = max(0, index - radius)
    end = min(len(items), index + radius + 1)
    return ",".join(items[start:end])


def build_order_repair_hint(pack_golden_path: Path, expected_refs: list[str]) -> str:
    return (
        f"repair_hint: reorder {pack_golden_path} to PACK_CASE_FILES order. "
        f"first={sample_items(expected_refs, 3)} last={sample_items(expected_refs[-3:], 3)}"
    )


def build_order_repair_cmd(pack_golden_path: Path, expected_refs: list[str]) -> str:
    # command example for deterministic golden.jsonl reordering to PACK_CASE_FILES order
    script = (
        "import json;from pathlib import Path;"
        f"refs={repr(expected_refs)};"
        f"p=Path({repr(str(pack_golden_path).replace('\\\\', '/'))});"
        "p.write_text('\\n'.join(json.dumps({'overlay_compare_case': r}, ensure_ascii=False) for r in refs)+'\\n', encoding='utf-8')"
    )
    return "python -c " + json.dumps(script, ensure_ascii=False)


def build_order_repair_cmd_short(pack_golden_path: Path) -> str:
    script = (
        "import json;from pathlib import Path;"
        f"p=Path({repr(str(pack_golden_path).replace('\\\\', '/'))});"
        "rows=[json.loads(x) for x in p.read_text(encoding='utf-8').splitlines() if x.strip()];"
        "rows.sort(key=lambda r:r.get('overlay_compare_case',''));"
        "p.write_text('\\n'.join(json.dumps(r, ensure_ascii=False) for r in rows)+'\\n', encoding='utf-8')"
    )
    return "python -c " + json.dumps(script, ensure_ascii=False)


def head_tail(items: list[str], size: int = 3) -> str:
    if not items:
        return "head=- tail=-"
    head = ",".join(items[:size])
    tail = ",".join(items[-size:])
    return f"head={head} tail={tail}"


def default_report_path(file_name: str) -> str:
    preferred = Path("I:/home/urihanl/ddn/codex/build/reports")
    if os.name == "nt":
        try:
            preferred.mkdir(parents=True, exist_ok=True)
        except OSError:
            pass
        return str(preferred / file_name)
    return f"build/reports/{file_name}"


def load_text(path: Path) -> str:
    if not path.exists():
        return ""
    try:
        return path.read_text(encoding="utf-8")
    except Exception:
        return ""


def count_nonempty_lines(text: str) -> int:
    return sum(1 for line in text.splitlines() if line.strip())


def load_overlay_compare_case_refs(path: Path) -> tuple[list[str], list[str]]:
    text = load_text(path)
    refs: list[str] = []
    errors: list[str] = []
    for idx, line in enumerate(text.splitlines(), 1):
        raw = line.strip()
        if not raw:
            continue
        try:
            row = json.loads(raw)
        except Exception:
            errors.append(f"line={idx}: invalid json")
            continue
        rel = row.get("overlay_compare_case")
        if not isinstance(rel, str) or not rel.strip():
            errors.append(f"line={idx}: missing overlay_compare_case")
            continue
        refs.append(rel.strip().replace("\\", "/"))
    return refs, errors


def build_criteria(root: Path) -> tuple[list[dict[str, object]], list[str], list[str], dict[str, object]]:
    criteria: list[dict[str, object]] = []
    failure_digest: list[str] = []
    pending_items: list[str] = []
    repair: dict[str, object] = {}

    age4_s2_text = load_text(root / AGE4_S2_TASK_PATH)
    s5_baseline_text = load_text(root / S5_BASELINE_TASK_PATH)
    s5_detailed_text = load_text(root / S5_DETAILED_TASK_PATH)
    slot_ui_text = load_text(root / AGE5_SLOT_UI_PATH)
    app_ui_text = load_text(root / AGE5_APP_UI_PATH)
    overlay_session_contract_text = load_text(root / OVERLAY_SESSION_CONTRACT_PATH)

    slot_labels_present = [label for label in SLOT_LABELS if label in slot_ui_text]
    slots_declared_ok = len(slot_labels_present) == len(SLOT_LABELS)
    criteria.append(
        {
            "name": "age5_slot_placeholders_declared",
            "ok": slots_declared_ok,
            "detail": f"present={len(slot_labels_present)}/{len(SLOT_LABELS)} file={AGE5_SLOT_UI_PATH}",
        }
    )
    if not slots_declared_ok:
        missing = [label for label in SLOT_LABELS if label not in set(slot_labels_present)]
        failure_digest.append(f"age5_slot_placeholders_declared: missing={clip(', '.join(missing), 200)}")
        pending_items.extend([f"AGE5 slot placeholder missing: {label}" for label in missing])

    slot_disabled_ok = bool(slot_labels_present) and slot_ui_text.count("class=\"age-slot\"") >= len(SLOT_LABELS) and "disabled" in slot_ui_text
    criteria.append(
        {
            "name": "age5_slot_placeholders_disabled",
            "ok": slot_disabled_ok,
            "detail": f"file={AGE5_SLOT_UI_PATH}",
        }
    )
    if not slot_disabled_ok:
        failure_digest.append("age5_slot_placeholders_disabled: disabled marker missing around AGE5 slots")
        pending_items.append("AGE5 슬롯 A/B/C를 비활성 placeholder 상태로 유지")

    age4_s2_slot_plan_ok = "AGE5 슬롯 A/B/C" in age4_s2_text
    criteria.append(
        {
            "name": "age4_s2_task_mentions_age5_slots",
            "ok": age4_s2_slot_plan_ok,
            "detail": f"path={AGE4_S2_TASK_PATH}",
        }
    )
    if not age4_s2_slot_plan_ok:
        failure_digest.append("age4_s2_task_mentions_age5_slots: missing token 'AGE5 슬롯 A/B/C'")
        pending_items.append("AGE4 S2 task 문서에 AGE5 슬롯 A/B/C 연결 문구 유지")

    s5_baseline_ok = bool(s5_baseline_text) and "baseline+variant" in s5_baseline_text and "축" in s5_baseline_text
    criteria.append(
        {
            "name": "s5_baseline_scope_doc_ready",
            "ok": s5_baseline_ok,
            "detail": f"path={S5_BASELINE_TASK_PATH}",
        }
    )
    if not s5_baseline_ok:
        failure_digest.append("s5_baseline_scope_doc_ready: missing baseline+variant or axis constraints")
        pending_items.append("S5 baseline variant 문서의 범위/축 제한 문구 유지")

    s5_detailed_ok = bool(s5_detailed_text) and "baseline+variant" in s5_detailed_text and "graph_kind" in s5_detailed_text
    criteria.append(
        {
            "name": "s5_detailed_scope_doc_ready",
            "ok": s5_detailed_ok,
            "detail": f"path={S5_DETAILED_TASK_PATH}",
        }
    )
    if not s5_detailed_ok:
        failure_digest.append("s5_detailed_scope_doc_ready: missing baseline+variant or graph_kind constraints")
        pending_items.append("S5 detailed 문서의 graph_kind/axis 동등성 조건 유지")

    s5_pack_hint_ok = PACK_HINT in s5_baseline_text and PACK_HINT in s5_detailed_text
    criteria.append(
        {
            "name": "s5_pack_hint_declared",
            "ok": s5_pack_hint_ok,
            "detail": f"hint={PACK_HINT}",
        }
    )
    if not s5_pack_hint_ok:
        failure_digest.append(f"s5_pack_hint_declared: missing hint '{PACK_HINT}' in S5 task docs")
        pending_items.append("S5 문서에 overlay compare pack 경로 힌트 유지")

    missing_baseline_dod = [token for token in S5_BASELINE_DOD_TOKENS if token not in s5_baseline_text]
    s5_baseline_dod_ok = len(missing_baseline_dod) == 0
    criteria.append(
        {
            "name": "s5_baseline_dod_checked",
            "ok": s5_baseline_dod_ok,
            "detail": f"missing={len(missing_baseline_dod)} path={S5_BASELINE_TASK_PATH} sample={sample_items(missing_baseline_dod)}",
        }
    )
    if not s5_baseline_dod_ok:
        failure_digest.append(f"s5_baseline_dod_checked: missing={clip(', '.join(missing_baseline_dod), 200)}")
        pending_items.append("S5 baseline 문서 DoD 체크박스를 완료 상태로 유지")

    missing_detailed_dod = [token for token in S5_DETAILED_DOD_TOKENS if token not in s5_detailed_text]
    s5_detailed_dod_ok = len(missing_detailed_dod) == 0
    criteria.append(
        {
            "name": "s5_detailed_dod_checked",
            "ok": s5_detailed_dod_ok,
            "detail": f"missing={len(missing_detailed_dod)} path={S5_DETAILED_TASK_PATH} sample={sample_items(missing_detailed_dod)}",
        }
    )
    if not s5_detailed_dod_ok:
        failure_digest.append(f"s5_detailed_dod_checked: missing={clip(', '.join(missing_detailed_dod), 200)}")
        pending_items.append("S5 detailed 문서 DoD 체크박스를 완료 상태로 유지")

    missing_s6_app_tokens = [token for token in S6_SESSION_CONTRACT_APP_TOKENS if token not in app_ui_text]
    missing_s6_module_tokens = [
        token for token in S6_SESSION_CONTRACT_MODULE_TOKENS if token not in overlay_session_contract_text
    ]
    s6_overlay_session_contract_ok = len(missing_s6_app_tokens) == 0 and len(missing_s6_module_tokens) == 0
    criteria.append(
        {
            "name": "s6_overlay_session_contract_wired",
            "ok": s6_overlay_session_contract_ok,
            "detail": "app_missing={} module_missing={} app_path={} module_path={}".format(
                len(missing_s6_app_tokens),
                len(missing_s6_module_tokens),
                AGE5_APP_UI_PATH,
                OVERLAY_SESSION_CONTRACT_PATH,
            ),
        }
    )
    if not s6_overlay_session_contract_ok:
        failure_digest.append(
            "s6_overlay_session_contract_wired: app_missing={} module_missing={}".format(
                sample_items(missing_s6_app_tokens),
                sample_items(missing_s6_module_tokens),
            )
        )
        pending_items.append("S6 overlay session contract 모듈과 app.js 연결 토큰 유지")

    missing_s6_view_combo_app_tokens = [
        token for token in S6_VIEW_COMBO_CONTRACT_APP_TOKENS if token not in app_ui_text
    ]
    missing_s6_view_combo_module_tokens = [
        token for token in S6_VIEW_COMBO_CONTRACT_MODULE_TOKENS if token not in overlay_session_contract_text
    ]
    s6_view_combo_contract_ok = (
        len(missing_s6_view_combo_app_tokens) == 0 and len(missing_s6_view_combo_module_tokens) == 0
    )
    criteria.append(
        {
            "name": "s6_view_combo_session_contract_wired",
            "ok": s6_view_combo_contract_ok,
            "detail": "app_missing={} module_missing={} app_path={} module_path={}".format(
                len(missing_s6_view_combo_app_tokens),
                len(missing_s6_view_combo_module_tokens),
                AGE5_APP_UI_PATH,
                OVERLAY_SESSION_CONTRACT_PATH,
            ),
        }
    )
    if not s6_view_combo_contract_ok:
        failure_digest.append(
            "s6_view_combo_session_contract_wired: app_missing={} module_missing={}".format(
                sample_items(missing_s6_view_combo_app_tokens),
                sample_items(missing_s6_view_combo_module_tokens),
            )
        )
        pending_items.append("S6 view_combo session contract 모듈과 app.js 연결 토큰 유지")

    missing_s6_session_pack_cases = [
        str(path) for path in S6_SESSION_PACK_CASE_FILES if not (root / path).exists()
    ]
    s6_session_pack_cases_present_ok = len(missing_s6_session_pack_cases) == 0
    criteria.append(
        {
            "name": "s6_session_pack_cases_present",
            "ok": s6_session_pack_cases_present_ok,
            "detail": f"present={len(S6_SESSION_PACK_CASE_FILES) - len(missing_s6_session_pack_cases)}/{len(S6_SESSION_PACK_CASE_FILES)} root={S6_SESSION_PACK_HINT}",
        }
    )
    if not s6_session_pack_cases_present_ok:
        failure_digest.append(
            f"s6_session_pack_cases_present: missing={clip(', '.join(missing_s6_session_pack_cases), 200)}"
        )
        pending_items.append("S6 session roundtrip pack 케이스(c01~c08) 파일 유지")

    s6_session_pack_golden_text = load_text(root / S6_SESSION_PACK_GOLDEN_PATH)
    s6_session_pack_golden_case_count = count_nonempty_lines(s6_session_pack_golden_text)
    s6_session_pack_golden_min_ok = s6_session_pack_golden_case_count >= S6_SESSION_PACK_MIN_CASE_COUNT
    criteria.append(
        {
            "name": "s6_session_pack_golden_min_cases",
            "ok": s6_session_pack_golden_min_ok,
            "detail": f"count={s6_session_pack_golden_case_count} required>={S6_SESSION_PACK_MIN_CASE_COUNT} path={S6_SESSION_PACK_GOLDEN_PATH}",
        }
    )
    if not s6_session_pack_golden_min_ok:
        failure_digest.append(
            f"s6_session_pack_golden_min_cases: count={s6_session_pack_golden_case_count} required>={S6_SESSION_PACK_MIN_CASE_COUNT}"
        )
        pending_items.append("S6 session roundtrip golden.jsonl 케이스 수를 최소 8개 이상으로 유지")

    missing_pack_cases = [str(path) for path in PACK_CASE_FILES if not (root / path).exists()]
    pack_cases_present_ok = len(missing_pack_cases) == 0
    criteria.append(
        {
            "name": "s5_pack_cases_present",
            "ok": pack_cases_present_ok,
            "detail": f"present={len(PACK_CASE_FILES) - len(missing_pack_cases)}/{len(PACK_CASE_FILES)} root={PACK_HINT}",
        }
    )
    if not pack_cases_present_ok:
        failure_digest.append(f"s5_pack_cases_present: missing={clip(', '.join(missing_pack_cases), 200)}")
        pending_items.append("S5 overlay compare pack 케이스(c01~c76) 파일 유지")

    pack_golden_text = load_text(root / PACK_GOLDEN_PATH)
    pack_golden_case_count = count_nonempty_lines(pack_golden_text)
    pack_golden_min_ok = pack_golden_case_count >= PACK_MIN_CASE_COUNT
    criteria.append(
        {
            "name": "s5_pack_golden_min_cases",
            "ok": pack_golden_min_ok,
            "detail": f"count={pack_golden_case_count} required>={PACK_MIN_CASE_COUNT} path={PACK_GOLDEN_PATH}",
        }
    )
    if not pack_golden_min_ok:
        failure_digest.append(
            f"s5_pack_golden_min_cases: count={pack_golden_case_count} required>={PACK_MIN_CASE_COUNT}"
        )
        pending_items.append("S5 overlay compare golden.jsonl 케이스 수를 최소 76개 이상으로 유지")

    pack_golden_refs, pack_golden_parse_errors = load_overlay_compare_case_refs(root / PACK_GOLDEN_PATH)
    pack_root = Path(PACK_HINT)
    expected_refs = [str(path.relative_to(pack_root)).replace("\\", "/") for path in PACK_CASE_FILES]
    repair_hint = build_order_repair_hint(PACK_GOLDEN_PATH, expected_refs)
    repair_cmd_short = build_order_repair_cmd_short(PACK_GOLDEN_PATH)
    repair_cmd = build_order_repair_cmd(PACK_GOLDEN_PATH, expected_refs)
    repair = {
        "order": {
            "hint": repair_hint,
            "repair_cmd_short": repair_cmd_short,
            "repair_cmd": repair_cmd,
            "expected_case_list_path": str(PACK_GOLDEN_PATH),
            "expected_case_count": len(expected_refs),
            "expected_case_head_tail": head_tail(expected_refs),
        }
    }
    expected_ref_set = set(expected_refs)
    actual_ref_set = set(pack_golden_refs)
    missing_in_golden = sorted(expected_ref_set - actual_ref_set)
    unknown_in_golden = sorted(actual_ref_set - expected_ref_set)
    seen: set[str] = set()
    duplicate_in_golden: set[str] = set()
    for ref in pack_golden_refs:
        if ref in seen:
            duplicate_in_golden.add(ref)
        seen.add(ref)
    golden_case_map_ok = (
        not pack_golden_parse_errors and not missing_in_golden and not unknown_in_golden and not duplicate_in_golden
    )
    criteria.append(
        {
            "name": "s5_pack_golden_case_map_match",
            "ok": golden_case_map_ok,
            "detail": "parse_errors={} missing={} unknown={} duplicate={} parse_sample={} missing_sample={} unknown_sample={} duplicate_sample={}".format(
                len(pack_golden_parse_errors),
                len(missing_in_golden),
                len(unknown_in_golden),
                len(duplicate_in_golden),
                sample_items(pack_golden_parse_errors),
                sample_items(missing_in_golden),
                sample_items(unknown_in_golden),
                sample_items(sorted(duplicate_in_golden)),
            ),
        }
    )
    if not golden_case_map_ok:
        issues: list[str] = []
        if pack_golden_parse_errors:
            issues.append(f"parse={clip(', '.join(pack_golden_parse_errors), 200)}")
            failure_digest.append(
                "s5_pack_golden_case_map_match.parse: sample={} full={}".format(
                    sample_items(pack_golden_parse_errors),
                    clip(full_items(pack_golden_parse_errors), 600),
                )
            )
        if missing_in_golden:
            issues.append(f"missing={clip(', '.join(missing_in_golden), 200)}")
            failure_digest.append(
                "s5_pack_golden_case_map_match.missing: sample={} full={}".format(
                    sample_items(missing_in_golden),
                    clip(full_items(missing_in_golden), 600),
                )
            )
        if unknown_in_golden:
            issues.append(f"unknown={clip(', '.join(unknown_in_golden), 200)}")
            failure_digest.append(
                "s5_pack_golden_case_map_match.unknown: sample={} full={}".format(
                    sample_items(unknown_in_golden),
                    clip(full_items(unknown_in_golden), 600),
                )
            )
        if duplicate_in_golden:
            issues.append(f"duplicate={clip(', '.join(sorted(duplicate_in_golden)), 200)}")
            duplicate_sorted = sorted(duplicate_in_golden)
            failure_digest.append(
                "s5_pack_golden_case_map_match.duplicate: sample={} full={}".format(
                    sample_items(duplicate_sorted),
                    clip(full_items(duplicate_sorted), 600),
                )
            )
        failure_digest.append(f"s5_pack_golden_case_map_match: {'; '.join(issues)}")
        pending_items.append("S5 overlay compare golden.jsonl과 PACK_CASE_FILES(c01~c76) 매핑 일치 유지")

    order_mismatch_index = -1
    if not pack_golden_parse_errors:
        min_len = min(len(expected_refs), len(pack_golden_refs))
        for idx in range(min_len):
            if expected_refs[idx] != pack_golden_refs[idx]:
                order_mismatch_index = idx
                break
        if order_mismatch_index < 0 and len(expected_refs) != len(pack_golden_refs):
            order_mismatch_index = min_len
    order_ok = (not pack_golden_parse_errors) and (order_mismatch_index < 0)
    expected_at = expected_refs[order_mismatch_index] if 0 <= order_mismatch_index < len(expected_refs) else "-"
    actual_at = pack_golden_refs[order_mismatch_index] if 0 <= order_mismatch_index < len(pack_golden_refs) else "-"
    criteria.append(
        {
            "name": "s5_pack_golden_case_order_stable",
            "ok": order_ok,
            "detail": "mismatch_index={} expected_at={} actual_at={} expected_window={} actual_window={}".format(
                (order_mismatch_index + 1) if order_mismatch_index >= 0 else 0,
                expected_at,
                actual_at,
                sample_window(expected_refs, order_mismatch_index),
                sample_window(pack_golden_refs, order_mismatch_index),
            ),
        }
    )
    if not order_ok:
        failure_digest.append(
            "s5_pack_golden_case_order_stable: mismatch_index={} expected_at={} actual_at={} expected_window={} actual_window={}".format(
                (order_mismatch_index + 1) if order_mismatch_index >= 0 else 0,
                expected_at,
                actual_at,
                sample_window(expected_refs, order_mismatch_index),
                sample_window(pack_golden_refs, order_mismatch_index),
            )
        )
        failure_digest.append(
            "s5_pack_golden_case_order_stable.head_tail: expected_{} actual_{}".format(
                head_tail(expected_refs),
                head_tail(pack_golden_refs),
            )
        )
        failure_digest.append(f"s5_pack_golden_case_order_stable.{repair_hint}")
        failure_digest.append(f"s5_pack_golden_case_order_stable.repair_cmd_short: {clip(repair_cmd_short, 700)}")
        failure_digest.append(f"s5_pack_golden_case_order_stable.repair_cmd: {clip(repair_cmd, 700)}")
        pending_items.append("S5 overlay compare golden.jsonl 케이스 순서를 c01~c76 고정 순서로 유지")

    invalid_case_tokens: list[str] = []
    for path in PACK_CASE_FILES:
        text = load_text(root / path)
        if "\"overlay_ok\"" not in text:
            invalid_case_tokens.append(str(path))
    pack_cases_token_ok = len(invalid_case_tokens) == 0
    criteria.append(
        {
            "name": "s5_pack_cases_overlay_ok_token",
            "ok": pack_cases_token_ok,
            "detail": f"invalid={len(invalid_case_tokens)}",
        }
    )
    if not pack_cases_token_ok:
        failure_digest.append(f"s5_pack_cases_overlay_ok_token: missing overlay_ok token in {clip(', '.join(invalid_case_tokens), 200)}")
        pending_items.append("S5 pack 케이스에 overlay_ok 기대값 유지")

    return criteria, failure_digest[:20], pending_items, repair


def main() -> int:
    parser = argparse.ArgumentParser(description="Evaluate AGE5 close-lite criteria from docs + UI slot declarations")
    parser.add_argument(
        "--report-out",
        default=default_report_path("age5_close_report.detjson"),
        help="output age5 close report path",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    report_out = Path(args.report_out)
    criteria, failure_digest, pending_items, repair = build_criteria(root)

    overall_ok = all(bool(row.get("ok", False)) for row in criteria)
    report = {
        "schema": "ddn.age5_close_report.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "overall_ok": overall_ok,
        "criteria": criteria,
        "paths": {
            "age4_s2_task": str(AGE4_S2_TASK_PATH),
            "s5_baseline_task": str(S5_BASELINE_TASK_PATH),
            "s5_detailed_task": str(S5_DETAILED_TASK_PATH),
            "slot_ui": str(AGE5_SLOT_UI_PATH),
            "app_ui": str(AGE5_APP_UI_PATH),
            "overlay_session_contract": str(OVERLAY_SESSION_CONTRACT_PATH),
            "pack_hint": PACK_HINT,
            "pack_golden": str(PACK_GOLDEN_PATH),
            "session_pack_hint": S6_SESSION_PACK_HINT,
            "session_pack_golden": str(S6_SESSION_PACK_GOLDEN_PATH),
        },
        "failure_digest": failure_digest[:20],
        "pending_items": pending_items,
        "repair": repair,
    }

    report_out.parent.mkdir(parents=True, exist_ok=True)
    report_out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    failed = sum(1 for row in criteria if not bool(row.get("ok", False)))
    print(f"[age5-close] overall_ok={int(overall_ok)} criteria={len(criteria)} failed={failed} report={report_out}")
    for row in criteria:
        print(f" - {row.get('name')}: ok={int(bool(row.get('ok', False)))}")
    if not overall_ok:
        for line in failure_digest[:8]:
            print(f"   {line}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
