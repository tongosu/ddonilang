#!/usr/bin/env python
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import uuid
from pathlib import Path


def ascii_safe(text: str) -> str:
    return text.encode("ascii", errors="backslashreplace").decode("ascii")


def fail(message: str) -> int:
    print(f"[pack-golden-graph-export-selftest] fail: {ascii_safe(message)}")
    return 1


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def canonical_json(data: object) -> str:
    return json.dumps(data, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def run_pack_golden(root: Path, *args: str) -> tuple[int, str]:
    cmd = [sys.executable, "-S", "tests/run_pack_golden.py", *args]
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    detail = proc.stderr.strip() or proc.stdout.strip() or "run_pack_golden failed"
    return proc.returncode, detail


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    source_pack = root / "pack" / "seamgrim_graph_v0_basics"
    source_input = source_pack / "c02_parabola" / "input.ddn"
    source_expected_graph = source_pack / "c02_parabola" / "expected.seamgrim.graph.v0.json"
    if not source_input.exists() or not source_expected_graph.exists():
        return fail("source seamgrim_graph_v0_basics c02 assets missing")

    temp_name = f"_tmp_pack_graph_export_{uuid.uuid4().hex[:8]}"
    temp_update_name = f"_tmp_pack_graph_export_update_{uuid.uuid4().hex[:8]}"
    temp_pack = root / "pack" / temp_name
    temp_update_pack = root / "pack" / temp_update_name
    try:
        absolute_input = source_input.resolve()
        expected_graph_obj = json.loads(source_expected_graph.read_text(encoding="utf-8"))
        write_text(
            temp_pack / "golden.jsonl",
            "\n".join(
                [
                    json.dumps(
                        {
                            "cmd": ["run", str(absolute_input)],
                            "expected_graph": "expected_graph.json",
                            "stdout": ["-1", "1.5", "-0.5", "0.75", "0", "0.5", "0.5", "0.75", "1", "1.5"],
                        },
                        ensure_ascii=False,
                    ),
                    json.dumps(
                        {
                            "cmd": ["run", "input.ddn"],
                            "cwd": "case_rel",
                            "expected_graph": "expected_graph.json",
                            "stdout": ["-1", "1.5", "-0.5", "0.75", "0", "0.5", "0.5", "0.75", "1", "1.5"],
                        },
                        ensure_ascii=False,
                    ),
                ]
            )
            + "\n",
        )
        write_text(
            temp_pack / "expected_graph.json",
            json.dumps(expected_graph_obj, ensure_ascii=False, indent=2) + "\n",
        )
        write_text(
            temp_pack / "case_rel" / "input.ddn",
            source_input.read_text(encoding="utf-8"),
        )

        code, detail = run_pack_golden(root, temp_name)
        if code != 0:
            return fail(f"run_pack_golden(default) failed: {detail}")

        write_text(
            temp_update_pack / "golden.jsonl",
            json.dumps(
                {
                    "cmd": ["run", str(absolute_input)],
                    "expected_graph": "generated/expected_graph.json",
                    "stdout": ["-1", "1.5", "-0.5", "0.75", "0", "0.5", "0.5", "0.75", "1", "1.5"],
                },
                ensure_ascii=False,
            )
            + "\n",
        )
        code, detail = run_pack_golden(root, "--update", temp_update_name)
        if code != 0:
            return fail(f"run_pack_golden(--update) failed: {detail}")

        generated_expected_graph = temp_update_pack / "generated" / "expected_graph.json"
        if not generated_expected_graph.exists():
            return fail("--update graph export expected file not generated")
        generated_obj = json.loads(generated_expected_graph.read_text(encoding="utf-8"))
        if canonical_json(generated_obj) != canonical_json(expected_graph_obj):
            return fail("generated expected_graph mismatch with source expected graph")

        code, detail = run_pack_golden(root, temp_update_name)
        if code != 0:
            return fail(f"run_pack_golden(after --update) failed: {detail}")

    finally:
        shutil.rmtree(temp_pack, ignore_errors=True)
        shutil.rmtree(temp_update_pack, ignore_errors=True)

    print("[pack-golden-graph-export-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
