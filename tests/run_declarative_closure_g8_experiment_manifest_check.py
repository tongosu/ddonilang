#!/usr/bin/env python
from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BUILD = ROOT / "build" / "declarative_closure_g8"
PACK = ROOT / "pack" / "sim_experiment_manifest_v1"


def fail(message: str) -> None:
    print(f"declarative_closure_g8_experiment_manifest_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def run(cmd: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
        check=False,
    )


def read_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8-sig"))
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON {path.relative_to(ROOT)}: {exc}")


def canonical_hash(payload: object) -> str:
    data = json.dumps(
        payload,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    return "sha256:" + hashlib.sha256(data).hexdigest()


def file_hash(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "sim_experiment_manifest_v1"])
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_manifest() -> None:
    contract = read_json(PACK / "contract.detjson")
    manifest = read_json(PACK / "experiment_manifest.detjson")
    required_fields = contract.get("required_fields")
    if not isinstance(required_fields, list) or not required_fields:
        fail("contract required_fields is missing")
    missing = [field for field in required_fields if field not in manifest]
    if missing:
        fail(f"manifest missing required fields: {missing!r}")

    if contract.get("runtime_claim") is not False or contract.get("product_code_change") is not False:
        fail("G8 contract must remain fixture-only")
    if contract.get("state_hash_formula_change") is not False:
        fail("G8 contract must not claim state_hash formula changes")
    if contract.get("plan_hash_status") != "pending_canonical_bytes_decision":
        fail("G8 contract plan_hash_status mismatch")

    if manifest.get("schema") != "ddn.sim.experiment_manifest.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("plan_hash_status") != "pending_canonical_bytes_decision":
        fail("manifest plan_hash_status must remain pending")
    if "plan_hash" in manifest:
        fail("manifest must not define plan_hash before canonical bytes are decided")

    source = manifest.get("source", {})
    source_path = ROOT / source.get("path", "")
    got_source_hash = file_hash(source_path)
    if source.get("hash_kind") != "sha256_file_bytes":
        fail("source hash_kind must be sha256_file_bytes")
    if source.get("hash") != got_source_hash or manifest.get("source_hash") != got_source_hash:
        fail("source_hash does not match source file bytes")

    if canonical_hash(manifest.get("run_config")) != manifest.get("run_config_hash"):
        fail("run_config_hash does not match canonical run_config")
    if canonical_hash(manifest.get("input_snapshot")) != manifest.get("input_snapshot_hash"):
        fail("input_snapshot_hash does not match canonical input_snapshot")

    policy = manifest.get("state_hash_policy", {})
    if policy.get("formula_change") is not False:
        fail("state_hash_policy.formula_change must be false")
    if policy.get("state_hash_inputs_manifest_field_only") is not True:
        fail("state_hash_inputs must remain manifest-field-only")


def check_referenced_solveplan() -> None:
    manifest = read_json(PACK / "experiment_manifest.detjson")
    output = manifest.get("outputs", [])[0]
    actual_path = ROOT / output["path"]
    actual_path.parent.mkdir(parents=True, exist_ok=True)
    proc = run(
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            *manifest["run_config"]["args"],
        ],
        timeout=300,
    )
    if proc.returncode != 0:
        fail(f"referenced solveplan run failed:\n{proc.stdout}")
    expected = read_json(ROOT / output["expected_path"])
    actual = read_json(actual_path)
    if actual != expected:
        fail("referenced solveplan artifact is not byte-equivalent JSON to expected_init_s0.detjson")
    if actual.get("state_hash_inputs", {}).get("exclude") != ["diagnostics", "execution_results"]:
        fail("referenced solveplan state_hash_inputs boundary changed")


def check_no_docs_ssot_change() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed:\n{proc.stdout}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_pack_golden()
    check_manifest()
    check_referenced_solveplan()
    check_no_docs_ssot_change()
    print("declarative_closure_g8_experiment_manifest_check: ok")


if __name__ == "__main__":
    main()
