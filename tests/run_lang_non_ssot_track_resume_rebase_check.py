from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_NON_SSOT_TRACK_RESUME_REBASE_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_NON_SSOT_TRACK_RESUME_REBASE_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_NON_SSOT_TRACK_RESUME_REBASE_20260610.md"
PACK = ROOT / "pack" / "lang_non_ssot_track_resume_rebase_v1"
MANIFEST = PACK / "non_ssot_track_resume_rebase.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_non_ssot_track_resume_rebase_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_PARKING = ROOT / "pack" / "lang_blocked_ssot_track_parking_rebase_v1" / "blocked_ssot_track_parking_rebase.detjson"
SOURCE_AUDIT = ROOT / "pack" / "lang_ssot_owner_landing_audit_rebase_v1" / "ssot_owner_landing_audit_rebase.detjson"
PARKING_CHECKER = ROOT / "tests" / "run_lang_blocked_ssot_track_parking_rebase_check.py"

WORK_ITEM = "LANG_NON_SSOT_TRACK_RESUME_REBASE_V1"
NEXT = "LANG_PACK_CHECKER_STABILITY_SWEEP_V1"
LANES = ["pack_checker_stability_lane", "golden_evidence_hygiene_lane", "non_ssot_product_boundary_lane"]
FALSE_FLAGS = [
    "runtime_claim",
    "product_code_change",
    "product_ui_change",
    "lesson_schema_change",
    "active_allowlist_mutation",
    "parser_frontdoor_change",
    "stdlib_surface_change",
    "ssot_edit_claim",
    "ssot_landed_claim",
    "backward_compat_break_claim",
    "post_ssot_product_gate_open_claim",
]


def fail(message: str) -> None:
    print(f"lang_non_ssot_track_resume_rebase_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")


def load_json(path: Path) -> dict:
    return json.loads(read(path))


def require(path: Path) -> None:
    if not path.exists():
        fail(f"missing required path: {path.relative_to(ROOT)}")


def require_contains(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing tokens: {missing}")


def run(cmd: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env.setdefault("CARGO_TARGET_DIR", str(ROOT / "build" / "cargo-target-checks"))
    return subprocess.run(
        cmd,
        cwd=ROOT,
        env=env,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
        check=False,
    )


def require_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed: {proc.stdout.strip()}")
    status_lines = [
        line for line in proc.stdout.splitlines()
        if line.strip() and not line.startswith("warning:")
    ]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def check_required_files() -> None:
    for path in [
        DOC,
        PROPOSAL,
        SSOT_NOTE,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        MANIFEST,
        CHECKER,
        DEV_SUMMARY,
        SOURCE_PARKING,
        SOURCE_AUDIT,
        PARKING_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    require_contains(
        DOC,
        [
            WORK_ITEM,
            "Resume Rows",
            "Non-SSOT track resume rebase: `1/1 = 100%`",
            "Non-SSOT resumable lanes: `3/3 = 100%`",
            "SSOT owner landing detected: `0/3 = 0%`",
            "ROADMAP_V2 전체: `queue-expanded 78/90 = 87%`",
            NEXT,
        ],
    )
    require_contains(PROPOSAL, [WORK_ITEM, "3/3 = 100%", "0/3 = 0%", "78/90 = 87%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "Non-SSOT resumable lanes is `3/3 = 100%`",
            "SSOT owner landing detected remains `0/3 = 0%`",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.non_ssot_track_resume_rebase.v1",
            "lang_non_ssot_track_resume_rebase_v1",
            "Non-SSOT track resume rebase: 1/1 = 100%",
            "Non-SSOT resumable lanes: 3/3 = 100%",
            "SSOT owner landing detected: 0/3 = 0%",
            "ROADMAP_V2 전체: queue-expanded 78/90 = 87%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_non_ssot_track_resume_rebase_v1",
        "kind": "lang_non_ssot_track_resume_rebase",
        "non_ssot_track_resume_rebase_claim": True,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_BLOCKED_SSOT_TRACK_PARKING_REBASE_V1",
        "proposal_doc": "docs/context/proposals/LANG_NON_SSOT_TRACK_RESUME_REBASE_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_NON_SSOT_TRACK_RESUME_REBASE_20260610.md",
        "decision_manifest": "pack/lang_non_ssot_track_resume_rebase_v1/non_ssot_track_resume_rebase.detjson",
        "source_blocked_ssot_track_parking_rebase": "pack/lang_blocked_ssot_track_parking_rebase_v1/blocked_ssot_track_parking_rebase.detjson",
        "source_ssot_owner_landing_audit_rebase": "pack/lang_ssot_owner_landing_audit_rebase_v1/ssot_owner_landing_audit_rebase.detjson",
        "non_ssot_track_resume_rebase_closed": 1,
        "non_ssot_track_resume_rebase_total": 1,
        "non_ssot_track_resume_rebase_percent": 100,
        "non_ssot_resumable_lanes_closed": 3,
        "non_ssot_resumable_lanes_total": 3,
        "non_ssot_resumable_lanes_percent": 100,
        "parked_ssot_rows_closed": 3,
        "parked_ssot_rows_total": 3,
        "parked_ssot_rows_percent": 100,
        "ssot_owner_landing_detected_closed": 0,
        "ssot_owner_landing_detected_total": 3,
        "ssot_owner_landing_detected_percent": 0,
        "post_ssot_product_gates_open_closed": 0,
        "post_ssot_product_gates_open_total": 3,
        "post_ssot_product_gates_open_percent": 0,
        "roadmap_v2_queue_expanded_closed": 78,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 87,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    for key in FALSE_FLAGS:
        if contract.get(key) is not False:
            fail(f"contract {key} must be false, got {contract.get(key)!r}")
    for source_key in ["source_blocked_ssot_track_parking_rebase", "source_ssot_owner_landing_audit_rebase"]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.non_ssot_track_resume_rebase.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    if manifest.get("non_ssot_track_resume_rebase_claim") is not True:
        fail("resume rebase claim must be true")
    for key in FALSE_FLAGS:
        if manifest.get(key) is not False:
            fail(f"manifest {key} must be false, got {manifest.get(key)!r}")

    expected_policy = {
        "id": "language_non_ssot_track_resume",
        "status": "resumed",
        "requires_docs_ssot_edit": False,
        "parked_ssot_track_stays_parked": True,
        "next_candidate": NEXT,
    }
    if manifest.get("resume_policy") != expected_policy:
        fail(f"resume policy mismatch: {manifest.get('resume_policy')!r}")

    rows = manifest.get("resume_rows", [])
    if [row.get("lane") for row in rows] != LANES:
        fail(f"resume lanes mismatch: {rows!r}")
    for index, row in enumerate(rows, start=1):
        if row.get("order") != index:
            fail(f"resume row order mismatch: {row!r}")
        if row.get("status") != "resumable":
            fail(f"resume row status mismatch: {row!r}")
        if row.get("requires_docs_ssot_edit") is not False:
            fail(f"resume row must not require docs/ssot edit: {row!r}")

    expected_plans = {
        "non_ssot_track_resume_rebase": {"closed": 1, "total": 1, "percent": 100},
        "non_ssot_resumable_lanes": {"closed": 3, "total": 3, "percent": 100},
        "parked_ssot_rows": {"closed": 3, "total": 3, "percent": 100},
        "ssot_owner_landing_detected": {"closed": 0, "total": 3, "percent": 0},
        "post_ssot_product_gates_open": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 78, "total": 90, "percent": 87},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    parking = load_json(SOURCE_PARKING)
    if parking.get("work_item") != "LANG_BLOCKED_SSOT_TRACK_PARKING_REBASE_V1":
        fail(f"parking work item mismatch: {parking.get('work_item')!r}")
    if parking.get("next_item") != WORK_ITEM:
        fail(f"parking next expected {WORK_ITEM}, got {parking.get('next_item')!r}")
    if parking.get("parking_policy", {}).get("non_ssot_work_allowed") is not True:
        fail(f"parking source must allow non-SSOT work: {parking.get('parking_policy')!r}")
    if parking.get("parked_ssot_rows") != {"closed": 3, "total": 3, "percent": 100}:
        fail(f"parking rows progress mismatch: {parking.get('parked_ssot_rows')!r}")

    audit = load_json(SOURCE_AUDIT)
    if audit.get("ssot_owner_landing_detected") != {"closed": 0, "total": 3, "percent": 0}:
        fail(f"audit owner landing progress mismatch: {audit.get('ssot_owner_landing_detected')!r}")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_non_ssot_track_resume_rebase_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    require_contains(
        PACK / "golden.jsonl",
        [
            WORK_ITEM,
            "schema: ddn.language.non_ssot_track_resume_rebase.v1",
            "non ssot resumable lanes: 3/3 = 100%",
            "parked ssot rows: 3/3 = 100%",
            "owner landing detected: 0/3 = 0%",
            "roadmap: 78/90 = 87%",
            NEXT,
        ],
    )


def check_previous_checker() -> None:
    proc = run([sys.executable, str(PARKING_CHECKER.relative_to(ROOT))], timeout=1200)
    if proc.returncode != 0:
        fail(f"{PARKING_CHECKER.relative_to(ROOT)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_golden()
    check_previous_checker()
    require_docs_ssot_clean()
    print("lang_non_ssot_track_resume_rebase_check: PASS")


if __name__ == "__main__":
    main()

