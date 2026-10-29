#!/usr/bin/env python
from __future__ import annotations

import json
import hashlib
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "ddn_local_project_graph_v1"
BASELINE = PACK / "baseline.detjson"
GOLDEN = PACK / "golden.jsonl"
PROJECT_MANIFEST = ROOT / "tool" / "Cargo.toml"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def run(cmd: list[str], timeout: int = 60) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
    )


def run_with_cwd(cmd: list[str], cwd: Path, timeout: int = 60) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
    )


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def load_last_json_object(text: str) -> dict:
    for line in reversed(text.splitlines()):
        stripped = line.strip()
        if stripped.startswith("{") and stripped.endswith("}"):
            return json.loads(stripped)
    raise json.JSONDecodeError("no JSON object line found", text, 0)


def check_files() -> int:
    for path in [
        PACK / "README.md",
        BASELINE,
        GOLDEN,
        PACK / "input.ddn",
        PACK / "expected" / "stdout.txt",
        PACK / "expected" / "root_discovery_v1.detjson",
        PACK / "expected" / "projectjson_fallback_v1.detjson",
        PACK / "expected" / "project_graph_v1.detjson",
        PACK / "expected" / "project_symbols_v1.detjson",
        PACK / "expected" / "project_module_reexport_v1.detjson",
        PACK / "expected" / "project_gaji_lock_v1.detjson",
        PACK / "expected" / "project_run_summary_v1.detjson",
        PACK / "expected" / "project_run_grid_summary_v1.detjson",
        PACK / "expected" / "project_run_alias_normal_v1.detjson",
        PACK / "expected" / "project_run_alias_spacing_v1.detjson",
        PACK / "fixtures" / "project_graph_v1.json",
        PACK / "fixtures" / "project_block_outside_root_v1.json",
        PACK / "fixtures" / "project_symbol_private_v1.json",
        PACK / "fixtures" / "project_module_reexport_v1.json",
        PACK / "fixtures" / "project_module_reexport_shuffle_v1.json",
        PACK / "fixtures" / "project_module_cycle_v1.json",
        PACK / "fixtures" / "project_module_top_level_effect_v1.json",
        PACK / "fixtures" / "project_module_duplicate_import_v1.json",
        PACK / "fixtures" / "project_module_duplicate_export_v1.json",
        PACK / "fixtures" / "project_gaji_lock_generate_v1.json",
        PACK / "fixtures" / "project_gaji_lock_frozen_v1.json",
        PACK / "fixtures" / "project_gaji_lock_missing_v1.json",
        PACK / "fixtures" / "project_gaji_lock_tampered_v1.json",
        PACK / "fixtures" / "project_gaji_lock_unpinned_v1.json",
        PACK / "fixtures" / "project_gaji_lock_transitive_missing_v1.json",
        PACK / "fixtures" / "project_gaji_lock_id_path_mismatch_v1.json",
        PACK / "fixtures" / "project_run_physics_v1.json",
        PACK / "fixtures" / "project_run_grid_v1.json",
        PACK / "fixtures" / "project_run_alias_normal_v1.json",
        PACK / "fixtures" / "project_run_alias_spacing_v1.json",
        PACK / "project_frontdoor" / "project_run_v1" / "뿌리.ddn",
        PACK / "project_frontdoor" / "project_run_v1" / "ddn.lock",
        PACK / "project_frontdoor" / "project_run_v1" / "app" / "main.ddn",
        PACK / "project_frontdoor" / "project_run_v1" / "lib" / "운동.ddn",
        PACK / "project_frontdoor" / "project_run_v1" / "vendor" / "edu.physics" / "gaji.toml",
        PACK / "project_frontdoor" / "project_run_v1" / "vendor" / "edu.physics" / "ddn" / "constants.ddn",
        PACK / "project_frontdoor" / "project_run_v1" / "vendor" / "edu.units" / "gaji.toml",
        ROOT / "tests" / "ddn_local_project_graph_runner.mjs",
    ]:
        if not path.exists():
            return fail("E_DDN_LOCAL_PROJECT_GRAPH_MISSING", str(path.relative_to(ROOT)))
    for case_id in [
        "project_root_discovery_v1",
        "project_projectjson_fallback_v1",
        "project_root_conflict_fail_v1",
        "project_root_ambiguous_fail_v1",
        "project_block_vs_projectjson_conflict_v1",
    ]:
        if not (PACK / "root_discovery" / case_id).exists():
            return fail("E_DDN_LOCAL_PROJECT_GRAPH_ROOT_CASE", case_id)
    return 0


def check_baseline() -> int:
    row = load_json(BASELINE)
    expected = {
        "schema": "ddn.local_project_graph.baseline.v1",
        "pack": "ddn_local_project_graph_v1",
        "gate": "G0",
        "branch": "codex/queue-20260706",
        "start_head": "12dbffc91bf8356b373b6c1a952591e0a6c02d16",
        "minimum_common_ancestor_verified": True,
        "docs_ssot_dirty": False,
        "target_overlap_at_g0": False,
    }
    for key, value in expected.items():
        if row.get(key) != value:
            return fail("E_DDN_LOCAL_PROJECT_GRAPH_BASELINE", f"{key}={row.get(key)!r}")

    matrix = row.get("capability_matrix")
    if not isinstance(matrix, list) or len(matrix) < 7:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_MATRIX", "capability matrix too small")
    surfaces = {entry.get("surface") for entry in matrix if isinstance(entry, dict)}
    for surface in [
        "뿌리.ddn root discovery",
        "프로젝트{} root block",
        "쓰임{} direct import",
        "드러냄{} direct export",
        "local gaji lock bridge",
        "CLI/WASM multi-file equality",
        "one-file project and portable artifact",
    ]:
        if surface not in surfaces:
            return fail("E_DDN_LOCAL_PROJECT_GRAPH_SURFACE", surface)

    refs = row.get("product_evidence_refs")
    if not isinstance(refs, list) or len(refs) < 8:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_REFS", "product refs too small")
    for ref in refs:
        if not str(ref.get("file", "")).strip():
            return fail("E_DDN_LOCAL_PROJECT_GRAPH_REF_PATH", repr(ref))
        line = int(ref["line"])
        if line <= 0:
            return fail("E_DDN_LOCAL_PROJECT_GRAPH_REF_LINE", repr(ref))

    non_claims = row.get("non_claims")
    required_false = [
        "LOCAL_PROJECT_GRAPH_V1_CLOSED",
        "LOCAL_GAJI_LOCK_BRIDGE_V1_CLOSED",
        "MULTIFILE_CLI_WASM_PARITY_V1_CLOSED",
        "SEAMGRIM_MULTIFILE_AUTHORING_V1_CLOSED",
        "state_hash_formula_change",
        "docs_ssot_modified",
    ]
    for key in required_false:
        if non_claims.get(key) is not False:
            return fail("E_DDN_LOCAL_PROJECT_GRAPH_NON_CLAIM", key)
    return 0


def check_golden() -> int:
    rows = [json.loads(line) for line in GOLDEN.read_text(encoding="utf-8").splitlines() if line.strip()]
    by_id = {row.get("case_id"): row for row in rows}
    required = {
        "G0_baseline_smoke",
        "project_root_discovery_v1",
        "project_projectjson_fallback_v1",
        "project_root_conflict_fail_v1",
        "project_root_ambiguous_fail_v1",
        "project_block_vs_projectjson_conflict_v1",
        "project_graph_v1",
        "project_block_outside_root_v1",
        "project_symbols_v1",
        "project_symbol_private_v1",
        "project_module_reexport_v1",
        "project_module_cycle_v1",
        "project_module_top_level_effect_v1",
        "project_module_duplicate_import_v1",
        "project_module_duplicate_export_v1",
        "project_gaji_lock_generate_v1",
        "project_gaji_lock_frozen_v1",
        "project_gaji_lock_missing_v1",
        "project_gaji_lock_tampered_v1",
        "project_gaji_lock_unpinned_v1",
        "project_gaji_lock_transitive_missing_v1",
        "project_gaji_lock_id_path_mismatch_v1",
        "project_run_frontdoor_v1",
        "project_run_virtual_physics_v1",
        "project_run_virtual_grid_v1",
        "project_run_alias_normal_v1",
        "project_run_alias_spacing_v1",
    }
    missing = sorted(required - set(by_id))
    if missing:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_GOLDEN_CASES", repr(missing))
    row = by_id["G0_baseline_smoke"]
    if row.get("cmd") != ["run", "pack/ddn_local_project_graph_v1/input.ddn", "--madi", "1"]:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_GOLDEN_CMD", repr(row.get("cmd")))
    if row.get("stdout_path") != "expected/stdout.txt":
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_GOLDEN_STDOUT", repr(row))
    stdout = (PACK / "expected" / "stdout.txt").read_text(encoding="utf-8")
    if stdout.strip() != "DDN_LOCAL_PROJECT_GRAPH_G0_BASELINE":
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_GOLDEN_STDOUT_CONTENT", stdout)
    if by_id["project_root_discovery_v1"].get("stdout_path") != "expected/root_discovery_v1.detjson":
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_ROOT_STDOUT", repr(by_id["project_root_discovery_v1"]))
    if by_id["project_projectjson_fallback_v1"].get("stdout_path") != "expected/projectjson_fallback_v1.detjson":
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_FALLBACK_STDOUT", repr(by_id["project_projectjson_fallback_v1"]))
    expected_codes = {
        "project_root_conflict_fail_v1": "E_LEGACY_PROJECT_ROOT_NAME",
        "project_root_ambiguous_fail_v1": "E_PROJECT_ROOT_AMBIGUOUS",
        "project_block_vs_projectjson_conflict_v1": "E_PROJECT_BLOCK_VS_PROJECTJSON_CONFLICT",
    }
    for case_id, code in expected_codes.items():
        row = by_id[case_id]
        if row.get("expected_error_code") != code or row.get("exit_code") != 2:
            return fail("E_DDN_LOCAL_PROJECT_GRAPH_ERROR_ROW", repr(row))
    if by_id["project_graph_v1"].get("stdout_path") != "expected/project_graph_v1.detjson":
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_PROJECT_GRAPH_ROW", repr(by_id["project_graph_v1"]))
    outside_row = by_id["project_block_outside_root_v1"]
    if outside_row.get("expected_error_code") != "E_PROJECT_BLOCK_OUTSIDE_ROOT" or outside_row.get("exit_code") != 2:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_OUTSIDE_ROOT_ROW", repr(outside_row))
    if by_id["project_symbols_v1"].get("stdout_path") != "expected/project_symbols_v1.detjson":
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_SYMBOL_ROW", repr(by_id["project_symbols_v1"]))
    private_row = by_id["project_symbol_private_v1"]
    if private_row.get("expected_error_code") != "E_SYMBOL_NOT_EXPORTED" or private_row.get("exit_code") != 2:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_PRIVATE_ROW", repr(private_row))
    if by_id["project_module_reexport_v1"].get("stdout_path") != "expected/project_module_reexport_v1.detjson":
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_REEXPORT_ROW", repr(by_id["project_module_reexport_v1"]))
    g5_codes = {
        "project_module_cycle_v1": "E_IMPORT_CYCLE",
        "project_module_top_level_effect_v1": "E_IMPORTED_TOP_LEVEL_EFFECT_UNSUPPORTED",
        "project_module_duplicate_import_v1": "E_IMPORT_ALIAS_DUPLICATE",
        "project_module_duplicate_export_v1": "E_EXPORT_DUPLICATE",
    }
    for case_id, code in g5_codes.items():
        row = by_id[case_id]
        if row.get("expected_error_code") != code or row.get("exit_code") != 2:
            return fail("E_DDN_LOCAL_PROJECT_GRAPH_G5_ERROR_ROW", repr(row))
    if by_id["project_gaji_lock_generate_v1"].get("stdout_path") != "expected/project_gaji_lock_v1.detjson":
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_LOCK_GENERATE_ROW", repr(by_id["project_gaji_lock_generate_v1"]))
    if by_id["project_gaji_lock_frozen_v1"].get("stdout_path") != "expected/project_gaji_lock_v1.detjson":
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_LOCK_FROZEN_ROW", repr(by_id["project_gaji_lock_frozen_v1"]))
    g6_codes = {
        "project_gaji_lock_missing_v1": "E_LOCK_MISSING",
        "project_gaji_lock_tampered_v1": "E_LOCK_HASH_MISMATCH",
        "project_gaji_lock_unpinned_v1": "E_GAJI_DEP_UNPINNED",
        "project_gaji_lock_transitive_missing_v1": "E_GAJI_TRANSITIVE_MISSING",
        "project_gaji_lock_id_path_mismatch_v1": "E_GAJI_ID_PATH_MISMATCH",
    }
    for case_id, code in g6_codes.items():
        row = by_id[case_id]
        if row.get("expected_error_code") != code or row.get("exit_code") != 2:
            return fail("E_DDN_LOCAL_PROJECT_GRAPH_G6_ERROR_ROW", repr(row))
    if by_id["project_run_frontdoor_v1"].get("stdout_path") != "expected/project_run_summary_v1.detjson":
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_PROJECT_RUN_ROW", repr(by_id["project_run_frontdoor_v1"]))
    if by_id["project_run_virtual_physics_v1"].get("stdout_path") != "expected/project_run_summary_v1.detjson":
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_PROJECT_RUN_PHYSICS_ROW", repr(by_id["project_run_virtual_physics_v1"]))
    if by_id["project_run_virtual_grid_v1"].get("stdout_path") != "expected/project_run_grid_summary_v1.detjson":
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_PROJECT_RUN_GRID_ROW", repr(by_id["project_run_virtual_grid_v1"]))
    alias_rows = {
        "project_run_alias_normal_v1": "expected/project_run_alias_normal_v1.detjson",
        "project_run_alias_spacing_v1": "expected/project_run_alias_spacing_v1.detjson",
    }
    for case_id, stdout_path in alias_rows.items():
        row = by_id[case_id]
        if row.get("stdout_path") != stdout_path or row.get("exit_code") != 0:
            return fail("E_DDN_LOCAL_PROJECT_GRAPH_ALIAS_ROW", repr(row))
    return 0


def project_cli(args: list[str], timeout: int = 120) -> subprocess.CompletedProcess[str]:
    return run(
        ["cargo", "run", "-q", "--manifest-path", str(PROJECT_MANIFEST), "--bin", "ddonirang", "--", *args],
        timeout=timeout,
    )


def payload_replace_file_text(payload: dict, path: str, old: str, new: str) -> None:
    for row in payload.get("files", []):
        if isinstance(row, dict) and row.get("path") == path:
            text = row.get("utf8_text")
            if not isinstance(text, str) or old not in text:
                raise AssertionError(f"{path}: replacement source not found")
            row["utf8_text"] = text.replace(old, new, 1)
            return
    raise AssertionError(f"{path}: file not found")


def run_virtual_project_payload(payload: dict, target: str = "앱") -> tuple[dict, str, str]:
    with tempfile.TemporaryDirectory(prefix="ddn-runtime-parity-") as tmp:
        fixture = Path(tmp) / "project.json"
        fixture.write_text(json.dumps(payload, ensure_ascii=False, sort_keys=True), encoding="utf-8")
        cli = project_cli(
            ["project-run", "--virtual", str(fixture), "--target", target, "--inspect-graph", "--inspect-lock"],
            timeout=180,
        )
        if cli.returncode != 0:
            raise RuntimeError(cli.stderr.strip() or cli.stdout.strip())
        wasm = run(["node", "tests/ddn_local_project_graph_runner.mjs", "run", str(fixture), target], timeout=120)
        if wasm.returncode != 0:
            raise RuntimeError(wasm.stderr.strip() or wasm.stdout.strip())
        if cli.stdout != wasm.stdout:
            raise RuntimeError("CLI/WASM output bytes differ")
        return json.loads(cli.stdout), cli.stdout, wasm.stdout


def run_virtual_project_error(payload: dict, target: str) -> tuple[str, str]:
    with tempfile.TemporaryDirectory(prefix="ddn-runtime-parity-error-") as tmp:
        fixture = Path(tmp) / "project.json"
        fixture.write_text(json.dumps(payload, ensure_ascii=False, sort_keys=True), encoding="utf-8")
        cli = project_cli(
            ["project-run", "--virtual", str(fixture), "--target", target, "--inspect-graph", "--inspect-lock"],
            timeout=180,
        )
        wasm = run(["node", "tests/ddn_local_project_graph_runner.mjs", "run", str(fixture), target], timeout=120)
    return f"{cli.stdout}\n{cli.stderr}", f"{wasm.stdout}\n{wasm.stderr}"


def check_official_run_matches_project_summary(doc: dict, label: str) -> int:
    linked_source = doc.get("linked_execution_source")
    if not isinstance(linked_source, str) or not linked_source.strip():
        return fail("E_DDN_LOCAL_PROJECT_RUN_LINKED_SOURCE", label)
    with tempfile.TemporaryDirectory(prefix="ddn-linked-run-") as tmp:
        tmp_dir = Path(tmp)
        source_path = tmp_dir / "linked.ddn"
        source_path.write_bytes(linked_source.encode("utf-8"))
        proc = project_cli(
            ["run", str(source_path), "--source-profile", "v1-core-v25", "--madi", "1"],
            timeout=180,
        )
        if proc.returncode != 0:
            return fail("E_DDN_LOCAL_PROJECT_RUN_OFFICIAL_CLI", f"{label}: {proc.stderr.strip() or proc.stdout.strip()}")
        official = json.loads(proc.stdout)
    if official.get("output_log_texts") != doc.get("runtime_output_log_texts"):
        return fail(
            "E_DDN_LOCAL_PROJECT_RUN_OFFICIAL_OUTPUT",
            f"{label}: {official.get('output_log_texts')!r} != {doc.get('runtime_output_log_texts')!r}",
        )
    if not str(official.get("state_hash", "")).strip():
        return fail("E_DDN_LOCAL_PROJECT_RUN_OFFICIAL_STATE", label)
    if not str(doc.get("runtime_state_hash", "")).strip():
        return fail("E_DDN_LOCAL_PROJECT_RUN_RUNTIME_STATE", label)
    if official.get("state_hash") != doc.get("runtime_state_hash"):
        return fail("E_DDN_LOCAL_PROJECT_RUN_SHARED_STATE_HASH", label)
    project_envelope = dict(doc.get("runtime_execution_envelope") or {})
    project_envelope.pop("domain_evidence_manifests", None)
    if official.get("execution_envelope") != project_envelope:
        return fail("E_DDN_LOCAL_PROJECT_RUN_SHARED_ENVELOPE", label)
    if official.get("canonical_ddn") != doc.get("linked_execution_canonical_ddn"):
        return fail("E_DDN_LOCAL_PROJECT_RUN_SHARED_CANONICAL", label)
    official_canonical = official.get("canonical_ddn")
    if not isinstance(official_canonical, str):
        return fail("E_DDN_LOCAL_PROJECT_RUN_OFFICIAL_CANON", label)
    official_canonical_sha = hashlib.sha256(official_canonical.encode("utf-8")).hexdigest()
    if not official_canonical_sha.strip():
        return fail("E_DDN_LOCAL_PROJECT_RUN_OFFICIAL_CANON", label)
    if official.get("effective_ticks") != doc.get("runtime_execution_envelope", {}).get("effective_ticks"):
        return fail("E_DDN_LOCAL_PROJECT_RUN_OFFICIAL_ENVELOPE", label)
    return 0


def check_execution_preparation(doc: dict, label: str) -> int:
    prep = doc.get("execution_preparation")
    if not isinstance(prep, dict):
        return fail("E_DDN_LOCAL_PROJECT_PREP_MISSING", label)
    expected_top = {
        "schema": "ddn.local_project.execution_preparation.v1",
        "root_path": doc.get("root_path"),
        "entry_path": doc.get("entry_path"),
        "target": doc.get("target"),
        "target_path": doc.get("target_path"),
        "file_count": doc.get("file_count"),
        "state_hash_formula_change": False,
    }
    for key, value in expected_top.items():
        if prep.get(key) != value:
            return fail("E_DDN_LOCAL_PROJECT_PREP_TOP", f"{label}: {key}={prep.get(key)!r}")
    graph = prep.get("graph")
    if not isinstance(graph, dict):
        return fail("E_DDN_LOCAL_PROJECT_PREP_GRAPH", label)
    if graph.get("project_graph_sha256") != doc.get("project_graph_sha256"):
        return fail("E_DDN_LOCAL_PROJECT_PREP_GRAPH_HASH", label)
    if not isinstance(graph.get("module_graph_order"), list) or not graph.get("module_graph_order"):
        return fail("E_DDN_LOCAL_PROJECT_PREP_MODULE_ORDER", repr(graph))
    if not str(graph.get("module_graph_sha256", "")).strip():
        return fail("E_DDN_LOCAL_PROJECT_PREP_MODULE_HASH", repr(graph))
    lock = prep.get("lock")
    if not isinstance(lock, dict):
        return fail("E_DDN_LOCAL_PROJECT_PREP_LOCK", label)
    if lock.get("lock_sha256") != doc.get("lock_sha256"):
        return fail("E_DDN_LOCAL_PROJECT_PREP_LOCK_HASH", label)
    if lock.get("inspected") != doc.get("lock_inspected"):
        return fail("E_DDN_LOCAL_PROJECT_PREP_LOCK_INSPECT", label)
    linked = prep.get("linked_source")
    if not isinstance(linked, dict):
        return fail("E_DDN_LOCAL_PROJECT_PREP_LINKED", label)
    if linked.get("source_sha256") != doc.get("linked_execution_source_sha256"):
        return fail("E_DDN_LOCAL_PROJECT_PREP_LINKED_SOURCE", label)
    if linked.get("canonical_sha256") != doc.get("linked_execution_canonical_sha256"):
        return fail("E_DDN_LOCAL_PROJECT_PREP_LINKED_CANON", label)
    request = prep.get("runtime_request")
    if not isinstance(request, dict) or request.get("requested_ticks") != 1:
        return fail("E_DDN_LOCAL_PROJECT_PREP_REQUEST", repr(request))
    limits = prep.get("limits")
    required_limits = {
        "max_project_files",
        "max_file_bytes",
        "max_serialized_artifact_bytes",
    }
    if not isinstance(limits, dict) or not required_limits.issubset(limits):
        return fail("E_DDN_LOCAL_PROJECT_PREP_LIMITS", repr(limits))
    provenance = prep.get("provenance")
    if not isinstance(provenance, dict):
        return fail("E_DDN_LOCAL_PROJECT_PREP_PROVENANCE", label)
    for key in ("imports", "exports", "resolved_symbols"):
        if not isinstance(provenance.get(key), list):
            return fail("E_DDN_LOCAL_PROJECT_PREP_PROVENANCE_FIELD", f"{label}: {key}")
    if prep.get("diagnostics") != []:
        return fail("E_DDN_LOCAL_PROJECT_PREP_DIAGS", repr(prep.get("diagnostics")))
    return 0


def check_project_observation_trace(doc: dict, label: str) -> int:
    trace = doc.get("project_observation_trace")
    if not isinstance(trace, dict):
        return fail("E_DDN_LOCAL_PROJECT_TRACE_MISSING", label)
    if trace.get("schema") != "ddn.public_observation_trace.v1":
        return fail("E_DDN_LOCAL_PROJECT_TRACE_SCHEMA", f"{label}: {trace.get('schema')!r}")
    samples = trace.get("samples")
    output_rows = doc.get("output_rows")
    if not isinstance(samples, list) or not isinstance(output_rows, list):
        return fail("E_DDN_LOCAL_PROJECT_TRACE_ROWS", label)
    if samples != output_rows:
        return fail("E_DDN_LOCAL_PROJECT_TRACE_ROW_COUNT", f"{label}: trace/output mismatch")
    trace_hash = doc.get("project_observation_trace_hash")
    if not str(trace_hash or "").startswith("sha256:"):
        return fail("E_DDN_LOCAL_PROJECT_TRACE_HASH", label)
    if not isinstance(trace.get("columns"), list):
        return fail("E_DDN_LOCAL_PROJECT_TRACE_COLUMNS", label)
    for index, row in enumerate(samples):
        if not isinstance(row, dict):
            return fail("E_DDN_LOCAL_PROJECT_TRACE_ROW_OBJECT", f"{label}: {index}")
        if row.get("state_hash") != doc.get("runtime_state_hash"):
            return fail("E_DDN_LOCAL_PROJECT_TRACE_ROW_STATE", f"{label}: {index}")
        if not isinstance(row.get("cells"), list):
            return fail("E_DDN_LOCAL_PROJECT_TRACE_ROW_CELLS", f"{label}: {index}")
    projection = doc.get("project_observation_table_projection")
    if not isinstance(projection, dict):
        return fail("E_DDN_LOCAL_PROJECT_TRACE_PROJECTION", label)
    if projection.get("source_schema") != trace.get("schema"):
        return fail("E_DDN_LOCAL_PROJECT_TRACE_PROJECTION_SCHEMA", label)
    if projection.get("source_trace_hash") != trace_hash:
        return fail("E_DDN_LOCAL_PROJECT_TRACE_PROJECTION_HASH", label)
    return 0


def virtual_request(files: list[tuple[str, str | None]], start_path: str = "app/main.ddn") -> dict:
    return {
        "schema": "ddn.project_root_discovery_request.v1",
        "start_path": start_path,
        "files": [
            {"path": path, **({"utf8_text": text} if text is not None else {})}
            for path, text in files
        ],
    }


def run_virtual_discover(payload: dict) -> subprocess.CompletedProcess[str]:
    tmp_dir = ROOT / "build" / "tmp" / "ddn_project_root_discovery"
    tmp_dir.mkdir(parents=True, exist_ok=True)
    path = tmp_dir / "virtual_request.json"
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    return project_cli(["project-discover", "--virtual", str(path)])


def check_root_discovery_product() -> int:
    root_case = PACK / "root_discovery" / "project_root_discovery_v1"
    disk = project_cli(["project-discover", str(root_case / "app" / "main.ddn")])
    if disk.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_ROOT_DISK", disk.stderr.strip() or disk.stdout.strip())
    virtual = run_virtual_discover(
        virtual_request(
            [
                ("뿌리.ddn", (root_case / "뿌리.ddn").read_text(encoding="utf-8")),
                ("ddn.project.json", (root_case / "ddn.project.json").read_text(encoding="utf-8")),
                ("app/main.ddn", None),
            ]
        )
    )
    if virtual.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_ROOT_VIRTUAL", virtual.stderr.strip() or virtual.stdout.strip())
    if disk.stdout != virtual.stdout:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_ROOT_EQUALITY", "disk and virtual root results differ")
    expected = (PACK / "expected" / "root_discovery_v1.detjson").read_text(encoding="utf-8")
    if disk.stdout != expected:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_ROOT_EXPECTED", disk.stdout)

    fallback_case = PACK / "root_discovery" / "project_projectjson_fallback_v1"
    disk_fallback = project_cli(["project-discover", str(fallback_case / "app" / "main.ddn")])
    if disk_fallback.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_FALLBACK_DISK", disk_fallback.stderr.strip())
    virtual_fallback = run_virtual_discover(
        virtual_request(
            [
                ("ddn.project.json", (fallback_case / "ddn.project.json").read_text(encoding="utf-8")),
                ("app/main.ddn", None),
            ]
        )
    )
    if virtual_fallback.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_FALLBACK_VIRTUAL", virtual_fallback.stderr.strip())
    if disk_fallback.stdout != virtual_fallback.stdout:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_FALLBACK_EQUALITY", "disk and virtual fallback results differ")
    expected_fallback = (PACK / "expected" / "projectjson_fallback_v1.detjson").read_text(encoding="utf-8")
    if disk_fallback.stdout != expected_fallback:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_FALLBACK_EXPECTED", disk_fallback.stdout)

    negative_cases = [
        (
            "project_root_conflict_fail_v1",
            [("프로젝트.ddn", None), ("app/main.ddn", None)],
            "E_LEGACY_PROJECT_ROOT_NAME",
        ),
        (
            "project_root_ambiguous_fail_v1",
            [("뿌리.ddn", None), ("프로젝트.ddn", None), ("app/main.ddn", None)],
            "E_PROJECT_ROOT_AMBIGUOUS",
        ),
        (
            "project_block_vs_projectjson_conflict_v1",
            [
                (
                    "뿌리.ddn",
                    (PACK / "root_discovery" / "project_block_vs_projectjson_conflict_v1" / "뿌리.ddn").read_text(
                        encoding="utf-8"
                    ),
                ),
                (
                    "ddn.project.json",
                    (
                        PACK
                        / "root_discovery"
                        / "project_block_vs_projectjson_conflict_v1"
                        / "ddn.project.json"
                    ).read_text(encoding="utf-8"),
                ),
                ("app/main.ddn", None),
            ],
            "E_PROJECT_BLOCK_VS_PROJECTJSON_CONFLICT",
        ),
    ]
    for case_id, files, code in negative_cases:
        case_dir = PACK / "root_discovery" / case_id
        disk_proc = project_cli(["project-discover", str(case_dir / "app" / "main.ddn")])
        virtual_proc = run_virtual_discover(virtual_request(files))
        for label, proc in (("disk", disk_proc), ("virtual", virtual_proc)):
            combined = f"{proc.stdout}\n{proc.stderr}"
            if proc.returncode == 0:
                return fail("E_DDN_LOCAL_PROJECT_GRAPH_NEGATIVE_EXIT", f"{case_id}/{label}")
            if code not in combined:
                return fail("E_DDN_LOCAL_PROJECT_GRAPH_NEGATIVE_CODE", f"{case_id}/{label}: {combined.strip()}")
    return 0


def check_project_graph_product() -> int:
    fixture = PACK / "fixtures" / "project_graph_v1.json"
    expected_path = PACK / "expected" / "project_graph_v1.detjson"
    cli = project_cli(["project-graph", str(fixture)], timeout=180)
    if cli.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_CLI_GRAPH", cli.stderr.strip() or cli.stdout.strip())
    node = run(["node", "tests/ddn_local_project_graph_runner.mjs", "graph", str(fixture)], timeout=120)
    if node.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_WASM_GRAPH", node.stderr.strip() or node.stdout.strip())
    expected = expected_path.read_text(encoding="utf-8")
    if cli.stdout != expected:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_EXPECTED_GRAPH", cli.stdout)
    if node.stdout != expected:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_WASM_EQUALITY", "WASM graph differs from expected")
    graph = json.loads(expected)
    checks = {
        "schema": "ddn.local_project_graph.v1",
        "project_name": "포물선연구실",
        "root_path": "뿌리.ddn",
        "entry_path": "app/main.ddn",
        "file_count": 6,
    }
    for key, value in checks.items():
        if graph.get(key) != value:
            return fail("E_DDN_LOCAL_PROJECT_GRAPH_FIELD", f"{key}={graph.get(key)!r}")
    if graph.get("targets", {}).get("앱") != "app/main.ddn":
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_TARGET", repr(graph.get("targets")))
    if graph.get("module_paths", {}).get("로컬") != "lib":
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_MODULE_PATH", repr(graph.get("module_paths")))
    if graph.get("gaji_aliases", {}).get("물리") != "vendor/edu.physics":
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_GAJI_ALIAS", repr(graph.get("gaji_aliases")))
    paths = [row.get("path") for row in graph.get("files", [])]
    if paths != sorted(paths):
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_FILE_SORT", repr(paths))
    if not str(graph.get("project_graph_sha256", "")).strip():
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_HASH", "missing hash")

    negative = project_cli(["project-graph", str(PACK / "fixtures" / "project_block_outside_root_v1.json")], timeout=180)
    combined = f"{negative.stdout}\n{negative.stderr}"
    if negative.returncode == 0:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_OUTSIDE_EXIT", "outside-root project block passed")
    if "E_PROJECT_BLOCK_OUTSIDE_ROOT" not in combined:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_OUTSIDE_CODE", combined.strip())
    return 0


def check_symbol_boundary_product() -> int:
    fixture = PACK / "fixtures" / "project_graph_v1.json"
    expected_path = PACK / "expected" / "project_symbols_v1.detjson"
    cli = project_cli(["project-symbols", str(fixture)], timeout=180)
    if cli.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_SYMBOL_CLI", cli.stderr.strip() or cli.stdout.strip())
    node = run(["node", "tests/ddn_local_project_graph_runner.mjs", "symbols", str(fixture)], timeout=120)
    if node.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_SYMBOL_WASM", node.stderr.strip() or node.stdout.strip())
    expected = expected_path.read_text(encoding="utf-8")
    if cli.stdout != expected:
        return fail("E_DDN_LOCAL_PROJECT_SYMBOL_EXPECTED", cli.stdout)
    if node.stdout != expected:
        return fail("E_DDN_LOCAL_PROJECT_SYMBOL_EQUALITY", "WASM symbols differ from expected")
    doc = json.loads(expected)
    if doc.get("schema") != "ddn.local_project_symbol_boundary.v1":
        return fail("E_DDN_LOCAL_PROJECT_SYMBOL_SCHEMA", repr(doc.get("schema")))
    if doc.get("output_rows") != ["7"]:
        return fail("E_DDN_LOCAL_PROJECT_SYMBOL_OUTPUT", repr(doc.get("output_rows")))
    resolved = doc.get("resolved_symbols", [])
    if len(resolved) != 1:
        return fail("E_DDN_LOCAL_PROJECT_SYMBOL_RESOLVED_COUNT", repr(resolved))
    row = resolved[0]
    if row.get("alias") != "운동" or row.get("symbol") != "속도구하기" or row.get("value") != "7.0":
        return fail("E_DDN_LOCAL_PROJECT_SYMBOL_RESOLVED", repr(row))
    if not str(doc.get("symbol_boundary_sha256", "")).strip():
        return fail("E_DDN_LOCAL_PROJECT_SYMBOL_HASH", "missing symbol hash")

    private = project_cli(["project-symbols", str(PACK / "fixtures" / "project_symbol_private_v1.json")], timeout=180)
    combined = f"{private.stdout}\n{private.stderr}"
    if private.returncode == 0:
        return fail("E_DDN_LOCAL_PROJECT_SYMBOL_PRIVATE_EXIT", "private symbol access passed")
    if "E_SYMBOL_NOT_EXPORTED" not in combined:
        return fail("E_DDN_LOCAL_PROJECT_SYMBOL_PRIVATE_CODE", combined.strip())
    return 0


def check_module_graph_product() -> int:
    fixture = PACK / "fixtures" / "project_module_reexport_v1.json"
    expected = (PACK / "expected" / "project_module_reexport_v1.detjson").read_text(encoding="utf-8")
    cli = project_cli(["project-symbols", str(fixture)], timeout=180)
    if cli.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_REEXPORT_CLI", cli.stderr.strip() or cli.stdout.strip())
    if cli.stdout != expected:
        return fail("E_DDN_LOCAL_PROJECT_REEXPORT_EXPECTED", cli.stdout)
    node = run(["node", "tests/ddn_local_project_graph_runner.mjs", "symbols", str(fixture)], timeout=120)
    if node.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_REEXPORT_WASM", node.stderr.strip() or node.stdout.strip())
    if node.stdout != expected:
        return fail("E_DDN_LOCAL_PROJECT_REEXPORT_WASM_EQUALITY", "WASM re-export symbols differ")
    doc = json.loads(expected)
    if doc.get("output_rows") != ["7"]:
        return fail("E_DDN_LOCAL_PROJECT_REEXPORT_OUTPUT", repr(doc.get("output_rows")))
    if doc.get("module_graph_order") != ["app/main.ddn", "lib/공식.ddn", "lib/운동.ddn"]:
        return fail("E_DDN_LOCAL_PROJECT_REEXPORT_ORDER", repr(doc.get("module_graph_order")))
    if not str(doc.get("module_graph_sha256", "")).strip():
        return fail("E_DDN_LOCAL_PROJECT_REEXPORT_GRAPH_HASH", "missing module graph hash")

    shuffled = project_cli(["project-symbols", str(PACK / "fixtures" / "project_module_reexport_shuffle_v1.json")], timeout=180)
    if shuffled.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_REEXPORT_SHUFFLE", shuffled.stderr.strip() or shuffled.stdout.strip())
    if shuffled.stdout != expected:
        return fail("E_DDN_LOCAL_PROJECT_REEXPORT_SHUFFLE_EQUALITY", "shuffle changed symbol artifact")

    negatives = [
        ("project_module_cycle_v1.json", "E_IMPORT_CYCLE"),
        ("project_module_top_level_effect_v1.json", "E_IMPORTED_TOP_LEVEL_EFFECT_UNSUPPORTED"),
    ]
    for filename, code in negatives:
        proc = project_cli(["project-symbols", str(PACK / "fixtures" / filename)], timeout=180)
        combined = f"{proc.stdout}\n{proc.stderr}"
        if proc.returncode == 0:
            return fail("E_DDN_LOCAL_PROJECT_MODULE_NEGATIVE_EXIT", filename)
        if code not in combined:
            return fail("E_DDN_LOCAL_PROJECT_MODULE_NEGATIVE_CODE", f"{filename}: {combined.strip()}")
    return 0


def check_module_duplicate_diagnostics_have_spans() -> int:
    cases = [
        ("project_module_duplicate_import_v1.json", "E_IMPORT_ALIAS_DUPLICATE", "app/main.ddn"),
        ("project_module_duplicate_export_v1.json", "E_EXPORT_DUPLICATE", "lib/운동.ddn"),
    ]
    for filename, code, path in cases:
        proc = project_cli(["project-symbols", str(PACK / "fixtures" / filename)], timeout=180)
        if proc.returncode == 0:
            return fail("E_DDN_LOCAL_PROJECT_MODULE_DUP_EXIT", filename)
        combined = (proc.stdout or proc.stderr).strip()
        try:
            doc = load_last_json_object(combined)
        except json.JSONDecodeError as exc:
            return fail("E_DDN_LOCAL_PROJECT_MODULE_DUP_JSON", f"{filename}: {exc}: {combined}")
        if doc.get("code") != code:
            return fail("E_DDN_LOCAL_PROJECT_MODULE_DUP_CODE", repr(doc))
        if doc.get("path") != path:
            return fail("E_DDN_LOCAL_PROJECT_MODULE_DUP_PATH", repr(doc))
        span = doc.get("source_span")
        if not isinstance(span, dict) or not isinstance(span.get("start"), int) or not isinstance(span.get("end"), int):
            return fail("E_DDN_LOCAL_PROJECT_MODULE_DUP_SPAN", repr(doc))
        if not (0 <= span["start"] < span["end"]):
            return fail("E_DDN_LOCAL_PROJECT_MODULE_DUP_SPAN_ORDER", repr(doc))
    return 0


def check_gaji_lock_product() -> int:
    fixture = PACK / "fixtures" / "project_gaji_lock_generate_v1.json"
    frozen_fixture = PACK / "fixtures" / "project_gaji_lock_frozen_v1.json"
    expected = (PACK / "expected" / "project_gaji_lock_v1.detjson").read_text(encoding="utf-8")
    cli = project_cli(["project-lock", str(fixture)], timeout=180)
    if cli.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_GAJI_LOCK_CLI", cli.stderr.strip() or cli.stdout.strip())
    if cli.stdout != expected:
        return fail("E_DDN_LOCAL_PROJECT_GAJI_LOCK_EXPECTED", cli.stdout)
    frozen = project_cli(["project-lock", "--frozen", str(frozen_fixture)], timeout=180)
    if frozen.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_GAJI_LOCK_FROZEN", frozen.stderr.strip() or frozen.stdout.strip())
    if frozen.stdout != expected:
        return fail("E_DDN_LOCAL_PROJECT_GAJI_LOCK_FROZEN_EXPECTED", frozen.stdout)
    node = run(["node", "tests/ddn_local_project_graph_runner.mjs", "lock", str(fixture)], timeout=120)
    if node.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_GAJI_LOCK_WASM", node.stderr.strip() or node.stdout.strip())
    if node.stdout != expected:
        return fail("E_DDN_LOCAL_PROJECT_GAJI_LOCK_WASM_EQUALITY", "WASM lock differs")
    doc = json.loads(expected)
    if doc.get("schema") != "ddn.local_gaji_lock.v1":
        return fail("E_DDN_LOCAL_PROJECT_GAJI_LOCK_SCHEMA", repr(doc.get("schema")))
    packages = doc.get("packages")
    if not isinstance(packages, list) or [row.get("id") for row in packages] != ["edu.physics", "edu.units"]:
        return fail("E_DDN_LOCAL_PROJECT_GAJI_LOCK_PACKAGES", repr(packages))
    physics = packages[0]
    units = packages[1]
    if physics.get("direct") is not True or physics.get("dependencies") != {"edu.units": "0.1.0"}:
        return fail("E_DDN_LOCAL_PROJECT_GAJI_LOCK_DIRECT", repr(physics))
    if units.get("direct") is not False or units.get("dependency_of") != ["edu.physics"]:
        return fail("E_DDN_LOCAL_PROJECT_GAJI_LOCK_TRANSITIVE", repr(units))
    if not str(doc.get("lock_sha256", "")).strip():
        return fail("E_DDN_LOCAL_PROJECT_GAJI_LOCK_HASH", "missing lock hash")

    negatives = [
        ("project_gaji_lock_missing_v1.json", ["--frozen"], "E_LOCK_MISSING"),
        ("project_gaji_lock_tampered_v1.json", [], "E_LOCK_HASH_MISMATCH"),
        ("project_gaji_lock_unpinned_v1.json", [], "E_GAJI_DEP_UNPINNED"),
        ("project_gaji_lock_transitive_missing_v1.json", [], "E_GAJI_TRANSITIVE_MISSING"),
        ("project_gaji_lock_id_path_mismatch_v1.json", [], "E_GAJI_ID_PATH_MISMATCH"),
    ]
    for filename, extra, code in negatives:
        proc = project_cli(["project-lock", *extra, str(PACK / "fixtures" / filename)], timeout=180)
        combined = f"{proc.stdout}\n{proc.stderr}"
        if proc.returncode == 0:
            return fail("E_DDN_LOCAL_PROJECT_GAJI_LOCK_NEGATIVE_EXIT", filename)
        if code not in combined:
            return fail("E_DDN_LOCAL_PROJECT_GAJI_LOCK_NEGATIVE_CODE", f"{filename}: {combined.strip()}")
    return 0


def check_project_run_frontdoor() -> int:
    start = PACK / "project_frontdoor" / "project_run_v1" / "app" / "main.ddn"
    expected = (PACK / "expected" / "project_run_summary_v1.detjson").read_text(encoding="utf-8")
    proc = project_cli(
        ["project-run", str(start), "--target", "앱", "--inspect-graph", "--inspect-lock"],
        timeout=180,
    )
    if proc.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_RUN_FRONTDOOR", proc.stderr.strip() or proc.stdout.strip())
    if proc.stdout != expected:
        return fail("E_DDN_LOCAL_PROJECT_RUN_EXPECTED", proc.stdout)
    doc = json.loads(expected)
    if doc.get("schema") != "ddn.local_project_run_summary.v1":
        return fail("E_DDN_LOCAL_PROJECT_RUN_SCHEMA", repr(doc.get("schema")))
    if doc.get("frontdoor") != "ddn.project-run.v1":
        return fail("E_DDN_LOCAL_PROJECT_RUN_FRONTDOOR_FIELD", repr(doc.get("frontdoor")))
    if doc.get("target") != "앱" or doc.get("target_path") != "app/main.ddn":
        return fail("E_DDN_LOCAL_PROJECT_RUN_TARGET", repr(doc))
    if doc.get("runtime_output_log_texts") != ["7", "10"]:
        return fail("E_DDN_LOCAL_PROJECT_RUN_OUTPUT", repr(doc.get("runtime_output_log_texts")))
    if doc.get("graph_inspected") is not True or doc.get("lock_inspected") is not True:
        return fail("E_DDN_LOCAL_PROJECT_RUN_INSPECT", repr(doc))
    for key in (
        "project_graph_sha256",
        "lock_sha256",
        "symbol_boundary_sha256",
        "runtime_state_hash",
        "linked_execution_source_sha256",
        "linked_execution_canonical_sha256",
        "bounded_state_sha256",
        "summary_sha256",
    ):
        if not str(doc.get(key, "")).strip():
            return fail("E_DDN_LOCAL_PROJECT_RUN_HASH", key)
    if not isinstance(doc.get("output_rows"), list) or not doc.get("output_rows"):
        return fail("E_DDN_LOCAL_PROJECT_RUN_OBSERVATION_ROWS", repr(doc.get("output_rows")))
    if not isinstance(doc.get("runtime_execution_envelope"), dict):
        return fail("E_DDN_LOCAL_PROJECT_RUN_ENVELOPE", repr(doc.get("runtime_execution_envelope")))
    prep_rc = check_execution_preparation(doc, "frontdoor")
    if prep_rc:
        return prep_rc
    trace_rc = check_project_observation_trace(doc, "frontdoor")
    if trace_rc:
        return trace_rc
    official_rc = check_official_run_matches_project_summary(doc, "frontdoor")
    if official_rc:
        return official_rc

    fixture_root = PACK / "project_frontdoor" / "project_run_v1"
    cwd_proc = run_with_cwd(
        [
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            str(PROJECT_MANIFEST),
            "--bin",
            "ddonirang",
            "--",
            "project-run",
            "--target",
            "앱",
            "--inspect-graph",
            "--inspect-lock",
        ],
        cwd=fixture_root,
        timeout=180,
    )
    if cwd_proc.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_RUN_CWD", cwd_proc.stderr.strip() or cwd_proc.stdout.strip())
    if cwd_proc.stdout != expected:
        return fail("E_DDN_LOCAL_PROJECT_RUN_CWD_EXPECTED", cwd_proc.stdout)

    virtual_fixture = PACK / "fixtures" / "project_run_physics_v1.json"
    for conflict_args in (
        ["project-run", str(start), "--virtual", str(virtual_fixture)],
        ["project-run", "--project", "project.ddnproj", "--virtual", str(virtual_fixture)],
    ):
        conflict = project_cli(conflict_args, timeout=180)
        combined = f"{conflict.stdout}\n{conflict.stderr}"
        if conflict.returncode == 0:
            return fail("E_DDN_LOCAL_PROJECT_RUN_CONFLICT_EXIT", repr(conflict_args))
        if "E_CLI_OPTION_CONFLICT" not in combined:
            return fail("E_DDN_LOCAL_PROJECT_RUN_CONFLICT_CODE", combined.strip())

    with tempfile.TemporaryDirectory(prefix="ddn-project-run-out-") as tmp:
        output = Path(tmp) / "nested" / "artifact.detjson"
        out_proc = project_cli(
            [
                "project-run",
                "--virtual",
                str(virtual_fixture),
                "--target",
                "앱",
                "--inspect-graph",
                "--inspect-lock",
                "--out",
                str(output),
            ],
            timeout=180,
        )
        if out_proc.returncode != 0 or out_proc.stdout:
            return fail(
                "E_DDN_LOCAL_PROJECT_RUN_ATOMIC_WRITE",
                out_proc.stderr.strip() or out_proc.stdout,
            )
        if output.read_text(encoding="utf-8") != expected:
            return fail("E_DDN_LOCAL_PROJECT_RUN_ATOMIC_CONTENT", str(output))
        output.write_text("sentinel", encoding="utf-8")
        replace_proc = project_cli(
            [
                "project-run",
                "--virtual",
                str(virtual_fixture),
                "--target",
                "앱",
                "--inspect-graph",
                "--inspect-lock",
                "--out",
                str(output),
            ],
            timeout=180,
        )
        if replace_proc.returncode != 0 or output.read_text(encoding="utf-8") != expected:
            return fail(
                "E_DDN_LOCAL_PROJECT_RUN_ATOMIC_REPLACE",
                replace_proc.stderr.strip() or str(output),
            )
        blocked_target = Path(tmp) / "blocked"
        blocked_target.mkdir()
        blocked_proc = project_cli(
            ["project-run", "--virtual", str(virtual_fixture), "--out", str(blocked_target)],
            timeout=180,
        )
        blocked_combined = f"{blocked_proc.stdout}\n{blocked_proc.stderr}"
        if blocked_proc.returncode == 0 or "E_PROJECT_RUN_WRITE" not in blocked_combined:
            return fail("E_DDN_LOCAL_PROJECT_RUN_ATOMIC_FAIL_CLOSED", blocked_combined.strip())
        if not blocked_target.is_dir():
            return fail("E_DDN_LOCAL_PROJECT_RUN_PARTIAL_PUBLISH", str(blocked_target))
    return 0


def check_wasm_virtual_project_run_parity() -> int:
    cases = [
        (
            "physics",
            PACK / "fixtures" / "project_run_physics_v1.json",
            PACK / "expected" / "project_run_summary_v1.detjson",
            ["7", "10"],
        ),
        (
            "grid",
            PACK / "fixtures" / "project_run_grid_v1.json",
            PACK / "expected" / "project_run_grid_summary_v1.detjson",
            ["3"],
        ),
    ]
    outputs: dict[str, str] = {}
    state_hashes: dict[str, str] = {}
    for label, fixture, expected_path, output_rows in cases:
        expected = expected_path.read_text(encoding="utf-8")
        cli = project_cli(
            ["project-run", "--virtual", str(fixture), "--target", "앱", "--inspect-graph", "--inspect-lock"],
            timeout=180,
        )
        if cli.returncode != 0:
            return fail("E_DDN_LOCAL_PROJECT_WASM_PARITY_CLI", f"{label}: {cli.stderr.strip() or cli.stdout.strip()}")
        wasm = run(["node", "tests/ddn_local_project_graph_runner.mjs", "run", str(fixture), "앱"], timeout=120)
        if wasm.returncode != 0:
            return fail("E_DDN_LOCAL_PROJECT_WASM_PARITY_RUNNER", f"{label}: {wasm.stderr.strip() or wasm.stdout.strip()}")
        if cli.stdout != wasm.stdout:
            return fail("E_DDN_LOCAL_PROJECT_WASM_PARITY_BYTES", label)
        if cli.stdout != expected:
            return fail("E_DDN_LOCAL_PROJECT_WASM_PARITY_EXPECTED", f"{label}: {cli.stdout}")
        doc = json.loads(expected)
        if doc.get("runtime_output_log_texts") != output_rows:
            return fail("E_DDN_LOCAL_PROJECT_WASM_PARITY_RUNTIME_OUTPUT", f"{label}: {doc.get('runtime_output_log_texts')!r}")
        if not str(doc.get("runtime_state_hash", "")).strip():
            return fail("E_DDN_LOCAL_PROJECT_WASM_PARITY_RUNTIME_STATE", label)
        if not str(doc.get("linked_execution_canonical_sha256", "")).strip():
            return fail("E_DDN_LOCAL_PROJECT_WASM_PARITY_LINKED_HASH", label)
        prep_rc = check_execution_preparation(doc, label)
        if prep_rc:
            return prep_rc
        trace_rc = check_project_observation_trace(doc, label)
        if trace_rc:
            return trace_rc
        official_rc = check_official_run_matches_project_summary(doc, label)
        if official_rc:
            return official_rc
        if not str(doc.get("bounded_state_sha256", "")).strip():
            return fail("E_DDN_LOCAL_PROJECT_WASM_PARITY_STATE_HASH", label)
        outputs[label] = expected
        state_hashes[label] = doc["bounded_state_sha256"]
    if outputs["physics"] == outputs["grid"]:
        return fail("E_DDN_LOCAL_PROJECT_WASM_NEGATIVE_OUTPUT", "physics/grid unexpectedly identical")
    if state_hashes["physics"] == state_hashes["grid"]:
        return fail("E_DDN_LOCAL_PROJECT_WASM_NEGATIVE_STATE", "physics/grid bounded state hashes unexpectedly identical")
    return 0


def check_alias_spacing_runtime_product() -> int:
    normal_fixture = PACK / "fixtures" / "project_run_alias_normal_v1.json"
    spacing_fixture = PACK / "fixtures" / "project_run_alias_spacing_v1.json"
    normal_expected = json.loads((PACK / "expected" / "project_run_alias_normal_v1.detjson").read_text(encoding="utf-8"))
    spacing_expected = json.loads((PACK / "expected" / "project_run_alias_spacing_v1.detjson").read_text(encoding="utf-8"))
    for label, fixture, expected in [
        ("normal", normal_fixture, normal_expected),
        ("spacing", spacing_fixture, spacing_expected),
    ]:
        proc = project_cli(["project-run", "--virtual", str(fixture), "--target", "앱", "--inspect-graph"], timeout=180)
        if proc.returncode != 0:
            return fail("E_DDN_LOCAL_PROJECT_ALIAS_SPACING_CLI", f"{label}: {proc.stderr.strip() or proc.stdout.strip()}")
        actual = json.loads(proc.stdout)
        if actual != expected:
            return fail("E_DDN_LOCAL_PROJECT_ALIAS_SPACING_EXPECTED", label)
        if actual.get("runtime_output_log_texts") != ["7", "10"]:
            return fail("E_DDN_LOCAL_PROJECT_ALIAS_SPACING_OUTPUT", f"{label}: {actual.get('runtime_output_log_texts')!r}")
        prep_rc = check_execution_preparation(actual, f"alias-{label}")
        if prep_rc:
            return prep_rc
        trace_rc = check_project_observation_trace(actual, f"alias-{label}")
        if trace_rc:
            return trace_rc
        linked = str(actual.get("linked_execution_source", ""))
        if "운동 . 속도구하기" in linked or "물리 . 중력" in linked:
            return fail("E_DDN_LOCAL_PROJECT_ALIAS_SPACING_LINKED_SOURCE", linked)

    equal_keys = [
        "symbol_boundary_sha256",
        "output_rows",
        "runtime_output_log_texts",
        "runtime_final_row",
        "runtime_state_hash",
        "runtime_execution_envelope",
        "linked_execution_source",
        "linked_execution_canonical_ddn",
        "linked_execution_source_sha256",
        "linked_execution_canonical_sha256",
        "project_observation_trace",
        "bounded_state_sha256",
    ]
    for key in equal_keys:
        if normal_expected.get(key) != spacing_expected.get(key):
            return fail("E_DDN_LOCAL_PROJECT_ALIAS_SPACING_EQUALITY", key)
    if normal_expected.get("project_graph_sha256") == spacing_expected.get("project_graph_sha256"):
        return fail("E_DDN_LOCAL_PROJECT_ALIAS_SPACING_GRAPH_CONTROL", "source-sensitive graph hashes unexpectedly equal")

    wasm_docs = []
    for label, fixture in [("normal", normal_fixture), ("spacing", spacing_fixture)]:
        wasm = run(["node", "tests/ddn_local_project_graph_runner.mjs", "run", str(fixture), "앱"], timeout=120)
        if wasm.returncode != 0:
            return fail("E_DDN_LOCAL_PROJECT_ALIAS_SPACING_WASM", f"{label}: {wasm.stderr.strip() or wasm.stdout.strip()}")
        wasm_docs.append(json.loads(wasm.stdout))
    for key in equal_keys:
        if wasm_docs[0].get(key) != wasm_docs[1].get(key):
            return fail("E_DDN_LOCAL_PROJECT_ALIAS_SPACING_WASM_EQUALITY", key)
        if wasm_docs[0].get(key) != normal_expected.get(key):
            return fail("E_DDN_LOCAL_PROJECT_ALIAS_SPACING_CLI_WASM_EQUALITY", key)
    return 0


def check_alias_string_comment_negative_control() -> int:
    payload = {
        "schema": "ddn.virtual_project_source.v1",
        "kind": "seamgrim.local_project_tree.v0",
        "root_path": "뿌리.ddn",
        "entry_path": "app/main.ddn",
        "files": [
            {
                "path": "뿌리.ddn",
                "role": "root",
                "utf8_text": (
                    "프로젝트 {\n"
                    "  이름: \"alias-string-comment\".\n"
                    "  첫글: \"./app/main.ddn\".\n"
                    "  목표 {\n"
                    "    앱: \"./app/main.ddn\".\n"
                    "  }\n"
                    "  모듈길 {\n"
                    "    로컬: \"./lib\".\n"
                    "  }\n"
                    "}.\n"
                ),
            },
            {
                "path": "app/main.ddn",
                "role": "ddn_source",
                "utf8_text": (
                    "쓰임 {\n"
                    "  운동: \"./lib/운동\".\n"
                    "}\n"
                    "메모 <- \"운동.속도구하기\".\n"
                    "메모 보여주기.\n"
                    "// 운동.속도구하기 보여주기.\n"
                    "운동 . 속도구하기 보여주기.\n"
                ),
            },
            {
                "path": "lib/운동.ddn",
                "role": "ddn_source",
                "utf8_text": "드러냄 {\n  속도구하기.\n}\n속도구하기 <- 7.\n",
            },
        ],
    }
    with tempfile.TemporaryDirectory(prefix="ddn-alias-comment-") as tmp:
        fixture = Path(tmp) / "project.json"
        fixture.write_text(json.dumps(payload, ensure_ascii=False, sort_keys=True), encoding="utf-8")
        proc = project_cli(["project-run", "--virtual", str(fixture), "--target", "앱", "--inspect-graph"], timeout=180)
    if proc.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_ALIAS_COMMENT_CLI", proc.stderr.strip() or proc.stdout.strip())
    doc = json.loads(proc.stdout)
    if doc.get("runtime_output_log_texts") != ["운동.속도구하기", "7"]:
        return fail("E_DDN_LOCAL_PROJECT_ALIAS_COMMENT_OUTPUT", repr(doc.get("runtime_output_log_texts")))
    linked = str(doc.get("linked_execution_source", ""))
    if '"운동.속도구하기"' not in linked:
        return fail("E_DDN_LOCAL_PROJECT_ALIAS_STRING_PRESERVED", linked)
    if "// 운동.속도구하기 보여주기." not in linked:
        return fail("E_DDN_LOCAL_PROJECT_ALIAS_COMMENT_TEXT_PRESERVED", linked)
    if linked.count("\n속도구하기 보여주기.") != 1:
        return fail("E_DDN_LOCAL_PROJECT_ALIAS_COMMENT_PRESERVED", linked)
    return 0


def check_project_runtime_parity_negative_controls() -> int:
    base_doc = json.loads((PACK / "expected" / "project_run_alias_normal_v1.detjson").read_text(encoding="utf-8"))
    base_payload = load_json(PACK / "fixtures" / "project_run_alias_normal_v1.json")

    value_payload = json.loads(json.dumps(base_payload, ensure_ascii=False))
    payload_replace_file_text(value_payload, "lib/운동.ddn", "속도구하기 <- 7.", "속도구하기 <- 8.")
    try:
        value_doc, _, _ = run_virtual_project_payload(value_payload)
    except RuntimeError as exc:
        return fail("E_DDN_LOCAL_PROJECT_RUNTIME_NEG_VALUE_RUN", str(exc))
    trace_rc = check_project_observation_trace(value_doc, "mutated-value")
    if trace_rc:
        return trace_rc
    if value_doc.get("runtime_output_log_texts") != ["8", "10"]:
        return fail("E_DDN_LOCAL_PROJECT_RUNTIME_NEG_VALUE_RUNTIME_OUTPUT", repr(value_doc.get("runtime_output_log_texts")))
    if value_doc.get("runtime_final_row", {}).get("속도구하기") != 8.0:
        return fail("E_DDN_LOCAL_PROJECT_RUNTIME_NEG_VALUE_FINAL", repr(value_doc.get("runtime_final_row")))
    changed_keys = [
        "runtime_state_hash",
        "linked_execution_source_sha256",
        "linked_execution_canonical_sha256",
        "bounded_state_sha256",
        "summary_sha256",
    ]
    for key in changed_keys:
        if value_doc.get(key) == base_doc.get(key):
            return fail("E_DDN_LOCAL_PROJECT_RUNTIME_NEG_VALUE_HASH", key)
    official_rc = check_official_run_matches_project_summary(value_doc, "mutated-value")
    if official_rc:
        return official_rc

    output_payload = json.loads(json.dumps(base_payload, ensure_ascii=False))
    payload_replace_file_text(output_payload, "app/main.ddn", "물리.중력 보여주기.\n", "")
    try:
        output_doc, _, _ = run_virtual_project_payload(output_payload)
    except RuntimeError as exc:
        return fail("E_DDN_LOCAL_PROJECT_RUNTIME_NEG_OUTPUT_RUN", str(exc))
    trace_rc = check_project_observation_trace(output_doc, "output-row-removal")
    if trace_rc:
        return trace_rc
    if output_doc.get("runtime_output_log_texts") != ["7"]:
        return fail("E_DDN_LOCAL_PROJECT_RUNTIME_NEG_RUNTIME_ROWS", repr(output_doc.get("runtime_output_log_texts")))
    for key in [
        "linked_execution_source_sha256",
        "linked_execution_canonical_sha256",
        "bounded_state_sha256",
        "summary_sha256",
    ]:
        if output_doc.get(key) == base_doc.get(key):
            return fail("E_DDN_LOCAL_PROJECT_RUNTIME_NEG_OUTPUT_HASH", key)
    official_rc = check_official_run_matches_project_summary(output_doc, "output-row-removal")
    if official_rc:
        return official_rc

    cli_error, wasm_error = run_virtual_project_error(base_payload, "없는목표")
    if "E_PROJECT_TARGET_MISSING" not in cli_error:
        return fail("E_DDN_LOCAL_PROJECT_RUNTIME_NEG_TARGET_CLI", cli_error.strip())
    if "E_PROJECT_TARGET_MISSING" not in wasm_error:
        return fail("E_DDN_LOCAL_PROJECT_RUNTIME_NEG_TARGET_WASM", wasm_error.strip())
    return 0


def check_real_linked_summary_control() -> int:
    payload = {
        "schema": "ddn.virtual_project_source.v1",
        "kind": "seamgrim.local_project_tree.v0",
        "root_path": "뿌리.ddn",
        "entry_path": "app/main.ddn",
        "files": [
            {
                "path": "뿌리.ddn",
                "role": "root",
                "utf8_text": (
                    "프로젝트 {\n"
                    "  이름: \"fake-summary-negative\".\n"
                    "  첫글: \"./app/main.ddn\".\n"
                    "  목표 {\n"
                    "    앱: \"./app/main.ddn\".\n"
                    "  }\n"
                    "}.\n"
                ),
            },
            {
                "path": "app/main.ddn",
                "role": "ddn_source",
                "utf8_text": "쓰임 {\n  도움: \"./lib/helper\".\n}\n도움.값 보여주기.\n",
            },
            {
                "path": "lib/helper.ddn",
                "role": "ddn_source",
                "utf8_text": "드러냄 {\n  값.\n}\n값 := 99.\n",
            },
        ],
    }
    with tempfile.TemporaryDirectory(prefix="ddn-fake-summary-") as tmp:
        fixture = Path(tmp) / "project.json"
        fixture.write_text(json.dumps(payload, ensure_ascii=False, sort_keys=True), encoding="utf-8")
        cli = project_cli(["project-run", "--virtual", str(fixture), "--target", "앱", "--inspect-graph", "--inspect-lock"], timeout=180)
        if cli.returncode != 0:
            return fail("E_DDN_LOCAL_PROJECT_REAL_SUMMARY_CLI", cli.stderr.strip() or cli.stdout)
        wasm = run(["node", "tests/ddn_local_project_graph_runner.mjs", "run", str(fixture), "앱"], timeout=120)
        if wasm.returncode != 0:
            return fail("E_DDN_LOCAL_PROJECT_REAL_SUMMARY_WASM", wasm.stderr.strip() or wasm.stdout)
        if cli.stdout != wasm.stdout:
            return fail("E_DDN_LOCAL_PROJECT_REAL_SUMMARY_PARITY", "CLI/WASM bytes differ")
        doc = json.loads(cli.stdout)
        if doc.get("frontdoor") != "ddn.project-run.v1":
            return fail("E_DDN_LOCAL_PROJECT_REAL_SUMMARY_FRONTDOOR", repr(doc.get("frontdoor")))
        if doc.get("runtime_output_log_texts") != ["99"]:
            return fail("E_DDN_LOCAL_PROJECT_REAL_SUMMARY_OUTPUT", repr(doc.get("runtime_output_log_texts")))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"])
    if proc.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_DDN_LOCAL_PROJECT_GRAPH_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (
        check_files,
        check_baseline,
        check_golden,
        check_root_discovery_product,
        check_project_graph_product,
        check_symbol_boundary_product,
        check_module_graph_product,
        check_module_duplicate_diagnostics_have_spans,
        check_gaji_lock_product,
        check_project_run_frontdoor,
        check_wasm_virtual_project_run_parity,
        check_alias_spacing_runtime_product,
        check_alias_string_comment_negative_control,
        check_project_runtime_parity_negative_controls,
        check_real_linked_summary_control,
        check_docs_ssot_clean,
    ):
        rc = check()
        if rc:
            return rc
    print("[ddn-local-project-graph-v1] OK G0 baseline + G2 root discovery + G3 project graph + G4 symbols + G5 module graph + G6 local gaji lock + G7 CLI project frontdoor + G8 WASM virtual parity")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
