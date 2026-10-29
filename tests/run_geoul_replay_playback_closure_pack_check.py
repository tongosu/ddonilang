#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


PACK_SCHEMA = "ddn.geoul_replay_playback_closure.pack.contract.v1"
REQUIRED_CASES = {
    "c00_record_check": ["geoul", "record", "check", "pack/geoul_min_schema_v0/record_ok.jsonl"],
    "c01_replay_verify": ["replay", "verify", "--geoul", "tools/teul-cli/tests/golden/W35/W35_G01_replay_verify/out/geoul", "--entry", "pack/gogae4_w35_replay_harness/input.ddn"],
    "c02_story_make": ["story", "make", "--geoul", "tools/teul-cli/tests/golden/W41/W41_G01_timeline_ui/out/geoul", "--out", "pack/geoul_replay_playback_closure_v1/out/story.detjson"],
    "c03_timeline_make": ["timeline", "make", "--geoul", "tools/teul-cli/tests/golden/W41/W41_G01_timeline_ui/out/geoul", "--story", "tools/teul-cli/tests/golden/W41/W41_G01_timeline_ui/out/geoul/story/story.detjson", "--out", "pack/geoul_replay_playback_closure_v1/out/timeline.detjson"],
    "c04_key_query": ["geoul", "query", "--geoul", "tools/teul-cli/tests/golden/W43/W43_G01_geoul_query/out/geoul", "--madi", "3", "--key", "점수"],
    "c05_key_backtrace": ["geoul", "backtrace", "--geoul", "tools/teul-cli/tests/golden/W43/W43_G02_geoul_backtrace/out/geoul", "--key", "점수", "--from", "0", "--to", "5"],
}


def fail(code: str, msg: str) -> int:
    print(f"[geoul-replay-playback-closure-pack-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(path: Path) -> object:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise ValueError(f"missing file: {path}")
    except Exception as exc:
        raise ValueError(f"invalid json: {path} ({exc})")


def load_jsonl(path: Path) -> list[dict]:
    rows: list[dict] = []
    for line_no, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.strip()
        if not line:
            continue
        try:
            row = json.loads(line)
        except Exception as exc:
            raise ValueError(f"invalid jsonl row: {path}:{line_no} ({exc})")
        if not isinstance(row, dict):
            raise ValueError(f"jsonl row must be object: {path}:{line_no}")
        rows.append(row)
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description="Geoul replay/playback closure pack checker")
    parser.add_argument("--pack", default="pack/geoul_replay_playback_closure_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required = [pack / "README.md", pack / "intent.md", pack / "contract.detjson", pack / "golden.jsonl", pack / "tests" / "README.md"]
    required.extend((pack / "cases" / case_id / "expected.json") for case_id in REQUIRED_CASES)
    required.extend([pack / "expected" / "story.detjson", pack / "expected" / "timeline.detjson"])
    missing = [str(path).replace("\\", "/") for path in required if not path.exists()]
    if missing:
        return fail("E_GEOUL_REPLAY_PLAYBACK_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
        golden = load_jsonl(pack / "golden.jsonl")
    except ValueError as exc:
        return fail("E_GEOUL_REPLAY_PLAYBACK_INVALID", str(exc))

    if not isinstance(contract, dict):
        return fail("E_GEOUL_REPLAY_PLAYBACK_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_GEOUL_REPLAY_PLAYBACK_SCHEMA", f"schema={contract.get('schema')}")
    if str(contract.get("evidence_tier", "")).strip() != "golden_closed":
        return fail("E_GEOUL_REPLAY_PLAYBACK_TIER", f"tier={contract.get('evidence_tier')}")
    if str(contract.get("closure_claim", "")).strip() != "yes":
        return fail("E_GEOUL_REPLAY_PLAYBACK_CLOSURE", f"closure={contract.get('closure_claim')}")
    for rel in contract.get("evidence_targets", []):
        if not Path(rel).exists():
            return fail("E_GEOUL_REPLAY_PLAYBACK_TARGET", rel)

    case_index = {str(row.get("id", "")).strip(): row for row in contract.get("cases", []) if isinstance(row, dict)}
    if set(case_index) != set(REQUIRED_CASES):
        return fail("E_GEOUL_REPLAY_PLAYBACK_CASE_SET", f"cases={sorted(case_index)}")
    golden_index = {str(row.get("id", "")).strip(): row for row in golden}
    if set(golden_index) != set(REQUIRED_CASES):
        return fail("E_GEOUL_REPLAY_PLAYBACK_GOLDEN_SET", f"golden={sorted(golden_index)}")

    for case_id, expected_cmd in REQUIRED_CASES.items():
        expected = load_json(pack / "cases" / case_id / "expected.json")
        if not isinstance(expected, dict):
            return fail("E_GEOUL_REPLAY_PLAYBACK_EXPECTED", case_id)
        golden_row = golden_index[case_id]
        if golden_row.get("cmd") != expected_cmd:
            return fail("E_GEOUL_REPLAY_PLAYBACK_CMD", case_id)
        if golden_row.get("stdout") != expected.get("stdout"):
            return fail("E_GEOUL_REPLAY_PLAYBACK_STDOUT", case_id)
        if case_id in {"c02_story_make", "c03_timeline_make"}:
            if str(golden_row.get("expected_meta", "")).strip() != str(expected.get("expected_meta", "")).strip():
                return fail("E_GEOUL_REPLAY_PLAYBACK_META", case_id)
            if not str(golden_row.get("meta_out", "")).strip().startswith("out/"):
                return fail("E_GEOUL_REPLAY_PLAYBACK_META_OUT", case_id)
        if int(golden_row.get("exit_code", -1)) != 0:
            return fail("E_GEOUL_REPLAY_PLAYBACK_EXIT", case_id)

    print("[geoul-replay-playback-closure-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(golden_index)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
