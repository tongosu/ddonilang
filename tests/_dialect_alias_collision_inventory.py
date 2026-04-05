from __future__ import annotations

import csv
from collections import defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
DIALECT_TSV = ROOT / "docs" / "context" / "notes" / "dialect" / "DDONIRANG_dialect_keywords_full_v8_20260215.tsv"

TAG_COLUMNS_EXCLUDED = {"kind", "ko_canon", "ko_alias", "notes", "sym3"}
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


def is_ident_start(ch: str) -> bool:
    return ch == "_" or ch.isalpha()


def is_ident_continue(ch: str) -> bool:
    return is_ident_start(ch) or ch.isnumeric()


def is_ident_like(text: str) -> bool:
    return bool(text) and is_ident_start(text[0]) and all(is_ident_continue(ch) for ch in text[1:])


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


def build_scoped_collisions(tsv_path: Path | None = None) -> dict[str, dict[str, tuple[str, ...]]]:
    path = tsv_path or DIALECT_TSV
    with path.open("r", encoding="utf-8-sig", newline="") as handle:
        rows = [{key: (value or "").strip() for key, value in row.items()} for row in csv.DictReader(handle, delimiter="\t")]

    scoped_maps: dict[str, dict[str, set[str]]] = defaultdict(lambda: defaultdict(set))
    for row in rows:
        if row.get("kind", "") == "josa":
            continue
        canon = normalize_keyword_token(row.get("ko_canon", ""))
        if canon is None:
            continue
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

    collisions: dict[str, dict[str, tuple[str, ...]]] = {}
    for scope, token_map in scoped_maps.items():
        scoped_collision = {
            token: tuple(sorted(values))
            for token, values in token_map.items()
            if len(values) > 1
        }
        if scoped_collision:
            collisions[scope] = dict(sorted(scoped_collision.items()))
    return dict(sorted(collisions.items()))


def build_inventory_report(tsv_path: Path | None = None) -> dict[str, object]:
    collisions = build_scoped_collisions(tsv_path)
    ko_collisions = collisions.get("ko", {})
    non_ko = {scope: mapping for scope, mapping in collisions.items() if scope != "ko"}
    non_ko_collision_count = sum(len(mapping) for mapping in non_ko.values())
    return {
        "schema": "ddn.dialect_alias_collision_inventory.v1",
        "ko_collision_count": len(ko_collisions),
        "non_ko_scope_count": len(non_ko),
        "non_ko_collision_count": non_ko_collision_count,
        "known_inventory_match": non_ko == KNOWN_NON_KO_COLLISIONS,
        "non_ko_scopes": [
            {
                "scope": scope,
                "collision_count": len(mapping),
                "collisions": [
                    {
                        "token": token,
                        "canonical_keywords": list(mapping[token]),
                    }
                    for token in sorted(mapping)
                ],
            }
            for scope, mapping in sorted(non_ko.items())
        ],
    }
