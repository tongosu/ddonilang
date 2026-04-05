#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path


VALID_TIERS = {"golden_closed", "runner_fill", "docs_first"}
VALID_STATUS_VALUES = {
    "active",
    "partial",
    "designed",
    "reserved",
    "implemented",
    "planned",
    "deprecated",
}
VALID_CLOSURE_CLAIM_VALUES = {"allowed", "blocked"}
DOCS_SSOT_REP10_PACKS = [
    "bogae_observe_basics",
    "bogae_view_headless",
    "bogae_web_out_determinism",
    "bogae_grid2d_smoke_v1",
    "bogae_backend_parity_console_web_v1",
    "econ_agent_baseline_market_v1",
    "econ_agent_lemons_v1",
    "econ_agent_macro_shock_v1",
    "nuri_gym_bandit_v1",
    "nuri_env_v0_determinism",
]
REPO_REP10_PACKS = [
    "bogae_observe_basics",
    "bogae_web_out_determinism",
    "bogae_grid2d_smoke_v1",
    "bogae_backend_parity_console_web_v1",
    "bogae_api_catalog_v1_basic",
    "bogae_web_viewer_v1",
    "nuri_gym_canon_contract_v1",
    "nuri_gym_cartpole",
    "nuri_gym_gridworld_v1",
    "eco_macro_micro_runner_smoke",
]
PROFILE_TO_PACKS = {
    "docs_ssot_rep10": DOCS_SSOT_REP10_PACKS,
    "repo_rep10": REPO_REP10_PACKS,
}
PROFILE_TO_PACK_ROOT = {
    "docs_ssot_rep10": "docs/ssot/pack",
    "repo_rep10": "pack",
}
TIER_RE = re.compile(r"(?mi)^\s*evidence_tier\s*:\s*([a-z_]+)\s*$")
STATUS_RE = re.compile(r"(?mi)^\s*(?:current_status|status)\s*:\s*([a-z_]+)\s*$")
CLOSURE_CLAIM_RE = re.compile(
    r"(?mi)^\s*(?:closure_claim|closure_claim_possible|closure)\s*:\s*([a-z_]+)\s*$"
)
SUGGESTION_HINTS = {
    "bogae_observe_basics": ("golden_closed", "drawlist/hash 결정성 골든 팩"),
    "bogae_view_headless": ("runner_fill", "headless 경로 smoke 성격"),
    "bogae_web_out_determinism": ("golden_closed", "web out 결정성 고정"),
    "bogae_grid2d_smoke_v1": ("runner_fill", "smoke/skeleton 표기"),
    "bogae_backend_parity_console_web_v1": ("runner_fill", "backend parity skeleton 표기"),
    "bogae_api_catalog_v1_basic": ("golden_closed", "API 카탈로그 기본 결정성"),
    "bogae_web_viewer_v1": ("golden_closed", "viewer 스모크+golden 보유"),
    "econ_agent_baseline_market_v1": ("docs_first", "경제 교재/설계 정렬 우선"),
    "econ_agent_lemons_v1": ("docs_first", "경제 교재/설계 정렬 우선"),
    "econ_agent_macro_shock_v1": ("docs_first", "경제 교재/설계 정렬 우선"),
    "nuri_gym_bandit_v1": ("docs_first", "누리Gym 설계/예시 성격"),
    "nuri_env_v0_determinism": ("docs_first", "환경 경계 설계/스키마 성격"),
    "nuri_gym_canon_contract_v1": ("golden_closed", "canon 계약 고정"),
    "nuri_gym_cartpole": ("docs_first", "min spec/샘플 중심"),
    "nuri_gym_gridworld_v1": ("golden_closed", "episode/dataset 골든 고정"),
    "eco_macro_micro_runner_smoke": ("runner_fill", "runner smoke 성격"),
}
SKELETON_RE = re.compile(r"(?i)\b(skeleton|smoke|todo_by_runner|runner)\b|스켈레톤|스모크")
DETERMINISM_RE = re.compile(r"(?i)\b(golden|hash|determinism|detjson)\b|결정성|재현")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check representative docs/ssot/pack README evidence_tier coverage"
    )
    parser.add_argument("--repo-root", default=".", help="Repository root path")
    parser.add_argument(
        "--pack-root",
        default="",
        help="Pack root relative to repo-root (empty => profile default)",
    )
    parser.add_argument(
        "--pack",
        action="append",
        dest="packs",
        default=[],
        help="Representative pack id (repeatable). Defaults to built-in list.",
    )
    parser.add_argument(
        "--profile",
        choices=sorted(PROFILE_TO_PACKS.keys()),
        default="docs_ssot_rep10",
        help="Built-in representative pack profile",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Fail when README/tier is missing or invalid",
    )
    parser.add_argument(
        "--report",
        default="",
        help="Optional output path for detjson report (relative to repo-root or absolute)",
    )
    parser.add_argument(
        "--fix-plan",
        default="",
        help="Optional output path for markdown fix plan (relative to repo-root or absolute)",
    )
    parser.add_argument(
        "--require-status",
        action="store_true",
        help="Require current_status/status line in README",
    )
    parser.add_argument(
        "--require-closure-claim",
        action="store_true",
        help="Require closure_claim/closure line in README",
    )
    parser.add_argument(
        "--enforce-closure-tier",
        action="store_true",
        help="Fail when closure_claim=allowed but evidence_tier is not golden_closed",
    )
    return parser.parse_args()


def resolve_report_path(repo_root: Path, report_arg: str) -> Path | None:
    if not report_arg:
        return None
    path = Path(report_arg)
    if not path.is_absolute():
        path = repo_root / path
    return path


def suggest_tier(pack: str, text: str | None) -> tuple[str, str]:
    if pack in SUGGESTION_HINTS:
        return SUGGESTION_HINTS[pack]
    if text is None:
        return "docs_first", "README 부재: 수동 분류 필요"
    if SKELETON_RE.search(text):
        return "runner_fill", "README에 skeleton/smoke/runner 힌트"
    if DETERMINISM_RE.search(text):
        return "golden_closed", "README에 golden/hash/determinism 힌트"
    return "docs_first", "명시 힌트 없음: docs-first 기본"


def normalize_closure_claim(value: str) -> str:
    norm = value.strip().lower()
    if norm in {"allowed", "allow", "yes", "true", "possible", "can"}:
        return "allowed"
    if norm in {"blocked", "deny", "denied", "no", "false", "impossible", "cannot"}:
        return "blocked"
    return norm


def write_fix_plan(path: Path, profile: str, pack_root_rel: str, suggestions: list[dict[str, str]]) -> None:
    lines = [
        "# PACK evidence_tier Fix Plan",
        "",
        f"- profile: `{profile}`",
        f"- pack_root: `{pack_root_rel}`",
        f"- missing_or_invalid_count: `{len(suggestions)}`",
        "",
        "아래 항목을 각 `README.md` 상단(제목 바로 아래)에 추가:",
        "",
    ]
    for item in suggestions:
        lines.extend(
            [
                f"- `{item['pack']}`",
                f"  - suggestion: `evidence_tier: {item['suggested_tier']}`",
                f"  - reason: {item['reason']}",
            ]
        )
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).resolve()
    pack_root_rel = args.pack_root if args.pack_root else PROFILE_TO_PACK_ROOT[args.profile]
    pack_root = (repo_root / pack_root_rel).resolve()
    packs = args.packs if args.packs else list(PROFILE_TO_PACKS[args.profile])

    missing_readme: list[str] = []
    missing_tier: list[str] = []
    invalid_tier: list[dict[str, str]] = []
    missing_status: list[str] = []
    invalid_status: list[dict[str, str]] = []
    missing_closure_claim: list[str] = []
    invalid_closure_claim: list[dict[str, str]] = []
    closure_tier_violation: list[dict[str, str]] = []
    ok_items: list[dict[str, str]] = []
    suggestions: list[dict[str, str]] = []

    for pack in packs:
        readme_path = pack_root / pack / "README.md"
        if not readme_path.exists():
            missing_readme.append(pack)
            suggested_tier, reason = suggest_tier(pack, None)
            suggestions.append(
                {
                    "pack": pack,
                    "status": "missing_readme",
                    "suggested_tier": suggested_tier,
                    "reason": reason,
                }
            )
            continue
        text = readme_path.read_text(encoding="utf-8")
        match = TIER_RE.search(text)
        if not match:
            missing_tier.append(pack)
            suggested_tier, reason = suggest_tier(pack, text)
            suggestions.append(
                {
                    "pack": pack,
                    "status": "missing_tier",
                    "suggested_tier": suggested_tier,
                    "reason": reason,
                }
            )
            continue
        tier = match.group(1)
        if tier not in VALID_TIERS:
            invalid_tier.append({"pack": pack, "value": tier})
            suggested_tier, reason = suggest_tier(pack, text)
            suggestions.append(
                {
                    "pack": pack,
                    "status": "invalid_tier",
                    "current_tier": tier,
                    "suggested_tier": suggested_tier,
                    "reason": reason,
                }
            )
            continue

        status_match = STATUS_RE.search(text)
        status_value = ""
        if status_match:
            status_value = status_match.group(1)
            if status_value not in VALID_STATUS_VALUES:
                invalid_status.append({"pack": pack, "value": status_value})
                suggestions.append(
                    {
                        "pack": pack,
                        "status": "invalid_status",
                        "current_status": status_value,
                        "suggested_status": "active",
                        "reason": "status/current_status는 운영 표준 값으로 유지",
                    }
                )
                continue
        elif args.require_status:
            missing_status.append(pack)
            suggestions.append(
                {
                    "pack": pack,
                    "status": "missing_status",
                    "suggested_status": "active",
                    "reason": "README 상단에 current_status 또는 status 표기를 권장",
                }
            )
            continue

        closure_match = CLOSURE_CLAIM_RE.search(text)
        closure_value = ""
        if closure_match:
            closure_value = normalize_closure_claim(closure_match.group(1))
            if closure_value not in VALID_CLOSURE_CLAIM_VALUES:
                invalid_closure_claim.append({"pack": pack, "value": closure_match.group(1)})
                suggestions.append(
                    {
                        "pack": pack,
                        "status": "invalid_closure_claim",
                        "current_closure_claim": closure_match.group(1),
                        "suggested_closure_claim": "blocked",
                        "reason": "closure_claim은 allowed|blocked 값만 사용",
                    }
                )
                continue
        elif args.require_closure_claim:
            missing_closure_claim.append(pack)
            suggestions.append(
                {
                    "pack": pack,
                    "status": "missing_closure_claim",
                    "suggested_closure_claim": "blocked",
                    "reason": "README 상단에 closure_claim 표기를 권장",
                }
            )
            continue

        if args.enforce_closure_tier and closure_value == "allowed" and tier != "golden_closed":
            closure_tier_violation.append({"pack": pack, "tier": tier, "closure_claim": closure_value})
            suggestions.append(
                {
                    "pack": pack,
                    "status": "closure_tier_violation",
                    "current_tier": tier,
                    "current_closure_claim": closure_value,
                    "suggested_closure_claim": "blocked",
                    "reason": "closure claim 근거는 golden_closed tier에 한정",
                }
            )
            continue
        ok_item = {"pack": pack, "tier": tier}
        if status_value:
            ok_item["status"] = status_value
        if closure_value:
            ok_item["closure_claim"] = closure_value
        ok_items.append(ok_item)

    issue_count = (
        len(missing_readme)
        + len(missing_tier)
        + len(invalid_tier)
        + len(missing_status)
        + len(invalid_status)
        + len(missing_closure_claim)
        + len(invalid_closure_claim)
        + len(closure_tier_violation)
    )
    report = {
        "schema": "ddn.pack_evidence_tier_check.v1",
        "strict": bool(args.strict),
        "require_status": bool(args.require_status),
        "require_closure_claim": bool(args.require_closure_claim),
        "enforce_closure_tier": bool(args.enforce_closure_tier),
        "repo_root": repo_root.as_posix(),
        "pack_root": pack_root.as_posix(),
        "pack_root_rel": pack_root_rel,
        "profile": args.profile,
        "total": len(packs),
        "ok_count": len(ok_items),
        "issue_count": issue_count,
        "missing_readme": missing_readme,
        "missing_tier": missing_tier,
        "invalid_tier": invalid_tier,
        "missing_status": missing_status,
        "invalid_status": invalid_status,
        "missing_closure_claim": missing_closure_claim,
        "invalid_closure_claim": invalid_closure_claim,
        "closure_tier_violation": closure_tier_violation,
        "suggested_fixes": suggestions,
        "ok_items": ok_items,
    }

    report_path = resolve_report_path(repo_root, args.report)
    if report_path is not None:
        report_path.parent.mkdir(parents=True, exist_ok=True)
        report_path.write_text(
            json.dumps(report, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )

    fix_plan_path = resolve_report_path(repo_root, args.fix_plan)
    if fix_plan_path is not None:
        write_fix_plan(fix_plan_path, args.profile, pack_root_rel, suggestions)

    detail = (
        f"total={report['total']} ok={report['ok_count']} issues={report['issue_count']} "
        f"missing_readme={len(missing_readme)} missing_tier={len(missing_tier)} "
        f"invalid_tier={len(invalid_tier)} missing_status={len(missing_status)} "
        f"invalid_status={len(invalid_status)} missing_closure_claim={len(missing_closure_claim)} "
        f"invalid_closure_claim={len(invalid_closure_claim)} "
        f"closure_tier_violation={len(closure_tier_violation)}"
    )
    print(f"check=pack_evidence_tier detail={detail}")

    if args.strict and issue_count > 0:
        print("check=pack_evidence_tier detail=strict_failed", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
