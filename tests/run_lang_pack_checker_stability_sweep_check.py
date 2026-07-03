from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_PACK_CHECKER_STABILITY_SWEEP_V1.md"
PROPOSAL = ROOT / "docs" / "context" / "proposals" / "LANG_PACK_CHECKER_STABILITY_SWEEP_V1.md"
SSOT_NOTE = ROOT / "docs" / "notes" / "SSOT_LANG_PACK_CHECKER_STABILITY_SWEEP_20260611.md"
PACK = ROOT / "pack" / "lang_pack_checker_stability_sweep_v1"
MANIFEST = PACK / "pack_checker_stability_sweep.detjson"
CONTRACT = PACK / "contract.detjson"
CHECKER = ROOT / "tests" / "run_lang_pack_checker_stability_sweep_check.py"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

SOURCE_RESUME = ROOT / "pack" / "lang_non_ssot_track_resume_rebase_v1" / "non_ssot_track_resume_rebase.detjson"
SOURCE_PARKING = ROOT / "pack" / "lang_blocked_ssot_track_parking_rebase_v1" / "blocked_ssot_track_parking_rebase.detjson"
RESUME_CHECKER = ROOT / "tests" / "run_lang_non_ssot_track_resume_rebase_check.py"

WORK_ITEM = "LANG_PACK_CHECKER_STABILITY_SWEEP_V1"
NEXT = "LANG_GOLDEN_EVIDENCE_HYGIENE_SWEEP_V1"
SWEEP_ROWS = ["py_compile_checker", "pack_golden_selftest", "previous_checker_boundary", "docs_ssot_boundary"]
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
    print(f"lang_pack_checker_stability_sweep_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE_RESUME,
        SOURCE_PARKING,
        RESUME_CHECKER,
    ]:
        require(path)


def check_docs() -> None:
    require_contains(
        DOC,
        [
            WORK_ITEM,
            "Sweep Rows",
            "Pack/checker stability sweep: `1/1 = 100%`",
            "Stability sweep rows: `4/4 = 100%`",
            "SSOT owner landing detected: `0/3 = 0%`",
            "ROADMAP_V2 전체: `queue-expanded 79/90 = 88%`",
            NEXT,
        ],
    )
    require_contains(PROPOSAL, [WORK_ITEM, "4/4 = 100%", "0/3 = 0%", "79/90 = 88%", NEXT])
    require_contains(
        SSOT_NOTE,
        [
            "Codex did not edit `docs/ssot/**`",
            "Stability sweep rows is `4/4 = 100%`",
            "SSOT owner landing detected remains `0/3 = 0%`",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            WORK_ITEM,
            "ddn.language.pack_checker_stability_sweep.v1",
            "lang_pack_checker_stability_sweep_v1",
            "Pack/checker stability sweep: 1/1 = 100%",
            "Stability sweep rows: 4/4 = 100%",
            "SSOT owner landing detected: 0/3 = 0%",
            "ROADMAP_V2 전체: queue-expanded 79/90 = 88%",
            "docs/ssot/** 변경 없음",
            NEXT,
        ],
    )


def check_contract() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "lang_pack_checker_stability_sweep_v1",
        "kind": "lang_pack_checker_stability_sweep",
        "pack_checker_stability_sweep_claim": True,
        "closed_by": WORK_ITEM,
        "based_on": "LANG_NON_SSOT_TRACK_RESUME_REBASE_V1",
        "proposal_doc": "docs/context/proposals/LANG_PACK_CHECKER_STABILITY_SWEEP_V1.md",
        "ssot_note": "docs/notes/SSOT_LANG_PACK_CHECKER_STABILITY_SWEEP_20260611.md",
        "decision_manifest": "pack/lang_pack_checker_stability_sweep_v1/pack_checker_stability_sweep.detjson",
        "source_non_ssot_track_resume_rebase": "pack/lang_non_ssot_track_resume_rebase_v1/non_ssot_track_resume_rebase.detjson",
        "source_blocked_ssot_track_parking_rebase": "pack/lang_blocked_ssot_track_parking_rebase_v1/blocked_ssot_track_parking_rebase.detjson",
        "pack_checker_stability_sweep_closed": 1,
        "pack_checker_stability_sweep_total": 1,
        "pack_checker_stability_sweep_percent": 100,
        "stability_sweep_rows_closed": 4,
        "stability_sweep_rows_total": 4,
        "stability_sweep_rows_percent": 100,
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
        "roadmap_v2_queue_expanded_closed": 79,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 88,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    for key in FALSE_FLAGS:
        if contract.get(key) is not False:
            fail(f"contract {key} must be false, got {contract.get(key)!r}")
    for source_key in ["source_non_ssot_track_resume_rebase", "source_blocked_ssot_track_parking_rebase"]:
        require(ROOT / contract[source_key])


def check_manifest() -> None:
    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.language.pack_checker_stability_sweep.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != WORK_ITEM:
        fail(f"work item mismatch: {manifest.get('work_item')!r}")
    if manifest.get("pack_checker_stability_sweep_claim") is not True:
        fail("stability sweep claim must be true")
    for key in FALSE_FLAGS:
        if manifest.get(key) is not False:
            fail(f"manifest {key} must be false, got {manifest.get(key)!r}")

    expected_policy = {
        "id": "language_pack_checker_stability_sweep",
        "status": "closed",
        "requires_docs_ssot_edit": False,
        "workspace_local_cargo_target_required": True,
        "recursive_checker_scope": "nearest_source_boundary",
        "next_candidate": NEXT,
    }
    if manifest.get("stability_policy") != expected_policy:
        fail(f"stability policy mismatch: {manifest.get('stability_policy')!r}")

    rows = manifest.get("sweep_rows", [])
    if [row.get("row") for row in rows] != SWEEP_ROWS:
        fail(f"sweep rows mismatch: {rows!r}")
    for index, row in enumerate(rows, start=1):
        if row.get("order") != index:
            fail(f"sweep row order mismatch: {row!r}")
        if row.get("status") != "stable":
            fail(f"sweep row status mismatch: {row!r}")
        if row.get("requires_docs_ssot_edit") is not False:
            fail(f"sweep row must not require docs/ssot edit: {row!r}")

    expected_plans = {
        "pack_checker_stability_sweep": {"closed": 1, "total": 1, "percent": 100},
        "stability_sweep_rows": {"closed": 4, "total": 4, "percent": 100},
        "non_ssot_resumable_lanes": {"closed": 3, "total": 3, "percent": 100},
        "parked_ssot_rows": {"closed": 3, "total": 3, "percent": 100},
        "ssot_owner_landing_detected": {"closed": 0, "total": 3, "percent": 0},
        "post_ssot_product_gates_open": {"closed": 0, "total": 3, "percent": 0},
        "roadmap_v2_queue_expanded_plan": {"closed": 79, "total": 90, "percent": 88},
    }
    for key, value in expected_plans.items():
        if manifest.get(key) != value:
            fail(f"plan {key} expected {value!r}, got {manifest.get(key)!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"next item mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    resume = load_json(SOURCE_RESUME)
    if resume.get("work_item") != "LANG_NON_SSOT_TRACK_RESUME_REBASE_V1":
        fail(f"resume work item mismatch: {resume.get('work_item')!r}")
    if resume.get("next_item") != WORK_ITEM:
        fail(f"resume next expected {WORK_ITEM}, got {resume.get('next_item')!r}")
    rows = resume.get("resume_rows", [])
    first = rows[0] if rows else {}
    if first.get("lane") != "pack_checker_stability_lane" or first.get("next_candidate") != WORK_ITEM:
        fail(f"resume first lane mismatch: {first!r}")

    parking = load_json(SOURCE_PARKING)
    if parking.get("parked_ssot_rows") != {"closed": 3, "total": 3, "percent": 100}:
        fail(f"parking rows progress mismatch: {parking.get('parked_ssot_rows')!r}")
    if parking.get("ssot_owner_landing_detected") != {"closed": 0, "total": 3, "percent": 0}:
        fail(f"parking owner landing progress mismatch: {parking.get('ssot_owner_landing_detected')!r}")


def check_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_pack_checker_stability_sweep_v1"], timeout=120)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    require_contains(
        PACK / "golden.jsonl",
        [
            WORK_ITEM,
            "schema: ddn.language.pack_checker_stability_sweep.v1",
            "stability sweep rows: 4/4 = 100%",
            "non ssot resumable lanes: 3/3 = 100%",
            "owner landing detected: 0/3 = 0%",
            "roadmap: 79/90 = 88%",
            NEXT,
        ],
    )


def check_previous_checker() -> None:
    proc = run([sys.executable, str(RESUME_CHECKER.relative_to(ROOT))], timeout=1200)
    if proc.returncode != 0:
        fail(f"{RESUME_CHECKER.relative_to(ROOT)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_manifest()
    check_source_alignment()
    check_golden()
    check_previous_checker()
    require_docs_ssot_clean()
    print("lang_pack_checker_stability_sweep_check: PASS")


if __name__ == "__main__":
    main()

