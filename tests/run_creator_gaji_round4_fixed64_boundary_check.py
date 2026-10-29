#!/usr/bin/env python
from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "creator_gaji_round4_fixed64_boundary_v1"
EVIDENCE = PACK / "r4_evidence.detjson"
MANIFEST = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm" / "ddonirang_tool.manifest.detjson"
WASM = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm" / "ddonirang_tool_bg.wasm"
JS_BINDING = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm" / "ddonirang_tool.js"
FLOAT_TOKEN = re.compile(r"(?<![A-Za-z0-9_])f(?:32|64)(?![A-Za-z0-9_])")


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def canonical_text_bytes(data: bytes) -> bytes:
    return data.replace(b"\r\n", b"\n")


def sha256(path: Path, *, text: bool = False) -> str:
    data = path.read_bytes()
    if text:
        data = canonical_text_bytes(data)
    return hashlib.sha256(data).hexdigest()


def source(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def main() -> int:
    evidence = json.loads(EVIDENCE.read_text(encoding="utf-8"))
    if evidence.get("schema") != "ddn.creator_gaji_round4_fixed64_boundary.evidence.v1":
        return fail("E_R4_SCHEMA", repr(evidence.get("schema")))

    core_fixed = source("core/src/fixed64.rs")
    for forbidden in ("from_f64_lossy", "SCALE_F64", "FIXED64_LINT_ALLOW"):
        if forbidden in core_fixed:
            return fail("E_R4_CORE_FLOAT_API", forbidden)
    for required in ("pub fn from_ratio", "pub fn parse_decimal", "fn parse_decimal_raw"):
        if required not in core_fixed:
            return fail("E_R4_CORE_DECIMAL_API", required)

    for path in (ROOT / "core" / "src").rglob("*.rs"):
        if path.name == "fixed64_lint_gate.rs":
            continue
        text = path.read_text(encoding="utf-8")
        if FLOAT_TOKEN.search(text) or "FIXED64_LINT_ALLOW" in text:
            return fail("E_R4_CORE_FLOAT_TOKEN", str(path.relative_to(ROOT)))

    lang_text = "\n".join(path.read_text(encoding="utf-8") for path in (ROOT / "lang" / "src").rglob("*.rs"))
    if "from_f64_lossy" in lang_text or "SCALE_F64" in lang_text or FLOAT_TOKEN.search(lang_text):
        return fail("E_R4_LANG_FLOAT_BOUNDARY", "lang still owns a binary float conversion")
    for path, required in {
        "lang/src/parser.rs": "Fixed64::parse_decimal(raw)",
        "lang/src/runtime.rs": "Fixed64::parse_decimal(trimmed)",
        "tool/src/main.rs": "Fixed64::parse_decimal(text)",
        "tool/src/ddn_runtime.rs": "Fixed64::parse_decimal",
        "tool/src/wasm_api.rs": "JsonValue::String(s) => Fixed64::parse_decimal(s)",
    }.items():
        if required not in source(path):
            return fail("E_R4_DECIMAL_PATH", f"{path}: {required}")

    adapter = source("tool/src/fixed64_boundary.rs")
    for required in ("trait Fixed64FloatBoundary", "const SCALE_F64", "fn from_f64_lossy"):
        if required not in adapter:
            return fail("E_R4_TOOL_ADAPTER", required)
    for path in ("tool/src/ddn_runtime.rs", "tool/src/main.rs", "tool/src/wasm_api.rs"):
        if "Fixed64FloatBoundary" not in source(path):
            return fail("E_R4_TOOL_ADAPTER_IMPORT", path)
    nurigym = source("tool/src/nurigym_workflow.rs")
    if "Fixed64::from_f64_lossy" in nurigym or "fn fixed64_from_reward_weight(value: f64)" not in nurigym:
        return fail("E_R4_TEUL_BOUNDARY", "reward weight boundary not explicit")

    vectors = evidence.get("determinism_vectors", {})
    if vectors.get("determinism_vector_v1") != [
        6442450944,
        2147483648,
        -2147483648,
        2147483648,
        8589934592,
        2147483648,
        0,
    ]:
        return fail("E_R4_DETERMINISM_VECTOR", repr(vectors.get("determinism_vector_v1")))
    if vectors.get("decimal_raw", {}).get("3.14") != 13486197309:
        return fail("E_R4_DECIMAL_VECTOR", repr(vectors.get("decimal_raw")))
    if vectors.get("ratio_raw", {}).get("27315/100") != 1173170316902:
        return fail("E_R4_RATIO_VECTOR", repr(vectors.get("ratio_raw")))

    state_hash = evidence.get("state_hash_invariant", {})
    if state_hash.get("result") != "PASS" or state_hash.get("committed_fixture_tests") != 12:
        return fail("E_R4_STATE_HASH", repr(state_hash))
    tests = evidence.get("tests", [])
    if len(tests) != 8 or any(row.get("result") != "PASS" for row in tests):
        return fail("E_R4_TESTS", repr(tests))

    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    artifact = evidence.get("wasm_artifact", {})
    comparisons = {
        "source_file_count": manifest.get("source_file_count"),
        "source_tree_sha256": manifest.get("source_tree_sha256"),
        "wasm_sha256": sha256(WASM),
        "wasm_size": WASM.stat().st_size,
        "js_binding_sha256": sha256(JS_BINDING, text=True),
        "manifest_sha256": sha256(MANIFEST, text=True),
    }
    for key, actual in comparisons.items():
        if artifact.get(key) != actual:
            return fail("E_R4_WASM_ARTIFACT", f"{key}: {artifact.get(key)!r} != {actual!r}")
    if manifest.get("wasm_sha256") != artifact.get("wasm_sha256"):
        return fail("E_R4_WASM_MANIFEST", repr(manifest.get("wasm_sha256")))
    for key in ("source_hash_mode", "js_binding_hash_mode", "wasm_hash_mode"):
        if manifest.get(key) != artifact.get(key):
            return fail("E_R4_WASM_HASH_MODE", f"{key}: {manifest.get(key)!r}")

    claims = evidence.get("claims", {})
    for key, expected in {
        "r4_complete": True,
        "r0_through_r7_complete": False,
        "overall_goal_complete": False,
        "external_creator_pilot_success_count": 0,
        "docs_ssot_change": False,
    }.items():
        if claims.get(key) != expected:
            return fail("E_R4_CLAIM", f"{key}={claims.get(key)!r}")
    ssot = subprocess.run(
        ["git", "status", "--short", "--", "docs/ssot"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=60,
    )
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_R4_SSOT_DIRTY", ssot.stdout.strip())
    print("[creator-gaji-round4-fixed64-boundary-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
