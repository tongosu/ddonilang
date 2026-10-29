from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACKS = [
    "rpg_player_actor_smoke_v1",
    "rpg_player_damage_signal_v1",
    "rpg_player_death_transition_v1",
    "rpg_enemy_spawn_signal_v1",
    "rpg_item_pickup_signal_v1",
    "rpg_npc_dialogue_signal_v1",
    "rpg_level_up_cascade_v1",
]


def run(cmd: list[str]) -> None:
    completed = subprocess.run(cmd, cwd=ROOT)
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)


def load_golden_stdout(pack_name: str) -> list[str]:
    lines = [
        json.loads(line)
        for line in (ROOT / "pack" / pack_name / "golden.jsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    assert len(lines) == 1
    return lines[0]["stdout"]


def main() -> int:
    run([sys.executable, "tests/run_pack_golden.py", *PACKS])

    assert load_golden_stdout("rpg_player_actor_smoke_v1") == ["80"]
    assert load_golden_stdout("rpg_player_damage_signal_v1") == ["75"]
    assert load_golden_stdout("rpg_player_death_transition_v1") == ["0", "사망"]
    assert load_golden_stdout("rpg_enemy_spawn_signal_v1") == ["2", "박쥐", "15"]
    assert load_golden_stdout("rpg_item_pickup_signal_v1") == ["3", "2", "철검"]
    assert load_golden_stdout("rpg_npc_dialogue_signal_v1") == ["2", "안녕 용사 방문=2"]
    assert load_golden_stdout("rpg_level_up_cascade_v1") == ["120", "2", "1", "레벨업"]
    print("rpg_phase2_signal_gate: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
