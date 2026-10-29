#!/usr/bin/env python
from __future__ import annotations

import json
import os
import re
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_projectile_representation_pair_v1"
LESSON = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_projectile_xy_v1"
PAIR = LESSON / "representation_pair.detjson"
DECLARATIVE = LESSON / "projectile_seum_s2.ddn"
SOLVEPLAN = LESSON / "projectile_seum_s2.solveplan.detjson"
EXPLICIT = LESSON / "lesson.ddn"
STATE_HASH_RE = re.compile(r"^state_hash=(blake3:[0-9a-f]{64})$", re.MULTILINE)


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
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def resolve_teul() -> Path | None:
    proc = run(["cargo", "metadata", "--no-deps", "--format-version", "1", "--manifest-path", "tools/teul-cli/Cargo.toml"], timeout=120)
    if proc.returncode != 0:
        return None
    payload = json.loads(proc.stdout)
    target = Path(str(payload.get("target_directory", "")))
    if not target.is_absolute():
        target = (ROOT / target).resolve()
    binary = target / "debug" / ("teul-cli.exe" if os.name == "nt" else "teul-cli")
    return binary if binary.exists() else None


def check_contract_and_pair() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        PAIR,
        DECLARATIVE,
        SOLVEPLAN,
        EXPLICIT,
        ROOT / "tests" / "seamgrim_projectile_representation_pair_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_REP_PAIR_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    for key in ("runtime_claim", "same_tickplan_claim", "same_observables_claim", "same_state_hash_claim", "declarative_runtime_claim", "docs_ssot_change"):
        if contract.get(key) is not False:
            return fail("E_REP_PAIR_CONTRACT_OVERCLAIM", f"{key}={contract.get(key)!r}")
    if contract.get("equivalence_class") != "teaching_pair":
        return fail("E_REP_PAIR_CONTRACT_CLASS", repr(contract.get("equivalence_class")))
    if contract.get("state_hash_integration") != "manifest_field_only_pending_tonni_confirmation":
        return fail("E_REP_PAIR_HASH_BOUNDARY", repr(contract.get("state_hash_integration")))

    pair = json.loads(read(PAIR))
    expected_false = ["same_tickplan_claim", "same_observables_claim", "same_state_hash_claim"]
    if pair.get("schema") != "ddn.seamgrim.representation_pair.v1" or pair.get("equivalence_class") != "teaching_pair":
        return fail("E_REP_PAIR_SCHEMA", repr(pair))
    if any(pair.get(key) is not False for key in expected_false):
        return fail("E_REP_PAIR_OVERCLAIM", repr({key: pair.get(key) for key in expected_false}))
    if pair.get("equivalence_status") != "not_asserted" or pair.get("right", {}).get("runtime_execution") != "not_landed":
        return fail("E_REP_PAIR_RUNTIME_BOUNDARY", repr(pair.get("right")))
    if pair.get("right", {}).get("product_entrypoint") != "teul-cli run --solveplan-out":
        return fail("E_REP_PAIR_ENTRYPOINT", repr(pair.get("right")))
    review = pair.get("review_boundary", {})
    if review != {
        "human_can_compare_meaning": True,
        "declarative_runtime_can_execute_world": False,
        "observable_equivalence_verified": False,
        "automatic_promotion_forbidden": True,
    }:
        return fail("E_REP_PAIR_REVIEW_BOUNDARY", repr(review))
    return 0


def check_solveplan(teul: Path) -> int:
    expected_bytes = SOLVEPLAN.read_bytes()
    generated: list[bytes] = []
    with tempfile.TemporaryDirectory(prefix="ddn_projectile_pair_") as tmp:
        for index in range(2):
            out = Path(tmp) / f"solveplan_{index}.detjson"
            proc = run([str(teul), "run", str(DECLARATIVE), "--solveplan-out", str(out)], timeout=180)
            if proc.returncode != 0 or not out.exists():
                return fail("E_REP_PAIR_SOLVEPLAN_RUN", proc.stdout.strip())
            generated.append(out.read_bytes())
    if generated[0] != generated[1] or generated[0] != expected_bytes:
        return fail("E_REP_PAIR_SOLVEPLAN_BYTES", "generated SolvePlan is not byte deterministic")

    artifact = json.loads(expected_bytes.decode("utf-8"))
    pair = json.loads(read(PAIR))
    summary = pair.get("solveplan_summary", {})
    if artifact.get("schema") != summary.get("schema") or artifact.get("stage") != summary.get("stage") or artifact.get("method") != summary.get("method"):
        return fail("E_REP_PAIR_SOLVEPLAN_SUMMARY", repr(summary))
    if len(artifact.get("relations", [])) != summary.get("relation_count") or len(artifact.get("lowering", {}).get("state_slots", [])) != summary.get("state_slot_count"):
        return fail("E_REP_PAIR_SOLVEPLAN_COUNTS", repr(summary))
    if artifact.get("diagnostics") != [] or summary.get("diagnostic_count") != 0:
        return fail("E_REP_PAIR_SOLVEPLAN_DIAGNOSTICS", repr(artifact.get("diagnostics")))
    return 0


def check_explicit_runtime(teul: Path) -> int:
    expected_card = json.loads(read(ROOT / "pack" / "onmungchi_10_worlds_pilot_v1" / "cards" / "physics_projectile_v1.detjson"))
    expected_hash = expected_card.get("execution_evidence", {}).get("state_hash")
    hashes: list[str] = []
    for _ in range(2):
        proc = run([str(teul), "run", str(EXPLICIT), "--madi", "240"], timeout=180)
        if proc.returncode != 0:
            return fail("E_REP_PAIR_EXPLICIT_RUN", proc.stdout.strip())
        match = STATE_HASH_RE.search(proc.stdout)
        if not match:
            return fail("E_REP_PAIR_EXPLICIT_HASH", proc.stdout.strip())
        hashes.append(match.group(1))
    if len(set(hashes)) != 1 or hashes[0] != expected_hash:
        return fail("E_REP_PAIR_EXPLICIT_DETERMINISM", f"{hashes} != {expected_hash}")
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    return fail(code, proc.stdout.strip()) if proc.returncode != 0 else 0


def main() -> int:
    rc = check_contract_and_pair()
    if rc:
        return rc
    teul = resolve_teul()
    if teul is None:
        return fail("E_REP_PAIR_TEUL", "teul-cli binary missing")
    for check in (check_solveplan, check_explicit_runtime):
        rc = check(teul)
        if rc:
            return rc
    gates = [
        (["node", "tests/seamgrim_projectile_representation_pair_runner.mjs"], "E_REP_PAIR_BROWSER", 150),
        (["python", "tests/run_pack_golden.py", "seamgrim_projectile_representation_pair_v1", "sim_ode_vector_v1"], "E_REP_PAIR_GOLDEN", 300),
        (["python", "tests/run_declarative_closure_g3_s2_check.py"], "E_REP_PAIR_G3", 600),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    docs = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if docs.returncode != 0 or docs.stdout.strip():
        return fail("E_REP_PAIR_SSOT_DIRTY", docs.stdout.strip())
    print("[seamgrim-projectile-representation-pair-v1] OK comparison-behavior-closed declarative-runtime-open")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
