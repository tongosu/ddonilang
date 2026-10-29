#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "work" / "beta" / "round0" / "BETA_USER_BUNDLE"
TEUL = ["cargo", "run", "--quiet", "--manifest-path", "tools/teul-cli/Cargo.toml", "--"]


REQUIRED = [
    "START_HERE.md",
    "CLI_ONE_PAGER.md",
    "TASKS.md",
    "FRICTION_LOG_TEMPLATE.detjson",
    "MANIFEST.detjson",
    "samples/pendulum_edit.ddn",
    "samples/falling_ball_starter.ddn",
    "samples/econ_demand_curve.ddn",
    "samples/grid_maze_probe.ddn",
    "fixtures/T4/syntax_bad_identifier.ddn",
    "fixtures/T4/type_bad_iterable.ddn",
    "fixtures/T4/contract_bad_mode.ddn",
    "fixtures/T4/expected_diagnostics.md",
    "session_prompts/P_TEACHER.md",
    "session_prompts/P_STUDENT.md",
    "session_prompts/P_DEVELOPER.md",
]

FORBIDDEN_TEXT = [
    "docs/ssot",
    "docs\\ssot",
    "docs/context",
    "docs\\context",
    "DEV_SUMMARY",
    "PROPOSAL_",
    "SSOT",
    "pack/",
    "pack\\",
]


def fail(message: str) -> int:
    print(f"beta_round0_bundle_check: FAIL: {message}", file=sys.stderr)
    return 1


def run_teul(args: list[str], *, timeout: int = 60) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        TEUL + args,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )


def combined_output(proc: subprocess.CompletedProcess[str]) -> str:
    return (proc.stdout or "") + "\n" + (proc.stderr or "")


def main() -> int:
    missing = [path for path in REQUIRED if not (BUNDLE / path).exists()]
    if missing:
        return fail(f"missing files: {missing}")

    manifest = json.loads((BUNDLE / "MANIFEST.detjson").read_text(encoding="utf-8"))
    if manifest.get("contains_internal_docs") is not False:
        return fail("manifest must explicitly mark internal docs as absent")
    manifest_files = set(manifest.get("files", []))
    if set(REQUIRED) - manifest_files:
        return fail(f"manifest missing required entries: {sorted(set(REQUIRED) - manifest_files)}")

    friction = json.loads((BUNDLE / "FRICTION_LOG_TEMPLATE.detjson").read_text(encoding="utf-8"))
    if friction.get("round") != 0 or "persona" not in friction:
        return fail("friction template schema is incomplete")

    for path in BUNDLE.rglob("*"):
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        for token in FORBIDDEN_TEXT:
            if token in text:
                rel = path.relative_to(ROOT).as_posix()
                return fail(f"forbidden internal reference {token!r} in {rel}")

    isolation_sentence = "제공된 번들 밖 지식으로 우회하지 않습니다"
    for name in ["P_TEACHER.md", "P_STUDENT.md", "P_DEVELOPER.md"]:
        text = (BUNDLE / "session_prompts" / name).read_text(encoding="utf-8")
        if isolation_sentence not in text:
            return fail(f"knowledge isolation sentence missing in {name}")

    sample_runs = [
        ["check", "work/beta/round0/BETA_USER_BUNDLE/samples/pendulum_edit.ddn"],
        ["run", "work/beta/round0/BETA_USER_BUNDLE/samples/pendulum_edit.ddn", "--madi", "2"],
        ["run", "work/beta/round0/BETA_USER_BUNDLE/samples/falling_ball_starter.ddn", "--madi", "2"],
        ["run", "work/beta/round0/BETA_USER_BUNDLE/samples/econ_demand_curve.ddn"],
        ["run", "work/beta/round0/BETA_USER_BUNDLE/samples/grid_maze_probe.ddn", "--madi", "2"],
    ]
    for args in sample_runs:
        proc = run_teul(args, timeout=90)
        if proc.returncode != 0:
            return fail(f"sample command failed: {args}\n{combined_output(proc)}")

    fixture_cases = [
        (
            ["check", "work/beta/round0/BETA_USER_BUNDLE/fixtures/T4/syntax_bad_identifier.ddn"],
            ["E_LEX_UNEXPECTED_CHAR", "예상치 못한 문자: ?"],
        ),
        (
            ["run", "work/beta/round0/BETA_USER_BUNDLE/fixtures/T4/type_bad_iterable.ddn"],
            ["E_RUNTIME_TYPE_MISMATCH", "iterable list/set/map"],
        ),
        (
            ["check", "work/beta/round0/BETA_USER_BUNDLE/fixtures/T4/contract_bad_mode.ddn"],
            ["E_PARSE_UNEXPECTED_TOKEN", "알림", "물림", "중단"],
        ),
    ]
    for args, expected_parts in fixture_cases:
        proc = run_teul(args, timeout=90)
        output = combined_output(proc)
        if proc.returncode == 0:
            return fail(f"fixture unexpectedly passed: {args}")
        missing_parts = [part for part in expected_parts if part not in output]
        if missing_parts:
            return fail(f"fixture output missing {missing_parts}: {args}\n{output}")

    print("beta_round0_bundle_check: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
