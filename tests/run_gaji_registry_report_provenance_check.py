from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def run(cmd: list[str]) -> None:
    completed = subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def sha256_bytes(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def sha256_json_obj(value: dict) -> str:
    text = json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
    return "sha256:" + hashlib.sha256(text.encode("utf-8")).hexdigest()


def normalize_path_text(path: str | Path) -> str:
    return str(path).replace("\\", "/")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="gaji_registry_provenance_") as td:
        root = Path(td)
        index = root / "registry.index.json"
        lock = root / "ddn.lock"
        verify_out = root / "registry.verify.report.json"
        audit_log = root / "registry.audit.jsonl"
        audit_out = root / "registry.audit.verify.report.json"

        write_json(
            index,
            {
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-provenance",
                "index_root_hash": "sha256:index-provenance",
                "entries": [
                    {
                        "schema": "ddn.registry.index_entry.v1",
                        "scope": "표준",
                        "name": "역학",
                        "version": "20.6.30",
                        "yanked": False,
                    }
                ],
            },
        )
        write_json(
            lock,
            {
                "schema_version": "v1",
                "registry_snapshot": {
                    "snapshot_id": "snap-provenance",
                    "index_root_hash": "sha256:index-provenance",
                },
                "packages": [
                    {
                        "id": "표준/역학",
                        "version": "20.6.30",
                        "path": "x",
                        "hash": "blake3:x",
                        "yanked": False,
                    }
                ],
            },
        )

        run(
            [
                "cargo",
                "run",
                "--manifest-path",
                "tools/teul-cli/Cargo.toml",
                "--",
                "gaji",
                "registry",
                "verify",
                "--index",
                str(index),
                "--lock",
                str(lock),
                "--out",
                str(verify_out),
            ]
        )
        verify_report = json.loads(verify_out.read_text(encoding="utf-8"))
        verify_provenance = verify_report["source_provenance"]
        assert verify_report["schema"] == "ddn.registry.verify_report.v1"
        assert verify_report["source_hash"] == sha256_json_obj(verify_provenance)
        assert verify_provenance["schema"] == "ddn.registry.verify_report_source_provenance.v1"
        assert verify_provenance["source_kind"] == "registry_verify_inputs.v1"
        assert normalize_path_text(verify_provenance["index_file"]) == normalize_path_text(index)
        assert verify_provenance["index_hash"] == sha256_bytes(index)
        assert normalize_path_text(verify_provenance["lock_file"]) == normalize_path_text(lock)
        assert verify_provenance["lock_hash"] == sha256_bytes(lock)

        run(
            [
                "cargo",
                "run",
                "--manifest-path",
                "tools/teul-cli/Cargo.toml",
                "--",
                "gaji",
                "registry",
                "publish",
                "--index",
                str(index),
                "--audit-log",
                str(audit_log),
                "--scope",
                "표준",
                "--name",
                "역학",
                "--version",
                "20.6.31",
                "--archive-sha256",
                "sha256:archive-provenance",
                "--token",
                "publisher-token",
                "--role",
                "publisher",
            ]
        )
        run(
            [
                "cargo",
                "run",
                "--manifest-path",
                "tools/teul-cli/Cargo.toml",
                "--",
                "gaji",
                "registry",
                "audit-verify",
                "--audit-log",
                str(audit_log),
                "--out",
                str(audit_out),
            ]
        )
        audit_report = json.loads(audit_out.read_text(encoding="utf-8"))
        audit_provenance = audit_report["source_provenance"]
        assert audit_report["schema"] == "ddn.registry.audit_verify_report.v1"
        assert audit_report["source_hash"] == sha256_json_obj(audit_provenance)
        assert (
            audit_provenance["schema"]
            == "ddn.registry.audit_verify_report_source_provenance.v1"
        )
        assert audit_provenance["source_kind"] == "registry_audit_log.v1"
        assert normalize_path_text(audit_provenance["audit_log_file"]) == normalize_path_text(
            audit_log
        )
        assert audit_provenance["audit_log_hash"] == sha256_bytes(audit_log)

    print("gaji_registry_report_provenance: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
