#!/usr/bin/env python
from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "toolchain_pack_0_v1"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"


def main() -> int:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    missing = [rel for rel in contract["required_paths"] if not (ROOT / rel).exists()]
    manifest_text = MANIFEST.read_text(encoding="utf-8")
    missing_tokens = [token for token in contract["required_manifest_tokens"] if token not in manifest_text]
    report = {
        "schema": "ddn.roadmap_v2.toolchain_pack_0.report.v1",
        "ok": not missing and not missing_tokens,
        "required_paths_present": not missing,
        "manifest_tokens_present": not missing_tokens,
    }
    expected = json.loads((PACK / "expected" / "pack_skeleton.detjson").read_text(encoding="utf-8"))
    if report != expected:
        print(
            f"[roadmap-v2-pack-skeleton] fail: missing_paths={missing} missing_tokens={missing_tokens}",
            file=sys.stderr,
        )
        return 1
    print("[roadmap-v2-pack-skeleton] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
