#!/usr/bin/env python3
"""Fail-closed source checker for FD01 readiness audit I1."""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "work" / "codex" / "20260827_ddn_v1_fd01_frontdoor_runtimebridge_readiness_audit_i1"


def text(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def load(name: str) -> dict:
    return json.loads((OUT / name).read_text(encoding="utf-8"))


def main() -> None:
    inventory = load("fd01_surface_inventory.detjson")
    dag = load("fd01_successor_dag.detjson")
    ledger = load("fd01_test_ledger.detjson")
    checks: list[str] = []

    def check(value: bool, message: str) -> None:
        if not value:
            raise SystemExit(f"FAIL: {message}")
        checks.append(message)

    lang = text("lang/src/lib.rs")
    runtime = text("tool/src/ddn_runtime.rs")
    surface = text("tool/src/runtime_surface.rs")
    analysis = text("tool/src/source_analysis.rs")
    wasm = text("tool/src/wasm_api.rs")
    bridge = text("solutions/seamgrim_ui_mvp/ui/runtime/runtime_bridge.js")
    workbench = text("solutions/seamgrim_ui_mvp/ui/workbench/workbench.js")
    vm = text("solutions/seamgrim_ui_mvp/ui/runtime/wasm_vm_runtime.js")
    index = text("solutions/seamgrim_ui_mvp/ui/runtime/index.js")

    check("pub fn parse_v25_with_mode" in lang, "typed V25 parser exists")
    check("pub fn from_v25_source_with_mode" in runtime, "typed V25 program constructor exists")
    check("Self::from_source_epoch_with_mode(source, file_path, mode, false)" in runtime, "default DdnProgram remains pre-V25")
    check("DdnProgram::from_source_with_mode(source, file_path, ParseMode::Strict)" in surface, "run-summary surface remains pre-V25")
    check("DdnProgram::from_source_with_mode(&request.source_text" in analysis, "source analysis remains pre-V25")
    check("DdnProgram::from_source_with_mode(source, source_path, mode)" in wasm, "default WASM parser remains pre-V25")
    check("wasm_source_analysis_snapshot_json" in bridge, "RuntimeBridge routes source analysis to WASM")
    check("E_SOURCE_ANALYSIS_WASM_REQUIRED" in bridge, "missing Rust source analysis fails closed")
    check("await runtimeBridge.sourceAnalysis" in workbench, "Workbench awaits authoritative source analysis")
    check("projectSourceAnalysisUnavailable" in workbench, "Workbench records unavailable analysis")
    check("inspectNativeObservationManifest" in vm, "WASM VM consumes native observation manifest")
    check('from "./observation_manifest.js"' not in vm, "WASM VM does not import JS manifest builder")
    check('from "./observation_manifest.js"' in index, "dead builder re-export is explicitly inventoried")
    check(inventory["current_v25_public_product_ingress_count"] == 0, "V25 public product ingress count remains zero")
    check(inventory["historical_pc02"]["blind_reapply_authorized"] is False, "historical PC02 blind reapply is forbidden")
    check(len(inventory["surfaces"]) == 11, "eleven readiness surfaces are classified")
    lesson = inventory["lesson_epoch_inventory"]
    check(lesson["total"] == 557 and lesson["v25_pass"] == 0, "all 557 lessons have zero V25 passes")
    check(lesson["pre_v25_only"] + lesson["neither"] == 557, "lesson epoch classes cover the corpus")
    check(lesson["global_default_switch_safe"] is False, "global V25 default switch is unsafe")
    check(len(ledger["runs"]) == 9, "nine live/static audit runs are recorded")
    ids = [node["id"] for node in dag["nodes"]]
    check(len(ids) == len(set(ids)), "successor DAG node IDs are unique")
    known = set(ids)
    check(all(set(node["depends_on"]) <= known for node in dag["nodes"]), "successor dependencies exist")
    check("CONTENT_SNIFFING_SOURCE_EPOCH_AUTODETECT" in dag["forbidden_shortcuts"], "source-epoch autodetect is forbidden")
    check("GLOBAL_V25_DEFAULT_SWITCH_BEFORE_LESSON_MIGRATION" in dag["forbidden_shortcuts"], "premature global switch is forbidden")
    check(dag["nodes"][-1]["id"] == "F10" and dag["nodes"][-1]["depends_on"] == ["F9"], "plain PASS binding follows fresh review")

    print(json.dumps({"status": "PASS", "checks": len(checks), "surfaces": len(inventory["surfaces"])}, sort_keys=True))


if __name__ == "__main__":
    main()
