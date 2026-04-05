from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "proof_certificate_verify_bundle_v1"


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


def run(cmd: list[str], *, cwd: Path, expect_ok: bool = True) -> subprocess.CompletedProcess[str]:
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
    if expect_ok and completed.returncode != 0:
        print(completed.stdout, end="")
        print(completed.stderr, end="")
        raise SystemExit(completed.returncode)
    return completed


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8").replace("\r\n", "\n")


def validate_profile(work: Path, profile: str, expected_name: str) -> None:
    expected = read_text(PACK / "expected" / expected_name)
    verified = run(
        [
            str(resolve_teul_cli_bin(ROOT)),
            "cert",
            "verify-proof-certificate",
            "--in",
            f"{profile}.proof.proof_certificate_v1.detjson",
        ],
        cwd=work,
    )
    stdout = verified.stdout.replace("\r\n", "\n")
    assert stdout == expected

    bundle_path = work / f"{profile}.proof.proof_certificate_v1.detjson"
    tampered = json.loads(bundle_path.read_text(encoding="utf-8"))
    tampered["proof_subject_hash"] = "sha256:" + ("0" * 64)
    tampered_path = work / f"{profile}.tampered.proof_certificate_v1.detjson"
    tampered_path.write_text(
        json.dumps(tampered, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    failed = run(
        [
            str(resolve_teul_cli_bin(ROOT)),
            "cert",
            "verify-proof-certificate",
            "--in",
            tampered_path.name,
        ],
        cwd=work,
        expect_ok=False,
    )
    assert failed.returncode != 0
    assert failed.stderr.strip() == "E_PROOF_CERT_VERIFY_SUBJECT_HASH_MISMATCH"


def main() -> int:
    exe = resolve_teul_cli_bin(ROOT)
    with tempfile.TemporaryDirectory(prefix="proof_certificate_verify_bundle_pack_") as td:
        work = Path(td)
        shutil.copytree(PACK / "fixtures", work / "fixtures")
        cases = [
            ("clean", "fixtures/input.ddn", "verify-bundle-clean", "clean.stdout.txt"),
            ("abort", "fixtures/input_abort.ddn", "verify-bundle-abort", "abort.stdout.txt"),
        ]
        for profile, input_rel, seed, expected_name in cases:
            run([str(exe), "cert", "keygen", "--out", f"keys_{profile}", "--seed", seed], cwd=work)
            run(
                [
                    str(exe),
                    "run",
                    input_rel,
                    "--proof-out",
                    f"{profile}.proof.detjson",
                    "--proof-cert-key",
                    f"keys_{profile}/cert_private.key",
                ],
                cwd=work,
            )
            validate_profile(work, profile, expected_name)

    print("proof_certificate_verify_bundle_pack: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
