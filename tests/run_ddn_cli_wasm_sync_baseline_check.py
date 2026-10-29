#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "ddn_cli_wasm_sync_gate_v1"
BASELINE = PACK / "baseline.detjson"


def fail(message: str) -> int:
    print(f"[ddn-cli-wasm-sync-baseline] fail: {message}")
    return 1


def run_git(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> int:
    try:
        doc = json.loads(BASELINE.read_text(encoding="utf-8"))
    except Exception as exc:
        return fail(f"cannot load baseline: {exc}")

    try:
        require(doc.get("schema") == "ddn.cli_wasm_sync.baseline.v1", "bad schema")
        scope = doc.get("scope")
        require(isinstance(scope, dict), "missing scope")
        require(scope.get("branch") == "codex/queue-20260706", "bad branch record")
        require(scope.get("head") == "9bd663d", "bad baseline head")
        require(scope.get("closure_claim") == "baseline_only_no_single_runtime_closure", "bad closure claim")
        require(scope.get("state_hash_policy") == "report_only_at_g0", "bad state hash policy")

        ancestor = run_git(["merge-base", "--is-ancestor", str(scope["head"]), "HEAD"])
        require(ancestor.returncode == 0, "baseline head is not an ancestor of current HEAD")
        ssot = run_git(["status", "--short", "--", "docs/ssot"])
        require(ssot.returncode == 0, "git status docs/ssot failed")
        require(ssot.stdout.strip() == "", "docs/ssot has changes")

        commands = doc.get("commands")
        require(isinstance(commands, list), "missing command rows")
        require(len(commands) == 5, "G0 must record 5 baseline commands")
        pass_rows = [row for row in commands if row.get("actual") == "PASS"]
        fail_rows = [row for row in commands if row.get("actual") == "FAIL"]
        require(len(pass_rows) == 3, "G0 pass command count mismatch")
        require(len(fail_rows) == 2, "G0 fail command count mismatch")
        for row in pass_rows:
            require(row.get("returncode") == 0, f"{row.get('name')}: pass row returncode must be 0")
            require(row.get("expected") == "PASS", f"{row.get('name')}: pass row expected mismatch")
        for row in fail_rows:
            require(row.get("returncode") != 0, f"{row.get('name')}: fail row returncode must be nonzero")
            require(row.get("expected") == "FAIL", f"{row.get('name')}: fail row expected mismatch")
            require(row.get("failure_is_baseline") is True, f"{row.get('name')}: baseline failure flag missing")

        dirty = doc.get("dirty_snapshot")
        require(isinstance(dirty, dict), "missing dirty snapshot")
        require(dirty.get("status_count") == 125, "dirty status count drift")
        require(dirty.get("modified_count") == 10, "modified count drift")
        require(dirty.get("untracked_count") == 115, "untracked count drift")
        require(len(dirty.get("modified_paths", [])) == 10, "modified path list must cover all tracked dirty files")
        open_logs = [row for row in dirty["modified_paths"] if row.get("generated_open_log")]
        require(len(open_logs) == 2, "generated open log coverage mismatch")

        wasm = doc.get("wasm_baseline")
        require(isinstance(wasm, dict), "missing wasm baseline")
        require(wasm.get("last_checked_in_wasm_commit") == "8655649", "bad last wasm commit")
        require(wasm["product_js"]["sha256"] == wasm["tool_pkg_js"]["sha256"], "product/tool JS hash mismatch")
        require(wasm["product_wasm"]["sha256"] == wasm["tool_pkg_wasm"]["sha256"], "product/tool WASM hash mismatch")
        exports = set(wasm.get("js_exports", []))
        for name in ("DdnWasmVm", "wasm_build_info", "wasm_canon_ddn", "wasm_preprocess_source"):
            require(name in exports, f"missing baseline JS export: {name}")
        methods = set(wasm.get("wasm_api_methods", []))
        for name in ("get_state_hash", "get_state_json", "step_one_with_input", "run_ticks"):
            require(name in methods, f"missing baseline WASM API method: {name}")

        delta = doc.get("post_wasm_source_delta")
        require(isinstance(delta, dict), "missing post-WASM source delta")
        require(delta.get("base_commit") == "8655649", "bad source delta base commit")
        require(delta.get("changed_file_count") == 31, "post-WASM changed file count drift")
        require(len(delta.get("risk_classes", [])) >= 3, "risk class coverage underflow")
        require("baseline_only_no_single_runtime_closure" in json.dumps(doc, ensure_ascii=False), "bounded claim missing")
    except AssertionError as exc:
        return fail(str(exc))

    print("[ddn-cli-wasm-sync-baseline] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

