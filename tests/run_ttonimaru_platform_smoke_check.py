#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PLATFORM = ROOT / "solutions" / "ttonimaru_platform"


def fail(detail: str) -> int:
    print(f"check=ttonimaru_platform_smoke detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    required = [
        PLATFORM / "api" / "app.py",
        PLATFORM / "api" / "auth.py",
        PLATFORM / "api" / "routes_internal_v0.py",
        PLATFORM / "api" / "routes_public_v1.py",
        PLATFORM / "storage" / "sqlite_store.py",
        PLATFORM / "contracts" / "publication_manifest.v1.json",
        PLATFORM / "contracts" / "package_metadata_stub.v1.json",
        PLATFORM / "contracts" / "share_redirect_contract.md",
    ]
    missing = [str(path) for path in required if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))

    app_text = _read(PLATFORM / "api" / "app.py")
    auth_text = _read(PLATFORM / "api" / "auth.py")
    public_text = _read(PLATFORM / "api" / "routes_public_v1.py")
    internal_text = _read(PLATFORM / "api" / "routes_internal_v0.py")
    store_text = _read(PLATFORM / "storage" / "sqlite_store.py")

    static_required = [
        ("FastAPI" in app_text, "fastapi_app_missing"),
        ("dev-owner-token" in auth_text, "owner_token_missing"),
        ("dev-viewer-token" in auth_text, "viewer_token_missing"),
        ("Authorization: Bearer" not in public_text, "public_text_should_not_embed_auth_prompt"),
        ("RedirectResponse" in public_text and "status_code=302" in public_text, "alias_302_redirect_missing"),
        ("package_metadata_stub.v1" in public_text, "package_stub_schema_missing"),
        ("install_supported" in public_text and "False" in public_text, "install_forbidden_stub_missing"),
        ("state_hash" in internal_text, "state_hash_passthrough_missing"),
        ("source_hash(ddn_source)" in store_text, "source_hash_missing"),
        ("state_hash_generated_by_server" not in store_text, "state_hash_generation_claim_forbidden"),
    ]
    failures = [name for ok, name in static_required if not ok]
    if failures:
        return fail(",".join(failures))

    proc = subprocess.run(
        [sys.executable, "-m", "pytest", str(PLATFORM / "tests"), "-q"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=120,
    )
    if proc.returncode != 0:
        detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or f"returncode={proc.returncode}"
        return fail("pytest_failed:" + detail)

    print((proc.stdout or "").strip())
    print("ttonimaru platform smoke check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

