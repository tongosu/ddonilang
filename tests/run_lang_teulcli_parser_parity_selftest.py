#!/usr/bin/env python
from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path


README_PATH = Path("tests/lang_teulcli_parser_parity/README.md")
LANG_SURFACE_FAMILY_README = Path("tests/lang_surface_family/README.md")
LANG_LIB = Path("lang/src/lib.rs")
TOOL_MAIN = Path("tool/src/main.rs")
TEUL_CLI_CANON = Path("tools/teul-cli/src/cli/canon.rs")
TEUL_CANON = Path("tools/teul-cli/src/canon.rs")

README_SNIPPETS = (
    "## Stable Contract",
    "`tool/src/main.rs`",
    "`tools/teul-cli/src/cli/canon.rs`",
    "`tools/teul-cli/src/canon.rs`",
    "`채비:`",
    "`채비 {`",
    "`보개장면`",
    "`보개마당`",
    "`W_BOGAE_MADANG_ALIAS_DEPRECATED`",
    "`구성`",
    "`짜임`",
    "`W_JJAIM_ALIAS_DEPRECATED`",
    "`조건 { ... }`",
    "`매김`",
    "`tool canon`",
    "`teul-cli canon`",
    "`python tests/run_lang_teulcli_parser_parity_selftest.py`",
    "`python tests/run_lang_surface_family_selftest.py`",
    "`python tests/run_lang_surface_family_contract_selftest.py`",
    "`python tests/run_ci_sanity_gate.py --profile core_lang`",
)

LANG_SURFACE_POINTERS = (
    "`tests/lang_teulcli_parser_parity/README.md`",
    "`python tests/run_lang_teulcli_parser_parity_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[lang-teulcli-parser-parity-selftest] fail: {message}")
    return 1


def ensure_snippets(path: Path, snippets: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for snippet in snippets:
        if snippet not in text:
            raise ValueError(f"missing snippet in {path}: {snippet}")


def ensure_exists(path: Path) -> None:
    if not path.exists():
        raise ValueError(f"missing file: {path}")


def ensure_lang_surface_pointer() -> None:
    text = LANG_SURFACE_FAMILY_README.read_text(encoding="utf-8")
    for pointer in LANG_SURFACE_POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {LANG_SURFACE_FAMILY_README}: {pointer}")


def resolve_bin(root: Path, stem: str) -> Path | None:
    suffix = ".exe" if __import__("os").name == "nt" else ""
    candidates = [
        root / "target" / "debug" / f"{stem}{suffix}",
        root / "target" / "release" / f"{stem}{suffix}",
        Path(f"I:/home/urihanl/ddn/codex/target/debug/{stem}.exe"),
        Path(f"I:/home/urihanl/ddn/codex/target/release/{stem}.exe"),
    ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    which = shutil.which(stem)
    return Path(which) if which else None


def ensure_bin(root: Path, stem: str, manifest_path: str) -> Path:
    found = resolve_bin(root, stem)
    if found is not None:
        return found
    proc = subprocess.run(
        ["cargo", "build", "--quiet", "--manifest-path", manifest_path],
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or "cargo build failed"
        raise ValueError(f"build failed for {stem}: {detail}")
    found = resolve_bin(root, stem)
    if found is None:
        raise ValueError(f"missing binary after build: {stem}")
    return found


def run_canon(root: Path, binary: Path, input_path: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(binary), "canon", str(input_path), "--emit", "ddn"],
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def payload(proc: subprocess.CompletedProcess[str]) -> str:
    return f"{proc.stdout}\n{proc.stderr}"


def frontdoor_ok(proc: subprocess.CompletedProcess[str]) -> bool:
    body = payload(proc)
    if proc.returncode != 0:
        return False
    fail_markers = (
        "파싱 실패:",
        "CANON_CHECK_FAIL",
        "E_CLI_READ",
        "E_CLI_WRITE",
    )
    return not any(marker in body for marker in fail_markers)


def require_contains(text: str, snippets: tuple[str, ...], label: str) -> None:
    for snippet in snippets:
        if snippet not in text:
            raise ValueError(f"{label}: missing snippet: {snippet}")


def require_not_contains(text: str, snippets: tuple[str, ...], label: str) -> None:
    for snippet in snippets:
        if snippet in text:
            raise ValueError(f"{label}: unexpected snippet: {snippet}")


def run_cases(root: Path) -> int:
    tool_bin = ensure_bin(root, "ddonirang-tool", "tool/Cargo.toml")
    teul_bin = ensure_bin(root, "teul-cli", "tools/teul-cli/Cargo.toml")
    tmp_dir = root / "build" / "tmp" / "lang_teulcli_parser_parity"
    tmp_dir.mkdir(parents=True, exist_ok=True)

    cases = (
        {
            "name": "block_header_colon_deprecated",
            "source": "채비: {\n  점수:수 <- 0.\n}.\n",
            "tool_ok": True,
            "teul_ok": True,
            "tool_contains": ("채비 {",),
            "teul_contains": ("채비 {", "W_BLOCK_HEADER_COLON_DEPRECATED"),
        },
        {
            "name": "bogae_madang_alias_parity",
            "source": '매틱:움직씨 = {\n  보개마당 {\n    #자막("테스트").\n  }.\n}\n',
            "tool_ok": True,
            "teul_ok": True,
            "tool_contains": ("보개마당 {",),
            "tool_absent": ("보개장면 {",),
            "teul_contains": ("보개마당 {",),
            "teul_absent": ("보개장면 {",),
        },
        {
            "name": "jjaim_alias_warning_parity",
            "source": "매틱:움직씨 = {\n  구성 {\n    상태 { x <- 0. }.\n  }.\n}\n",
            "tool_ok": True,
            "teul_ok": True,
            "tool_contains": ("짜임 {",),
            "tool_absent": ("구성 {",),
            "teul_contains": ("짜임 {",),
            "teul_absent": ("구성 {",),
        },
        {
            "name": "decl_maegim_alias_parity",
            "source": "채비 {\n  g:수 = (9.8) 조건 {\n    범위: 1..20.\n  }.\n}.\n",
            "tool_ok": True,
            "teul_ok": True,
            "tool_contains": ("매김 {",),
            "tool_absent": ("조건 {",),
            "teul_contains": ("매김 {",),
            "teul_absent": ("조건 {",),
        },
        {
            "name": "hook_alias_normalization_parity",
            "source": "(처음)할때 {\n  t <- 0.\n}.\n(매틱)마다 {\n  t <- t + 1.\n}.\n",
            "tool_ok": True,
            "teul_ok": True,
            "tool_contains": ("(시작)할때 {", "(매마디)마다 {"),
            "tool_absent": ("(처음)할때 {", "(매틱)마다 {"),
            "teul_contains": ("(시작)할때 {", "(매마디)마다 {"),
            "teul_absent": ("(처음)할때 {", "(매틱)마다 {"),
        },
        {
            "name": "hook_every_n_madi_parity",
            "source": "(3마디)마다 {\n  t <- t + 1.\n}.\n",
            "tool_ok": True,
            "teul_ok": True,
            "tool_contains": ("(3마디)마다 {",),
            "teul_contains": ("(3마디)마다 {",),
        },
        {
            "name": "hook_every_n_colon_forbidden_parity",
            "source": "(3마디)마다: {\n  t <- t + 1.\n}.\n",
            "tool_ok": True,
            "teul_ok": True,
            "tool_contains": ("(3마디)마다:", "W_CANON_PASSTHROUGH"),
            "teul_contains": ("(3마디)마다:", "W_CANON_PASSTHROUGH"),
        },
    )

    for case in cases:
        input_path = tmp_dir / f"{case['name']}.ddn"
        input_path.write_text(case["source"], encoding="utf-8")
        tool_proc = run_canon(root, tool_bin, input_path)
        teul_proc = run_canon(root, teul_bin, input_path)

        tool_payload = payload(tool_proc)
        teul_payload = payload(teul_proc)

        if frontdoor_ok(tool_proc) != bool(case["tool_ok"]):
            raise ValueError(
                f"{case['name']}: tool outcome mismatch "
                f"(rc={tool_proc.returncode} stdout={tool_proc.stdout.strip()} stderr={tool_proc.stderr.strip()})"
            )
        if frontdoor_ok(teul_proc) != bool(case["teul_ok"]):
            raise ValueError(
                f"{case['name']}: teul-cli outcome mismatch "
                f"(rc={teul_proc.returncode} stdout={teul_proc.stdout.strip()} stderr={teul_proc.stderr.strip()})"
            )

        require_contains(tool_payload, tuple(case.get("tool_contains", ())), f"{case['name']}:tool")
        require_contains(teul_payload, tuple(case.get("teul_contains", ())), f"{case['name']}:teul")
        require_not_contains(tool_payload, tuple(case.get("tool_absent", ())), f"{case['name']}:tool")
        require_not_contains(teul_payload, tuple(case.get("teul_absent", ())), f"{case['name']}:teul")

    return len(cases)


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    try:
        ensure_exists(README_PATH)
        ensure_exists(LANG_SURFACE_FAMILY_README)
        ensure_exists(LANG_LIB)
        ensure_exists(TOOL_MAIN)
        ensure_exists(TEUL_CLI_CANON)
        ensure_exists(TEUL_CANON)
        ensure_snippets(README_PATH, README_SNIPPETS)
        ensure_lang_surface_pointer()
        case_count = run_cases(root)
    except ValueError as exc:
        return fail(str(exc))

    print(f"[lang-teulcli-parser-parity-selftest] ok cases={case_count} parity={case_count} drift=0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
