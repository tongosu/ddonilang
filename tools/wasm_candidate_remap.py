#!/usr/bin/env python3
"""R3N candidate-only WASM source remap, inventory, seal and projection custody."""

from __future__ import annotations

import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
from typing import NoReturn


POLICY_SCHEMA = "ddn.wasm_source_remap_policy.r3n_candidate.v1"
RECEIPT_SCHEMA = "ddn.wasm_source_remap_receipt.r3n_candidate.v1"
SUBSTITUTION_SCHEMA = "ddn.public_projection_wasm_candidate_substitution.r3n.v1"
POLICY_MAPPINGS = (
    ("REPOSITORY_ROOT", "/ddn-src"),
    ("CARGO_REGISTRY_ROOT", "/ddn-cargo-registry"),
    ("RUSTUP_TOOLCHAIN_ROOT", "/ddn-rust-toolchain"),
)
GENERATED_FILES = (
    ".gitignore",
    "ddonirang_tool.d.ts",
    "ddonirang_tool.js",
    "ddonirang_tool_bg.wasm",
    "ddonirang_tool_bg.wasm.d.ts",
    "package.json",
)
PROJECTION_FAMILY = {
    "ddonirang_tool.d.ts": "ddonirang_tool.d.ts",
    "ddonirang_tool.js": "ddonirang_tool.js",
    "ddonirang_tool_bg.wasm": "ddonirang_tool_bg.wasm",
    "package.json": "package.json",
}
PROJECTION_WASM_DIR = PurePosixPath("solutions/seamgrim_ui_mvp/ui/wasm")
PATH_START = re.compile(
    r"(?ix)(?:"
    r"(?<![A-Z0-9_])[A-Z]:[\\/]"
    r"|\\\\(?:[?.][\\/]|[A-Z0-9._$ -]+[\\/][A-Z0-9._$ -]+[\\/])"
    r"|\bfile://(?:/[A-Z]:[\\/]|/|[^/\s]+/)"
    r"|/(?:home|users|mnt)/[^\r\n\"'<>\s]*(?:ddn|codex|build|work|official[-_]store)"
    r")"
)
PRIVATE_PATH = re.compile(r"(?i)(?:^|[\\/])(?:pack|work|docs[\\/]ssot)(?:[\\/]|$)")
OFFICIAL_STORE = re.compile(r"(?i)official[-_]store")
SECRET_TOKEN = re.compile(r"(?i)(?:password|passwd|secret|credential|api[_-]?key|token)")
GIT_OBJECT_ID = re.compile(r"^[0-9a-f]{40}$")


def fail(code: str, detail: str) -> NoReturn:
    raise SystemExit(f"{code}: {detail}")


def canonical_bytes(value: object) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        + "\n"
    ).encode("utf-8")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_row(path: Path) -> dict[str, object]:
    data = path.read_bytes()
    return {"bytes": len(data), "sha256": sha256(data)}


def run_text(*args: str, cwd: Path | None = None) -> str:
    result = subprocess.run(
        list(args), cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        text=True, encoding="utf-8", errors="replace", check=False,
    )
    if result.returncode != 0:
        fail("E_WASM_REMAP_TOOL_COMMAND", result.stderr.strip() or " ".join(args))
    return result.stdout.strip()


def run_bytes(*args: str, cwd: Path | None = None) -> bytes:
    result = subprocess.run(
        list(args), cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        fail("E_WASM_REMAP_TOOL_COMMAND", detail or " ".join(args))
    return result.stdout


def git_blob_sha256(repo: Path, relative_path: str) -> str:
    """Hash the fixed-HEAD blob, independent of LF/CRLF materialization."""
    return sha256(run_bytes("git", "show", f"HEAD:{relative_path}", cwd=repo))


def resolved_directory(value: str | Path, label: str) -> Path:
    path = Path(value).expanduser()
    try:
        resolved = path.resolve(strict=True)
    except OSError as error:
        fail("E_WASM_REMAP_REQUIRED_ROOT_MISSING", f"{label}: {error}")
    if not resolved.is_dir():
        fail("E_WASM_REMAP_REQUIRED_ROOT_MISSING", f"{label}: {resolved}")
    return resolved


def comparison_key(path: Path) -> str:
    value = str(path).replace("/", "\\").rstrip("\\")
    return os.path.normcase(value).casefold()


def boundary_prefix(parent: str, child: str) -> bool:
    return child == parent or child.startswith(parent + "\\")


def build_contract(repo_root: str | Path, cargo_home: str | Path, rust_sysroot: str | Path) -> dict[str, object]:
    repo = resolved_directory(repo_root, "REPOSITORY_ROOT")
    cargo = resolved_directory(cargo_home, "CARGO_HOME")
    registry = resolved_directory(cargo / "registry" / "src", "CARGO_REGISTRY_ROOT")
    sysroot = resolved_directory(rust_sysroot, "RUSTUP_TOOLCHAIN_ROOT")
    physical = (
        ("REPOSITORY_ROOT", repo, "/ddn-src"),
        ("CARGO_REGISTRY_ROOT", registry, "/ddn-cargo-registry"),
        ("RUSTUP_TOOLCHAIN_ROOT", sysroot, "/ddn-rust-toolchain"),
    )
    keys = [comparison_key(path) for _, path, _ in physical]
    if len(set(keys)) != len(keys):
        fail("E_WASM_REMAP_DUPLICATE_ROOT", repr(keys))
    for index, left in enumerate(keys):
        for right in keys[index + 1 :]:
            if boundary_prefix(left, right) or boundary_prefix(right, left):
                fail("E_WASM_REMAP_OVERLAPPING_ROOT", f"{left} :: {right}")
    virtuals = [virtual for _, _, virtual in physical]
    if len(set(virtuals)) != len(virtuals):
        fail("E_WASM_REMAP_DUPLICATE_VIRTUAL_ROOT", repr(virtuals))
    semantic = {
        "schema": POLICY_SCHEMA,
        "mapping_policy": [
            {"root_family": role, "virtual_root": virtual}
            for role, virtual in POLICY_MAPPINGS
        ],
        "normalization": "resolved_directory_windows_casefold_boundary_v1",
        "source_materialization": "fixed_head_git_blob_lf_clone_v1",
        "unknown_required_root": "FAIL_CLOSED",
        "inherited_rustflags": "FAIL_CLOSED_E_WASM_BUILD_INHERITED_RUSTFLAGS_UNSUPPORTED",
    }
    identity = "sha256:" + sha256(canonical_bytes(semantic))
    return {
        "compiler_flags": [
            f"--remap-path-prefix={path}={virtual}" for _, path, virtual in physical
        ],
        "physical_roots": {role: str(path) for role, path, _ in physical},
        "policy": {**semantic, "policy_identity": identity},
    }


def read_uleb(data: bytes, offset: int) -> tuple[int, int]:
    value = 0
    shift = 0
    while True:
        if offset >= len(data) or shift > 63:
            fail("E_WASM_BINARY_INVALID", "invalid ULEB section size")
        byte = data[offset]
        offset += 1
        value |= (byte & 0x7F) << shift
        if byte < 0x80:
            return value, offset
        shift += 7


def wasm_sections(data: bytes) -> list[dict[str, int | str]]:
    if len(data) < 8 or data[:4] != b"\0asm":
        return []
    sections: list[dict[str, int | str]] = []
    offset = 8
    ordinal = 0
    while offset < len(data):
        header = offset
        section_id = data[offset]
        offset += 1
        size, payload = read_uleb(data, offset)
        end = payload + size
        if end > len(data):
            fail("E_WASM_BINARY_INVALID", f"section {section_id} exceeds file")
        ordinal += 1
        sections.append(
            {
                "ordinal": ordinal,
                "section_id": section_id,
                "header_offset": header,
                "payload_offset": payload,
                "payload_end": end,
                "payload_bytes": size,
                "payload_sha256": sha256(data[payload:end]),
            }
        )
        offset = end
    return sections


def section_for_offset(sections: list[dict[str, int | str]], offset: int) -> dict[str, int | str] | None:
    for section in sections:
        if int(section["payload_offset"]) <= offset < int(section["payload_end"]):
            return section
    return None


def printable_runs(data: bytes):
    start: int | None = None
    for index, byte in enumerate(data):
        printable = byte >= 0x20 and byte != 0x7F
        if printable and start is None:
            start = index
        elif not printable and start is not None:
            yield start, data[start:index]
            start = None
    if start is not None:
        yield start, data[start:]


def family_for_path(path: str, roots: dict[str, str] | None) -> str:
    normalized = path.replace("/", "\\").casefold()
    if roots:
        for family in ("CARGO_REGISTRY_ROOT", "RUSTUP_TOOLCHAIN_ROOT", "REPOSITORY_ROOT"):
            root = roots.get(family)
            if root:
                key = str(root).replace("/", "\\").rstrip("\\").casefold()
                if normalized == key or normalized.startswith(key + "\\"):
                    return family
    if "\\.cargo\\registry\\src\\" in normalized:
        return "CARGO_REGISTRY_ROOT"
    if "\\.rustup\\toolchains\\" in normalized:
        return "RUSTUP_TOOLCHAIN_ROOT"
    if "\\.cargo\\" in normalized:
        return "CARGO_HOME_OTHER"
    if "\\.rustup\\" in normalized or "\\rustlib\\" in normalized:
        return "TOOLCHAIN_OTHER"
    return "UNKNOWN"


def origin_hint(path: str, family: str) -> str:
    normalized = path.replace("\\", "/")
    if family == "CARGO_REGISTRY_ROOT" and "/registry/src/" in normalized.casefold():
        tail = re.split(r"(?i)/registry/src/", normalized, maxsplit=1)[1]
        parts = tail.split("/")
        return "/".join(parts[1:]) if len(parts) > 1 else tail
    if family == "RUSTUP_TOOLCHAIN_ROOT" and "/library/" in normalized.casefold():
        return "library/" + re.split(r"(?i)/library/", normalized, maxsplit=1)[1]
    return normalized.rsplit("/", 1)[-1]


def remap_target_for_path(
    path: str, family: str, roots: dict[str, str] | None
) -> str:
    virtual_roots = dict(POLICY_MAPPINGS)
    virtual = virtual_roots.get(family)
    if virtual is None:
        return "UNKNOWN"
    physical_root: str | None = None
    if roots and roots.get(family):
        physical_root = str(roots[family]).rstrip("\\/")
    elif family == "CARGO_REGISTRY_ROOT":
        match = re.match(r"(?i)^(.*?[\\/]\.cargo[\\/]registry[\\/]src)(?=[\\/]|$)", path)
        physical_root = None if match is None else match.group(1)
    elif family == "RUSTUP_TOOLCHAIN_ROOT":
        match = re.match(r"(?i)^(.*?[\\/]\.rustup[\\/]toolchains[\\/][^\\/]+)(?=[\\/]|$)", path)
        physical_root = None if match is None else match.group(1)
    elif family == "REPOSITORY_ROOT" and roots and roots.get(family):
        physical_root = str(roots[family]).rstrip("\\/")
    if physical_root is None:
        return "UNKNOWN"
    path_key = comparison_key(Path(path))
    root_key = comparison_key(Path(physical_root))
    if not boundary_prefix(root_key, path_key):
        return "UNKNOWN"
    return virtual + path[len(physical_root) :]


def json_string_values(value: object):
    if isinstance(value, str):
        yield value
    elif isinstance(value, list):
        for item in value:
            yield from json_string_values(item)
    elif isinstance(value, dict):
        for item in value.values():
            yield from json_string_values(item)


def inventory_file(
    path: Path,
    roots: dict[str, str] | None = None,
    candidate_data: bytes | None = None,
) -> dict[str, object]:
    data = path.read_bytes()
    sections = wasm_sections(data)
    scan_data = data
    scan_scope = "RAW_FILE_BYTES"
    if path.suffix == ".detjson":
        try:
            document = json.loads(data.decode("utf-8"))
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            fail("E_WASM_CANDIDATE_JSON_INVALID", f"{path.name}: {error}")
        scan_data = b"\0".join(
            value.encode("utf-8") for value in json_string_values(document)
        )
        scan_scope = "CANONICAL_JSON_STRING_VALUES"
    rows: list[dict[str, object]] = []
    private_count = 0
    official_count = 0
    virtual_counts: Counter[str] = Counter()
    for virtual in (value for _, value in POLICY_MAPPINGS):
        virtual_counts[virtual] = scan_data.count(virtual.encode("utf-8"))
    for run_offset, raw in printable_runs(scan_data):
        text = raw.decode("utf-8", errors="replace")
        private_count += len(list(PRIVATE_PATH.finditer(text)))
        official_count += len(list(OFFICIAL_STORE.finditer(text)))
        for match in PATH_START.finditer(text):
            prefix = text[: match.start()].encode("utf-8", errors="replace")
            byte_offset = run_offset + len(prefix)
            decoded = text[match.start() :]
            raw_path = raw[len(prefix) :]
            family = family_for_path(decoded, roots)
            remap_target = remap_target_for_path(decoded, family, roots)
            section = section_for_offset(sections, byte_offset)
            rows.append(
                {
                    "byte_offset": byte_offset,
                    "raw_wasm_byte_offset": byte_offset,
                    "decoded_path": decoded,
                    "original_path": decoded,
                    "raw_matched_bytes_hex": raw_path.hex(),
                    "byte_length": len(raw_path),
                    "original_byte_slice_sha256": sha256(raw_path),
                    "root_family": family,
                    "root_prefix": (roots or {}).get(family, "UNKNOWN"),
                    "remap_target": remap_target,
                    "candidate_occurrence_count": (
                        None
                        if candidate_data is None or remap_target == "UNKNOWN"
                        else candidate_data.count(remap_target.encode("utf-8"))
                    ),
                    "section_id": None if section is None else section["section_id"],
                    "section_ordinal": None if section is None else section["ordinal"],
                    "originating_crate_or_file_hint": origin_hint(decoded, family),
                    "secret_or_credential": bool(SECRET_TOKEN.search(decoded)),
                    "remap_possible": family != "UNKNOWN",
                }
            )
    duplicates = Counter(str(row["decoded_path"]) for row in rows)
    for row in rows:
        row["identical_string_occurrences"] = duplicates[str(row["decoded_path"])]
    families = Counter(str(row["root_family"]) for row in rows)
    return {
        "path": path.name,
        "bytes": len(data),
        "sha256": sha256(data),
        "scan_scope": scan_scope,
        "internal_absolute_path_count": len(rows),
        "private_include_count": private_count,
        "official_store_path_count": official_count,
        "unknown_path_count": families.get("UNKNOWN", 0),
        "root_family_counts": dict(sorted(families.items())),
        "unique_path_count": len(duplicates),
        "virtual_root_occurrences": dict(sorted(virtual_counts.items())),
        "sections": sections,
        "rows": rows,
    }


def offset_bijection(original: Path, candidate: Path) -> dict[str, object]:
    original_data = original.read_bytes()
    candidate_data = candidate.read_bytes()
    report = inventory_file(original, candidate_data=candidate_data)
    candidate_report = inventory_file(candidate)
    rows = report["rows"]
    assert isinstance(rows, list)
    offsets: list[int] = []
    invalid_slice_count = 0
    remap_occurrence_mismatch_count = 0
    evidence_rows: list[dict[str, object]] = []
    for raw in rows:
        assert isinstance(raw, dict)
        offset = int(raw["raw_wasm_byte_offset"])
        length = int(raw["byte_length"])
        expected_slice = bytes.fromhex(str(raw["raw_matched_bytes_hex"]))
        actual_slice = original_data[offset : offset + length]
        offsets.append(offset)
        if actual_slice != expected_slice or sha256(actual_slice) != raw["original_byte_slice_sha256"]:
            invalid_slice_count += 1
        if int(raw["candidate_occurrence_count"] or 0) != 1:
            remap_occurrence_mismatch_count += 1
        evidence_rows.append(
            {
                "original_path": raw["original_path"],
                "root_class": raw["root_family"],
                "raw_wasm_byte_offset": offset,
                "byte_length": length,
                "original_byte_slice_sha256": raw["original_byte_slice_sha256"],
                "remap_target": raw["remap_target"],
                "candidate_occurrence_count": raw["candidate_occurrence_count"],
            }
        )
    duplicate_offset_count = len(offsets) - len(set(offsets))
    duplicate_or_missing = duplicate_offset_count + invalid_slice_count
    result = {
        "schema": "ddn.wasm_embedded_path_offset_bijection.r3n_fu01.v1",
        "original": file_row(original),
        "candidate": file_row(candidate),
        "root_class_counts": report["root_family_counts"],
        "original_offset_bijection": f"{len(rows) - duplicate_or_missing}/{len(rows)}",
        "duplicate_or_missing_offset": duplicate_or_missing,
        "candidate_internal_path_count": candidate_report["internal_absolute_path_count"],
        "unknown_path_count": report["unknown_path_count"],
        "candidate_occurrence_mismatch_count": remap_occurrence_mismatch_count,
        "rows": evidence_rows,
    }
    if (
        len(rows) != 115
        or duplicate_or_missing != 0
        or int(candidate_report["internal_absolute_path_count"]) != 0
        or int(report["unknown_path_count"]) != 0
        or remap_occurrence_mismatch_count != 0
    ):
        fail(
            "E_WASM_CANDIDATE_OFFSET_BIJECTION",
            json.dumps(
                {
                    "rows": len(rows),
                    "duplicate_or_missing_offset": duplicate_or_missing,
                    "candidate_internal_path_count": candidate_report["internal_absolute_path_count"],
                    "unknown_path_count": report["unknown_path_count"],
                    "candidate_occurrence_mismatch_count": remap_occurrence_mismatch_count,
                },
                sort_keys=True,
            ),
        )
    return result


def inventory_target(path: Path, roots: dict[str, str] | None = None) -> dict[str, object]:
    files = [path] if path.is_file() else sorted(item for item in path.rglob("*") if item.is_file())
    reports = [inventory_file(item, roots) for item in files]
    return {
        "schema": "ddn.wasm_embedded_path_inventory.r3n.v1",
        "target_file_count": len(files),
        "internal_absolute_path_count": sum(int(row["internal_absolute_path_count"]) for row in reports),
        "private_include_count": sum(int(row["private_include_count"]) for row in reports),
        "official_store_path_count": sum(int(row["official_store_path_count"]) for row in reports),
        "unknown_path_count": sum(int(row["unknown_path_count"]) for row in reports),
        "files": reports,
    }


def artifact_rows(output: Path) -> dict[str, object]:
    rows: dict[str, object] = {}
    for name in GENERATED_FILES:
        path = output / name
        if not path.is_file():
            fail("E_WASM_CANDIDATE_ARTIFACT_MISSING", name)
        rows[name] = file_row(path)
    return rows


def require_fixed_source(repo: Path) -> tuple[str, str]:
    """Read-only gate: a caller's materialization flag is not Git custody."""
    if os.environ.get("DDN_WASM_CANONICAL_SOURCE_MATERIALIZED") != "1":
        fail("E_WASM_CANDIDATE_MATERIALIZATION_REQUIRED", "fixed-HEAD LF source required")
    checked = subprocess.run(
        ["git", "--no-optional-locks", "diff", "--quiet", "HEAD", "--"],
        cwd=repo, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
        timeout=30,
    )
    if checked.returncode == 1:
        fail("E_WASM_CANDIDATE_SOURCE_DIRTY", "working source differs from HEAD")
    if checked.returncode != 0:
        fail("E_WASM_CANDIDATE_SOURCE_IDENTITY_UNAVAILABLE", "Git source check failed")
    # Bounded source inventory only; never scan the shared root's foreign work.
    untracked = subprocess.run(
        ["git", "--no-optional-locks", "ls-files", "--others", "--exclude-standard", "-z", "--",
         "core", "lang", "tool", "Cargo.toml", "Cargo.lock", ".cargo", "scripts", "tools",
         "solutions/seamgrim_ui_mvp/samples"],
        cwd=repo, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
        timeout=30,
    )
    if untracked.returncode != 0:
        fail("E_WASM_CANDIDATE_SOURCE_IDENTITY_UNAVAILABLE", "Git source inventory failed")
    if untracked.stdout:
        fail("E_WASM_CANDIDATE_SOURCE_UNTRACKED", "source inputs are not bound to HEAD")
    return (
        run_text("git", "--no-optional-locks", "rev-parse", "HEAD", cwd=repo),
        run_text("git", "--no-optional-locks", "show", "-s", "--format=%T", "HEAD", cwd=repo),
    )


def semantic_receipt_payload(
    repo: Path, contract: dict[str, object], output: Path, profile: str, target: str,
    rustc_version: str, cargo_version: str, wasm_tool_version: str,
) -> dict[str, object]:
    if os.environ.get("CARGO_NET_OFFLINE", "").casefold() not in {"1", "true"}:
        fail("E_WASM_CANDIDATE_OFFLINE_REQUIRED", "CARGO_NET_OFFLINE=true")
    source_identity = require_fixed_source(repo)
    roots = contract["physical_roots"]
    assert isinstance(roots, dict)
    inventory = inventory_target(output, {str(k): str(v) for k, v in roots.items()})
    for key in (
        "internal_absolute_path_count",
        "private_include_count",
        "official_store_path_count",
        "unknown_path_count",
    ):
        if int(inventory[key]) != 0:
            fail("E_WASM_CANDIDATE_PATH_REMAINS", f"{key}={inventory[key]}")
    policy = contract["policy"]
    assert isinstance(policy, dict)
    if require_fixed_source(repo) != source_identity:
        fail("E_WASM_CANDIDATE_SOURCE_CHANGED", "source changed while sealing")
    return {
        "schema": RECEIPT_SCHEMA,
        "status": "CANDIDATE_ONLY_NOT_TRACKED_NOT_PUBLIC_MEMBERSHIP",
        "source_commit": source_identity[0],
        "source_tree": source_identity[1],
        "cargo_lock_git_blob_sha256": git_blob_sha256(repo, "Cargo.lock"),
        "build_entrypoint_git_blob_sha256": git_blob_sha256(
            repo, "scripts/build_wasm_tool.ps1"
        ),
        "policy": policy,
        "profile": profile,
        "target": target,
        "environment_allowlist": {
            "CARGO_NET_OFFLINE": "true",
            "CARGO_TARGET_DIR": "LOCAL_VALUE_EXCLUDED_FROM_SEMANTIC_IDENTITY",
            "DDN_BUILD_DIR": "LOCAL_VALUE_EXCLUDED_FROM_SEMANTIC_IDENTITY",
            "DDN_WASM_CANONICAL_SOURCE_MATERIALIZED": "1",
        },
        "toolchain": {
            "rustc": rustc_version,
            "cargo": cargo_version,
            "wasm_tool": wasm_tool_version,
        },
        "artifacts": artifact_rows(output),
        "path_scan": {
            key: inventory[key]
            for key in (
                "target_file_count",
                "internal_absolute_path_count",
                "private_include_count",
                "official_store_path_count",
                "unknown_path_count",
            )
        },
        "tracked_wasm_write": 0,
        "tracked_manifest_write": 0,
        "public_membership": False,
    }


def verify_semantic_identity(receipt: dict[str, object]) -> str:
    claimed = str(receipt.get("semantic_identity", ""))
    payload = dict(receipt)
    payload.pop("semantic_identity", None)
    actual = "sha256:" + sha256(canonical_bytes(payload))
    if claimed and claimed != actual:
        fail("E_WASM_CANDIDATE_RECEIPT_IDENTITY", f"{claimed} != {actual}")
    return actual


def bind_projection_source_identity(
    candidate_receipt: dict[str, object], projection_receipt: dict[str, object]
) -> dict[str, str]:
    candidate_commit = candidate_receipt.get("source_commit")
    candidate_tree = candidate_receipt.get("source_tree")
    projection_commit = projection_receipt.get("source_commit")
    projection_tree = projection_receipt.get("source_tree_identity")
    values = {
        "candidate_receipt.source_commit": candidate_commit,
        "candidate_receipt.source_tree": candidate_tree,
        "projection_receipt.source_commit": projection_commit,
    }
    for field, value in values.items():
        if not isinstance(value, str) or GIT_OBJECT_ID.fullmatch(value) is None:
            fail("E_WASM_CANDIDATE_PROJECTION_SOURCE_IDENTITY_MISMATCH", f"{field} malformed")
    if (
        not isinstance(projection_tree, str)
        or not projection_tree.startswith("git-tree:")
        or GIT_OBJECT_ID.fullmatch(projection_tree.removeprefix("git-tree:")) is None
    ):
        fail(
            "E_WASM_CANDIDATE_PROJECTION_SOURCE_IDENTITY_MISMATCH",
            "projection_receipt.source_tree_identity malformed",
        )
    expected_tree = "git-tree:" + str(candidate_tree)
    if candidate_commit != projection_commit or expected_tree != projection_tree:
        fail(
            "E_WASM_CANDIDATE_PROJECTION_SOURCE_IDENTITY_MISMATCH",
            f"candidate={candidate_commit}/{expected_tree} projection={projection_commit}/{projection_tree}",
        )
    return {"source_commit": str(candidate_commit), "source_tree_identity": expected_tree}


def tree_identity(files: dict[str, object]) -> str:
    rows = []
    for path, raw in sorted(files.items()):
        if not isinstance(raw, dict):
            fail("E_WASM_CANDIDATE_PROJECTION_TREE", path)
        rows.append(f"{path}\t{raw.get('bytes')}\t{raw.get('sha256')}\n".encode("utf-8"))
    return "sha256:" + sha256(b"".join(rows))


def substitute_projection(projection: Path, receipt_dir: Path, candidate_dir: Path) -> dict[str, object]:
    tree_path = receipt_dir / "PUBLIC_TREE_MANIFEST.detjson"
    projection_receipt_path = receipt_dir / "PUBLIC_PROJECTION_RECEIPT.detjson"
    candidate_receipt_path = candidate_dir / "R3N_WASM_REMAP_RECEIPT.detjson"
    for path in (tree_path, projection_receipt_path, candidate_receipt_path):
        if not path.is_file():
            fail("E_WASM_CANDIDATE_SUBSTITUTION_INPUT", str(path))
    tree = json.loads(tree_path.read_text(encoding="utf-8"))
    projection_receipt = json.loads(projection_receipt_path.read_text(encoding="utf-8"))
    candidate_receipt = json.loads(candidate_receipt_path.read_text(encoding="utf-8"))
    candidate_identity = verify_semantic_identity(candidate_receipt)
    source_identity = bind_projection_source_identity(candidate_receipt, projection_receipt)
    files = tree.get("files")
    if not isinstance(files, dict):
        fail("E_WASM_CANDIDATE_PROJECTION_TREE", "files")
    before_identity = str(tree.get("public_tree_identity", ""))
    artifacts = candidate_receipt.get("artifacts")
    if not isinstance(artifacts, dict):
        fail("E_WASM_CANDIDATE_RECEIPT", "artifacts")
    for source_name, target_name in PROJECTION_FAMILY.items():
        target = (PROJECTION_WASM_DIR / target_name).as_posix()
        raw_entry = files.get(target)
        if not isinstance(raw_entry, dict) or raw_entry.get("role") != "TRACKED_WASM_FAMILY":
            fail("E_WASM_CANDIDATE_SUBSTITUTION_TARGET", target)
        source = candidate_dir / source_name
        expected = artifacts.get(source_name)
        if not source.is_file() or not isinstance(expected, dict) or file_row(source) != expected:
            fail("E_WASM_CANDIDATE_ARTIFACT_IDENTITY", source_name)
        destination = projection.joinpath(*PurePosixPath(target).parts)
        payload = source.read_bytes()
        destination.write_bytes(payload)
        raw_entry["bytes"] = len(payload)
        raw_entry["sha256"] = sha256(payload)
        raw_entry["candidate_substitution"] = candidate_identity
    candidate_manifest = {
        "schema": "ddn.wasm_candidate_projection_manifest.r3n.v1",
        "status": "CANDIDATE_ONLY_NOT_TRACKED_NOT_PUBLIC_MEMBERSHIP",
        "candidate_receipt_identity": candidate_identity,
        "source_commit": source_identity["source_commit"],
        "source_tree_identity": source_identity["source_tree_identity"],
        "remap_policy_identity": candidate_receipt["policy"]["policy_identity"],
        "artifacts": {name: artifacts[name] for name in sorted(PROJECTION_FAMILY)},
    }
    manifest_target = (PROJECTION_WASM_DIR / "ddonirang_tool.manifest.detjson").as_posix()
    manifest_entry = files.get(manifest_target)
    if not isinstance(manifest_entry, dict) or manifest_entry.get("role") != "TRACKED_WASM_FAMILY":
        fail("E_WASM_CANDIDATE_SUBSTITUTION_TARGET", manifest_target)
    manifest_bytes = canonical_bytes(candidate_manifest)
    projection.joinpath(*PurePosixPath(manifest_target).parts).write_bytes(manifest_bytes)
    manifest_entry["bytes"] = len(manifest_bytes)
    manifest_entry["sha256"] = sha256(manifest_bytes)
    manifest_entry["candidate_substitution"] = candidate_identity
    after_identity = tree_identity(files)
    tree["public_tree_identity"] = after_identity
    projection_receipt["included_path_role_closure_identity"] = after_identity
    projection_receipt["output_public_tree_identity"] = after_identity
    projection_receipt["validation_receipt_identity"] = "OPEN"
    projection_receipt["candidate_artifact_identity"] = candidate_identity
    projection_receipt["remap_policy_identity"] = candidate_receipt["policy"]["policy_identity"]
    tree_path.write_bytes(canonical_bytes(tree))
    substitution = {
        "schema": SUBSTITUTION_SCHEMA,
        "status": "TEST_OWNED_ISOLATED_OUTPUT_ONLY",
        "source_commit": projection_receipt["source_commit"],
        "source_tree_identity": projection_receipt["source_tree_identity"],
        "projection_manifest_identity": projection_receipt["projection_manifest_identity"],
        "before_public_tree_identity": before_identity,
        "after_public_tree_identity": after_identity,
        "candidate_artifact_identity": candidate_identity,
        "remap_policy_identity": candidate_receipt["policy"]["policy_identity"],
        "tracked_source_write_count": 0,
        "public_membership": False,
    }
    substitution_bytes = canonical_bytes(substitution)
    substitution_path = receipt_dir / "R3N_WASM_CANDIDATE_SUBSTITUTION_RECEIPT.detjson"
    substitution_path.write_bytes(substitution_bytes)
    projection_receipt["candidate_substitution_receipt_identity"] = "sha256:" + sha256(substitution_bytes)
    projection_receipt_path.write_bytes(canonical_bytes(projection_receipt))
    return substitution


def main() -> None:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    contract_parser = sub.add_parser("contract")
    contract_parser.add_argument("--repo-root", required=True)
    contract_parser.add_argument("--cargo-home", required=True)
    contract_parser.add_argument("--rust-sysroot", required=True)
    inspect_parser = sub.add_parser("inspect")
    inspect_parser.add_argument("--target", required=True, type=Path)
    inspect_parser.add_argument("--repo-root")
    inspect_parser.add_argument("--cargo-home")
    inspect_parser.add_argument("--rust-sysroot")
    inspect_parser.add_argument("--output", type=Path)
    inspect_parser.add_argument("--require-zero", action="store_true")
    offset_parser = sub.add_parser("prove-offsets")
    offset_parser.add_argument("--original", required=True, type=Path)
    offset_parser.add_argument("--candidate", required=True, type=Path)
    offset_parser.add_argument("--output", type=Path)
    seal_parser = sub.add_parser("seal")
    seal_parser.add_argument("--repo-root", required=True, type=Path)
    seal_parser.add_argument("--cargo-home", required=True)
    seal_parser.add_argument("--rust-sysroot", required=True)
    seal_parser.add_argument("--output-dir", required=True, type=Path)
    seal_parser.add_argument("--profile", required=True)
    seal_parser.add_argument("--target", required=True)
    seal_parser.add_argument("--rustc-version", required=True)
    seal_parser.add_argument("--cargo-version", required=True)
    seal_parser.add_argument("--wasm-tool-version", required=True)
    substitute_parser = sub.add_parser("substitute-projection")
    substitute_parser.add_argument("--projection", required=True, type=Path)
    substitute_parser.add_argument("--receipt-dir", required=True, type=Path)
    substitute_parser.add_argument("--candidate-dir", required=True, type=Path)
    args = parser.parse_args()

    if args.command == "contract":
        print(canonical_bytes(build_contract(args.repo_root, args.cargo_home, args.rust_sysroot)).decode(), end="")
        return
    if args.command == "inspect":
        roots = None
        if args.repo_root or args.cargo_home or args.rust_sysroot:
            if not (args.repo_root and args.cargo_home and args.rust_sysroot):
                fail("E_WASM_REMAP_REQUIRED_ROOT_MISSING", "all inventory roots required")
            roots = build_contract(args.repo_root, args.cargo_home, args.rust_sysroot)["physical_roots"]
            assert isinstance(roots, dict)
        report = inventory_target(args.target, None if roots is None else {str(k): str(v) for k, v in roots.items()})
        if args.require_zero and any(
            int(report[key]) != 0
            for key in ("internal_absolute_path_count", "private_include_count", "official_store_path_count", "unknown_path_count")
        ):
            fail("E_WASM_CANDIDATE_PATH_REMAINS", json.dumps({key: report[key] for key in report if key.endswith("count")}, sort_keys=True))
        data = canonical_bytes(report)
        if args.output:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_bytes(data)
        else:
            sys.stdout.buffer.write(data)
        return
    if args.command == "seal":
        contract = build_contract(args.repo_root, args.cargo_home, args.rust_sysroot)
        payload = semantic_receipt_payload(
            args.repo_root.resolve(), contract, args.output_dir.resolve(), args.profile, args.target,
            args.rustc_version, args.cargo_version, args.wasm_tool_version,
        )
        payload["semantic_identity"] = verify_semantic_identity(payload)
        receipt_path = args.output_dir / "R3N_WASM_REMAP_RECEIPT.detjson"
        receipt_path.write_bytes(canonical_bytes(payload))
        print(json.dumps({"receipt": str(receipt_path), "semantic_identity": payload["semantic_identity"]}, sort_keys=True))
        return
    if args.command == "prove-offsets":
        data = canonical_bytes(offset_bijection(args.original.resolve(), args.candidate.resolve()))
        if args.output:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_bytes(data)
        else:
            sys.stdout.buffer.write(data)
        return
    substitution = substitute_projection(args.projection.resolve(), args.receipt_dir.resolve(), args.candidate_dir.resolve())
    print(json.dumps(substitution, sort_keys=True))


if __name__ == "__main__":
    main()
