from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "gogae8_w88_bundle_hash_parity"


def run(cmd: list[str]) -> None:
    completed = subprocess.run(cmd, cwd=ROOT)
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_bytes(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def sha256_json_obj(value: dict) -> str:
    text = json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
    return "sha256:" + hashlib.sha256(text.encode("utf-8")).hexdigest()


def main() -> int:
    run([sys.executable, "tests/run_pack_golden.py", "gogae8_w88_bundle_hash_parity"])
    run(
        [
            "cargo",
            "run",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            "bundle",
            "parity",
            "pack/gogae8_w88_bundle_hash_parity/bundle_in",
            "pack/gogae8_w88_bundle_hash_parity/inputs/inputs_001.detjson",
            "--out",
            "pack/gogae8_w88_bundle_hash_parity/out",
            "--wasm-hash",
            "pack/gogae8_w88_bundle_hash_parity/expect/outputs_hash_001.txt",
        ]
    )

    manifest = read_json(PACK / "bundle_in" / "manifest.detjson")
    provenance = manifest["source_provenance"]
    assert manifest["source_hash"] == sha256_json_obj(provenance)
    assert provenance["schema"] == "seulgi.bundle_source_provenance.v1"
    assert provenance["source_kind"] == "bundle_parity_inputs.v1"
    assert provenance["model_file"] == "model_mlp_v1.detjson"
    assert provenance["weights_file"] == "weights_v1.detbin"
    assert provenance["eval_report_file"] == "eval_report.detjson"
    assert provenance["eval_config_file"] == "eval_config.json"
    assert provenance["artifact_file"] == "artifact.detjson"
    assert provenance["model_hash"] == sha256_bytes(PACK / "bundle_in" / "model_mlp_v1.detjson")
    assert provenance["weights_hash"] == sha256_bytes(PACK / "bundle_in" / "weights_v1.detbin")
    assert provenance["eval_report_hash"] == sha256_bytes(PACK / "bundle_in" / "eval_report.detjson")
    assert provenance["eval_config_hash"] == sha256_bytes(PACK / "eval_config.json")
    assert provenance["artifact_hash"] == sha256_bytes(PACK / "artifact.detjson")
    assert manifest["artifact_hash"] == provenance["artifact_hash"]
    assert manifest["eval_report_hash"] == provenance["eval_report_hash"]

    print("seulgi_bundle_provenance: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
