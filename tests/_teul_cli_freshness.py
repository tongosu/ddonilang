from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path

DEFAULT_SKIP_ENV_KEYS = (
    "DDN_RUN_PACK_GOLDEN_SKIP_BIN_FRESHNESS_CHECK",
    "DDN_SKIP_TEUL_CLI_BIN_FRESHNESS_CHECK",
)


def latest_teul_cli_source_mtime(root: Path) -> float:
    manifest = root / "tools" / "teul-cli" / "Cargo.toml"
    src_dir = root / "tools" / "teul-cli" / "src"
    if not manifest.exists() or not src_dir.exists():
        return 0.0
    latest = manifest.stat().st_mtime
    for file in src_dir.rglob("*.rs"):
        mtime = file.stat().st_mtime
        if mtime > latest:
            latest = mtime
    return latest


def is_teul_cli_bin_fresh(
    root: Path,
    candidate: Path,
    *,
    skip_env_keys: tuple[str, ...] = DEFAULT_SKIP_ENV_KEYS,
) -> bool:
    for env_key in skip_env_keys:
        if os.environ.get(env_key, "").strip() == "1":
            return True
    try:
        bin_mtime = candidate.stat().st_mtime
    except OSError:
        return False
    source_latest = latest_teul_cli_source_mtime(root)
    if source_latest <= 0.0:
        return True
    return bin_mtime >= source_latest


def resolve_teul_cli_bin(
    root: Path,
    *,
    candidates: list[Path],
    include_which: bool = False,
) -> Path | None:
    for candidate in candidates:
        if candidate.exists() and is_teul_cli_bin_fresh(root, candidate):
            return candidate
    if include_which:
        which = shutil.which("teul-cli")
        if which:
            resolved = Path(which)
            if resolved.exists() and is_teul_cli_bin_fresh(root, resolved):
                return resolved
    return None


def build_teul_cli_cmd(
    root: Path,
    args: list[str],
    *,
    candidates: list[Path],
    include_which: bool = False,
    manifest_path: Path | None = None,
) -> list[str]:
    teul_cli = resolve_teul_cli_bin(
        root,
        candidates=candidates,
        include_which=include_which,
    )
    if teul_cli is not None:
        return [str(teul_cli), *args]
    manifest = manifest_path or (root / "tools" / "teul-cli" / "Cargo.toml")
    return [
        "cargo",
        "run",
        "-q",
        "--manifest-path",
        str(manifest),
        "--",
        *args,
    ]


def ensure_teul_cli_bin(
    root: Path,
    *,
    candidates: list[Path],
    include_which: bool = False,
    manifest_path: Path | None = None,
    build_env: dict[str, str] | None = None,
) -> Path:
    found = resolve_teul_cli_bin(
        root,
        candidates=candidates,
        include_which=include_which,
    )
    if found is not None:
        return found

    manifest = manifest_path or (root / "tools" / "teul-cli" / "Cargo.toml")
    env = os.environ.copy()
    if build_env:
        env.update(build_env)
    build = subprocess.run(
        ["cargo", "build", "--manifest-path", str(manifest)],
        cwd=root,
        env=env,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if build.returncode != 0:
        raise SystemExit(build.returncode)

    found = resolve_teul_cli_bin(
        root,
        candidates=candidates,
        include_which=include_which,
    )
    if found is not None:
        return found
    raise FileNotFoundError("missing teul-cli binary")
