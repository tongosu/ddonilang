#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/bogae_shape_alias_contract/README.md")
API_CATALOG_README = Path("pack/bogae_api_catalog_v1_basic/README.md")
BG_KEY_README = Path("pack/bogae_bg_key_v1/README.md")
CANVAS_PRECEDENCE_README = Path("pack/bogae_canvas_key_precedence_v1/README.md")
CANVAS_SSOT_PRECEDENCE_README = Path("pack/bogae_canvas_ssot_key_precedence_v1/README.md")
LISTKEY_README = Path("pack/bogae_drawlist_listkey_v1/README.md")
TRAIT_ALIAS_README = Path("pack/bogae_drawlist_trait_alias_v1/README.md")
TRAIT_SSOT_ALIAS_README = Path("pack/bogae_shape_trait_ssot_alias_v1/README.md")
SANITY_GATE = Path("tests/run_ci_sanity_gate.py")

PACK_POINTERS = (
    "`tests/bogae_shape_alias_contract/README.md`",
    "`python tests/run_bogae_shape_alias_contract_selftest.py`",
)

README_SNIPPETS = (
    "## Stable Contract",
    "`pack/bogae_bg_key_v1`",
    "`pack/bogae_canvas_key_precedence_v1`",
    "`pack/bogae_canvas_ssot_key_precedence_v1`",
    "`pack/bogae_drawlist_listkey_v1`",
    "`pack/bogae_drawlist_trait_alias_v1`",
    "`pack/bogae_shape_trait_ssot_alias_v1`",
    "`pack/bogae_api_catalog_v1_basic/README.md`",
    "`보개_바탕색` ≡ `bogae_bg`",
    "`보개_바탕_가로/세로` 우선, `보개_그림판_가로/세로`와 `bogae_canvas_w/h`는 하위 호환 alias",
    "`생김새.특성` 우선, `생김새.결`과 `모양.트레잇`는 하위 호환 alias",
    "`특성`",
    "`결`",
    "`트레잇`",
    "`python tests/run_bogae_shape_alias_contract_selftest.py`",
    "`python tests/run_pack_golden.py bogae_bg_key_v1 bogae_canvas_key_precedence_v1 bogae_canvas_ssot_key_precedence_v1 bogae_drawlist_listkey_v1 bogae_drawlist_trait_alias_v1 bogae_shape_trait_ssot_alias_v1`",
    "`python tests/run_ci_sanity_gate.py --profile core_lang`",
)


def fail(message: str) -> int:
    print(f"[bogae-shape-alias-contract-selftest] fail: {message}")
    return 1


def load_jsonl(path: Path) -> list[dict]:
    return [
        json.loads(line)
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]


def collect_hashes(path: Path) -> list[str]:
    rows = load_jsonl(path)
    hashes: list[str] = []
    for row in rows:
        stdout = row.get("stdout", [])
        for item in stdout:
            if isinstance(item, str) and item.startswith("bogae_hash="):
                hashes.append(item.split("=", 1)[1])
        direct = row.get("bogae_hash")
        if isinstance(direct, str) and direct:
            hashes.append(direct)
    return hashes


def ensure_snippets(path: Path, snippets: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for snippet in snippets:
        if snippet not in text:
            raise ValueError(f"missing snippet in {path}: {snippet}")


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in PACK_POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def main() -> int:
    try:
        ensure_snippets(README_PATH, README_SNIPPETS)
        ensure_pointers(API_CATALOG_README)
        ensure_pointers(BG_KEY_README)
        ensure_pointers(CANVAS_PRECEDENCE_README)
        ensure_pointers(CANVAS_SSOT_PRECEDENCE_README)
        ensure_pointers(LISTKEY_README)
        ensure_pointers(TRAIT_ALIAS_README)
        ensure_pointers(TRAIT_SSOT_ALIAS_README)
        ensure_snippets(
            SANITY_GATE,
            (
                '"bogae_shape_alias_contract_selftest"',
                '[py, "tests/run_bogae_shape_alias_contract_selftest.py"]',
            ),
        )

        bg_hashes = collect_hashes(Path("pack/bogae_bg_key_v1/golden.jsonl"))
        canvas_hashes = collect_hashes(Path("pack/bogae_canvas_key_precedence_v1/golden.jsonl"))
        canvas_ssot_hashes = collect_hashes(Path("pack/bogae_canvas_ssot_key_precedence_v1/golden.jsonl"))
        list_hashes = collect_hashes(Path("pack/bogae_drawlist_listkey_v1/golden.jsonl"))
        trait_hashes = collect_hashes(Path("pack/bogae_drawlist_trait_alias_v1/golden.jsonl"))
        trait_ssot_hashes = collect_hashes(Path("pack/bogae_shape_trait_ssot_alias_v1/golden.jsonl"))

        if len(set(bg_hashes)) != 1:
            raise ValueError("bogae_bg_key_v1 hash set is not stable")
        if len(set(canvas_hashes)) != 1:
            raise ValueError("bogae_canvas_key_precedence_v1 hash set is not stable")
        if len(set(canvas_ssot_hashes)) != 1:
            raise ValueError("bogae_canvas_ssot_key_precedence_v1 hash set is not stable")
        if len(set(list_hashes)) != 1:
            raise ValueError("bogae_drawlist_listkey_v1 hash set is not stable")
        if len(set(trait_hashes)) != 1:
            raise ValueError("bogae_drawlist_trait_alias_v1 hash set is not stable")
        if len(set(trait_ssot_hashes)) != 1:
            raise ValueError("bogae_shape_trait_ssot_alias_v1 hash set is not stable")

        if bg_hashes[0] != canvas_hashes[0]:
            raise ValueError(
                f"canvas precedence hash mismatch: {bg_hashes[0]} != {canvas_hashes[0]}"
            )
        if bg_hashes[0] != canvas_ssot_hashes[0]:
            raise ValueError(
                f"ssot canvas precedence hash mismatch: {bg_hashes[0]} != {canvas_ssot_hashes[0]}"
            )
        if list_hashes[0] != trait_hashes[0]:
            raise ValueError(
                f"trait alias hash mismatch: {list_hashes[0]} != {trait_hashes[0]}"
            )
        if list_hashes[0] != trait_ssot_hashes[0]:
            raise ValueError(
                f"ssot trait alias hash mismatch: {list_hashes[0]} != {trait_ssot_hashes[0]}"
            )
    except ValueError as exc:
        return fail(str(exc))

    print("[bogae-shape-alias-contract-selftest] ok pairs=4")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
