#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wasm_sealed_runtime_binding_v1"
RUNNER = ROOT / "tests" / "seamgrim_wasm_sealed_runtime_binding_runner.mjs"


def fail(message: str) -> int:
    print(f"[seamgrim-wasm-sealed-runtime-binding] fail: {message}")
    return 1


def run(cmd: list[str], *, timeout: int = 360) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )


def validate_report(report: dict, contract: dict) -> list[str]:
    failures: list[str] = []
    if report.get("schema") != "seamgrim.wasm.sealed_runtime_binding.report.v1":
        failures.append(f"bad report schema: {report.get('schema')}")
    if report.get("ok") is not True:
        failures.append("runner ok=false")
    if report.get("normal_case_count") != 16:
        failures.append(f"normal_case_count expected 16 got {report.get('normal_case_count')}")
    if report.get("worlds") != contract.get("worlds"):
        failures.append("world list mismatch")
    if report.get("viewports") != contract.get("viewports"):
        failures.append("viewport list mismatch")
    normal = report.get("normal")
    if not isinstance(normal, list) or len(normal) != 16:
        failures.append("normal rows missing")
    else:
        for row in normal:
            if row.get("artifact_id") != report.get("artifact_id"):
                failures.append(f"{row.get('lesson_id')}/{row.get('viewport')}: artifact id mismatch")
            if not str(row.get("fingerprint", "")).startswith("sha256:"):
                failures.append(f"{row.get('lesson_id')}/{row.get('viewport')}: fingerprint missing")
            if row.get("hash_title_has_manifest") is not True:
                failures.append(f"{row.get('lesson_id')}/{row.get('viewport')}: hash title missing manifest")
    for key in ("tamper", "js_content_tamper"):
        tamper = report.get(key) if isinstance(report.get(key), dict) else {}
        if tamper.get("diag_code") != "E_WASM_ARTIFACT_HASH_MISMATCH":
            failures.append(f"{key} diag mismatch: {tamper.get('diag_code')}")
        if tamper.get("error_text_includes_korean_verification") is not True:
            failures.append(f"{key} Korean diagnostic missing")
    non_claims = report.get("non_claims") if isinstance(report.get("non_claims"), dict) else {}
    if non_claims.get("state_hash_formula_change") is not False:
        failures.append("state_hash_formula_change must be false")
    if non_claims.get("full_cli_wasm_single_runtime_closure") is not False:
        failures.append("full single runtime closure must be false")
    return failures


def selftest(contract: dict) -> list[str]:
    base = {
        "schema": "seamgrim.wasm.sealed_runtime_binding.report.v1",
        "ok": True,
        "artifact_id": "ddonirang_tool.web.release",
        "normal_case_count": 16,
        "worlds": contract["worlds"],
        "viewports": contract["viewports"],
        "normal": [
            {
                "lesson_id": contract["worlds"][0],
                "viewport": contract["viewports"][0],
                "artifact_id": "ddonirang_tool.web.release",
                "fingerprint": "sha256:" + "0" * 64,
                "hash_title_has_manifest": True,
            }
            for _ in range(16)
        ],
        "tamper": {
            "diag_code": "E_WASM_ARTIFACT_HASH_MISMATCH",
            "error_text_includes_korean_verification": True,
        },
        "js_content_tamper": {
            "diag_code": "E_WASM_ARTIFACT_HASH_MISMATCH",
            "error_text_includes_korean_verification": True,
        },
        "non_claims": {
            "state_hash_formula_change": False,
            "full_cli_wasm_single_runtime_closure": False,
        },
    }

    def mutate(mutator):
        doc = json.loads(json.dumps(base, ensure_ascii=False))
        mutator(doc)
        return doc

    cases = {
        "missing_case": lambda doc: doc.__setitem__("normal_case_count", 15),
        "missing_fingerprint": lambda doc: doc["normal"][0].__setitem__("fingerprint", ""),
        "tamper_not_detected": lambda doc: doc["tamper"].__setitem__("diag_code", ""),
        "js_content_tamper_not_detected": lambda doc: doc["js_content_tamper"].__setitem__("diag_code", ""),
        "bad_nonclaim": lambda doc: doc["non_claims"].__setitem__("state_hash_formula_change", True),
    }
    failures: list[str] = []
    for name, mutator in cases.items():
        if not validate_report(mutate(mutator), contract):
            failures.append(f"selftest {name} unexpectedly passed")
    return failures


def main() -> int:
    required = [PACK / name for name in ("README.md", "contract.detjson", "input.ddn", "golden.jsonl")]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail(f"missing pack files: {missing}")
    if not RUNNER.exists():
        return fail(f"missing runner: {RUNNER}")
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "seamgrim.wasm.sealed_runtime_binding.contract.v1":
        return fail(f"bad contract schema: {contract.get('schema')}")
    commands = [
        (["node", "--check", str(RUNNER)], 60),
        (["node", "--no-warnings", str(RUNNER)], 600),
        (["python", "tests/run_pack_golden.py", "seamgrim_wasm_sealed_runtime_binding_v1"], 120),
    ]
    report: dict | None = None
    for cmd, timeout in commands:
        proc = run(cmd, timeout=timeout)
        if proc.returncode != 0:
            return fail(proc.stderr.strip() or proc.stdout.strip() or f"{cmd} rc={proc.returncode}")
        if cmd[0] == "node" and "--no-warnings" in cmd:
            try:
                report = json.loads(proc.stdout)
            except json.JSONDecodeError as exc:
                return fail(f"runner emitted invalid json: {exc}")
    assert report is not None
    failures = validate_report(report, contract)
    failures.extend(selftest(contract))
    if failures:
        return fail("; ".join(failures[:8]))
    print("[seamgrim-wasm-sealed-runtime-binding-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
