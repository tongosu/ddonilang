#!/usr/bin/env python
from __future__ import annotations

import json
import os
import subprocess
import tempfile
from pathlib import Path

from _teul_cli_freshness import ensure_teul_cli_bin as shared_ensure_teul_cli_bin


README_PATH = Path("tests/proof_certificate_v1_signed_emit/README.md")
PACK_README = Path("pack/age4_proof_detjson_smoke_v1/README.md")
RUNTIME_EMIT_README = Path("tests/proof_certificate_v1_runtime_emit/README.md")
PROMOTION_README = Path("tests/proof_certificate_v1_promotion/README.md")
INPUT_PATH = Path("pack/age4_proof_detjson_smoke_v1/input.ddn")

README_SNIPPETS = (
    "## Stable Contract",
    "`pack/age4_proof_detjson_smoke_v1/input.ddn`",
    "`tools/teul-cli/src/cli/cert.rs`",
    "`tools/teul-cli/src/cli/run.rs`",
    "`tests/proof_certificate_v1_runtime_emit/README.md`",
    "`tests/proof_certificate_v1_promotion/README.md`",
    "`python tests/run_proof_certificate_v1_signed_emit_selftest.py`",
    "`proof_certificate_v1_signed_emit_selftest`",
    "`ddn.cert_manifest.v1`",
    "`ddn.proof_certificate_v1.v1`",
)
POINTERS = (
    "`tests/proof_certificate_v1_signed_emit/README.md`",
    "`python tests/run_proof_certificate_v1_signed_emit_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-v1-signed-emit-selftest] fail: {message}")
    return 1


def teul_cli_candidates(root: Path) -> list[Path]:
    suffix = ".exe" if os.name == "nt" else ""
    return [
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        root / "target" / "debug" / f"teul-cli{suffix}",
    ]


def ensure_teul_cli_bin(root: Path) -> Path:
    try:
        return shared_ensure_teul_cli_bin(
            root,
            candidates=teul_cli_candidates(root),
            include_which=False,
            build_env={"RUST_MIN_STACK": str(64 * 1024 * 1024)},
        )
    except FileNotFoundError as exc:
        raise ValueError("missing teul-cli binary after cargo build") from exc
    except SystemExit as exc:
        raise ValueError(f"cargo build failed with exit code: {exc.code}") from exc


def run_teul_cli(
    root: Path, teul_cli_bin: Path, args: list[str]
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(teul_cli_bin), *args],
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
        env={**os.environ, "RUST_MIN_STACK": str(64 * 1024 * 1024)},
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


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")
    try:
        teul_cli_bin = ensure_teul_cli_bin(root)
        ensure_pointers(PACK_README)
        ensure_pointers(RUNTIME_EMIT_README)
        ensure_pointers(PROMOTION_README)
        with tempfile.TemporaryDirectory(prefix="proof_certificate_v1_signed_emit_") as td:
            td_path = Path(td)
            key_dir = td_path / "cert"
            proof_path = td_path / "signed.proof.detjson"
            cert_manifest_path = sidecar_path(proof_path, "cert_manifest")
            bundle_path = sidecar_path(proof_path, "proof_certificate_v1")

            keygen = run_teul_cli(
                root,
                teul_cli_bin,
                ["cert", "keygen", "--out", str(key_dir).replace("\\", "/"), "--seed", "signed-emit-selftest"],
            )
            if keygen.returncode != 0:
                raise ValueError(f"cert keygen failed stdout={keygen.stdout!r} stderr={keygen.stderr!r}")
            private_key = key_dir / "cert_private.key"
            if not private_key.exists():
                raise ValueError("missing private key")

            run_result = run_teul_cli(
                root,
                teul_cli_bin,
                [
                    "run",
                    str(INPUT_PATH).replace("\\", "/"),
                    "--proof-out",
                    str(proof_path).replace("\\", "/"),
                    "--proof-cert-key",
                    str(private_key).replace("\\", "/"),
                ],
            )
            if run_result.returncode != 0:
                raise ValueError(f"run failed stdout={run_result.stdout!r} stderr={run_result.stderr!r}")
            if not cert_manifest_path.exists():
                raise ValueError("missing cert manifest sidecar")
            if not bundle_path.exists():
                raise ValueError("missing proof_certificate_v1 bundle")

            verify = run_teul_cli(
                root,
                teul_cli_bin,
                ["cert", "verify", "--in", str(cert_manifest_path).replace("\\", "/")],
            )
            if verify.returncode != 0:
                raise ValueError(f"cert verify failed stdout={verify.stdout!r} stderr={verify.stderr!r}")

            proof_doc = load_json(proof_path)
            cert_manifest_doc = load_json(cert_manifest_path)
            bundle_doc = load_json(bundle_path)
            if bundle_doc.get("schema") != "ddn.proof_certificate_v1.v1":
                raise ValueError("bundle schema mismatch")
            if bundle_doc.get("source_proof_schema") != proof_doc.get("schema"):
                raise ValueError("bundle source_proof_schema mismatch")
            if bundle_doc.get("source_proof_kind") != proof_doc.get("kind"):
                raise ValueError("bundle source_proof_kind mismatch")
            if bundle_doc.get("proof_subject_hash") != cert_manifest_doc.get("subject_hash"):
                raise ValueError("bundle subject hash mismatch")
            if bundle_doc.get("cert_manifest") != cert_manifest_doc:
                raise ValueError("bundle cert_manifest mismatch")
            if bundle_doc.get("cert_pubkey") != cert_manifest_doc.get("pubkey"):
                raise ValueError("bundle cert_pubkey mismatch")
            if bundle_doc.get("cert_signature") != cert_manifest_doc.get("signature"):
                raise ValueError("bundle cert_signature mismatch")
            if bundle_doc.get("verified") is not True:
                raise ValueError("bundle verified mismatch")
            if int(bundle_doc.get("contract_diag_count", -1)) != 0:
                raise ValueError("bundle contract_diag_count mismatch")
    except ValueError as exc:
        return fail(str(exc))
    print("[proof-certificate-v1-signed-emit-selftest] ok signed_sidecars=2")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
