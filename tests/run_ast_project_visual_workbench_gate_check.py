#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "ast_project_visual_workbench_gate_v1"
WORKBENCH_RUNNER = ROOT / "tests" / "seamgrim_multifile_project_workbench_runner.mjs"
GRAPH_RUNNER = ROOT / "tests" / "ddn_local_project_graph_runner.mjs"
NORMAL_FIXTURE = ROOT / "pack" / "ddn_local_project_graph_v1" / "fixtures" / "project_run_alias_normal_v1.json"
SPACING_FIXTURE = ROOT / "pack" / "ddn_local_project_graph_v1" / "fixtures" / "project_run_alias_spacing_v1.json"
REQUIRED_VIEWPORTS = {"1440x900", "1024x768", "778x912", "390x844"}


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def run(cmd: list[str], *, timeout: int) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def parse_first_json(stdout: str) -> dict:
    start = stdout.find("{")
    end = stdout.rfind("}")
    if start < 0 or end < start:
        raise ValueError(stdout.strip())
    return json.loads(stdout[start:end + 1])


def require_files() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "golden.jsonl",
        PACK / "input.ddn",
        GRAPH_RUNNER,
        WORKBENCH_RUNNER,
        NORMAL_FIXTURE,
        SPACING_FIXTURE,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    return fail("E_AST_PROJECT_VISUAL_GATE_MISSING", repr(missing)) if missing else 0


def check_contract() -> int:
    doc = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "ast_project_visual_workbench_gate_v1",
        "kind": "ast_project_visual_workbench_integrated_gate",
        "gate": "G14",
        "runtime_state_hash_formula_change": False,
        "parser_grammar_change": False,
        "docs_ssot_change": False,
        "product_behavior_claim": True,
        "checker": "tests/run_ast_project_visual_workbench_gate_check.py",
    }
    for key, value in expected.items():
        if doc.get(key) != value:
            return fail("E_AST_PROJECT_VISUAL_GATE_CONTRACT", f"{key}={doc.get(key)!r}")
    required = {
        "normal_vs_whitespace_alias_raw_wasm_project_summary",
        "linked_source_hash_equality",
        "runtime_state_hash_equality",
        "project_observation_trace_equality",
        "source_graph_hash_negative_control",
        "workbench_imported_canvas_differs_from_static_preview",
        "workbench_module_value_change_updates_trace",
        "workbench_module_value_change_updates_canvas_signature",
        "workbench_module_value_change_updates_canvas_pixels",
        "workbench_four_viewport_evidence",
    }
    found = set(doc.get("direct_evidence", []))
    if not required.issubset(found):
        return fail("E_AST_PROJECT_VISUAL_GATE_COVERS", repr(sorted(found)))
    return 0


def graph_summary(fixture: Path) -> dict:
    proc = run(["node", str(GRAPH_RUNNER.relative_to(ROOT)), "run", str(fixture.relative_to(ROOT)), "앱"], timeout=180)
    if proc.returncode != 0:
        raise RuntimeError(proc.stdout.strip())
    return parse_first_json(proc.stdout)


def check_ast_module_lowering() -> int:
    try:
        normal = graph_summary(NORMAL_FIXTURE)
        spacing = graph_summary(SPACING_FIXTURE)
    except Exception as exc:
        return fail("E_AST_PROJECT_VISUAL_GATE_GRAPH_RUN", str(exc))
    fields = [
        "output_rows",
        "runtime_output_log_texts",
        "runtime_final_row",
        "runtime_state_hash",
        "linked_execution_source_sha256",
        "linked_execution_canonical_sha256",
        "bounded_state_sha256",
        "symbol_boundary_sha256",
    ]
    for field in fields:
        if normal.get(field) != spacing.get(field):
            return fail("E_AST_PROJECT_VISUAL_GATE_ALIAS_PARITY", f"{field} mismatch")
    envelope_fields = ["canonical_ddn_sha256", "effective_ticks", "input_snapshot", "requested_ticks", "rng_seed"]
    for field in envelope_fields:
        if normal.get("runtime_execution_envelope", {}).get(field) != spacing.get("runtime_execution_envelope", {}).get(field):
            return fail("E_AST_PROJECT_VISUAL_GATE_ENVELOPE_PARITY", f"{field} mismatch")
    normal_trace = normal.get("project_observation_trace", {})
    spacing_trace = spacing.get("project_observation_trace", {})
    if normal_trace.get("trace_sha256") != spacing_trace.get("trace_sha256"):
        return fail("E_AST_PROJECT_VISUAL_GATE_TRACE_PARITY", "trace hash mismatch")
    if [row.get("value_text") for row in normal_trace.get("rows", [])] != ["7", "10"]:
        return fail("E_AST_PROJECT_VISUAL_GATE_TRACE_VALUES", repr(normal_trace.get("rows")))
    if any(row.get("state_hash_participation") is not False for row in normal_trace.get("rows", [])):
        return fail("E_AST_PROJECT_VISUAL_GATE_TRACE_STATE_HASH", repr(normal_trace.get("rows")))
    if normal.get("project_graph_sha256") == spacing.get("project_graph_sha256"):
        return fail("E_AST_PROJECT_VISUAL_GATE_GRAPH_NEGATIVE", "source graph hash should remain source-sensitive")
    linked = str(normal.get("linked_execution_source", ""))
    if "운동 ." in linked or "물리 ." in linked:
        return fail("E_AST_PROJECT_VISUAL_GATE_LINKED_SOURCE", linked)
    return 0


def check_workbench_visual_binding() -> int:
    proc = run(["node", str(WORKBENCH_RUNNER.relative_to(ROOT))], timeout=900)
    if proc.returncode != 0:
        return fail("E_AST_PROJECT_VISUAL_GATE_WORKBENCH_RUN", proc.stdout.strip())
    try:
        result = parse_first_json(proc.stdout)
    except Exception as exc:
        return fail("E_AST_PROJECT_VISUAL_GATE_WORKBENCH_JSON", str(exc))
    if result.get("schema") != "seamgrim.multifile_project_workbench.runner_result.v1":
        return fail("E_AST_PROJECT_VISUAL_GATE_WORKBENCH_SCHEMA", repr(result))
    if set(result.get("viewports", [])) != REQUIRED_VIEWPORTS:
        return fail("E_AST_PROJECT_VISUAL_GATE_WORKBENCH_VIEWPORTS", repr(result.get("viewports")))
    if result.get("physics_output") != ["7", "10"] or result.get("grid_output") != ["3"]:
        return fail("E_AST_PROJECT_VISUAL_GATE_WORKBENCH_OUTPUTS", repr(result))
    if result.get("physics_graph") == result.get("grid_graph"):
        return fail("E_AST_PROJECT_VISUAL_GATE_WORKBENCH_GRAPH_DELTA", repr(result))
    evidence_path = Path(str(result.get("evidence_report", "")))
    if not evidence_path.exists():
        return fail("E_AST_PROJECT_VISUAL_GATE_EVIDENCE_MISSING", str(evidence_path))
    evidence = json.loads(evidence_path.read_text(encoding="utf-8"))
    rows = evidence.get("viewports")
    if not isinstance(rows, list) or {row.get("viewport") for row in rows} != REQUIRED_VIEWPORTS:
        return fail("E_AST_PROJECT_VISUAL_GATE_EVIDENCE_VIEWPORTS", repr(rows))
    for row in rows:
        label = str(row.get("viewport"))
        workbench = row.get("workbench") if isinstance(row.get("workbench"), dict) else {}
        main = row.get("main") if isinstance(row.get("main"), dict) else {}
        opened = workbench.get("opened") if isinstance(workbench.get("opened"), dict) else {}
        variant = workbench.get("variant") if isinstance(workbench.get("variant"), dict) else {}
        if main.get("project_mode") != "1" or main.get("keyboard_focus") != "ddn-textarea":
            return fail("E_AST_PROJECT_VISUAL_GATE_MAIN_EDITOR", f"{label}: {main!r}")
        if float(opened.get("canvas_height") or 0) < 160:
            return fail("E_AST_PROJECT_VISUAL_GATE_CANVAS_HEIGHT", f"{label}: {opened!r}")
        if int(opened.get("overflow_x") or 0) > 8:
            return fail("E_AST_PROJECT_VISUAL_GATE_OVERFLOW", f"{label}: {opened!r}")
        if opened.get("source_readonly") is not False or opened.get("keyboard_focus") != "source-editor":
            return fail("E_AST_PROJECT_VISUAL_GATE_EDITOR_ACCESS", f"{label}: {opened!r}")
        semantic_rows = opened.get("semantic_rows", [])
        semantic_values = [str(item.get("value")) for item in semantic_rows if isinstance(item, dict)]
        semantic_state_flags = [str(item.get("stateHashParticipation")) for item in semantic_rows if isinstance(item, dict)]
        if semantic_values != ["7", "10"] or semantic_state_flags != ["0", "0"]:
            return fail("E_AST_PROJECT_VISUAL_GATE_SEMANTIC_ROWS", f"{label}: {semantic_rows!r}")
        for key in ("project_run_trace_sha256", "project_run_bounded_state_sha256", "project_run_lineage_hash", "canvas_signature", "canvas_pixel_hash"):
            if not str(opened.get(key) or "").strip():
                return fail("E_AST_PROJECT_VISUAL_GATE_OPENED_FIELD", f"{label}: missing {key}")
        if opened.get("project_run_trace_sha256") == variant.get("trace_sha256"):
            return fail("E_AST_PROJECT_VISUAL_GATE_TRACE_DELTA", label)
        if opened.get("project_run_bounded_state_sha256") == variant.get("bounded_state_sha256"):
            return fail("E_AST_PROJECT_VISUAL_GATE_STATE_DELTA", label)
        if opened.get("project_run_lineage_hash") == variant.get("lineage_hash"):
            return fail("E_AST_PROJECT_VISUAL_GATE_LINEAGE_DELTA", label)
        if opened.get("canvas_signature") == variant.get("canvas_signature"):
            return fail("E_AST_PROJECT_VISUAL_GATE_CANVAS_SIGNATURE_DELTA", label)
        if opened.get("canvas_pixel_hash") == variant.get("canvas_pixel_hash"):
            return fail("E_AST_PROJECT_VISUAL_GATE_CANVAS_PIXEL_DELTA", label)
        if variant.get("canvas_signature") != variant.get("view_artifact_hash"):
            return fail("E_AST_PROJECT_VISUAL_GATE_VIEW_HASH_BOUNDARY", f"{label}: {variant!r}")
        for shot in [main.get("screenshot"), *workbench.get("screenshots", [])]:
            if not Path(str(shot)).exists():
                return fail("E_AST_PROJECT_VISUAL_GATE_SCREENSHOT_MISSING", f"{label}: {shot}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_AST_PROJECT_VISUAL_GATE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_AST_PROJECT_VISUAL_GATE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (require_files, check_contract):
        rc = check()
        if rc:
            return rc
    gates = [
        (["python", "tests/run_pack_golden.py", "ast_project_visual_workbench_gate_v1"], "E_AST_PROJECT_VISUAL_GATE_GOLDEN", 180),
    ]
    for cmd, code, timeout in gates:
        proc = run(cmd, timeout=timeout)
        if proc.returncode != 0:
            return fail(code, proc.stdout.strip())
    for check in (check_ast_module_lowering, check_workbench_visual_binding, check_docs_ssot_clean):
        rc = check()
        if rc:
            return rc
    print("[ast-project-visual-workbench-gate-v1] OK G14 integrated gate")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
