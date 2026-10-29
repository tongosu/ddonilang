#!/usr/bin/env python
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / "solutions" / "seamgrim_ui_mvp" / "tools"
BACKUP_TOOL = ROOT / "solutions" / "seamgrim_ui_mvp" / "deploy" / "private_pilot_backup.py"
PACK = ROOT / "pack" / "seamgrim_private_pilot_release_contract_v1"
sys.path.insert(0, str(TOOLS))

from project_store import SeamgrimProjectStore, stable_json, sha256_hex  # noqa: E402


def fail(message: str) -> None:
    raise SystemExit(f"[seamgrim-private-pilot-backup] FAIL: {message}")


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def run(args: list[str], *, timeout: int = 300, expect_ok: bool = True) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        args,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )
    if expect_ok and proc.returncode != 0:
        fail(f"{' '.join(args)} failed\nSTDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}")
    if not expect_ok and proc.returncode == 0:
        fail(f"{' '.join(args)} unexpectedly passed\nSTDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}")
    return proc


def git(args: list[str]) -> str:
    return run(["git", *args]).stdout.strip()


def artifact(name: str, ddn: str) -> dict:
    snapshot = {
        "schema": "seamgrim.project_snapshot.v0",
        "project_name": name,
        "source_lesson_id": "private_pilot_backup_restore",
        "ddn_text": ddn,
        "lesson_plan_text": "[lesson]\nlesson_id = \"private_pilot_backup_restore\"\n",
        "controls_snapshot": {"selected_view": "space2d", "run_mode": "sealed", "controls": [], "view_authoring": {}},
        "local_package_payload": None,
        "package_hash_compat": "",
        "lineage": {
            "source_lesson_id": "private_pilot_backup_restore",
            "source_project_id": "",
            "source_revision_id": "",
            "source_package_hash": "",
            "origin_content_sha256": "",
            "origin_kind": "checker",
        },
        "metadata": {"product": "seamgrim_ui_mvp", "builder": "private_pilot_backup_checker"},
        "boundaries": {
            "account_required": False,
            "public_listing": False,
            "server_executes_ddn_for_save": False,
            "state_hash_participation": False,
            "candidate_package_hash_security_claim": False,
        },
    }
    canonical = stable_json(snapshot)
    return {
        "schema": "seamgrim.project_snapshot_artifact.v0",
        "project_snapshot": snapshot,
        "canonical_json": canonical,
        "content_sha256": sha256_hex(canonical),
        "canonical_byte_length": len(canonical.encode("utf-8")),
    }


def make_store(data_dir: Path) -> dict[str, str]:
    store = SeamgrimProjectStore(data_dir, sqlite_synchronous="FULL")
    first = store.create_project(artifact("사설 백업", "1 보여주기.\n"), title="사설 백업")
    caps = store.issue_capabilities(first["project_id"])
    second = store.add_revision(
        first["project_id"],
        artifact("사설 백업", "1 보여주기.\n2 보여주기.\n"),
        expected_parent_revision_id=first["head_revision_id"],
    )
    run_record = store.create_run(
        first["project_id"],
        source_revision_id=second["head_revision_id"],
        recipe_hash="sha256:private-pilot-backup-recipe",
        seed="backup-seed",
    )
    checkpointed = store.add_run_checkpoint(
        first["project_id"],
        run_record["server_run_id"],
        {
            "sequence": 0,
            "committed_madi": 1,
            "input_snapshots": [{"name": "start", "value": 1}],
            "state_hashes": ["hash:backup-state-1"],
            "frame_summary": {"view": "space2d", "objects": 2},
        },
    )
    finalized = store.finalize_run(first["project_id"], run_record["server_run_id"], final_status="completed")
    clone = store.clone_project(first["project_id"], title="사설 백업 복제")
    audit = store.audit_integrity()
    require(audit.get("ok") is True, f"source audit failed: {audit}")
    return {
        "project_id": first["project_id"],
        "revision_id": second["head_revision_id"],
        "content_sha256": second["content_sha256"],
        "run_id": finalized["server_run_id"],
        "checkpoint_sha256": checkpointed["checkpoints"][0]["checkpoint_sha256"],
        "clone_project_id": clone["project_id"],
        "edit_capability": caps["edit"],
        "view_capability": caps["view"],
    }


def parse_json_stdout(proc: subprocess.CompletedProcess[str]) -> dict:
    try:
        data = json.loads(proc.stdout.strip().splitlines()[-1])
    except Exception as exc:
        fail(f"stdout is not JSON: {exc}: {proc.stdout!r}")
    require(isinstance(data, dict), "stdout JSON must be object")
    return data


def main() -> None:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    backup_contract = contract.get("backup_restore")
    require(isinstance(backup_contract, dict), "backup_restore contract missing")

    with tempfile.TemporaryDirectory(prefix="seamgrim_private_backup_", ignore_cleanup_errors=True) as raw:
        root = Path(raw)
        data_dir = root / "data"
        ids = make_store(data_dir)
        backup_dir = root / "backup" / "snapshot-1"
        restore_dir = root / "restore"

        inside_backup = run(
            [
                sys.executable,
                str(BACKUP_TOOL),
                "backup",
                "--data-dir",
                str(data_dir),
                "--backup-dir",
                str(data_dir / "bad-backup"),
            ],
            expect_ok=False,
        )
        require("must be outside source data directory" in inside_backup.stderr, "backup inside data dir must be rejected")

        backup = parse_json_stdout(
            run(
                [
                    sys.executable,
                    str(BACKUP_TOOL),
                    "backup",
                    "--data-dir",
                    str(data_dir),
                    "--backup-dir",
                    str(backup_dir),
                    "--source-commit",
                    git(["rev-parse", "HEAD"]),
                    "--release-manifest-sha256",
                    "1" * 64,
                ],
                timeout=180,
            )
        )
        require(backup.get("status") == "PASS", f"backup failed: {backup}")
        manifest = json.loads((backup_dir / "backup_manifest.detjson").read_text(encoding="utf-8"))
        require(manifest.get("integrity_ok") is True, "backup manifest must seal pre-backup audit")
        require(manifest.get("raw_live_db_copy_claim") is False, "raw live db copy claim must be false")
        require(manifest.get("live_restore_automatic") is False, "live restore must not be automatic")
        require(manifest.get("source_data_dir_redacted") != str(data_dir), "manifest must redact source data dir")

        restore = parse_json_stdout(
            run(
                [
                    sys.executable,
                    str(BACKUP_TOOL),
                    "restore-check",
                    "--backup-dir",
                    str(backup_dir),
                    "--restore-dir",
                    str(restore_dir),
                ],
                timeout=180,
            )
        )
        require(restore.get("status") == "PASS", f"restore check failed: {restore}")

        restored = SeamgrimProjectStore(restore_dir, sqlite_synchronous="FULL")
        record = restored.get_project(ids["project_id"])
        require(record["head_revision_id"] == ids["revision_id"], "restored project head mismatch")
        require(record["content_sha256"] == ids["content_sha256"], "restored content hash mismatch")
        run_record = restored.get_run(ids["project_id"], ids["run_id"])
        require(run_record["final_status"] == "completed", "restored run lineage mismatch")
        require(run_record["checkpoints"][0]["checkpoint_sha256"] == ids["checkpoint_sha256"], "checkpoint mismatch")
        clone_record = restored.get_project(ids["clone_project_id"])
        require(clone_record["origin_project_id"] == ids["project_id"], "clone lineage mismatch")
        require(restored.authorize_capability(ids["project_id"], ids["view_capability"], required_role="view") == "view", "view cap mismatch")

        non_empty_restore = root / "non-empty-restore"
        non_empty_restore.mkdir()
        (non_empty_restore / "marker").write_text("x", encoding="utf-8")
        rejected_nonempty = run(
            [
                sys.executable,
                str(BACKUP_TOOL),
                "restore-check",
                "--backup-dir",
                str(backup_dir),
                "--restore-dir",
                str(non_empty_restore),
            ],
            expect_ok=False,
        )
        require("restore target must be empty" in rejected_nonempty.stderr, "non-empty restore target must be rejected")

        corrupt_backup = root / "backup-corrupt"
        shutil.copytree(backup_dir, corrupt_backup)
        object_files = [path for path in (corrupt_backup / "objects").rglob("*.json")]
        require(object_files, "backup object files missing for corruption test")
        object_files[0].write_text("{\"corrupt\":true}", encoding="utf-8")
        corrupt = run(
            [
                sys.executable,
                str(BACKUP_TOOL),
                "restore-check",
                "--backup-dir",
                str(corrupt_backup),
                "--restore-dir",
                str(root / "restore-corrupt"),
            ],
            expect_ok=False,
        )
        require("backup object hash mismatch" in corrupt.stderr, "corrupt backup must be rejected before restore")

        incomplete_backup = root / "backup-incomplete"
        shutil.copytree(backup_dir, incomplete_backup)
        shutil.rmtree(incomplete_backup / "objects")
        incomplete = run(
            [
                sys.executable,
                str(BACKUP_TOOL),
                "restore-check",
                "--backup-dir",
                str(incomplete_backup),
                "--restore-dir",
                str(root / "restore-incomplete"),
            ],
            expect_ok=False,
        )
        require("backup objects missing" in incomplete.stderr, "incomplete backup must be rejected")

        offhost = parse_json_stdout(
            run(
                [
                    sys.executable,
                    str(BACKUP_TOOL),
                    "plan-offhost-copy",
                    "--backup-dir",
                    str(backup_dir),
                    "--destination-label",
                    "offhost-dry-run",
                ]
            )
        )
        require(offhost.get("status") == "DRY_RUN_ONLY", "offhost copy must be dry-run only")
        require(offhost.get("contains_secret") is False, "offhost plan must be secret-free")

    pack = run([sys.executable, "tests/run_pack_golden.py", "seamgrim_private_pilot_release_contract_v1"], timeout=900)
    require("pack golden ok" in pack.stdout, "pack marker failed")
    ssot_status = git(["status", "--short", "--", "docs/ssot"])
    require(not ssot_status.strip(), f"docs/ssot changed:\n{ssot_status}")
    print("[seamgrim-private-pilot-backup] PASS")


if __name__ == "__main__":
    main()
