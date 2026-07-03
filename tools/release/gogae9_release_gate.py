#!/usr/bin/env python
from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def sha256_bytes(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def run_cmd(args: list[str], timeout: int = 300) -> dict:
    proc = subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )
    output = proc.stdout or ""
    return {
        "cmd": args,
        "ok": proc.returncode == 0,
        "exit_code": proc.returncode,
        "started_at_utc": "1980-01-01T00:00:00+00:00",
        "output_hash": sha256_bytes(output.encode("utf-8")),
        "output_tail": output.splitlines()[-8:],
    }


def teul_cli_cmd(args: list[str]) -> list[str]:
    suffix = ".exe" if os.name == "nt" else ""
    for candidate in [
        Path("I:/home/urihanl/ddn/codex/target/debug") / f"teul-cli{suffix}",
        Path("C:/ddn/codex/target/debug") / f"teul-cli{suffix}",
        ROOT / "target" / "debug" / f"teul-cli{suffix}",
    ]:
        if candidate.exists():
            return [str(candidate), *args]
    return ["cargo", "run", "-q", "--manifest-path", str(ROOT / "tools" / "teul-cli" / "Cargo.toml"), "--", *args]


def extract_prefixed(lines: list[str], prefix: str) -> str:
    for line in lines:
        if line.startswith(prefix):
            return line[len(prefix):].strip()
    return ""


def release_commands() -> list[list[str]]:
    py = sys.executable
    return [
        [py, "tests/run_pack_golden.py", "gogae9_w89_self_evolving_code", "gogae9_w90_meta_universe", "gogae9_w91_malmoi_docset", "gogae9_w92_aot_compiler_v2", "gogae9_w93_universe_gui", "gogae9_w94_social_sim", "gogae9_w95_cert", "gogae9_w96_somssi_hub", "gogae9_w97_self_heal", "gogae9_w99_evolving_universe"],
        [py, "tests/run_w89_self_evolving_code_pack_check.py"],
        [py, "tests/run_w92_aot_pack_check.py"],
        [py, "tests/run_w93_universe_pack_check.py"],
        [py, "tests/run_w94_social_pack_check.py"],
        [py, "tests/run_w95_cert_pack_check.py"],
        [py, "tests/run_w96_somssi_pack_check.py"],
        [py, "tests/run_w97_self_heal_pack_check.py"],
        [py, "tests/run_w99_evolving_universe_pack_check.py"],
    ]


def build_manifest(out_dir: Path) -> int:
    out_dir.mkdir(parents=True, exist_ok=True)
    rows = [run_cmd(cmd, timeout=600) for cmd in release_commands()]
    overall_pass = all(row["ok"] for row in rows)
    pack_results = {
        "schema": "ddn.gogae9.w98.pack_results.v1",
        "generated_at_utc": "1980-01-01T00:00:00+00:00",
        "suite": "gogae9_w89_w99_release_gate",
        "overall_pass": overall_pass,
        "rows": rows,
    }
    pack_results_path = out_dir / "pack_results.detjson"
    write_json(pack_results_path, pack_results)
    pack_results_hash = sha256_file(pack_results_path)

    cert_dir = out_dir / "cert"
    keygen = run_cmd(teul_cli_cmd(["cert", "keygen", "--out", str(cert_dir), "--seed", "gogae9-w98-release"]), timeout=120)
    sign = run_cmd(teul_cli_cmd(["cert", "sign", "--in", str(pack_results_path), "--key", str(cert_dir / "cert_private.key"), "--out", str(out_dir / "pack_results.cert.json")]), timeout=120)
    verify = run_cmd(teul_cli_cmd(["cert", "verify", "--in", str(out_dir / "pack_results.cert.json")]), timeout=120)
    cert_subject_hash = extract_prefixed(sign["output_tail"], "cert_subject_hash=")
    cert_pubkey = extract_prefixed(sign["output_tail"], "cert_pubkey=")
    if not keygen["ok"] or not sign["ok"] or not verify["ok"]:
        overall_pass = False

    manifest_seed = {
        "schema": "ddn.gogae9.w98.release_gate_manifest.v1",
        "generated_at_utc": "1980-01-01T00:00:00+00:00",
        "release": "v14.0-gate0",
        "ssot_reference": "docs/ssot/walks/gogae9/w98_release_v14/README.md",
        "included_steps": ["W89", "W90", "W91", "W92", "W93", "W94", "W95", "W96", "W97", "W98", "W99"],
        "overall_pass": overall_pass,
        "pack_results": "pack_results.detjson",
        "pack_results_hash": pack_results_hash,
        "workspace_bundle_hash": sha256_bytes(json.dumps(rows, ensure_ascii=False, sort_keys=True).encode("utf-8")),
        "cert_proof": "pack_results.cert.json",
        "cert_subject_hash": cert_subject_hash,
        "cert_pubkey": cert_pubkey,
        "cert_verify_pass": verify["ok"],
    }
    release_manifest_hash = sha256_bytes(json.dumps(manifest_seed, ensure_ascii=False, sort_keys=True).encode("utf-8"))
    manifest = dict(manifest_seed)
    manifest["release_manifest_hash"] = release_manifest_hash
    manifest_path = out_dir / "release_manifest.detjson"
    write_json(manifest_path, manifest)

    print(f"release_manifest={manifest_path}")
    print(f"pack_results_hash={pack_results_hash}")
    print(f"cert_subject_hash={cert_subject_hash}")
    print(f"release_manifest_hash={release_manifest_hash}")
    if not overall_pass:
        for row in rows:
            if not row["ok"]:
                print(f"failed_cmd={' '.join(row['cmd'])}", file=sys.stderr)
                for line in row["output_tail"]:
                    print(line, file=sys.stderr)
        return 1
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Build and verify the gogae9 W98 release gate manifest.")
    parser.add_argument("--out", default="build/release/gogae9_w98_release_gate")
    args = parser.parse_args()
    out = Path(args.out)
    if not out.is_absolute():
        out = ROOT / out
    return build_manifest(out)


if __name__ == "__main__":
    raise SystemExit(main())
