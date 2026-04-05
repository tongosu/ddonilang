from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACKS = ["gogae5_w53_dataset_export", "gogae5_w55_smart_errand_integration"]


def run(cmd: list[str]) -> None:
    completed = subprocess.run(cmd, cwd=ROOT)
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)


def load_first_jsonl(path: Path) -> dict:
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            return json.loads(line)
    raise AssertionError(f"empty jsonl: {path}")


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def run_teul(args: list[str]) -> None:
    run(
        [
            "cargo",
            "run",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            *args,
        ]
    )


def main() -> int:
    run([sys.executable, "tests/run_pack_golden.py", *PACKS])
    run_teul(
        [
            "dataset",
            "export",
            "--geoul",
            "build/geoul_w53",
            "--format",
            "nurigym_v0",
            "--out",
            "build/dataset_w53",
        ]
    )
    run_teul(
        [
            "dataset",
            "export",
            "--geoul",
            "build/geoul_w55",
            "--format",
            "nurigym_v0",
            "--out",
            "build/dataset_w55",
        ]
    )

    cases = [
        ("build/geoul_w53/manifest.detjson", "build/dataset_w53/nurigym.dataset.jsonl"),
        ("build/geoul_w55/manifest.detjson", "build/dataset_w55/nurigym.dataset.jsonl"),
    ]
    for manifest_rel, dataset_rel in cases:
        manifest = load_json(ROOT / manifest_rel)
        header = load_first_jsonl(ROOT / dataset_rel)
        assert header["schema"] == "nurigym.dataset.v1"
        assert header["source_hash"] == manifest["audit_hash"]
        assert header["source_provenance"]["schema"] == "nurigym.source_provenance.v1"
        assert header["source_provenance"]["audit_hash"] == manifest["audit_hash"]
        assert header["source_provenance"]["entry_file"] == manifest["entry_file"]
        assert header["source_provenance"]["entry_hash"] == manifest["entry_hash"]
        assert (
            header["source_provenance"]["age_target_source"]
            == manifest["age_target_source"]
        )
        assert (
            header["source_provenance"]["age_target_value"]
            == manifest["age_target_value"]
        )
    print("dataset_export_v0_provenance: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
