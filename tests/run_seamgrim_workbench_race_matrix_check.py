#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "tests" / "seamgrim_workbench_race_matrix_runner.mjs"
SUMMARY = ROOT / "out" / "seamgrim_workbench_g14" / "summary.detjson"
FRESHNESS = ROOT / "tests" / "run_ddn_cli_wasm_freshness_check.py"
OUT_DIR = ROOT / "out" / "seamgrim_workbench_g14"


REQUIRED_CASES = {
    "run_double_click_one_loop_one_run",
    "source_edit_during_live_preserves_sealed_run",
    "pause_resume_spam_no_duplicate_steps",
    "stop_stale_callback_and_finalize_retry_guard",
    "project_switch_during_live_drops_old_callbacks",
    "reopen_during_live_explicitly_clears_run_and_pump",
    "server_checkpoint_delay_failure_no_world_step_duplication",
    "string_comment_tick_surface_not_live",
    "legacy_selected_view_tamper_and_import_cancel_world_invariant",
    "overlay_magnifier_visibility_state_hash_invariant",
}

REQUIRED_RUNNER_TOKENS = [
    "api.startRun(), api.startRun()",
    "api.pauseRun(), api.pauseRun()",
    "api.resumeRun(), api.resumeRun()",
    "api.stopRun(), api.stopRun()",
    "E_G14_INJECTED_CHECKPOINT_FAILURE",
    "legacy-selected-view-tamper.seamgrim-project.json",
    "source_hash",
    "server_archive_status",
]


def run(cmd: list[str], *, expect_ok: bool = True, timeout: int = 180) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )
    if expect_ok and proc.returncode != 0:
        raise RuntimeError(
            "{} failed with {}\nSTDOUT:\n{}\nSTDERR:\n{}".format(
                " ".join(cmd),
                proc.returncode,
                proc.stdout[-4000:],
                proc.stderr[-4000:],
            )
        )
    if not expect_ok and proc.returncode == 0:
        raise RuntimeError("{} unexpectedly passed\nSTDOUT:\n{}".format(" ".join(cmd), proc.stdout[-4000:]))
    return proc


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # pragma: no cover - diagnostic path
        raise RuntimeError(f"{path.as_posix()} is not valid JSON: {exc}") from exc


def check_runner_static_contract() -> None:
    text = RUNNER.read_text(encoding="utf-8")
    missing = [token for token in REQUIRED_RUNNER_TOKENS if token not in text]
    require(not missing, f"runner missing required tokens: {missing}")
    require("child.kill()" in text and "SEAMGRIM_RUN_OWNER_ID" in text, "runner must own and stop its server process")


def check_summary() -> dict:
    require(SUMMARY.exists(), f"summary missing: {SUMMARY.as_posix()}")
    data = load_json(SUMMARY)
    require(data.get("schema") == "seamgrim.workbench_race_matrix.runner_result.v1", "summary schema mismatch")
    require("failure" not in data, f"runner recorded failure: {data.get('failure')}")
    cases = {str(row.get("id", "")) for row in data.get("cases", []) if isinstance(row, dict)}
    missing = sorted(REQUIRED_CASES - cases)
    require(not missing, f"summary missing cases: {missing}")
    require(data.get("browser_issues") == [], f"browser issues present: {data.get('browser_issues')}")
    require(int(data.get("api_faults", {}).get("delayed_failed_checkpoint_posts", 0)) >= 2, "checkpoint failure injection did not fire")
    return data


def write_json(path: Path, data: dict) -> None:
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def expect_freshness_failure(manifest: Path, label: str, expected_token: str) -> dict:
    proc = run([sys.executable, str(FRESHNESS.relative_to(ROOT)), "--manifest", str(manifest.relative_to(ROOT))], expect_ok=False, timeout=120)
    combined = f"{proc.stdout}\n{proc.stderr}"
    require(expected_token in combined, f"{label}: expected {expected_token!r} in freshness failure output, got:\n{combined[-2000:]}")
    return {
        "label": label,
        "returncode": proc.returncode,
        "expected_token": expected_token,
        "output_tail": combined[-1000:],
    }


def run_freshness_negative_controls() -> list[dict]:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    current = OUT_DIR / "freshness_current.detjson"
    run([sys.executable, str(FRESHNESS.relative_to(ROOT)), "--write-manifest", "--manifest", str(current.relative_to(ROOT))], timeout=120)
    base = load_json(current)

    stale_manifest = dict(base)
    stale_manifest["wasm_sha256"] = "0" * 64
    stale_path = OUT_DIR / "freshness_stale_wasm_manifest_negative.detjson"
    write_json(stale_path, stale_manifest)

    dirty_source = dict(base)
    dirty_source["source_tree_sha256"] = "1" * 64
    dirty_source_path = OUT_DIR / "freshness_dirty_source_tree_negative.detjson"
    write_json(dirty_source_path, dirty_source)

    return [
        expect_freshness_failure(stale_path, "stale_wasm_manifest_negative", "wasm_sha256 mismatch"),
        expect_freshness_failure(dirty_source_path, "dirty_source_tree_negative", "source_tree_sha256 mismatch"),
    ]


def check_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=30)
    require(proc.stdout.strip() == "", f"docs/ssot dirty:\n{proc.stdout}")


def main() -> int:
    check_runner_static_contract()
    run(["node", str(RUNNER.relative_to(ROOT))], timeout=180)
    summary = check_summary()
    freshness_negatives = run_freshness_negative_controls()
    check_docs_ssot_clean()
    report = {
        "schema": "seamgrim.workbench_race_matrix.wrapper_result.v1",
        "runner_cases": [row.get("id") for row in summary.get("cases", [])],
        "freshness_negative_controls": freshness_negatives,
        "docs_ssot_clean": True,
    }
    (OUT_DIR / "wrapper_summary.detjson").write_text(
        json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print("[seamgrim-workbench-race-matrix-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
