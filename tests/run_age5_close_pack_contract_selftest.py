#!/usr/bin/env python
from __future__ import annotations

import importlib.util
import json
import shutil
import uuid
from pathlib import Path


def load_age5_close_module(root: Path):
    path = root / "tests" / "run_age5_close.py"
    spec = importlib.util.spec_from_file_location("age5_close_mod", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"module load failed: {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def write_jsonl(path: Path, key: str, refs: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    rows = [json.dumps({key: ref}, ensure_ascii=False) for ref in refs]
    path.write_text("\n".join(rows) + "\n", encoding="utf-8")


def criteria_ok_map(criteria: list[dict[str, object]]) -> dict[str, bool]:
    out: dict[str, bool] = {}
    for row in criteria:
        name = str(row.get("name", "")).strip()
        if not name:
            continue
        out[name] = bool(row.get("ok", False))
    return out


def has_digest_prefix(lines: list[str], prefix: str) -> bool:
    return any(str(line).startswith(prefix) for line in lines)


def resolve_tmp_root(root: Path) -> Path:
    candidates = [
        Path("I:/home/urihanl/ddn/codex/build/tmp"),
        Path("C:/ddn/codex/build/tmp"),
        root / "build" / "tmp",
    ]
    for candidate in candidates:
        try:
            candidate.mkdir(parents=True, exist_ok=True)
            return candidate
        except OSError:
            continue
    fallback = root / "build" / "tmp"
    fallback.mkdir(parents=True, exist_ok=True)
    return fallback


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    mod = load_age5_close_module(root)

    pack_root = Path(mod.PACK_HINT)
    session_pack_root = Path(mod.S6_SESSION_PACK_HINT)
    expected_s5_refs = [str(path.relative_to(pack_root)).replace("\\", "/") for path in mod.PACK_CASE_FILES]
    expected_s6_refs = [str(path.relative_to(session_pack_root)).replace("\\", "/") for path in mod.S6_SESSION_PACK_CASE_FILES]

    temp_root = resolve_tmp_root(root) / f"age5_close_pack_contract_selftest_{uuid.uuid4().hex[:8]}"
    s5_ordered = temp_root / "s5_ordered.golden.jsonl"
    s5_shuffled = temp_root / "s5_shuffled.golden.jsonl"
    s6_ordered = temp_root / "s6_ordered.golden.jsonl"
    s6_shuffled = temp_root / "s6_shuffled.golden.jsonl"

    write_jsonl(s5_ordered, "overlay_compare_case", expected_s5_refs)
    write_jsonl(s6_ordered, "overlay_session_case", expected_s6_refs)
    if len(expected_s5_refs) >= 2:
        s5_swapped = expected_s5_refs[:]
        s5_swapped[0], s5_swapped[1] = s5_swapped[1], s5_swapped[0]
    else:
        s5_swapped = expected_s5_refs[:]
    if len(expected_s6_refs) >= 2:
        s6_swapped = expected_s6_refs[:]
        s6_swapped[0], s6_swapped[1] = s6_swapped[1], s6_swapped[0]
    else:
        s6_swapped = expected_s6_refs[:]
    write_jsonl(s5_shuffled, "overlay_compare_case", s5_swapped)
    write_jsonl(s6_shuffled, "overlay_session_case", s6_swapped)

    orig_s5_golden = mod.PACK_GOLDEN_PATH
    orig_s6_golden = mod.S6_SESSION_PACK_GOLDEN_PATH
    orig_surface_contracts = mod.AGE5_SURFACE_PACK_CONTRACTS

    try:
        # case 1: ordered golden should satisfy s5/s6 map+order criteria.
        mod.PACK_GOLDEN_PATH = s5_ordered.resolve()
        mod.S6_SESSION_PACK_GOLDEN_PATH = s6_ordered.resolve()
        criteria_ok, _, _, repair_ok = mod.build_criteria(root, strict=True)
        ok_map = criteria_ok_map(criteria_ok)
        required_true = [
            "s5_pack_golden_case_map_match",
            "s5_pack_golden_case_order_stable",
            "s6_session_pack_golden_case_map_match",
            "s6_session_pack_golden_case_order_stable",
            "age5_surface_pack_contract_paths_present",
            "age5_surface_pack_contract_min_cases",
            "age5_surface_pack_contract_tokens_present",
        ]
        for name in required_true:
            if not ok_map.get(name, False):
                print(f"[age5-close-pack-contract-selftest] fail: ordered case must pass {name}")
                return 1
        order_repair = repair_ok.get("order")
        session_order_repair = repair_ok.get("session_order")
        if not isinstance(order_repair, dict) or not isinstance(session_order_repair, dict):
            print("[age5-close-pack-contract-selftest] fail: missing repair.order/session_order")
            return 1
        if int(order_repair.get("expected_case_count", -1)) != len(expected_s5_refs):
            print("[age5-close-pack-contract-selftest] fail: repair.order expected_case_count mismatch")
            return 1
        if int(session_order_repair.get("expected_case_count", -1)) != len(expected_s6_refs):
            print("[age5-close-pack-contract-selftest] fail: repair.session_order expected_case_count mismatch")
            return 1

        # case 2: s6 order mismatch should fail only order criterion and emit repair digest.
        mod.PACK_GOLDEN_PATH = s5_ordered.resolve()
        mod.S6_SESSION_PACK_GOLDEN_PATH = s6_shuffled.resolve()
        criteria_s6_bad, failure_s6_bad, _, _ = mod.build_criteria(root, strict=True)
        s6_bad_map = criteria_ok_map(criteria_s6_bad)
        if not s6_bad_map.get("s6_session_pack_golden_case_map_match", False):
            print("[age5-close-pack-contract-selftest] fail: s6 shuffled should keep case map match")
            return 1
        if s6_bad_map.get("s6_session_pack_golden_case_order_stable", True):
            print("[age5-close-pack-contract-selftest] fail: s6 shuffled must fail case order stable")
            return 1
        if not has_digest_prefix(failure_s6_bad, "s6_session_pack_golden_case_order_stable.repair_cmd:"):
            print("[age5-close-pack-contract-selftest] fail: s6 repair_cmd digest missing")
            return 1

        # case 3: s5 order mismatch should fail only order criterion and emit repair digest.
        mod.PACK_GOLDEN_PATH = s5_shuffled.resolve()
        mod.S6_SESSION_PACK_GOLDEN_PATH = s6_ordered.resolve()
        criteria_s5_bad, failure_s5_bad, _, _ = mod.build_criteria(root, strict=True)
        s5_bad_map = criteria_ok_map(criteria_s5_bad)
        if not s5_bad_map.get("s5_pack_golden_case_map_match", False):
            print("[age5-close-pack-contract-selftest] fail: s5 shuffled should keep case map match")
            return 1
        if s5_bad_map.get("s5_pack_golden_case_order_stable", True):
            print("[age5-close-pack-contract-selftest] fail: s5 shuffled must fail case order stable")
            return 1
        if not has_digest_prefix(failure_s5_bad, "s5_pack_golden_case_order_stable.repair_cmd:"):
            print("[age5-close-pack-contract-selftest] fail: s5 repair_cmd digest missing")
            return 1

        # case 4: surface min_cases over-constraint should fail min-case criterion only.
        min_case_bad_contracts = [dict(item) for item in orig_surface_contracts]
        if not min_case_bad_contracts:
            print("[age5-close-pack-contract-selftest] fail: missing AGE5_SURFACE_PACK_CONTRACTS")
            return 1
        first_contract = dict(min_case_bad_contracts[0])
        first_contract["min_cases"] = 10**9
        min_case_bad_contracts[0] = first_contract
        mod.AGE5_SURFACE_PACK_CONTRACTS = min_case_bad_contracts
        mod.PACK_GOLDEN_PATH = s5_ordered.resolve()
        mod.S6_SESSION_PACK_GOLDEN_PATH = s6_ordered.resolve()
        criteria_min_bad, failure_min_bad, _, _ = mod.build_criteria(root, strict=True)
        min_bad_map = criteria_ok_map(criteria_min_bad)
        if not min_bad_map.get("age5_surface_pack_contract_paths_present", False):
            print("[age5-close-pack-contract-selftest] fail: min-case bad should keep paths_present ok")
            return 1
        if min_bad_map.get("age5_surface_pack_contract_min_cases", True):
            print("[age5-close-pack-contract-selftest] fail: min-case bad must fail min_cases criterion")
            return 1
        if not has_digest_prefix(failure_min_bad, "age5_surface_pack_contract_min_cases:"):
            print("[age5-close-pack-contract-selftest] fail: min-case failure digest missing")
            return 1

        # case 5: surface missing token should fail token criterion only.
        token_bad_contracts = [dict(item) for item in orig_surface_contracts]
        first_contract_token = dict(token_bad_contracts[0])
        token_list = list(first_contract_token.get("tokens", []))
        token_list.append("__AGE5_SURFACE_TOKEN_MISSING_SELFTEST__")
        first_contract_token["tokens"] = token_list
        token_bad_contracts[0] = first_contract_token
        mod.AGE5_SURFACE_PACK_CONTRACTS = token_bad_contracts
        mod.PACK_GOLDEN_PATH = s5_ordered.resolve()
        mod.S6_SESSION_PACK_GOLDEN_PATH = s6_ordered.resolve()
        criteria_token_bad, failure_token_bad, _, _ = mod.build_criteria(root, strict=True)
        token_bad_map = criteria_ok_map(criteria_token_bad)
        if not token_bad_map.get("age5_surface_pack_contract_paths_present", False):
            print("[age5-close-pack-contract-selftest] fail: token bad should keep paths_present ok")
            return 1
        if not token_bad_map.get("age5_surface_pack_contract_min_cases", False):
            print("[age5-close-pack-contract-selftest] fail: token bad should keep min_cases ok")
            return 1
        if token_bad_map.get("age5_surface_pack_contract_tokens_present", True):
            print("[age5-close-pack-contract-selftest] fail: token bad must fail tokens_present criterion")
            return 1
        if not has_digest_prefix(failure_token_bad, "age5_surface_pack_contract_tokens_present:"):
            print("[age5-close-pack-contract-selftest] fail: token failure digest missing")
            return 1

        print("[age5-close-pack-contract-selftest] ok")
        return 0
    finally:
        mod.PACK_GOLDEN_PATH = orig_s5_golden
        mod.S6_SESSION_PACK_GOLDEN_PATH = orig_s6_golden
        mod.AGE5_SURFACE_PACK_CONTRACTS = orig_surface_contracts
        shutil.rmtree(temp_root, ignore_errors=True)


if __name__ == "__main__":
    raise SystemExit(main())
