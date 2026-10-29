#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "creator_gaji_round7_handoff_v1"
AGGREGATE = PACK / "r0_r7_aggregate.detjson"
IMPL_REPORT = ROOT / "docs" / "reports" / "impl" / "REPORT_CREATOR_GAJI_ROUND0_GREEN_BASELINE_V1_20260717.md"
AUDIT_REQUEST = ROOT / "docs" / "reports" / "audit" / "REQUEST_CREATOR_GAJI_ROUND0_GREEN_BASELINE_INDEPENDENT_REVIEW_V1_20260717.md"
SESSION_DELTA = ROOT / "work" / "_session_deltas" / "20260717_codex_creator_gaji_round0_green_baseline.md"


def fail(code: str, detail: str) -> int:
    print(f"[creator-gaji-round7-handoff] fail {code}: {detail}")
    return 1


def validate_aggregate(doc: dict) -> list[str]:
    failures: list[str] = []
    if doc.get("schema") != "ddn.creator_gaji.round7_handoff.aggregate.v1":
        failures.append("schema")
    claims = doc.get("claims") if isinstance(doc.get("claims"), dict) else {}
    for key, expected in {
        "public_wedge_native_creator_contract_closed": True,
        "durable_gate_reproducible": True,
        "ai_prompt_current_ssot_golden_closed": True,
        "fixed64_no_float_core_gate_closed": True,
        "creator_gaji_round0_green_baseline": True,
        "r0_through_r7_complete": True,
        "overall_goal_complete": True,
        "behavior_closed": True,
        "docs_closed": True,
        "ready_to_start_external_creator_round0": True,
        "external_creator_pilot_started": False,
        "external_creator_pilot_success_count": 0,
        "independent_review_requested": True,
        "independent_review_completed": False,
        "docs_ssot_change_count": 0,
        "state_hash_formula_change": False,
        "unrelated_dirty_and_untracked_preserved": True,
    }.items():
        if claims.get(key) != expected:
            failures.append(f"claim:{key}")
    correction = doc.get("correction") if isinstance(doc.get("correction"), dict) else {}
    for key, expected in {
        "baseline_head": "56862318cae8f1b43bb36d1513c3dcec3cba7ccd",
        "baseline_freshness_status": "FAIL",
        "baseline_failures": ["source_tree_sha256 mismatch", "source_files mismatch"],
        "prior_sync_gate_claim_valid": False,
        "recovery_head": "G1_SELF",
        "recovery_freshness_status": "PASS",
    }.items():
        if correction.get(key) != expected:
            failures.append(f"correction:{key}")
    rounds = doc.get("rounds")
    expected_commits = {
        "R0": "141069d",
        "R1": "0043995",
        "R2": "cd75407",
        "R3": "88dd727",
        "R4": "87b28b0",
        "R5": "42ead21",
        "R6": "662b58c",
        "R7": "SELF",
    }
    if not isinstance(rounds, list) or len(rounds) != 8:
        failures.append("round_count")
    else:
        by_id = {str(row.get("id", "")): row for row in rounds if isinstance(row, dict)}
        if set(by_id) != set(expected_commits):
            failures.append("round_ids")
        else:
            for round_id, commit in expected_commits.items():
                row = by_id[round_id]
                if row.get("commit") != commit:
                    failures.append(f"round_commit:{round_id}")
                if row.get("result") != "PASS":
                    failures.append(f"round_result:{round_id}")
                evidence = ROOT / str(row.get("evidence", ""))
                if not evidence.exists():
                    failures.append(f"round_evidence:{round_id}")
    summary = doc.get("verification_summary") if isinstance(doc.get("verification_summary"), dict) else {}
    for key, expected in {
        "public_wedge_final_runs": 1,
        "durable_final_gate_runs": 2,
        "r2_persistence_component_runs": 3,
        "r2_capability_component_runs": 3,
        "r2_owned_residual_processes": 0,
        "r2_owned_residual_sockets": 0,
        "r2_owned_residual_temp_dirs": 0,
        "creator_four_viewport_runs": 12,
        "sealed_wasm_normal_cases": 16,
        "sealed_wasm_tamper_cases": 2,
        "vol4_cli_wasm_cases": 38,
        "cli_wasm_sync_gate_substeps": 11,
        "ci_sanity_core_lang": "PASS",
    }.items():
        if summary.get(key) != expected:
            failures.append(f"summary:{key}")
    return failures


def require_markers(path: Path, markers: tuple[str, ...], failures: list[str]) -> None:
    if not path.exists():
        failures.append(f"missing:{path.relative_to(ROOT)}")
        return
    text = path.read_text(encoding="utf-8")
    for marker in markers:
        if marker not in text:
            failures.append(f"marker:{path.relative_to(ROOT)}:{marker}")


def main() -> int:
    for path in (PACK / "README.md", AGGREGATE):
        if not path.exists():
            return fail("E_R7_MISSING", str(path.relative_to(ROOT)))
    doc = json.loads(AGGREGATE.read_text(encoding="utf-8"))
    failures = validate_aggregate(doc)

    require_markers(
        IMPL_REPORT,
        (
            "CREATOR_GAJI_ROUND0_GREEN_BASELINE",
            "READY_TO_START_EXTERNAL_CREATOR_ROUND0",
            "외부 창작자 성공 수: **0**",
            "`docs/ssot/**` 변경: **0건**",
            "56862318cae8f1b43bb36d1513c3dcec3cba7ccd",
            "G1_SELF",
        ),
        failures,
    )
    require_markers(
        AUDIT_REQUEST,
        ("독립 검토 요청", "실제 외부 창작자 Round 0는 수행하지 않는다", "662b58c"),
        failures,
    )
    require_markers(
        SESSION_DELTA,
        ("Status: Reflected", "외부 창작자 성공 수: 0", "R0~R7"),
        failures,
    )
    require_markers(
        ROOT / "docs" / "reports" / "impl" / "INDEX.md",
        ("REPORT_CREATOR_GAJI_ROUND0_GREEN_BASELINE_V1_20260717.md",),
        failures,
    )
    require_markers(
        ROOT / "docs" / "reports" / "audit" / "INDEX.md",
        ("REQUEST_CREATOR_GAJI_ROUND0_GREEN_BASELINE_INDEPENDENT_REVIEW_V1_20260717.md",),
        failures,
    )

    for commit in ("141069d", "0043995", "cd75407", "88dd727", "87b28b0", "42ead21", "662b58c"):
        proc = subprocess.run(
            ["git", "cat-file", "-e", f"{commit}^{{commit}}"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if proc.returncode != 0:
            failures.append(f"commit_missing:{commit}")

    mutated = json.loads(json.dumps(doc, ensure_ascii=False))
    mutated["claims"]["external_creator_pilot_success_count"] = 1
    if not validate_aggregate(mutated):
        failures.append("selftest:pilot_overclaim")
    mutated = json.loads(json.dumps(doc, ensure_ascii=False))
    mutated["rounds"][6]["result"] = "FAIL"
    if not validate_aggregate(mutated):
        failures.append("selftest:failed_round")
    mutated = json.loads(json.dumps(doc, ensure_ascii=False))
    mutated["correction"]["prior_sync_gate_claim_valid"] = True
    if not validate_aggregate(mutated):
        failures.append("selftest:invalid_historical_correction")

    if failures:
        return fail("E_R7_AGGREGATE", "; ".join(failures[:12]))
    print("[creator-gaji-round7-handoff-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
