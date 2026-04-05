#!/usr/bin/env python
from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def ensure_utf8_stdio() -> None:
    for stream_name in ("stdout", "stderr"):
        stream = getattr(sys, stream_name, None)
        if stream is None:
            continue
        reconfigure = getattr(stream, "reconfigure", None)
        if callable(reconfigure):
            try:
                reconfigure(encoding="utf-8")
            except Exception:
                pass


def default_report_path(file_name: str) -> str:
    preferred = Path("I:/home/urihanl/ddn/codex/build/reports")
    if os.name == "nt":
        try:
            preferred.mkdir(parents=True, exist_ok=True)
        except OSError:
            pass
        return str(preferred / file_name)
    return f"build/reports/{file_name}"


def load_payload(path: Path) -> dict | None:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def to_repo_relative(path: Path) -> str:
    try:
        return str(path.resolve().relative_to(ROOT.resolve())).replace("\\", "/")
    except Exception:
        return str(path).replace("\\", "/")


def canonical_sha256(payload: object) -> str:
    raw = json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return "sha256:" + hashlib.sha256(raw).hexdigest()


def collect_proof_files(inputs: list[str]) -> tuple[list[Path], list[str]]:
    files: set[Path] = set()
    missing: list[str] = []
    for raw in inputs:
        candidate = Path(raw)
        if not candidate.is_absolute():
            candidate = ROOT / candidate
        if candidate.is_file():
            files.add(candidate.resolve())
            continue
        if candidate.is_dir():
            for path in candidate.rglob("proof.detjson"):
                if path.is_file():
                    files.add(path.resolve())
            continue
        missing.append(str(candidate))
    return (sorted(files), missing)


def build_artifact_row(path: Path, payload: dict) -> dict[str, object]:
    proof_runtime = payload.get("proof_runtime")
    solver_translation = payload.get("solver_translation")
    runtime_error = payload.get("runtime_error")
    runtime_error_code = ""
    if isinstance(runtime_error, dict):
        runtime_error_code = str(runtime_error.get("code", "")).strip()
    state_hash = str(payload.get("state_hash", "")).strip()
    row = {
        "path": to_repo_relative(path),
        "entry": str(payload.get("entry", "")).strip(),
        "verified": bool(payload.get("verified", False)),
        "runtime_error_code": runtime_error_code or None,
        "state_hash": state_hash or None,
        "has_state_hash": state_hash.startswith("blake3:"),
        "proof_runtime_count": int(payload.get("proof_runtime_count", 0) or 0),
        "proof_block_count": 0,
        "proof_check_count": 0,
        "solver_check_count": 0,
        "solver_search_count": 0,
        "solver_open_count": 0,
    }
    if isinstance(proof_runtime, dict):
        row["proof_block_count"] = int(proof_runtime.get("proof_block_count", 0) or 0)
        row["proof_check_count"] = int(proof_runtime.get("proof_check_count", 0) or 0)
        row["solver_check_count"] = int(proof_runtime.get("solver_check_count", 0) or 0)
        row["solver_search_count"] = int(proof_runtime.get("solver_search_count", 0) or 0)
    if isinstance(solver_translation, dict):
        row["solver_open_count"] = int(solver_translation.get("solver_open_count", 0) or 0)
    return row


def build_report(
    paths: list[Path],
    input_roots: list[str],
    missing_inputs: list[str],
    invalid_paths: list[str],
    top: int,
) -> dict[str, object]:
    artifacts: list[dict[str, object]] = []
    runtime_error_counts: Counter[str] = Counter()
    proof_block_result_counts: Counter[str] = Counter()
    solver_runtime_counts: Counter[str] = Counter()
    proof_check_result_counts: Counter[str] = Counter()
    runtime_error_artifact_count = 0
    runtime_error_state_hash_present_count = 0
    runtime_error_missing_state_hash_entries: list[str] = []

    for path in paths:
        payload = load_payload(path)
        if payload is None:
            invalid_paths.append(to_repo_relative(path))
            continue
        artifact = build_artifact_row(path, payload)
        artifacts.append(artifact)

        runtime_error = payload.get("runtime_error")
        if isinstance(runtime_error, dict):
            code = str(runtime_error.get("code", "")).strip()
            if code:
                runtime_error_counts[code] += 1
                runtime_error_artifact_count += 1
                if bool(artifact.get("has_state_hash", False)):
                    runtime_error_state_hash_present_count += 1
                else:
                    entry = str(artifact.get("entry", "")).strip() or str(artifact.get("path", "")).strip() or "-"
                    runtime_error_missing_state_hash_entries.append(entry)

        proof_runtime = payload.get("proof_runtime")
        items = proof_runtime.get("items") if isinstance(proof_runtime, dict) else None
        if isinstance(items, list):
            for item in items:
                if not isinstance(item, dict):
                    continue
                kind = str(item.get("kind", "")).strip()
                if kind == "proof_block":
                    result = str(item.get("result", "")).strip() or "(unknown)"
                    proof_block_result_counts[result] += 1
                elif kind == "solver_check":
                    solver_runtime_counts["check"] += 1
                elif kind == "solver_search":
                    operation = str(item.get("operation", "")).strip() or "search"
                    solver_runtime_counts[operation] += 1
                elif kind == "proof_check":
                    passed = item.get("passed")
                    bucket = "passed" if passed is True else "failed"
                    proof_check_result_counts[bucket] += 1

    artifacts.sort(key=lambda row: (str(row.get("entry", "")), str(row.get("path", ""))))
    verified_count = sum(1 for row in artifacts if bool(row.get("verified", False)))
    unverified_count = len(artifacts) - verified_count

    failure_digest: list[str] = []
    for row in artifacts:
        if bool(row.get("verified", False)):
            continue
        entry = str(row.get("entry", "")).strip() or "-"
        code = str(row.get("runtime_error_code", "") or "-").strip() or "-"
        failure_digest.append(f"entry={entry} runtime_error={code} path={row.get('path', '-')}")
    for path in invalid_paths:
        failure_digest.append(f"invalid_proof_detjson={path}")
    for path in missing_inputs:
        failure_digest.append(f"missing_input={str(path).replace(chr(92), '/')}")

    report: dict[str, object] = {
        "schema": "ddn.proof_artifact_summary.v1",
        "input_roots": [str(Path(item)).replace("\\", "/") for item in input_roots],
        "artifact_count": len(artifacts),
        "verified_count": verified_count,
        "unverified_count": unverified_count,
        "missing_input_count": len(missing_inputs),
        "missing_inputs": [str(path).replace("\\", "/") for path in missing_inputs],
        "invalid_artifact_count": len(invalid_paths),
        "invalid_artifacts": list(invalid_paths),
        "runtime_error_counts": [
            {"code": key, "count": runtime_error_counts[key]} for key in sorted(runtime_error_counts)
        ],
        "proof_block_result_counts": [
            {"result": key, "count": proof_block_result_counts[key]} for key in sorted(proof_block_result_counts)
        ],
        "solver_runtime_counts": [
            {"operation": key, "count": solver_runtime_counts[key]} for key in sorted(solver_runtime_counts)
        ],
        "proof_check_result_counts": [
            {"result": key, "count": proof_check_result_counts[key]} for key in sorted(proof_check_result_counts)
        ],
        "runtime_error_artifact_count": runtime_error_artifact_count,
        "runtime_error_state_hash_present_count": runtime_error_state_hash_present_count,
        "runtime_error_missing_state_hash_entries": runtime_error_missing_state_hash_entries[: max(1, int(top))],
        "artifacts": artifacts,
        "failure_digest": failure_digest[: max(1, int(top))],
    }
    report["summary_hash"] = canonical_sha256(report)
    return report


def main() -> int:
    ensure_utf8_stdio()
    parser = argparse.ArgumentParser(description="Render aggregate summary from proof.detjson artifacts")
    parser.add_argument("inputs", nargs="*", default=["pack"], help="proof.detjson file or directory to scan")
    parser.add_argument(
        "--report-out",
        default=default_report_path("proof_artifact_summary.detjson"),
        help="output summary detjson path",
    )
    parser.add_argument("--top", type=int, default=16, help="max failure digest lines kept in report")
    args = parser.parse_args()

    files, missing_inputs = collect_proof_files([str(item) for item in args.inputs])
    invalid_paths: list[str] = []
    report = build_report(files, list(args.inputs), missing_inputs, invalid_paths, max(1, int(args.top)))

    report_out = Path(args.report_out)
    if not report_out.is_absolute():
        report_out = ROOT / report_out
    report_out.parent.mkdir(parents=True, exist_ok=True)
    report_out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    print(
        f"[proof-summary] artifacts={report['artifact_count']} verified={report['verified_count']} "
        f"unverified={report['unverified_count']} invalid={report['invalid_artifact_count']} report={report_out}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
