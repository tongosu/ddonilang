#!/usr/bin/env python
from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "creator_gaji_round5_regression_v1"
EVIDENCE = PACK / "r5_evidence.detjson"
MANIFEST = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm" / "ddonirang_tool.manifest.detjson"
WASM = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm" / "ddonirang_tool_bg.wasm"
JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm" / "ddonirang_tool.js"
TEXT_HASH_MODE = "crlf_to_lf_then_sha256"
RAW_HASH_MODE = "raw_bytes_sha256"


def fail(code: str, detail: str) -> int:
    print(f"[creator-gaji-round5-regression] fail {code}: {detail}")
    return 1


def canonical_text_bytes(data: bytes) -> bytes:
    return data.replace(b"\r\n", b"\n")


def sha256(path: Path, *, text: bool = False) -> str:
    data = path.read_bytes()
    if text:
        data = canonical_text_bytes(data)
    return hashlib.sha256(data).hexdigest()


def validate_hash_modes(manifest: dict) -> list[str]:
    failures: list[str] = []
    if manifest.get("source_hash_mode") != TEXT_HASH_MODE:
        failures.append("source_hash_mode")
    if manifest.get("metadata_text_hash_mode") != TEXT_HASH_MODE:
        failures.append("metadata_text_hash_mode")
    if manifest.get("js_binding_hash_mode") != TEXT_HASH_MODE:
        failures.append("js_binding_hash_mode")
    if manifest.get("wasm_hash_mode") != RAW_HASH_MODE:
        failures.append("wasm_hash_mode")
    return failures


def validate_evidence(evidence: dict) -> list[str]:
    failures: list[str] = []
    if evidence.get("schema") != "ddn.creator_gaji_round5_regression.evidence.v1":
        failures.append("schema")
    claims = evidence.get("claims") if isinstance(evidence.get("claims"), dict) else {}
    for key, expected in {
        "r5_complete": True,
        "r0_through_r7_complete": False,
        "overall_goal_complete": False,
        "docs_ssot_change": False,
        "state_hash_formula_change": False,
        "external_creator_pilot_success_count": 0,
    }.items():
        if claims.get(key) != expected:
            failures.append(f"claim:{key}")
    rows = evidence.get("regression_matrix")
    if not isinstance(rows, list) or len(rows) != 16:
        failures.append("regression_matrix_count")
    else:
        ids = {str(row.get("id", "")) for row in rows if isinstance(row, dict)}
        required = {
            "sealed_wasm_browser_binding",
            "creator_gaji_aggregate",
            "four_viewport_creator_circulation",
            "native_multifile_full_promotion",
            "approved_workbench_quality",
            "local_gaji_cli_wasm_parity",
            "multifile_portable_roundtrip",
            "grid_vertical",
            "attack_performance",
            "view_only_state_hash_invariant",
            "disk_project_crlf_portability",
            "local_project_gaji_wave1",
            "vol4_cli_wasm_parity",
            "wasm_freshness",
            "true_parity_equality",
            "disk_crlf_unit",
        }
        if ids != required:
            failures.append("regression_matrix_ids")
        if any(not isinstance(row, dict) or row.get("result") != "PASS" for row in rows):
            failures.append("regression_matrix_result")
    return failures


def required_markers() -> dict[Path, tuple[str, ...]]:
    return {
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm_page_common.js": (
            'const CRLF_TO_LF_SHA256_MODE = "crlf_to_lf_then_sha256";',
            "canonicalizeCrLfBuffer",
            "js_hash_mode: jsHashMode",
            "wasm_hash_mode: wasmHashMode",
        ),
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js": (
            "state.activeFilePath = localProjectEntryPath(project);",
        ),
        ROOT / "tool" / "src" / "local_project.rs": (
            "utf8_text: canonicalize_disk_text(utf8_text)",
            "canonicalizes_disk_crlf_without_changing_lf_or_lone_cr",
        ),
        ROOT / "tests" / "seamgrim_wasm_sealed_runtime_binding_runner.mjs": (
            "js_content_tamper",
            "tamperJs: true",
        ),
        ROOT / "tests" / "seamgrim_gaji_bundle_runner.mjs": (
            'emptyProjectActivePath === "app/main.ddn"',
        ),
        ROOT / "tests" / "seamgrim_native_project_durable_vertical_runner.mjs": (
            "view local run appended server archive",
        ),
        ROOT / "tests" / "run_ddonirang_vol4_bundle_cli_wasm_parity_check.py": (
            "DDN_VOL4_BUNDLE_PATH",
        ),
    }


def main() -> int:
    for path in (PACK / "README.md", EVIDENCE, MANIFEST, WASM, JS):
        if not path.exists():
            return fail("E_R5_MISSING", str(path.relative_to(ROOT)))
    evidence = json.loads(EVIDENCE.read_text(encoding="utf-8"))
    failures = validate_evidence(evidence)
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    failures.extend(f"manifest:{item}" for item in validate_hash_modes(manifest))

    artifact = evidence.get("wasm_artifact") if isinstance(evidence.get("wasm_artifact"), dict) else {}
    actual = {
        "source_hash_mode": manifest.get("source_hash_mode"),
        "source_file_count": manifest.get("source_file_count"),
        "source_tree_sha256": manifest.get("source_tree_sha256"),
        "js_binding_hash_mode": manifest.get("js_binding_hash_mode"),
        "js_binding_sha256": sha256(JS, text=True),
        "wasm_hash_mode": manifest.get("wasm_hash_mode"),
        "wasm_sha256": sha256(WASM),
        "wasm_size": WASM.stat().st_size,
        "manifest_sha256": sha256(MANIFEST, text=True),
    }
    for key, value in actual.items():
        if artifact.get(key) != value:
            failures.append(f"artifact:{key}")
    if manifest.get("js_binding_sha256") != actual["js_binding_sha256"]:
        failures.append("manifest:js_binding_sha256")
    if manifest.get("wasm_sha256") != actual["wasm_sha256"]:
        failures.append("manifest:wasm_sha256")

    for path, markers in required_markers().items():
        text = path.read_text(encoding="utf-8")
        for marker in markers:
            if marker not in text:
                failures.append(f"marker:{path.relative_to(ROOT)}:{marker}")

    mutated_manifest = dict(manifest)
    mutated_manifest["js_binding_hash_mode"] = "unsupported"
    if not validate_hash_modes(mutated_manifest):
        failures.append("selftest:unsupported_hash_mode")
    mutated_evidence = json.loads(json.dumps(evidence, ensure_ascii=False))
    mutated_evidence["regression_matrix"][0]["result"] = "FAIL"
    if not validate_evidence(mutated_evidence):
        failures.append("selftest:failed_regression_row")
    if canonical_text_bytes(b"a\r\nb\r\n") != b"a\nb\n":
        failures.append("selftest:crlf_equivalence")

    if failures:
        return fail("E_R5_EVIDENCE", "; ".join(failures[:12]))
    print("[creator-gaji-round5-regression-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
