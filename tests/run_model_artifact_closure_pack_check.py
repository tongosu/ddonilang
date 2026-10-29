#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


PACK_SCHEMA = "ddn.model_artifact_closure.pack.contract.v1"
EXPECTED_CASES = {
    "c01_infer_only_mlp": ["infer", "mlp", "pack/gogae8_w85_ondevice_infer_v1/model.detjson", "pack/gogae8_w85_ondevice_infer_v1/input.detjson", "--out", "pack/model_artifact_closure_v1/out/infer"],
    "c02_eval_pass_artifact": ["eval", "pack/gogae8_w87_eval_suite_v1/eval_config_pass.json", "--out", "pack/model_artifact_closure_v1/out/pass"],
    "c03_eval_fail_artifact": ["eval", "pack/gogae8_w87_eval_suite_v1/eval_config_fail.json", "--out", "pack/model_artifact_closure_v1/out/fail"]
}


def fail(code: str, msg: str) -> int:
    print(f"[model-artifact-closure-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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
    parser = argparse.ArgumentParser(description="Model artifact closure pack checker")
    parser.add_argument("--pack", default="pack/model_artifact_closure_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required = [pack / "README.md", pack / "intent.md", pack / "contract.detjson", pack / "golden.jsonl", pack / "tests" / "README.md", pack / "expected" / "artifact_pass.detjson", pack / "expected" / "artifact_fail.detjson"]
    required.extend((pack / "cases" / case_id / "expected.json") for case_id in EXPECTED_CASES)
    missing = [str(path).replace("\\", "/") for path in required if not path.exists()]
    if missing:
        return fail("E_MODEL_ARTIFACT_CLOSURE_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
        golden = load_jsonl(pack / "golden.jsonl")
    except ValueError as exc:
        return fail("E_MODEL_ARTIFACT_CLOSURE_INVALID", str(exc))

    if not isinstance(contract, dict):
        return fail("E_MODEL_ARTIFACT_CLOSURE_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_MODEL_ARTIFACT_CLOSURE_SCHEMA", f"schema={contract.get('schema')}")
    if str(contract.get("evidence_tier", "")).strip() != "golden_closed":
        return fail("E_MODEL_ARTIFACT_CLOSURE_TIER", f"tier={contract.get('evidence_tier')}")
    if str(contract.get("closure_claim", "")).strip() != "yes":
        return fail("E_MODEL_ARTIFACT_CLOSURE_CLAIM", f"closure={contract.get('closure_claim')}")
    for rel in contract.get("evidence_targets", []):
        if not Path(rel).exists():
            return fail("E_MODEL_ARTIFACT_CLOSURE_TARGET", rel)

    golden_index = {str(row.get("id", "")).strip(): row for row in golden}
    if set(golden_index) != set(EXPECTED_CASES):
        return fail("E_MODEL_ARTIFACT_CLOSURE_GOLDEN_SET", f"golden={sorted(golden_index)}")

    for case_id, expected_cmd in EXPECTED_CASES.items():
        expected = load_json(pack / "cases" / case_id / "expected.json")
        if not isinstance(expected, dict):
            return fail("E_MODEL_ARTIFACT_CLOSURE_EXPECTED", case_id)
        row = golden_index[case_id]
        if row.get("cmd") != expected_cmd:
            return fail("E_MODEL_ARTIFACT_CLOSURE_CMD", case_id)
        if row.get("stdout") != expected.get("stdout"):
            return fail("E_MODEL_ARTIFACT_CLOSURE_STDOUT", case_id)
        if case_id != "c01_infer_only_mlp":
            if str(row.get("expected_meta", "")).strip() != str(expected.get("expected_meta", "")).strip():
                return fail("E_MODEL_ARTIFACT_CLOSURE_META", case_id)
            if not str(row.get("meta_out", "")).strip().startswith("out/"):
                return fail("E_MODEL_ARTIFACT_CLOSURE_META_OUT", case_id)
        if int(row.get("exit_code", -1)) != 0:
            return fail("E_MODEL_ARTIFACT_CLOSURE_EXIT", case_id)

    print("[model-artifact-closure-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(golden_index)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
