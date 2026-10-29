from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "pack" / "creator_gaji_round0_green_baseline_v1" / "r0_baseline.detjson"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"E_CREATOR_GAJI_ROUND0_BASELINE: {message}")


def git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )


def main() -> int:
    payload = json.loads(BASELINE.read_text(encoding="utf-8"))
    require(payload.get("schema") == "ddn.creator_gaji_round0_green_baseline.r0.v1", "schema")
    require(payload.get("branch") == "codex/queue-20260706", "branch")
    require(payload.get("start_head") == "adf019cfdf02078c67b3a1f39b334e765bc34ce2", "START_HEAD")
    require(payload.get("stage_clean") is True, "stage was not clean at START_HEAD")
    require(payload.get("docs_ssot_clean") is True, "docs/ssot was not clean at START_HEAD")
    require(payload.get("target_paths_clean_at_start") is True, "target ownership not sealed")
    require(payload.get("independent_review", {}).get("result") == "PASS_WITH_FOLLOW_UP", "review result")
    require(payload.get("external_creator_success_count") == 0, "external creator count must stay zero")

    ancestor = git("merge-base", "--is-ancestor", payload["minimum_ancestor"], payload["start_head"])
    require(ancestor.returncode == 0, "minimum ancestor is not in START_HEAD")
    creator_head = git("merge-base", "--is-ancestor", payload["creator_range_head"], payload["start_head"])
    require(creator_head.returncode == 0, "creator-gaji range is not in START_HEAD")
    ssot = git("status", "--short", "--", "docs/ssot")
    require(ssot.returncode == 0 and not ssot.stdout.strip(), "docs/ssot is dirty")

    reproductions = {row["id"]: row for row in payload.get("reproductions", [])}
    expected_red = {
        "public_one_file_component",
        "public_wedge_aggregate",
        "durable_capability_component",
        "durable_aggregate",
        "ai_prompt_current_ssot",
        "fixed64_no_float_core",
    }
    require(expected_red <= reproductions.keys(), "missing red reproduction")
    for reproduction_id in expected_red:
        row = reproductions[reproduction_id]
        require(row.get("status") == "EXPECTED_RED", f"{reproduction_id} status")
        require(int(row.get("exit_code", 0)) != 0, f"{reproduction_id} exit code")
        require(int(row.get("elapsed_ms", 0)) > 0, f"{reproduction_id} elapsed time")
        require(bool(row.get("failure")), f"{reproduction_id} failure evidence")
    persistence = reproductions.get("durable_persistence_component", {})
    require(persistence.get("status") == "PASS_STANDALONE", "persistence standalone split")
    require(persistence.get("exit_code") == 0, "persistence standalone exit code")

    claims = payload.get("closure_claims", {})
    require(claims and not any(claims.values()), "R0 must not claim final closure")
    resources = payload.get("post_reproduction_owned_resources", {})
    require(resources.get("process_count") == 0, "owned process residual")
    require(resources.get("listening_socket_count") == 0, "owned socket residual")
    print("[creator-gaji-round0-followup-baseline] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
