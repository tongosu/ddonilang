#!/usr/bin/env python
from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path


README_PATH = Path("tests/proof_certificate_v1_runtime_emit/README.md")
PACK_README = Path("pack/age4_proof_detjson_smoke_v1/README.md")
DRAFT_CONTRACT_README = Path("tests/proof_certificate_v1_draft_contract/README.md")
PROMOTION_README = Path("tests/proof_certificate_v1_promotion/README.md")
CLEAN_INPUT = Path("pack/age4_proof_detjson_smoke_v1/input.ddn")
ABORT_INPUT = Path("pack/age4_proof_detjson_smoke_v1/input_abort.ddn")

README_SNIPPETS = (
    "## Stable Contract",
    "`pack/age4_proof_detjson_smoke_v1/input.ddn`",
    "`pack/age4_proof_detjson_smoke_v1/input_abort.ddn`",
    "`tests/proof_certificate_v1_draft_contract/README.md`",
    "`tests/proof_certificate_v1_promotion/README.md`",
    "`python tests/run_proof_certificate_v1_runtime_emit_selftest.py`",
    "`proof_certificate_v1_runtime_emit_selftest`",
    "`ddn.proof_certificate_v1_runtime_candidate.v1`",
    "`ddn.proof_certificate_v1_runtime_draft_artifact.v1`",
)
POINTERS = (
    "`tests/proof_certificate_v1_runtime_emit/README.md`",
    "`python tests/run_proof_certificate_v1_runtime_emit_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-v1-runtime-emit-selftest] fail: {message}")
    return 1


def resolve_teul_cli_bin(root: Path) -> Path | None:
    suffix = ".exe" if os.name == "nt" else ""
    candidates = [
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        root / "target" / "debug" / f"teul-cli{suffix}",
    ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return None


def ensure_teul_cli_bin(root: Path) -> Path:
    existing = resolve_teul_cli_bin(root)
    if existing is not None:
        return existing
    build = subprocess.run(
        ["cargo", "build", "--manifest-path", "tools/teul-cli/Cargo.toml"],
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
        env={**os.environ, "RUST_MIN_STACK": str(64 * 1024 * 1024)},
    )
    if build.returncode != 0:
        raise ValueError(
            f"cargo build failed stdout={build.stdout!r} stderr={build.stderr!r}"
        )
    teul_cli_bin = resolve_teul_cli_bin(root)
    if teul_cli_bin is None:
        raise ValueError("missing teul-cli binary after cargo build")
    return teul_cli_bin


def run_teul_cli(
    root: Path, teul_cli_bin: Path, args: list[str]
) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env.setdefault("RUST_MIN_STACK", str(64 * 1024 * 1024))
    cmd = [str(teul_cli_bin), *args]
    return subprocess.run(
        cmd,
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
    )


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def sidecar_path(path: Path, label: str) -> Path:
    file_name = path.name
    stem = file_name[:-8] if file_name.endswith(".detjson") else file_name
    return path.with_name(f"{stem}.{label}.detjson")


def validate_case(
    root: Path,
    teul_cli_bin: Path,
    source: Path,
    *,
    profile: str,
    verified: bool,
    contract_diag_count: int,
) -> None:
    with tempfile.TemporaryDirectory(prefix=f"proof_certificate_v1_runtime_emit_{profile}_") as td:
        td_path = Path(td)
        proof_path = td_path / f"{profile}.proof.detjson"
        candidate_path = sidecar_path(proof_path, "proof_certificate_v1_candidate")
        artifact_path = sidecar_path(proof_path, "proof_certificate_v1_draft_artifact")
        result = run_teul_cli(
            root,
            teul_cli_bin,
            ["run", str(source).replace("\\", "/"), "--proof-out", str(proof_path).replace("\\", "/")],
        )
        if result.returncode != 0:
            raise ValueError(
                f"{profile}: teul-cli run failed stdout={result.stdout!r} stderr={result.stderr!r}"
            )
        if not proof_path.exists():
            raise ValueError(f"{profile}: missing proof output")
        if not candidate_path.exists():
            raise ValueError(f"{profile}: missing candidate sidecar")
        if not artifact_path.exists():
            raise ValueError(f"{profile}: missing artifact sidecar")

        proof_bytes = proof_path.read_bytes()
        proof_doc = load_json(proof_path)
        candidate_doc = load_json(candidate_path)
        artifact_doc = load_json(artifact_path)
        normalized_proof_path = str(proof_path).replace("\\", "/")
        expected_subject_hash = "sha256:" + hashlib.sha256(proof_bytes).hexdigest()

        if candidate_doc.get("schema") != "ddn.proof_certificate_v1_runtime_candidate.v1":
            raise ValueError(f"{profile}: candidate schema mismatch")
        if artifact_doc.get("schema") != "ddn.proof_certificate_v1_runtime_draft_artifact.v1":
            raise ValueError(f"{profile}: artifact schema mismatch")
        if candidate_doc.get("source_proof_path") != normalized_proof_path:
            raise ValueError(f"{profile}: source_proof_path mismatch")
        if artifact_doc.get("source_proof_path") != normalized_proof_path:
            raise ValueError(f"{profile}: artifact source_proof_path mismatch")
        if candidate_doc.get("source_proof_schema") != proof_doc.get("schema"):
            raise ValueError(f"{profile}: source_proof_schema mismatch")
        if candidate_doc.get("source_proof_kind") != proof_doc.get("kind"):
            raise ValueError(f"{profile}: source_proof_kind mismatch")
        if candidate_doc.get("profile") != profile:
            raise ValueError(f"{profile}: candidate profile mismatch")
        if artifact_doc.get("profile") != profile:
            raise ValueError(f"{profile}: artifact profile mismatch")
        if bool(candidate_doc.get("verified")) != verified:
            raise ValueError(f"{profile}: verified mismatch")
        if int(candidate_doc.get("contract_diag_count", -1)) != contract_diag_count:
            raise ValueError(f"{profile}: contract_diag_count mismatch")
        if candidate_doc.get("proof_subject_hash") != expected_subject_hash:
            raise ValueError(f"{profile}: proof_subject_hash mismatch")
        for key in (
            "canonical_body_hash",
            "proof_runtime_hash",
            "solver_translation_hash",
            "state_hash",
            "trace_hash",
        ):
            if candidate_doc.get(key) != proof_doc.get(key):
                raise ValueError(f"{profile}: candidate field mismatch {key}")
        if artifact_doc.get("candidate_manifest") != candidate_doc:
            raise ValueError(f"{profile}: artifact candidate_manifest mismatch")
        if int(artifact_doc.get("shared_shell_key_count", -1)) != 6:
            raise ValueError(f"{profile}: shared_shell_key_count mismatch")
        if int(artifact_doc.get("state_delta_key_count", -1)) != 6:
            raise ValueError(f"{profile}: state_delta_key_count mismatch")
        if artifact_doc["shared_shell"].get("source_proof_schema") != proof_doc.get("schema"):
            raise ValueError(f"{profile}: shared_shell schema mismatch")
        if artifact_doc["state_delta"].get("proof_subject_hash") != expected_subject_hash:
            raise ValueError(f"{profile}: state_delta proof_subject_hash mismatch")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")
    try:
        teul_cli_bin = ensure_teul_cli_bin(root)
        ensure_pointers(PACK_README)
        ensure_pointers(DRAFT_CONTRACT_README)
        ensure_pointers(PROMOTION_README)
        validate_case(
            root,
            teul_cli_bin,
            CLEAN_INPUT,
            profile="clean",
            verified=True,
            contract_diag_count=0,
        )
        validate_case(
            root,
            teul_cli_bin,
            ABORT_INPUT,
            profile="abort",
            verified=False,
            contract_diag_count=1,
        )
    except ValueError as exc:
        return fail(str(exc))
    print("[proof-certificate-v1-runtime-emit-selftest] ok profiles=2 sidecars=4")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
