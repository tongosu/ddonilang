from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "proof_certificate_verify_report_provenance_v1"


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


def validate_profile(work: Path, profile: str, expected_name: str) -> None:
    report_path = work / "verify" / f"{profile}.verify.report.detjson"
    report = read_json(report_path)
    expected = read_json(PACK / "expected" / expected_name)
    assert report == expected
    provenance = report["source_provenance"]
    assert provenance["input_bundle_hash"] == sha256_bytes(
        work / "proof" / f"{profile}.proof.proof_certificate_v1.detjson"
    )
    assert provenance["source_proof_hash"] == sha256_bytes(work / "proof" / f"{profile}.proof.detjson")


def main() -> int:
    exe = resolve_teul_cli_bin(ROOT)
    with tempfile.TemporaryDirectory(prefix="proof_certificate_verify_report_pack_") as td:
        work = Path(td)
        shutil.copytree(PACK / "fixtures", work / "fixtures")
        (work / "proof").mkdir()
        (work / "keys").mkdir()
        (work / "verify").mkdir()

        cases = [
            ("clean", "fixtures/input.ddn", "verify-report-clean", "clean.verify.report.detjson"),
            ("abort", "fixtures/input_abort.ddn", "verify-report-abort", "abort.verify.report.detjson"),
        ]
        for profile, input_rel, seed, expected_name in cases:
            run([str(exe), "cert", "keygen", "--out", f"keys/{profile}", "--seed", seed], cwd=work)
            run(
                [
                    str(exe),
                    "run",
                    input_rel,
                    "--proof-out",
                    f"proof/{profile}.proof.detjson",
                    "--proof-cert-key",
                    f"keys/{profile}/cert_private.key",
                ],
                cwd=work,
            )
            verify_ok = run(
                [
                    str(exe),
                    "cert",
                    "verify-proof-certificate",
                    "--in",
                    f"proof/{profile}.proof.proof_certificate_v1.detjson",
                    "--out",
                    f"verify/{profile}.verify.report.detjson",
                ],
                cwd=work,
            )
            assert "proof_certificate_verify_report=" in verify_ok.stdout
            validate_profile(work, profile, expected_name)

            bundle_path = work / "proof" / f"{profile}.proof.proof_certificate_v1.detjson"
            tampered = read_json(bundle_path)
            tampered["proof_subject_hash"] = "sha256:" + ("0" * 64)
            tampered_path = work / "proof" / f"{profile}.tampered.proof_certificate_v1.detjson"
            tampered_path.write_text(
                json.dumps(tampered, ensure_ascii=False, indent=2) + "\n",
                encoding="utf-8",
            )
            failed_report = work / "verify" / f"{profile}.tampered.verify.report.detjson"
            bad = subprocess.run(
                [
                    str(exe),
                    "cert",
                    "verify-proof-certificate",
                    "--in",
                    str(tampered_path.relative_to(work)).replace("\\", "/"),
                    "--out",
                    str(failed_report.relative_to(work)).replace("\\", "/"),
                ],
                cwd=work,
                env={**os.environ, "RUST_MIN_STACK": str(64 * 1024 * 1024)},
                text=True,
                encoding="utf-8",
                errors="replace",
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            assert bad.returncode != 0
            assert not failed_report.exists()

    print("proof_certificate_verify_report_pack: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
