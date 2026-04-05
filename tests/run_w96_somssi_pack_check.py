#!/usr/bin/env python
from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

PROGRESS_ENV_KEY = "DDN_W96_SOMSSI_PACK_CHECK_PROGRESS_JSON"


def fail(code: str, msg: str) -> int:
    print(f"[w96-pack-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def write_progress_snapshot(
    path_text: str,
    *,
    status: str,
    current_case: str,
    last_completed_case: str,
    current_probe: str,
    last_completed_probe: str,
    total_elapsed_ms: int,
) -> None:
    if not str(path_text).strip():
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.w96_somssi_pack_check.progress.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": status,
        "current_case": current_case,
        "last_completed_case": last_completed_case,
        "current_probe": current_probe,
        "last_completed_probe": last_completed_probe,
        "total_elapsed_ms": int(total_elapsed_ms),
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def load_json(path: Path) -> dict:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise ValueError(f"missing file: {path}")
    except Exception as exc:
        raise ValueError(f"invalid json: {path} ({exc})")
    if not isinstance(data, dict):
        raise ValueError(f"json root must be object: {path}")
    return data


def expected_sim_state_hash(adapter_id: str, sim_seed: str) -> str:
    raw = f"{adapter_id}|{sim_seed}|ticks=16|model=v1".encode("utf-8")
    return "sha256:" + hashlib.sha256(raw).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description="W96 somssi-hub pack contract checker")
    parser.add_argument(
        "--pack",
        default="pack/gogae9_w96_somssi_hub",
        help="pack directory path",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    progress_path = str(os.environ.get(PROGRESS_ENV_KEY, "")).strip()
    started_at = time.perf_counter()
    current_case = "-"
    last_completed_case = "-"
    current_probe = "-"
    last_completed_probe = "-"

    def update_progress(status: str) -> None:
        write_progress_snapshot(
            progress_path,
            status=status,
            current_case=current_case,
            last_completed_case=last_completed_case,
            current_probe=current_probe,
            last_completed_probe=last_completed_probe,
            total_elapsed_ms=int((time.perf_counter() - started_at) * 1000),
        )

    def start_case(name: str) -> None:
        nonlocal current_case, current_probe
        current_case = name
        current_probe = "-"
        update_progress("running")

    def complete_case(name: str) -> None:
        nonlocal current_case, current_probe, last_completed_case
        current_case = "-"
        current_probe = "-"
        last_completed_case = name
        update_progress("running")

    def start_probe(name: str) -> None:
        nonlocal current_probe
        current_probe = name
        update_progress("running")

    def complete_probe(name: str) -> None:
        nonlocal current_probe, last_completed_probe
        current_probe = "-"
        last_completed_probe = name
        update_progress("running")

    update_progress("running")
    pack = Path(args.pack)
    if not pack.is_absolute():
        pack = root / pack

    start_case("validate.required_files")
    start_probe("collect_required_files")
    required_files = [
        pack / "README.md",
        pack / "intent.md",
        pack / "adapter_registry.json",
        pack / "golden.detjson",
        pack / "golden.jsonl",
        pack / "inputs" / "c00_contract_anchor" / "input.ddn",
        pack / "inputs" / "c00_contract_anchor" / "expected_canon.ddn",
    ]
    complete_probe("collect_required_files")
    start_probe("validate_required_files")
    missing = [str(path).replace("\\", "/") for path in required_files if not path.exists()]
    if missing:
        return fail("E_W96_PACK_FILE_MISSING", ",".join(missing))
    complete_probe("validate_required_files")
    complete_case("validate.required_files")

    start_case("validate.readme_tokens")
    start_probe("load_readme")
    readme_text = (pack / "README.md").read_text(encoding="utf-8")
    complete_probe("load_readme")
    start_probe("validate_readme_tokens")
    for token in (
        "Pack ID: `pack/gogae9_w96_somssi_hub`",
        "adapter_registry.json",
        "golden.detjson",
        "golden.jsonl",
    ):
        if token not in readme_text:
            return fail("E_W96_README_TOKEN_MISSING", token)
    complete_probe("validate_readme_tokens")
    complete_case("validate.readme_tokens")

    start_case("validate.registry_doc")
    start_probe("load_registry_doc")
    try:
        registry = load_json(pack / "adapter_registry.json")
    except ValueError as exc:
        return fail("E_W96_REGISTRY_JSON_INVALID", str(exc))
    complete_probe("load_registry_doc")
    start_probe("validate_registry_doc")
    if str(registry.get("schema", "")).strip() != "ddn.gogae9.w96.adapter_registry.v1":
        return fail("E_W96_REGISTRY_SCHEMA", f"schema={registry.get('schema')}")

    adapters = registry.get("adapters")
    if not isinstance(adapters, list) or not adapters:
        return fail("E_W96_REGISTRY_EMPTY", "adapters must be non-empty list")
    if len(adapters) < 100:
        return fail("E_W96_REGISTRY_COUNT", f"adapter_count={len(adapters)} expected>=100")

    ids: list[str] = []
    sim_rows: list[dict] = []
    for idx, row in enumerate(adapters, 1):
        if not isinstance(row, dict):
            return fail("E_W96_REGISTRY_ROW_INVALID", f"index={idx} type={type(row).__name__}")
        adapter_id = str(row.get("id", "")).strip()
        if not adapter_id:
            return fail("E_W96_REGISTRY_ID_MISSING", f"index={idx}")
        ids.append(adapter_id)
        kind = str(row.get("kind", "")).strip()
        if kind not in {"sim", "stub", "live"}:
            return fail("E_W96_REGISTRY_KIND_INVALID", f"id={adapter_id} kind={kind}")
        open_mode = str(row.get("open_mode", "")).strip()
        if open_mode not in {"recorded", "replay", "deny"}:
            return fail("E_W96_REGISTRY_OPEN_MODE", f"id={adapter_id} open_mode={open_mode}")
        record_required = bool(row.get("record_required", False))
        if kind == "live":
            if not record_required or open_mode != "recorded":
                return fail(
                    "E_SOMSSI_OUTSIDE_EFFECT",
                    f"id={adapter_id} kind=live record_required={record_required} open_mode={open_mode}",
                )
        if kind == "sim":
            sim_rows.append(row)

    if ids != sorted(ids):
        return fail("E_SOMSSI_REGISTRY_DUP", "adapter ids must be sorted")
    if len(set(ids)) != len(ids):
        return fail("E_SOMSSI_REGISTRY_DUP", "adapter ids must be unique")

    if len(sim_rows) < 10:
        return fail("E_W96_SIM_COUNT", f"sim_adapter_count={len(sim_rows)} expected>=10")
    complete_probe("validate_registry_doc")
    complete_case("validate.registry_doc")

    expected_hash_by_id: dict[str, str] = {}
    for row in sim_rows:
        adapter_id = str(row.get("id", "")).strip()
        start_case(f"case.{adapter_id or 'unknown'}")
        start_probe("load_case_row")
        adapter_id = str(row.get("id", "")).strip()
        sim_seed = str(row.get("sim_seed", "")).strip()
        expected = str(row.get("expected_state_hash", "")).strip()
        if not sim_seed:
            return fail("E_W96_SIM_SEED_MISSING", f"id={adapter_id}")
        if not expected.startswith("sha256:"):
            return fail("E_W96_SIM_HASH_FORMAT", f"id={adapter_id} expected_state_hash={expected}")
        complete_probe("load_case_row")
        start_probe("validate_case_row")
        recomputed = expected_sim_state_hash(adapter_id, sim_seed)
        if expected != recomputed:
            return fail(
                "E_W96_SIM_HASH_MISMATCH",
                f"id={adapter_id} expected={expected} recomputed={recomputed}",
            )
        expected_hash_by_id[adapter_id] = expected
        complete_probe("validate_case_row")
        complete_case(f"case.{adapter_id}")

    start_case("validate.golden_doc")
    start_probe("load_golden_doc")
    try:
        golden = load_json(pack / "golden.detjson")
    except ValueError as exc:
        return fail("E_W96_GOLDEN_JSON_INVALID", str(exc))
    complete_probe("load_golden_doc")
    start_probe("validate_golden_doc")
    if str(golden.get("schema", "")).strip() != "ddn.gogae9.w96.somssi_report.v1":
        return fail("E_W96_GOLDEN_SCHEMA", f"schema={golden.get('schema')}")
    if not bool(golden.get("overall_pass", False)):
        return fail("E_W96_GOLDEN_NOT_PASS", "overall_pass must be true")
    if not bool(golden.get("deterministic_registry_order", False)):
        return fail("E_W96_GOLDEN_ORDER_FLAG", "deterministic_registry_order=false")
    if not bool(golden.get("live_recorded_only", False)):
        return fail("E_W96_GOLDEN_LIVE_FLAG", "live_recorded_only=false")

    golden_count = int(golden.get("adapter_count", -1))
    if golden_count != len(adapters):
        return fail("E_W96_GOLDEN_COUNT", f"golden.adapter_count={golden_count} actual={len(adapters)}")
    golden_sim_count = int(golden.get("sim_adapter_count", -1))
    if golden_sim_count != len(sim_rows):
        return fail(
            "E_W96_GOLDEN_SIM_COUNT",
            f"golden.sim_adapter_count={golden_sim_count} actual={len(sim_rows)}",
        )
    complete_probe("validate_golden_doc")

    start_probe("validate_golden_cases")
    golden_rows = golden.get("cases")
    if not isinstance(golden_rows, list) or not golden_rows:
        return fail("E_W96_GOLDEN_CASES_EMPTY", "cases must be non-empty list")
    if len(golden_rows) != len(sim_rows):
        return fail(
            "E_W96_GOLDEN_CASES_COUNT",
            f"golden_cases={len(golden_rows)} sim_rows={len(sim_rows)}",
        )
    for idx, row in enumerate(golden_rows, 1):
        if not isinstance(row, dict):
            return fail("E_W96_GOLDEN_CASE_ROW_INVALID", f"index={idx} type={type(row).__name__}")
        adapter_id = str(row.get("id", "")).strip()
        if not adapter_id:
            return fail("E_W96_GOLDEN_CASE_ID_MISSING", f"index={idx}")
        if not bool(row.get("state_hash_equal", False)):
            return fail("E_W96_GOLDEN_CASE_FLAG_FAIL", f"id={adapter_id}")
        expected = str(row.get("expected_state_hash", "")).strip()
        if expected != expected_hash_by_id.get(adapter_id, ""):
            return fail(
                "E_W96_GOLDEN_CASE_HASH_MISMATCH",
                f"id={adapter_id} golden={expected} cases={expected_hash_by_id.get(adapter_id, '')}",
            )
    complete_probe("validate_golden_cases")
    complete_case("validate.golden_doc")

    start_case("validate.golden_jsonl")
    start_probe("load_golden_jsonl")
    lines = [line for line in (pack / "golden.jsonl").read_text(encoding="utf-8").splitlines() if line.strip()]
    if not lines:
        return fail("E_W96_GOLDEN_JSONL_EMPTY", "golden.jsonl must contain at least one case")
    try:
        first = json.loads(lines[0])
    except Exception as exc:
        return fail("E_W96_GOLDEN_JSONL_INVALID", f"line1 invalid json ({exc})")
    complete_probe("load_golden_jsonl")
    start_probe("validate_golden_jsonl")
    if not isinstance(first, dict):
        return fail("E_W96_GOLDEN_JSONL_CASE_INVALID", "line1 must be object")
    cmd = first.get("cmd")
    if not isinstance(cmd, list) or len(cmd) < 2 or str(cmd[0]) != "canon":
        return fail("E_W96_GOLDEN_CMD", f"cmd={cmd}")
    if str(first.get("expected_warning_code", "")).strip() != "W_BLOCK_HEADER_COLON_DEPRECATED":
        return fail(
            "E_W96_GOLDEN_WARNING_CODE",
            f"expected_warning_code={first.get('expected_warning_code')}",
        )
    complete_probe("validate_golden_jsonl")
    complete_case("validate.golden_jsonl")

    update_progress("passed")
    print("[w96-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"adapter_count={len(adapters)}")
    print(f"sim_adapter_count={len(sim_rows)}")
    if sim_rows:
        first_id = str(sim_rows[0].get("id", "")).strip()
        print(f"sample_state_hash={expected_hash_by_id.get(first_id, '-')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
