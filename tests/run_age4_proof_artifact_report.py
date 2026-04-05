#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

PROOF_PACKS = [
    "pack/age1_immediate_proof_smoke_v1",
    "pack/age4_proof_clock_replay_missing_failure_v1",
    "pack/age4_proof_clock_replay_parse_failure_v1",
    "pack/age4_proof_clock_replay_tamper_failure_v1",
    "pack/age4_proof_file_read_replay_missing_failure_v1",
    "pack/age4_proof_file_read_replay_parse_failure_v1",
    "pack/age4_proof_file_read_replay_tamper_failure_v1",
    "pack/age4_proof_input_replay_missing_failure_v1",
    "pack/age4_proof_input_replay_parse_failure_v1",
    "pack/age4_proof_input_replay_tamper_failure_v1",
    "pack/age4_proof_solver_deny_failure_v1",
    "pack/age4_proof_solver_open_replay_v1",
    "pack/age4_proof_solver_replay_missing_failure_v1",
    "pack/age4_proof_solver_replay_parse_failure_v1",
    "pack/age4_proof_solver_replay_tamper_failure_v1",
    "pack/age4_proof_solver_search_replay_v1",
    "pack/age4_proof_solver_translation_smoke_v1",
]

EXPECTED_RUNTIME_ERROR_COUNTS = {
    "E_OPEN_DENIED": 1,
    "E_OPEN_LOG_PARSE": 4,
    "E_OPEN_LOG_TAMPER": 4,
    "E_OPEN_REPLAY_MISS": 4,
}

EXPECTED_SOLVER_RUNTIME_COUNTS = {
    "check": 6,
    "counterexample": 2,
    "solve": 1,
}

EXPECTED_PROOF_BLOCK_RESULT_COUNTS = {
    "성공": 1,
    "실패": 1,
}

EXPECTED_ARTIFACT_COUNT = 17
EXPECTED_VERIFIED_COUNT = 4
EXPECTED_UNVERIFIED_COUNT = 13
EXPECTED_RUNTIME_ERROR_STATEHASH_PRESENT_COUNT = 13


def format_failed_criteria_preview(criteria: object) -> str:
    if not isinstance(criteria, list):
        return "-"
    names = [
        str(row.get("name", "")).strip()
        for row in criteria
        if isinstance(row, dict) and not bool(row.get("ok", False)) and str(row.get("name", "")).strip()
    ]
    if not names:
        return "-"
    preview = names[:2]
    if len(names) > 2:
        preview.append(f"+{len(names) - 2}more")
    return ",".join(preview)


def default_report_path(file_name: str) -> str:
    preferred = Path("I:/home/urihanl/ddn/codex/build/reports")
    if os.name == "nt":
        try:
            preferred.mkdir(parents=True, exist_ok=True)
        except OSError:
            pass
        return str(preferred / file_name)
    return f"build/reports/{file_name}"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def counter_map(rows: object, key: str) -> dict[str, int]:
    result: dict[str, int] = {}
    if not isinstance(rows, list):
        return result
    for row in rows:
        if not isinstance(row, dict):
            continue
        label = str(row.get(key, "")).strip()
        if not label:
            continue
        result[label] = int(row.get("count", 0) or 0)
    return result


def clip(text: str, limit: int = 160) -> str:
    normalized = " ".join(str(text).split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def run_summary(summary_out: Path) -> tuple[int, str, str]:
    cmd = [
        sys.executable,
        "tools/scripts/render_proof_artifact_summary.py",
        *PROOF_PACKS,
        "--report-out",
        str(summary_out),
        "--top",
        "16",
    ]
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return (int(proc.returncode), str(proc.stdout or ""), str(proc.stderr or ""))


def build_report(summary_doc: dict | None, summary_path: Path, rc: int, stdout: str, stderr: str) -> dict[str, object]:
    criteria: list[dict[str, object]] = []
    failure_digest: list[str] = []

    summary_ok = isinstance(summary_doc, dict) and str(summary_doc.get("schema", "")).strip() == "ddn.proof_artifact_summary.v1"
    criteria.append(
        {
            "name": "proof_summary_generated",
            "ok": rc == 0 and summary_ok,
            "detail": f"rc={rc} summary={summary_path}",
        }
    )
    if rc != 0:
        failure_digest.append(f"proof_summary_generated: rc={rc} stderr={clip(stderr)}")
    elif not summary_ok:
        failure_digest.append(f"proof_summary_generated: missing_or_invalid_summary={summary_path}")

    artifact_count = int(summary_doc.get("artifact_count", 0) or 0) if summary_ok else 0
    verified_count = int(summary_doc.get("verified_count", 0) or 0) if summary_ok else 0
    unverified_count = int(summary_doc.get("unverified_count", 0) or 0) if summary_ok else 0
    invalid_count = int(summary_doc.get("invalid_artifact_count", 0) or 0) if summary_ok else 0
    missing_count = int(summary_doc.get("missing_input_count", 0) or 0) if summary_ok else 0

    count_ok = (
        artifact_count == EXPECTED_ARTIFACT_COUNT
        and verified_count == EXPECTED_VERIFIED_COUNT
        and unverified_count == EXPECTED_UNVERIFIED_COUNT
        and invalid_count == 0
        and missing_count == 0
    )
    criteria.append(
        {
            "name": "proof_artifact_counts_match",
            "ok": count_ok,
            "detail": (
                f"artifacts={artifact_count}/{EXPECTED_ARTIFACT_COUNT} "
                f"verified={verified_count}/{EXPECTED_VERIFIED_COUNT} "
                f"unverified={unverified_count}/{EXPECTED_UNVERIFIED_COUNT} "
                f"invalid={invalid_count} missing={missing_count}"
            ),
        }
    )
    if not count_ok:
        failure_digest.append(
            "proof_artifact_counts_match: "
            + clip(
                f"artifacts={artifact_count} verified={verified_count} "
                f"unverified={unverified_count} invalid={invalid_count} missing={missing_count}"
            )
        )

    runtime_error_counts = counter_map(summary_doc.get("runtime_error_counts") if summary_ok else None, "code")
    runtime_ok = runtime_error_counts == EXPECTED_RUNTIME_ERROR_COUNTS
    criteria.append(
        {
            "name": "proof_runtime_error_taxonomy_complete",
            "ok": runtime_ok,
            "detail": json.dumps(runtime_error_counts, ensure_ascii=False, sort_keys=True),
        }
    )
    if not runtime_ok:
        failure_digest.append(
            "proof_runtime_error_taxonomy_complete: "
            + clip(json.dumps(runtime_error_counts, ensure_ascii=False, sort_keys=True))
        )

    runtime_error_artifact_count = int(summary_doc.get("runtime_error_artifact_count", 0) or 0) if summary_ok else 0
    runtime_error_state_hash_present_count = (
        int(summary_doc.get("runtime_error_state_hash_present_count", 0) or 0) if summary_ok else 0
    )
    runtime_error_missing_state_hash_entries = (
        summary_doc.get("runtime_error_missing_state_hash_entries", []) if summary_ok else []
    )
    runtime_state_hash_ok = (
        runtime_error_artifact_count == EXPECTED_UNVERIFIED_COUNT
        and runtime_error_state_hash_present_count == EXPECTED_RUNTIME_ERROR_STATEHASH_PRESENT_COUNT
        and runtime_error_artifact_count == runtime_error_state_hash_present_count
        and runtime_error_missing_state_hash_entries == []
    )
    criteria.append(
        {
            "name": "proof_runtime_error_statehash_preserved",
            "ok": runtime_state_hash_ok,
            "detail": (
                f"runtime_error_artifacts={runtime_error_artifact_count} "
                f"statehash_present={runtime_error_state_hash_present_count} "
                f"missing={json.dumps(runtime_error_missing_state_hash_entries, ensure_ascii=False)}"
            ),
        }
    )
    if not runtime_state_hash_ok:
        failure_digest.append(
            "proof_runtime_error_statehash_preserved: "
            + clip(
                f"runtime_error_artifacts={runtime_error_artifact_count} "
                f"statehash_present={runtime_error_state_hash_present_count} "
                f"missing={json.dumps(runtime_error_missing_state_hash_entries, ensure_ascii=False)}"
            )
        )

    solver_runtime_counts = counter_map(summary_doc.get("solver_runtime_counts") if summary_ok else None, "operation")
    solver_ok = solver_runtime_counts == EXPECTED_SOLVER_RUNTIME_COUNTS
    criteria.append(
        {
            "name": "proof_solver_runtime_coverage_complete",
            "ok": solver_ok,
            "detail": json.dumps(solver_runtime_counts, ensure_ascii=False, sort_keys=True),
        }
    )
    if not solver_ok:
        failure_digest.append(
            "proof_solver_runtime_coverage_complete: "
            + clip(json.dumps(solver_runtime_counts, ensure_ascii=False, sort_keys=True))
        )

    proof_block_counts = counter_map(summary_doc.get("proof_block_result_counts") if summary_ok else None, "result")
    proof_block_ok = proof_block_counts == EXPECTED_PROOF_BLOCK_RESULT_COUNTS
    criteria.append(
        {
            "name": "proof_block_result_coverage_complete",
            "ok": proof_block_ok,
            "detail": json.dumps(proof_block_counts, ensure_ascii=False, sort_keys=True),
        }
    )
    if not proof_block_ok:
        failure_digest.append(
            "proof_block_result_coverage_complete: "
            + clip(json.dumps(proof_block_counts, ensure_ascii=False, sort_keys=True))
        )

    summary_hash = str(summary_doc.get("summary_hash", "")).strip() if summary_ok else ""
    hash_ok = summary_hash.startswith("sha256:")
    criteria.append(
        {
            "name": "proof_summary_hash_present",
            "ok": hash_ok,
            "detail": summary_hash or "-",
        }
    )
    if not hash_ok:
        failure_digest.append("proof_summary_hash_present: summary_hash missing")

    overall_ok = all(bool(row.get("ok", False)) for row in criteria)
    failed_criteria_preview = format_failed_criteria_preview(criteria)
    return {
        "schema": "ddn.age4.proof_artifact_report.v1",
        "generated_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "overall_ok": overall_ok,
        "criteria": criteria,
        "failed_criteria_preview": failed_criteria_preview,
        "failure_digest": failure_digest[:16],
        "proof_summary_path": str(summary_path).replace("\\", "/"),
        "proof_summary_hash": summary_hash or "-",
        "proof_summary_stdout": clip(stdout),
        "artifact_count": artifact_count,
        "verified_count": verified_count,
        "unverified_count": unverified_count,
        "runtime_error_counts": summary_doc.get("runtime_error_counts", []) if summary_ok else [],
        "runtime_error_artifact_count": runtime_error_artifact_count,
        "runtime_error_state_hash_present_count": runtime_error_state_hash_present_count,
        "solver_runtime_counts": summary_doc.get("solver_runtime_counts", []) if summary_ok else [],
        "proof_block_result_counts": summary_doc.get("proof_block_result_counts", []) if summary_ok else [],
        "pack_count": len(PROOF_PACKS),
        "packs": list(PROOF_PACKS),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate AGE4 proof artifact aggregate report")
    parser.add_argument(
        "--proof-summary-out",
        default=default_report_path("proof_artifact_summary.detjson"),
        help="child proof artifact summary path",
    )
    parser.add_argument(
        "--report-out",
        default=default_report_path("age4_proof_artifact_report.detjson"),
        help="output age4 proof artifact report path",
    )
    args = parser.parse_args()

    summary_out = Path(args.proof_summary_out)
    if not summary_out.is_absolute():
        summary_out = ROOT / summary_out
    report_out = Path(args.report_out)
    if not report_out.is_absolute():
        report_out = ROOT / report_out

    rc, stdout, stderr = run_summary(summary_out)
    summary_doc = load_json(summary_out)
    report = build_report(summary_doc, summary_out, rc, stdout, stderr)

    report_out.parent.mkdir(parents=True, exist_ok=True)
    report_out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    failed = sum(1 for row in report["criteria"] if isinstance(row, dict) and not bool(row.get("ok", False)))
    print(
        f"[age4-proof-report] overall_ok={int(bool(report['overall_ok']))} "
        f"criteria={len(report['criteria'])} failed={failed} "
        f"failed_preview={report.get('failed_criteria_preview', '-')} report={report_out}"
    )
    if not bool(report["overall_ok"]):
        for line in report["failure_digest"][:8]:
            print(f" - {line}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
