#!/usr/bin/env python
import argparse
import json
import subprocess
import sys
from pathlib import Path


def canonical_json(data) -> str:
    return json.dumps(data, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def run_export_graph(root: Path, input_path: Path, output_path: Path) -> None:
    export_graph = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "export_graph.py"
    cmd = [sys.executable, str(export_graph), str(input_path), str(output_path)]
    result = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        msg = result.stderr.strip() or result.stdout.strip() or "export_graph failed"
        raise RuntimeError(msg)


def check_graph(root: Path, pack_dir: Path, strict: bool, warnings: list[str]) -> None:
    input_path = pack_dir / "lesson.ddn"
    expected_path = pack_dir / "expected.graph.v0.json"
    if not input_path.exists():
        raise RuntimeError(f"missing lesson.ddn in {pack_dir}")
    if not expected_path.exists():
        raise RuntimeError(f"missing expected.graph.v0.json in {pack_dir}")

    temp_output = pack_dir / ".tmp.graph.v0.json"
    try:
        run_export_graph(root, input_path, temp_output)
    except RuntimeError as exc:
        msg = f"graph export failed for {pack_dir}: {exc}"
        if strict:
            raise RuntimeError(msg) from exc
        warnings.append(msg)
        return

    expected = json.loads(expected_path.read_text(encoding="utf-8"))
    actual = json.loads(temp_output.read_text(encoding="utf-8"))
    temp_output.unlink(missing_ok=True)

    if canonical_json(expected) != canonical_json(actual):
        raise RuntimeError(f"graph json mismatch: {pack_dir}")


def check_scene_session(root: Path, pack_dir: Path) -> None:
    checker = root / "tests" / "run_seamgrim_scene_session_check.py"
    cmd = [sys.executable, str(checker), str(pack_dir)]
    result = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        msg = result.stderr.strip() or result.stdout.strip() or "scene/session check failed"
        raise RuntimeError(msg)


def main() -> int:
    parser = argparse.ArgumentParser(description="Run seamgrim graph + scene/session checks")
    parser.add_argument("packs", nargs="*", help="pack lesson directories")
    parser.add_argument("--strict-graph", action="store_true", help="fail if graph export fails")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    if args.packs:
        pack_dirs = [Path(p) for p in args.packs]
    else:
        pack_dirs = [root / "pack" / "edu_pilot_phys_econ" / "lesson_phys_01"]

    warnings: list[str] = []
    for pack_dir in pack_dirs:
        check_graph(root, pack_dir, args.strict_graph, warnings)
        check_scene_session(root, pack_dir)

    if warnings:
        print("seamgrim full check ok (graph warnings)")
        for warning in warnings:
            safe = warning.encode("ascii", "backslashreplace").decode("ascii")
            print(safe)
    else:
        print("seamgrim full check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
