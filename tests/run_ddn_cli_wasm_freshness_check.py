#!/usr/bin/env python
from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
WASM_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm"
MANIFEST = WASM_DIR / "ddonirang_tool.manifest.detjson"
JS_BINDING = WASM_DIR / "ddonirang_tool.js"
WASM_BINARY = WASM_DIR / "ddonirang_tool_bg.wasm"

SOURCE_ROOTS = [
    "core/src",
    "lang/src",
    "numeric/src",
    "proof/src",
    "symbolic/src",
    "tool/src",
]
SOURCE_FILES = [
    "Cargo.toml",
    "Cargo.lock",
    "core/Cargo.toml",
    "lang/Cargo.toml",
    "lang/src/dialect_keywords_v8.tsv",
    "numeric/Cargo.toml",
    "proof/Cargo.toml",
    "symbolic/Cargo.toml",
    "tool/Cargo.toml",
    "scripts/build_wasm_tool.ps1",
    "scripts/copy_wasm_tool_to_ui.ps1",
]
REQUIRED_EXPORTS = [
    "DdnWasmVm",
    "wasm_build_info",
    "wasm_canon_ddn",
    "wasm_preprocess_source",
]
SCHEMA = "ddn.wasm_artifact_manifest.v1"
TEXT_HASH_MODE = "crlf_to_lf_then_sha256"
WASM_HASH_MODE = "raw_bytes_sha256"
REPRODUCIBILITY_CONTRACT = {
    "required_build_entrypoint": "scripts/build_wasm_tool.ps1",
    "source_path_remap_destination": "/ddn-src",
    "direct_wasm_pack_without_source_remap": "unsupported_nonreproducible_environment",
}
BUILD_RECEIPT = None


def rel(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_text_bytes(data: bytes) -> bytes:
    return data.replace(b"\r\n", b"\n")


def sha256_text_file(path: Path) -> str:
    return sha256_bytes(canonical_text_bytes(path.read_bytes()))


def sha256_raw_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def collect_source_files() -> list[Path]:
    files: list[Path] = []
    for root_rel in SOURCE_ROOTS:
        root = ROOT / root_rel
        files.extend(path for path in root.rglob("*.rs") if path.is_file())
    for file_rel in SOURCE_FILES:
        path = ROOT / file_rel
        if path.exists():
            files.append(path)
    return sorted(set(files), key=lambda path: rel(path))


def source_fingerprint(files: list[Path]) -> tuple[str, list[dict[str, object]]]:
    rows: list[dict[str, object]] = []
    for path in files:
        data = canonical_text_bytes(path.read_bytes())
        rows.append(
            {
                "path": rel(path),
                "bytes": len(data),
                "sha256": sha256_bytes(data),
            }
        )
    payload = json.dumps(rows, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return sha256_bytes(payload), rows


def run_text(cmd: list[str]) -> str:
    if cmd[0] == "git":
        cmd = ["git", "-c", f"safe.directory={ROOT.as_posix()}", *cmd[1:]]
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        raise RuntimeError(f"{cmd!r} failed: {proc.stderr.strip() or proc.stdout.strip()}")
    return proc.stdout.strip()


def extract_exports(js_text: str) -> list[str]:
    out: list[str] = []
    for line in js_text.splitlines():
        match = re.match(r"export\s+(?:class|function)\s+([A-Za-z0-9_]+)\b", line.strip())
        if match:
            out.append(match.group(1))
    return sorted(out)


def extract_state_schema() -> str:
    text = (ROOT / "tool" / "src" / "wasm_api.rs").read_text(encoding="utf-8")
    match = re.search(r'ENGINE_RESPONSE_SCHEMA:\s*&str\s*=\s*"([^"]+)"', text)
    if not match:
        raise RuntimeError("ENGINE_RESPONSE_SCHEMA not found")
    return match.group(1)


def build_manifest() -> dict[str, object]:
    source_files = collect_source_files()
    tree_hash, source_rows = source_fingerprint(source_files)
    js_text = JS_BINDING.read_text(encoding="utf-8")
    if BUILD_RECEIPT is None:
        rustc_version = run_text(["rustc", "--version"])
        cargo_version = run_text(["cargo", "--version"])
        wasm_pack_version = run_text(["wasm-pack", "--version"])
    else:
        receipt = json.loads(BUILD_RECEIPT.read_text(encoding="utf-8"))
        payload = dict(receipt)
        claimed = payload.pop("semantic_identity")
        encoded = (json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode("utf-8")
        if claimed != "sha256:" + sha256_bytes(encoded):
            raise RuntimeError("build receipt identity mismatch")
        if receipt["source_commit"] != run_text(["git", "rev-parse", "HEAD"]) or receipt["source_tree"] != run_text(["git", "rev-parse", "HEAD^{tree}"]):
            raise RuntimeError("build receipt source mismatch")
        if run_text(["git", "status", "--porcelain", "--untracked-files=no"]):
            raise RuntimeError("build receipt source must be clean")
        for name in ("ddonirang_tool.js", "ddonirang_tool_bg.wasm"):
            artifact = WASM_DIR / name
            if receipt["artifacts"][name] != {"bytes": artifact.stat().st_size, "sha256": sha256_raw_file(artifact)}:
                raise RuntimeError(f"build receipt artifact mismatch: {name}")
        if receipt["profile"] != "release" or receipt["target"] != "web":
            raise RuntimeError("build receipt target mismatch")
        if any(receipt["path_scan"][key] != 0 for key in ("internal_absolute_path_count", "private_include_count", "official_store_path_count", "unknown_path_count")):
            raise RuntimeError("build receipt private paths remain")
        rustc_version = receipt["toolchain"]["rustc"]
        cargo_version = receipt["toolchain"]["cargo"]
        wasm_pack_version = receipt["toolchain"]["wasm_tool"]
    build_script = ROOT / "scripts" / "build_wasm_tool.ps1"
    copy_script = ROOT / "scripts" / "copy_wasm_tool_to_ui.ps1"
    build_text = build_script.read_text(encoding="utf-8")
    if "$cargoTargetRoot" not in build_text or "Join-Path $cargoTargetRoot" not in build_text:
        raise RuntimeError("WASM build input must resolve from CARGO_TARGET_DIR when configured")
    return {
        "schema": SCHEMA,
        "artifact_id": "ddonirang_tool.web.release",
        "source_roots": SOURCE_ROOTS,
        "source_hash_mode": TEXT_HASH_MODE,
        "source_file_count": len(source_rows),
        "source_tree_sha256": tree_hash,
        "source_files": source_rows,
        "metadata_text_hash_mode": TEXT_HASH_MODE,
        "cargo_lock_sha256": sha256_text_file(ROOT / "Cargo.lock"),
        "build_script_sha256": sha256_text_file(build_script),
        "copy_script_sha256": sha256_text_file(copy_script),
        "rustc_version": rustc_version,
        "cargo_version": cargo_version,
        "wasm_bindgen_or_pack_version": wasm_pack_version,
        "profile": "release",
        "target": "web",
        "reproducibility_contract": REPRODUCIBILITY_CONTRACT,
        "wasm_path": "solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool_bg.wasm",
        "js_binding_path": "solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool.js",
        "wasm_hash_mode": WASM_HASH_MODE,
        "wasm_sha256": sha256_raw_file(WASM_BINARY),
        "js_binding_hash_mode": TEXT_HASH_MODE,
        "js_binding_sha256": sha256_text_file(JS_BINDING),
        "wasm_size": WASM_BINARY.stat().st_size,
        "js_binding_size": len(canonical_text_bytes(JS_BINDING.read_bytes())),
        "js_wasm_reference": "ddonirang_tool_bg.wasm",
        "export_names": extract_exports(js_text),
        "state_schema": extract_state_schema(),
        "generated_by": {
            "tool": "tests/run_ddn_cli_wasm_freshness_check.py --write-manifest",
            "build_command": "powershell -ExecutionPolicy Bypass -File scripts\\build_wasm_tool.ps1 -Profile release -Target web",
            "copy_command": "scripts\\copy_wasm_tool_to_ui.ps1 -SourceRoot $env:DDN_BUILD_DIR\\wasm\\ddonirang_tool (requires explicit DDN_BUILD_DIR)",
        },
        "formula_change_claim": False,
        "closure_claim": "freshness_only_no_full_cli_wasm_equivalence",
    }


def validate_manifest(doc: dict[str, object], expected: dict[str, object]) -> list[str]:
    failures: list[str] = []
    for key in (
        "schema",
        "artifact_id",
        "source_roots",
        "source_hash_mode",
        "source_file_count",
        "source_tree_sha256",
        "metadata_text_hash_mode",
        "cargo_lock_sha256",
        "build_script_sha256",
        "copy_script_sha256",
        "rustc_version",
        "cargo_version",
        "wasm_bindgen_or_pack_version",
        "profile",
        "target",
        "reproducibility_contract",
        "wasm_path",
        "js_binding_path",
        "wasm_hash_mode",
        "wasm_sha256",
        "js_binding_hash_mode",
        "js_binding_sha256",
        "wasm_size",
        "js_binding_size",
        "js_wasm_reference",
        "export_names",
        "state_schema",
    ):
        if doc.get(key) != expected.get(key):
            failures.append(f"{key} mismatch")
    source_files = doc.get("source_files")
    if source_files != expected.get("source_files"):
        failures.append("source_files mismatch")
    generated_by = doc.get("generated_by")
    if not isinstance(generated_by, dict):
        failures.append("generated_by missing")
    else:
        for key in ("tool", "build_command", "copy_command"):
            if not str(generated_by.get(key, "")).strip():
                failures.append(f"generated_by.{key} missing")
    if "ddonirang_tool_bg.wasm" not in JS_BINDING.read_text(encoding="utf-8"):
        failures.append("JS binding does not reference ddonirang_tool_bg.wasm")
    exports = set(doc.get("export_names", []))
    for name in REQUIRED_EXPORTS:
        if name not in exports:
            failures.append(f"required export missing: {name}")
    if doc.get("formula_change_claim") is not False:
        failures.append("formula_change_claim must be false")
    if doc.get("closure_claim") != "freshness_only_no_full_cli_wasm_equivalence":
        failures.append("closure_claim overstates freshness scope")
    return failures


def run_negative_selftests(expected: dict[str, object]) -> list[str]:
    cases = {
        "source_change_fixture": {"source_tree_sha256": "0" * 64},
        "wasm_change_fixture": {"wasm_sha256": "1" * 64},
        "binding_change_fixture": {"js_binding_sha256": "2" * 64},
    }
    failures: list[str] = []
    for name, patch in cases.items():
        mutated = dict(expected)
        mutated.update(patch)
        if not validate_manifest(mutated, expected):
            failures.append(f"{name} unexpectedly passed")
    lf = b"first\nsecond\n"
    crlf = b"first\r\nsecond\r\n"
    if canonical_text_bytes(lf) != canonical_text_bytes(crlf):
        failures.append("line_ending_equivalence failed")
    if sha256_bytes(canonical_text_bytes(lf)) == sha256_bytes(canonical_text_bytes(b"first\nchanged\n")):
        failures.append("text_content_change unexpectedly preserved hash")
    return failures


def write_manifest(path: Path, doc: dict[str, object]) -> None:
    path.write_bytes((json.dumps(doc, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode("utf-8"))


def main() -> int:
    global ROOT, WASM_DIR, JS_BINDING, WASM_BINARY, BUILD_RECEIPT
    parser = argparse.ArgumentParser(description="Validate product WASM source/artifact freshness manifest")
    parser.add_argument("--write-manifest", action="store_true", help="rewrite manifest from current source/artifacts")
    parser.add_argument("--manifest", default=str(MANIFEST), help="manifest path")
    parser.add_argument("--source-root", type=Path, help="fixed build source checkout")
    parser.add_argument("--wasm-dir", type=Path, help="actual candidate artifact directory")
    parser.add_argument("--build-receipt", type=Path, help="sealed official remap build receipt")
    args = parser.parse_args()
    if args.source_root or args.wasm_dir or args.build_receipt:
        if not (args.source_root and args.wasm_dir and args.build_receipt):
            parser.error("source-root, wasm-dir and build-receipt must be provided together")
        ROOT = args.source_root.resolve(strict=True)
        WASM_DIR = args.wasm_dir.resolve(strict=True)
        JS_BINDING = WASM_DIR / "ddonirang_tool.js"
        WASM_BINARY = WASM_DIR / "ddonirang_tool_bg.wasm"
        BUILD_RECEIPT = args.build_receipt.resolve(strict=True)

    manifest_path = Path(args.manifest)
    expected = build_manifest()
    if args.write_manifest:
        write_manifest(manifest_path, expected)
        print(f"[ddn-cli-wasm-freshness] wrote {manifest_path}")
        return 0

    if not manifest_path.exists():
        print(f"[ddn-cli-wasm-freshness] fail: missing manifest: {manifest_path}")
        return 1
    try:
        doc = json.loads(manifest_path.read_text(encoding="utf-8"))
    except Exception as exc:
        print(f"[ddn-cli-wasm-freshness] fail: invalid manifest: {exc}")
        return 1

    failures = validate_manifest(doc, expected)
    failures.extend(run_negative_selftests(expected))
    if failures:
        for item in failures:
            print(f"[ddn-cli-wasm-freshness] fail: {item}")
        return 1
    print("[ddn-cli-wasm-freshness] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
