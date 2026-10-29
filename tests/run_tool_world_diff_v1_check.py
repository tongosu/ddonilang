#!/usr/bin/env python
from __future__ import annotations

import json
import os
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "tool_world_diff_v1"
LESSON = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_projectile_xy_v1"
ARTIFACT = LESSON / "world_diff_gravity_half.detjson"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def run(cmd: list[str], timeout: int = 300) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def resolve_teul() -> Path | None:
    proc = run(["cargo", "metadata", "--no-deps", "--format-version", "1", "--manifest-path", "tools/teul-cli/Cargo.toml"], 120)
    if proc.returncode != 0:
        return None
    target = Path(json.loads(proc.stdout)["target_directory"])
    if not target.is_absolute():
        target = (ROOT / target).resolve()
    binary = target / "debug" / ("teul-cli.exe" if os.name == "nt" else "teul-cli")
    return binary if binary.exists() else None


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def generate(teul: Path, baseline: Path, candidate: Path, out: Path, madi: int) -> subprocess.CompletedProcess[str]:
    return run([
        str(teul),
        "world-diff",
        str(baseline),
        str(candidate),
        "--madi",
        str(madi),
        "--world-id",
        "physics.projectile",
        "--candidate-origin",
        "ai_fixture",
        "--out",
        str(out),
    ], 180)


def check_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        PACK / "fixtures" / "baseline.ddn",
        PACK / "fixtures" / "candidate.ddn",
        LESSON / "lesson.ddn",
        LESSON / "candidates" / "ai_gravity_half.ddn",
        ARTIFACT,
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "world_diff_review.js",
        ROOT / "tests" / "seamgrim_world_diff_review_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_WORLD_DIFF_MISSING", repr(missing))
    contract = load(PACK / "contract.detjson")
    if contract.get("toolchain_entrypoint") != "teul-cli world-diff" or contract.get("artifact_schema") != "ddn.world_diff.v1":
        return fail("E_WORLD_DIFF_CONTRACT", repr(contract))
    for key in ("automatic_apply", "file_write", "full_ast_diff_claim", "solveplan_diff_claim", "first_divergence_madi_claim", "cli_wasm_state_hash_parity_claim", "docs_ssot_change"):
        if contract.get(key) is not False:
            return fail("E_WORLD_DIFF_OVERCLAIM", f"{key}={contract.get(key)!r}")
    if contract.get("candidate_authority_before_review") != "none" or contract.get("human_confirmation_required") is not True:
        return fail("E_WORLD_DIFF_AUTHORITY", repr(contract))
    return 0


def check_generated(teul: Path) -> int:
    fixture_baseline = PACK / "fixtures" / "baseline.ddn"
    fixture_candidate = PACK / "fixtures" / "candidate.ddn"
    with tempfile.TemporaryDirectory(prefix="ddn_world_diff_v1_") as temp_dir:
        temp = Path(temp_dir)
        same_paths = [temp / "same_a.detjson", temp / "same_b.detjson"]
        changed_paths = [temp / "changed_a.detjson", temp / "changed_b.detjson"]
        for path in same_paths:
            proc = generate(teul, fixture_baseline, fixture_baseline, path, 3)
            if proc.returncode != 0:
                return fail("E_WORLD_DIFF_SAME_RUN", proc.stdout.strip())
        for path in changed_paths:
            proc = generate(teul, fixture_baseline, fixture_candidate, path, 3)
            if proc.returncode != 0:
                return fail("E_WORLD_DIFF_CHANGED_RUN", proc.stdout.strip())
        if same_paths[0].read_bytes() != same_paths[1].read_bytes() or changed_paths[0].read_bytes() != changed_paths[1].read_bytes():
            return fail("E_WORLD_DIFF_BYTES", "artifact generation is not byte deterministic")
        same = load(same_paths[0])
        changed = load(changed_paths[0])
        if same.get("status") != "no_change" or same.get("change_count") != 0 or same.get("changes") != []:
            return fail("E_WORLD_DIFF_SAME_NOT_EMPTY", repr(same))
        if same.get("execution_preview", {}).get("state_hash_equal") is not True:
            return fail("E_WORLD_DIFF_SAME_HASH", repr(same.get("execution_preview")))
        changes = changed.get("changes", [])
        if changed.get("change_count") != 1 or len(changes) != 1:
            return fail("E_WORLD_DIFF_CHANGED_COUNT", repr(changes))
        row = changes[0]
        if (row.get("axis"), row.get("subject"), row.get("before", {}).get("canon"), row.get("after", {}).get("canon")) != ("control_init", "중력가속도", "9.8", "4.9"):
            return fail("E_WORLD_DIFF_CHANGED_AXIS", repr(row))
        if changed.get("authority") != "none" or changed.get("review", {}).get("auto_apply") is not False:
            return fail("E_WORLD_DIFF_CHANGED_AUTHORITY", repr(changed))

        lesson_paths = [temp / "lesson_a.detjson", temp / "lesson_b.detjson"]
        for path in lesson_paths:
            proc = generate(teul, LESSON / "lesson.ddn", LESSON / "candidates" / "ai_gravity_half.ddn", path, 240)
            if proc.returncode != 0:
                return fail("E_WORLD_DIFF_LESSON_RUN", proc.stdout.strip())
        if lesson_paths[0].read_bytes() != lesson_paths[1].read_bytes() or lesson_paths[0].read_bytes() != ARTIFACT.read_bytes():
            return fail("E_WORLD_DIFF_LESSON_BYTES", "committed lesson artifact differs from product command")
    artifact = load(ARTIFACT)
    execution = artifact.get("execution_preview", {})
    if execution.get("engine") != "teul-cli":
        return fail("E_WORLD_DIFF_EXECUTION_ENGINE", repr(execution))
    if execution.get("state_hash_equal") is not False or execution.get("trace_hash_equal") is not False:
        return fail("E_WORLD_DIFF_EXECUTION_PREVIEW", repr(execution))
    if execution.get("first_divergence_madi") is not None or execution.get("first_divergence_status") != "not_computed":
        return fail("E_WORLD_DIFF_DIVERGENCE_OVERCLAIM", repr(execution))
    false_claims = artifact.get("false_claims", {})
    if any(false_claims.get(key) is not False for key in ("ai_call", "automatic_authority", "automatic_apply", "file_write", "full_ast_diff", "solveplan_diff", "first_divergence_madi")):
        return fail("E_WORLD_DIFF_FALSE_CLAIMS", repr(false_claims))
    return 0


def gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout)
    return fail(code, proc.stdout.strip()) if proc.returncode != 0 else 0


def main() -> int:
    rc = check_contract()
    if rc:
        return rc
    teul = resolve_teul()
    if teul is None:
        return fail("E_WORLD_DIFF_TEUL", "teul-cli binary missing")
    rc = check_generated(teul)
    if rc:
        return rc
    gates = [
        (["cargo", "test", "--manifest-path", "tools/teul-cli/Cargo.toml", "world_diff", "--", "--nocapture"], "E_WORLD_DIFF_UNIT", 300),
        (["node", "tests/seamgrim_world_diff_review_runner.mjs"], "E_WORLD_DIFF_BROWSER", 180),
        (["python", "tests/run_pack_golden.py", "tool_world_diff_v1"], "E_WORLD_DIFF_GOLDEN", 300),
        (["python", "tests/run_seulgi_gatekeeper_pack_check.py"], "E_WORLD_DIFF_SEULGI_GATE", 180),
        (["python", "tests/run_seamgrim_projectile_representation_pair_check.py"], "E_WORLD_DIFF_W4_REGRESSION", 900),
        (["python", "tests/run_onmungchi_10_worlds_pilot_check.py"], "E_WORLD_DIFF_ONMUNGCHI", 900),
    ]
    for cmd, code, timeout in gates:
        rc = gate(cmd, code, timeout)
        if rc:
            return rc
    docs = run(["git", "status", "--short", "--", "docs/ssot"], 60)
    if docs.returncode != 0 or docs.stdout.strip():
        return fail("E_WORLD_DIFF_SSOT_DIRTY", docs.stdout.strip())
    print("[tool-world-diff-v1] OK toolchain-behavior-closed review-behavior-closed ai-authority-none")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
