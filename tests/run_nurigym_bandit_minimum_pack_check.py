#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "NURIGYM_BANDIT_MINIMUM_PACK_V1.md"
PACK = ROOT / "pack" / "nuri_gym_bandit_v1"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )


def require_files() -> int:
    required = [
        DOC,
        QUEUE,
        PACK / "golden.jsonl",
        PACK / "input.json",
        PACK / "input_alt.json",
        PACK / "golden.txt",
        PACK / "golden_alt.txt",
        ROOT / "core" / "src" / "nurigym" / "bandit.rs",
        ROOT / "tests" / "run_nurigym_cartpole_pendulum_expected_refresh_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_NURIGYM_BANDIT_MINIMUM_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_doc() -> int:
    return require_tokens(
        DOC,
        [
            "NURIGYM_BANDIT_MINIMUM_PACK_V1",
            "nurigym.bandit1d",
            "arm_left",
            "arm_right",
            "preferred action",
            "dataset_hash=sha256:d9b4d27663acfe7fa8875587bc954055d6e8b1a2d8fff0b4c453e30f8773fe5b",
            "dataset_hash=sha256:65ea5c5fbee279305fa1dd43c7aa26f7ad1220285a668f734748a98b92ecad2d",
            "ROADMAP_V2_A2_NURIGYM_FINAL_CLOSURE_RECHECK_V1",
            "does not claim",
            "Python/Web parity",
            "dataset registry",
            "training workflow",
        ],
        "E_NURIGYM_BANDIT_MINIMUM_DOC",
    )


def check_pack_files() -> int:
    expected = {
        PACK / "golden.txt": "dataset_hash=sha256:d9b4d27663acfe7fa8875587bc954055d6e8b1a2d8fff0b4c453e30f8773fe5b",
        PACK / "golden_alt.txt": "dataset_hash=sha256:65ea5c5fbee279305fa1dd43c7aa26f7ad1220285a668f734748a98b92ecad2d",
    }
    for path, value in expected.items():
        if read(path).strip() != value:
            return fail("E_NURIGYM_BANDIT_MINIMUM_GOLDEN", str(path.relative_to(ROOT)))
    golden = read(PACK / "golden.jsonl")
    for token in ["nurigym", "run", "nuri_gym_bandit_v1", "golden.txt", "golden_alt.txt"]:
        if token not in golden:
            return fail("E_NURIGYM_BANDIT_MINIMUM_GOLDEN_JSONL", token)
    for rel in ["RUN_LOG.txt", "SHA256SUMS.txt"]:
        if (PACK / rel).exists():
            return fail("E_NURIGYM_BANDIT_MINIMUM_SIDE_FILE", rel)
    return 0


def run_focused_tests() -> int:
    commands = [
        ["cargo", "test", "--manifest-path", "core/Cargo.toml", "nurigym_bandit", "--", "--nocapture"],
        [
            "cargo",
            "test",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "nurigym_bandit",
            "--",
            "--nocapture",
        ],
    ]
    for command in commands:
        result = run(command)
        if result.returncode != 0:
            return fail("E_NURIGYM_BANDIT_MINIMUM_FOCUSED_TEST", result.stdout.strip())
        if "running 0 tests" in result.stdout or "0 tests" in result.stdout:
            return fail("E_NURIGYM_BANDIT_MINIMUM_ZERO_TESTS", " ".join(command))
    return 0


def run_pack_golden() -> int:
    result = run(["python", "tests/run_pack_golden.py", "nuri_gym_bandit_v1"])
    if result.returncode != 0:
        return fail("E_NURIGYM_BANDIT_MINIMUM_PACK_GOLDEN", result.stdout.strip())
    return 0


def check_dataset(build_dir: Path, expected_count: int, expected_agent: int, preferred: int) -> int:
    dataset = build_dir / "nurigym.dataset.jsonl"
    action_spec = build_dir / "action_spec.detjson"
    obs_spec = build_dir / "obs_spec.detjson"
    for path in [dataset, action_spec, obs_spec]:
        if not path.exists():
            return fail("E_NURIGYM_BANDIT_MINIMUM_BUILD_MISSING", str(path.relative_to(ROOT)))

    header, *steps = [json.loads(line) for line in read(dataset).splitlines() if line.strip()]
    if header.get("schema") != "nurigym.dataset.v1":
        return fail("E_NURIGYM_BANDIT_MINIMUM_HEADER_SCHEMA", str(header))
    if header.get("env_id") != "nurigym.bandit1d":
        return fail("E_NURIGYM_BANDIT_MINIMUM_ENV", str(header.get("env_id")))
    if header.get("count") != expected_count:
        return fail("E_NURIGYM_BANDIT_MINIMUM_COUNT", str(header.get("count")))
    if header.get("agent_ids") != [expected_agent]:
        return fail("E_NURIGYM_BANDIT_MINIMUM_AGENT", str(header.get("agent_ids")))
    if len(steps) != expected_count:
        return fail("E_NURIGYM_BANDIT_MINIMUM_STEP_COUNT", str(len(steps)))

    rewards = []
    for idx, step in enumerate(steps):
        obs = step["observation"]
        next_obs = step["next_observation"]
        action = int(step["action"]["value"])
        reward = int(step["reward"])
        if obs["slot_count"] != 2 or next_obs["slot_count"] != 2:
            return fail("E_NURIGYM_BANDIT_MINIMUM_SLOT_COUNT", str(step))
        if obs["values"][0] != idx or obs["values"][1] != preferred:
            return fail("E_NURIGYM_BANDIT_MINIMUM_OBS", str(obs))
        if next_obs["values"][0] != idx + 1 or next_obs["values"][1] != preferred:
            return fail("E_NURIGYM_BANDIT_MINIMUM_NEXT_OBS", str(next_obs))
        if reward != (1 if action == preferred else 0):
            return fail("E_NURIGYM_BANDIT_MINIMUM_REWARD", str(step))
        rewards.append(reward)
    if not steps[-1]["done"]:
        return fail("E_NURIGYM_BANDIT_MINIMUM_DONE", str(steps[-1]))
    if "arm_left" not in read(action_spec) or "arm_right" not in read(action_spec):
        return fail("E_NURIGYM_BANDIT_MINIMUM_ACTION_SPEC", read(action_spec))
    if '"slot_count":2' not in read(obs_spec):
        return fail("E_NURIGYM_BANDIT_MINIMUM_OBS_SPEC", read(obs_spec))
    if 1 not in rewards or 0 not in rewards:
        return fail("E_NURIGYM_BANDIT_MINIMUM_REWARD_COVERAGE", str(rewards))
    return 0


def check_generated_outputs() -> int:
    checks = [
        (ROOT / "build" / "nuri_gym_bandit_v1", 4, 0, 1),
        (ROOT / "build" / "nuri_gym_bandit_v1_alt", 3, 4, -1),
    ]
    for args in checks:
        rc = check_dataset(*args)
        if rc:
            return rc
    return 0


def check_product_dispatch() -> int:
    text = read(ROOT / "tools" / "teul-cli" / "src" / "cli" / "nurigym.rs")
    for token in ["nurigym.bandit1d", "run_bandit", "BanditEnv", "arm_left", "arm_right"]:
        if token not in text:
            return fail("E_NURIGYM_BANDIT_MINIMUM_DISPATCH", token)
    core_mod = read(ROOT / "core" / "src" / "nurigym" / "mod.rs")
    if "pub mod bandit;" not in core_mod:
        return fail("E_NURIGYM_BANDIT_MINIMUM_CORE_MOD", "pub mod bandit missing")
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "NURIGYM_BANDIT_MINIMUM_PACK_V1",
        "closed by `NURIGYM_BANDIT_MINIMUM_PACK_V1.md`",
        "nuri_gym_bandit_v1",
        "ROADMAP_V2_A2_NURIGYM_FINAL_CLOSURE_RECHECK_V1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "approval-gated",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_NURIGYM_BANDIT_MINIMUM_QUEUE", str(missing))
    if "1. `NURIGYM_BANDIT_MINIMUM_PACK_V1`" in text:
        return fail("E_NURIGYM_BANDIT_MINIMUM_STILL_NEXT", "Bandit item is still next")
    return 0


def check_docs_ssot_clean() -> int:
    result = run(["git", "status", "--short", "--", "docs/ssot"])
    if result.returncode != 0:
        return fail("E_NURIGYM_BANDIT_MINIMUM_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_NURIGYM_BANDIT_MINIMUM_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc,
        check_pack_files,
        check_product_dispatch,
        run_focused_tests,
        run_pack_golden,
        check_generated_outputs,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[nurigym-bandit-minimum-pack-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
