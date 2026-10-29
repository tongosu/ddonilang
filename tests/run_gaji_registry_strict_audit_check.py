#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


REQUIRED_TOKEN_MAP = {
    "tools/teul-cli/src/main.rs": [
        "GajiCommands::Install {",
        "GajiCommands::Update {",
        "GajiCommands::Vendor {",
        '#[arg(long = "verify-registry")]',
        '#[arg(long = "strict-registry")]',
        '#[arg(long = "registry-audit-log")]',
        '#[arg(long = "verify-registry-audit")]',
        '#[arg(long = "registry-audit-verify-out")]',
        '#[arg(long = "frozen-lockfile")]',
        '"--verify-registry".to_string()',
        '"--strict-registry".to_string()',
        '"--registry-audit-log".to_string()',
        '"--verify-registry-audit".to_string()',
        '"--registry-audit-verify-out".to_string()',
        '"--frozen-lockfile".to_string()',
        '"verify-registry requires index"',
        '"verify-registry-audit requires log"',
        '"registry_snapshot": {',
        '.get("registry_audit")',
    ],
    "tools/teul-cli/src/cli/gaji_registry.rs": [
        "pub const VERIFY_DUPLICATE_RESOLUTION_POLICY",
        '"ddn.registry.verify_report.v1"',
        '"ddn.registry.audit_verify_report.v1"',
        '"ddn.registry.verify_report_source_provenance.v1"',
        '"ddn.registry.audit_verify_report_source_provenance.v1"',
        '"source_hash"',
        '"source_provenance"',
        '"frozen-lockfile requires ddn.lock registry_snapshot(snapshot_id/index_root_hash)"',
        '"frozen-lockfile requires registry_snapshot(snapshot_id/index_root_hash)"',
        '"registry_snapshot.snapshot_id is missing"',
    ],
}


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    missing: list[str] = []

    for rel_path, tokens in REQUIRED_TOKEN_MAP.items():
        target = root / rel_path
        if not target.exists():
            print(f"missing target: {target}")
            return 1
        text = target.read_text(encoding="utf-8")
        for token in tokens:
            if token not in text:
                missing.append(f"{rel_path}::{token}")

    if missing:
        print("gaji registry strict/audit check failed:")
        for token in missing[:16]:
            print(f" - missing token: {token}")
        return 1

    print("gaji registry strict/audit check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
