from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = "gogae8_w86_imitation_learning_v1"


def run(cmd: list[str]) -> None:
    completed = subprocess.run(cmd, cwd=ROOT)
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)


def main() -> int:
    run([sys.executable, "tests/run_pack_golden.py", PACK])

    dataset_path = ROOT / "pack" / PACK / "out" / "imitation.dataset.jsonl"
    config_path = ROOT / "pack" / PACK / "imitation_config.json"
    replay_path = ROOT / "pack" / PACK / "replay.jsonl"
    lines = [
        json.loads(line)
        for line in dataset_path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    header = lines[0]
    assert header["schema"] == "imitation.dataset.v1"
    assert header["source_hash"].startswith("sha256:")
    assert header["source_provenance"]["schema"] == "seulgi.source_provenance.v1"
    assert header["source_provenance"]["source_kind"] == "replay_jsonl"
    assert header["source_provenance"]["replay_file"] == replay_path.name
    assert header["source_provenance"]["config_file"] == config_path.name
    import hashlib

    replay_hash = "sha256:" + hashlib.sha256(replay_path.read_bytes()).hexdigest()
    config_hash = "sha256:" + hashlib.sha256(config_path.read_bytes()).hexdigest()
    assert header["source_hash"] == replay_hash
    assert header["source_provenance"]["replay_hash"] == replay_hash
    assert header["source_provenance"]["config_hash"] == config_hash
    assert header["count"] == len(lines) - 1
    print("imitation_provenance: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
