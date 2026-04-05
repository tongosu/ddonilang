#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/bogae_alias_viewer_family/README.md")
ALIAS_FAMILY_README = Path("tests/bogae_alias_family/README.md")
WEB_VIEWER_README = Path("pack/bogae_web_viewer_v1/README.md")
WEB_VIEWER_INPUT = Path("pack/bogae_web_viewer_v1/input.ddn")
WEB_VIEWER_GOLDEN = Path("pack/bogae_web_viewer_v1/golden.jsonl")
WEB_OUT_README = Path("pack/bogae_web_out_determinism/README.md")
WEB_OUT_INPUT = Path("pack/bogae_web_out_determinism/input.ddn")
WEB_OUT_GOLDEN = Path("pack/bogae_web_out_determinism/golden/webout_001_manifest_hash.test.json")
W13_VIEW_PIPELINE = Path("tools/teul-cli/tests/golden/W13/W13_G02_view_detbin_pipeline/main.ddn")
SANITY_GATE = Path("tests/run_ci_sanity_gate.py")

POINTERS = (
    "`tests/bogae_alias_viewer_family/README.md`",
    "`python tests/run_bogae_alias_viewer_family_selftest.py`",
)

README_SNIPPETS = (
    "## Stable Contract",
    "`tests/bogae_alias_family/README.md`",
    "`pack/bogae_web_viewer_v1/README.md`",
    "`tools/teul-cli/tests/golden/W13/W13_G02_view_detbin_pipeline/main.ddn`",
    "`pack/bogae_web_out_determinism/README.md`",
    "`pack/bogae_web_out_determinism/golden/webout_001_manifest_hash.test.json`",
    "`보개_그림판_가로/세로`",
    "`보개_바탕색`",
    "`보개_그림판_목록`",
    "`결`",
    "`생김새.결`",
    "`살림.보개_*`",
    "`python tests/run_bogae_alias_viewer_family_selftest.py`",
    "`python tests/run_pack_golden.py bogae_web_viewer_v1`",
    "`python tests/run_ci_sanity_gate.py --profile core_lang`",
)


def fail(message: str) -> int:
    print(f"[bogae-alias-viewer-family-selftest] fail: {message}")
    return 1


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


def ensure_contains(path: Path, snippets: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for snippet in snippets:
        if snippet not in text:
            raise ValueError(f"{path}: missing {snippet}")


def load_jsonl(path: Path) -> list[dict]:
    return [
        json.loads(line)
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]


def main() -> int:
    try:
        ensure_snippets(README_PATH, README_SNIPPETS)
        ensure_pointers(ALIAS_FAMILY_README)
        ensure_pointers(WEB_VIEWER_README)
        ensure_pointers(WEB_OUT_README)
        ensure_snippets(
            SANITY_GATE,
            (
                '"bogae_alias_viewer_family_selftest"',
                '[py, "tests/run_bogae_alias_viewer_family_selftest.py"]',
            ),
        )

        ensure_contains(
            WEB_VIEWER_INPUT,
            ("보개_그림판_가로", "보개_그림판_세로", "보개_바탕색", "보개_그림판_목록", "결"),
        )
        ensure_contains(W13_VIEW_PIPELINE, ("보개_그림판_가로", "bogae_bg", "생김새.결"))
        rows = load_jsonl(WEB_VIEWER_GOLDEN)
        if len(rows) != 1:
            raise ValueError(f"{WEB_VIEWER_GOLDEN}: expected 1 row")
        row = rows[0]
        cli = row.get("cli", [])
        if "--bogae" not in cli or "web" not in cli:
            raise ValueError(f"{WEB_VIEWER_GOLDEN}: missing --bogae web")
        stdout = row.get("stdout", [])
        if not any(isinstance(item, str) and item.startswith("bogae_hash=") for item in stdout):
            raise ValueError(f"{WEB_VIEWER_GOLDEN}: missing bogae_hash stdout")

        ensure_contains(WEB_OUT_INPUT, ("살림.보개_그림판_가로", "살림.보개_바탕색", "생김새.결"))
        golden = json.loads(WEB_OUT_GOLDEN.read_text(encoding="utf-8"))
        if golden.get("name") != "webout_001_manifest_hash":
            raise ValueError(f"{WEB_OUT_GOLDEN}: unexpected test name")
        observations = golden.get("det_expected", {}).get("expected_observations", [])
        if not any(isinstance(item, str) and item.startswith("manifest_hash=blake3:") for item in observations):
            raise ValueError(f"{WEB_OUT_GOLDEN}: missing manifest_hash observation")
    except ValueError as exc:
        return fail(str(exc))

    print("[bogae-alias-viewer-family-selftest] ok surfaces=3")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
