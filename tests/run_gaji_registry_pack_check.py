from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

from _teul_cli_freshness import ensure_teul_cli_bin as shared_ensure_teul_cli_bin


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "gaji_registry_report_provenance_v1"


def teul_cli_candidates(root: Path) -> list[Path]:
    suffix = ".exe" if os.name == "nt" else ""
    return [
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        root / "target" / "debug" / f"teul-cli{suffix}",
    ]


def resolve_teul_cli_bin(root: Path) -> Path:
    return shared_ensure_teul_cli_bin(
        root,
        candidates=teul_cli_candidates(root),
        include_which=False,
        build_env={"RUST_MIN_STACK": str(64 * 1024 * 1024)},
    )


def run(cmd: list[str], *, cwd: Path) -> None:
    env = dict(os.environ)
    env.setdefault("RUST_MIN_STACK", str(64 * 1024 * 1024))
    completed = subprocess.run(
        cmd,
        cwd=cwd,
        env=env,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_bytes(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def sha256_json_obj(value: dict) -> str:
    text = json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
    return "sha256:" + hashlib.sha256(text.encode("utf-8")).hexdigest()


def main() -> int:
    exe = resolve_teul_cli_bin(ROOT)
    with tempfile.TemporaryDirectory(prefix="gaji_registry_pack_") as td:
        work = Path(td)
        shutil.copytree(PACK / "fixtures", work / "fixtures")
        (work / "out").mkdir()

        run(
            [
                str(exe),
                "gaji",
                "registry",
                "verify",
                "--index",
                "fixtures/registry.index.json",
                "--lock",
                "fixtures/ddn.lock",
                "--out",
                "out/registry.verify.report.json",
            ],
            cwd=work,
        )
        verify_report = read_json(work / "out" / "registry.verify.report.json")
        expected_verify = read_json(PACK / "expected" / "registry.verify.report.json")
        assert verify_report == expected_verify
        verify_provenance = verify_report["source_provenance"]
        assert verify_report["source_hash"] == sha256_json_obj(verify_provenance)
        assert verify_provenance["index_hash"] == sha256_bytes(work / "fixtures" / "registry.index.json")
        assert verify_provenance["lock_hash"] == sha256_bytes(work / "fixtures" / "ddn.lock")

        run(
            [
                str(exe),
                "gaji",
                "registry",
                "publish",
                "--index",
                "fixtures/registry.index.json",
                "--audit-log",
                "out/registry.audit.jsonl",
                "--scope",
                "표준",
                "--name",
                "역학",
                "--version",
                "20.6.31",
                "--archive-sha256",
                "sha256:archive-pack",
                "--token",
                "publisher-token",
                "--role",
                "publisher",
                "--at",
                "2026-03-23T00:00:00Z",
            ],
            cwd=work,
        )
        run(
            [
                str(exe),
                "gaji",
                "registry",
                "audit-verify",
                "--audit-log",
                "out/registry.audit.jsonl",
                "--out",
                "out/registry.audit.verify.report.json",
            ],
            cwd=work,
        )

        audit_report = read_json(work / "out" / "registry.audit.verify.report.json")
        expected_audit = read_json(PACK / "expected" / "registry.audit.verify.report.json")
        assert audit_report == expected_audit
        audit_provenance = audit_report["source_provenance"]
        assert audit_report["source_hash"] == sha256_json_obj(audit_provenance)
        assert audit_provenance["audit_log_hash"] == sha256_bytes(work / "out" / "registry.audit.jsonl")

    print("gaji_registry_pack: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
