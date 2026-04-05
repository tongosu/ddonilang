#!/usr/bin/env python
"""셈그림 플레이그라운드 스모크 검증.

node tests/seamgrim_playground_smoke_runner.mjs 를 실행하고
pack/seamgrim_playground_smoke_v1/expected/playground_smoke.detjson 과 비교한다.

사용:
  python tests/run_seamgrim_playground_smoke_check.py
  python tests/run_seamgrim_playground_smoke_check.py --update   # golden 갱신
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).parent.parent
PACK_DIR = ROOT / "pack" / "seamgrim_playground_smoke_v1"
RUNNER = ROOT / "tests" / "seamgrim_playground_smoke_runner.mjs"
EXPECTED_PATH = PACK_DIR / "expected" / "playground_smoke.detjson"


def canonical_json(data: object) -> str:
    return json.dumps(data, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def run_runner(update: bool = False) -> tuple[int, str, str]:
    cmd = ["node", "--no-warnings", str(RUNNER), str(PACK_DIR)]
    if update:
        cmd.append("--update")
    result = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return result.returncode, result.stdout.strip(), result.stderr.strip()


def main() -> int:
    parser = argparse.ArgumentParser(description="셈그림 플레이그라운드 스모크 검증")
    parser.add_argument("--update", action="store_true", help="golden 파일 갱신")
    args = parser.parse_args()

    if args.update:
        rc, stdout, stderr = run_runner(update=True)
        if rc != 0:
            print(f"[FAIL] runner --update 실패 (exit={rc})", file=sys.stderr)
            if stderr:
                print(stderr, file=sys.stderr)
            return 1
        print(stdout or f"[update] {EXPECTED_PATH}")
        return 0

    # 검증 모드
    rc, stdout, stderr = run_runner(update=False)

    if rc != 0:
        print(f"[FAIL] runner 실패 (exit={rc})", file=sys.stderr)
        if stderr:
            print(stderr, file=sys.stderr)
        if stdout:
            print(stdout, file=sys.stderr)
        return 1

    # runner stdout = { pack_id, outputs }
    try:
        payload = json.loads(stdout)
    except json.JSONDecodeError as exc:
        print(f"[FAIL] runner stdout JSON 파싱 실패: {exc}", file=sys.stderr)
        print(stdout[:500], file=sys.stderr)
        return 1

    actual = payload.get("outputs", {}).get("expected/playground_smoke.detjson", {})

    if not EXPECTED_PATH.exists():
        print(f"[FAIL] golden 파일 없음: {EXPECTED_PATH}", file=sys.stderr)
        print("  → python tests/run_seamgrim_playground_smoke_check.py --update 으로 생성하세요.")
        return 1

    expected = json.loads(EXPECTED_PATH.read_text(encoding="utf-8"))

    # all_ok 필드만 핵심 비교, 케이스별 상세는 경고만
    if not actual.get("all_ok", False):
        print("[FAIL] 플레이그라운드 스모크 케이스 실패", file=sys.stderr)
        for case in actual.get("cases", []):
            if not case.get("ok"):
                cid = case.get("id", "?")
                details = {k: v for k, v in case.items() if k not in ("id", "ok", "state_hash_final")}
                print(f"  [{cid}] {json.dumps(details, ensure_ascii=False)}", file=sys.stderr)
        return 1

    # golden과 all_ok / pass_count / case_count 일치 확인
    mismatches: list[str] = []
    for key in ("all_ok", "case_count", "pass_count"):
        if actual.get(key) != expected.get(key):
            mismatches.append(f"{key}: expected={expected.get(key)!r} actual={actual.get(key)!r}")

    # 케이스별 ok 필드 비교
    actual_by_id = {c["id"]: c for c in actual.get("cases", [])}
    expected_by_id = {c["id"]: c for c in expected.get("cases", [])}
    for cid, exp_case in expected_by_id.items():
        act_case = actual_by_id.get(cid, {})
        if act_case.get("ok") != exp_case.get("ok"):
            mismatches.append(f"case[{cid}].ok: expected={exp_case.get('ok')!r} actual={act_case.get('ok')!r}")
        if act_case.get("maegim_count") != exp_case.get("maegim_count"):
            mismatches.append(
                f"case[{cid}].maegim_count: expected={exp_case.get('maegim_count')!r} actual={act_case.get('maegim_count')!r}"
            )

    if mismatches:
        print("[FAIL] golden 불일치:", file=sys.stderr)
        for m in mismatches:
            print(f"  {m}", file=sys.stderr)
        return 1

    cases = actual.get("cases", [])
    print(f"[ok] 플레이그라운드 스모크 PASS ({actual.get('pass_count', 0)}/{actual.get('case_count', 0)})")
    for c in cases:
        cid = c.get("id", "?")
        ticks = c.get("ticks_run", 0)
        sliders = c.get("slider_count", 0)
        maegim = c.get("maegim_count", 0)
        det = "OK" if c.get("deterministic_ok") else "NG"
        print(f"  [{cid}] ticks={ticks} sliders={sliders} maegim={maegim} 결정론={det}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
