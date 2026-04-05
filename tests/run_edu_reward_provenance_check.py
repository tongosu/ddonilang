from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
EDU_PACK = ROOT / "pack" / "gogae8_w83_edu_accuracy_v1"
REWARD_PACK = ROOT / "pack" / "gogae8_w81_reward_v1"


def run(cmd: list[str]) -> None:
    completed = subprocess.run(cmd, cwd=ROOT)
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)


def run_teul(args: list[str]) -> None:
    run(
        [
            "cargo",
            "run",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            *args,
        ]
    )


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_text(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_text(encoding="utf-8").encode("utf-8")).hexdigest()


def sha256_bytes(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    run(
        [
            sys.executable,
            "tests/run_pack_golden.py",
            "gogae8_w81_reward_v1",
            "gogae8_w83_edu_accuracy_v1",
        ]
    )

    run_teul(
        [
            "edu",
            "accuracy",
            "pack/gogae8_w83_edu_accuracy_v1/scenario_accel.json",
            "--out",
            "pack/gogae8_w83_edu_accuracy_v1/out_accel",
        ]
    )
    run_teul(
        [
            "edu",
            "accuracy",
            "pack/gogae8_w83_edu_accuracy_v1/scenario_pendulum.json",
            "--out",
            "pack/gogae8_w83_edu_accuracy_v1/out_pendulum",
        ]
    )
    run_teul(
        [
            "reward",
            "check",
            "pack/gogae8_w81_reward_v1/input_ok.json",
            "--out",
            "pack/gogae8_w81_reward_v1/out_ok",
        ]
    )

    for scenario_name, out_name in (
        ("scenario_accel.json", "out_accel"),
        ("scenario_pendulum.json", "out_pendulum"),
    ):
        input_path = EDU_PACK / scenario_name
        report_path = EDU_PACK / out_name / "edu.report.detjson"
        report = read_json(report_path)
        input_hash = sha256_bytes(input_path)
        assert report["source_hash"] == input_hash
        assert report["source_provenance"]["schema"] == "edu.source_provenance.v1"
        assert report["source_provenance"]["source_kind"] == "edu_scenario.v1"
        assert report["source_provenance"]["input_file"] == scenario_name
        assert report["source_provenance"]["input_hash"] == input_hash

    reward_input = REWARD_PACK / "input_ok.json"
    reward_log_path = REWARD_PACK / "out_ok" / "reward.log.detjson"
    reward_report_path = REWARD_PACK / "out_ok" / "reward.report.detjson"
    reward_log = read_json(reward_log_path)
    reward_report = read_json(reward_report_path)
    reward_input_hash = sha256_bytes(reward_input)
    reward_log_hash = sha256_text(reward_log_path)

    assert reward_log["source_hash"] == reward_input_hash
    assert reward_log["source_provenance"]["schema"] == "reward.source_provenance.v1"
    assert reward_log["source_provenance"]["source_kind"] == "reward_check.v1"
    assert reward_log["source_provenance"]["input_file"] == "input_ok.json"
    assert reward_log["source_provenance"]["input_hash"] == reward_input_hash

    assert reward_report["source_hash"] == reward_input_hash
    assert reward_report["source_provenance"]["schema"] == "reward.source_provenance.v1"
    assert reward_report["source_provenance"]["source_kind"] == "reward_check.v1"
    assert reward_report["source_provenance"]["input_file"] == "input_ok.json"
    assert reward_report["source_provenance"]["input_hash"] == reward_input_hash
    assert reward_report["source_provenance"]["log_hash"] == reward_log_hash

    print("edu_reward_provenance: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
