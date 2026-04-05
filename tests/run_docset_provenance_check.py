from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "gogae9_w91_malmoi_docset"
OUT = ROOT / "build" / "docset_provenance_check"


def run(cmd: list[str]) -> None:
    completed = subprocess.run(cmd, cwd=ROOT)
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)


def sha256_text(text: str) -> str:
    return "sha256:" + hashlib.sha256(text.encode("utf-8")).hexdigest()


def sha256_bytes(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def sha256_json_obj(value: dict) -> str:
    text = json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
    return sha256_text(text)


def main() -> int:
    run([sys.executable, "tests/run_pack_golden.py", "gogae9_w91_malmoi_docset"])
    if OUT.exists():
        shutil.rmtree(OUT)
    run(
        [
            "cargo",
            "run",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            "doc",
            "verify",
            "--pack",
            "pack/gogae9_w91_malmoi_docset",
            "--out",
            str(OUT),
        ]
    )

    docset = json.loads((OUT / "docset.json").read_text(encoding="utf-8"))
    provenance = docset["source_provenance"]
    assert docset["source_hash"] == sha256_json_obj(provenance)
    assert provenance["schema"] == "malmoi.docset_source_provenance.v1"
    assert provenance["source_kind"] == "docset_sources.v1"
    files = provenance["files"]
    assert provenance["file_count"] == len(files)
    assert len(docset["entries"]) == len(files)

    source_hashes: dict[str, str] = {}
    for item in files:
        path = item["path"]
        source_path = ROOT / Path(path)
        assert source_path.exists(), path
        file_hash = sha256_bytes(source_path)
        assert item["sha256"] == file_hash
        source_hashes[path] = file_hash

    for entry in docset["entries"]:
        assert entry["source_path"] in source_hashes
        assert entry["sha256"] == source_hashes[entry["source_path"]]

    print("docset_provenance: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
