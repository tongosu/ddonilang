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
BUILD_RELEASE = ROOT / "solutions" / "seamgrim_ui_mvp" / "deploy" / "build_private_pilot_release.py"
DEPLOY_TOOL = ROOT / "solutions" / "seamgrim_ui_mvp" / "deploy" / "private_pilot_deploy.py"
PACK = ROOT / "pack" / "seamgrim_private_pilot_release_contract_v1"
sys.path.insert(0, str(TOOLS))

from project_store import SeamgrimProjectStore, stable_json, sha256_hex  # noqa: E402


def fail(message: str) -> None:
    raise SystemExit(f"[seamgrim-private-pilot-deploy] FAIL: {message}")


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


def parse(proc: subprocess.CompletedProcess[str]) -> dict:
    payload = json.loads(proc.stdout.strip().splitlines()[-1])
    require(isinstance(payload, dict), "stdout JSON must be object")
    return payload


def artifact(name: str) -> dict:
    snapshot = {
        "schema": "seamgrim.project_snapshot.v0",
        "project_name": name,
        "source_lesson_id": "private_pilot_deploy",
        "ddn_text": "1 보여주기.\n",
        "lesson_plan_text": "[lesson]\nlesson_id = \"private_pilot_deploy\"\n",
        "controls_snapshot": {"selected_view": "space2d", "run_mode": "sealed", "controls": [], "view_authoring": {}},
        "local_package_payload": None,
        "package_hash_compat": "",
        "lineage": {
            "source_lesson_id": "private_pilot_deploy",
            "source_project_id": "",
            "source_revision_id": "",
            "source_package_hash": "",
            "origin_content_sha256": "",
            "origin_kind": "checker",
        },
        "metadata": {"product": "seamgrim_ui_mvp", "builder": "private_pilot_deploy_checker"},
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


def prepare_data(data_dir: Path) -> str:
    store = SeamgrimProjectStore(data_dir, sqlite_synchronous="FULL")
    record = store.create_project(artifact("배포 검증"), title="배포 검증")
    store.issue_capabilities(record["project_id"])
    audit = store.audit_integrity()
    require(audit.get("ok") is True, f"data audit failed: {audit}")
    return record["project_id"]


def clone_release_with_id(source_manifest: Path, target_root: Path, suffix: str) -> Path:
    source_root = source_manifest.parent
    target = target_root / f"{source_root.name}-{suffix}"
    shutil.copytree(source_root, target)
    manifest_path = target / "manifest.detjson"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["release_id"] = f"{manifest['release_id']}-{suffix}"
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, sort_keys=True, indent=2) + "\n", encoding="utf-8")
    return manifest_path


def main() -> None:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    deploy_contract = contract.get("deploy_rollback")
    require(isinstance(deploy_contract, dict), "deploy_rollback contract missing")

    with tempfile.TemporaryDirectory(prefix="seamgrim_private_deploy_", ignore_cleanup_errors=True) as raw:
        root = Path(raw)
        build_out = root / "release-build"
        manifest = Path(parse(run([sys.executable, str(BUILD_RELEASE), "--out", str(build_out), "--clean"], timeout=900))["manifest"])
        manifest_b = clone_release_with_id(manifest, root / "release-variants", "next")
        bad_manifest = clone_release_with_id(manifest, root / "release-variants", "bad")
        wasm = bad_manifest.parent / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm" / "ddonirang_tool.js"
        wasm.write_text(wasm.read_text(encoding="utf-8") + "\n// tampered\n", encoding="utf-8")

        data_dir = root / "data"
        project_id = prepare_data(data_dir)
        deploy_root = root / "deploy-root"
        backup_root = root / "backups"

        plan = parse(
            run(
                [
                    sys.executable,
                    str(DEPLOY_TOOL),
                    "plan",
                    "--release-manifest",
                    str(manifest),
                    "--deploy-root",
                    str(deploy_root),
                    "--data-dir",
                    str(data_dir),
                    "--backup-root",
                    str(backup_root),
                ]
            )
        )
        require(plan.get("will_change_pointer") is True, "initial plan should change pointer")
        require(plan.get("data_restore_included") is False, "deploy plan must not include data restore")

        first = parse(
            run(
                [
                    sys.executable,
                    str(DEPLOY_TOOL),
                    "apply",
                    "--release-manifest",
                    str(manifest),
                    "--deploy-root",
                    str(deploy_root),
                    "--data-dir",
                    str(data_dir),
                    "--backup-root",
                    str(backup_root),
                ],
                timeout=900,
            )
        )
        require(first.get("pointer_changed") is True, f"first apply failed: {first}")
        require(Path(first.get("backup_manifest", "")).is_file(), "pre-deploy backup manifest missing")

        idempotent = parse(
            run(
                [
                    sys.executable,
                    str(DEPLOY_TOOL),
                    "apply",
                    "--release-manifest",
                    str(manifest),
                    "--deploy-root",
                    str(deploy_root),
                    "--data-dir",
                    str(data_dir),
                    "--backup-root",
                    str(backup_root),
                ],
                timeout=900,
            )
        )
        require(idempotent.get("idempotent") is True and idempotent.get("pointer_changed") is False, "second apply must be idempotent")

        verify = parse(run([sys.executable, str(DEPLOY_TOOL), "verify", "--deploy-root", str(deploy_root), "--data-dir", str(data_dir)], timeout=120))
        require(verify.get("health_ok") is True and verify.get("readiness_ok") is True, f"canary verify failed: {verify}")

        second = parse(
            run(
                [
                    sys.executable,
                    str(DEPLOY_TOOL),
                    "apply",
                    "--release-manifest",
                    str(manifest_b),
                    "--deploy-root",
                    str(deploy_root),
                    "--data-dir",
                    str(data_dir),
                    "--backup-root",
                    str(backup_root),
                ],
                timeout=900,
            )
        )
        require(second.get("pointer_changed") is True, "second release apply should change pointer")
        pointer_before_bad = (deploy_root / "current_release.detjson").read_text(encoding="utf-8")
        bad = run(
            [
                sys.executable,
                str(DEPLOY_TOOL),
                "apply",
                "--release-manifest",
                str(bad_manifest),
                "--deploy-root",
                str(deploy_root),
                "--data-dir",
                str(data_dir),
                "--backup-root",
                str(backup_root),
            ],
            timeout=900,
            expect_ok=False,
        )
        require("E_PRIVATE_PILOT_DEPLOY" in bad.stderr or "E_RELEASE_VERIFY" in bad.stderr, "bad release must be rejected")
        require((deploy_root / "current_release.detjson").read_text(encoding="utf-8") == pointer_before_bad, "bad release changed current pointer")

        rollback = parse(run([sys.executable, str(DEPLOY_TOOL), "rollback", "--deploy-root", str(deploy_root)], timeout=120))
        require(rollback.get("data_restore_included") is False, "rollback must not restore data")
        pointer = json.loads((deploy_root / "current_release.detjson").read_text(encoding="utf-8"))
        require(pointer.get("release_id") == first.get("release_id"), "rollback did not restore previous release pointer")
        restored_store = SeamgrimProjectStore(data_dir, sqlite_synchronous="FULL")
        require(restored_store.get_project(project_id)["project_id"] == project_id, "rollback must not delete data")

    pack = run([sys.executable, "tests/run_pack_golden.py", "seamgrim_private_pilot_release_contract_v1"], timeout=900)
    require("pack golden ok" in pack.stdout, "pack marker failed")
    ssot = run(["git", "status", "--short", "--", "docs/ssot"])
    require(not ssot.stdout.strip(), f"docs/ssot changed:\n{ssot.stdout}")
    print("[seamgrim-private-pilot-deploy] PASS")


if __name__ == "__main__":
    main()
