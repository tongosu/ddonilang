from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TRAIN_PACK = ROOT / "pack" / "gogae8_w82_seulgi_train_v2"
EVAL_PACK = ROOT / "pack" / "gogae8_w87_eval_suite_v1"


def run(cmd: list[str]) -> None:
    completed = subprocess.run(cmd, cwd=ROOT)
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_text(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_text(encoding="utf-8").encode("utf-8")).hexdigest()


def sha256_bytes(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def sha256_canonical_json(path: Path) -> str:
    text = path.read_text(encoding="utf-8")
    value = json.loads(text)
    canonical = json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
    return "sha256:" + hashlib.sha256(canonical.encode("utf-8")).hexdigest()


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
    run(
        [
            sys.executable,
            "tests/run_pack_golden.py",
            "gogae8_w82_seulgi_train_v2",
            "gogae8_w87_eval_suite_v1",
        ]
    )
    run_teul(
        [
            "train",
            "pack/gogae8_w82_seulgi_train_v2/train_config.json",
            "--out",
            "pack/gogae8_w82_seulgi_train_v2/out",
        ]
    )
    run_teul(
        [
            "eval",
            "pack/gogae8_w87_eval_suite_v1/eval_config_pass.json",
            "--out",
            "pack/gogae8_w87_eval_suite_v1/out/pass",
        ]
    )
    run_teul(
        [
            "eval",
            "pack/gogae8_w87_eval_suite_v1/eval_config_fail.json",
            "--out",
            "pack/gogae8_w87_eval_suite_v1/out/fail",
        ]
    )

    train_config = TRAIN_PACK / "train_config.json"
    train_report = read_json(TRAIN_PACK / "out" / "train.report.detjson")
    train_artifact = read_json(TRAIN_PACK / "out" / "artifact.detjson")
    train_config_hash = sha256_canonical_json(train_config)
    assert train_report["source_hash"] == train_config_hash
    assert train_report["source_provenance"]["config_file"] == train_config.name
    assert train_report["source_provenance"]["config_hash"] == train_config_hash
    assert train_artifact["source_provenance"]["config_file"] == train_config.name
    assert train_artifact["source_provenance"]["config_hash"] == train_config_hash
    assert train_artifact["source_provenance"]["train_report_hash"] == train_artifact["eval_report_hash"]
    assert train_artifact["weights_hash"] == sha256_bytes(TRAIN_PACK / "out" / "weights.bin")

    for case in ("pass", "fail"):
        config_path = EVAL_PACK / f"eval_config_{case}.json"
        report_path = EVAL_PACK / "out" / case / "eval_report.detjson"
        artifact_path = EVAL_PACK / f"expected_artifact_{case}.detjson"
        config_hash = sha256_text(config_path)
        report = read_json(report_path)
        artifact = read_json(artifact_path)
        config = read_json(config_path)
        weights_path = EVAL_PACK / read_json(EVAL_PACK / config["model_path"])["weights_path"]
        input_artifact_path = EVAL_PACK / config["artifact_path"]
        assert report["source_hash"] == config_hash
        assert report["source_provenance"]["config_file"] == config_path.name
        assert report["source_provenance"]["config_hash"] == config_hash
        assert artifact["source_provenance"]["config_file"] == config_path.name
        assert artifact["source_provenance"]["config_hash"] == config_hash
        assert artifact["source_provenance"]["report_hash"] == report["report_hash"]
        assert artifact["source_provenance"]["artifact_input_hash"] == sha256_text(input_artifact_path)
        assert artifact["source_provenance"]["weights_hash"] == sha256_bytes(weights_path)

    print("train_eval_provenance: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
