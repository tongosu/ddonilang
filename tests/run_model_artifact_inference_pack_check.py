#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


PACK_SCHEMA = "ddn.model_artifact_inference.pack.contract.v1"
EXPECTED_CMD = [
    "infer",
    "mlp",
    "pack/gogae8_w85_ondevice_infer_v1/model.detjson",
    "pack/gogae8_w85_ondevice_infer_v1/input.detjson",
    "--out",
    "build/tmp/model_artifact_infer_minimum",
]
EXPECTED_STDOUT = [
    "{\"schema\":\"seulgi.infer_output.v1\",\"model_hash\":\"blake3:1391c84a4d985d4f30cfd3b34d7042b7c51e31e82b2f2f5944c822ad9d71e8bd\",\"output\":[-9]}"
]


def fail(code: str, msg: str) -> int:
    print(f"[model-artifact-inference-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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
    parser = argparse.ArgumentParser(description="Model artifact inference pack checker")
    parser.add_argument("--pack", default="pack/model_artifact_inference_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required = [
        pack / "README.md",
        pack / "intent.md",
        pack / "contract.detjson",
        pack / "golden.jsonl",
        pack / "cases" / "c01_infer_only_mlp" / "expected.json",
        pack / "tests" / "README.md",
    ]
    missing = [str(path).replace("\\", "/") for path in required if not path.exists()]
    if missing:
        return fail("E_MODEL_ARTIFACT_INFER_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
        golden = load_jsonl(pack / "golden.jsonl")
        expected = load_json(pack / "cases" / "c01_infer_only_mlp" / "expected.json")
    except ValueError as exc:
        return fail("E_MODEL_ARTIFACT_INFER_INVALID", str(exc))

    if not isinstance(contract, dict):
        return fail("E_MODEL_ARTIFACT_INFER_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_MODEL_ARTIFACT_INFER_SCHEMA", f"schema={contract.get('schema')}")
    if str(contract.get("evidence_tier", "")).strip() != "runner_fill":
        return fail("E_MODEL_ARTIFACT_INFER_TIER", f"tier={contract.get('evidence_tier')}")
    if str(contract.get("closure_claim", "")).strip() != "no":
        return fail("E_MODEL_ARTIFACT_INFER_CLOSURE", f"closure={contract.get('closure_claim')}")
    for rel in contract.get("evidence_targets", []):
        if not Path(rel).exists():
            return fail("E_MODEL_ARTIFACT_INFER_TARGET", rel)
    if len(golden) != 1 or str(golden[0].get("id", "")).strip() != "c01_infer_only_mlp":
        return fail("E_MODEL_ARTIFACT_INFER_GOLDEN_SET", str(golden))
    row = golden[0]
    if row.get("cmd") != EXPECTED_CMD:
        return fail("E_MODEL_ARTIFACT_INFER_CMD", str(row.get("cmd")))
    if row.get("stdout") != EXPECTED_STDOUT or expected.get("stdout") != EXPECTED_STDOUT:
        return fail("E_MODEL_ARTIFACT_INFER_STDOUT", "stdout mismatch")
    if int(row.get("exit_code", -1)) != 0:
        return fail("E_MODEL_ARTIFACT_INFER_EXIT", str(row.get("exit_code")))

    print("[model-artifact-inference-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print("cases=1")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
