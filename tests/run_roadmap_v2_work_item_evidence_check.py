#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "toolchain_pack_2_v1"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"


def fail(message: str) -> int:
    print(f"[roadmap-v2-evidence] fail: {message}", file=sys.stderr)
    return 1


def sort_json(value):
    if isinstance(value, list):
        return [sort_json(item) for item in value]
    if isinstance(value, dict):
        return {key: sort_json(value[key]) for key in sorted(value)}
    return value


def format_json(value) -> str:
    return json.dumps(sort_json(value), ensure_ascii=False, indent=2) + "\n"


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def load_contract() -> dict:
    payload = read_json(PACK / "fixtures" / "work_items.detjson")
    if payload.get("schema") != "ddn.roadmap_v2.work_item_evidence_contract.v1":
        raise RuntimeError("work_items.detjson schema mismatch")
    return payload


def tracker_statuses() -> dict[str, str]:
    text = TRACKER.read_text(encoding="utf-8")
    out: dict[str, str] = {}
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line.startswith("|") or "`" not in line:
            continue
        cols = [col.strip() for col in line.strip("|").split("|")]
        if len(cols) < 4:
            continue
        match = re.search(r"`([^`]+)`", cols[1])
        if not match:
            continue
        out[match.group(1)] = cols[3]
    return out


def manifest_mentions(*needles: str) -> bool:
    text = MANIFEST.read_text(encoding="utf-8")
    return all(str(needle) in text for needle in needles)


def run_checker(script_rel: str) -> bool:
    proc = subprocess.run(
        [sys.executable, script_rel],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=240,
    )
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip()
        raise RuntimeError(f"{script_rel} failed: {detail}")
    return True


def ssot_diff_clean() -> bool:
    proc = subprocess.run(
        ["git", "diff", "--name-only", "--", "docs/ssot"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=30,
    )
    if proc.returncode != 0:
        raise RuntimeError((proc.stderr or proc.stdout or "git diff failed").strip())
    return not proc.stdout.strip()


def build_report(contract: dict) -> dict:
    statuses = tracker_statuses()
    rows: list[dict] = []
    executed = 0
    closed = 0
    for item in contract.get("work_items", []):
        item_id = str(item.get("id", "")).strip()
        if not item_id:
            raise RuntimeError("contract item missing id")
        expected_status = str(item.get("expected_status", "")).strip()
        closure_level = str(item.get("closure_level", "")).strip()
        status = statuses.get(item_id, "")
        if status != expected_status:
            raise RuntimeError(f"{item_id}: tracker status {status!r} != {expected_status!r}")
        if closure_level == "closed":
            closed += 1

        report_rel = str(item.get("report", "")).strip()
        pack_rel = str(item.get("pack", "")).strip()
        checker_rel = str(item.get("checker", "")).strip()
        expected_rel = str(item.get("expected_artifact", "")).strip()
        for label, rel in (
            ("report", report_rel),
            ("pack", pack_rel),
            ("checker", checker_rel),
            ("expected_artifact", expected_rel),
        ):
            if not rel:
                raise RuntimeError(f"{item_id}: missing {label}")

        report_exists = (ROOT / report_rel).exists()
        pack_exists = (ROOT / pack_rel).exists()
        checker_exists = (ROOT / checker_rel).exists()
        expected_exists = (ROOT / expected_rel).exists()
        if not report_exists:
            raise RuntimeError(f"{item_id}: report missing: {report_rel}")
        if not pack_exists:
            raise RuntimeError(f"{item_id}: pack missing: {pack_rel}")
        if not checker_exists:
            raise RuntimeError(f"{item_id}: checker missing: {checker_rel}")
        if not expected_exists:
            raise RuntimeError(f"{item_id}: expected artifact missing: {expected_rel}")
        if not manifest_mentions(item_id, checker_rel):
            raise RuntimeError(f"{item_id}: evidence manifest does not mention id/checker")

        checker_pass: bool | str
        if bool(item.get("execute_checker")):
            checker_pass = run_checker(checker_rel)
            executed += 1
        else:
            checker_pass = "self"

        rows.append(
            {
                "id": item_id,
                "status": status,
                "closure_level": closure_level,
                "report": report_rel,
                "report_exists": report_exists,
                "pack": pack_rel,
                "pack_exists": pack_exists,
                "checker": checker_rel,
                "checker_pass": checker_pass,
                "expected_artifact": expected_rel,
                "expected_artifact_exists": expected_exists,
            }
        )

    ta1 = contract.get("ta1", {}) if isinstance(contract.get("ta1"), dict) else {}
    ta1_official_closed = (ROOT / "docs" / "status" / "roadmap_v2" / "타-1_REPORT_20260428.md").exists()
    ta1_runner_basis = (ROOT / "tests" / "run_pack_golden.py").exists() and (ROOT / "tests" / "run_seamgrim_malblock_codegen_check.py").exists()
    if bool(ta1.get("official_closed_report_required")) and not ta1_official_closed:
        raise RuntimeError("타-1 official closure report is required but missing")
    if bool(ta1.get("runner_basis_required")) and not ta1_runner_basis:
        raise RuntimeError("타-1 runner basis is required but missing")

    ssot_clean = ssot_diff_clean()
    if bool(contract.get("ssot_untouched_required")) and not ssot_clean:
        raise RuntimeError("docs/ssot diff is not clean")

    return {
        "schema": "ddn.roadmap_v2.work_item_evidence_report.v1",
        "summary": {
            "tracked_item_count": len(rows),
            "closed_item_count": closed,
            "executed_check_count": executed,
            "ta1_official_closed": ta1_official_closed,
            "ta1_runner_basis_satisfied": ta1_runner_basis,
            "ssot_untouched_required": bool(contract.get("ssot_untouched_required")),
            "ssot_diff_clean": ssot_clean,
        },
        "work_items": rows,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="ROADMAP_V2 work item evidence gate")
    parser.add_argument("--update", action="store_true", help="expected report 갱신")
    args = parser.parse_args()

    try:
        report = build_report(load_contract())
        expected_path = PACK / "expected" / "work_item_evidence.detjson"
        actual_text = format_json(report)
        if args.update:
            expected_path.parent.mkdir(parents=True, exist_ok=True)
            expected_path.write_text(actual_text, encoding="utf-8")
            print(f"[roadmap-v2-evidence] updated {expected_path.relative_to(ROOT)}")
            return 0
        expected_text = expected_path.read_text(encoding="utf-8")
        if expected_text != actual_text:
            raise RuntimeError(f"expected mismatch: {expected_path.relative_to(ROOT)}")
    except Exception as exc:
        return fail(str(exc))

    print(
        "[roadmap-v2-evidence] ok "
        f"tracked={report['summary']['tracked_item_count']} "
        f"closed={report['summary']['closed_item_count']} "
        f"executed={report['summary']['executed_check_count']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
