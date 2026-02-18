#!/usr/bin/env python
import json
import subprocess
from pathlib import Path
import sys


def canonical_json(data) -> str:
    return json.dumps(data, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def load_cases(pack_dir: Path) -> list[dict]:
    lines = pack_dir.joinpath("golden.jsonl").read_text(encoding="utf-8-sig").splitlines()
    cases = []
    for idx, line in enumerate(lines, 1):
        if not line.strip():
            continue
        try:
            data = json.loads(line)
        except json.JSONDecodeError as exc:
            raise ValueError(f"{pack_dir}/golden.jsonl line {idx}: {exc}")
        if "input_path" not in data or "expected_graph" not in data:
            raise ValueError(f"{pack_dir}/golden.jsonl line {idx}: missing input_path/expected_graph")
        cases.append(data)
    return cases


def run_case(root: Path, pack_dir: Path, case: dict) -> tuple[bool, str]:
    export_graph = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "export_graph.py"
    input_path = pack_dir / case["input_path"]
    expected_path = pack_dir / case["expected_graph"]
    temp_output = pack_dir / ".tmp.graph.v0.json"

    cmd = [sys.executable, str(export_graph), str(input_path), str(temp_output)]
    result = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        return False, result.stderr.strip() or result.stdout.strip()

    expected = json.loads(expected_path.read_text(encoding="utf-8"))
    actual = json.loads(temp_output.read_text(encoding="utf-8"))
    temp_output.unlink(missing_ok=True)

    if canonical_json(expected) != canonical_json(actual):
        return False, "graph json mismatch"
    return True, "ok"


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    pack_dir = root / "pack" / "seamgrim_graph_v0_basics"
    if not pack_dir.exists():
        print("missing pack/seamgrim_graph_v0_basics")
        return 1
    cases = load_cases(pack_dir)
    failures = []
    for case in cases:
        ok, detail = run_case(root, pack_dir, case)
        if not ok:
            failures.append((case.get("input_path"), detail))
    if failures:
        for failure in failures:
            print(failure)
        return 1
    print("seamgrim graph golden ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
