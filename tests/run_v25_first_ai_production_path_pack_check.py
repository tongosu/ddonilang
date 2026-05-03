#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


PACK_SCHEMA = "ddn.v25_first_ai_production_path.pack.contract.v1"
EXPECTED_CASES = {
    "c01_dataset_export": ["dataset", "export", "--geoul", "pack/nuri_gym_canon_contract_v1/out/geoul", "--format", "nurigym_v1", "--out", "pack/v25_first_ai_production_path_v1/out/export", "--env-id", "nurigym.gridmaze2d"],
    "c02_train_artifact": ["train", "pack/v25_first_ai_production_path_v1/train_config.json", "--out", "pack/v25_first_ai_production_path_v1/out/train"],
    "c03_runtime_infer": ["infer", "mlp", "pack/v25_first_ai_production_path_v1/model.detjson", "pack/v25_first_ai_production_path_v1/infer_input.detjson", "--out", "pack/v25_first_ai_production_path_v1/out/infer"]
}


def fail(code: str, msg: str) -> int:
    print(f"[v25-first-ai-production-path-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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
    parser = argparse.ArgumentParser(description="v25 first AI production path pack checker")
    parser.add_argument("--pack", default="pack/v25_first_ai_production_path_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required = [pack / "README.md", pack / "intent.md", pack / "contract.detjson", pack / "golden.jsonl", pack / "train_config.json", pack / "model.detjson", pack / "infer_input.detjson", pack / "expected" / "dataset_hash.txt", pack / "expected" / "train_artifact.detjson", pack / "tests" / "README.md"]
    required.extend((pack / "cases" / case_id / "expected.json") for case_id in EXPECTED_CASES)
    missing = [str(path).replace("\\", "/") for path in required if not path.exists()]
    if missing:
        return fail("E_V25_PATH_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
        golden = load_jsonl(pack / "golden.jsonl")
        train_config = load_json(pack / "train_config.json")
        model = load_json(pack / "model.detjson")
        infer_input = load_json(pack / "infer_input.detjson")
    except ValueError as exc:
        return fail("E_V25_PATH_INVALID", str(exc))

    if not isinstance(contract, dict):
        return fail("E_V25_PATH_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_V25_PATH_SCHEMA", f"schema={contract.get('schema')}")
    if str(contract.get("evidence_tier", "")).strip() != "golden_closed":
        return fail("E_V25_PATH_TIER", f"tier={contract.get('evidence_tier')}")
    if str(contract.get("closure_claim", "")).strip() != "yes":
        return fail("E_V25_PATH_CLAIM", f"closure={contract.get('closure_claim')}")
    for rel in contract.get("evidence_targets", []):
        if not Path(rel).exists():
            return fail("E_V25_PATH_TARGET", rel)

    if not isinstance(train_config, dict) or not isinstance(model, dict) or not isinstance(infer_input, dict):
        return fail("E_V25_PATH_JSON_TYPE", "config/model/input must be object")
    dataset_hash = (pack / "expected" / "dataset_hash.txt").read_text(encoding="utf-8").strip()
    if str(train_config.get("dataset_hash", "")).strip() != dataset_hash:
        return fail("E_V25_PATH_DATASET_HASH", str(train_config.get("dataset_hash")))
    expected_bytes = (
        int(model.get("input_size", 0)) * int(model.get("hidden_size", 0))
        + int(model.get("hidden_size", 0))
        + int(model.get("output_size", 0)) * int(model.get("hidden_size", 0))
        + int(model.get("output_size", 0))
    ) * 2
    if int(train_config.get("weights_len", -1)) != expected_bytes:
        return fail("E_V25_PATH_WEIGHTS_LEN", str(train_config.get("weights_len")))
    if str(model.get("weights_path", "")).strip() != "out/train/weights.bin":
        return fail("E_V25_PATH_WEIGHTS_PATH", str(model.get("weights_path")))
    if len(infer_input.get("input", [])) != int(model.get("input_size", 0)):
        return fail("E_V25_PATH_INPUT_LEN", str(infer_input.get("input")))

    golden_index = {str(row.get("id", "")).strip(): row for row in golden}
    if set(golden_index) != set(EXPECTED_CASES):
        return fail("E_V25_PATH_GOLDEN_SET", f"golden={sorted(golden_index)}")

    for case_id, expected_cmd in EXPECTED_CASES.items():
        expected = load_json(pack / "cases" / case_id / "expected.json")
        if not isinstance(expected, dict):
            return fail("E_V25_PATH_EXPECTED", case_id)
        row = golden_index[case_id]
        if row.get("cmd") != expected_cmd:
            return fail("E_V25_PATH_CMD", case_id)
        if row.get("stdout") != expected.get("stdout"):
            return fail("E_V25_PATH_STDOUT", case_id)
        if case_id in {"c01_dataset_export", "c02_train_artifact"}:
            if str(row.get("expected_meta", "")).strip() != str(expected.get("expected_meta", "")).strip():
                return fail("E_V25_PATH_META", case_id)
            if not str(row.get("meta_out", "")).strip().startswith("out/"):
                return fail("E_V25_PATH_META_OUT", case_id)
        if int(row.get("exit_code", -1)) != 0:
            return fail("E_V25_PATH_EXIT", case_id)

    print("[v25-first-ai-production-path-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(golden_index)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
