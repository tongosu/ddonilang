#!/usr/bin/env python
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_malblock_roundtrip_subset_v1"
CASES_PATH = PACK / "fixtures" / "cases.detjson"
EXPECTED_PATH = PACK / "expected" / "malblock_roundtrip_subset.detjson"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"


def fail(message: str) -> int:
    print(f"[seamgrim-malblock-roundtrip-subset] fail: {message}", file=sys.stderr)
    return 1


def sort_json(value):
    if isinstance(value, list):
        return [sort_json(item) for item in value]
    if isinstance(value, dict):
        return {key: sort_json(value[key]) for key in sorted(value)}
    return value


def format_json(value) -> str:
    return json.dumps(sort_json(value), ensure_ascii=False, indent=2) + "\n"


def sha256_text(text: str) -> str:
    return f"sha256:{hashlib.sha256(text.encode('utf-8')).hexdigest()}"


def resolve_teul_cli_prefix() -> list[str]:
    suffix = ".exe" if sys.platform.startswith("win") else ""
    candidates = [
        ROOT / "target" / "debug" / f"teul-cli{suffix}",
        ROOT / "target" / "release" / f"teul-cli{suffix}",
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"I:/home/urihanl/ddn/codex/target/release/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/release/teul-cli{suffix}"),
    ]
    for candidate in candidates:
        if candidate.exists():
            return [str(candidate)]
    return [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        str(ROOT / "tools" / "teul-cli" / "Cargo.toml"),
        "--",
    ]


def run_cmd(cmd: list[str], *, timeout: int = 180, cwd: Path = ROOT) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )


def teul_cli_text(args: list[str], *, timeout: int = 180) -> str:
    cmd = [*resolve_teul_cli_prefix(), *args]
    proc = run_cmd(cmd, timeout=timeout)
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip()
        raise RuntimeError(f"{' '.join(cmd)}\n{detail}")
    return proc.stdout


def load_cases() -> dict:
    payload = json.loads(CASES_PATH.read_text(encoding="utf-8"))
    if payload.get("schema") != "ddn.seamgrim_malblock_roundtrip_subset_cases.v1":
        raise RuntimeError("cases.detjson schema mismatch")
    cases = payload.get("cases")
    if not isinstance(cases, list) or not cases:
        raise RuntimeError("cases.detjson must contain cases")
    return payload


def run_roundtrip_runner(case: dict, temp_root: Path) -> dict:
    case_id = str(case.get("id", "")).strip()
    if not case_id:
        raise RuntimeError("case missing id")
    pack_dir = temp_root / case_id
    fixtures_dir = pack_dir / "fixtures"
    expected_dir = pack_dir / "expected"
    fixtures_dir.mkdir(parents=True, exist_ok=True)
    expected_dir.mkdir(parents=True, exist_ok=True)
    (fixtures_dir / "source.ddn").write_text(str(case.get("source", "")), encoding="utf-8")

    proc = run_cmd(
        [
            "node",
            "--no-warnings",
            "tests/block_editor_roundtrip_runner.mjs",
            str(pack_dir),
            "--update",
        ],
        timeout=180,
    )
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip()
        raise RuntimeError(f"{case_id}: block editor roundtrip runner failed\n{detail}")
    return json.loads((expected_dir / "block_editor_roundtrip.detjson").read_text(encoding="utf-8"))


def validate_required(case_id: str, actual: dict, case: dict) -> None:
    if actual.get("schema") != "ddn.block_editor_roundtrip_smoke.v1":
        raise RuntimeError(f"{case_id}: unexpected runner schema")
    if actual.get("block_plan_schema") != "ddn.block_editor_plan.v1":
        raise RuntimeError(f"{case_id}: unexpected block plan schema")
    if actual.get("decode_errors") != []:
        raise RuntimeError(f"{case_id}: decode_errors must be empty")
    if actual.get("canon_equal") is not True:
        raise RuntimeError(f"{case_id}: canon_equal must be true")

    raw_count = int(actual.get("raw_block_count", -1))
    if "expected_raw_block_count" in case and raw_count != int(case["expected_raw_block_count"]):
        raise RuntimeError(f"{case_id}: raw_block_count={raw_count}")
    if "expected_min_raw_block_count" in case and raw_count < int(case["expected_min_raw_block_count"]):
        raise RuntimeError(f"{case_id}: raw_block_count {raw_count} below minimum")

    counts = actual.get("block_kind_counts")
    if not isinstance(counts, dict):
        raise RuntimeError(f"{case_id}: block_kind_counts missing")
    for kind in case.get("required_block_kinds", []):
        if int(counts.get(str(kind), 0)) < 1:
            raise RuntimeError(f"{case_id}: required block kind missing: {kind}")

    top_level = [str(item) for item in actual.get("top_level_block_kinds", [])]
    for kind in case.get("required_top_level_block_kinds", []):
        if str(kind) not in top_level:
            raise RuntimeError(f"{case_id}: required top-level block kind missing: {kind}")

    canon_before = str(actual.get("canon_before", ""))
    for needle in case.get("expected_canon_contains", []):
        if str(needle) not in canon_before:
            raise RuntimeError(f"{case_id}: canon_before missing {needle!r}")

    raw_text = "\n".join(str(item.get("raw_text", "")) for item in actual.get("raw_blocks", []))
    for needle in case.get("required_raw_text_contains", []):
        if str(needle) not in raw_text:
            raise RuntimeError(f"{case_id}: raw block text missing {needle!r}")


def run_encoded_output(case_id: str, encoded_ddn: str, expected_stdout: list[str], madi: int, temp_root: Path) -> list[str]:
    source_path = temp_root / f"{case_id}.encoded.ddn"
    source_path.write_text(encoded_ddn, encoding="utf-8")
    stdout = teul_cli_text(["run", str(source_path), "--madi", str(madi)], timeout=180)
    lines = [
        line.rstrip("\r")
        for line in stdout.splitlines()
        if line.strip()
        and not line.startswith("state_hash=")
        and not line.startswith("trace_hash=")
    ]
    if lines != [str(item) for item in expected_stdout]:
        raise RuntimeError(f"{case_id}: encoded run stdout mismatch: {lines!r}")
    return lines


def validate_case(case: dict, actual: dict, temp_root: Path) -> dict:
    case_id = str(case.get("id", "")).strip()
    validate_required(case_id, actual, case)
    expected_stdout = [str(item) for item in case.get("expected_stdout", [])]
    run_stdout = []
    if expected_stdout:
        run_stdout = run_encoded_output(
            case_id,
            str(actual.get("encoded_ddn", "")),
            expected_stdout,
            int(case.get("madi", 1)),
            temp_root,
        )
    return {
        "id": case_id,
        "block_plan_schema": str(actual.get("block_plan_schema", "")),
        "canon_equal": bool(actual.get("canon_equal")),
        "canon_hash": str(actual.get("canon_before_hash", "")),
        "encoded_hash": str(actual.get("encoded_hash", "")),
        "block_kind_counts": actual.get("block_kind_counts", {}),
        "top_level_block_kinds": [str(item) for item in actual.get("top_level_block_kinds", [])],
        "raw_block_count": int(actual.get("raw_block_count", 0)),
        "raw_blocks": actual.get("raw_blocks", []),
        "run_stdout": run_stdout,
    }


def generate_report() -> dict:
    payload = load_cases()
    cases = payload["cases"]
    with tempfile.TemporaryDirectory(prefix="ddn-malblock-roundtrip-subset-") as temp_dir:
        temp_root = Path(temp_dir)
        rows = []
        for case in cases:
            actual = run_roundtrip_runner(case, temp_root)
            rows.append(validate_case(case, actual, temp_root))
    return {
        "schema": "ddn.seamgrim_malblock_roundtrip_subset_report.v1",
        "case_count": len(rows),
        "supported_case_count": sum(1 for row in rows if int(row["raw_block_count"]) == 0),
        "raw_fallback_case_count": sum(1 for row in rows if int(row["raw_block_count"]) > 0),
        "all_canon_equal": all(bool(row["canon_equal"]) for row in rows),
        "excluded_surfaces": payload.get("excluded_surfaces", []),
        "cases": rows,
    }


def check_reference_docs() -> None:
    root_doc = ROOT / "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1.md"
    if not root_doc.exists():
        raise RuntimeError("missing SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1.md")
    text = root_doc.read_text(encoding="utf-8")
    required = [
        "ROADMAP_V2 `라-2`",
        "DDN -> 말블록",
        "canonBlockEditorPlan",
        "canonical DDN",
        "raw/opaque blocks",
        "(조건)이 될때",
        "(조건)인 동안",
        "ROADMAP_V2_LA2_FINAL_CLOSURE_V1",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        raise RuntimeError(f"root doc missing tokens: {missing}")


def check_queue() -> None:
    if not QUEUE.exists():
        raise RuntimeError("missing NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md")
    text = QUEUE.read_text(encoding="utf-8")
    required = [
        "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1",
        "closed by `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1.md`",
        "pack/seamgrim_malblock_roundtrip_subset_v1",
        "ROADMAP_V2_LA2_FINAL_CLOSURE_V1",
        "closed by `ROADMAP_V2_LA2_FINAL_CLOSURE_V1.md`",
        "ROADMAP_V2_A1_NURIGYM_REBASE_V1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "No automatic next development item is selected.",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        raise RuntimeError(f"queue missing tokens: {missing}")
    if "1. `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1`" in text:
        raise RuntimeError("SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1 is still listed as next open item")
    if "1. `ROADMAP_V2_LA2_FINAL_CLOSURE_V1`" in text:
        raise RuntimeError("ROADMAP_V2_LA2_FINAL_CLOSURE_V1 is still listed as next open item")
    if "1. `ROOT_LOW_RISK_RETIRE_DELETE_V1`" in text:
        raise RuntimeError("ROOT_LOW_RISK_RETIRE_DELETE_V1 is still listed as next open item")


def check_docs_ssot_clean() -> None:
    result = run_cmd(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if result.returncode != 0:
        raise RuntimeError(result.stdout.strip() or result.stderr.strip())
    if result.stdout.strip():
        raise RuntimeError(f"docs/ssot dirty: {result.stdout.strip()}")


def main() -> int:
    parser = argparse.ArgumentParser(description="SEAMGRIM 말블록 DDN roundtrip subset checker")
    parser.add_argument("--update", action="store_true", help="expected report 갱신")
    args = parser.parse_args()

    try:
        check_reference_docs()
        report = generate_report()
        actual_text = format_json(report)
        if args.update:
            EXPECTED_PATH.parent.mkdir(parents=True, exist_ok=True)
            EXPECTED_PATH.write_text(actual_text, encoding="utf-8")
            print(f"[seamgrim-malblock-roundtrip-subset] updated {EXPECTED_PATH.relative_to(ROOT)}")
            return 0
        expected_text = EXPECTED_PATH.read_text(encoding="utf-8")
        if expected_text != actual_text:
            raise RuntimeError(f"expected mismatch: {EXPECTED_PATH.relative_to(ROOT)}")
        check_queue()
        check_docs_ssot_clean()
    except Exception as exc:
        return fail(str(exc))

    print(
        "[seamgrim-malblock-roundtrip-subset] ok "
        f"cases={report['case_count']} supported={report['supported_case_count']} raw={report['raw_fallback_case_count']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
