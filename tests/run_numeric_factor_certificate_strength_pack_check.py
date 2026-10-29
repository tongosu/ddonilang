#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKS = {
    "numeric_factor_certificate_route_v1": "ddn.numeric_factor_certificate_route.pack.contract.v1",
    "proof_numeric_factor_certificate_strength_v1": "ddn.proof_numeric_factor_certificate_strength.pack.contract.v1",
}


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def main() -> int:
    for name, schema in PACKS.items():
        pack = ROOT / "pack" / name
        required = [pack / "README.md", pack / "contract.detjson", pack / "golden.jsonl"]
        missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
        if missing:
            return fail("E_NUMERIC_CERT_MISSING", f"{name}: {missing}")
        contract = json.loads((pack / "contract.detjson").read_text(encoding="utf-8"))
        if contract.get("schema") != schema:
            return fail("E_NUMERIC_CERT_SCHEMA", name)
    fixture = ROOT / "pack" / "proof_numeric_factor_certificate_strength_v1" / "factor_result.detjson"
    value = json.loads(fixture.read_text(encoding="utf-8"))
    cert = value.get("certificate", {})
    if not value.get("job_hash", "").startswith("sha256:") or not cert.get("prime_checks"):
        return fail("E_NUMERIC_CERT_STRENGTH", "missing job_hash or prime_checks")
    print("numeric factor certificate strength pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

