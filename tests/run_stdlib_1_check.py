#!/usr/bin/env python
import subprocess
import sys
from pathlib import Path


REQUIRED_PACKS = [
    "stdlib_text_basics",
    "stdlib_charim_basics",
    "stdlib_range_basics",
    "stdlib_math_basics",
    "stdlib_map_basics",
    "std_grid_cell_read_write_v1",
    "std_grid_bounds_collision_v1",
    "std_input_map_keyboard_v1",
    "std_input_map_web_snapshot_v1",
    "stdlib_1_v1",
]


def run(cmd: list[str], root: Path) -> None:
    proc = subprocess.run(cmd, cwd=root, text=True, encoding="utf-8", errors="replace")
    if proc.returncode != 0:
        raise SystemExit(proc.returncode)


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    missing = [name for name in REQUIRED_PACKS if not (root / "pack" / name / "golden.jsonl").exists()]
    if missing:
        print(f"[stdlib-1-check] fail missing_packs={','.join(missing)}", file=sys.stderr)
        return 1
    run([sys.executable, "tests/run_pack_golden.py", *REQUIRED_PACKS], root)
    run([sys.executable, "tests/run_stdlib_catalog_check.py"], root)
    print("[stdlib-1-check] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
