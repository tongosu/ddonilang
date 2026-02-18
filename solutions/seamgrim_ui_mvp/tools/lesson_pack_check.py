from __future__ import annotations

from pathlib import Path
import sys
import tomllib


KNOWN_VIEW_SCHEMAS = {
    "seamgrim.graph.v0",
}

KNOWN_GAUGE_SCHEMAS = {
    "seamgrim.graph.v0",
    "seamgrim.table.v0",
    "seamgrim.text.v0",
}


def find_repo_root(start: Path) -> Path:
    for parent in [start] + list(start.parents):
        if (parent / "docs" / "ssot").exists() and (parent / "pack").exists():
            return parent
    raise SystemExit("repo root not found (docs/ssot + pack missing)")


def parse_schema_id(entry: str) -> str:
    return entry.split(":", 1)[0].strip()


def load_toml(path: Path) -> dict:
    data = path.read_bytes()
    return tomllib.loads(data.decode("utf-8"))


def validate_pack(pack_dir: Path) -> list[str]:
    errors: list[str] = []
    meta = pack_dir / "meta.toml"
    view_spec = pack_dir / "view_spec.toml"
    lesson = pack_dir / "lesson.ddn"
    if not meta.exists():
        errors.append(f"{pack_dir}: meta.toml 누락")
    if not view_spec.exists():
        errors.append(f"{pack_dir}: view_spec.toml 누락")
        return errors
    if not lesson.exists():
        errors.append(f"{pack_dir}: lesson.ddn 누락")

    spec = load_toml(view_spec)
    if spec.get("schema") != "SeamgrimViewSpecV0":
        errors.append(f"{pack_dir}: view_spec schema 불일치")

    required_views = spec.get("required_views")
    required_gauges = spec.get("required_gauges")
    if not isinstance(required_views, list) or not required_views:
        errors.append(f"{pack_dir}: required_views 누락/비어있음")
    if not isinstance(required_gauges, list) or not required_gauges:
        errors.append(f"{pack_dir}: required_gauges 누락/비어있음")

    for entry in required_views or []:
        schema = parse_schema_id(str(entry))
        if schema not in KNOWN_VIEW_SCHEMAS:
            errors.append(f"{pack_dir}: 미확인 view schema: {schema}")

    for entry in required_gauges or []:
        schema = parse_schema_id(str(entry))
        if schema not in KNOWN_GAUGE_SCHEMAS:
            errors.append(f"{pack_dir}: 미확인 gauge schema: {schema}")

    return errors


def main() -> int:
    root = find_repo_root(Path(__file__).resolve())
    args = sys.argv[1:]
    if args:
        pack_paths = [root / arg for arg in args]
    else:
        pack_paths = [
            root / "pack" / "edu_s1_function_graph",
            root / "pack" / "edu_p1_constant_accel",
            root / "pack" / "edu_e1_supply_demand_tax",
        ]

    errors: list[str] = []
    for pack_dir in pack_paths:
        if not pack_dir.exists():
            errors.append(f"{pack_dir}: pack 폴더가 없습니다")
            continue
        errors.extend(validate_pack(pack_dir))

    if errors:
        for err in errors:
            print(f"ERROR: {err}")
        return 1

    print("lesson pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
