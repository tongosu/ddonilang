#!/usr/bin/env python
from __future__ import annotations

import json
import os
import subprocess
import tempfile
from pathlib import Path

from _teul_cli_freshness import ensure_teul_cli_bin as shared_ensure_teul_cli_bin


README_PATH = Path("tests/proof_certificate_v1_verify_bundle/README.md")
PACK_README = Path("pack/age4_proof_detjson_smoke_v1/README.md")
SIGNED_PROFILES_README = Path("tests/proof_certificate_v1_signed_emit_profiles/README.md")
SIGNED_CONTRACT_README = Path("tests/proof_certificate_v1_signed_contract/README.md")
PROMOTION_README = Path("tests/proof_certificate_v1_promotion/README.md")
CLEAN_INPUT = Path("pack/age4_proof_detjson_smoke_v1/input.ddn")
ABORT_INPUT = Path("pack/age4_proof_detjson_smoke_v1/input_abort.ddn")

README_SNIPPETS = (
    "## Stable Contract",
    "`tools/teul-cli/src/cli/cert.rs`",
    "`tools/teul-cli/src/main.rs`",
    "`pack/age4_proof_detjson_smoke_v1/input.ddn`",
    "`pack/age4_proof_detjson_smoke_v1/input_abort.ddn`",
    "`tests/proof_certificate_v1_signed_emit_profiles/README.md`",
    "`tests/proof_certificate_v1_signed_contract/README.md`",
    "`python tests/run_proof_certificate_v1_verify_bundle_selftest.py`",
    "`proof_certificate_v1_verify_bundle_selftest`",
    "`teul-cli cert verify-proof-certificate --in <proof_certificate_v1.detjson>`",
)
POINTERS = (
    "`tests/proof_certificate_v1_verify_bundle/README.md`",
    "`python tests/run_proof_certificate_v1_verify_bundle_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-v1-verify-bundle-selftest] fail: {message}")
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


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def sidecar_path(path: Path, label: str) -> Path:
    file_name = path.name
    stem = file_name[:-8] if file_name.endswith(".detjson") else file_name
    return path.with_name(f"{stem}.{label}.detjson")


def write_json(path: Path, payload: dict) -> None:
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def validate_profile(
    root: Path,
    teul_cli_bin: Path,
    source: Path,
    *,
    profile: str,
    seed: str,
    out_dir: Path,
) -> None:
    key_dir = out_dir / f"{profile}_cert"
    proof_path = out_dir / f"{profile}.proof.detjson"
    bundle_path = sidecar_path(proof_path, "proof_certificate_v1")
    keygen = run_teul_cli(
        root,
        teul_cli_bin,
        ["cert", "keygen", "--out", str(key_dir).replace("\\", "/"), "--seed", seed],
    )
    if keygen.returncode != 0:
        raise ValueError(f"{profile}: keygen failed stdout={keygen.stdout!r} stderr={keygen.stderr!r}")
    private_key = key_dir / "cert_private.key"
    run_result = run_teul_cli(
        root,
        teul_cli_bin,
        [
            "run",
            str(source).replace("\\", "/"),
            "--proof-out",
            str(proof_path).replace("\\", "/"),
            "--proof-cert-key",
            str(private_key).replace("\\", "/"),
        ],
    )
    if run_result.returncode != 0:
        raise ValueError(f"{profile}: run failed stdout={run_result.stdout!r} stderr={run_result.stderr!r}")
    verify_ok = run_teul_cli(
        root,
        teul_cli_bin,
        ["cert", "verify-proof-certificate", "--in", str(bundle_path).replace("\\", "/")],
    )
    if verify_ok.returncode != 0:
        raise ValueError(
            f"{profile}: verify-proof-certificate failed stdout={verify_ok.stdout!r} stderr={verify_ok.stderr!r}"
        )
    if "proof_certificate_verify=ok" not in verify_ok.stdout:
        raise ValueError(f"{profile}: missing verify ok stdout")
    if f"proof_certificate_profile={profile}" not in verify_ok.stdout:
        raise ValueError(f"{profile}: missing verify profile stdout")

    bundle_doc = json.loads(bundle_path.read_text(encoding="utf-8"))
    tampered_path = out_dir / f"{profile}.tampered.proof_certificate_v1.detjson"
    bundle_doc["proof_subject_hash"] = "sha256:" + ("0" * 64)
    write_json(tampered_path, bundle_doc)
    verify_bad = run_teul_cli(
        root,
        teul_cli_bin,
        ["cert", "verify-proof-certificate", "--in", str(tampered_path).replace("\\", "/")],
    )
    if verify_bad.returncode == 0:
        raise ValueError(f"{profile}: tampered verify unexpectedly passed")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")
    try:
        teul_cli_bin = ensure_teul_cli_bin(root)
        ensure_pointers(PACK_README)
        ensure_pointers(SIGNED_PROFILES_README)
        ensure_pointers(SIGNED_CONTRACT_README)
        ensure_pointers(PROMOTION_README)
        with tempfile.TemporaryDirectory(prefix="proof_certificate_v1_verify_bundle_") as td:
            td_path = Path(td)
            validate_profile(
                root,
                teul_cli_bin,
                CLEAN_INPUT,
                profile="clean",
                seed="verify-bundle-clean",
                out_dir=td_path,
            )
            validate_profile(
                root,
                teul_cli_bin,
                ABORT_INPUT,
                profile="abort",
                seed="verify-bundle-abort",
                out_dir=td_path,
            )
    except ValueError as exc:
        return fail(str(exc))

    print("[proof-certificate-v1-verify-bundle-selftest] ok profiles=2 tamper=2")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
