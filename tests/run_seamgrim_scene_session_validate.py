#!/usr/bin/env python
import argparse
import json
from pathlib import Path
import sys


def fail(msg: str) -> None:
    print(msg)
    raise SystemExit(1)


def require_keys(obj: dict, keys: list[str], label: str) -> None:
    for key in keys:
        if key not in obj:
            fail(f"missing {label} field: {key}")


def validate_scene(path: Path) -> None:
    data = json.loads(path.read_text(encoding="utf-8"))
    if data.get("schema") != "seamgrim.scene.v0":
        fail(f"{path}: schema mismatch")
    require_keys(data, ["fps", "sections"], f"{path}")
    if not isinstance(data["sections"], list):
        fail(f"{path}: sections must be list")
    for section in data["sections"]:
        if not isinstance(section, dict):
            fail(f"{path}: section must be object")
        require_keys(section, ["id", "tick_from", "tick_to", "commands"], f"{path}")
        if not isinstance(section["commands"], list):
            fail(f"{path}: section.commands must be list")
        for cmd in section["commands"]:
            if not isinstance(cmd, dict):
                fail(f"{path}: command must be object")
            require_keys(cmd, ["verb", "target"], f"{path}")
            if not isinstance(cmd["target"], dict):
                fail(f"{path}: command.target must be object")


def validate_session(path: Path) -> None:
    data = json.loads(path.read_text(encoding="utf-8"))
    if data.get("schema") != "seamgrim.session.v0":
        fail(f"{path}: schema mismatch")
    require_keys(
        data,
        [
            "inputs",
            "layers",
            "required_views",
            "layout_preset",
            "graph_ref",
            "scene_ref",
            "cursor",
        ],
        f"{path}",
    )
    if not isinstance(data["inputs"], dict):
        fail(f"{path}: inputs must be object")
    if not isinstance(data["layers"], list):
        fail(f"{path}: layers must be list")
    if not isinstance(data["required_views"], list):
        fail(f"{path}: required_views must be list")


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate seamgrim scene/session expected JSON")
    parser.add_argument(
        "packs",
        nargs="*",
        help="pack lesson directories (default: pack/edu_pilot_phys_econ/lesson_phys_01)",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    if args.packs:
        pack_dirs = [Path(p) for p in args.packs]
    else:
        pack_dirs = [root / "pack" / "edu_pilot_phys_econ" / "lesson_phys_01"]

    for pack in pack_dirs:
        scene_path = pack / "expected.scene.v0.json"
        session_path = pack / "expected.session.v0.json"
        if not scene_path.exists() or not session_path.exists():
            fail(f"missing expected scene/session in {pack}")
        validate_scene(scene_path)
        validate_session(session_path)

    print("seamgrim scene/session validation ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
