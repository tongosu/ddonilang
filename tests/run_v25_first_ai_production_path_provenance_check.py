from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "v25_first_ai_production_path_v1"
TRAIN_PACK = ROOT / "pack" / "gogae8_w82_seulgi_train_v2"
EVAL_PACK = ROOT / "pack" / "gogae8_w87_eval_suite_v1"


def run(cmd: list[str]) -> None:
    completed = subprocess.run(cmd, cwd=ROOT)
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_canonical_json(path: Path) -> str:
    value = json.loads(path.read_text(encoding="utf-8"))
    canonical = json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
    return "sha256:" + hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def sha256_bytes(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


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
    run([sys.executable, "tests/run_train_eval_provenance_check.py"])
    run([sys.executable, "tests/run_pack_golden.py", "model_artifact_inference_v1"])
    run([sys.executable, "tests/run_pack_golden.py", "v25_first_ai_production_path_v1"])
    run_teul(
        [
            "dataset",
            "export",
            "--geoul",
            "pack/nuri_gym_canon_contract_v1/out/geoul",
            "--format",
            "nurigym_v1",
            "--out",
            "pack/v25_first_ai_production_path_v1/out/export",
            "--env-id",
            "nurigym.gridmaze2d",
        ]
    )
    run_teul(
        [
            "train",
            "pack/v25_first_ai_production_path_v1/train_config.json",
            "--out",
            "pack/v25_first_ai_production_path_v1/out/train",
        ]
    )
    run_teul(
        [
            "infer",
            "mlp",
            "pack/v25_first_ai_production_path_v1/model.detjson",
            "pack/v25_first_ai_production_path_v1/infer_input.detjson",
            "--out",
            "pack/v25_first_ai_production_path_v1/out/infer",
        ]
    )

    train_config = read_json(PACK / "train_config.json")
    model = read_json(PACK / "model.detjson")
    dataset_hash = (PACK / "expected" / "dataset_hash.txt").read_text(encoding="utf-8").strip()
    train_report = read_json(PACK / "out" / "train" / "train.report.detjson")
    train_artifact = read_json(PACK / "out" / "train" / "artifact.detjson")
    infer_output = read_json(PACK / "out" / "infer" / "infer.output.detjson")

    train_config_hash = sha256_canonical_json(PACK / "train_config.json")
    assert train_config["dataset_hash"] == dataset_hash
    assert train_report["source_hash"] == train_config_hash
    assert train_report["source_provenance"]["dataset_hash"] == dataset_hash
    assert train_artifact["train_config_hash"] == train_config_hash
    assert train_artifact["source_provenance"]["dataset_hash"] == dataset_hash
    assert train_artifact["weights_hash"] == sha256_bytes(PACK / "out" / "train" / "weights.bin")
    assert model["schema"] == "seulgi.mlp.v1"
    assert model["weights_path"] == "out/train/weights.bin"
    assert infer_output["schema"] == "seulgi.infer_output.v1"

    w82_artifact = read_json(TRAIN_PACK / "out" / "artifact.detjson")
    w87_pass_artifact = read_json(EVAL_PACK / "expected_artifact_pass.detjson")
    for artifact in (train_artifact, w82_artifact, w87_pass_artifact):
        assert artifact["schema"] == "seulgi.model_artifact.v1"
        assert "ssot_bundle_hash" in artifact
        assert "weights_hash" in artifact
        assert "source_provenance" in artifact

    print("v25_first_ai_production_path_provenance: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
