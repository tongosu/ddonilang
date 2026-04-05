#!/usr/bin/env python
from __future__ import annotations

import re
import sys
from pathlib import Path


CHECKLIST_REL = "docs/context/proposals/PROPOSAL_IMJA_ALRIM_MAEGIM_SIGNAL_SSOT_SYNC_CHECKLIST_V1_20260308.md"
AGENTS_REL = "AGENTS.md"
SSOT_DIR_REL = "docs/ssot/ssot"

ALLOWED_PENDING_LINES: set[str] = set()

TARGET_TOKENS: dict[str, list[str]] = {
    "tools/teul-cli/src/lang/parser.rs": [
        "TokenKind::SignalArrow",
        "TokenKind::Ident(text) if text == \"받으면\"",
        "name == \"조건\" || name == \"매김\"",
        "name == \"분할수\"",
        "알림.이름",
    ],
    "tools/teul-cli/src/runtime/eval.rs": [
        "signal_send_dispatches_typed_and_generic_receive_hooks",
        "signal_send_dispatch_order_is_typed_conditional_then_typed_then_generic_conditional_then_generic",
        "normalize_temperature_literal",
        "temperature_literals_freezing_point_equivalence_across_units",
        "E_SELF_OUTSIDE_IMJA",
    ],
    "tools/teul-cli/src/runtime/error.rs": [
        "RuntimeError::JeOutsideImja",
        "\"E_SELF_OUTSIDE_IMJA\"",
    ],
    "tool/src/ddn_runtime.rs": [
        "temperature_literals_compare_after_kelvin_normalization",
        "template_format_can_render_temperature_in_celsius_and_fahrenheit",
        "77@F",
    ],
    "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py": [
        "MAEGIM_CONTROL_SCHEMA = \"ddn.maegim_control_plan.v1\"",
        "MAEGIM_CONTROL_LEGACY_RANGE_WARNING_CODE = \"W_LEGACY_RANGE_COMMENT_DEPRECATED\"",
        "\"warnings\": warnings",
        "\"maegim_control_warning_count\"",
        "\"maegim_control_warning_codes\"",
    ],
    "solutions/seamgrim_ui_mvp/ui/components/control_parser.js": [
        "\"ddn.maegim_control_plan.v1\"",
        "Array.isArray(parsed.warnings)",
        "W_LEGACY_RANGE_COMMENT_DEPRECATED",
    ],
}


def fail(code: str, msg: str) -> int:
    print(f"[ssot-sync-dr173-177-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def check_targets(repo_root: Path) -> tuple[bool, str]:
    for rel_path, tokens in TARGET_TOKENS.items():
        path = repo_root / rel_path
        if not path.exists():
            return False, f"E_SSOT_SYNC_DR173_177_FILE_MISSING::{rel_path}"
        text = path.read_text(encoding="utf-8")
        for token in tokens:
            if token not in text:
                return False, f"E_SSOT_SYNC_DR173_177_TOKEN_MISSING::{rel_path}::{token}"
    return True, f"targets={len(TARGET_TOKENS)}"


def check_pending_scope(repo_root: Path) -> tuple[bool, str]:
    path = repo_root / CHECKLIST_REL
    if not path.exists():
        return False, f"E_SSOT_SYNC_DR173_177_CHECKLIST_MISSING::{CHECKLIST_REL}"
    pending = [
        line.strip()
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.strip().startswith("- [ ]")
    ]
    drift = [line for line in pending if line not in ALLOWED_PENDING_LINES]
    if drift:
        return (
            False,
            f"E_SSOT_SYNC_DR173_177_PENDING_SCOPE_DRIFT::{'; '.join(drift[:3])}",
        )
    if len(pending) != len(ALLOWED_PENDING_LINES):
        return (
            False,
            f"E_SSOT_SYNC_DR173_177_PENDING_COUNT_MISMATCH::expected={len(ALLOWED_PENDING_LINES)}::actual={len(pending)}",
        )
    return True, f"pending_items={len(pending)}"


def parse_version_from_manifest_name(name: str) -> tuple[int, int, int] | None:
    match = re.fullmatch(r"SSOT_ALL_MANIFEST_v(\d+)\.(\d+)\.(\d+)\.md", name.strip())
    if not match:
        return None
    return tuple(int(part) for part in match.groups())


def resolve_current_ssot_version(repo_root: Path) -> tuple[bool, str]:
    ssot_dir = repo_root / SSOT_DIR_REL
    if not ssot_dir.exists():
        return False, f"E_SSOT_SYNC_VERSION_SSOT_DIR_MISSING::{SSOT_DIR_REL}"

    manifests = []
    for path in ssot_dir.glob("SSOT_ALL_MANIFEST_v*.md"):
        parsed = parse_version_from_manifest_name(path.name)
        if parsed is not None:
            manifests.append((parsed, path))
    if not manifests:
        return False, "E_SSOT_SYNC_VERSION_MANIFEST_MISSING::SSOT_ALL_MANIFEST_v*.md"

    manifests.sort(key=lambda item: item[0], reverse=True)
    latest_version, latest_manifest_path = manifests[0]
    version_text = f"v{latest_version[0]}.{latest_version[1]}.{latest_version[2]}"
    manifest_text = latest_manifest_path.read_text(encoding="utf-8")
    if f"- version: {version_text}" not in manifest_text:
        return (
            False,
            "E_SSOT_SYNC_VERSION_MANIFEST_DECL_MISMATCH::"
            f"{latest_manifest_path.relative_to(repo_root)}::expected={version_text}",
        )

    return True, version_text


def check_ssot_version_alignment(repo_root: Path) -> tuple[bool, str]:
    ok, detail = resolve_current_ssot_version(repo_root)
    if not ok:
        return False, detail
    version_text = detail

    ssot_dir = repo_root / SSOT_DIR_REL
    expected_index = ssot_dir / f"SSOT_INDEX_{version_text}.md"
    expected_roadmap = ssot_dir / f"SSOT_ROADMAP_CATALOG_{version_text}.md"
    for expected in (expected_index, expected_roadmap):
        if not expected.exists():
            return (
                False,
                "E_SSOT_SYNC_VERSION_FILE_MISSING::"
                f"{expected.relative_to(repo_root)}",
            )

    agents_path = repo_root / AGENTS_REL
    if not agents_path.exists():
        return False, f"E_SSOT_SYNC_VERSION_AGENTS_MISSING::{AGENTS_REL}"
    agents_text = agents_path.read_text(encoding="utf-8")

    required_tokens = [
        f"docs/ssot/ssot/SSOT_INDEX_{version_text}.md",
        f"현재 SSOT 버전: **{version_text}**",
        f"docs/ssot/ssot/SSOT_ROADMAP_CATALOG_{version_text}.md",
        f"(파일명: `*_{version_text}.md`)",
    ]
    for token in required_tokens:
        if token not in agents_text:
            return False, f"E_SSOT_SYNC_VERSION_AGENTS_TOKEN_MISSING::{token}"

    roadmap_tokens = re.findall(r"SSOT_ROADMAP_CATALOG_v\d+\.\d+\.\d+\.md", agents_text)
    drift_tokens = sorted(set(token for token in roadmap_tokens if token != f"SSOT_ROADMAP_CATALOG_{version_text}.md"))
    if drift_tokens:
        return (
            False,
            "E_SSOT_SYNC_VERSION_AGENTS_ROADMAP_DRIFT::"
            + ",".join(drift_tokens),
        )

    return True, f"ssot_version={version_text}"


def main() -> int:
    repo_root = Path(__file__).resolve().parent.parent

    ok, detail = check_ssot_version_alignment(repo_root)
    if not ok:
        code, msg = detail.split("::", 1) if "::" in detail else ("E_SSOT_SYNC_VERSION_INVALID", detail)
        return fail(code, msg)

    ok, detail = check_targets(repo_root)
    if not ok:
        code, msg = detail.split("::", 1) if "::" in detail else ("E_SSOT_SYNC_DR173_177_INVALID", detail)
        return fail(code, msg)

    ok, detail = check_pending_scope(repo_root)
    if not ok:
        code, msg = detail.split("::", 1) if "::" in detail else ("E_SSOT_SYNC_DR173_177_INVALID", detail)
        return fail(code, msg)

    print(f"[ssot-sync-dr173-177-check] ok {detail}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
