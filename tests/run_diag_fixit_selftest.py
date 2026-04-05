#!/usr/bin/env python
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
PACKS = ("diag_fixit_coverage_v1", "diag_fixit_json_schema_v1")
README_SNIPPETS = {
    Path("pack/diag_fixit_coverage_v1/README.md"): (
        "PLN-20260322-DIAG-FIXIT-COVERAGE-01",
        "`teul-cli canon --fixits-json`",
        "`python tests/run_pack_golden.py --manifest-path tools/teul-cli/Cargo.toml diag_fixit_coverage_v1`",
    ),
    Path("pack/diag_fixit_json_schema_v1/README.md"): (
        "`[]`, `replace`, `insert`, `delete`",
        "`python tests/run_pack_golden.py --manifest-path tools/teul-cli/Cargo.toml diag_fixit_json_schema_v1`",
    ),
}
PARSE_FIXIT_PROBES = (
    ("unexpected_token", "E_PARSE_UNEXPECTED_TOKEN", "살림.x <- 1 2.\n"),
    ("expected_expr", "E_PARSE_EXPECTED_EXPR", "살림.x <- .\n"),
    ("expected_target", "E_PARSE_EXPECTED_TARGET", "(1 + 2) <- 3.\n"),
    ("expected_rparen", "E_PARSE_EXPECTED_RPAREN", "살림.x <- (1 + 2.\n"),
    ("expected_rbrace", "E_PARSE_EXPECTED_RBRACE", "너머 {\n  살림.x <- () 너머.시각.지금.\n"),
    (
        "maegim_grouped",
        "E_PARSE_MAEGIM_GROUPED_VALUE_REQUIRED",
        "채비 {\n  g:수 = 9.8 매김 {\n    간격: 1.\n  }.\n}.\n",
    ),
    (
        "block_header_forbidden",
        "E_BLOCK_HEADER_COLON_FORBIDDEN",
        "그릇채비: {\n  점수:수 <- 0.\n}.\n",
    ),
    (
        "event_alias",
        "E_EVENT_SURFACE_ALIAS_FORBIDDEN",
        "\"jump\"라는 소식이 오면 {\n  살림.y <- 1.\n}.\n",
    ),
    (
        "receive_outside_imja",
        "E_RECEIVE_OUTSIDE_IMJA",
        "기상청:움직씨 = {\n  알림을 받으면 {\n    참 보여주기.\n  }.\n}.\n",
    ),
    (
        "immediate_proof_io",
        "E_PARSE_IMMEDIATE_PROOF_IO_FORBIDDEN",
        "검사 밝히기 {\n  너머 {\n    없음.\n  }.\n}.\n",
    ),
)


def emit_text_safely(text: str, *, stream=None) -> None:
    if not text:
        return
    target = stream if stream is not None else sys.stdout
    try:
        print(text, file=target)
        return
    except UnicodeEncodeError:
        encoding = getattr(target, "encoding", None) or "utf-8"
        safe = text.encode(encoding, errors="replace").decode(encoding, errors="replace")
        print(safe, file=target)


def fail(message: str) -> int:
    emit_text_safely(f"[diag-fixit-selftest] fail: {message}")
    return 1


def ensure_exists(path: Path) -> None:
    if not path.exists():
        raise ValueError(f"missing file: {path}")


def ensure_snippets(path: Path, snippets: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for snippet in snippets:
        if snippet not in text:
            raise ValueError(f"missing snippet in {path}: {snippet}")


def resolve_teul_cli_bin(root: Path) -> Path | None:
    suffix = ".exe" if __import__("os").name == "nt" else ""
    candidates = [
        root / "target" / "debug" / f"teul-cli{suffix}",
        root / "target" / "release" / f"teul-cli{suffix}",
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"I:/home/urihanl/ddn/codex/target/release/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
    ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    which = shutil.which("teul-cli")
    return Path(which) if which else None


def ensure_teul_cli(root: Path) -> Path:
    found = resolve_teul_cli_bin(root)
    if found is not None:
        return found
    proc = subprocess.run(
        ["cargo", "build", "--quiet", "--manifest-path", "tools/teul-cli/Cargo.toml"],
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or "cargo build failed"
        raise ValueError(f"build failed for teul-cli: {detail}")
    found = resolve_teul_cli_bin(root)
    if found is None:
        raise ValueError("missing binary after build: teul-cli")
    return found


def validate_fixits_payload(payload: list[dict], *, allow_empty: bool) -> None:
    if not isinstance(payload, list):
        raise ValueError("fixits-json must be a JSON array")
    if not allow_empty and not payload:
        raise ValueError("fixits-json must contain at least one suggestion")

    previous_order: tuple[str, int, int] | None = None
    for idx, entry in enumerate(payload, 1):
        if not isinstance(entry, dict):
            raise ValueError(f"entry #{idx} must be an object")
        for key in ("code", "message", "span", "suggestion"):
            if key not in entry:
                raise ValueError(f"entry #{idx} missing key: {key}")
        span = entry["span"]
        suggestion = entry["suggestion"]
        if not isinstance(span, dict):
            raise ValueError(f"entry #{idx} span must be object")
        if not isinstance(suggestion, dict):
            raise ValueError(f"entry #{idx} suggestion must be object")
        for key in ("file", "start_line", "start_col", "end_line", "end_col"):
            if key not in span:
                raise ValueError(f"entry #{idx} span missing key: {key}")
        kind = suggestion.get("kind")
        if kind not in {"replace", "insert", "delete"}:
            raise ValueError(f"entry #{idx} suggestion.kind invalid: {kind}")
        order_key = (str(span["file"]), int(span["start_line"]), int(span["start_col"]))
        if previous_order is not None and order_key < previous_order:
            raise ValueError(f"entry #{idx} is not sorted")
        previous_order = order_key


def run_pack_goldens(root: Path) -> None:
    cmd = [
        sys.executable,
        "tests/run_pack_golden.py",
        "--manifest-path",
        "tools/teul-cli/Cargo.toml",
        *PACKS,
    ]
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or "run_pack_golden failed"
        raise ValueError(detail)


def run_probe(binary: Path, name: str, expected_code: str, source: str) -> None:
    with tempfile.TemporaryDirectory(prefix=f"diag_fixit_{name}_") as temp_dir_text:
        temp_dir = Path(temp_dir_text)
        input_path = temp_dir / "input.ddn"
        output_path = temp_dir / "fixits.json"
        input_path.write_text(source, encoding="utf-8")
        proc = subprocess.run(
            [str(binary), "canon", "input.ddn", "--fixits-json", "fixits.json"],
            cwd=temp_dir,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if not output_path.exists():
            raise ValueError(f"{name}: missing fixits.json")
        payload = json.loads(output_path.read_text(encoding="utf-8"))
        validate_fixits_payload(payload, allow_empty=False)
        codes = {str(entry.get("code", "")) for entry in payload}
        if expected_code not in codes:
            raise ValueError(f"{name}: missing expected fixit code {expected_code}; got {sorted(codes)}")
        if proc.returncode == 0 and expected_code.startswith("E_"):
            # parse-derived fixit can coexist with a success canon path. That is acceptable.
            return


def main() -> int:
    try:
        for pack in PACKS:
            ensure_exists(ROOT / "pack" / pack / "golden.jsonl")
        for relative, snippets in README_SNIPPETS.items():
            ensure_exists(ROOT / relative)
            ensure_snippets(ROOT / relative, snippets)
        run_pack_goldens(ROOT)
        binary = ensure_teul_cli(ROOT)
        for name, expected_code, source in PARSE_FIXIT_PROBES:
            run_probe(binary, name, expected_code, source)
    except ValueError as exc:
        return fail(str(exc))

    print("[diag-fixit-selftest] ok packs=2 probes=10")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
