#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def fail(detail: str) -> int:
    print(f"check=seamgrim_sharing_publishing_surface detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    runner = ROOT / "tests" / "seamgrim_sharing_publishing_surface_runner.mjs"
    contract_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "platform_contract.js"
    index_html = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
    missing = [str(path) for path in (runner, contract_js, index_html) if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))

    proc = subprocess.run(
        ["node", "--no-warnings", str(runner)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or f"returncode={proc.returncode}"
        return fail(f"node_runner_failed:{detail}")

    contract_text = _read(contract_js)
    html_text = _read(index_html)
    static_required = [
        ("export const ShareKind = Object.freeze({" in contract_text, "share_kind_missing"),
        ('LINK: "link"' in contract_text, "share_kind_link_missing"),
        ('CLONE: "clone"' in contract_text, "share_kind_clone_missing"),
        ('PACKAGE: "package"' in contract_text, "share_kind_package_missing"),
        ("export const Role = Object.freeze({" in contract_text, "role_missing"),
        ('OWNER: "owner"' in contract_text, "role_owner_missing"),
        ('EDITOR: "editor"' in contract_text, "role_editor_missing"),
        ('VIEWER: "viewer"' in contract_text, "role_viewer_missing"),
        ('PUBLISHER: "publisher"' in contract_text, "role_publisher_missing"),
        ("export const Visibility = Object.freeze({" in contract_text, "visibility_missing"),
        ('PRIVATE: "private"' in contract_text, "visibility_private_missing"),
        ('TEAM: "team"' in contract_text, "visibility_team_missing"),
        ('INTERNAL: "internal"' in contract_text, "visibility_internal_missing"),
        ('PUBLIC: "public"' in contract_text, "visibility_public_missing"),
        ("export const PublishPolicy = Object.freeze({" in contract_text, "publish_policy_missing"),
        ("ARTIFACT_TRACKS_DRAFT: false" in contract_text, "publish_policy_tracks_draft_violation"),
        ('REPUBLISH_MODE: "new_artifact"' in contract_text, "publish_policy_republish_mode_missing"),
        ("SOURCE_REVISION_ID_REQUIRED: true" in contract_text, "publish_policy_source_revision_missing"),
        ("btn-share-clone" in html_text, "menu_share_clone_missing"),
        ("btn-publish" in html_text, "menu_publish_missing"),
    ]
    failures = [name for ok, name in static_required if not ok]
    if failures:
        return fail(",".join(failures))

    detail = (proc.stdout or "").strip() or "seamgrim sharing/publishing surface check ok"
    print(detail)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

