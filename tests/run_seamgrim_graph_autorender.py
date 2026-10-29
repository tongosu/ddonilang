#!/usr/bin/env python
import argparse
import json
import subprocess
from pathlib import Path


def canonical_json(data) -> str:
    return json.dumps(data, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def run_case(root: Path, pack_dir: Path, case: dict) -> tuple[bool, str]:
    runner = root / "tests" / "seamgrim_graph_autorender_runner.mjs"
    fixture_rel = case.get("fixture")
    expected_rel = case.get("expected_graph")
    prefer_patch = bool(case.get("prefer_patch", False))
    if not isinstance(fixture_rel, str) or not isinstance(expected_rel, str):
        return False, "missing fixture/expected_graph"

    cmd = [
        "node",
        "--no-warnings",
        str(runner),
        str(pack_dir),
        fixture_rel,
        "true" if prefer_patch else "false",
    ]
    result = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        return False, result.stderr.strip() or result.stdout.strip() or "runner failed"

    actual = json.loads(result.stdout)
    expected = json.loads((pack_dir / expected_rel).read_text(encoding="utf-8"))
    if canonical_json(actual) != canonical_json(expected):
        return False, "graph mismatch"
    return True, "ok"


def main() -> int:
    parser = argparse.ArgumentParser(description="Run seamgrim_graph_autorender_v1 golden")
    parser.add_argument("pack", nargs="?", default="seamgrim_graph_autorender_v1")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    pack_dir = root / "pack" / args.pack
    if not pack_dir.exists():
        print(f"missing pack: {pack_dir}")
        return 1
    golden_path = pack_dir / "golden.jsonl"
    if not golden_path.exists():
        print(f"missing golden: {golden_path}")
        return 1

    failures = []
    for idx, line in enumerate(golden_path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            continue
        case = json.loads(line)
        ok, detail = run_case(root, pack_dir, case)
        if not ok:
            failures.append((idx, detail, case))

    if failures:
        for f in failures:
            print(f)
        return 1
    print("seamgrim graph autorender ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
