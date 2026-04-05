#!/usr/bin/env python
from __future__ import annotations

import json
import os
import subprocess
import tempfile
import hashlib
from pathlib import Path


README_PATH = Path("tests/proof_certificate_v1_verify_report/README.md")
PACK_README = Path("pack/age4_proof_detjson_smoke_v1/README.md")
VERIFY_BUNDLE_README = Path("tests/proof_certificate_v1_verify_bundle/README.md")
SIGNED_CONTRACT_README = Path("tests/proof_certificate_v1_signed_contract/README.md")
PROMOTION_README = Path("tests/proof_certificate_v1_promotion/README.md")
CLEAN_INPUT = Path("pack/age4_proof_detjson_smoke_v1/input.ddn")
ABORT_INPUT = Path("pack/age4_proof_detjson_smoke_v1/input_abort.ddn")

README_SNIPPETS = (
    "## Stable Contract",
    "`tools/teul-cli/src/cli/cert.rs`",
    "`tools/teul-cli/src/main.rs`",
    "`tests/proof_certificate_v1_verify_bundle/README.md`",
    "`tests/proof_certificate_v1_signed_contract/README.md`",
    "`python tests/run_proof_certificate_v1_verify_report_selftest.py`",
    "`proof_certificate_v1_verify_report_selftest`",
    "`ddn.proof_certificate_v1.verify_report.v1`",
)
POINTERS = (
    "`tests/proof_certificate_v1_verify_report/README.md`",
    "`python tests/run_proof_certificate_v1_verify_report_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-v1-verify-report-selftest] fail: {message}")
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


def sha256_bytes(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def validate_profile(
    root: Path,
    teul_cli_bin: Path,
    source: Path,
    *,
    profile: str,
    verified: bool,
    contract_diag_count: int,
    seed: str,
    out_dir: Path,
) -> None:
    key_dir = out_dir / f"{profile}_cert"
    proof_path = out_dir / f"{profile}.proof.detjson"
    bundle_path = sidecar_path(proof_path, "proof_certificate_v1")
    report_path = out_dir / f"{profile}.verify.report.detjson"
    bad_report_path = out_dir / f"{profile}.tampered.verify.report.detjson"
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
        [
            "cert",
            "verify-proof-certificate",
            "--in",
            str(bundle_path).replace("\\", "/"),
            "--out",
            str(report_path).replace("\\", "/"),
        ],
    )
    if verify_ok.returncode != 0:
        raise ValueError(f"{profile}: verify report failed stdout={verify_ok.stdout!r} stderr={verify_ok.stderr!r}")
    if not report_path.exists():
        raise ValueError(f"{profile}: missing verify report")
    report = json.loads(report_path.read_text(encoding="utf-8"))
    if report.get("schema") != "ddn.proof_certificate_v1.verify_report.v1":
        raise ValueError(f"{profile}: report schema mismatch")
    if report.get("ok") is not True:
        raise ValueError(f"{profile}: report ok mismatch")
    if report.get("source_hash") != sha256_bytes(bundle_path):
        raise ValueError(f"{profile}: report source_hash mismatch")
    if report.get("profile") != profile:
        raise ValueError(f"{profile}: report profile mismatch")
    if report.get("verified") is not verified:
        raise ValueError(f"{profile}: report verified mismatch")
    if int(report.get("contract_diag_count", -1)) != contract_diag_count:
        raise ValueError(f"{profile}: report contract_diag_count mismatch")
    if report.get("cert_manifest_schema") != "ddn.cert_manifest.v1":
        raise ValueError(f"{profile}: report cert_manifest_schema mismatch")
    if report.get("cert_algo") != "sha256-proto":
        raise ValueError(f"{profile}: report cert_algo mismatch")
    provenance = report.get("source_provenance")
    if not isinstance(provenance, dict):
        raise ValueError(f"{profile}: report source_provenance missing")
    if provenance.get("schema") != "ddn.proof_certificate_v1.verify_report_source_provenance.v1":
        raise ValueError(f"{profile}: report source_provenance schema mismatch")
    if provenance.get("source_kind") != "proof_certificate_bundle.v1":
        raise ValueError(f"{profile}: report source_provenance source_kind mismatch")
    if provenance.get("input_bundle_file") != str(bundle_path).replace("\\", "/"):
        raise ValueError(f"{profile}: report input_bundle_file mismatch")
    if provenance.get("input_bundle_hash") != sha256_bytes(bundle_path):
        raise ValueError(f"{profile}: report input_bundle_hash mismatch")
    if provenance.get("source_proof_file") != str(proof_path).replace("\\", "/"):
        raise ValueError(f"{profile}: report source_proof_file mismatch")
    if provenance.get("source_proof_hash") != sha256_bytes(proof_path):
        raise ValueError(f"{profile}: report source_proof_hash mismatch")
    bundle = json.loads(bundle_path.read_text(encoding="utf-8"))
    proof = json.loads(proof_path.read_text(encoding="utf-8"))
    if report.get("proof_subject_hash") != bundle.get("proof_subject_hash"):
        raise ValueError(f"{profile}: report proof_subject_hash mismatch")
    if report.get("cert_subject_hash") != bundle.get("proof_subject_hash"):
        raise ValueError(f"{profile}: report cert_subject_hash mismatch")
    if report.get("cert_signature") != bundle.get("cert_signature"):
        raise ValueError(f"{profile}: report cert_signature mismatch")
    for key in (
        "canonical_body_hash",
        "proof_runtime_hash",
        "solver_translation_hash",
        "state_hash",
        "trace_hash",
    ):
        if report.get(key) != proof.get(key):
            raise ValueError(f"{profile}: report {key} mismatch")
    if "proof_certificate_verify_report=" not in verify_ok.stdout:
        raise ValueError(f"{profile}: missing verify report stdout")

    tampered = json.loads(bundle_path.read_text(encoding="utf-8"))
    tampered["proof_subject_hash"] = "sha256:" + ("0" * 64)
    tampered_path = out_dir / f"{profile}.tampered.proof_certificate_v1.detjson"
    tampered_path.write_text(json.dumps(tampered, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    verify_bad = run_teul_cli(
        root,
        teul_cli_bin,
        [
            "cert",
            "verify-proof-certificate",
            "--in",
            str(tampered_path).replace("\\", "/"),
            "--out",
            str(bad_report_path).replace("\\", "/"),
        ],
    )
    if verify_bad.returncode == 0:
        raise ValueError(f"{profile}: tampered verify unexpectedly passed")
    if bad_report_path.exists():
        raise ValueError(f"{profile}: tampered verify report should not exist")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")
    try:
        teul_cli_bin = ensure_teul_cli_bin(root)
        ensure_pointers(PACK_README)
        ensure_pointers(VERIFY_BUNDLE_README)
        ensure_pointers(SIGNED_CONTRACT_README)
        ensure_pointers(PROMOTION_README)
        with tempfile.TemporaryDirectory(prefix="proof_certificate_v1_verify_report_") as td:
            td_path = Path(td)
            validate_profile(
                root,
                teul_cli_bin,
                CLEAN_INPUT,
                profile="clean",
                verified=True,
                contract_diag_count=0,
                seed="verify-report-clean",
                out_dir=td_path,
            )
            validate_profile(
                root,
                teul_cli_bin,
                ABORT_INPUT,
                profile="abort",
                verified=False,
                contract_diag_count=1,
                seed="verify-report-abort",
                out_dir=td_path,
            )
    except ValueError as exc:
        return fail(str(exc))
    print("[proof-certificate-v1-verify-report-selftest] ok profiles=2 reports=2")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
