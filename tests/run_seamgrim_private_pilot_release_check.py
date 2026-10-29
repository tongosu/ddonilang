from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_private_pilot_release_contract_v1"
OUT = ROOT / "out" / "seamgrim_private_pilot_release_check"
BUILD_SCRIPT = ROOT / "solutions" / "seamgrim_ui_mvp" / "deploy" / "build_private_pilot_release.py"
FORBIDDEN_PREFIXES = (".git/", "build/", "docs/", "out/", "pack/", "tests/", "work/")
REQUIRED_FIELDS = {
    "source_commit",
    "source_tree_hash",
    "release_created_at",
    "python_version",
    "sqlite_version",
    "rust_toolchain",
    "teul_cli_sha256",
    "wasm_sha256",
    "wasm_binding_sha256",
    "static_asset_manifest_sha256",
    "project_store_schema",
    "deploy_config_sha256",
    "checker_summary",
}


def fail(message: str) -> None:
    raise SystemExit(f"[seamgrim-private-pilot-release] FAIL: {message}")


def run(args: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        args,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )
    if proc.returncode != 0:
        fail(f"{' '.join(args)} failed\nSTDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}")
    return proc


def git(args: list[str]) -> str:
    return run(["git", *args]).stdout.strip()


def load_json(path: Path) -> dict:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        fail(f"{path}: JSON root must be object")
    return data


def main() -> None:
    contract = load_json(PACK / "contract.detjson")
    if contract.get("schema") != "seamgrim.private_pilot.release_contract.v1":
        fail("contract schema mismatch")
    if set(contract.get("required_manifest_fields", [])) != REQUIRED_FIELDS:
        fail("contract required manifest field set mismatch")

    if OUT.exists():
        shutil.rmtree(OUT)
    proc = run([sys.executable, str(BUILD_SCRIPT), "--out", str(OUT), "--clean"], timeout=600)
    try:
        build_result = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        fail(f"build stdout is not JSON: {exc}: {proc.stdout!r}")
    manifest_path = Path(build_result.get("manifest", ""))
    if not manifest_path.is_file():
        fail(f"manifest missing: {manifest_path}")

    artifact_root = manifest_path.parent
    verify_script = artifact_root / "verify_private_pilot_release.py"
    if not verify_script.is_file():
        fail("artifact self verifier missing")
    verify_proc = run([sys.executable, str(verify_script), str(manifest_path)], timeout=120)
    if '"status": "PASS"' not in verify_proc.stdout:
        fail(f"self verifier did not report PASS: {verify_proc.stdout}")

    manifest = load_json(manifest_path)
    if manifest.get("schema") != "seamgrim.private_pilot.release_manifest.v1":
        fail("manifest schema mismatch")
    if manifest.get("source_commit") != git(["rev-parse", "HEAD"]):
        fail("source_commit does not match HEAD")
    if manifest.get("source_tree_hash") != git(["rev-parse", "HEAD^{tree}"]):
        fail("source_tree_hash does not match HEAD tree")
    if not REQUIRED_FIELDS.issubset(set(manifest)):
        fail("manifest missing required fields")

    paths = [str(row.get("path", "")).replace("\\", "/") for row in manifest.get("files", [])]
    if not paths:
        fail("manifest file list is empty")
    for rel in paths:
        if any(rel.startswith(prefix) for prefix in FORBIDDEN_PREFIXES):
            fail(f"forbidden path included in release artifact: {rel}")
        if "__pycache__" in rel or rel.endswith((".pyc", ".log", ".tmp")):
            fail(f"generated/local artifact included: {rel}")
    if "solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool_bg.wasm" not in paths:
        fail("wasm artifact missing")
    if "solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool.js" not in paths:
        fail("wasm binding missing")
    if not any(path.startswith("bin/teul-cli") for path in paths):
        fail("teul-cli binary missing")

    runtime_identity = manifest.get("runtime_identity")
    if not isinstance(runtime_identity, dict) or runtime_identity.get("user") != "seamgrim":
        fail("runtime identity must be non-root seamgrim")
    dependency_status = manifest.get("dependency_pin_status")
    if not isinstance(dependency_status, dict) or dependency_status.get("uses_latest_tag") is not False:
        fail("dependency status must reject floating latest tags")
    if dependency_status.get("digest_pin") != "verification_required":
        fail("base image digest must remain verification_required until operator-approved source exists")
    if manifest.get("release_filesystem_policy", {}).get("read_only_possible") is not True:
        fail("release filesystem must be read-only possible")
    if manifest.get("build_context_policy", {}).get("allowlist_only") is not True:
        fail("build context must be allowlist-only")

    dockerfile = (ROOT / "solutions" / "seamgrim_ui_mvp" / "deploy" / "Dockerfile").read_text(encoding="utf-8")
    if "USER seamgrim" not in dockerfile:
        fail("Dockerfile must switch runtime to non-root seamgrim user")
    if "latest" in dockerfile.lower():
        fail("Dockerfile must not use latest tags")

    pack_proc = run([sys.executable, "tests/run_pack_golden.py", "seamgrim_private_pilot_release_contract_v1"], timeout=900)
    if "pack golden ok" not in pack_proc.stdout:
        fail("pack golden did not report PASS")

    ssot_status = git(["status", "--short", "--", "docs/ssot"])
    if ssot_status.strip():
        fail(f"docs/ssot changed:\n{ssot_status}")
    print("[seamgrim-private-pilot-release] PASS")


if __name__ == "__main__":
    main()
