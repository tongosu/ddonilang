from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PIPELINE_FIELDS = {
    "raw_action",
    "resolved_action",
    "normalized_action",
    "action_status",
    "merge_candidates",
    "merged_action",
    "applied_action",
    "missing_input",
}
PIPELINE_STATUSES = {"맞음", "가만히", "잘림", "거부"}


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


def read_text(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def read_json(path: str) -> dict:
    return json.loads(read_text(path))


def read_jsonl(path: Path) -> list[dict]:
    return [
        json.loads(line)
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]


def write_json(path: Path, payload: dict) -> None:
    path.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


def main() -> int:
    run(
        [
            sys.executable,
            "tests/run_pack_golden.py",
            "nuri_gym_canon_contract_v1",
            "nuri_gym_gridworld_v1",
        ]
    )

    canon_build = ROOT / "build" / "nuri_gym_canon_contract_check"
    grid_build = ROOT / "build" / "nuri_gym_gridworld_check"
    geoul_dir = canon_build / "geoul"
    export_dir = canon_build / "export"

    run_teul(
        [
            "run",
            "pack/nuri_gym_canon_contract_v1/input.ddn",
            "--madi",
            "2",
            "--geoul-out",
            str(geoul_dir),
        ]
    )
    run_teul(
        [
            "dataset",
            "export",
            "--geoul",
            str(geoul_dir),
            "--format",
            "nurigym_v1",
            "--out",
            str(export_dir),
            "--env-id",
            "nurigym.gridmaze2d",
        ]
    )
    run_teul(
        [
            "nurigym",
            "run",
            "pack\\nuri_gym_gridworld_v1\\input.json",
            "--out",
            str(grid_build),
        ]
    )

    geoul_manifest = json.loads((geoul_dir / "manifest.detjson").read_text(encoding="utf-8"))
    header = json.loads((export_dir / "dataset_header.detjson").read_text(encoding="utf-8"))
    episode_lines = [
        json.loads(line)
        for line in (export_dir / "episode_000001.detjsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    dataset_hash = (export_dir / "dataset_hash.txt").read_text(encoding="utf-8").strip()
    assert header["schema"] == "nurigym.dataset_header.v1"
    assert header["version"] == "nurigym_v1"
    assert header["env_id"] == "nurigym.gridmaze2d"
    assert header["source_hash"].startswith("blake3:")
    assert header["source_hash"] == geoul_manifest["audit_hash"]
    assert header["age_target"] == geoul_manifest["age_target_value"]
    assert header["action_space"]["schema"] == "nurigym.action_space.v1"
    assert header["observation_space"]["schema"] == "nurigym.observation_space.v1"
    assert header["source_provenance"]["schema"] == "nurigym.source_provenance.v1"
    assert header["source_provenance"]["audit_hash"] == header["source_hash"]
    assert header["source_provenance"]["entry_file"] == geoul_manifest["entry_file"]
    assert header["source_provenance"]["entry_hash"] == geoul_manifest["entry_hash"]
    assert (
        header["source_provenance"]["age_target_source"]
        == geoul_manifest["age_target_source"]
    )
    assert episode_lines[0]["schema"] == "nurigym.episode.v1"
    assert episode_lines[0]["source_hash"] == header["source_hash"]
    assert episode_lines[0]["step_count"] == len(episode_lines) - 1
    for index, line in enumerate(episode_lines[1:]):
        assert line["schema"] == "nurigym.step.v1"
        assert line["step"] == index
        assert line["state_hash"] == line["observation"]["state_hash"]
        assert line["terminal"] == (index == len(episode_lines) - 2)
    combined = (export_dir / "dataset_header.detjson").read_text(encoding="utf-8").strip()
    combined += "\n"
    combined += (export_dir / "episode_000001.detjsonl").read_text(encoding="utf-8")
    expected_hash = "sha256:" + hashlib.sha256(combined.encode("utf-8")).hexdigest()
    assert dataset_hash == expected_hash

    gridworld_lines = read_jsonl(grid_build / "nurigym.dataset.jsonl")
    assert gridworld_lines[0]["schema"] == "nurigym.dataset.v1"
    assert gridworld_lines[0]["env_id"] == "nurigym.gridmaze2d"
    assert gridworld_lines[0]["source_hash"].startswith("sha256:")
    assert gridworld_lines[0]["source_provenance"]["schema"] == "nurigym.source_provenance.v1"
    assert gridworld_lines[0]["source_provenance"]["source_kind"] == "input_json"
    assert gridworld_lines[0]["source_provenance"]["input_file"] == "input.json"
    assert (
        gridworld_lines[0]["source_provenance"]["input_hash"]
        == gridworld_lines[0]["source_hash"]
    )
    assert gridworld_lines[0]["count"] == len(gridworld_lines) - 1
    assert gridworld_lines[-1]["done"] is True
    assert gridworld_lines[-1]["observation"]["schema"] == "nurigym.obs.v1"

    pipeline_build = ROOT / "build" / "nuri_gym_shared_sync_pipeline_consumer_check"
    pipeline_input = pipeline_build / "input_pipeline.json"
    pipeline_build.mkdir(parents=True, exist_ok=True)
    write_json(
        pipeline_input,
        {
            "env_id": "nurigym.gridmaze2d",
            "seed": 20260404,
            "episode_id": 29,
            "shared_env": True,
            "shared_env_mode": "sync",
            "merge": "priority",
            "noop_action": 0,
            "max_steps": 1,
            "emit_action_pipeline": True,
            "out_of_range_policy": "clip",
            "gridmaze_layouts": [
                {"width": 3, "height": 3, "goal": [2, 2], "obstacles": []}
            ],
            "agents": [
                {"agent_id": 1, "actions": [5]},
                {"agent_id": 2, "actions": [1]},
            ],
        },
    )
    run_teul(
        [
            "nurigym",
            "run",
            str(pipeline_input),
            "--out",
            str(pipeline_build),
        ]
    )
    pipeline_lines = read_jsonl(pipeline_build / "nurigym.dataset.jsonl")
    assert pipeline_lines[0]["schema"] == "nurigym.dataset.v1"
    assert pipeline_lines[0]["env_id"] == "nurigym.gridmaze2d"
    assert pipeline_lines[0]["source_provenance"]["schema"] == "nurigym.source_provenance.v1"
    pipeline_steps = [line for line in pipeline_lines[1:] if line.get("schema") == "nurigym.step.v1"]
    assert pipeline_steps
    observed_statuses: set[str] = set()
    for line in pipeline_steps:
        assert PIPELINE_FIELDS.issubset(set(line.keys()))
        status = str(line["action_status"])
        assert status in PIPELINE_STATUSES
        observed_statuses.add(status)
    assert "잘림" in observed_statuses

    imitation_config = pipeline_build / "imitation_config.json"
    imitation_out = pipeline_build / "imitation_out"
    write_json(
        imitation_config,
        {
            "schema": "seulgi.imitation_config.v1",
            "replay_path": "nurigym.dataset.jsonl",
            "train_seed": 20260404,
            "max_epochs": 2,
            "env_id": "nurigym.gridmaze2d",
        },
    )
    run_teul(["imitation", str(imitation_config), "--out", str(imitation_out)])
    imitation_lines = read_jsonl(imitation_out / "imitation.dataset.jsonl")
    assert imitation_lines[0]["schema"] == "imitation.dataset.v1"
    assert imitation_lines[0]["env_id"] == "nurigym.gridmaze2d"
    assert imitation_lines[0]["count"] == len(imitation_lines) - 1
    assert len(imitation_lines) - 1 == len(pipeline_steps)
    for line in imitation_lines[1:]:
        assert line["schema"] == "imitation.sample.v1"

    print("nuri_gym_contract: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
