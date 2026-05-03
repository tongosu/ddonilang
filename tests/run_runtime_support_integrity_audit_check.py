#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def fail(message: str) -> int:
    print(f"[runtime-support-integrity-audit] fail: {message}")
    return 1


def read(rel_path: str) -> str:
    return (ROOT / rel_path).read_text(encoding="utf-8")


def ensure_absent(rel_path: str, tokens: list[str]) -> list[str]:
    text = read(rel_path)
    return [f"{rel_path}::{token}" for token in tokens if token in text]


def main() -> int:
    missing: list[str] = []
    violations: list[str] = []

    required = {
        "tests/seamgrim_wasm_cli_runtime_parity_runner.mjs": [
            "closureClaim === \"yes\"",
            "forbids acceptance_only",
            "currentline_model",
        ],
        "tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py": [
            "zip_extracted_raw_currentline_strict_parity_runner",
            "\"closure_claim\": \"yes\"",
            "\"currentline_model\": {",
            "raw_text = body + \"\\n\"",
            "input_path.read_text(encoding=\"utf-8\") != raw_text",
        ],
        "tool/src/wasm_api.rs": [
            "ddonirang_lang::apply_currentline_cell(source, context_json.as_deref())",
            "self.update_logic(&result.project_source)?",
        ],
        "lang/src/ast.rs": [
            "Receive {",
            "Send {",
        ],
        "lang/src/parser.rs": [
            "try_parse_receive_stmt",
            "Stmt::Send {",
            "in_imja_seed_body",
        ],
        "tool/src/ddn_runtime.rs": [
            "dispatch_signal_send",
            "dispatch_signal_to_receiver",
            "imja_signal_send_dispatches_receive_handler",
        ],
        "tools/teul-cli/src/main.rs": [
            "#[command(name = \"currentline-run\")]",
            "ddonirang_lang::apply_currentline_cell(",
            "currentline.project_source.as_bytes()",
        ],
        "solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js": [
            "const currentlineContext = obj.currentline_context ?? nestedState.currentline_context ?? null;",
            "currentline_context: currentlineContext",
        ],
    }
    for rel_path, tokens in required.items():
        text = read(rel_path)
        for token in tokens:
            if token not in text:
                missing.append(f"{rel_path}::{token}")

    violations.extend(
        ensure_absent(
            "tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py",
            [
                "to_executable_source",
                "apply_cell",
                "lowered_blocks",
                "compiled_source",
                "class Session",
                "acceptance_only",
            ],
        )
    )
    for rel_path in [
        "tests/run_ddonirang_vol2_bundle_cli_wasm_parity_check.py",
        "tests/run_ddonirang_vol3_bundle_cli_wasm_parity_check.py",
        "tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py",
    ]:
        text = read(rel_path)
        if "\"closure_claim\": \"yes\"" in text and "acceptance_only" in text:
            violations.append(f"{rel_path}::closure_claim=yes with acceptance_only")

    for rel_path in [
        "tool/src/wasm_api.rs",
        "tools/teul-cli/src/main.rs",
        "tests/seamgrim_wasm_cli_runtime_parity_runner.mjs",
    ]:
        if "compiled_source" in read(rel_path):
            violations.append(f"{rel_path}::compiled_source escape hatch token")

    context_path = ROOT / "pack" / "ddonirang_vol4_currentline_context_v1" / "context.detjson"
    context = json.loads(context_path.read_text(encoding="utf-8"))
    if not isinstance(context.get("prelude_source"), str) or not context["prelude_source"].strip():
        violations.append("pack/ddonirang_vol4_currentline_context_v1/context.detjson::missing raw prelude_source")
    for forbidden_key in ["resources", "resource_snapshot", "compiled_source", "project_source"]:
        if forbidden_key in context:
            violations.append(
                f"pack/ddonirang_vol4_currentline_context_v1/context.detjson::{forbidden_key}"
            )

    if missing or violations:
        for item in missing[:20]:
            print(f"missing: {item}")
        for item in violations[:20]:
            print(f"violation: {item}")
        return fail(f"missing={len(missing)} violations={len(violations)}")

    print("[runtime-support-integrity-audit] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
