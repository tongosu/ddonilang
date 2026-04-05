from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "proof_certificate_v1_signed_emit_v1"


def resolve_teul_cli_bin(root: Path) -> Path:
    suffix = ".exe" if os.name == "nt" else ""
    candidates = [
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        root / "target" / "debug" / f"teul-cli{suffix}",
    ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    build = subprocess.run(
        ["cargo", "build", "--manifest-path", "tools/teul-cli/Cargo.toml"],
        cwd=root,
        env={**os.environ, "RUST_MIN_STACK": str(64 * 1024 * 1024)},
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if build.returncode != 0:
        raise SystemExit(build.returncode)
    for candidate in candidates:
        if candidate.exists():
            return candidate
    raise FileNotFoundError("missing teul-cli binary")


def run(cmd: list[str], *, cwd: Path) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run(
        cmd,
        cwd=cwd,
        env={**os.environ, "RUST_MIN_STACK": str(64 * 1024 * 1024)},
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if completed.returncode != 0:
        print(completed.stdout, end="")
        print(completed.stderr, end="")
        raise SystemExit(completed.returncode)
    return completed


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_bytes(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    exe = resolve_teul_cli_bin(ROOT)
    with tempfile.TemporaryDirectory(prefix="proof_certificate_v1_signed_emit_pack_") as td:
        work = Path(td)
        shutil.copytree(PACK / "fixtures", work / "fixtures")
        run([str(exe), "cert", "keygen", "--out", "cert", "--seed", "signed-emit-selftest"], cwd=work)
        run(
            [
                str(exe),
                "run",
                "fixtures/input.ddn",
                "--proof-out",
                "signed.proof.detjson",
                "--proof-cert-key",
                "cert/cert_private.key",
            ],
            cwd=work,
        )
        proof_path = work / "signed.proof.detjson"
        cert_path = work / "signed.proof.cert_manifest.detjson"
        bundle_path = work / "signed.proof.proof_certificate_v1.detjson"
        proof = read_json(proof_path)
        cert = read_json(cert_path)
        bundle = read_json(bundle_path)
        actual = {
            "profile": bundle["profile"],
            "verified": bundle["verified"],
            "contract_diag_count": bundle["contract_diag_count"],
            "proof_hash": sha256_bytes(proof_path),
            "cert_manifest_hash": sha256_bytes(cert_path),
            "bundle_hash": sha256_bytes(bundle_path),
            "proof_entry": proof["entry"],
            "proof_subject_hash": bundle["proof_subject_hash"],
            "cert_pubkey": bundle["cert_pubkey"],
            "cert_signature": bundle["cert_signature"],
            "cert_subject_hash": cert["subject_hash"],
            "subject_path": cert["subject_path"],
            "source_proof_path": bundle["source_proof_path"],
            "source_proof_schema": bundle["source_proof_schema"],
            "source_proof_kind": bundle["source_proof_kind"],
            "runtime_candidate_profile": bundle["runtime_candidate"]["profile"],
            "runtime_candidate_verified": bundle["runtime_candidate"]["verified"],
            "runtime_candidate_diag_count": bundle["runtime_candidate"]["contract_diag_count"],
            "runtime_artifact_profile": bundle["runtime_draft_artifact"]["profile"],
            "runtime_artifact_source_proof_path": bundle["runtime_draft_artifact"]["source_proof_path"],
        }
        expected = read_json(PACK / "expected" / "clean.summary.detjson")
        assert actual == expected

    print("proof_certificate_v1_signed_emit_pack: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
