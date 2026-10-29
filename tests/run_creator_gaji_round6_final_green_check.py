#!/usr/bin/env python
from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "creator_gaji_round6_final_green_v1"
EVIDENCE = PACK / "r6_evidence.detjson"
MANIFEST = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm" / "ddonirang_tool.manifest.detjson"
WASM = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm" / "ddonirang_tool_bg.wasm"
JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm" / "ddonirang_tool.js"
TEXT_HASH_MODE = "crlf_to_lf_then_sha256"
RAW_HASH_MODE = "raw_bytes_sha256"


def fail(code: str, detail: str) -> int:
    print(f"[creator-gaji-round6-final-green] fail {code}: {detail}")
    return 1


def canonical_text_bytes(data: bytes) -> bytes:
    return data.replace(b"\r\n", b"\n")


def sha256(path: Path, *, text: bool = False) -> str:
    data = path.read_bytes()
    if text:
        data = canonical_text_bytes(data)
    return hashlib.sha256(data).hexdigest()


def validate_evidence(evidence: dict) -> list[str]:
    failures: list[str] = []
    if evidence.get("schema") != "ddn.creator_gaji.round6_final_green.evidence.v1":
        failures.append("schema")
    claims = evidence.get("claims") if isinstance(evidence.get("claims"), dict) else {}
    for key, expected in {
        "r6_complete": True,
        "r7_complete": False,
        "r0_through_r7_complete": False,
        "overall_goal_complete": False,
        "behavior_closed": True,
        "docs_ssot_change": False,
        "state_hash_formula_change": False,
        "external_creator_pilot_success_count": 0,
    }.items():
        if claims.get(key) != expected:
            failures.append(f"claim:{key}")
    rows = evidence.get("test_matrix")
    required = {
        "public_physics_wedge",
        "durable_quality_gate_twice",
        "fixed64_lint",
        "core_all",
        "lang_minimal",
        "tool_minimal",
        "tool_default",
        "root_workspace_check",
        "teul_cli_independent_check",
        "wasm_freshness",
        "cli_wasm_true_parity",
        "cli_wasm_sync_gate",
        "ci_sanity_core_lang",
        "diff_and_ssot_boundary",
    }
    if not isinstance(rows, list) or {str(row.get("id", "")) for row in rows} != required:
        failures.append("test_matrix_ids")
    elif any(not isinstance(row, dict) or row.get("result") != "PASS" for row in rows):
        failures.append("test_matrix_result")
    else:
        durable = next(row for row in rows if row.get("id") == "durable_quality_gate_twice")
        sync = next(row for row in rows if row.get("id") == "cli_wasm_sync_gate")
        if durable.get("runs") != 2:
            failures.append("durable_runs")
        if sync.get("substeps") != 11:
            failures.append("sync_substeps")
    return failures


def required_markers() -> dict[Path, tuple[str, ...]]:
    return {
        ROOT / "tests" / "_run_pack_golden_impl.py": (
            'env["CARGO_TARGET_DIR"] = str(resolve_pack_golden_cargo_target_dir(env))',
        ),
        ROOT / "tool" / "src" / "ai_prompt.rs": (
            'let canonical = text.replace("\\r\\n", "\\n");',
            "ai_prompt_text_canonicalizes_crlf_and_rejects_lone_cr",
            'canonicalize_prompt_bytes("ai_prompt_golden", golden)',
        ),
        ROOT / "tool" / "src" / "main.rs": (
            "(표:(글, 수)짝맞춤~을) 통과:셈씨 = {",
        ),
    }


def main() -> int:
    for path in (PACK / "README.md", EVIDENCE, MANIFEST, WASM, JS):
        if not path.exists():
            return fail("E_R6_MISSING", str(path.relative_to(ROOT)))
    evidence = json.loads(EVIDENCE.read_text(encoding="utf-8"))
    failures = validate_evidence(evidence)
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
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
    if manifest.get("source_hash_mode") != TEXT_HASH_MODE:
        failures.append("manifest:source_hash_mode")
    if manifest.get("js_binding_hash_mode") != TEXT_HASH_MODE:
        failures.append("manifest:js_binding_hash_mode")
    if manifest.get("wasm_hash_mode") != RAW_HASH_MODE:
        failures.append("manifest:wasm_hash_mode")
    if manifest.get("js_binding_sha256") != actual["js_binding_sha256"]:
        failures.append("manifest:js_binding_sha256")
    if manifest.get("wasm_sha256") != actual["wasm_sha256"]:
        failures.append("manifest:wasm_sha256")

    for path, markers in required_markers().items():
        text = path.read_text(encoding="utf-8")
        for marker in markers:
            if marker not in text:
                failures.append(f"marker:{path.relative_to(ROOT)}:{marker}")

    mutated = json.loads(json.dumps(evidence, ensure_ascii=False))
    mutated["test_matrix"][0]["result"] = "FAIL"
    if not validate_evidence(mutated):
        failures.append("selftest:failed_test_row")
    mutated = json.loads(json.dumps(evidence, ensure_ascii=False))
    mutated["claims"]["r7_complete"] = True
    if not validate_evidence(mutated):
        failures.append("selftest:overclaim")

    if failures:
        return fail("E_R6_EVIDENCE", "; ".join(failures[:12]))
    print("[creator-gaji-round6-final-green-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
