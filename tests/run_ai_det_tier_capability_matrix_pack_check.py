#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


PACK_SCHEMA = "ddn.ai_det_tier_capability_matrix.pack.contract.v1"
REQUIRED_CASES = {
    "c01_hooks_continue_not_open_world": ("hooks_continue", "cmd", "pack/lang_continue_skip_v1/c01_foreach_skip_ok/input.ddn"),
    "c02_iterable_not_auto_open": ("iterable", "cmd", "tools/teul-cli/tests/golden/W97/W97_G03_foreach_bad_iterable/main.ddn"),
    "c03_overlay_view_only_not_state_hash": ("overlay", "smoke_golden", "smoke_with_view_boundary.v1.json"),
    "c04_ai_model_kind_infer_only": ("ai_model_kind", "cmd", "pack/gogae8_w85_ondevice_infer_v1/model.detjson"),
}
REQUIRED_AXES = {"ai_model_kind", "hooks", "continue", "iterable", "overlay"}


def fail(code: str, msg: str) -> int:
    print(f"[ai-det-tier-capability-matrix-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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
        line = raw.strip()
        if not line:
            continue
        try:
            row = json.loads(line)
        except Exception as exc:
            raise ValueError(f"invalid jsonl row: {path}:{line_no} ({exc})")
        if not isinstance(row, dict):
            raise ValueError(f"jsonl row must be object: {path}:{line_no}")
        rows.append(row)
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description="AI det-tier capability matrix supporting pack checker")
    parser.add_argument("--pack", default="pack/ai_det_tier_capability_matrix_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required = [pack / "README.md", pack / "intent.md", pack / "contract.detjson", pack / "golden.jsonl", pack / "tests" / "README.md"]
    required.extend((pack / "cases" / case_id / "expected.json") for case_id in REQUIRED_CASES)
    missing = [str(path).replace("\\", "/") for path in required if not path.exists()]
    if missing:
        return fail("E_AI_DET_TIER_MATRIX_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
        golden = load_jsonl(pack / "golden.jsonl")
    except ValueError as exc:
        return fail("E_AI_DET_TIER_MATRIX_INVALID", str(exc))

    if not isinstance(contract, dict):
        return fail("E_AI_DET_TIER_MATRIX_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_AI_DET_TIER_MATRIX_SCHEMA", f"schema={contract.get('schema')}")
    if str(contract.get("evidence_tier", "")).strip() != "runner_fill":
        return fail("E_AI_DET_TIER_MATRIX_TIER", f"tier={contract.get('evidence_tier')}")
    if str(contract.get("closure_claim", "")).strip() != "no":
        return fail("E_AI_DET_TIER_MATRIX_CLOSURE", f"closure={contract.get('closure_claim')}")
    axes = set(contract.get("required_axes", []))
    if axes != REQUIRED_AXES:
        return fail("E_AI_DET_TIER_MATRIX_AXES", f"axes={sorted(axes)}")
    if str(contract.get("gaji_contract_form", "")).strip() != "top_level_det_tier_openness":
        return fail("E_AI_DET_TIER_MATRIX_GAJI_FORM", str(contract.get("gaji_contract_form")))
    forbidden_keys = set(contract.get("forbidden_keys", []))
    if forbidden_keys != {"requires.det", "requires.open"}:
        return fail("E_AI_DET_TIER_MATRIX_FORBIDDEN_KEYS", f"keys={sorted(forbidden_keys)}")

    for rel in contract.get("evidence_targets", []):
        if not Path(rel).exists():
            return fail("E_AI_DET_TIER_MATRIX_TARGET", rel)

    cases = contract.get("cases")
    if not isinstance(cases, list):
        return fail("E_AI_DET_TIER_MATRIX_CASES_TYPE", "cases must be list")
    case_index = {str(row.get("id", "")).strip(): row for row in cases if isinstance(row, dict)}
    if set(case_index) != set(REQUIRED_CASES):
        return fail("E_AI_DET_TIER_MATRIX_CASE_SET", f"cases={sorted(case_index)}")
    golden_index = {str(row.get("id", "")).strip(): row for row in golden}
    if set(golden_index) != set(REQUIRED_CASES):
        return fail("E_AI_DET_TIER_MATRIX_GOLDEN_SET", f"golden={sorted(golden_index)}")
    for case_id, (topic, runner_mode, runner_target) in REQUIRED_CASES.items():
        if str(case_index[case_id].get("topic", "")).strip() != topic:
            return fail("E_AI_DET_TIER_MATRIX_TOPIC", case_id)
        if str(case_index[case_id].get("runner_mode", "")).strip() != runner_mode:
            return fail("E_AI_DET_TIER_MATRIX_MODE", case_id)
        if str(case_index[case_id].get("runner_target", "")).strip() != runner_target:
            return fail("E_AI_DET_TIER_MATRIX_TARGET_ROW", case_id)
        expected = load_json(pack / "cases" / case_id / "expected.json")
        if not isinstance(expected, dict) or str(expected.get("topic", "")).strip() != topic:
            return fail("E_AI_DET_TIER_MATRIX_EXPECTED", case_id)
        golden_row = golden_index[case_id]
        if runner_mode == "smoke_golden":
            if str(golden_row.get("smoke_golden", "")).strip() != runner_target:
                return fail("E_AI_DET_TIER_MATRIX_GOLDEN_TARGET", case_id)
        else:
            cmd = golden_row.get("cmd")
            if not isinstance(cmd, list) or not cmd:
                return fail("E_AI_DET_TIER_MATRIX_CMD", case_id)
            if case_id == "c04_ai_model_kind_infer_only":
                if cmd[:4] != ["infer", "mlp", "pack/gogae8_w85_ondevice_infer_v1/model.detjson", "pack/gogae8_w85_ondevice_infer_v1/input.detjson"]:
                    return fail("E_AI_DET_TIER_MATRIX_INFER_CMD", case_id)
            elif len(cmd) < 2 or cmd[0] != "run" or cmd[1] != runner_target:
                return fail("E_AI_DET_TIER_MATRIX_CMD_TARGET", case_id)

    gaji_samples = [
        Path("gaji/std_charim/gaji.toml"),
        Path("gaji/bogae/space2d/gaji.toml"),
    ]
    for sample in gaji_samples:
        if not sample.exists():
            return fail("E_AI_DET_TIER_MATRIX_GAJI_MISSING", str(sample).replace("\\", "/"))
        text = sample.read_text(encoding="utf-8")
        if "requires.det" in text or "requires.open" in text:
            return fail("E_AI_DET_TIER_MATRIX_FORBIDDEN_DOTKEY", str(sample).replace("\\", "/"))
        if "det_tier" not in text or "openness" not in text:
            return fail("E_AI_DET_TIER_MATRIX_GAJI_FIELDS", str(sample).replace("\\", "/"))

    print("[ai-det-tier-capability-matrix-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(golden_index)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
