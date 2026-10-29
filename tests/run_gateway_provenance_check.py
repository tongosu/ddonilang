from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "gogae9_w90_meta_universe"


def run(cmd: list[str]) -> None:
    completed = subprocess.run(cmd, cwd=ROOT)
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_bytes(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def sha256_json_obj(value: dict) -> str:
    text = json.dumps(value, ensure_ascii=False, separators=(",", ":"))
    return "sha256:" + hashlib.sha256(text.encode("utf-8")).hexdigest()


def main() -> int:
    run([sys.executable, "tests/run_pack_golden.py", "gogae9_w90_meta_universe"])

    load_report = read_json(PACK / "golden_report.json")
    load_provenance = load_report["source_provenance"]
    assert load_report["source_hash"] == sha256_json_obj(load_provenance)
    assert load_provenance["schema"] == "gateway.source_provenance.v1"
    assert load_provenance["source_kind"] == "gateway_load_sim_options.v1"
    assert load_provenance["clients"] == 4
    assert load_provenance["ticks"] == 3
    assert load_provenance["seed"] == 1
    assert load_provenance["realms"] == 2
    assert load_provenance["tick_hz"] == 60
    assert load_provenance["threads"] == 4

    world_hash = sha256_bytes(PACK / "world.ddn")
    cases = [
        ("golden_report_serve.json", "gateway_serve_input.v1", PACK / "inputs" / "input_001.detjson", None),
        ("golden_report_jsonl.json", "gateway_serve_input.v1", PACK / "inputs" / "input_002.jsonl", None),
        ("golden_report_sam.json", "gateway_serve_input.v1", PACK / "inputs" / "input_sam_v0.json", None),
        ("golden_report_tcp.json", "gateway_serve_listen.v1", None, PACK / "inputs" / "input_002.jsonl"),
        ("golden_report_udp.json", "gateway_serve_listen.v1", None, PACK / "inputs" / "input_002.jsonl"),
    ]
    for report_name, source_kind, input_path, send_path in cases:
        report = read_json(PACK / report_name)
        provenance = report["source_provenance"]
        assert report["source_hash"] == sha256_json_obj(provenance)
        assert provenance["schema"] == "gateway.source_provenance.v1"
        assert provenance["source_kind"] == source_kind
        assert provenance["world_file"] == "pack/gogae9_w90_meta_universe/world.ddn"
        assert provenance["world_hash"] == world_hash
        if input_path is not None:
            assert provenance["input_file"] == str(input_path.relative_to(ROOT)).replace("\\", "/")
            assert provenance["input_hash"] == sha256_bytes(input_path)
        if send_path is not None:
            assert provenance["send_file"] == str(send_path.relative_to(ROOT)).replace("\\", "/")
            assert provenance["send_hash"] == sha256_bytes(send_path)
            assert provenance["listen_proto"] in ("tcp", "udp")
            assert provenance["listen_addr"] == "127.0.0.1:0"
            assert provenance["listen_max_events"] == 4
            assert provenance["listen_timeout_ms"] == 200

    print("gateway_provenance: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
