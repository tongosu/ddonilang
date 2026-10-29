#!/usr/bin/env python
from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "sim_agent_rng_stream_v1"
DERIVATION = "(global_seed, agent_id, stream_name)"


def fail(message: str) -> None:
    print(f"declarative_closure_g10_agent_rng_stream_red_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def run(cmd: list[str], *, timeout: int = 240) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
        check=False,
    )


def read_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8-sig"))
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON {path.relative_to(ROOT)}: {exc}")


def derive(global_seed: int, agent_id: str, stream_name: str, draw_count: int) -> list[str]:
    values: list[str] = []
    for draw_index in range(draw_count):
        payload = f"{global_seed}\0{agent_id}\0{stream_name}\0{draw_index}".encode("utf-8")
        values.append(hashlib.sha256(payload).hexdigest()[:16])
    return values


def keyed_streams(rows: list[dict]) -> dict[tuple[str, str], dict]:
    out: dict[tuple[str, str], dict] = {}
    for row in rows:
        key = (row.get("agent_id"), row.get("stream_name"))
        if key in out:
            fail(f"duplicate stream key: {key!r}")
        out[key] = row
    return out


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "sim_agent_rng_stream_v1"])
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_contract() -> None:
    contract = read_json(PACK / "contract.detjson")
    if contract.get("derivation") != DERIVATION:
        fail(f"derivation mismatch: {contract.get('derivation')!r}")
    for field in ("runtime_claim", "product_code_change", "runtime_rng_change", "state_hash_formula_change"):
        if contract.get(field) is not False:
            fail(f"{field} must be false")


def check_stream_fixtures() -> None:
    expected = read_json(PACK / "expected_streams.detjson")
    if expected.get("derivation") != DERIVATION:
        fail("expected_streams derivation mismatch")
    expected_streams = keyed_streams(expected.get("streams", []))
    for key, row in expected_streams.items():
        computed = derive(expected["global_seed"], key[0], key[1], row["draw_count"])
        if computed != row.get("draws_hex16"):
            fail(f"expected_streams computed draws mismatch for {key!r}")

    fixtures = [
        read_json(PACK / "streams_order_a.detjson"),
        read_json(PACK / "streams_order_b.detjson"),
    ]
    traversal_orders = []
    for fixture in fixtures:
        if fixture.get("derivation") != DERIVATION:
            fail(f"{fixture.get('fixture_id')} derivation mismatch")
        if fixture.get("global_seed") != expected.get("global_seed"):
            fail(f"{fixture.get('fixture_id')} global_seed mismatch")
        traversal = [(row.get("agent_id"), row.get("stream_name")) for row in fixture.get("traversal_order", [])]
        traversal_orders.append(traversal)
        if set(traversal) != set(expected_streams):
            fail(f"{fixture.get('fixture_id')} stream set mismatch")
        got = keyed_streams(fixture.get("draws_by_stream", []))
        if set(got) != set(expected_streams):
            fail(f"{fixture.get('fixture_id')} draws stream set mismatch")
        for key, row in got.items():
            expected_row = expected_streams[key]
            if row.get("draws_hex16") != expected_row.get("draws_hex16"):
                fail(f"{fixture.get('fixture_id')} draws mismatch for {key!r}")
    if traversal_orders[0] == traversal_orders[1]:
        fail("order A/B traversal order must differ")


def check_no_docs_ssot_change() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed:\n{proc.stdout}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_pack_golden()
    check_contract()
    check_stream_fixtures()
    check_no_docs_ssot_change()
    print("declarative_closure_g10_agent_rng_stream_red_check: ok")


if __name__ == "__main__":
    main()
