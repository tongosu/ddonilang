#!/usr/bin/env python
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
APPROVAL_PHRASE = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다"
WORK_ITEM = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1"
BUILD_ROOT_CANDIDATES = [
    Path(os.environ.get("DDN_CODEX_BUILD_ROOT", "")) if os.environ.get("DDN_CODEX_BUILD_ROOT") else None,
    Path("I:/home/urihanl/ddn/codex/build"),
    Path("C:/ddn/codex/build"),
]
FIXED_ZIP_DATE = (2026, 6, 11, 0, 0, 0)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def select_build_root() -> Path:
    for candidate in BUILD_ROOT_CANDIDATES:
        if candidate and candidate.exists():
            return candidate
    fallback = Path("C:/ddn/codex/build")
    fallback.mkdir(parents=True, exist_ok=True)
    return fallback


def iter_static_files(source: Path) -> list[Path]:
    files = [
        path
        for path in source.rglob("*")
        if path.is_file()
        and "__pycache__" not in path.parts
        and "node_modules" not in path.parts
    ]
    return sorted(files, key=lambda path: path.relative_to(source).as_posix())


def build_static_bundle(source: Path, target: Path) -> None:
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists():
        target.unlink()
    with zipfile.ZipFile(target, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for path in iter_static_files(source):
            rel = path.relative_to(source).as_posix()
            info = zipfile.ZipInfo(rel, date_time=FIXED_ZIP_DATE)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            archive.writestr(info, path.read_bytes())


def build_local_package_sample(target: Path) -> None:
    pack_dir = ROOT / "pack" / "studio_local_share_and_packaging_v1"
    contract_path = pack_dir / "contract.detjson"
    golden_path = pack_dir / "golden.jsonl"
    payload = {
        "schema": "ddn.studio.public_release.local_package_sample.v1",
        "work_item": WORK_ITEM,
        "source_pack": "pack/studio_local_share_and_packaging_v1",
        "source_contract_sha256": sha256_file(contract_path),
        "source_golden_sha256": sha256_file(golden_path),
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "cloud_sync_claim": False,
    }
    write_json(target, payload)


def copy_rc_matrix(target: Path) -> None:
    source = ROOT / "pack" / "studio_release_candidate_v1" / "rc_matrix.detjson"
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)


def write_checksum_manifest(output_root: Path, asset_paths: list[Path], target: Path) -> list[dict]:
    rows = []
    for path in sorted(asset_paths, key=lambda item: item.name):
        rows.append(
            {
                "path": path.name,
                "sha256": sha256_file(path),
                "bytes": path.stat().st_size,
            }
        )
    target.write_text(
        "".join(f"{row['sha256']}  {row['path']}\n" for row in rows),
        encoding="utf-8",
    )
    return rows


def execute_release(approval_phrase: str) -> dict:
    if approval_phrase != APPROVAL_PHRASE:
        raise SystemExit("approval phrase mismatch")

    asset_plan = read_json(ROOT / "pack" / "studio_public_release_asset_plan_v1" / "release_assets.detjson")
    build_root = select_build_root()
    output_root = build_root / "studio_release"
    output_root.mkdir(parents=True, exist_ok=True)

    static_bundle = output_root / "studio-static-bundle.zip"
    package_sample = output_root / "studio-local-package-sample.detjson"
    rc_matrix = output_root / "studio-rc-matrix.detjson"
    checksum_manifest = output_root / "SHA256SUMS.txt"
    release_manifest = output_root / "release_manifest.detjson"

    build_static_bundle(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui", static_bundle)
    build_local_package_sample(package_sample)
    copy_rc_matrix(rc_matrix)
    checksum_rows = write_checksum_manifest(output_root, [package_sample, rc_matrix, static_bundle], checksum_manifest)

    release_payload = {
        "schema": "ddn.studio.public_release_execution.v1",
        "work_item": WORK_ITEM,
        "approval_phrase": approval_phrase,
        "approval_confirmed": True,
        "release_execution_claim": True,
        "local_release_artifacts_generated": True,
        "public_release_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "cloud_sync_claim": False,
        "artifact_signing_claim": False,
        "output_root": output_root.as_posix(),
        "asset_plan": "pack/studio_public_release_asset_plan_v1/release_assets.detjson",
        "asset_plan_sha256": sha256_file(ROOT / "pack" / "studio_public_release_asset_plan_v1" / "release_assets.detjson"),
        "checksum_manifest": checksum_manifest.name,
        "artifacts": checksum_rows,
        "planned_assets": [asset["id"] for asset in asset_plan["assets"]],
        "external_publish_status": "not_attempted_no_github_cli_auth_evidence",
    }
    write_json(release_manifest, release_payload)
    return release_payload


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--approval-phrase", required=True)
    args = parser.parse_args()
    payload = execute_release(args.approval_phrase)
    print(json.dumps(payload, ensure_ascii=False, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
