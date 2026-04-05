#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


REQUIRED_CASES = {
    "c01_human_input_replay": ("사람", "실주입", "세계영향"),
    "c02_seulgi_injection_replay": ("슬기", "실주입", "세계영향"),
    "c03_gatekeeper_reject": ("슬기", "실주입", "세계영향"),
    "c04_schedule_event_same_boundary": ("일정", "실주입", "세계영향"),
    "c05_relay_event_replay": ("이어전달", "재연주입", "세계영향"),
    "c06_rollout_event_replay": ("펼침실행", "재연주입", "보기만"),
}
REPLAY_STABILITY_SCHEMA = "ddn.external_intent_boundary.replay_stability.v1"

REQUIRED_INPUT_SOURCES = ["사람", "슬기", "밖일", "일정", "이어전달", "펼침실행"]
REQUIRED_INJECTION_MODES = ["실주입", "재연주입"]
REQUIRED_ACTION_KINDS = ["보기만", "세계영향"]
REQUIRED_INVARIANTS = {
    "direct_state_mutation_forbidden",
    "sam_boundary_required",
    "gatekeeper_or_policy_check_required",
    "replay_source_recall_forbidden",
    "determinism_after_injection",
}


def fail(code: str, msg: str) -> int:
    print(f"[external-intent-boundary-pack-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(path: Path) -> object:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise ValueError(f"missing file: {path}")
    except Exception as exc:
        raise ValueError(f"invalid json: {path} ({exc})")


def load_jsonl(path: Path) -> list[dict]:
    rows: list[dict] = []
    for line_no, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        text = raw.strip()
        if not text:
            continue
        try:
            row = json.loads(text)
        except Exception as exc:
            raise ValueError(f"invalid jsonl row: {path}:{line_no} ({exc})")
        if not isinstance(row, dict):
            raise ValueError(f"jsonl row must be object: {path}:{line_no}")
        rows.append(row)
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description="External intent boundary pack contract checker")
    parser.add_argument(
        "--pack",
        default="pack/external_intent_boundary_v1",
        help="pack directory path",
    )
    args = parser.parse_args()

    pack = Path(args.pack)
    required_files = [
        pack / "README.md",
        pack / "intent.md",
        pack / "boundary_contract.detjson",
        pack / "golden.jsonl",
        pack / "replay_stability_100x.detjson",
    ]
    missing = [str(path).replace("\\", "/") for path in required_files if not path.exists()]
    if missing:
        return fail("E_EIB_FILE_MISSING", ",".join(missing))

    try:
        contract_doc = load_json(pack / "boundary_contract.detjson")
    except ValueError as exc:
        return fail("E_EIB_CONTRACT_INVALID", str(exc))
    if not isinstance(contract_doc, dict):
        return fail("E_EIB_CONTRACT_TYPE", "contract root must be object")
    if str(contract_doc.get("schema", "")).strip() != "ddn.external_intent_boundary.pack.contract.v1":
        return fail("E_EIB_SCHEMA", f"schema={contract_doc.get('schema')}")

    legacy_top_keys = {"origin_kinds"}
    legacy_hits = sorted(key for key in legacy_top_keys if key in contract_doc)
    if legacy_hits:
        return fail("E_EIB_LEGACY_KEY", f"legacy_top_keys={legacy_hits}")

    input_sources = contract_doc.get("입력원천_목록")
    if not isinstance(input_sources, list):
        return fail("E_EIB_INPUT_SOURCES_TYPE", "입력원천_목록 must be list")
    actual_input_sources = [str(item).strip() for item in input_sources if str(item).strip()]
    if sorted(actual_input_sources) != sorted(REQUIRED_INPUT_SOURCES):
        return fail("E_EIB_INPUT_SOURCES", f"입력원천_목록={actual_input_sources}")

    injection_modes = contract_doc.get("주입방식_목록")
    if not isinstance(injection_modes, list):
        return fail("E_EIB_INJECTION_MODES_TYPE", "주입방식_목록 must be list")
    actual_injection_modes = [str(item).strip() for item in injection_modes if str(item).strip()]
    if sorted(actual_injection_modes) != sorted(REQUIRED_INJECTION_MODES):
        return fail("E_EIB_INJECTION_MODES", f"주입방식_목록={actual_injection_modes}")

    action_kinds = contract_doc.get("행동갈래_목록")
    if not isinstance(action_kinds, list):
        return fail("E_EIB_ACTION_KINDS_TYPE", "행동갈래_목록 must be list")
    actual_action_kinds = [str(item).strip() for item in action_kinds if str(item).strip()]
    if sorted(actual_action_kinds) != sorted(REQUIRED_ACTION_KINDS):
        return fail("E_EIB_ACTION_KINDS", f"행동갈래_목록={actual_action_kinds}")

    invariants = contract_doc.get("invariants")
    if not isinstance(invariants, list):
        return fail("E_EIB_INVARIANTS_TYPE", "invariants must be list")
    invariant_ids: set[str] = set()
    for row in invariants:
        if not isinstance(row, dict):
            return fail("E_EIB_INVARIANT_ROW_TYPE", f"type={type(row).__name__}")
        invariant_id = str(row.get("id", "")).strip()
        if not invariant_id:
            return fail("E_EIB_INVARIANT_ID_MISSING", "id missing")
        if not bool(row.get("ok", False)):
            return fail("E_EIB_INVARIANT_NOT_OK", f"id={invariant_id}")
        invariant_ids.add(invariant_id)
    missing_invariants = sorted(REQUIRED_INVARIANTS - invariant_ids)
    if missing_invariants:
        return fail("E_EIB_INVARIANT_MISSING", ",".join(missing_invariants))

    replay_doc = contract_doc.get("replay")
    if not isinstance(replay_doc, dict):
        return fail("E_EIB_REPLAY_TYPE", "replay must be object")
    if "source_recall_forbidden" in replay_doc or "reinject_recorded_inputs_only" in replay_doc:
        return fail("E_EIB_LEGACY_KEY", "legacy replay keys are not allowed")
    if not bool(replay_doc.get("원천재호출_금지", False)):
        return fail("E_EIB_REPLAY_SOURCE_RECALL", "원천재호출_금지 must be true")
    if not bool(replay_doc.get("기록입력만_재주입", False)):
        return fail("E_EIB_REPLAY_REINJECT", "기록입력만_재주입 must be true")

    cases = contract_doc.get("cases")
    if not isinstance(cases, list):
        return fail("E_EIB_CASES_TYPE", "cases must be list")
    case_map: dict[str, dict] = {}
    for row in cases:
        if not isinstance(row, dict):
            return fail("E_EIB_CASE_ROW_TYPE", f"type={type(row).__name__}")
        case_id = str(row.get("id", "")).strip()
        if not case_id:
            return fail("E_EIB_CASE_ID_MISSING", "id missing")
        case_map[case_id] = row
    if sorted(case_map.keys()) != sorted(REQUIRED_CASES.keys()):
        return fail("E_EIB_CASE_SET", f"case_ids={sorted(case_map.keys())}")

    for case_id, expected_origin in REQUIRED_CASES.items():
        row = case_map[case_id]
        if any(
            key in row
            for key in (
                "origin_kind",
                "inject_mode",
                "action_kind",
                "replay_no_external_recall",
                "expected_state_hash_stable",
                "expected_commit",
            )
        ):
            return fail("E_EIB_LEGACY_KEY", f"{case_id}:legacy_case_keys")
        expected_source, expected_injection, expected_action = expected_origin
        if str(row.get("입력원천", "")).strip() != expected_source:
            return fail("E_EIB_CASE_SOURCE", f"{case_id}:{row.get('입력원천')}")
        if str(row.get("주입방식", "")).strip() != expected_injection:
            return fail("E_EIB_CASE_INJECTION", f"{case_id}:{row.get('주입방식')}")
        if str(row.get("행동갈래", "")).strip() != expected_action:
            return fail("E_EIB_CASE_ACTION", f"{case_id}:{row.get('행동갈래')}")
        if not bool(row.get("재연_원천재호출_없음", False)):
            return fail("E_EIB_CASE_REPLAY", f"{case_id}:재연_원천재호출_없음")
        if not bool(row.get("예상_상태해시_안정", False)):
            return fail("E_EIB_CASE_HASH", f"{case_id}:예상_상태해시_안정")

    try:
        golden_rows = load_jsonl(pack / "golden.jsonl")
    except ValueError as exc:
        return fail("E_EIB_GOLDEN_INVALID", str(exc))
    if not golden_rows:
        return fail("E_EIB_GOLDEN_EMPTY", "golden.jsonl must be non-empty")

    golden_ids: list[str] = []
    for row in golden_rows:
        case_id = str(row.get("id", "")).strip()
        if not case_id:
            return fail("E_EIB_GOLDEN_ID_MISSING", "id missing")
        if case_id not in REQUIRED_CASES:
            return fail("E_EIB_GOLDEN_UNKNOWN_CASE", case_id)
        if not bool(row.get("expect_ok", False)):
            return fail("E_EIB_GOLDEN_EXPECT_OK", case_id)
        if "expected_commit" in row:
            return fail("E_EIB_LEGACY_KEY", f"{case_id}:expected_commit")
        expected_commit = row.get("예상_확정")
        if not isinstance(expected_commit, bool):
            return fail("E_EIB_GOLDEN_EXPECTED_COMMIT", f"{case_id}:예상_확정")
        golden_ids.append(case_id)
    if sorted(golden_ids) != sorted(REQUIRED_CASES.keys()):
        return fail("E_EIB_GOLDEN_CASE_SET", f"golden_ids={sorted(golden_ids)}")

    try:
        replay_stability_doc = load_json(pack / "replay_stability_100x.detjson")
    except ValueError as exc:
        return fail("E_EIB_REPLAY_STABILITY_INVALID", str(exc))
    if not isinstance(replay_stability_doc, dict):
        return fail("E_EIB_REPLAY_STABILITY_TYPE", "replay_stability root must be object")
    if str(replay_stability_doc.get("schema", "")).strip() != REPLAY_STABILITY_SCHEMA:
        return fail("E_EIB_REPLAY_STABILITY_SCHEMA", f"schema={replay_stability_doc.get('schema')}")
    if str(replay_stability_doc.get("case_id", "")).strip() != "c01_human_input_replay":
        return fail("E_EIB_REPLAY_STABILITY_CASE", f"case_id={replay_stability_doc.get('case_id')}")
    runs = replay_stability_doc.get("runs")
    if not isinstance(runs, int) or runs < 100:
        return fail("E_EIB_REPLAY_STABILITY_RUNS", f"runs={runs}")
    hashes = replay_stability_doc.get("state_hashes")
    if not isinstance(hashes, list) or len(hashes) != int(runs):
        return fail(
            "E_EIB_REPLAY_STABILITY_HASH_COUNT",
            f"runs={runs} len={len(hashes) if isinstance(hashes, list) else -1}",
        )
    normalized_hashes = [str(item).strip() for item in hashes]
    if any(not item.startswith("sha256:") for item in normalized_hashes):
        return fail("E_EIB_REPLAY_STABILITY_HASH_FORMAT", "hash must start with sha256:")
    if len(set(normalized_hashes)) != 1:
        return fail("E_EIB_REPLAY_STABILITY_HASH_UNSTABLE", "state_hashes must be identical")

    print("[external-intent-boundary-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(case_map)}")
    print(f"invariants={len(invariant_ids)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
