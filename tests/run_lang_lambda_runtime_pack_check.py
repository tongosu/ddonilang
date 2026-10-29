#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKS = {
    "lang_lambda_capture_runtime_v1": "ddn.lang_lambda_capture_runtime.pack.contract.v1",
    "lang_lambda_return_runtime_v1": "ddn.lang_lambda_return_runtime.pack.contract.v1",
    "lang_lambda_store_runtime_v1": "ddn.lang_lambda_store_runtime.pack.contract.v1",
}


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def main() -> int:
    for name, schema in PACKS.items():
        pack = ROOT / "pack" / name
        required = [pack / "README.md", pack / "contract.detjson", pack / "input.ddn", pack / "golden.jsonl"]
        missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
        if missing:
            return fail("E_LAMBDA_PACK_MISSING", f"{name}: {missing}")
        contract = json.loads((pack / "contract.detjson").read_text(encoding="utf-8"))
        if contract.get("schema") != schema or contract.get("evidence_tier") != "runner_fill":
            return fail("E_LAMBDA_CONTRACT", name)
        rows = [
            json.loads(line)
            for line in (pack / "golden.jsonl").read_text(encoding="utf-8").splitlines()
            if line.strip()
        ]
        if len(rows) != 1 or rows[0].get("cmd", [None])[0] != "run":
            return fail("E_LAMBDA_GOLDEN", name)
    print("lambda runtime pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

