#!/usr/bin/env python
import argparse
import json
from pathlib import Path


def fail(msg: str) -> None:
    print(msg)
    raise SystemExit(1)


def require_keys(obj: dict, keys: list[str], label: str) -> None:
    for key in keys:
        if key not in obj:
            fail(f"missing {label} field: {key}")


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def validate_scene(path: Path, required_views: set[str]) -> None:
    data = load_json(path)
    if data.get("schema") != "seamgrim.scene.v0":
        fail(f"{path}: schema mismatch")
    require_keys(data, ["fps", "sections"], f"{path}")
    if not isinstance(data["sections"], list):
        fail(f"{path}: sections must be list")
    if not isinstance(data.get("bindings", []), list):
        fail(f"{path}: bindings must be list")

    prev_tick_to = None
    seen_ids = set()
    for section in data["sections"]:
        if not isinstance(section, dict):
            fail(f"{path}: section must be object")
        require_keys(section, ["id", "tick_from", "tick_to", "commands"], f"{path}")
        if section["id"] in seen_ids:
            fail(f"{path}: duplicate section id {section['id']}")
        seen_ids.add(section["id"])
        tick_from = section["tick_from"]
        tick_to = section["tick_to"]
        if not isinstance(tick_from, int) or not isinstance(tick_to, int):
            fail(f"{path}: tick_from/tick_to must be int")
        if tick_from > tick_to:
            fail(f"{path}: tick_from > tick_to in {section['id']}")
        if prev_tick_to is not None and tick_from < prev_tick_to:
            fail(f"{path}: overlapping sections at {section['id']}")
        prev_tick_to = tick_to
        if not isinstance(section["commands"], list):
            fail(f"{path}: section.commands must be list")
        for cmd in section["commands"]:
            if not isinstance(cmd, dict):
                fail(f"{path}: command must be object")
            require_keys(cmd, ["verb", "target"], f"{path}")
            target = cmd["target"]
            if not isinstance(target, dict):
                fail(f"{path}: command.target must be object")
            view = target.get("view")
            if view is not None and view not in required_views:
                fail(f"{path}: target view {view} not in required_views")

    for bind in data.get("bindings", []):
        if not isinstance(bind, dict):
            fail(f"{path}: binding must be object")
        from_view = bind.get("from", {}).get("view") if isinstance(bind.get("from"), dict) else None
        to_view = bind.get("to", {}).get("view") if isinstance(bind.get("to"), dict) else None
        if from_view and from_view not in required_views:
            fail(f"{path}: binding from.view {from_view} not in required_views")
        if to_view and to_view not in required_views:
            fail(f"{path}: binding to.view {to_view} not in required_views")


def validate_session(path: Path, meta: dict) -> set[str]:
    data = load_json(path)
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
    required_views = set(data["required_views"])

    if meta:
        meta_views = set(meta.get("required_views", []))
        if meta_views and meta_views != required_views:
            fail(f"{path}: required_views mismatch meta.toml")
        meta_preset = meta.get("layout_preset")
        if meta_preset and meta_preset != data.get("layout_preset"):
            fail(f"{path}: layout_preset mismatch meta.toml")

    return required_views


def load_meta(meta_path: Path) -> dict:
    if not meta_path.exists():
        return {}
    meta = {}
    for line in meta_path.read_text(encoding="utf-8").splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip().strip('"')
        if key == "required_views":
            value = value.strip("[]")
            views = [v.strip().strip('"') for v in value.split(",") if v.strip()]
            meta[key] = views
        else:
            meta[key] = value
    return meta


def validate_pack(pack_dir: Path) -> None:
    scene_path = pack_dir / "expected.scene.v0.json"
    session_path = pack_dir / "expected.session.v0.json"
    graph_path = pack_dir / "expected.graph.v0.json"
    meta_path = pack_dir / "meta.toml"

    if not scene_path.exists() or not session_path.exists():
        fail(f"missing expected scene/session in {pack_dir}")
    if not graph_path.exists():
        fail(f"missing expected graph in {pack_dir}")

    meta = load_meta(meta_path)
    required_views = validate_session(session_path, meta)
    validate_scene(scene_path, required_views)

    session = load_json(session_path)
    if session.get("graph_ref") != graph_path.name:
        fail(f"{pack_dir}: graph_ref should be {graph_path.name}")
    if session.get("scene_ref") != scene_path.name:
        fail(f"{pack_dir}: scene_ref should be {scene_path.name}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate seamgrim scene/session expected JSON")
    parser.add_argument("packs", nargs="*", help="pack lesson directories")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    pack_dirs = [Path(p) for p in args.packs] if args.packs else [root / "pack" / "edu_pilot_phys_econ" / "lesson_phys_01"]

    for pack in pack_dirs:
        validate_pack(pack)

    print("seamgrim scene/session check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
