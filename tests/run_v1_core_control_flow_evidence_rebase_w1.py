from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PACK = ROOT / "pack" / "v1_core_control_flow_evidence_rebase_w1"
PROFILE = "v1-core-v25"
REQUIRED_IDS = (
    "CF01_CONDITION_TRUE_FALSE_ELSE",
    "CF02_WHILE_ZERO",
    "CF03_WHILE_ONE",
    "CF04_WHILE_MULTIPLE",
    "CF05_FOREACH_ZERO",
    "CF06_FOREACH_ONE",
    "CF07_FOREACH_MULTIPLE_ORDER",
    "CF08_NESTED_CURRENT_ITEM",
    "CF09_BRANCH_FAILURE_ATOMIC",
    "CF10_LOOP_FAILURE_ATOMIC",
)
FORBIDDEN_FAILURE_KEYS = {
    "output_log_texts",
    "output_rows",
    "resources",
    "patch",
    "state_hash",
    "public_observation_trace_hash",
}
FOCUSED_TESTS = (
    "natural_iteration_current_item_runs_through_existing_foreach_runtime",
    "foreach_scope_restores_or_removes_binding_for_container_matrix",
    "failed_foreach_run_keeps_world_hash_and_returns_no_committed_output",
)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def normalized_source_bytes(path: Path) -> bytes:
    return path.read_bytes().replace(b"\r\n", b"\n")


def canonical(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def pack_root() -> Path:
    override = os.environ.get("DDN_VC11_PACK_ROOT", "").strip()
    return Path(override).resolve() if override else DEFAULT_PACK


def fail(message: str) -> "None":
    raise SystemExit(message)


def load_manifest(*, require_receipts: bool) -> tuple[Path, dict[str, Any]]:
    pack = pack_root()
    manifest_path = pack / "manifest.detjson"
    if not manifest_path.is_file():
        fail(f"VC11_MANIFEST_MISSING path={manifest_path}")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("schema") != "ddn.v1_core_control_flow_evidence_rebase_w1.v1":
        fail("VC11_MANIFEST_SCHEMA_MISMATCH")
    if manifest.get("source_profile_identity") != PROFILE:
        fail("VC11_PROFILE_IDENTITY_MISMATCH")
    if manifest.get("repeat_count") != 3:
        fail("VC11_REPEAT_COUNT_MISMATCH")
    cases = manifest.get("cases")
    if not isinstance(cases, list) or len(cases) != len(REQUIRED_IDS):
        fail(f"VC11_CASE_COUNT_MISMATCH expected={len(REQUIRED_IDS)}")
    ids = tuple(case.get("id") for case in cases)
    if ids != REQUIRED_IDS:
        fail("VC11_CASE_ID_OR_ORDER_MISMATCH")
    for case in cases:
        if case.get("kind") not in {"success", "failure"} or case.get("ticks") != 1:
            fail(f"VC11_CASE_CONTRACT_INVALID id={case.get('id')}")
        fixture = pack / str(case.get("path", ""))
        if not fixture.is_file():
            fail(f"VC11_FIXTURE_MISSING id={case['id']}")
        actual_fixture_hash = sha256(normalized_source_bytes(fixture))
        if actual_fixture_hash != case.get("fixture_sha256"):
            fail(f"VC11_FIXTURE_HASH_MISMATCH id={case['id']}")
        receipt = pack / str(case.get("receipt", ""))
        if require_receipts:
            if not receipt.is_file():
                fail(f"VC11_RECEIPT_MISSING id={case['id']}")
            if sha256(normalized_source_bytes(receipt)) != case.get("receipt_sha256"):
                fail(f"VC11_RECEIPT_HASH_MISMATCH id={case['id']}")
    return pack, manifest


def rust_harness_source() -> str:
    return r'''use std::fs;
use std::io::{self, Read};
use ddonirang_tool::runtime_surface::run_summary_from_supported_source_profile;
use serde_json::{json, Value};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read request");
    let request: Value = serde_json::from_str(&input).expect("parse request");
    let repeat_count = request["repeat_count"].as_u64().expect("repeat_count");
    let mut out = Vec::new();
    for case in request["cases"].as_array().expect("cases") {
        let id = case["id"].as_str().expect("id");
        let path = case["absolute_path"].as_str().expect("absolute_path");
        let source_path = case["source_path"].as_str().expect("source_path");
        let profile = case["profile"].as_str().expect("profile");
        let ticks = case["ticks"].as_u64().expect("ticks") as u32;
        let source = fs::read_to_string(path)
            .expect("fixture utf8")
            .replace("\r\n", "\n");
        let mut runs = Vec::new();
        for _ in 0..repeat_count {
            let result = match run_summary_from_supported_source_profile(
                &source,
                source_path,
                Some(ticks),
                Some(profile),
            ) {
                Ok(payload) => json!({"ok": true, "payload": payload}),
                Err(raw) => {
                    let error = serde_json::from_str::<Value>(&raw)
                        .unwrap_or_else(|_| json!({"raw": raw}));
                    json!({"ok": false, "error": error})
                }
            };
            runs.push(result);
        }
        out.push(json!({"id": id, "runs": runs}));
    }
    println!("{}", json!({"schema": "ddn.vc11.native.actual.v1", "cases": out}));
}
'''


def run_native(pack: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    tool_path = (ROOT / "tool").resolve().as_posix()
    with tempfile.TemporaryDirectory(prefix="ddn-vc11-native-") as tmp_text:
        tmp = Path(tmp_text)
        (tmp / "src").mkdir()
        (tmp / "Cargo.toml").write_text(
            "[package]\nname = \"ddn-vc11-native-harness\"\nversion = \"0.0.0\"\nedition = \"2021\"\n"
            "[dependencies]\n"
            f"ddonirang-tool = {{ path = \"{tool_path}\" }}\n"
            "serde_json = \"1\"\n",
            encoding="utf-8",
        )
        (tmp / "src" / "main.rs").write_text(rust_harness_source(), encoding="utf-8")
        request = {
            "repeat_count": manifest["repeat_count"],
            "cases": [
                {
                    "id": case["id"],
                    "absolute_path": str((pack / case["path"]).resolve()),
                    "source_path": f"pack/v1_core_control_flow_evidence_rebase_w1/{case['path']}",
                    "profile": manifest["source_profile_identity"],
                    "ticks": case["ticks"],
                }
                for case in manifest["cases"]
            ],
        }
        env = os.environ.copy()
        env["CARGO_BUILD_JOBS"] = "1"
        completed = subprocess.run(
            ["cargo", "run", "--quiet", "--offline", "--manifest-path", str(tmp / "Cargo.toml")],
            cwd=ROOT,
            input=canonical(request),
            text=True,
            encoding="utf-8",
            capture_output=True,
            env=env,
        )
        if completed.returncode != 0:
            fail(f"VC11_NATIVE_HARNESS_FAILED exit={completed.returncode}\n{completed.stdout}{completed.stderr}")
        lines = [line for line in completed.stdout.splitlines() if line.strip()]
        if len(lines) != 1:
            fail(f"VC11_NATIVE_HARNESS_OUTPUT_COUNT expected=1 actual={len(lines)}")
        return json.loads(lines[0])


def forbidden_keys(value: Any) -> set[str]:
    found: set[str] = set()
    if isinstance(value, dict):
        for key, child in value.items():
            if key in FORBIDDEN_FAILURE_KEYS:
                found.add(key)
            found.update(forbidden_keys(child))
    elif isinstance(value, list):
        for child in value:
            found.update(forbidden_keys(child))
    return found


def receipt_projection(result: dict[str, Any]) -> dict[str, Any]:
    if result.get("ok") is False:
        error = result.get("error")
        if not isinstance(error, dict):
            fail("VC11_FAILURE_DIAGNOSTIC_MISSING")
        return {
            "ok": False,
            "diagnostic": {
                "code": error.get("code"),
                "phase": error.get("phase"),
                "source_path": error.get("source_path"),
                "source_span": error.get("source_span"),
                "offending_field": error.get("offending_field"),
            },
        }
    payload = result.get("payload")
    if not isinstance(payload, dict):
        fail("VC11_SUCCESS_PAYLOAD_MISSING")
    envelope = payload.get("execution_envelope")
    if not isinstance(envelope, dict):
        fail("VC11_EXECUTION_ENVELOPE_MISSING")
    work_cost = payload.get("work_cost_receipt")
    if not isinstance(work_cost, dict):
        fail("VC11_WORK_COST_RECEIPT_MISSING")
    return {
        "ok": True,
        "receipt": {
            "schema": payload.get("schema"),
            "source_profile_identity": payload.get("source_profile_identity"),
            "source_path": payload.get("source_path"),
            "ticks_run": payload.get("ticks_run"),
            "output_log_texts": payload.get("output_log_texts"),
            "state_hash": payload.get("state_hash"),
            "public_observation_trace_hash": payload.get("public_observation_trace_hash"),
            "canonical_ddn_sha256": envelope.get("canonical_ddn_sha256"),
            "preprocessed_source_sha256": envelope.get("preprocessed_source_sha256"),
            "source_bytes_sha256": envelope.get("source_bytes_sha256"),
            "work_cost_schema": work_cost.get("schema"),
            "work_cost_counters": work_cost.get("counters"),
        },
    }


def validate_actual(
    pack: Path,
    manifest: dict[str, Any],
    actual: dict[str, Any],
    *,
    compare_receipts: bool,
    enforce_outcomes: bool,
) -> dict[str, Any]:
    if actual.get("schema") != "ddn.vc11.native.actual.v1":
        fail("VC11_NATIVE_SCHEMA_MISMATCH")
    actual_cases = actual.get("cases")
    if not isinstance(actual_cases, list) or len(actual_cases) != len(REQUIRED_IDS):
        fail("VC11_NATIVE_CASE_COUNT_MISMATCH")
    projected: list[dict[str, Any]] = []
    for case, observed in zip(manifest["cases"], actual_cases, strict=True):
        if observed.get("id") != case["id"]:
            fail(f"VC11_NATIVE_CASE_ORDER_MISMATCH id={case['id']}")
        runs = observed.get("runs")
        if not isinstance(runs, list) or len(runs) != manifest["repeat_count"]:
            fail(f"VC11_NATIVE_REPEAT_MISMATCH id={case['id']}")
        first = runs[0]
        if any(run != first for run in runs[1:]):
            fail(f"VC11_NATIVE_NONDETERMINISTIC id={case['id']}")
        expected_ok = case["kind"] == "success"
        if enforce_outcomes and first.get("ok") is not expected_ok:
            fail(f"VC11_OUTCOME_MISMATCH id={case['id']}")
        if not expected_ok:
            leaked = forbidden_keys(first.get("error"))
            if leaked:
                fail(f"VC11_FAILURE_PARTIAL_PUBLISH id={case['id']} keys={sorted(leaked)}")
        receipt_value = receipt_projection(first)
        if compare_receipts:
            expected = json.loads((pack / case["receipt"]).read_text(encoding="utf-8"))
            if receipt_value != expected:
                fail(f"VC11_RECEIPT_MISMATCH id={case['id']}")
        projected.append({"id": case["id"], "result": first, "receipt": receipt_value})
    return {"schema": "ddn.vc11.native.fixed.v1", "cases": projected}


def run_checked(command: list[str], marker: str) -> str:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        capture_output=True,
    )
    output = completed.stdout + completed.stderr
    if completed.returncode != 0 or marker not in output:
        fail(f"VC11_SUPPLEMENTARY_FAILED marker={marker}\n{output}")
    return output


def run_supplementary() -> None:
    run_checked([sys.executable, "tests/run_lang_continue_skip_pack_check.py"], "cases=4")
    run_checked([sys.executable, "tests/run_lang_generic_iterable_order_contract_pack_check.py"], "cases=3")
    for name in FOCUSED_TESTS:
        output = run_checked(
            ["cargo", "test", "-p", "ddonirang-tool", "--lib", name],
            f"{name} ... ok",
        )
        if "1 passed" not in output or "running 0 tests" in output:
            fail(f"VC11_ZERO_MATCH_OR_COUNT_MISMATCH test={name}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--actual-json", action="store_true", help="print official native actuals without receipt comparison")
    parser.add_argument("--emit-native-json", action="store_true", help="print validated native fixed projection only")
    parser.add_argument("--skip-supplementary", action="store_true")
    args = parser.parse_args()

    require_receipts = not args.actual_json
    pack, manifest = load_manifest(require_receipts=require_receipts)
    actual = run_native(pack, manifest)
    projection = validate_actual(
        pack,
        manifest,
        actual,
        compare_receipts=require_receipts,
        enforce_outcomes=not args.actual_json,
    )
    if args.actual_json:
        print(json.dumps(
            {
                "schema": "ddn.vc11.receipt.capture.v1",
                "cases": [{"id": case["id"], "receipt": case["receipt"]} for case in projection["cases"]],
            },
            ensure_ascii=False,
            indent=2,
            sort_keys=True,
        ))
        return
    if not args.skip_supplementary:
        run_supplementary()
    if args.emit_native_json:
        print(canonical(projection))
        return
    success_count = sum(case["kind"] == "success" for case in manifest["cases"])
    failure_count = len(manifest["cases"]) - success_count
    print(
        "VC11_NATIVE_EVIDENCE_PASS "
        f"cases={len(manifest['cases'])} success={success_count} failure={failure_count} "
        f"deterministic_repeats={manifest['repeat_count']} partial_publish=0 "
        "supplementary_cases=10 supplementary_commands=5"
    )


if __name__ == "__main__":
    main()
