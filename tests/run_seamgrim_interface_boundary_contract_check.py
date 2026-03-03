#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

CONTRACT_SCHEMA = "ddn.seamgrim.interface_boundary_contract.v1"


def fail(msg: str) -> int:
    print(f"seamgrim interface boundary contract check failed: {msg}")
    return 1


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    contract_path = root / "tests" / "contracts" / "seamgrim_interface_boundary_contract.detjson"
    contract = load_json(contract_path)
    if contract is None:
        return fail(f"invalid contract json: {contract_path}")
    if str(contract.get("schema", "")).strip() != CONTRACT_SCHEMA:
        return fail(f"schema mismatch: {contract.get('schema')}")

    rows = contract.get("files")
    if not isinstance(rows, list) or not rows:
        return fail("files must be a non-empty list")

    missing: list[str] = []
    checked_files = 0
    for row in rows:
        if not isinstance(row, dict):
            missing.append("contract: non-object row")
            continue
        rel_path = str(row.get("path", "")).strip()
        if not rel_path:
            missing.append("contract: missing path")
            continue
        target = root / rel_path
        if not target.exists():
            missing.append(f"{rel_path}: file missing")
            continue
        try:
            text = target.read_text(encoding="utf-8")
        except Exception as exc:
            missing.append(f"{rel_path}: read failed: {exc}")
            continue
        required_tokens = row.get("required_tokens")
        if not isinstance(required_tokens, list):
            missing.append(f"{rel_path}: required_tokens must be list")
            continue
        checked_files += 1
        for token in required_tokens:
            token_text = str(token)
            if token_text not in text:
                missing.append(f"{rel_path}: missing token: {token_text}")

    if missing:
        print("seamgrim interface boundary contract check failed:")
        for item in missing[:16]:
            print(f" - {item}")
        return 1

    print(
        "seamgrim interface boundary contract check ok "
        f"(files={checked_files} contract={contract_path})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
