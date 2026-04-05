#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/seamgrim_wasm_web_smoke_contract/README.md")
PACK_CONTRACT_README = Path("pack/seamgrim_wasm_web_smoke_contract_v1/README.md")
WASM_PACK_README = Path("pack/seamgrim_wasm_v0_smoke/README.md")
INTERACTIVE_PACK_README = Path("pack/seamgrim_interactive_event_smoke_v1/README.md")
TEMP_PACK_README = Path("pack/seamgrim_temp_lesson_smoke_v1/README.md")
MOYANG_PACK_README = Path("pack/seamgrim_moyang_render_smoke_v1/README.md")

WASM_TRACE = Path("pack/seamgrim_wasm_v0_smoke/expected/state_hash_trace.detjson")
INTERACTIVE = Path("pack/seamgrim_interactive_event_smoke_v1/expected/interactive_event.detjson")
TEMP = Path("pack/seamgrim_temp_lesson_smoke_v1/expected/temp_lesson.detjson")
MOYANG = Path("pack/seamgrim_moyang_render_smoke_v1/expected/moyang_render.detjson")

README_SNIPPETS = (
    "## Stable Contract",
    "`pack/seamgrim_wasm_v0_smoke/README.md`",
    "`pack/seamgrim_interactive_event_smoke_v1/README.md`",
    "`pack/seamgrim_temp_lesson_smoke_v1/README.md`",
    "`pack/seamgrim_moyang_render_smoke_v1/README.md`",
    "`pack/seamgrim_wasm_v0_smoke/expected/state_hash_trace.detjson`",
    "`pack/seamgrim_interactive_event_smoke_v1/expected/interactive_event.detjson`",
    "`pack/seamgrim_temp_lesson_smoke_v1/expected/temp_lesson.detjson`",
    "`pack/seamgrim_moyang_render_smoke_v1/expected/moyang_render.detjson`",
    "`pack/seamgrim_wasm_web_smoke_contract_v1/expected/seamgrim_wasm_web_smoke_contract.stdout.txt`",
    "`pack/seamgrim_wasm_web_smoke_contract_v1/expected/seamgrim_wasm_web_real_smoke.stdout.txt`",
    "`python tests/run_seamgrim_wasm_web_smoke_contract_selftest.py`",
    "`python tests/run_seamgrim_wasm_web_smoke_contract_pack_check.py`",
    "`python tests/run_seamgrim_wasm_smoke.py seamgrim_wasm_v0_smoke seamgrim_interactive_event_smoke_v1 seamgrim_temp_lesson_smoke_v1 seamgrim_moyang_render_smoke_v1 --skip-ui-common --skip-ui-pendulum --skip-wrapper --skip-vm-runtime --skip-space2d-source-gate`",
    "wasm bridge state_hash trace + web interactive event + web temperature table + web moyang render",
)
PACK_README_SNIPPETS = (
    "`expected/seamgrim_wasm_web_smoke_contract.stdout.txt`",
    "`expected/seamgrim_wasm_web_real_smoke.stdout.txt`",
    "`python tests/run_seamgrim_wasm_web_smoke_contract_pack_check.py`",
)
POINTERS = (
    "`tests/seamgrim_wasm_web_smoke_contract/README.md`",
    "`python tests/run_seamgrim_wasm_web_smoke_contract_selftest.py`",
)
INTERACTIVE_STEP_IDS = (
    "baseline",
    "right_down",
    "right_hold",
    "right_up",
    "left_down",
    "left_up",
    "up_down",
    "up_hold",
    "up_up",
    "disabled_right",
    "reenabled_baseline",
)


def fail(message: str) -> int:
    print(f"[seamgrim-wasm-web-smoke-contract-selftest] fail: {message}")
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def ensure_snippets(path: Path, snippets: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for snippet in snippets:
        if snippet not in text:
            raise ValueError(f"missing snippet in {path}: {snippet}")


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def validate_wasm_trace() -> None:
    doc = load_json(WASM_TRACE)
    if doc.get("algo") != "blake3":
        raise ValueError("wasm_trace.algo != blake3")
    rows = doc.get("rows")
    if not isinstance(rows, list) or len(rows) != 2:
        raise ValueError("wasm_trace.rows length != 2")
    madis = [row.get("madi") for row in rows]
    if madis != [0, 1]:
        raise ValueError(f"wasm_trace.madi order mismatch: {madis}")
    hashes = [row.get("state_hash", "") for row in rows]
    if not all(isinstance(h, str) and h.startswith("blake3:") for h in hashes):
        raise ValueError("wasm_trace.state_hash must start with blake3:")
    if hashes[0] == hashes[1]:
        raise ValueError("wasm_trace state_hash should change between madi 0 and 1")


def validate_interactive() -> None:
    doc = load_json(INTERACTIVE)
    if doc.get("schema") != "seamgrim.web.interactive_event_smoke.v1":
        raise ValueError("interactive.schema mismatch")
    if doc.get("parse_diag_codes") != []:
        raise ValueError("interactive.parse_diag_codes must be empty")
    for key in (
        "right_moves_view",
        "hold_keeps_right_pressed",
        "release_clears_right",
        "left_moves_view",
        "spin_requires_pulse",
        "disabled_input_blocks_event",
        "rendered_all",
    ):
        if doc.get(key) is not True:
            raise ValueError(f"interactive.{key} must be true")

    steps = doc.get("steps")
    if not isinstance(steps, list) or len(steps) != len(INTERACTIVE_STEP_IDS):
        raise ValueError("interactive.steps length mismatch")
    step_by_id = {step.get("id"): step for step in steps}
    if tuple(step_by_id.keys()) != INTERACTIVE_STEP_IDS:
        raise ValueError("interactive step ids/order mismatch")

    right_down = step_by_id["right_down"]
    if right_down["shape"]["x"] != 1 or right_down["flags"]["right"] is not True:
        raise ValueError("interactive.right_down right move contract mismatch")
    if right_down["prevented"]["keydown"] is not True:
        raise ValueError("interactive.right_down keydown should be prevented")

    up_down = step_by_id["up_down"]
    up_hold = step_by_id["up_hold"]
    if up_down["flags"]["spin"] is not True:
        raise ValueError("interactive.up_down spin should be true")
    if up_hold["flags"]["spin"] is not False:
        raise ValueError("interactive.up_hold spin should reset to false")

    disabled = step_by_id["disabled_right"]
    if disabled["input_enabled"] is not False or disabled["prevented"]["keydown"] is not True:
        raise ValueError("interactive.disabled_right input gating mismatch")


def validate_temp() -> None:
    doc = load_json(TEMP)
    if doc.get("schema") != "seamgrim.web.temp_lesson_smoke.v1":
        raise ValueError("temp.schema mismatch")
    if doc.get("prepared_body_has_temp_formats") is not True:
        raise ValueError("temp.prepared_body_has_temp_formats must be true")
    if doc.get("parse_diag_codes") != []:
        raise ValueError("temp.parse_diag_codes must be empty")
    if doc.get("rendered_table") is not True:
        raise ValueError("temp.rendered_table must be true")
    if doc.get("rendered_contains_celsius") is not True or doc.get("rendered_contains_fahrenheit") is not True:
        raise ValueError("temp rendered temperature markers missing")

    first_row = doc.get("first_row", {})
    last_row = doc.get("last_row", {})
    if first_row.get("t") != 0 or first_row.get("celsius") != "80.0@C" or first_row.get("fahrenheit") != "176.0@F":
        raise ValueError("temp.first_row contract mismatch")
    if last_row.get("t") != 2 or last_row.get("celsius") != "72.0@C" or last_row.get("fahrenheit") != "161.6@F":
        raise ValueError("temp.last_row contract mismatch")


def validate_moyang() -> None:
    doc = load_json(MOYANG)
    if doc.get("schema") != "seamgrim.web.moyang_render_smoke.v1":
        raise ValueError("moyang.schema mismatch")
    if doc.get("tick_count") != 3:
        raise ValueError("moyang.tick_count must be 3")
    for key in ("state_hash_equal_ab", "state_hash_equal_ac", "radius_diff_ab", "fill_diff_ac", "rendered_all"):
        if doc.get(key) is not True:
            raise ValueError(f"moyang.{key} must be true")
    if doc.get("arcs_all") != [1, 1, 1]:
        raise ValueError("moyang.arcs_all mismatch")

    cases = doc.get("cases")
    if not isinstance(cases, list) or len(cases) != 3:
        raise ValueError("moyang.cases length mismatch")
    ids = [case.get("id") for case in cases]
    if ids != ["a", "b", "c"]:
        raise ValueError(f"moyang.case ids mismatch: {ids}")
    state_hashes = {case.get("state_hash") for case in cases}
    if len(state_hashes) != 1:
        raise ValueError("moyang state_hash should stay equal across a/b/c")
    for case in cases:
        if case.get("space2d_source") != "observation-output-lines":
            raise ValueError("moyang.space2d_source mismatch")
        if case.get("rendered") is not True:
            raise ValueError("moyang case rendered should be true")


def main() -> int:
    try:
        ensure_snippets(README_PATH, README_SNIPPETS)
        ensure_snippets(PACK_CONTRACT_README, PACK_README_SNIPPETS)
        ensure_pointers(WASM_PACK_README)
        ensure_pointers(INTERACTIVE_PACK_README)
        ensure_pointers(TEMP_PACK_README)
        ensure_pointers(MOYANG_PACK_README)
        validate_wasm_trace()
        validate_interactive()
        validate_temp()
        validate_moyang()
    except (ValueError, KeyError, TypeError) as exc:
        return fail(str(exc))

    print("[seamgrim-wasm-web-smoke-contract-selftest] ok packs=4")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
