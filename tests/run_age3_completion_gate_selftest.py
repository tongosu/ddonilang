#!/usr/bin/env python
from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

REUSE_REPORT_ENV_KEY = "DDN_AGE3_COMPLETION_GATE_SELFTEST_REUSE_REPORT"
REPORT_PATH_ENV_KEY = "DDN_AGE3_COMPLETION_GATE_REPORT_JSON"
PACK_REPORT_PATH_ENV_KEY = "DDN_AGE3_COMPLETION_GATE_PACK_REPORT_JSON"


def fail(msg: str) -> int:
    print(f"[age3-completion-gate-selftest] fail: {msg}")
    return 1


def run_gate(report: Path, pack_report: Path) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_age3_completion_gate.py",
        "--report-out",
        str(report),
        "--pack-report-out",
        str(pack_report),
    ]
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def load_json(path: Path) -> dict:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError(f"json root must be object: {path}")
    return data


def env_path(key: str) -> Path | None:
    raw = str(os.environ.get(key, "")).strip()
    if not raw:
        return None
    return Path(raw)


def load_age3_gate_module(root: Path):
    script_path = root / "tests" / "run_age3_completion_gate.py"
    spec = importlib.util.spec_from_file_location("run_age3_completion_gate", script_path)
    if spec is None or spec.loader is None:
        raise ValueError(f"unable to load module spec: {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def verify_run_cmd_retry_semantics(root: Path) -> None:
    module = load_age3_gate_module(root)
    run_cmd = getattr(module, "run_cmd", None)
    if run_cmd is None:
        raise ValueError("run_age3_completion_gate.py missing run_cmd")

    with tempfile.TemporaryDirectory(prefix="age3_completion_gate_retry_") as tmp:
        tmp_path = Path(tmp)
        marker = tmp_path / "retry.marker"
        flaky_script = tmp_path / "flaky_once.py"
        flaky_script.write_text(
            "\n".join(
                [
                    "import pathlib",
                    "import sys",
                    "marker = pathlib.Path(sys.argv[1])",
                    "if not marker.exists():",
                    "    marker.write_text('1', encoding='utf-8')",
                    "    print('first-fail')",
                    "    sys.stderr.write('transient-fail\\n')",
                    "    raise SystemExit(9)",
                    "print('ok-second')",
                    "",
                ]
            ),
            encoding="utf-8",
        )
        rc, out, err = run_cmd(root, [sys.executable, str(flaky_script), str(marker)], "retry-semantics")
        if int(rc) != 0:
            raise ValueError(f"run_cmd retry success path must return 0, got={rc}")
        if "ok-second" not in out:
            raise ValueError(f"run_cmd retry success path missing rerun output: {out!r}")
        if err.strip():
            raise ValueError(f"run_cmd retry success path stderr should be empty: {err!r}")

        fail_script = tmp_path / "always_fail.py"
        fail_script.write_text(
            "\n".join(
                [
                    "import sys",
                    "print('always-fail')",
                    "sys.stderr.write('persistent-fail\\n')",
                    "raise SystemExit(4)",
                    "",
                ]
            ),
            encoding="utf-8",
        )
        rc_fail, out_fail, err_fail = run_cmd(root, [sys.executable, str(fail_script)], "retry-semantics-fail")
        if int(rc_fail) != 4:
            raise ValueError(f"run_cmd persistent failure must keep rerun rc=4, got={rc_fail}")
        if "always-fail" not in out_fail and "persistent-fail" not in err_fail:
            raise ValueError("run_cmd persistent failure must expose rerun output")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    try:
        verify_run_cmd_retry_semantics(root)
    except ValueError as exc:
        return fail(str(exc))

    with tempfile.TemporaryDirectory(prefix="age3_completion_gate_selftest_") as tmp:
        report = Path(tmp) / "age3_completion_gate.detjson"
        pack_report = Path(tmp) / "age3_completion_pack_report.detjson"

        reuse_report = str(os.environ.get(REUSE_REPORT_ENV_KEY, "")).strip() == "1"
        env_report = env_path(REPORT_PATH_ENV_KEY)
        env_pack_report = env_path(PACK_REPORT_PATH_ENV_KEY)
        if reuse_report and env_report is not None and env_report.exists():
            report = env_report
            if env_pack_report is not None:
                pack_report = env_pack_report
        else:
            proc = run_gate(report, pack_report)
            if proc.returncode != 0:
                return fail(f"gate failed out={proc.stdout} err={proc.stderr}")
        if not report.exists():
            return fail(f"report missing: {report}")

        doc = load_json(report)
        if str(doc.get("schema", "")) != "ddn.age3.completion_gate.v1":
            return fail(f"schema mismatch: {doc.get('schema')}")
        if not bool(doc.get("overall_ok", False)):
            return fail("overall_ok must be true")

        criteria = doc.get("criteria")
        if not isinstance(criteria, list):
            return fail("criteria must be list")
        criteria_map: dict[str, bool] = {}
        for row in criteria:
            if not isinstance(row, dict):
                continue
            name = str(row.get("name", "")).strip()
            if not name:
                continue
            criteria_map[name] = bool(row.get("ok", False))

        required = [
            "age3_ssot_walk_pack_contract_sync",
            "gogae7_pack_set_pass",
            "bogae_backend_profile_smoke_pass",
            "lang_teulcli_parser_parity_selftest_pass",
            "diag_fixit_selftest_pass",
            "lang_maegim_smoke_pack_pass",
            "lang_unit_temp_smoke_pack_pass",
            "gate0_contract_abort_state_check_pass",
            "block_editor_roundtrip_check_pass",
            "seamgrim_vol4_runtime_track_check_pass",
            "seamgrim_wasm_canon_contract_check_pass",
            "proof_runtime_minimum_check_pass",
            "seamgrim_wasm_web_smoke_contract_pass",
            "seamgrim_wasm_web_step_check_pass",
            "bogae_geoul_visibility_smoke_pass",
            "external_intent_boundary_pack_pass",
            "seulgi_v1_pack_pass",
            "sam_inputsnapshot_contract_pack_pass",
            "sam_ai_ordering_pack_pass",
            "seulgi_gatekeeper_pack_pass",
            "external_intent_seulgi_walk_alignment_pass",
            "age3_doc_paths_exist",
        ]
        missing = [name for name in required if name not in criteria_map]
        if missing:
            return fail(f"missing criteria: {missing}")
        failed = [name for name in required if not criteria_map.get(name, False)]
        if failed:
            return fail(f"criteria must pass: {failed}")
        failure_codes = doc.get("failure_codes")
        if not isinstance(failure_codes, list):
            return fail("failure_codes must be list")

        sync = doc.get("ssot_walk_pack_contract_sync")
        if not isinstance(sync, dict):
            return fail("ssot_walk_pack_contract_sync must be object")
        if sync.get("missing_docs") not in ([], tuple()):
            return fail(f"ssot missing docs: {sync.get('missing_docs')}")
        if sync.get("unresolved_patterns") not in ([], tuple()):
            return fail(f"ssot unresolved patterns: {sync.get('unresolved_patterns')}")
        if sync.get("unresolved_walks") not in ([], tuple()):
            return fail(f"ssot unresolved walks: {sync.get('unresolved_walks')}")
        if sync.get("missing_walks_from_gate") not in ([], tuple()):
            return fail(f"ssot missing walks from gate: {sync.get('missing_walks_from_gate')}")
        if sync.get("missing_packs_from_gate_candidates") not in ([], tuple()):
            return fail(
                f"ssot missing packs from gate candidates: {sync.get('missing_packs_from_gate_candidates')}"
            )
        required_packs = sync.get("required_packs")
        if not isinstance(required_packs, list) or not required_packs:
            return fail("required_packs must be non-empty list")

        shard_policy = doc.get("shard_policy")
        if not isinstance(shard_policy, dict):
            return fail("shard_policy must be object")
        lookback = shard_policy.get("lookback")
        source_reports_used = shard_policy.get("source_reports_used")
        requested = shard_policy.get("requested")
        effective = shard_policy.get("effective")
        if not isinstance(requested, dict) or not isinstance(effective, dict):
            return fail("shard_policy requested/effective must be object")
        if int(lookback or 0) <= 0:
            return fail("shard_policy lookback must be positive")
        if source_reports_used is None or int(source_reports_used) < 0:
            return fail("shard_policy source_reports_used must be non-negative")
        if int(effective.get("gogae7_packs", 0)) <= 0:
            return fail("shard_policy effective gogae7_packs must be positive")
        gogae7_pack_shards = doc.get("gogae7_pack_shards")
        if not isinstance(gogae7_pack_shards, list) or not gogae7_pack_shards:
            return fail("gogae7_pack_shards must be non-empty list")
        for row in gogae7_pack_shards:
            if not isinstance(row, dict):
                return fail("gogae7_pack_shards row must be object")
            if int(row.get("shard_index", 0)) <= 0:
                return fail("gogae7_pack_shards shard_index must be positive")
            if not isinstance(row.get("packs"), list):
                return fail("gogae7_pack_shards packs must be list")
        pack_report_doc = str(doc.get("gogae7_pack_report_path", "")).strip()
        if pack_report_doc:
            pack_report = Path(pack_report_doc)
        if not pack_report.exists():
            return fail(f"pack report missing: {pack_report}")

        smoke_report_path_text = str(doc.get("bogae_geoul_visibility_smoke_report_path", "")).strip()
        if not smoke_report_path_text:
            return fail("bogae_geoul_visibility_smoke_report_path missing")
        smoke_report_path = Path(smoke_report_path_text)
        if not smoke_report_path.exists():
            return fail(f"bogae geoul smoke report missing: {smoke_report_path}")
        expected_suffix = ".bogae_geoul_visibility_smoke.detjson"
        if not smoke_report_path.name.endswith(expected_suffix):
            return fail(f"unexpected smoke report name: {smoke_report_path.name}")
        if not smoke_report_path.name.startswith(report.stem):
            return fail(
                f"smoke report name should start with gate report stem: "
                f"gate={report.stem} smoke={smoke_report_path.name}"
            )
        smoke_doc = load_json(smoke_report_path)
        if str(smoke_doc.get("schema", "")) != "ddn.bogae_geoul_visibility_smoke.v1":
            return fail(f"smoke schema mismatch: {smoke_doc.get('schema')}")
        if not bool(smoke_doc.get("overall_ok", False)):
            return fail("bogae geoul smoke overall_ok must be true")
        smoke_checks = smoke_doc.get("checks")
        if not isinstance(smoke_checks, list) or not smoke_checks:
            return fail("bogae geoul smoke checks must be non-empty list")
        smoke_check_map: dict[str, bool] = {}
        for row in smoke_checks:
            if not isinstance(row, dict):
                continue
            name = str(row.get("name", "")).strip()
            if not name:
                continue
            smoke_check_map[name] = bool(row.get("ok", False))
        smoke_required_checks = [
            "static_teul_cli_run_ok",
            "static_viewer_manifest_has_frames",
            "sim_teul_cli_run_ok",
            "sim_viewer_manifest_has_frames",
            "sim_frame_count_min_2",
            "sim_state_hash_changes",
            "sim_bogae_hash_changes",
        ]
        smoke_missing_checks = [name for name in smoke_required_checks if name not in smoke_check_map]
        if smoke_missing_checks:
            return fail(f"bogae geoul smoke missing required checks: {smoke_missing_checks}")
        smoke_failed_checks = [name for name in smoke_required_checks if not smoke_check_map.get(name, False)]
        if smoke_failed_checks:
            return fail(f"bogae geoul smoke required checks failed: {smoke_failed_checks}")

        step_report_path_text = str(doc.get("seamgrim_wasm_web_step_check_report_path", "")).strip()
        if not step_report_path_text:
            return fail("seamgrim_wasm_web_step_check_report_path missing")
        step_report_path = Path(step_report_path_text)
        if not step_report_path.exists():
            return fail(f"seamgrim wasm/web step report missing: {step_report_path}")
        expected_step_suffix = ".seamgrim_wasm_web_step_check.detjson"
        if not step_report_path.name.endswith(expected_step_suffix):
            return fail(f"unexpected step report name: {step_report_path.name}")
        if not step_report_path.name.startswith(report.stem):
            return fail(
                f"step report name should start with gate report stem: "
                f"gate={report.stem} step={step_report_path.name}"
            )
        step_doc = load_json(step_report_path)
        if str(step_doc.get("schema", "")) != "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1":
            return fail(f"seamgrim wasm/web step schema mismatch: {step_doc.get('schema')}")
        if not bool(step_doc.get("ok", False)):
            return fail("seamgrim wasm/web step report ok must be true")
        if str(step_doc.get("status", "")).strip() != "pass":
            return fail(f"seamgrim wasm/web step status must be pass: {step_doc.get('status')}")
        if str(step_doc.get("code", "")).strip() != "OK":
            return fail(f"seamgrim wasm/web step code must be OK: {step_doc.get('code')}")
        try:
            checked_files = int(step_doc.get("checked_files", -1))
        except Exception:
            checked_files = -1
        if checked_files < 20:
            return fail(f"seamgrim wasm/web step checked_files too small: {checked_files}")
        if int(step_doc.get("missing_count", -1)) != 0:
            return fail(f"seamgrim wasm/web step missing_count must be 0: {step_doc.get('missing_count')}")

    print("[age3-completion-gate-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
