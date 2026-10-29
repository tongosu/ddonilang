#!/usr/bin/env python
from __future__ import annotations

import csv
from collections import defaultdict
from pathlib import Path


README_PATH = Path("tests/dialect_alias_collision_contract/README.md")
LANG_SURFACE_FAMILY_README = Path("tests/lang_surface_family/README.md")
DIALECT_TSV = Path("docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v8_20260215.tsv")
DIALECT_AUDIT = Path("docs/context/notes/dialect/DIALECT_KEYWORD_GAP_AUDIT_20260301.md")
DIALECT_RS = Path("lang/src/dialect.rs")

README_SNIPPETS = (
    "## Stable Contract",
    "`docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v8_20260215.tsv`",
    "`docs/context/notes/dialect/DIALECT_KEYWORD_GAP_AUDIT_20260301.md`",
    "`lang/src/dialect.rs`",
    "`샘 -> 샘입력`",
    "`입력 -> 입력 블록`",
    "`ko` scope",
    "non-`ko` same-dialect collision inventory",
    "`구성 -> 짜임`",
    "`효과/바깥 -> 너머`",
    "`소식/사건/알람 -> 알림`",
    "`보개장면 -> 보개마당`",
    "`올때 -> 오면`",
    "`python tests/run_dialect_alias_collision_contract_selftest.py`",
    "`python tests/run_dialect_alias_collision_inventory_report_selftest.py`",
    "`python tests/run_lang_surface_family_selftest.py`",
    "`python tests/run_ci_sanity_gate.py --profile core_lang`",
    "`ddn.dialect_alias_collision_inventory.v1`",
)

LANG_SURFACE_POINTERS = (
    "`tests/dialect_alias_collision_contract/README.md`",
    "`python tests/run_dialect_alias_collision_contract_selftest.py`",
)

TAG_COLUMNS_EXCLUDED = {"kind", "ko_canon", "ko_alias", "notes", "sym3"}
PINNED_ALIASES = {
    "샘입력": "샘",
    "구성": "짜임",
    "효과": "너머",
    "바깥": "너머",
    "소식": "알림",
    "사건": "알림",
    "알람": "알림",
    "보개장면": "보개마당",
    "올때": "오면",
}
REMOVED_LEGACY_KEYWORD_ALIASES = {
    ("값함수", "셈씨"),
    ("일묶음씨", "갈래씨"),
    ("유지하고", "늘지켜보고"),
    ("검사할때", "늘지켜보고"),
    ("설정보개", "보개"),
    ("보임", "보개"),
}
KNOWN_NON_KO_COLLISIONS = {
    "ay": {
        "janiwa": ("아님", "없음"),
        "mantawi": ("샘", "입력"),
        "mayachawi": ("이음씨", "짝맞춤"),
        "mayampi": ("하나하나", "함께"),
        "pacha": ("누리", "마디"),
        "pachanxa": ("대해", "동안"),
        "sapa": ("마다", "바탕"),
        "ukhamaxa": ("이면", "일때"),
        "uñjaña": ("늘지켜보고", "톺아보기"),
        "yatiyawi": ("알림", "알림씨"),
    },
    "eu": {
        "bakoitza": ("마다", "저마다"),
        "sarrera": ("샘", "입력"),
    },
    "kn": {
        "prati": ("대해", "마다"),
        "prayatnisu": ("해보고", "해보기"),
    },
    "qu": {
        "kutichiy": ("되돌림", "되살리기"),
        "mana": ("아님", "없음"),
        "ruraykuy": ("해보고", "해보기"),
        "tikray": ("그릇채비", "바뀔때"),
        "tukuy": ("끝", "모두"),
        "willay": ("알림", "알림씨"),
        "yaykuy": ("샘", "입력"),
    },
    "ta": {
        "illai": ("아님", "없음"),
        "muRaiyuDu": ("해보고", "해보기"),
        "paRRi": ("대해", "따라"),
        "paariththu": ("살핌말", "톺아보기"),
        "uLLiidu": ("샘", "입력"),
    },
    "te": {
        "aapu": ("막기", "멈추기"),
        "ayite": ("이면", "일때"),
        "mariyu": ("그리고", "되풀이"),
    },
    "tr": {
        "girdi": ("샘", "입력"),
    },
}


def fail(message: str) -> int:
    print(f"[dialect-alias-collision-contract-selftest] fail: {message}")
    return 1


def ensure_exists(path: Path) -> None:
    if not path.exists():
        raise ValueError(f"missing file: {path}")


def ensure_snippets(path: Path, snippets: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for snippet in snippets:
        if snippet not in text:
            raise ValueError(f"missing snippet in {path}: {snippet}")


def ensure_lang_surface_pointer() -> None:
    text = LANG_SURFACE_FAMILY_README.read_text(encoding="utf-8")
    for pointer in LANG_SURFACE_POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {LANG_SURFACE_FAMILY_README}: {pointer}")


def is_ident_start(ch: str) -> bool:
    return ch == "_" or ch.isalpha()


def is_ident_continue(ch: str) -> bool:
    return is_ident_start(ch) or ch.isnumeric()


def is_ident_like(text: str) -> bool:
    if not text:
        return False
    if not is_ident_start(text[0]):
        return False
    return all(is_ident_continue(ch) for ch in text[1:])


def normalize_keyword_token(token: str) -> str | None:
    text = token.strip()
    if not text:
        return None
    if text.endswith(":"):
        text = text[:-1]
    if not text or "#" in text or " " in text or "\t" in text:
        return None
    if not is_ident_like(text):
        return None
    return text


def split_keyword_alias_tokens(raw: str) -> list[str]:
    tokens: list[str] = []
    for value in raw.split("/"):
        for subvalue in value.split("|"):
            for item in subvalue.split(","):
                for piece in item.split(";"):
                    normalized = normalize_keyword_token(piece)
                    if normalized is not None:
                        tokens.append(normalized)
    return tokens


def load_rows() -> list[dict[str, str]]:
    with DIALECT_TSV.open("r", encoding="utf-8-sig", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        return [{key: (value or "").strip() for key, value in row.items()} for row in reader]


def build_scoped_token_maps(
    rows: list[dict[str, str]]
) -> tuple[dict[str, dict[str, set[str]]], dict[str, dict[str, str]]]:
    scoped_maps: dict[str, dict[str, set[str]]] = defaultdict(lambda: defaultdict(set))
    by_canon: dict[str, dict[str, str]] = {}
    for row in rows:
        kind = row.get("kind", "")
        if kind == "josa":
            continue
        canon = normalize_keyword_token(row.get("ko_canon", ""))
        if canon is None:
            continue
        by_canon[canon] = row
        scoped_maps["ko"][canon].add(canon)
        for alias in split_keyword_alias_tokens(row.get("ko_alias", "")):
            if (alias, canon) in REMOVED_LEGACY_KEYWORD_ALIASES:
                continue
            scoped_maps["ko"][alias].add(canon)
        for column, raw in row.items():
            if column in TAG_COLUMNS_EXCLUDED:
                continue
            token = normalize_keyword_token(raw)
            if token is not None:
                scoped_maps[column][token].add(canon)
    return scoped_maps, by_canon


def ensure_no_collisions(scoped_maps: dict[str, dict[str, set[str]]]) -> None:
    ko_collisions = {
        token: tuple(sorted(canon_set))
        for token, canon_set in scoped_maps.get("ko", {}).items()
        if len(canon_set) > 1
    }
    if ko_collisions:
        token, canon_list = sorted(ko_collisions.items())[0]
        raise ValueError(f"ko alias collision detected: token={token} -> {','.join(canon_list)}")

    actual_non_ko = {
        scope: {
            token: tuple(sorted(canon_set))
            for token, canon_set in token_map.items()
            if len(canon_set) > 1
        }
        for scope, token_map in scoped_maps.items()
        if scope != "ko"
    }
    actual_non_ko = {scope: mapping for scope, mapping in actual_non_ko.items() if mapping}
    if actual_non_ko != KNOWN_NON_KO_COLLISIONS:
        raise ValueError(
            "non-ko collision inventory drift: "
            f"expected={KNOWN_NON_KO_COLLISIONS} actual={actual_non_ko}"
        )


def ensure_pinned_split(by_canon: dict[str, dict[str, str]], ko_token_map: dict[str, set[str]]) -> None:
    sam_row = by_canon.get("샘")
    if sam_row is None:
        raise ValueError("missing canonical row: 샘")
    sam_aliases = set(split_keyword_alias_tokens(sam_row.get("ko_alias", "")))
    if "샘입력" not in sam_aliases:
        raise ValueError("샘 row is missing ko alias: 샘입력")
    if "입력" in sam_aliases:
        raise ValueError("샘 row still contains conflicting ko alias: 입력")
    if "입력" not in by_canon:
        raise ValueError("missing canonical row: 입력")
    if ko_token_map.get("샘입력") != {"샘"}:
        raise ValueError(f"unexpected token mapping for 샘입력: {sorted(ko_token_map.get('샘입력', set()))}")
    if ko_token_map.get("입력") != {"입력"}:
        raise ValueError(f"unexpected token mapping for 입력: {sorted(ko_token_map.get('입력', set()))}")


def ensure_pinned_aliases(ko_token_map: dict[str, set[str]]) -> None:
    for alias, canon in PINNED_ALIASES.items():
        mapped = ko_token_map.get(alias)
        if mapped != {canon}:
            raise ValueError(f"unexpected token mapping for {alias}: {sorted(mapped or set())}")


def main() -> int:
    try:
        ensure_exists(README_PATH)
        ensure_exists(LANG_SURFACE_FAMILY_README)
        ensure_exists(DIALECT_TSV)
        ensure_exists(DIALECT_AUDIT)
        ensure_exists(DIALECT_RS)
        ensure_snippets(README_PATH, README_SNIPPETS)
        ensure_lang_surface_pointer()
        rows = load_rows()
        scoped_maps, by_canon = build_scoped_token_maps(rows)
        ensure_no_collisions(scoped_maps)
        ko_token_map = scoped_maps.get("ko", {})
        ensure_pinned_split(by_canon, ko_token_map)
        ensure_pinned_aliases(ko_token_map)
    except ValueError as exc:
        return fail(str(exc))

    print(
        "[dialect-alias-collision-contract-selftest] ok "
        f"canon={len(by_canon)} ko_tokens={len(ko_token_map)} scopes={len(scoped_maps)} pinned={len(PINNED_ALIASES) + 2}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
