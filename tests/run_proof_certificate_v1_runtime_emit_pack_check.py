from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

from _teul_cli_freshness import ensure_teul_cli_bin


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "proof_certificate_v1_runtime_emit_v1"


def teul_cli_candidates(root: Path) -> list[Path]:
    suffix = ".exe" if os.name == "nt" else ""
    return [
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        root / "target" / "debug" / f"teul-cli{suffix}",
    ]


def resolve_teul_cli_bin(root: Path) -> Path:
    return ensure_teul_cli_bin(
        root,
        candidates=teul_cli_candidates(root),
        include_which=False,
        build_env={"RUST_MIN_STACK": str(64 * 1024 * 1024)},
    )


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


def validate_profile(work: Path, profile: str) -> None:
    expected = read_json(PACK / "expected" / f"{profile}.summary.detjson")
    proof_path = work / f"{profile}.proof.detjson"
    candidate_path = work / f"{profile}.proof.proof_certificate_v1_candidate.detjson"
    artifact_path = work / f"{profile}.proof.proof_certificate_v1_draft_artifact.detjson"
    proof = read_json(proof_path)
    candidate = read_json(candidate_path)
    artifact = read_json(artifact_path)

    actual = {
        "profile": profile,
        "verified": candidate["verified"],
        "contract_diag_count": candidate["contract_diag_count"],
        "proof_hash": sha256_bytes(proof_path),
        "candidate_hash": sha256_bytes(candidate_path),
        "artifact_hash": sha256_bytes(artifact_path),
        "proof_entry": proof["entry"],
        "proof_subject_hash": candidate["proof_subject_hash"],
        "source_proof_path": candidate["source_proof_path"],
        "source_proof_schema": candidate["source_proof_schema"],
        "source_proof_kind": candidate["source_proof_kind"],
        "proof_state_hash": proof["state_hash"],
        "proof_trace_hash": proof["trace_hash"],
        "candidate_state_hash": candidate["state_hash"],
        "candidate_trace_hash": candidate["trace_hash"],
        "artifact_profile": artifact["profile"],
        "artifact_source_proof_path": artifact["source_proof_path"],
        "shared_shell_key_count": artifact["shared_shell_key_count"],
        "state_delta_key_count": artifact["state_delta_key_count"],
        "state_delta_verified": artifact["state_delta"]["verified"],
        "state_delta_diag_count": artifact["state_delta"]["contract_diag_count"],
    }
    assert actual == expected


def main() -> int:
    exe = resolve_teul_cli_bin(ROOT)
    with tempfile.TemporaryDirectory(prefix="proof_certificate_v1_runtime_emit_pack_") as td:
        work = Path(td)
        shutil.copytree(PACK / "fixtures", work / "fixtures")
        for profile, input_rel in (("clean", "fixtures/input.ddn"), ("abort", "fixtures/input_abort.ddn")):
            run(
                [
                    str(exe),
                    "run",
                    input_rel,
                    "--proof-out",
                    f"{profile}.proof.detjson",
                ],
                cwd=work,
            )
            validate_profile(work, profile)

    print("proof_certificate_v1_runtime_emit_pack: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
