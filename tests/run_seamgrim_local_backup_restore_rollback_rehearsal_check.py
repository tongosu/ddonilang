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
sys.path.insert(0, str(TOOLS))

from project_store import SeamgrimProjectStore, ProjectValidationError, stable_json, sha256_hex  # noqa: E402


def fail(code: str, message: str) -> int:
    safe = str(message).encode("ascii", errors="backslashreplace").decode("ascii")
    print(f"{code}: {safe}")
    return 1


def run(cmd: list[str], *, timeout: int = 600) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def artifact(name: str, ddn: str, source_id: str) -> dict:
    snapshot = {
        "schema": "seamgrim.project_snapshot.v0",
        "project_name": name,
        "source_lesson_id": source_id,
        "ddn_text": ddn,
        "lesson_plan_text": f"[lesson]\nlesson_id = \"{source_id}\"\n",
        "controls_snapshot": {"selected_view": "space2d", "run_mode": "sealed", "controls": []},
        "lineage": {"origin_kind": "g10_local_rehearsal", "source_lesson_id": source_id},
    }
    canonical = stable_json(snapshot)
    return {
        "schema": "seamgrim.project_snapshot_artifact.v0",
        "project_snapshot": snapshot,
        "canonical_json": canonical,
        "content_sha256": sha256_hex(canonical),
    }


def checkpoint(label: str, previous: str = "", sequence: int = 0) -> dict:
    return {
        "sequence": sequence,
        "committed_madi": sequence + 1,
        "previous_checkpoint_sha256": previous,
        "input_snapshots": [{"label": label, "value": sequence}],
        "state_hashes": [f"candidate:g10:{label}:{sequence}"],
        "frame_summary": {"local_run_id": label, "frame_count": sequence + 1, "latest_frame_id": sequence, "renderer_pixels_stored": False},
    }


def create_project_with_run(store: SeamgrimProjectStore, *, project_id: str, revision_id: str, run_id: str, name: str, ddn: str, source_id: str) -> dict:
    first = store.create_project(artifact(name, ddn, source_id), title=name, project_id=project_id, revision_id=revision_id)
    caps = store.issue_capabilities(first["project_id"])
    second = store.add_revision(
        first["project_id"],
        artifact(f"{name} 수정", f"{ddn}\n2 보여주기.\n", source_id),
        expected_parent_revision_id=first["head_revision_id"],
    )
    run_record = store.create_run(
        first["project_id"],
        source_revision_id=second["head_revision_id"],
        recipe_hash=f"candidate:g10:{source_id}:recipe",
        seed=f"g10-{source_id}",
        server_run_id=run_id,
    )
    first_cp = store.add_run_checkpoint(first["project_id"], run_record["server_run_id"], checkpoint(source_id, sequence=0))
    first_sha = first_cp["checkpoints"][0]["checkpoint_sha256"]
    second_cp = store.add_run_checkpoint(first["project_id"], run_record["server_run_id"], checkpoint(source_id, previous=first_sha, sequence=1))
    final = store.finalize_run(first["project_id"], run_record["server_run_id"], final_status="completed")
    return {
        "project_id": first["project_id"],
        "revision_id": second["head_revision_id"],
        "content_sha256": second["content_sha256"],
        "run_id": final["server_run_id"],
        "checkpoint_sha256": second_cp["checkpoints"][-1]["checkpoint_sha256"],
        "edit_capability": caps["edit"],
        "view_capability": caps["view"],
    }


def compare_restored(restored: SeamgrimProjectStore, expected: dict) -> None:
    record = restored.get_project(expected["project_id"])
    if record["head_revision_id"] != expected["revision_id"]:
        raise AssertionError(f"head revision mismatch: {record}")
    if record["content_sha256"] != expected["content_sha256"]:
        raise AssertionError(f"content hash mismatch: {record}")
    run_record = restored.get_run(expected["project_id"], expected["run_id"])
    if run_record["final_status"] != "completed":
        raise AssertionError(f"run final status mismatch: {run_record}")
    if run_record["checkpoints"][-1]["checkpoint_sha256"] != expected["checkpoint_sha256"]:
        raise AssertionError(f"checkpoint chain mismatch: {run_record}")
    if restored.authorize_capability(expected["project_id"], expected["view_capability"], required_role="view") != "view":
        raise AssertionError("view capability did not survive restore")
    if restored.authorize_capability(expected["project_id"], expected["edit_capability"], required_role="edit") != "edit":
        raise AssertionError("edit capability did not survive restore")


def expect_project_validation(fn, label: str) -> None:
    try:
        fn()
    except ProjectValidationError:
        return
    raise AssertionError(f"{label}: expected ProjectValidationError")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="seamgrim-g10-", ignore_cleanup_errors=True) as raw:
        root = Path(raw)
        data_dir = root / "data"
        store = SeamgrimProjectStore(data_dir, sqlite_synchronous="FULL")
        projectile = create_project_with_run(
            store,
            project_id="sgp_g10projectile",
            revision_id="sgr_g10projectile",
            run_id="sgrun_g10projectile",
            name="G10 포물선",
            ddn="설정 { 제목: \"G10 포물선\". 마디수: 4. }.\n1 보여주기.\n",
            source_id="rep_phys_projectile_xy_v1",
        )
        pendulum = create_project_with_run(
            store,
            project_id="sgp_g10pendulum",
            revision_id="sgr_g10pendulum",
            run_id="sgrun_g10pendulum",
            name="G10 진자",
            ddn="설정 { 제목: \"G10 진자\". 마디수: 4. }.\n1 보여주기.\n",
            source_id="rep_phys_pendulum_wave1_v1",
        )
        audit = store.audit_integrity()
        if audit.get("ok") is not True or audit.get("counts", {}).get("projects") != 2 or audit.get("counts", {}).get("runs") != 2:
            return fail("E_SEAMGRIM_G10_SOURCE_AUDIT", repr(audit))

        backup_dir = root / "backup"
        backup = store.backup_to(backup_dir)
        if backup.get("audit", {}).get("ok") is not True:
            return fail("E_SEAMGRIM_G10_BACKUP", repr(backup))

        restore_dir = root / "restore-empty"
        restored = SeamgrimProjectStore.restore_from_backup(backup_dir, restore_dir, sqlite_synchronous="FULL")
        restore_audit = restored.audit_integrity()
        if restore_audit.get("ok") is not True:
            return fail("E_SEAMGRIM_G10_RESTORE_AUDIT", repr(restore_audit))
        try:
            compare_restored(restored, projectile)
            compare_restored(restored, pendulum)
        except AssertionError as exc:
            return fail("E_SEAMGRIM_G10_RESTORE_COMPARE", str(exc))

        corrupt = root / "backup-corrupt"
        shutil.copytree(backup_dir, corrupt)
        object_files = list((corrupt / "objects").rglob("*.json"))
        if not object_files:
            return fail("E_SEAMGRIM_G10_CORRUPT_SETUP", "missing backup object")
        object_files[0].write_text("{\"corrupt\":true}", encoding="utf-8")
        expect_project_validation(lambda: SeamgrimProjectStore.restore_from_backup(corrupt, root / "restore-corrupt"), "corrupt restore")

        incomplete = root / "backup-incomplete"
        shutil.copytree(backup_dir, incomplete)
        shutil.rmtree(incomplete / "objects")
        expect_project_validation(lambda: SeamgrimProjectStore.restore_from_backup(incomplete, root / "restore-incomplete"), "incomplete restore")

        pressure = root / "disk-pressure-scratch"
        pressure.mkdir()
        for index in range(8):
            (pressure / f"pressure-{index}.bin").write_bytes(b"0" * 4096)
        if sum(path.stat().st_size for path in pressure.iterdir()) <= 0:
            return fail("E_SEAMGRIM_G10_PRESSURE", "scratch pressure files missing")

    for cmd in [
        [sys.executable, "tests/run_seamgrim_private_pilot_backup_restore_check.py"],
        [sys.executable, "tests/run_seamgrim_private_pilot_deploy_rollback_check.py"],
        [sys.executable, "tests/run_pack_golden.py", "seamgrim_actual_container_rehearsal_v1"],
    ]:
        proc = run(cmd, timeout=900)
        if proc.returncode:
            return fail("E_SEAMGRIM_G10_CMD", f"{cmd!r}: {proc.stdout[-2500:]}")

    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_G10_SSOT", ssot.stdout.strip())

    print("[seamgrim-local-backup-restore-rollback-rehearsal] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
