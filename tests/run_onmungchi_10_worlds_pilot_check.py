#!/usr/bin/env python
from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
import json
import os
import re
import subprocess
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "onmungchi_10_worlds_pilot_v1"
TEUL_MANIFEST = ROOT / "tools" / "teul-cli" / "Cargo.toml"
STATE_HASH_RE = re.compile(r"^state_hash=(blake3:[0-9a-f]{64})$", re.MULTILINE)
TRACE_HASH_RE = re.compile(r"^trace_hash=(blake3:[0-9a-f]{64})$", re.MULTILINE)


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(cmd: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
    )


def resolve_teul_binary() -> Path | None:
    proc = run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1", "--manifest-path", str(TEUL_MANIFEST)],
        timeout=120,
    )
    if proc.returncode != 0:
        return None
    try:
        payload = json.loads(proc.stdout)
    except json.JSONDecodeError:
        return None
    target = Path(str(payload.get("target_directory", "")))
    if not target.is_absolute():
        target = (ROOT / target).resolve()
    exe = target / "debug" / ("teul-cli.exe" if os.name == "nt" else "teul-cli")
    return exe if exe.exists() else None


def ensure_teul_binary() -> Path:
    binary = resolve_teul_binary()
    if binary is not None:
        return binary
    proc = run(["cargo", "build", "--quiet", "--manifest-path", str(TEUL_MANIFEST)], timeout=600)
    if proc.returncode != 0:
        raise RuntimeError(f"teul build failed: {(proc.stdout + proc.stderr).strip()}")
    binary = resolve_teul_binary()
    if binary is None:
        raise RuntimeError("teul binary missing after build")
    return binary


def normalize_ddn(text: str) -> str:
    return text.replace("\r\n", "\n").rstrip() + "\n"


def parse_anchor(anchor: str) -> tuple[Path, int] | None:
    match = re.fullmatch(r"(.+\.md):(\d+)", str(anchor).strip())
    if not match:
        return None
    path = ROOT / match.group(1)
    return path, int(match.group(2))


def check_static_contract() -> tuple[list[dict[str, object]], str | None]:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "pilot.detjson",
        PACK / "corpus_card.schema.json",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return [], f"missing:{missing}"

    contract = json.loads(read(PACK / "contract.detjson"))
    expected_contract = {
        "pack": "onmungchi_10_worlds_pilot_v1",
        "world_count": 10,
        "feasibility_grade": "ga",
        "machine_gate_count": 4,
        "human_gate_count": 1,
        "owner_decision_required": True,
        "corpus_inclusion_claim": False,
        "runtime_claim": False,
        "product_code_change": False,
        "docs_ssot_change": False,
    }
    for key, value in expected_contract.items():
        if contract.get(key) != value:
            return [], f"contract:{key}={contract.get(key)!r}"

    pilot = json.loads(read(PACK / "pilot.detjson"))
    if pilot.get("world_count") != 10 or pilot.get("included_count") != 0:
        return [], f"pilot_counts:{pilot}"
    if pilot.get("verified_pending_human_count") != 10 or pilot.get("corpus_inclusion_claim") is not False:
        return [], f"pilot_boundary:{pilot}"
    if pilot.get("machine_gate_names") != ["pack_pass", "canonical_roundtrip", "remix_handles", "few_shot_pair"]:
        return [], f"machine_gates:{pilot.get('machine_gate_names')}"
    if pilot.get("human_gate_names") != ["read_aloud"]:
        return [], f"human_gates:{pilot.get('human_gate_names')}"

    card_paths = pilot.get("cards")
    if not isinstance(card_paths, list) or len(card_paths) != 10 or len(set(card_paths)) != 10:
        return [], f"card_paths:{card_paths}"
    cards: list[dict[str, object]] = []
    for relative in card_paths:
        card_path = PACK / str(relative)
        if not card_path.exists():
            return [], f"card_missing:{relative}"
        cards.append(json.loads(read(card_path)))
    return cards, None


def check_card_shape(card: dict[str, object], index_rows: dict[str, dict[str, object]]) -> str | None:
    required = {
        "schema",
        "card_id",
        "world_concept_id",
        "name",
        "one_line",
        "lifecycle",
        "feasibility",
        "curriculum",
        "source",
        "surfaces",
        "execution_evidence",
        "few_shot",
        "remix_handles",
        "read_aloud",
        "lineage",
        "gate_status",
        "owner_decision",
    }
    missing = sorted(required - set(card))
    card_id = str(card.get("card_id", ""))
    if missing:
        return f"{card_id}:missing:{missing}"
    if card.get("schema") != "ddn.corpus_card.v1.candidate" or card.get("lifecycle") != "검증중":
        return f"{card_id}:schema_or_lifecycle"

    feasibility = card.get("feasibility")
    if not isinstance(feasibility, dict) or feasibility.get("grade") != "ga":
        return f"{card_id}:grade"
    anchor = parse_anchor(str(feasibility.get("ssot_anchor", "")))
    if anchor is None:
        return f"{card_id}:anchor_format"
    anchor_path, line_number = anchor
    if not anchor_path.exists():
        return f"{card_id}:anchor_missing"
    lines = read(anchor_path).splitlines()
    if line_number < 1 or line_number > len(lines) or "가등급" not in lines[line_number - 1]:
        return f"{card_id}:anchor_line"

    source = card.get("source")
    if not isinstance(source, dict):
        return f"{card_id}:source"
    lesson_id = str(source.get("lesson_id", ""))
    source_path = ROOT / str(source.get("path", ""))
    if not source_path.exists() or source_path != ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / lesson_id / "lesson.ddn":
        return f"{card_id}:source_path"
    index_row = index_rows.get(lesson_id)
    if not index_row or index_row.get("source") != "representative_v1":
        return f"{card_id}:representative_index"
    existing_pack_id = source.get("existing_pack_id")
    if existing_pack_id is not None:
        pack_lesson = ROOT / "pack" / str(existing_pack_id) / "lesson.ddn"
        if not pack_lesson.exists() or read(pack_lesson) != read(source_path):
            return f"{card_id}:existing_pack_sync"
    madi = source.get("representative_madi")
    if not isinstance(madi, int) or madi <= 0:
        return f"{card_id}:representative_madi"

    few_shot = card.get("few_shot")
    if not isinstance(few_shot, list) or len(few_shot) != 2:
        return f"{card_id}:few_shot_count"
    audiences = {str(row.get("audience", "")) for row in few_shot if isinstance(row, dict)}
    if audiences != {"child", "teacher"}:
        return f"{card_id}:few_shot_audiences:{audiences}"
    if any(not str(row.get("intent", "")).strip() for row in few_shot if isinstance(row, dict)):
        return f"{card_id}:few_shot_intent"

    handles = card.get("remix_handles")
    if not isinstance(handles, list) or not handles:
        return f"{card_id}:remix_handles"
    for handle in handles:
        if not isinstance(handle, dict) or not all(str(handle.get(key, "")).strip() for key in ("parameter", "range", "effect")):
            return f"{card_id}:remix_handle_shape"

    gate_status = card.get("gate_status")
    expected_gates = {
        "pack_pass": "passed",
        "canonical_roundtrip": "passed",
        "read_aloud": "pending_human_review",
        "remix_handles": "passed",
        "few_shot_pair": "structurally_passed",
    }
    if gate_status != expected_gates:
        return f"{card_id}:gate_status:{gate_status}"
    read_aloud = card.get("read_aloud")
    if not isinstance(read_aloud, dict) or read_aloud.get("status") != "pending_human_review" or read_aloud.get("reviewer") is not None:
        return f"{card_id}:read_aloud_boundary"
    owner = card.get("owner_decision")
    if not isinstance(owner, dict) or owner.get("status") != "pending" or owner.get("decision_ref") is not None:
        return f"{card_id}:owner_boundary"

    evidence = card.get("execution_evidence")
    if not isinstance(evidence, dict):
        return f"{card_id}:execution_evidence"
    if not STATE_HASH_RE.fullmatch(f"state_hash={evidence.get('state_hash', '')}"):
        return f"{card_id}:state_hash_format"
    if not TRACE_HASH_RE.fullmatch(f"trace_hash={evidence.get('trace_hash', '')}"):
        return f"{card_id}:trace_hash_format"
    return None


def run_card_runtime_check(card: dict[str, object], teul: Path) -> tuple[str, str | None]:
    card_id = str(card["card_id"])
    source = card["source"]
    evidence = card["execution_evidence"]
    assert isinstance(source, dict)
    assert isinstance(evidence, dict)
    source_path = str((ROOT / str(source["path"])).as_posix())
    madi = str(source["representative_madi"])

    checked = run([str(teul), "check", source_path], timeout=120)
    if checked.returncode != 0:
        return card_id, f"check:{(checked.stdout + checked.stderr).strip()}"

    canon1 = run([str(teul), "canon", source_path, "--emit", "ddn"], timeout=120)
    if canon1.returncode != 0 or "W_CANON_" in canon1.stderr:
        return card_id, f"canon1:{(canon1.stdout + canon1.stderr).strip()}"
    canonical = normalize_ddn(canon1.stdout)
    with tempfile.TemporaryDirectory(prefix=f"ddn_onmungchi_{card_id}_") as tmp:
        canon_path = Path(tmp) / "canonical.ddn"
        canon_path.write_text(canonical, encoding="utf-8", newline="\n")
        canon2 = run([str(teul), "canon", str(canon_path), "--emit", "ddn"], timeout=120)
    if canon2.returncode != 0 or "W_CANON_" in canon2.stderr:
        return card_id, f"canon2:{(canon2.stdout + canon2.stderr).strip()}"
    if normalize_ddn(canon2.stdout) != canonical:
        return card_id, "canonical_roundtrip_mismatch"

    outputs: list[str] = []
    for _ in range(2):
        executed = run([str(teul), "run", source_path, "--madi", madi], timeout=180)
        if executed.returncode != 0:
            return card_id, f"run:{(executed.stdout + executed.stderr).strip()}"
        outputs.append(executed.stdout)
    state_hashes = [STATE_HASH_RE.search(output) for output in outputs]
    trace_hashes = [TRACE_HASH_RE.search(output) for output in outputs]
    if any(match is None for match in state_hashes) or any(match is None for match in trace_hashes):
        return card_id, "execution_hash_missing"
    actual_state = [match.group(1) for match in state_hashes if match]
    actual_trace = [match.group(1) for match in trace_hashes if match]
    if len(set(actual_state)) != 1 or actual_state[0] != evidence.get("state_hash"):
        return card_id, f"state_hash:{actual_state}:{evidence.get('state_hash')}"
    if len(set(actual_trace)) != 1 or actual_trace[0] != evidence.get("trace_hash"):
        return card_id, f"trace_hash:{actual_trace}:{evidence.get('trace_hash')}"
    return card_id, None


def check_pack_golden() -> str | None:
    proc = run(["python", "tests/run_pack_golden.py", "onmungchi_10_worlds_pilot_v1"], timeout=300)
    return None if proc.returncode == 0 else (proc.stdout + proc.stderr).strip()


def check_docs_ssot_clean() -> str | None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return (proc.stdout + proc.stderr).strip()
    return proc.stdout.strip() or None


def main() -> int:
    cards, static_error = check_static_contract()
    if static_error:
        return fail("E_ONMUNGCHI_STATIC", static_error)

    index_payload = json.loads(read(ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "index.json"))
    index_rows = {
        str(row.get("id")): row
        for row in index_payload.get("lessons", [])
        if isinstance(row, dict) and row.get("id")
    }
    card_ids: list[str] = []
    concept_ids: list[str] = []
    lesson_ids: list[str] = []
    for card in cards:
        error = check_card_shape(card, index_rows)
        if error:
            return fail("E_ONMUNGCHI_CARD", error)
        card_ids.append(str(card["card_id"]))
        concept_ids.append(str(card["world_concept_id"]))
        source = card["source"]
        assert isinstance(source, dict)
        lesson_ids.append(str(source["lesson_id"]))
    if len(set(card_ids)) != 10 or len(set(concept_ids)) != 10 or len(set(lesson_ids)) != 10:
        return fail("E_ONMUNGCHI_UNIQUE", f"cards={card_ids} concepts={concept_ids} lessons={lesson_ids}")

    try:
        teul = ensure_teul_binary()
    except RuntimeError as error:
        return fail("E_ONMUNGCHI_TEUL", str(error))
    workers = max(1, min(10, os.cpu_count() or 4))
    with ThreadPoolExecutor(max_workers=workers) as pool:
        results = list(pool.map(lambda card: run_card_runtime_check(card, teul), cards))
    failures = [f"{card_id}:{error}" for card_id, error in results if error]
    if failures:
        return fail("E_ONMUNGCHI_RUNTIME", " | ".join(failures))

    golden_error = check_pack_golden()
    if golden_error:
        return fail("E_ONMUNGCHI_GOLDEN", golden_error)
    ssot_error = check_docs_ssot_clean()
    if ssot_error:
        return fail("E_ONMUNGCHI_SSOT_DIRTY", ssot_error)

    print(
        "[onmungchi-10-worlds-pilot-v1] OK "
        "cards=10 machine_gates=40 human_readaloud_pending=10 owner_pending=10 included=0"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
